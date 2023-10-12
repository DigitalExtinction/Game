use bincode::{Decode, Encode};
pub use chat::{ChatMessage, ChatMessageError, MAX_CHAT_LEN};
use de_types::{objects::ActiveObjectType, player::Player};
pub use entity::{EntityNet, NetEntityIndex};
pub use geom::{TransformNet, Vec2Net, Vec3Net, Vec4Net};
pub use path::{PathError, PathNet};
pub use projectile::NetProjectile;

mod chat;
mod entity;
mod geom;
mod path;
mod projectile;

/// Messages to be sent by a player/client or occasionally the game server to
/// other players.
#[derive(Debug, Decode)]
pub struct FromPlayers {
    /// ID of the sending player.
    source: Player,
    /// Original message.
    message: ToPlayers,
}

impl FromPlayers {
    /// ID of the sending player
    pub fn source(&self) -> Player {
        self.source
    }

    pub fn message(&self) -> &ToPlayers {
        &self.message
    }
}

/// Messages to be sent by a player/client or occasionally the game server to
/// other players.
#[derive(Debug, Encode, Clone, Copy)]
pub struct BorrowedFromPlayers<'a> {
    /// ID of the sending player.
    source: Player,
    /// Original message.
    message: &'a ToPlayers,
}

impl<'a> BorrowedFromPlayers<'a> {
    pub fn new(source: Player, message: &'a ToPlayers) -> Self {
        Self { source, message }
    }
}

/// Message to be sent by a player/client or occasionally the game server to
/// the game server for the distribution to other game players.
///
/// All messages controlling an active entity / object must be local on the
/// sending computer.
#[derive(Debug, Encode, Decode)]
pub enum ToPlayers {
    Chat(ChatMessage),
    /// Spawn a new active object on the map.
    Spawn {
        entity: EntityNet,
        player: Player,
        object_type: ActiveObjectType,
        transform: TransformNet,
    },
    /// Despawn an active object type.
    Despawn {
        entity: EntityNet,
    },
    /// Set path to be followed for an object. Any preexisting path will be
    /// replaced by this one.
    SetPath {
        entity: EntityNet,
        waypoints: Option<PathNet>,
    },
    /// Instantaneously transform an object.
    ///
    /// This has no effect on scheduled path as it just moves the object which
    /// then continues following the path.
    Transform {
        entity: EntityNet,
        transform: TransformNet,
    },
    /// Changes entity health by an amount.
    ChangeHealth {
        entity: EntityNet,
        delta: HealthDelta,
    },
    /// Some kind of projectile was spawned (e.g. rocket, laser trail).
    Projectile(NetProjectile),
}

#[derive(Debug, Encode, Decode)]
pub struct HealthDelta(f32);

impl TryFrom<f32> for HealthDelta {
    type Error = &'static str;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err("Got non-finite health delta.")
        }
    }
}

impl From<&HealthDelta> for f32 {
    fn from(delta: &HealthDelta) -> f32 {
        delta.0
    }
}
