use bevy::prelude::Component;

#[derive(Component)]
pub struct SolidObject;

/// Active object controlled by a player.
#[derive(Component)]
pub struct Active;

/// Active object which can be played by the local player.
#[derive(Component)]
pub struct Playable;

/// A unit which can move around the map.
#[derive(Component)]
pub struct Movable;
