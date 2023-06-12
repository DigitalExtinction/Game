//! This crate implements various entity behavior systems.

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use chase::ChasePlugin;
pub use chase::{ChaseSet, ChaseTarget, ChaseTargetEvent};

mod chase;

pub struct BehaviourPluginGroup;

impl PluginGroup for BehaviourPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(ChasePlugin)
    }
}
