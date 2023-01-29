use de_core::player::Player;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    hash::MapHasher,
    size::{MapBounds, MapBoundsValidationError},
};

pub const MAX_MAP_NAME_LEN: usize = 16;

/// General information about a map. It does not hold full content of the map
/// (i.e. location of objects on the map).
#[derive(Serialize, Deserialize, Clone)]
pub struct MapMetadata {
    name: String,
    bounds: MapBounds,
    max_player: Player,
}

impl MapMetadata {
    /// Creates a new map description.
    ///
    /// # Arguments
    ///
    /// * `name` - name of the map.
    ///
    /// * `bounds` - bounds of the map.
    ///
    /// * `max_player` - maximum number of players which can play on the map.
    ///   For example, if the value is [de_core::player::Player::Player3], then
    ///   Player1 to `PlayerN` can play.
    ///
    /// # Panics
    ///
    /// Panics if any of the map parameters is invalid.
    pub fn new(name: String, bounds: MapBounds, max_player: Player) -> Self {
        let map = Self {
            name,
            bounds,
            max_player,
        };
        map.validate().unwrap();
        map
    }

    pub(crate) fn update_hash(&self, hasher: &mut MapHasher) {
        hasher.update_str(&self.name);
        hasher.update_vec2(self.bounds.min());
        hasher.update_vec2(self.bounds.max());
        hasher.update_u8(self.max_player.to_num());
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn bounds(&self) -> MapBounds {
        self.bounds
    }

    pub fn max_player(&self) -> Player {
        self.max_player
    }

    pub(crate) fn validate(&self) -> Result<(), MapMetadataValidationError> {
        if self.name.is_empty() {
            return Err(MapMetadataValidationError::MapName(
                "map name is empty".into(),
            ));
        }
        if self.name.len() > MAX_MAP_NAME_LEN {
            return Err(MapMetadataValidationError::MapName(format!(
                "map name too long: {} > {}",
                self.name.len(),
                MAX_MAP_NAME_LEN
            )));
        }

        if let Err(error) = self.bounds.validate() {
            return Err(MapMetadataValidationError::MapBounds { source: error });
        }

        if self.max_player < Player::Player2 {
            return Err(MapMetadataValidationError::MaxPlayers(self.max_player));
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum MapMetadataValidationError {
    #[error("invalid map name: {0}")]
    MapName(String),
    #[error("invalid map bounds")]
    MapBounds { source: MapBoundsValidationError },
    #[error("map has to have at least 2 players, got {0}")]
    MaxPlayers(Player),
}
