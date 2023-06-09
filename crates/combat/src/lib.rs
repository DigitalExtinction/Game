pub use attack::AttackEvent;
use attack::AttackPlugin;
use bevy::{
    app::PluginGroupBuilder,
    prelude::{PluginGroup, SystemSet},
};
use laser::LaserPlugin;
use trail::TrailPlugin;

mod attack;
mod laser;
mod sightline;
mod trail;

pub struct CombatPluginGroup;

impl PluginGroup for CombatPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(LaserPlugin)
            .add(AttackPlugin)
            .add(TrailPlugin)
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum AttackingSet {
    Attack,
    Charge,
    Fire,
}
