use std::fmt;

use bevy::prelude::*;
use de_types::objects::ObjectType;

/// Active object which can be played by any player (including AI).
#[derive(Component)]
pub struct Active;

/// Active object which is locally simulated (e.g. played by a local AI or
/// local player).
#[derive(Component)]
pub struct Local;

/// Active object which can be played by the local player.
#[derive(Component)]
pub struct Playable;

/// A rigid object which can not move.
#[derive(Component)]
pub struct StaticSolid;

/// An rigid object which can move.
#[derive(Component)]
pub struct MovableSolid;

#[derive(Component, Deref, Clone, Copy)]
pub struct ObjectTypeComponent(ObjectType);

impl fmt::Display for ObjectTypeComponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ObjectType> for ObjectTypeComponent {
    fn from(object_type: ObjectType) -> Self {
        Self(object_type)
    }
}
