use anyhow::{bail, Context};
use bevy::reflect::TypeUuid;
use serde::Deserialize;
use std::io::Read;
use tar::Archive;

#[derive(Clone, Copy, Debug, TypeUuid, Deserialize)]
#[uuid = "bbf80d94-c4de-4c7c-9bdc-552ef25aff4e"]
pub struct MapSize(pub f32);

#[derive(Debug, TypeUuid, Deserialize)]
#[uuid = "2f2f3f01-8184-4824-beab-50ed0d81550e"]
pub struct MapDescription {
    pub size: MapSize,
}

pub fn load_from_slice(bytes: &[u8]) -> anyhow::Result<MapDescription> {
    let mut map: Option<MapDescription> = None;

    let mut archive = Archive::new(bytes);
    for entry in archive.entries()? {
        let mut entry = entry?;
        if entry.path()?.to_str().map_or(false, |p| p == "map.json") {
            let mut buf: Vec<u8> = Vec::new();
            entry.read_to_end(&mut buf)?;
            map = Some(serde_json::from_slice(buf.as_slice()).context("Failed to parse map.json")?);
        }
    }

    let map = match map {
        Some(map_description) => map_description,
        None => bail!("map.json entry is not present"),
    };

    validate_map(&map)?;
    Ok(map)
}

fn validate_map(map: &MapDescription) -> anyhow::Result<()> {
    if !map.size.0.is_finite() {
        bail!("Map size has to be finite, got: {}", map.size.0);
    }
    if map.size.0 <= 0. {
        bail!("Map size has to be positive, got: {}", map.size.0);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, path::PathBuf};

    #[test]
    fn test_map_parsing() {
        let mut map_bytes = Vec::new();
        let mut test_map = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_map.push("test_data/test-map.tar");
        File::open(test_map)
            .unwrap()
            .read_to_end(&mut map_bytes)
            .unwrap();
        let map = load_from_slice(map_bytes.as_slice()).unwrap();
        assert_eq!(map.size.0, 108.1);
    }
}
