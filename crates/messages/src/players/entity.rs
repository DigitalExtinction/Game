#[cfg(feature = "bevy")]
use bevy::ecs::entity::Entity;
use bincode::{Decode, Encode};

/// Bevy ECS Entity derived identification of an entity.
#[derive(Clone, Copy, Debug, Encode, Decode, Hash, PartialEq, Eq)]
pub struct EntityNet(u32);

#[cfg(feature = "bevy")]
impl From<Entity> for EntityNet {
    fn from(entity: Entity) -> Self {
        Self(entity.index())
    }
}
