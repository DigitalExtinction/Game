use bevy::{
    app::PluginGroupBuilder,
    prelude::{App, Plugin, PluginGroup, SystemLabel},
};
use de_core::{gconfig::GameConfig, player::Player, state::GameState};
use de_index::IndexPlugin;
use de_movement::MovementPlugin;
use de_pathing::PathingPlugin;
use iyes_loopless::prelude::*;

use self::{
    camera::CameraPlugin, command::CommandPlugin, maploader::MapLoaderPlugin,
    pointer::PointerPlugin, selection::SelectionPlugin, spawner::SpawnerPlugin,
};

mod camera;
mod command;
mod maploader;
mod pointer;
mod selection;
mod spawner;
mod terrain;

pub struct GamePluginGroup;

impl PluginGroup for GamePluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(GamePlugin)
            .add(MapLoaderPlugin)
            .add(CameraPlugin)
            .add(SelectionPlugin)
            .add(PointerPlugin)
            .add(CommandPlugin)
            .add(SpawnerPlugin)
            .add(IndexPlugin)
            .add(PathingPlugin)
            .add(MovementPlugin);
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
enum Labels {
    PreInputUpdate,
    InputUpdate,
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(GameState::Loading)
            .insert_resource(GameConfig::new("map.tar", Player::Player1));
    }
}
