use std::path::{Path, PathBuf};

use bevy::prelude::Resource;

use crate::player::{Player, PlayerRange};

/// This resource is automatically removed when
/// [`crate::state::AppState::InGame`] is exited.
#[derive(Resource)]
pub struct GameConfig {
    map_path: PathBuf,
    max_player: Player,
    locals: LocalPlayers,
}

impl GameConfig {
    pub fn new<P: Into<PathBuf>>(map_path: P, max_player: Player, locals: LocalPlayers) -> Self {
        if let Err(err) = locals.validate(max_player) {
            panic!("Invalid LocalPlayers configuration: {err:?}");
        }

        Self {
            map_path: map_path.into(),
            max_player,
            locals,
        }
    }

    pub fn map_path(&self) -> &Path {
        self.map_path.as_path()
    }

    pub fn players(&self) -> PlayerRange {
        PlayerRange::up_to(self.max_player)
    }

    pub fn locals(&self) -> &LocalPlayers {
        &self.locals
    }
}

/// Info about players directly controlled or simulated on this computer.
///
/// "Playable" is the player directly controlled by the user of this computer.
///
/// "Local" is either a "Playable" player or a player controlled by the AI on
/// this computer. During a multiplayer game, each AI player is simulated by
/// exactly one computer.
pub struct LocalPlayers {
    playable: Player,
}

impl LocalPlayers {
    /// # Arguments
    ///
    /// * `playable` - the player controlled locally by the user.
    pub fn new(playable: Player) -> Self {
        Self { playable }
    }

    /// The player controlled directly by the user on this computer.
    pub fn playable(&self) -> Player {
        self.playable
    }

    /// Returns true if the player is controlled directly by the user on this
    /// computer.
    pub fn is_playable(&self, player: Player) -> bool {
        self.playable == player
    }

    fn validate(&self, max_player: Player) -> Result<(), String> {
        if self.playable > max_player {
            return Err(format!(
                "Playable player {} is larger than maximum number of players {max_player}.",
                self.playable
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_config() {
        let config = GameConfig::new(
            "/some/path",
            Player::Player4,
            LocalPlayers::new(Player::Player1),
        );
        assert_eq!(config.map_path().to_string_lossy(), "/some/path");
    }
}
