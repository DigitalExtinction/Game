use core::fmt;
use std::cmp::Ordering;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Copy, Clone, Debug, Serialize, Deserialize, Component, PartialEq, Eq, Hash)]
pub enum Player {
    #[default]
    Player1,
    Player2,
    Player3,
    Player4,
}

impl Player {
    pub const MAX_PLAYERS: usize = 4;

    pub const fn to_num(self) -> u8 {
        match self {
            Self::Player1 => 1,
            Self::Player2 => 2,
            Self::Player3 => 3,
            Self::Player4 => 4,
        }
    }

    fn next(self) -> Option<Self> {
        self.to_num()
            .checked_add(1)
            .and_then(|num| Self::try_from(num).ok())
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

impl TryFrom<u8> for Player {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Player::Player1),
            2 => Ok(Player::Player2),
            3 => Ok(Player::Player3),
            4 => Ok(Player::Player4),
            value => Err(format!(
                "Player number must be between 1 and 4, got {value}."
            )),
        }
    }
}

pub struct PlayerRange {
    start: Player,
    stop: Player,
    current: Option<Player>,
}

impl PlayerRange {
    /// Returns inclusive player range from first player to `stop`.
    pub fn up_to(stop: Player) -> Self {
        Self::new(Player::Player1, stop)
    }

    /// # Arguments
    ///
    /// * `start` - first player, inclusive.
    ///
    /// * `stop` - last player, inclusive.
    pub fn new(start: Player, stop: Player) -> Self {
        assert!(start <= stop);
        Self {
            start,
            stop,
            current: Some(start),
        }
    }

    pub fn contains(&self, player: Player) -> bool {
        self.start <= player && player <= self.stop
    }
}

impl Iterator for PlayerRange {
    type Item = Player;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(current) => {
                self.current = current.next();
                Some(current)
            }
            None => {
                self.current = Some(self.start);
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = match self.current {
            Some(current) => 1 + (self.stop.to_num() - current.to_num()) as usize,
            None => 0,
        };
        (size, Some(size))
    }
}

impl ExactSizeIterator for PlayerRange {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range() {
        let mut range = PlayerRange::new(Player::Player2, Player::Player4);

        assert_eq!(range.next(), Some(Player::Player2));
        assert_eq!(range.next(), Some(Player::Player3));
        assert_eq!(range.next(), Some(Player::Player4));
        assert_eq!(range.next(), None);

        assert_eq!(range.next(), Some(Player::Player2));
        assert_eq!(range.next(), Some(Player::Player3));
        assert_eq!(range.next(), Some(Player::Player4));
        assert_eq!(range.next(), None);
    }
}
