use anyhow::{Context, Result};
use futures_util::TryStreamExt;
use log::info;
use sqlx::{query, sqlite::SqliteRow, Pool, Row, Sqlite, SqliteExecutor};
use thiserror::Error;

use super::model::{Game, GameConfig};
use crate::{
    auth::model::MAX_USERNAME_LEN,
    db::{SQLITE_CONSTRAINT_PRIMARYKEY, SQLITE_CONSTRAINT_UNIQUE},
    db_error,
    games::model::{MAX_GAME_NAME_LEN, MAX_MAP_NAME_LEN},
};

#[derive(Clone)]
pub(super) struct Games {
    pool: &'static Pool<Sqlite>,
}

impl Games {
    /// This method sets up the database by creating required tables if they do
    /// not already exist.
    ///
    /// It is supposed users were already setup.
    pub(super) async fn init(pool: &'static Pool<Sqlite>) -> Result<Self> {
        let init_query = format!(
            include_str!("init.sql"),
            username_len = MAX_USERNAME_LEN,
            game_name_len = MAX_GAME_NAME_LEN,
            map_name_lenght = MAX_MAP_NAME_LEN,
        );

        info!("Initializing games...");
        query(&init_query)
            .execute(pool)
            .await
            .context("DB initialization failed")?;
        Ok(Self { pool })
    }

    /// This method creates a new game in the DB and places all users to it.
    pub(super) async fn list(&self) -> Result<Vec<GameConfig>> {
        let mut rows = query("SELECT * FROM games;").fetch(self.pool);
        let mut games = Vec::with_capacity(rows.size_hint().0);
        while let Some(row) = rows
            .try_next()
            .await
            .context("Failed to retrieve a game from the DB")?
        {
            games.push(GameConfig::try_from(row)?);
        }

        Ok(games)
    }

    /// This method creates a new game in the DB and places all users to it.
    pub(super) async fn create(&self, game: Game) -> Result<(), CreationError> {
        let game_config = game.config();

        let mut transaction = self.pool.begin().await.map_err(CreationError::Database)?;

        let result = query("INSERT INTO games (name, max_players, map_name) VALUES(?, ?, ?);")
            .bind(game_config.name())
            .bind(game_config.max_players())
            .bind(game_config.map_name())
            .execute(&mut transaction)
            .await;
        db_error!(
            result,
            CreationError::NameTaken,
            SQLITE_CONSTRAINT_PRIMARYKEY
        );
        result.map_err(CreationError::Database)?;

        for username in game.players() {
            Self::add_player_inner(&mut transaction, username, game_config.name())
                .await
                .map_err(CreationError::AdditionError)?;
        }

        transaction
            .commit()
            .await
            .map_err(CreationError::Database)?;

        Ok(())
    }

    pub(super) async fn add_player_inner<'c, E>(
        executor: E,
        username: &str,
        game: &str,
    ) -> Result<(), AdditionError>
    where
        E: SqliteExecutor<'c>,
    {
        let result = query("INSERT INTO players (username, game) VALUES (?, ?);")
            .bind(username)
            .bind(game)
            .bind(game)
            .execute(executor)
            .await;

        db_error!(
            result,
            AdditionError::AlreadyInAGame,
            SQLITE_CONSTRAINT_UNIQUE
        );
        result.map_err(AdditionError::Database)?;

        Ok(())
    }
}

#[derive(Error, Debug)]
pub(super) enum CreationError {
    #[error("Game name is already taken")]
    NameTaken,
    #[error("Could not add all players to the game")]
    AdditionError(#[source] AdditionError),
    #[error("A database error encountered")]
    Database(#[source] sqlx::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub(super) enum AdditionError {
    #[error("User is already in another game")]
    AlreadyInAGame,
    #[error("A database error encountered")]
    Database(#[source] sqlx::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl TryFrom<SqliteRow> for GameConfig {
    type Error = anyhow::Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        let name: String = row.try_get("name")?;
        let max_players: u8 = row.try_get("max_players")?;
        let map_name: String = row.try_get("map_name")?;
        Ok(Self::new(name, max_players, map_name))
    }
}
