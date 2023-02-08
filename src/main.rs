#[cfg(not(target_os = "macos"))]
use bevy::window::CursorGrabMode;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::WindowMode,
};
use de_behaviour::BehaviourPluginGroup;
use de_camera::CameraPluginGroup;
use de_combat::CombatPluginGroup;
use de_conf::ConfigPluginGroup;
use de_controller::ControllerPluginGroup;
use de_core::{
    state::{AppState, GameState},
    CorePluginGroup,
};
use de_gui::GuiPluginGroup;
use de_index::IndexPluginGroup;
use de_loader::LoaderPluginGroup;
use de_lobby_client::LobbyClientPluginGroup;
use de_menu::MenuPluginGroup;
use de_movement::MovementPluginGroup;
use de_objects::ObjectsPluginGroup;
use de_pathing::PathingPluginGroup;
use de_signs::SignsPluginGroup;
use de_spawner::SpawnerPluginGroup;
use de_terrain::TerrainPluginGroup;
use iyes_loopless::prelude::*;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_SHA: &str = env!("GIT_SHA");

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Digital Extinction".to_string(),
                mode: WindowMode::BorderlessFullscreen,
                ..Default::default()
            },
            ..default()
        }))
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(GamePlugin)
        .add_plugins(ConfigPluginGroup)
        .add_plugins(GuiPluginGroup)
        .add_plugins(LobbyClientPluginGroup)
        .add_plugins(MenuPluginGroup)
        .add_plugins(CorePluginGroup)
        .add_plugins(ObjectsPluginGroup)
        .add_plugins(TerrainPluginGroup)
        .add_plugins(LoaderPluginGroup)
        .add_plugins(IndexPluginGroup)
        .add_plugins(PathingPluginGroup)
        .add_plugins(SignsPluginGroup)
        .add_plugins(SpawnerPluginGroup)
        .add_plugins(MovementPluginGroup)
        .add_plugins(ControllerPluginGroup)
        .add_plugins(CameraPluginGroup)
        .add_plugins(BehaviourPluginGroup)
        .add_plugins(CombatPluginGroup);

    // This has to be after LogPlugin is inserted.
    info!(
        "Starting Digital Extinction {{ \"Version\": \"{}\", \"GitSha\": \"{}\" }}",
        CARGO_PKG_VERSION, GIT_SHA
    );

    app.run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state_before_stage(CoreStage::PreUpdate, AppState::AppLoading)
            .add_loopless_state_before_stage(CoreStage::PreUpdate, GameState::None);

        #[cfg(not(target_os = "macos"))]
        {
            app.add_enter_system(AppState::AppLoading, cursor_grab_system);
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn cursor_grab_system(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_cursor_grab_mode(CursorGrabMode::Confined);
}
