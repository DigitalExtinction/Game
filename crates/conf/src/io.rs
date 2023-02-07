use anyhow::{Context, Result};
use async_std::{fs, path::Path};
use bevy::prelude::info;

use crate::{conf, persisted};

pub(super) async fn load_conf(path: &Path) -> Result<conf::Configuration> {
    match load_conf_text(path).await? {
        Some(text) => {
            let persistent: persisted::Configuration =
                serde_yaml::from_str(text.as_str()).context("Failed to parse DE configuration")?;
            conf::Configuration::try_from(persistent)
        }
        None => Ok(conf::Configuration::default()),
    }
}

/// Loads configuration file to a string. Returns Ok(None) if the configuration
/// file does not exist.
async fn load_conf_text(path: &Path) -> Result<Option<String>> {
    if path.is_file().await {
        info!("Loading configuration from {}", path.to_string_lossy());
        fs::read_to_string(path).await.map(Some).with_context(|| {
            format!(
                "Could not load DE configuration file: {}",
                path.to_string_lossy(),
            )
        })
    } else {
        info!(
            "Configuration does not exist or is not a file, using defaults: {}",
            path.to_string_lossy()
        );
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use async_std::{path::PathBuf, task};
    use de_uom::Metre;

    use super::*;

    #[test]
    fn test_load_conf() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("conf.yaml");
        let conf = task::block_on(load_conf(path.as_path())).unwrap();

        assert_eq!(
            conf.multiplayer().server().as_str(),
            "http://example.com/de/"
        );
        assert_eq!(conf.camera().min_distance(), Metre::new(12.5));
        assert_eq!(conf.camera().max_distance(), Metre::new(250.));
    }
}
