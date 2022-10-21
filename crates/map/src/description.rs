use core::f32::consts::TAU;

use ahash::AHashMap;
use bevy::prelude::Transform;
use de_core::{
    objects::{ActiveObjectType, InactiveObjectType, PLAYER_MAX_BUILDINGS, PLAYER_MAX_UNITS},
    player::Player,
    projection::ToMsl,
};
use glam::{Quat, Vec2};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::size::{MapBounds, MapBoundsValidationError};

#[derive(Serialize, Deserialize)]
pub struct Map {
    bounds: MapBounds,
    max_player: Player,
    objects: Vec<Object>,
}

impl Map {
    /// Creates an empty new map.
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
    pub fn empty(bounds: MapBounds, max_player: Player) -> Self {
        let map = Self {
            bounds,
            max_player,
            objects: Vec::new(),
        };
        map.validate().unwrap();
        map
    }

    pub fn bounds(&self) -> MapBounds {
        self.bounds
    }

    pub fn max_player(&self) -> Player {
        self.max_player
    }

    pub fn objects(&self) -> &[Object] {
        self.objects.as_slice()
    }

    /// Insert an object to the map.
    ///
    /// # Panics
    ///
    /// Panics if the object is placed out of the map bounds, has an invalid
    /// player or is otherwise invalid.
    pub fn insert_object(&mut self, object: Object) {
        object.validate(self.bounds, self.max_player).unwrap();
        self.objects.push(object);
    }

    /// Creates a new placement on the map.
    ///
    /// # Arguments
    ///
    /// * `position` - (x, y) coordinates of the object relative to (0, 0) (as
    ///   opposed to map bounds origin).
    ///
    /// * `heading` - (counter clockwise) rotation in radians of the object
    ///   around y axis (facing upwards).
    ///
    /// # Panics
    ///
    /// Panics if position is out of bounds of the map or if heading is not a
    /// number between 0 (inclusive) and 2π (exclusive).
    pub fn new_placement(&self, position: Vec2, heading: f32) -> Placement {
        Placement::new(self.bounds, position, heading)
    }

    pub(crate) fn validate(&self) -> Result<(), MapValidationError> {
        if let Err(error) = self.bounds.validate() {
            return Err(MapValidationError::MapBounds { source: error });
        }

        if self.max_player < Player::Player2 {
            return Err(MapValidationError::MaxPlayers(self.max_player));
        }

        #[derive(Default)]
        struct Counter {
            buildings: usize,
            units: usize,
        }

        let mut counts: AHashMap<Player, Counter> = AHashMap::new();

        for (i, object) in self.objects.iter().enumerate() {
            if let InnerObject::Active(object) = object.inner() {
                let counter = counts
                    .entry(object.player())
                    .or_insert_with(Counter::default);

                match object.object_type() {
                    ActiveObjectType::Building(_) => counter.buildings += 1,
                    ActiveObjectType::Unit(_) => counter.units += 1,
                }
            }

            if let Err(error) = object.validate(self.bounds, self.max_player) {
                return Err(MapValidationError::Object {
                    index: i,
                    source: error,
                });
            }
        }

        for (&player, counter) in counts.iter() {
            if counter.buildings > PLAYER_MAX_BUILDINGS {
                return Err(MapValidationError::MaxBuildings {
                    player,
                    max: PLAYER_MAX_BUILDINGS,
                    number: counter.buildings,
                });
            }
            if counter.units > PLAYER_MAX_UNITS {
                return Err(MapValidationError::MaxUnits {
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
pub enum MapValidationError {
    #[error("invalid map bounds")]
    MapBounds { source: MapBoundsValidationError },
    #[error("map has to have at least 2 players, got {0}")]
    MaxPlayers(Player),
    #[error("maximum number {player} buildings is {max}, got {number}")]
    MaxBuildings {
        player: Player,
        max: usize,
        number: usize,
    },
    #[error("maximum number of {player} units is {max}, got {number}")]
    MaxUnits {
        player: Player,
        max: usize,
        number: usize,
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

    /// Object placement on the map.
    pub fn placement(&self) -> Placement {
        self.placement
    }

    pub fn inner(&self) -> &InnerObject {
        &self.inner
    }

    fn validate(
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

    pub fn object_type(&self) -> InactiveObjectType {
        self.object_type
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Placement {
    position: Vec2,
    heading: f32,
}

impl Placement {
    fn new(map_bounds: MapBounds, position: Vec2, heading: f32) -> Self {
        let position = Self { position, heading };
        position.validate(map_bounds).unwrap();
        position
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// Produces world to object transform which can be used to position the
    /// object on the map.
    pub fn to_transform(self) -> Transform {
        let rotation = Quat::from_rotation_y(self.heading);
        Transform {
            translation: self.position.to_msl(),
            rotation,
            ..Default::default()
        }
    }

    fn validate(&self, map_bounds: MapBounds) -> Result<(), PlacementValidationError> {
        if !map_bounds.contains(self.position) {
            return Err(PlacementValidationError::OutOfMapBound(self.position));
        }
        if !self.heading.is_finite() || self.heading < 0. || self.heading >= TAU {
            return Err(PlacementValidationError::InvalidHeading(self.heading));
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum PlacementValidationError {
    #[error("position ({}, {}) is out of map bounds", .0.x, .0.y)]
    OutOfMapBound(Vec2),
    #[error("heading must be between 0 and τ (2π), got: {0}")]
    InvalidHeading(f32),
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use de_core::objects::UnitType;

    use super::*;

    #[test]
    fn test_map() {
        let mut map = Map::empty(MapBounds::new(Vec2::new(1000., 1000.)), Player::Player3);
        let object_a = Object::new(
            map.new_placement(Vec2::new(20., 25.), 0.),
            InnerObject::Active(ActiveObject::new(
                ActiveObjectType::Unit(UnitType::Attacker),
                Player::Player1,
            )),
        );
        map.insert_object(object_a);

        map.validate().unwrap();
        assert_eq!(map.bounds(), MapBounds::new(Vec2::new(1000., 1000.)));
        assert_eq!(map.max_player(), Player::Player3);
    }

    #[test]
    fn test_map_validation() {
        let map = Map {
            bounds: MapBounds::new(Vec2::new(5., 5.)),
            max_player: Player::Player4,
            objects: vec![
                Object {
                    placement: Placement {
                        position: Vec2::new(1., 1.),
                        heading: 0.,
                    },
                    inner: InnerObject::Active(ActiveObject::new(
                        ActiveObjectType::Unit(UnitType::Attacker),
                        Player::Player2,
                    )),
                },
                Object {
                    placement: Placement {
                        position: Vec2::new(100., 0.),
                        heading: 0.,
                    },
                    inner: InnerObject::Active(ActiveObject::new(
                        ActiveObjectType::Unit(UnitType::Attacker),
                        Player::Player1,
                    )),
                },
            ],
        };

        let result = map.validate();
        match result {
            Ok(()) => unreachable!(),
            Err(error) => {
                let mut chain = Vec::new();

                let mut error: Option<&(dyn Error)> = Some(&error);
                while let Some(inner) = error {
                    chain.push(format!("{}", inner));
                    error = inner.source();
                }

                assert_eq!(chain.len(), 3);
                assert_eq!(chain[0], "invalid objects[1]");
                assert_eq!(chain[1], "invalid object placement");
                assert_eq!(chain[2], "position (100, 0) is out of map bounds");
            }
        }
    }
}
