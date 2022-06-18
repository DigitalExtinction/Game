#![allow(clippy::modulo_one)] // Caused by derive(Enum) on an enum with only a
                              // single variant.

use std::fmt;

use bevy::prelude::*;
use enum_map::Enum;
use serde::{Deserialize, Serialize};

/// Active object which can be played by the local player.
#[derive(Component)]
pub struct Playable;

/// A rigid object which can not move.
#[derive(Component)]
pub struct StaticSolid;

/// An rigid object which can move.
#[derive(Component)]
pub struct MovableSolid;

#[derive(Enum)]
pub enum ObjectType {
    Active(ActiveObjectType),
    Inactive(InactiveObjectType),
}

#[derive(Copy, Clone, Debug, Component, Serialize, Deserialize, PartialEq, Enum)]
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

#[derive(Copy, Clone, Debug, Component, Serialize, Deserialize, PartialEq, Enum)]
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
