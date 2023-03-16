use bevy::{app::PluginGroupBuilder, prelude::*};

pub struct ConstructionPluginGroup;

impl PluginGroup for ConstructionPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
    }
}
