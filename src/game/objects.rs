use bevy::prelude::Component;

#[derive(Component)]
pub struct SolidObject;

/// Active object controlled by a player.
#[derive(Component)]
pub struct Active {
    player: u8,
}

impl Active {
    pub fn new(player: u8) -> Self {
        Self { player }
    }

    pub fn player(&self) -> u8 {
        self.player
    }
}

/// Active object which can be played by the local player.
#[derive(Component)]
pub struct Playable;

/// A unit which can move around the map.
#[derive(Component)]
pub struct Movable;

#[derive(Copy, Clone, Component, PartialEq)]
pub enum ActiveObjectType {
    Base,
    PowerHub,
    Attacker,
}
