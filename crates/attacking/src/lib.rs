pub use attack::AttackEvent;
use attack::AttackPlugin;
use beam::BeamPlugin;
use bevy::{
    app::PluginGroupBuilder,
    prelude::{PluginGroup, SystemLabel},
};
use laser::LaserPlugin;

mod attack;
mod beam;
mod laser;
mod sightline;

pub struct AttackingPluginGroup;

impl PluginGroup for AttackingPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(LaserPlugin).add(AttackPlugin).add(BeamPlugin);
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub enum AttackingLabels {
    Attack,
    Update,
    Aim,
    Fire,
    Beam,
}
