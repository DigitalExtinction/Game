use std::io;

use async_std::fs::{File, OpenOptions};
use async_std::path::Path;
use async_std::prelude::*;
use async_tar::{Archive, Builder, EntryType, Header};
use thiserror::Error;

use crate::description::{Map, MapValidationError};

macro_rules! loading_io_error {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(err) => return Err(MapLoadingError::Io { source: err }),
        }
    };
}

macro_rules! storing_io_error {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(err) => return Err(MapStoringError::Io { source: err }),
        }
    };
}

const MAP_JSON_ENTRY: &str = "map.json";

/// Load a map TAR file.
pub async fn load_map<P: AsRef<Path>>(path: P) -> Result<Map, MapLoadingError> {
    let mut file = loading_io_error!(File::open(&path).await);
    let archive = Archive::new(&mut file);
    let mut entries = loading_io_error!(archive.entries());

    let mut map = None;

    while let Some(entry) = entries.next().await {
        let mut entry = loading_io_error!(entry);
        let path = loading_io_error!(entry.path());
        if path.to_str().map_or(false, |p| p == MAP_JSON_ENTRY) {
            let entry_size = loading_io_error!(entry.header().entry_size());
            let mut buf: Vec<u8> = Vec::with_capacity(entry_size as usize);
            loading_io_error!(entry.read_to_end(&mut buf).await);
            map = match serde_json::from_slice(buf.as_slice()) {
                Ok(map_inner) => Some(map_inner),
                Err(error) => return Err(MapLoadingError::JsonParsing { source: error }),
            };
        }
    }

    let map: Map = match map {
        Some(map) => map,
        None => {
            return Err(MapLoadingError::ArchiveContent(format!(
                "{} entry is not present",
                MAP_JSON_ENTRY
            )));
        }
    };

    if let Err(error) = map.validate() {
        return Err(MapLoadingError::Validation { source: error });
    }

    Ok(map)
}

#[derive(Error, Debug)]
pub enum MapLoadingError {
    #[error(transparent)]
    Io { source: io::Error },
    #[error("{0}")]
    ArchiveContent(String),
    #[error("map JSON parsing error")]
    JsonParsing { source: serde_json::Error },
    #[error(transparent)]
    Validation { source: MapValidationError },
}

/// Writes a map to a TAR file. Overwrites the file if it already exists.
pub async fn store_map<P: AsRef<Path>>(map: &Map, path: P) -> Result<(), MapStoringError> {
    let file = storing_io_error!(
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .await
    );

    let mut archive = Builder::new(file);

    let map_data = match serde_json::ser::to_vec(map) {
        Ok(data) => data,
        Err(error) => return Err(MapStoringError::JsonSerialization { source: error }),
    };
    let mut map_header = Header::new_gnu();
    map_header.set_entry_type(EntryType::Regular);
    map_header.set_mode(0x400);
    map_header.set_size(map_data.len() as u64);
    storing_io_error!(
        archive
            .append_data(
                &mut map_header,
                Path::new(MAP_JSON_ENTRY),
                map_data.as_slice(),
            )
            .await
    );

    Ok(())
}

#[derive(Error, Debug)]
pub enum MapStoringError {
    #[error(transparent)]
    Io { source: io::Error },
    #[error("map JSON serialization error")]
    JsonSerialization { source: serde_json::Error },
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use async_std::task;
    use de_core::{objects::ActiveObjectType, player::Player};
    use glam::Vec2;
    use parry2d::{bounding_volume::AABB, math::Point};
    use tempfile::Builder;

    use super::*;
    use crate::{
        description::{ActiveObject, Map, Object, ObjectType},
        size::MapBounds,
    };

    #[test]
    fn test_store_load() {
        let bounds = MapBounds::new(Vec2::new(1000., 2000.));
        let mut map = Map::empty(bounds, Player::Player4);

        let bases = [
            (Vec2::new(-400., -900.), Player::Player1),
            (Vec2::new(400., -900.), Player::Player2),
            (Vec2::new(400., 900.), Player::Player3),
            (Vec2::new(-400., 900.), Player::Player4),
        ];

        for (base_position, player) in bases {
            map.insert_object(Object::new(
                map.new_placement(base_position, 0.),
                ObjectType::Active(ActiveObject::new(ActiveObjectType::Base, player)),
            ));
        }

        let tmp_dir = Builder::new().prefix("de_map_").tempdir().unwrap();
        let mut tmp_dir_path = PathBuf::from(tmp_dir.path());
        tmp_dir_path.push("test-map.tar");

        task::block_on(store_map(&map, tmp_dir_path.as_path())).unwrap();
        let loaded_map = task::block_on(load_map(tmp_dir_path.as_path())).unwrap();

        assert_eq!(
            loaded_map.bounds().aabb(),
            AABB::new(Point::new(-500., -1000.), Point::new(500., 1000.))
        );
    }
}
