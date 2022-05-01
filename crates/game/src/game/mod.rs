use bevy::{
    app::PluginGroupBuilder,
    prelude::{App, Plugin, PluginGroup, SystemLabel},
};
use de_core::{gconfig::GameConfig, player::Player};
use iyes_loopless::prelude::*;

use self::{
    camera::CameraPlugin, command::CommandPlugin, maploader::MapLoaderPlugin,
    movement::MovementPlugin, pointer::PointerPlugin, selection::SelectionPlugin,
    spawner::SpawnerPlugin,
};

mod camera;
mod collisions;
mod command;
mod maploader;
mod movement;
mod pointer;
mod selection;
mod spawner;
mod terrain;
pub mod tree;

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
            .add(MovementPlugin)
            .add(SpawnerPlugin);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    Loading,
    Playing,
}