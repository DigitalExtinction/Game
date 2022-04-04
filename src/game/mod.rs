use self::{
    camera::CameraPlugin, config::GameConfig, maploader::MapLoaderPlugin,
    selection::SelectionPlugin,
};
use crate::AppStates;
use bevy::{
    app::PluginGroupBuilder,
    prelude::{App, Plugin, PluginGroup, ResMut, State, SystemSet},
};

pub mod config;

mod camera;
mod collisions;
mod mapdescr;
mod maploader;
mod objects;
mod selection;
mod terrain;

pub struct GamePluginGroup;

impl PluginGroup for GamePluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(GamePlugin)
            .add(MapLoaderPlugin)
            .add(CameraPlugin)
            .add(SelectionPlugin);
    }
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state(GameStates::Waiting)
            .insert_resource(GameConfig::new("map.tar"))
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
