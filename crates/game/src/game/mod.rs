use bevy::{
    app::PluginGroupBuilder,
    prelude::{App, Plugin, PluginGroup},
};
use de_core::{gconfig::GameConfig, player::Player, state::GameState};
use iyes_loopless::prelude::*;

use self::{maploader::MapLoaderPlugin, spawner::SpawnerPlugin};

mod maploader;
mod spawner;

pub struct GamePluginGroup;

impl PluginGroup for GamePluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(GamePlugin)
            .add(MapLoaderPlugin)
            .add(SpawnerPlugin);
    }
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(GameState::Loading)
            .insert_resource(GameConfig::new("map.tar", Player::Player1));
    }
}
