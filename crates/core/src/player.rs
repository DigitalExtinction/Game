use core::fmt;
use std::cmp::Ordering;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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
