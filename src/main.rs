use bevy::{prelude::*, window::WindowMode};
use de_camera::CameraPluginGroup;
use de_controller::ControllerPluginGroup;
use de_core::{gconfig::GameConfig, player::Player, state::GameState, CorePluginGroup};
use de_index::IndexPluginGroup;
use de_loader::LoaderPluginGroup;
use de_movement::MovementPluginGroup;
use de_objects::ObjectsPluginGroup;
use de_pathing::PathingPluginGroup;
use de_spawner::SpawnerPluginGroup;
use iyes_loopless::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Digital Extinction".to_string(),
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .add_plugins(CorePluginGroup)
        .add_plugins(ObjectsPluginGroup)
        .add_plugins(LoaderPluginGroup)
        .add_plugins(IndexPluginGroup)
        .add_plugins(PathingPluginGroup)
        .add_plugins(SpawnerPluginGroup)
        .add_plugins(MovementPluginGroup)
        .add_plugins(ControllerPluginGroup)
        .add_plugins(CameraPluginGroup)
        .run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(GameState::Loading)
            .insert_resource(GameConfig::new("map.tar", Player::Player1));
    }
}
