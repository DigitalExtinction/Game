mod battery;
mod graph;

pub use battery::Battery;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
pub use graph::{EnergyReceiver, NearbyUnits};

use crate::battery::BatteryPlugin;

pub struct EnergyPluginGroup;

impl PluginGroup for EnergyPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(BatteryPlugin)
            .add(graph::GraphPlugin)
    }
}
