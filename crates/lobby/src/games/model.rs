use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

pub(super) const MAX_GAME_NAME_LEN: usize = 32;
pub(super) const MAX_MAP_NAME_LEN: usize = 32;
const MAX_PLAYERS: u8 = 4;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Game {
    config: GameConfig,
    players: Vec<String>,
}

impl Game {
    /// Creates a new game with the author being the only player.
    pub(super) fn new(config: GameConfig, author: String) -> Self {
        Self {
            config,
            players: vec![author],
        }
    }

    pub(super) fn config(&self) -> &GameConfig {
        &self.config
    }

    pub(super) fn players(&self) -> &[String] {
        self.players.as_slice()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GamePartial {
    config: GameConfig,
    num_players: u8,
}

impl GamePartial {
    pub(super) fn new(config: GameConfig, num_players: u8) -> Self {
        Self {
            config,
            num_players,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GameConfig {
    name: String,
    max_players: u8,
    map_name: String,
}

impl GameConfig {
    pub(super) fn new(name: String, max_players: u8, map_name: String) -> Self {
        Self {
            name,
            max_players,
            map_name,
        }
    }

    pub(super) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(super) fn max_players(&self) -> u8 {
        self.max_players
    }

    pub(super) fn map_name(&self) -> &str {
        self.map_name.as_str()
    }
}

impl GameConfig {
    pub(super) fn validate(&self) -> Result<()> {
        ensure!(!self.name.is_empty(), "Game name cannot be empty.");
        ensure!(
            self.name == self.name.trim(),
            "Game name must not start or end with whitespace."
        );
        ensure!(
            self.name.len() <= MAX_GAME_NAME_LEN,
            "Game name is too long: {} > {}",
            self.name.len(),
            MAX_GAME_NAME_LEN
        );

        ensure!(
            self.max_players >= 2,
            "Maximum number of players must be at least 2."
        );
        ensure!(
            self.max_players <= MAX_PLAYERS,
            "Maximum number of players must be at most {}.",
            MAX_PLAYERS
        );

        ensure!(
            self.map_name.len() <= MAX_MAP_NAME_LEN,
            "Map name is too long: {} > {}",
            self.map_name.len(),
            MAX_MAP_NAME_LEN
        );

        Ok(())
    }
}
