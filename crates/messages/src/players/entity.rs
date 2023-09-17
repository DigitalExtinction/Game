use bincode::{Decode, Encode};
use de_types::player::Player;

/// Bevy ECS Entity derived identification of an entity.
#[derive(Clone, Copy, Debug, Encode, Decode, Hash, PartialEq, Eq)]
pub struct EntityNet {
    player: Player,
    index: u32,
}

impl EntityNet {
    /// # Arguments
    ///
    /// * `player` - the human player executing the entity simulating game
    ///   instance.
    ///
    /// * `index` - locally unique index of the entity.
    pub fn new(player: Player, index: u32) -> Self {
        Self { player, index }
    }

    pub fn index(&self) -> u32 {
        self.index
    }
}
