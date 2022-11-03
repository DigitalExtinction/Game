use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use map::MapLoaderPlugin;

mod map;
mod map_select;

pub struct LoaderPluginGroup;

impl PluginGroup for LoaderPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(MapLoaderPlugin);
        group.add(map_select::MapSelectPlugin);
    }
}
