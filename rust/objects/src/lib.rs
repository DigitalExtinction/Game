use bevy::prelude::Component;

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
