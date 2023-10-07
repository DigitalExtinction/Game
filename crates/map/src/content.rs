use ahash::AHashMap;
use de_types::{
    objects::{ActiveObjectType, InactiveObjectType, PLAYER_MAX_BUILDINGS, PLAYER_MAX_UNITS},
    player::Player,
};
use enum_map::Enum;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    hash::MapHasher,
    meta::MapMetadata,
    placement::{Placement, PlacementValidationError},
    size::MapBounds,
};

/// Content of the map.
///
/// This object is potentially large.
#[derive(Serialize, Deserialize)]
pub struct MapContent {
    objects: Vec<Object>,
}

impl MapContent {
    /// Returns a slice of all objects placed on the map.
    pub fn objects(&self) -> &[Object] {
        self.objects.as_slice()
    }

    pub(crate) fn empty() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub(crate) fn update_hash(&self, hasher: &mut MapHasher) {
        for object in &self.objects {
            object.update_hash(hasher);
        }
    }

    /// Inserts an object to the map.
    ///
    /// This method does no validation which is why it is only `pub(crate)`.
    pub(crate) fn insert_object(&mut self, object: Object) {
        self.objects.push(object);
    }

    pub(crate) fn validate(&self, metadata: &MapMetadata) -> Result<(), MapContentValidationError> {
        #[derive(Default)]
        struct Counter {
            buildings: u32,
            units: u32,
        }

        let mut counts: AHashMap<Player, Counter> = AHashMap::new();

        for (i, object) in self.objects.iter().enumerate() {
            if let InnerObject::Active(object) = object.inner() {
                let counter = counts.entry(object.player()).or_default();

                match object.object_type() {
                    ActiveObjectType::Building(_) => counter.buildings += 1,
                    ActiveObjectType::Unit(_) => counter.units += 1,
                }
            }

            if let Err(error) = object.validate(metadata.bounds(), metadata.max_player()) {
                return Err(MapContentValidationError::Object {
                    index: i,
                    source: error,
                });
            }
        }

        for (&player, counter) in counts.iter() {
            if counter.buildings > PLAYER_MAX_BUILDINGS {
                return Err(MapContentValidationError::MaxBuildings {
                    player,
                    max: PLAYER_MAX_BUILDINGS,
                    number: counter.buildings,
                });
            }
            if counter.units > PLAYER_MAX_UNITS {
                return Err(MapContentValidationError::MaxUnits {
                    player,
                    max: PLAYER_MAX_UNITS,
                    number: counter.units,
                });
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum MapContentValidationError {
    #[error("maximum number {player} buildings is {max}, got {number}")]
    MaxBuildings {
        player: Player,
        max: u32,
        number: u32,
    },
    #[error("maximum number of {player} units is {max}, got {number}")]
    MaxUnits {
        player: Player,
        max: u32,
        number: u32,
    },
    #[error("invalid objects[{index}]")]
    Object {
        index: usize,
        source: ObjectValidationError,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Object {
    placement: Placement,
    inner: InnerObject,
}

impl Object {
    pub fn new(placement: Placement, inner: InnerObject) -> Self {
        Self { placement, inner }
    }

    fn update_hash(&self, hasher: &mut MapHasher) {
        self.placement.update_hash(hasher);
        self.inner.update_hash(hasher);
    }

    /// Object placement on the map.
    pub fn placement(&self) -> Placement {
        self.placement
    }

    pub fn inner(&self) -> &InnerObject {
        &self.inner
    }

    pub(crate) fn validate(
        &self,
        map_bounds: MapBounds,
        max_player: Player,
    ) -> Result<(), ObjectValidationError> {
        if let Err(error) = self.placement.validate(map_bounds) {
            return Err(ObjectValidationError::PlacementError { source: error });
        }

        match &self.inner {
            InnerObject::Active(object) => {
                if let Err(error) = object.validate(max_player) {
                    return Err(ObjectValidationError::ActiveObjectError { source: error });
                }
            }
            InnerObject::Inactive(_) => (),
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ObjectValidationError {
    #[error("invalid object placement")]
    PlacementError { source: PlacementValidationError },
    #[error("active object error")]
    ActiveObjectError { source: ActiveObjectValidationError },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum InnerObject {
    Active(ActiveObject),
    Inactive(InactiveObject),
}

impl InnerObject {
    fn update_hash(&self, hasher: &mut MapHasher) {
        match self {
            Self::Active(object) => object.update_hash(hasher),
            Self::Inactive(object) => object.update_hash(hasher),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ActiveObject {
    object_type: ActiveObjectType,
    player: Player,
}

impl ActiveObject {
    pub fn new(object_type: ActiveObjectType, player: Player) -> Self {
        Self {
            object_type,
            player,
        }
    }

    fn update_hash(&self, hasher: &mut MapHasher) {
        hasher.update_usize(self.object_type.into_usize());
        hasher.update_u8(self.player.to_num());
    }

    pub fn object_type(&self) -> ActiveObjectType {
        self.object_type
    }

    pub fn player(&self) -> Player {
        self.player
    }

    fn validate(&self, max_player: Player) -> Result<(), ActiveObjectValidationError> {
        if self.player > max_player {
            return Err(ActiveObjectValidationError::MaxPlayerError {
                max_player,
                player: self.player,
            });
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ActiveObjectValidationError {
    #[error("maximum player is {max_player}, got player {player}")]
    MaxPlayerError { max_player: Player, player: Player },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InactiveObject {
    object_type: InactiveObjectType,
}

impl InactiveObject {
    pub fn new(object_type: InactiveObjectType) -> Self {
        Self { object_type }
    }

    fn update_hash(&self, hasher: &mut MapHasher) {
        hasher.update_usize(self.object_type.into_usize());
    }

    pub fn object_type(&self) -> InactiveObjectType {
        self.object_type
    }
}
