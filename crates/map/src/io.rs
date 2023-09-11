use std::io;

use async_std::{
    fs::{File, OpenOptions},
    io::{ReadExt, Write},
    path::Path,
    stream::StreamExt,
};
use async_tar::{Archive, Builder, Entry, EntryType, Header};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

use crate::{
    map::{Map, MapValidationError},
    meta::MapMetadata,
};

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

/// Maps are normally named with this suffix.
pub const MAP_FILE_SUFFIX: &str = ".dem.tar";
const METADATA_JSON_ENTRY: &str = "metadata.json";
const CONTENT_JSON_ENTRY: &str = "content.json";

type LoadingResult<T> = Result<T, MapLoadingError>;
type StoringResult = Result<(), MapStoringError>;

/// Load map metadata from a map file.
pub async fn load_metadata<P: AsRef<Path>>(path: P) -> LoadingResult<MapMetadata> {
    let mut file = loading_io_error!(File::open(&path).await);
    let archive = Archive::new(&mut file);
    let mut entries = loading_io_error!(archive.entries());

    while let Some(entry) = entries.next().await {
        let mut entry = loading_io_error!(entry);
        let path = loading_io_error!(entry.path());
        let Some(path) = path.to_str() else {
            return Err(MapLoadingError::ArchiveContent(String::from(
                "The map archive contains an entry with non-UTF-8 path.",
            )));
        };

        if path == METADATA_JSON_ENTRY {
            return deserialize_entry(&mut entry).await;
        }
    }

    Err(MapLoadingError::ArchiveContent(format!(
        "{METADATA_JSON_ENTRY} entry is not present"
    )))
}

/// Load a map TAR file.
pub async fn load_map<P: AsRef<Path>>(path: P) -> LoadingResult<Map> {
    let mut file = loading_io_error!(File::open(&path).await);
    let archive = Archive::new(&mut file);
    let mut entries = loading_io_error!(archive.entries());

    let mut map_meta = None;
    let mut map_content = None;

    while let Some(entry) = entries.next().await {
        let mut entry = loading_io_error!(entry);
        let path = loading_io_error!(entry.path());
        let Some(path) = path.to_str() else {
            return Err(MapLoadingError::ArchiveContent(String::from(
                "The map archive contains an entry with non-UTF-8 path.",
            )));
        };

        if path == METADATA_JSON_ENTRY {
            map_meta = deserialize_entry(&mut entry).await?;
        } else if path == CONTENT_JSON_ENTRY {
            map_content = deserialize_entry(&mut entry).await?;
        }
    }

    let map_meta = unwrap(METADATA_JSON_ENTRY, map_meta)?;
    let map_content = unwrap(CONTENT_JSON_ENTRY, map_content)?;
    let map = Map::new(map_meta, map_content);

    if let Err(error) = map.validate() {
        return Err(MapLoadingError::Validation { source: error });
    }

    Ok(map)
}

async fn deserialize_entry<T: DeserializeOwned>(
    entry: &mut Entry<Archive<&mut File>>,
) -> LoadingResult<T> {
    let entry_size = loading_io_error!(entry.header().entry_size());
    let mut buf: Vec<u8> = Vec::with_capacity(entry_size.try_into().unwrap());
    loading_io_error!(entry.read_to_end(&mut buf).await);
    match serde_json::from_slice(buf.as_slice()) {
        Ok(map_inner) => Ok(map_inner),
        Err(error) => Err(MapLoadingError::JsonParsing { source: error }),
    }
}

fn unwrap<T>(entry_name: &str, wrapped: Option<T>) -> LoadingResult<T> {
    match wrapped {
        Some(wrapped) => Ok(wrapped),
        None => Err(MapLoadingError::ArchiveContent(format!(
            "{entry_name} entry is not present"
        ))),
    }
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
pub async fn store_map<P: AsRef<Path>>(map: &Map, path: P) -> StoringResult {
    let file = storing_io_error!(
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .await
    );

    let mut archive = Builder::new(file);

    serialize_entry(&mut archive, METADATA_JSON_ENTRY, map.metadata()).await?;
    serialize_entry(&mut archive, CONTENT_JSON_ENTRY, map.content()).await?;

    Ok(())
}

async fn serialize_entry<W, T>(
    archive: &mut Builder<W>,
    entry_name: &str,
    part: &T,
) -> StoringResult
where
    W: Write + Unpin + Send + Sync,
    T: ?Sized + Serialize,
{
    let data = match serde_json::ser::to_vec(part) {
        Ok(data) => data,
        Err(error) => return Err(MapStoringError::JsonSerialization { source: error }),
    };

    let mut header = Header::new_gnu();
    header.set_entry_type(EntryType::Regular);
    header.set_mode(0x400);
    header.set_size(data.len().try_into().unwrap());
    storing_io_error!(
        archive
            .append_data(&mut header, Path::new(entry_name), data.as_slice())
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
    use de_types::{
        objects::{ActiveObjectType, BuildingType},
        player::Player,
    };
    use glam::Vec2;
    use parry2d::{bounding_volume::Aabb, math::Point};
    use tempfile::Builder;

    use super::*;
    use crate::{
        content::{ActiveObject, InnerObject, Object},
        map::Map,
        meta::MapMetadata,
        size::MapBounds,
    };

    #[test]
    fn test_store_load() {
        let bounds = MapBounds::new(Vec2::new(1000., 2000.));
        let mut map = Map::empty(MapMetadata::new("Test Map".into(), bounds, Player::Player4));

        let bases = [
            (Vec2::new(-400., -900.), Player::Player1),
            (Vec2::new(400., -900.), Player::Player2),
            (Vec2::new(400., 900.), Player::Player3),
            (Vec2::new(-400., 900.), Player::Player4),
        ];

        for (base_position, player) in bases {
            map.insert_object(Object::new(
                map.new_placement(base_position, 0.),
                InnerObject::Active(ActiveObject::new(
                    ActiveObjectType::Building(BuildingType::Base),
                    player,
                )),
            ));
        }

        let tmp_dir = Builder::new().prefix("de_map_").tempdir().unwrap();
        let mut tmp_dir_path = PathBuf::from(tmp_dir.path());
        tmp_dir_path.push("test-map.dem.tar");

        task::block_on(store_map(&map, tmp_dir_path.as_path())).unwrap();
        let loaded_map = task::block_on(load_map(tmp_dir_path.as_path())).unwrap();

        assert_eq!(
            loaded_map.metadata().bounds().aabb(),
            Aabb::new(Point::new(-500., -1000.), Point::new(500., 1000.))
        );
    }

    #[test]
    fn test_load_metadata() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let mut map_path = PathBuf::from(manifest_dir);
        map_path.push("tests");
        map_path.push("test-map.dem.tar");

        let metadata = task::block_on(load_metadata(map_path)).unwrap();
        assert_eq!(metadata.name(), "A Test Map 🦀");
    }
}
