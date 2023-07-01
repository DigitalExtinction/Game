use std::path::{Path, PathBuf};

use bevy::prelude::Resource;

use crate::player::{Player, PlayerRange};

/// This resource is automatically removed when
/// [`crate::state::AppState::InGame`] is exited.
#[derive(Resource)]
pub struct GameConfig {
    map_path: PathBuf,
    player: Player,
    max_player: Player,
}

impl GameConfig {
    pub fn new<P: Into<PathBuf>>(map_path: P, player: Player, max_player: Player) -> Self {
        assert!(player <= max_player);
        Self {
            map_path: map_path.into(),
            player,
            max_player,
        }
    }

    pub fn map_path(&self) -> &Path {
        self.map_path.as_path()
    }

    pub fn player(&self) -> Player {
        self.player
    }

    pub fn is_local_player(&self, player: Player) -> bool {
        self.player == player
    }

    pub fn players(&self) -> PlayerRange {
        PlayerRange::up_to(self.max_player)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_config() {
        let config = GameConfig::new("/some/path", Player::Player1, Player::Player4);
        assert_eq!(config.map_path().to_string_lossy(), "/some/path");
    }
}
