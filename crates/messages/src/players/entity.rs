#[cfg(feature = "bevy")]
use bevy::ecs::entity::Entity;
use bincode::{Decode, Encode};

/// Bevy ECS Entity derived identification of an entity.
#[derive(Debug, Encode, Decode)]
pub struct EntityNet(u32);

impl EntityNet {
    pub fn index(&self) -> u32 {
        self.0
    }
}

#[cfg(feature = "bevy")]
impl From<Entity> for EntityNet {
    fn from(entity: Entity) -> Self {
        Self(entity.index())
    }
}
