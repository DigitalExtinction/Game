use crate::game::GamePluginGroup;
use bevy::{prelude::*, window::WindowMode};

pub mod game;
pub mod math;

pub fn start() {
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppStates {
    Game,
}
