use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use map::MapLoaderPlugin;

mod map;

pub struct LoaderPluginGroup;

impl PluginGroup for LoaderPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(MapLoaderPlugin)
    }
}
