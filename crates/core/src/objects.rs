#![allow(clippy::modulo_one)] // Caused by derive(Enum) on an enum with only a
                              // single variant.

use std::fmt;

use bevy::prelude::*;
use enum_map::Enum;
use serde::{Deserialize, Serialize};

/// Maximum number of buildings belonging to a single player.
pub const PLAYER_MAX_BUILDINGS: u32 = 128;
/// Maximum number of units belonging to a single player.
pub const PLAYER_MAX_UNITS: u32 = 1024;

/// Active object which can be played by any player.
#[derive(Component)]
pub struct Active;

/// Active object which can be played by the local player.
#[derive(Component)]
pub struct Playable;

/// A rigid object which can not move.
#[derive(Component)]
pub struct StaticSolid;

/// An rigid object which can move.
#[derive(Component)]
pub struct MovableSolid;

#[derive(Enum, Component, Copy, Clone, PartialEq, Eq)]
pub enum ObjectType {
    Active(ActiveObjectType),
    Inactive(InactiveObjectType),
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Active(active) => write!(f, "Active -> {active}"),
            Self::Inactive(inactive) => write!(f, "Inactive -> {inactive}"),
        }
    }
}

#[derive(Copy, Clone, Debug, Component, Serialize, Deserialize, PartialEq, Eq, Enum)]
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

#[derive(Copy, Clone, Debug, Component, Serialize, Deserialize, PartialEq, Eq, Enum)]
pub enum ActiveObjectType {
    Building(BuildingType),
    Unit(UnitType),
}

impl fmt::Display for ActiveObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Building(building) => write!(f, "Building -> {building}"),
            Self::Unit(unit) => write!(f, "Unit -> {unit}"),
        }
    }
}

#[derive(Copy, Clone, Debug, Component, Serialize, Deserialize, PartialEq, Eq, Enum)]
pub enum BuildingType {
    Base,
    PowerHub,
}

impl fmt::Display for BuildingType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Base => write!(f, "Base"),
            Self::PowerHub => write!(f, "Power Hub"),
        }
    }
}

#[derive(Copy, Clone, Hash, Debug, Component, Serialize, Deserialize, PartialEq, Eq, Enum)]
pub enum UnitType {
    Attacker,
}

impl fmt::Display for UnitType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Attacker => write!(f, "Attacker"),
        }
    }
}
