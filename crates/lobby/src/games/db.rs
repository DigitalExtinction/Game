use std::net::SocketAddr;

use anyhow::{Context, Result};
use de_lobby_model::{
    Game, GameConfig, GameListing, GameMap, GamePartial, GamePlayer, GamePlayerInfo, GameSetup,
    MAP_HASH_LEN, MAX_GAME_NAME_LEN, MAX_MAP_NAME_LEN, MAX_USERNAME_LEN,
};
use futures_util::TryStreamExt;
use log::info;
use sqlx::{
    postgres::{PgExecutor, PgRow},
    query, Pool, Postgres, Row,
};
use thiserror::Error;

use crate::{
    db::{FromRow, SQLITE_CONSTRAINT_FOREIGNKEY, SQLITE_CONSTRAINT_PRIMARYKEY},
    db_error_code, db_error_message,
};

// This should correspond to the longest valid socket address. IPv6 hast up to
// 39 characters + colon + 5 characters for port number.
const SERVER_LEN: usize = 45;

#[derive(Clone)]
pub(super) struct Games {
    pool: &'static Pool<Postgres>,
}

impl Games {
    /// This method sets up the database by creating required tables if they do
    /// not already exist.
    ///
    /// It is supposed users were already setup.
    pub(super) async fn init(pool: &'static Pool<Postgres>) -> Result<Self> {
        let init_query = format!(
            include_str!("init.sql"),
            username_len = MAX_USERNAME_LEN,
            game_name_len = MAX_GAME_NAME_LEN,
            map_name_len = MAX_MAP_NAME_LEN,
            map_hash_len = MAP_HASH_LEN,
            server_len = SERVER_LEN,
        );

        info!("Initializing games...");
        query(&init_query)
            .execute(pool)
            .await
            .context("DB initialization failed")?;
        Ok(Self { pool })
    }

    /// This method creates a new game in the DB and places all users to it.
    pub(super) async fn list(&self) -> Result<GameListing> {
        let mut rows = query(
            "SELECT games.*, count(players.ordinal) as num_players \
             FROM games \
             LEFT JOIN players ON (games.name = players.game) \
             GROUP BY games.name;",
        )
        .fetch(self.pool);
        let mut games = GameListing::empty();
        while let Some(row) = rows
            .try_next()
            .await
            .context("Failed to retrieve a game from the DB")?
        {
            games.push(GamePartial::try_from_row(row)?);
        }

        Ok(games)
    }

    /// This method retrieves complete info about a single game.
    pub(super) async fn get(&self, game: &str) -> Result<Option<Game>> {
        let Some(game_row) = query("SELECT * FROM games WHERE name = ?;")
            .bind(game)
            .fetch_optional(self.pool)
            .await
            .context("Failed to retrieve a game from the DB")?
        else {
            return Ok(None);
        };

        let setup = GameSetup::try_from_row(game_row)?;

        let mut players = Vec::new();
        let mut player_rows = query("SELECT ordinal, username FROM players WHERE game = ?;")
            .bind(game)
            .fetch(self.pool);

        while let Some(player_row) = player_rows
            .try_next()
            .await
            .context("Failed to retrieve game players from the DB")?
        {
            players.push(GamePlayer::try_from_row(player_row)?);
        }

        Ok(Some(Game::new(setup, players)))
    }

    /// This method creates a new game in the DB and places all users to it.
    pub(super) async fn create(&self, game: Game) -> Result<(), CreationError> {
        let game_setup = game.setup();
        let game_config = game_setup.config();

        let mut transaction = self.pool.begin().await.map_err(CreationError::Database)?;

        let result =
            query("INSERT INTO games (name, max_players, map_hash, map_name, server) VALUES(?, ?, ?, ?, ?);")
                .bind(game_config.name())
                .bind(game_config.max_players() as i16)
                .bind(game_config.map().hash())
                .bind(game_config.map().name())
                .bind(game_setup.server().to_string())
                .execute(&mut transaction)
                .await;
        db_error_code!(
            result,
            CreationError::NameTaken,
            SQLITE_CONSTRAINT_PRIMARYKEY
        );
        result.map_err(CreationError::Database)?;

        let mut author = true;
        for username in game.players() {
            Self::add_player_inner(&mut transaction, author, username, game_config.name())
                .await
                .map_err(CreationError::AdditionError)?;
            author = false;
        }

        transaction
            .commit()
            .await
            .map_err(CreationError::Database)?;

        Ok(())
    }

    pub(super) async fn add_player(
        &self,
        player: &GamePlayer,
        game: &str,
    ) -> Result<(), AdditionError> {
        Self::add_player_inner(self.pool, false, player, game).await
    }

