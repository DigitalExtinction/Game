use crate::map::plugin::MapPlugin;
use crate::{camera::MainCameraPlugin, selection::SelectionPlugin};
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
        group.add(MainCameraPlugin).add(SelectionPlugin);
    }
}
