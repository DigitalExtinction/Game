use glam::Vec2;
use thiserror::Error;

use crate::{
    content::{MapContent, MapContentValidationError, Object},
    meta::{MapMetadata, MapMetadataValidationError},
    placement::Placement,
};

pub struct Map {
    metadata: MapMetadata,
    content: MapContent,
}

impl Map {
    /// Creates a new empty map (i.e. with no objects place on it).
    pub fn empty(metadata: MapMetadata) -> Self {
        Self::new(metadata, MapContent::empty())
    }

    pub(crate) fn new(metadata: MapMetadata, content: MapContent) -> Self {
        Self { metadata, content }
    }

    pub fn metadata(&self) -> &MapMetadata {
        &self.metadata
    }

    pub fn content(&self) -> &MapContent {
        &self.content
    }

    /// Insert an object to the map.
    ///
    /// # Panics
    ///
    /// Panics if the object is placed out of the map bounds, has an invalid
    /// player or is otherwise invalid.
    pub fn insert_object(&mut self, object: Object) {
        object
            .validate(self.metadata.bounds(), self.metadata.max_player())
            .unwrap();
        self.content.insert_object(object);
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
        let placement = Placement::new(position, heading);
        placement.validate(self.metadata.bounds()).unwrap();
        placement
    }

    pub(crate) fn validate(&self) -> Result<(), MapValidationError> {
        if let Err(error) = self.metadata.validate() {
            return Err(MapValidationError::Metadata { source: error });
        }
        if let Err(error) = self.content.validate(&self.metadata) {
            return Err(MapValidationError::Content { source: error });
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum MapValidationError {
    #[error("invalid map metadata")]
    Metadata { source: MapMetadataValidationError },
    #[error("invalid map content")]
    Content { source: MapContentValidationError },
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use de_core::{
        objects::{ActiveObjectType, UnitType},
        player::Player,
    };
    use glam::Vec2;

    use super::*;
    use crate::{
        content::{ActiveObject, InnerObject},
        placement::Placement,
        size::MapBounds,
    };

    #[test]
    fn test_map() {
        let mut map = Map::empty(MapMetadata::new(
            MapBounds::new(Vec2::new(1000., 1000.)),
            Player::Player3,
        ));
        let object_a = Object::new(
            map.new_placement(Vec2::new(20., 25.), 0.),
            InnerObject::Active(ActiveObject::new(
                ActiveObjectType::Unit(UnitType::Attacker),
                Player::Player1,
            )),
        );
        map.insert_object(object_a);

        map.validate().unwrap();
        assert_eq!(
            map.metadata().bounds(),
            MapBounds::new(Vec2::new(1000., 1000.))
        );
        assert_eq!(map.metadata().max_player(), Player::Player3);
    }

    #[test]
    fn test_map_validation() {
        let mut content = MapContent::empty();
        content.insert_object(Object::new(
            Placement::new(Vec2::new(1., 1.), 0.),
            InnerObject::Active(ActiveObject::new(
                ActiveObjectType::Unit(UnitType::Attacker),
                Player::Player2,
            )),
        ));
        content.insert_object(Object::new(
            Placement::new(Vec2::new(100., 0.), 0.),
            InnerObject::Active(ActiveObject::new(
                ActiveObjectType::Unit(UnitType::Attacker),
                Player::Player1,
            )),
        ));

        let map = Map::new(
            MapMetadata::new(MapBounds::new(Vec2::new(5., 5.)), Player::Player4),
            content,
        );

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

                assert_eq!(chain.len(), 4);
                assert_eq!(chain[0], "invalid map content");
                assert_eq!(chain[1], "invalid objects[1]");
                assert_eq!(chain[2], "invalid object placement");
                assert_eq!(chain[3], "position (100, 0) is out of map bounds");
            }
        }
    }
}
