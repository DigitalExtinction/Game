use de_core::player::Player;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::size::{MapBounds, MapBoundsValidationError};

/// General information about a map. It does not hold full content of the map
/// (i.e. location of objects on the map).
#[derive(Serialize, Deserialize)]
pub struct MapMetadata {
    bounds: MapBounds,
    max_player: Player,
}

impl MapMetadata {
    /// Creates a new map description.
    ///
    /// # Arguments
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
    pub fn new(bounds: MapBounds, max_player: Player) -> Self {
        let map = Self { bounds, max_player };
        map.validate().unwrap();
        map
    }

    pub fn bounds(&self) -> MapBounds {
        self.bounds
    }

    pub fn max_player(&self) -> Player {
        self.max_player
    }

    pub(crate) fn validate(&self) -> Result<(), MapMetadataValidationError> {
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
    #[error("invalid map bounds")]
    MapBounds { source: MapBoundsValidationError },
    #[error("map has to have at least 2 players, got {0}")]
    MaxPlayers(Player),
}
