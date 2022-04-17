use self::{
    camera::CameraPlugin, command::CommandPlugin, config::GameConfig, maploader::MapLoaderPlugin,
    movement::MovementPlugin, pointer::PointerPlugin, positions::PositionPlugin,
    selection::SelectionPlugin, spawner::SpawnerPlugin,
};
use crate::AppStates;
use bevy::{
    app::PluginGroupBuilder,
    prelude::{App, Plugin, PluginGroup, ResMut, State, SystemLabel, SystemSet},
};

pub mod config;

mod camera;
mod collisions;
mod command;
mod mapdescr;
mod maploader;
mod movement;
mod objects;
mod pointer;
mod positions;
mod selection;
mod spawner;
mod terrain;
pub mod tree;

const MAX_ACTIVE_OBJECTS: usize = 10000;

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
            .add(SpawnerPlugin)
            .add(PositionPlugin);
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
        app.add_state(GameStates::Waiting)
            .insert_resource(GameConfig::new("map.tar", 0))
            .add_system_set(SystemSet::on_enter(AppStates::Game).with_system(start_game));
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameStates {
    Waiting,
    Loading,
    Playing,
}

fn start_game(mut game_state: ResMut<State<GameStates>>) {
    game_state.set(GameStates::Loading).unwrap();
}
