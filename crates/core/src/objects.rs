use std::fmt;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// Any rigid object on the map which cannot be passed through.
#[derive(Component)]
pub struct SolidObject;

/// Active object which can be played by the local player.
#[derive(Component)]
pub struct Playable;

/// An object which can move.
#[derive(Component)]
pub struct Movable;

#[derive(Copy, Clone, Debug, Component, Serialize, Deserialize)]
pub enum InactiveObjectType {
    Tree,
}

impl fmt::Display for InactiveObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Tree => write!(f, "Tree"),
        }
    }
}

#[derive(Copy, Clone, Debug, Component, Serialize, Deserialize)]
pub enum ActiveObjectType {
    Base,
    PowerHub,
    Attacker,
}

impl fmt::Display for ActiveObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Base => write!(f, "Base"),
            Self::PowerHub => write!(f, "Power Hub"),
            Self::Attacker => write!(f, "Attacker"),
        }
    }
}
