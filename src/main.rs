use bevy::{prelude::*, window::WindowMode};
use de_camera::CameraPluginGroup;
use de_controller::ControllerPluginGroup;
use de_game::game::GamePluginGroup;
use de_index::IndexPluginGroup;
use de_movement::MovementPluginGroup;
use de_pathing::PathingPluginGroup;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Digital Extinction".to_string(),
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePluginGroup)
        .add_plugins(IndexPluginGroup)
        .add_plugins(PathingPluginGroup)
        .add_plugins(MovementPluginGroup)
        .add_plugins(ControllerPluginGroup)
        .add_plugins(CameraPluginGroup)
        .run();
}
