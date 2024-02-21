use anyhow::{Context, Result};
use async_std::{fs, path::Path};
use tracing::info;

/// Loads configuration file to a string. Returns Ok(None) if the configuration
/// file does not exist.
pub(crate) async fn load_conf_text(path: &Path) -> Result<Option<String>> {
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
    use std::net::{IpAddr, Ipv6Addr};

    use async_std::{path::PathBuf, task};
    use de_uom::Metre;

    use crate::conf::Configuration;

    #[test]
    fn test_load_conf() {
        tracing_subscriber::fmt::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("conf.yaml");
        let conf = task::block_on(Configuration::load(path.as_path())).unwrap();

        assert_eq!(
            conf.multiplayer().lobby().as_str(),
            "http://example.com/de/"
        );
        assert_eq!(
            conf.multiplayer().connector().ip(),
            IpAddr::V6(Ipv6Addr::LOCALHOST)
        );
        assert_eq!(conf.multiplayer().connector().port(), 8083);
        assert_eq!(conf.camera().min_distance(), Metre::new(12.5));
        assert_eq!(conf.camera().max_distance(), Metre::new(250.));
    }
}
