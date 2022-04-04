use std::path::{Path, PathBuf};

pub struct GameConfig {
    map_path: PathBuf,
}

impl GameConfig {
    pub fn new<P: Into<PathBuf>>(map_path: P) -> Self {
        Self {
            map_path: map_path.into(),
        }
    }

    pub fn map_path(&self) -> &Path {
        self.map_path.as_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_config() {
        let config = GameConfig::new("/some/path");
        assert_eq!(config.map_path().to_string_lossy(), "/some/path");
    }
}
