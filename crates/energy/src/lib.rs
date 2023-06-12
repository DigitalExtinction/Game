mod battery;

pub use battery::Battery;
pub use battery::EnergyUnit;

use crate::battery::BatteryPlugin;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

pub struct EnergyPluginGroup;

impl PluginGroup for EnergyPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(BatteryPlugin)
    }
}
