use std::path::{Path, PathBuf};

use bevy::prelude::Resource;
use de_types::player::{Player, PlayerRange};
use tinyvec::{array_vec, ArrayVec};

/// This resource is automatically removed when
/// [`crate::state::AppState::InGame`] is exited.
#[derive(Resource)]
pub struct GameConfig {
    map_path: PathBuf,
    multiplayer: bool,
    locals: LocalPlayers,
}

impl GameConfig {
    pub fn new<P: Into<PathBuf>>(map_path: P, multiplayer: bool, locals: LocalPlayers) -> Self {
        Self {
            map_path: map_path.into(),
            multiplayer,
            locals,
        }
    }

    pub fn map_path(&self) -> &Path {
        self.map_path.as_path()
    }

    pub fn multiplayer(&self) -> bool {
        self.multiplayer
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
    locals: ArrayVec<[Player; Player::MAX_PLAYERS]>,
}

impl LocalPlayers {
    pub fn from_max_player(playable: Player, max_player: Player) -> Self {
        Self::from_range(playable, PlayerRange::up_to(max_player))
    }

    pub fn from_range(playable: Player, locals: PlayerRange) -> Self {
        Self::new(playable, locals.collect())
    }

    pub fn from_single(playable: Player) -> Self {
        Self::new(playable, array_vec!(_ => playable))
    }

    /// # Arguments
    ///
    /// * `playable` - the player controlled locally by the user.
    ///
    /// * `locals` - other players simulated locally on this computer. It must
    ///   include `playable`.
    pub fn new(playable: Player, locals: ArrayVec<[Player; Player::MAX_PLAYERS]>) -> Self {
        assert!((*locals).contains(&playable));
        Self { playable, locals }
    }

    /// The player controlled directly by the user on this computer.
    pub fn playable(&self) -> Player {
        self.playable
    }

    pub fn locals(&self) -> &[Player] {
        self.locals.as_slice()
    }

    /// Returns true if the player is controlled directly by the user on this
    /// computer.
    pub fn is_playable(&self, player: Player) -> bool {
        self.playable == player
    }

    /// Returns true if the player is simulated by this computer.
    pub fn is_local(&self, player: Player) -> bool {
        self.locals.contains(&player)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_config() {
        let config = GameConfig::new(
            "/some/path",
            false,
            LocalPlayers::from_max_player(Player::Player1, Player::Player4),
        );
        assert_eq!(config.map_path().to_string_lossy(), "/some/path");
    }
}
