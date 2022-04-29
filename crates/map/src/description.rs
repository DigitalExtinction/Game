use core::f32::consts::TAU;

use bevy::prelude::Transform;
use de_core::{
    objects::{ActiveObjectType, InactiveObjectType},
    player::Player,
};
use glam::{Quat, Vec2, Vec3};
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
    ///   opposed to map bounds origin). (x, y) coordinates correspond to (x,
    ///   z) coordinates in the 3D world.
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

        for (i, object) in self.objects.iter().enumerate() {
            if let Err(error) = object.validate(self.bounds, self.max_player) {
                return Err(MapValidationError::Object {
                    index: i,
                    source: error,
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
    #[error("invalid objects[{index}]")]
    Object {
        index: usize,
        source: ObjectValidationError,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Object {
    placement: Placement,
    object_type: ObjectType,
}

impl Object {
    pub fn new(placement: Placement, object_type: ObjectType) -> Self {
        Self {
            placement,
            object_type,
        }
    }

    /// Object placement on the map.
    pub fn placement(&self) -> Placement {
        self.placement
    }

    pub fn object_type(&self) -> &ObjectType {
        &self.object_type
    }

    fn validate(
        &self,
        map_bounds: MapBounds,
        max_player: Player,
    ) -> Result<(), ObjectValidationError> {
        if let Err(error) = self.placement.validate(map_bounds) {
            return Err(ObjectValidationError::PlacementError { source: error });
        }

        match &self.object_type {
            ObjectType::Active(object) => {
                if let Err(error) = object.validate(max_player) {
                    return Err(ObjectValidationError::ActiveObjectError { source: error });
                }
            }
            ObjectType::Inactive(_) => (),
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

#[derive(Serialize, Deserialize)]
pub enum ObjectType {
    Active(ActiveObject),
    Inactive(InactiveObject),
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

    /// Produces world to object transform which can be used to position the
    /// object on the map.
    pub fn to_transform(self) -> Transform {
        let translation = Vec3::new(self.position.x, 0., self.position.y);
        let rotation = Quat::from_rotation_y(self.heading);
        Transform {
            translation,
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

    use super::*;

    #[test]
    fn test_map() {
        let mut map = Map::empty(MapBounds::new(Vec2::new(1000., 1000.)), Player::Player3);
        let object_a = Object::new(
            map.new_placement(Vec2::new(20., 25.), 0.),
            ObjectType::Active(ActiveObject::new(
                ActiveObjectType::Attacker,
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
                    object_type: ObjectType::Active(ActiveObject::new(
                        ActiveObjectType::Attacker,
                        Player::Player2,
                    )),
                },
                Object {
                    placement: Placement {
                        position: Vec2::new(100., 0.),
                        heading: 0.,
                    },
                    object_type: ObjectType::Active(ActiveObject::new(
                        ActiveObjectType::Attacker,
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
