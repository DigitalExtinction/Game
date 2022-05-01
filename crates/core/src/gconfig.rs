use std::path::{Path, PathBuf};

use crate::player::Player;

pub struct GameConfig {
    map_path: PathBuf,
    player: Player,
}

impl GameConfig {
    pub fn new<P: Into<PathBuf>>(map_path: P, player: Player) -> Self {
        Self {
            map_path: map_path.into(),
            player,
        }
    }

    pub fn map_path(&self) -> &Path {
        self.map_path.as_path()
    }

    pub fn player(&self) -> Player {
        self.player
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_config() {
        let config = GameConfig::new("/some/path", Player::Player1);
        assert_eq!(config.map_path().to_string_lossy(), "/some/path");
    }
}
