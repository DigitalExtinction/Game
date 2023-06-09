mod battery;

pub use battery::component::Battery;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

pub struct EnergyPluginGroup;

impl PluginGroup for EnergyPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(battery::BatteryPlugin)
    }
}
