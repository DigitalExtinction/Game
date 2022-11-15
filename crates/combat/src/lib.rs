pub use attack::AttackEvent;
use attack::AttackPlugin;
use bevy::{
    app::PluginGroupBuilder,
    prelude::{PluginGroup, SystemLabel},
};
use laser::LaserPlugin;
use trail::TrailPlugin;

mod attack;
mod laser;
mod sightline;
mod trail;

pub struct CombatPluginGroup;

impl PluginGroup for CombatPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(LaserPlugin).add(AttackPlugin).add(TrailPlugin);
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
enum AttackingLabels {
    Update,
    Fire,
}
