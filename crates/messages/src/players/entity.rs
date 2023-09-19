#[cfg(feature = "bevy")]
use bevy::ecs::entity::Entity;
use bincode::{Decode, Encode};
use de_types::player::Player;

/// Bevy ECS Entity derived identification of an entity.
#[derive(Clone, Copy, Debug, Encode, Decode, Hash, PartialEq, Eq)]
pub struct EntityNet {
    player: Player,
    index: NetEntityIndex,
}

impl EntityNet {
    /// # Arguments
    ///
    /// * `player` - the human player executing the entity simulating game
    ///   instance.
    ///
    /// * `index` - locally unique index of the entity.
    pub fn new(player: Player, index: NetEntityIndex) -> Self {
        Self { player, index }
    }

    pub fn player(&self) -> Player {
        self.player
    }

    pub fn index(&self) -> NetEntityIndex {
        self.index
    }
}

#[derive(Clone, Copy, Debug, Encode, Decode, Hash, PartialEq, Eq)]
pub struct NetEntityIndex(u32);

impl From<NetEntityIndex> for u32 {
    fn from(index: NetEntityIndex) -> u32 {
        index.0
    }
}

#[cfg(feature = "bevy")]
impl From<Entity> for NetEntityIndex {
    fn from(entity: Entity) -> Self {
        Self(entity.index())
    }
}
