use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use map::MapLoaderPlugin;

mod map;

pub struct LoaderPluginGroup;

impl PluginGroup for LoaderPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(MapLoaderPlugin);
    }
}
