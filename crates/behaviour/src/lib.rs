//! This crate implements various entity behavior systems.

use attack::AttackPlugin;
pub use attack::AttackTarget;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

mod attack;

pub struct BehaviourPluginGroup;

impl PluginGroup for BehaviourPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(AttackPlugin);
    }
}
