use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use map::MapLoaderPlugin;
use readiness::ReadinessPlugin;

mod map;
mod readiness;

pub struct LoaderPluginGroup;

impl PluginGroup for LoaderPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(MapLoaderPlugin)
            .add(ReadinessPlugin)
    }
}