    async fn add_player_inner<'c, E>(
        executor: E,
        author: bool,
        player: &GamePlayer,
        game: &str,
    ) -> Result<(), AdditionError>
    where
        E: PgExecutor<'c>,
    {
        let result =
            query("INSERT INTO players (ordinal, author, username, game) VALUES (?, ?, ?, ?);")
                .bind(player.info().ordinal() as i16)
                .bind(author)
                .bind(player.username())
                .bind(game)
                .execute(executor)
                .await;

        db_error_code!(
            result,
            AdditionError::UserOrGameDoesNotExist,
            SQLITE_CONSTRAINT_FOREIGNKEY
        );

        db_error_message!(
            result,
            AdditionError::AlreadyInAGame,
            "UNIQUE constraint failed: players.username"
        );
        db_error_message!(
            result,
            AdditionError::OrdinalConflict,
            "UNIQUE constraint failed: players.game, players.ordinal"
        );
        db_error_message!(result, AdditionError::OrdinalTooLarge, "TOO-LARGE-ORDINAL");

        result.map_err(AdditionError::Database)?;

        Ok(())
    }

    /// Removes a player from a game. Deletes the game if the player was the
    /// game author.
    pub(super) async fn remove_player(
        &self,
        username: &str,
        game: &str,
    ) -> Result<(), RemovalError> {
        let mut transaction = self.pool.begin().await.map_err(RemovalError::Database)?;

        let mut rows = query("SELECT author FROM players WHERE username = ? AND game = ?;")
            .bind(username)
            .bind(game)
            .fetch(self.pool);

        let action = match rows.try_next().await.map_err(RemovalError::Database)? {
            Some(row) => {
                let author: bool = row.try_get("author").map_err(RemovalError::Database)?;
                if author {
                    RemovalAction::Abandoned
                } else {
                    RemovalAction::Removed
                }
            }
            None => return Err(RemovalError::NotInTheGame),
        };

        match action {
            RemovalAction::Abandoned => {
                query("DELETE FROM games WHERE name = ?;")
                    .bind(game)
                    .execute(&mut transaction)
                    .await
                    .map_err(RemovalError::Database)?;
            }
            RemovalAction::Removed => {
                Self::remove_player_inner(&mut transaction, username, game).await?;
            }
        }

        transaction.commit().await.map_err(RemovalError::Database)?;
        Ok(())
    }

    async fn remove_player_inner<'c, E>(
        executor: E,
        username: &str,
        game: &str,
    ) -> Result<(), RemovalError>
    where
        E: PgExecutor<'c>,
    {
        let query_result = query("DELETE FROM players WHERE username = ? AND game = ?;")
            .bind(username)
            .bind(game)
            .execute(executor)
            .await
            .map_err(RemovalError::Database)?;

        let rows_affected = query_result.rows_affected();
        assert!(rows_affected <= 1);
        if rows_affected == 0 {
            return Err(RemovalError::NotInTheGame);
        }

        Ok(())
    }
}

/// Action taken during removal of a player from a game.
enum RemovalAction {
    /// The game was abandoned and all players removed from the game.
    Abandoned,
    /// The player left the game without any further action taken.
    Removed,
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
    #[error("Another player already joined the game with the same ordinal")]
    OrdinalConflict,
    #[error("Player ordinal is larger than maximum number of players in the game")]
    OrdinalTooLarge,
    #[error("The user or the game does not exist")]
    UserOrGameDoesNotExist,
    #[error("A database error encountered")]
    Database(#[source] sqlx::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub(super) enum RemovalError {
    #[error("User is not in the game")]
    NotInTheGame,
    #[error("A database error encountered")]
    Database(#[source] sqlx::Error),
}

impl FromRow for GamePlayer {
    type Error = anyhow::Error;

    fn try_from_row(row: PgRow) -> Result<Self, Self::Error> {
        let username: String = row.try_get("username")?;
        let ordinal: u8 = row.try_get::<i16, &str>("ordinal")?.try_into()?;
        Ok(Self::new(username, GamePlayerInfo::new(ordinal)))
    }
}

impl FromRow for GameSetup {
    type Error = anyhow::Error;

    fn try_from_row(row: PgRow) -> Result<Self, Self::Error> {
        let server: String = row.try_get("server")?;
        let server: SocketAddr = server.parse()?;
        let config = GameConfig::try_from_row(row)?;
        Ok(Self::new(server, config))
    }
}

impl FromRow for GamePartial {
    type Error = anyhow::Error;

    fn try_from_row(row: PgRow) -> Result<Self, Self::Error> {
        let num_players: u8 = row.try_get::<i16, &str>("num_players")?.try_into()?;
        let config = GameConfig::try_from_row(row)?;
        Ok(Self::new(config, num_players))
    }
}

impl FromRow for GameConfig {
    type Error = anyhow::Error;

    fn try_from_row(row: PgRow) -> Result<Self, Self::Error> {
        let name: String = row.try_get("name")?;
        let max_players: u8 = row.try_get::<i16, &str>("max_players")?.try_into()?;
        let map = GameMap::try_from_row(row)?;
        Ok(Self::new(name, max_players, map))
    }
}

impl FromRow for GameMap {
    type Error = anyhow::Error;

    fn try_from_row(row: PgRow) -> Result<Self, Self::Error> {
        let hash: String = row.try_get("map_hash")?;
        let name: String = row.try_get("map_name")?;
        Ok(Self::new(hash, name))
    }
}
