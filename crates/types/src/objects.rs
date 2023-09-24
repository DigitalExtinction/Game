#![allow(clippy::modulo_one)] // Caused by derive(Enum) on an enum with only a
                              // single variant.

use std::fmt;

use bincode::{Decode, Encode};
use enum_iterator::Sequence;
use enum_map::Enum;
use serde::{Deserialize, Serialize};

/// Maximum number of buildings belonging to a single player.
pub const PLAYER_MAX_BUILDINGS: u32 = 128;
/// Maximum number of units belonging to a single player.
pub const PLAYER_MAX_UNITS: u32 = 1024;

#[derive(Debug, Encode, Decode, Enum, Sequence, Copy, Clone, PartialEq, Eq, Hash)]
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

#[derive(
    Debug, Encode, Decode, Enum, Sequence, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash,
)]
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

#[derive(
    Debug, Encode, Decode, Enum, Sequence, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash,
)]
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

#[derive(
    Debug, Encode, Decode, Enum, Sequence, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash,
)]
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

#[derive(
    Debug, Encode, Decode, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Enum, Sequence, Hash,
)]
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
