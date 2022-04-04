use bevy::{prelude::*, window::WindowMode};
use de::{game::GamePluginGroup, AppStates};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Digital Extinction".to_string(),
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_state(AppStates::Game)
        .add_plugins(GamePluginGroup)
        .run();
}
