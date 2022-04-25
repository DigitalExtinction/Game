use crate::description::{Map, MapValidationError};
use async_std::fs::File;
use async_std::path::Path;
use async_std::prelude::*;
use async_tar::Archive;
use std::io;
use thiserror::Error;

macro_rules! loading_io_error {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(err) => return Err(MapLoadingError::Io { source: err }),
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

    let map = match map {
        Some(map) => map,
        None => {
            return Err(MapLoadingError::ArchiveContent(format!(
                "{} entry is not present",
                MAP_JSON_ENTRY
            )))
        }
    };

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
