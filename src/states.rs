use crate::camera::MainCameraPlugin;
use crate::map::plugin::MapPlugin;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameStates {
    Loading,
    InGame,
}

pub struct GameLoadingPluginGroup;

impl PluginGroup for GameLoadingPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(MapPlugin);
    }
}

pub struct InGamePluginGroup;

impl PluginGroup for InGamePluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(MainCameraPlugin);
    }
}
