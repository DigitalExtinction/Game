use bevy::prelude::*;
use de_types::player::Player;

#[derive(Clone, Copy, Component, Deref)]
pub struct PlayerComponent(Player);

impl From<Player> for PlayerComponent {
    fn from(player: Player) -> Self {
        Self(player)
    }
}
