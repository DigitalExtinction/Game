use std::{
    fmt::{self, Debug, Write},
    num::ParseIntError,
};

use async_std::path::{Path, PathBuf};
use glam::Vec2;
use sha3::{Digest, Sha3_256};
use thiserror::Error;

use crate::io::MAP_FILE_SUFFIX;

#[derive(PartialEq, Eq)]
pub struct MapHash([u8; 32]);

impl MapHash {
    fn new(hash: [u8; 32]) -> Self {
        Self(hash)
    }

    /// Constructs the map hash from a hexadecimal string.
    pub(crate) fn from_hex(hex: &str) -> Result<Self, HexError> {
        if hex.len() != 64 {
            return Err(HexError::InvalidLenError);
        }

        let mut bytes = [0; 32];
        for i in 0..32 {
            bytes[i] = match u8::from_str_radix(&hex[(2 * i)..(2 * i + 2)], 16) {
                Ok(val) => val,
                Err(error) => return Err(HexError::ParseByteError(error)),
            };
        }

        Ok(Self::new(bytes))
    }

    /// Converts the map hash into a hexadecimal string.
    fn to_hex(&self) -> String {
        let mut hex = String::with_capacity(64);
        for &byte in self.0.iter() {
            write!(&mut hex, "{byte:02x}").unwrap();
        }
        hex
    }

    /// Returns a map file path with canonical map file name.
    ///
    /// The file name starts the hexadecimal hash followed by canonical map
    /// file suffix.
    pub fn construct_path<P: Into<PathBuf>>(&self, dir: P) -> PathBuf {
        let mut dir: PathBuf = dir.into();
        dir.push(format!("{}{}", self.to_hex(), MAP_FILE_SUFFIX));
        dir
    }
}

impl Debug for MapHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", self.to_hex())
    }
}

#[derive(Error, Debug)]
pub enum HexError {
    #[error("Hexadecimal hash is not 64 characters long.")]
    InvalidLenError,
    #[error(transparent)]
    ParseByteError(ParseIntError),
}

impl TryFrom<&Path> for MapHash {
    type Error = PathError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let Some(file_name) = path.file_name() else { return Err(PathError::FileNameError("No file name in the path.")) };
        let Some(file_name) = file_name.to_str() else { return Err(PathError::FileNameError("File name is not a valid UTF-8 string.")) };

        if file_name.ends_with(MAP_FILE_SUFFIX) {
            Self::from_hex(&file_name[0..file_name.len() - MAP_FILE_SUFFIX.len()])
                .map_err(PathError::Hex)
        } else {
            Err(PathError::FileNameError(
                "File name does not end with proper suffix.",
            ))
        }
    }
}

#[derive(Error, Debug)]
pub enum PathError {
    #[error("{0}")]
    FileNameError(&'static str),
    #[error("Hexadecimal parsing failed")]
    Hex(#[source] HexError),
}

pub(crate) struct MapHasher {
    hasher: Sha3_256,
}

impl MapHasher {
    pub(crate) fn new() -> Self {
        Self {
            hasher: Sha3_256::new(),
        }
    }

    pub(crate) fn update_u8(&mut self, value: u8) {
        self.update([value])
    }

    /// Update the hash with an usize. The usize is first converted to u64 for
    /// interoperability.
    pub(crate) fn update_usize(&mut self, value: usize) {
        self.update((value as u64).to_be_bytes())
    }

    pub(crate) fn update_f32(&mut self, value: f32) {
        self.update(value.to_be_bytes())
    }

    pub(crate) fn update_vec2(&mut self, value: Vec2) {
        self.update_f32(value.x);
        self.update_f32(value.y);
    }

    pub(crate) fn update_str(&mut self, value: &str) {
        self.update(value.as_bytes())
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        self.hasher.update(data)
    }

    pub(crate) fn finalize(self) -> MapHash {
        let mut bytes = [0u8; 32].into();
        self.hasher.finalize_into(&mut bytes);
        MapHash::new(bytes.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_path() {
        let hash = MapHash::try_from(Path::new(
            "/a/b/abcdeffffffffffffffffffffff9876543210fffffffffffffffffffffffffff.dem.tar",
        ))
        .unwrap();
        assert_eq!(
            hash.construct_path(Path::new("/c/d")),
            Path::new(
                "/c/d/abcdeffffffffffffffffffffff9876543210fffffffffffffffffffffffffff.dem.tar"
            )
        );
    }

    #[test]
    fn test_hasher() {
        let mut hasher_a = MapHasher::new();
        hasher_a.update_str("test");
        let hash_a = hasher_a.finalize();

        let mut hasher_b = MapHasher::new();
        hasher_b.update_str("test");
        hasher_b.update_f32(2.0);
        let hash_b = hasher_b.finalize();

        assert_eq!(
            hash_a.to_hex(),
            "36f028580bb02cc8272a9a020f4200e346e276ae664e45ee80745574e2f5ab80"
        );
        assert_ne!(hash_a.to_hex(), hash_b.to_hex());
    }
}
