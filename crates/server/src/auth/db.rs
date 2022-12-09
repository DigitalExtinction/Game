use anyhow::{Context, Result};
use log::info;
use sqlx::{query, sqlite::SqliteRow, Pool, Row, Sqlite};
use thiserror::Error;

use super::{
    model::{User, UserWithPassword, UsernameAndPassword},
    passwd::{DbPassword, MAX_PASS_HASH_LEN, MAX_PASS_SALT_LEN},
};
use crate::{auth::model::MAX_USERNAME_LEN, db::SQLITE_CONSTRAINT_PRIMARYKEY, db_error};

#[derive(Clone)]
pub struct Users {
    pool: &'static Pool<Sqlite>,
}

impl Users {
    /// This method sets up the database by creating required tables if they do
    /// not already exist.
    pub(super) async fn init(pool: &'static Pool<Sqlite>) -> Result<Self> {
        let init_query = format!(
            include_str!("init.sql"),
            username_len = MAX_USERNAME_LEN,
            pass_hash_len = MAX_PASS_HASH_LEN,
            pass_salt_len = MAX_PASS_SALT_LEN,
        );

        info!("Initializing users...");
        query(&init_query)
            .execute(pool)
            .await
            .context("DB initialization failed")?;
        Ok(Self { pool })
    }

    /// This method registers a new user by inserting a record to the database
    /// or returns an error if that is not possible (e.g. the username is
    /// already taken).
    pub(super) async fn register(&self, user: &UserWithPassword) -> Result<(), RegistrationError> {
        info!("Registering user {}...", user.user().username());

        let password = DbPassword::generate(user.password()).map_err(RegistrationError::Other)?;
        let result = query("INSERT INTO users (username, pass_hash, pass_salt) VALUES(?, ?, ?);")
            .bind(user.user().username())
            .bind(password.b64_encode_pwd_hash()?)
            .bind(password.salt_str())
            .execute(self.pool)
            .await;

        db_error!(
            result,
            RegistrationError::UsernameTaken,
            SQLITE_CONSTRAINT_PRIMARYKEY
        );
        result.map_err(RegistrationError::Database)?;
        Ok(())
    }

    /// Validates username and password of the user. Returns true if the user
    /// exists and the password is correct.
    pub(super) async fn login(&self, user: &UsernameAndPassword) -> Result<bool> {
        info!("Logging in user {}...", user.username());

        let row = query("SELECT pass_hash, pass_salt FROM users WHERE username = ?;")
            .bind(user.username())
            .fetch_optional(self.pool)
            .await?;
        let Some(row) = row else { return Ok(false) };
        Ok(DbPassword::try_from(row)?.check(user.password()))
    }
}

#[derive(Error, Debug)]
pub(super) enum RegistrationError {
    #[error("Username is already taken")]
    UsernameTaken,
    #[error("A database error encountered")]
    Database(#[source] sqlx::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl TryFrom<SqliteRow> for DbPassword {
    type Error = anyhow::Error;

    fn try_from(row: SqliteRow) -> Result<Self> {
        let hashed: &str = row
            .try_get("pass_hash")
            .context("Failed to retrieve password hash from the DB")?;
        let salt: &str = row
            .try_get("pass_salt")
            .context("Failed to retrieve password salt from the DB")?;
        Self::try_from((hashed, salt))
    }
}

impl TryFrom<SqliteRow> for User {
    type Error = anyhow::Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        let username: String = row.try_get("username")?;
        Ok(Self::new(username))
    }
}
