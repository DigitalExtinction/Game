use bevy::prelude::Component;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Component, PartialEq, Eq)]
pub enum Player {
    Player1,
    Player2,
    Player3,
    Player4,
}

impl Player {
    fn to_num(self) -> u8 {
        match self {
            Self::Player1 => 1,
            Self::Player2 => 2,
            Self::Player3 => 3,
            Self::Player4 => 4,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "player {}", self.to_num())
    }
}

impl PartialOrd for Player {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_num().partial_cmp(&other.to_num())
    }
}

impl Ord for Player {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// Any rigid object on the map which cannot be passed through.
#[derive(Component)]
pub struct SolidObject;

/// Active object controlled by a player.
#[derive(Component)]
pub struct Active;

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

#[derive(Copy, Clone, Debug, Component, Serialize, Deserialize)]
pub enum ActiveObjectType {
    Base,
    PowerHub,
    Attacker,
}
