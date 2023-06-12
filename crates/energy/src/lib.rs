mod battery;

pub use battery::Battery;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

use crate::battery::BatteryPlugin;

pub struct EnergyPluginGroup;

impl PluginGroup for EnergyPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(BatteryPlugin)
    }
}
