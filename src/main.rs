use bevy::log::LogPlugin;
#[cfg(not(target_os = "macos"))]
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::WindowMode,
};
use de_behaviour::BehaviourPluginGroup;
use de_camera::CameraPluginGroup;
use de_combat::CombatPluginGroup;
use de_conf::ConfigPluginGroup;
use de_construction::ConstructionPluginGroup;
use de_controller::ControllerPluginGroup;
use de_core::{state::AppState, transition::DeStateTransition, CorePluginGroup};
use de_gui::GuiPluginGroup;
use de_index::IndexPluginGroup;
use de_loader::LoaderPluginGroup;
use de_lobby_client::LobbyClientPluginGroup;
use de_log::LogPluginGroup;
use de_menu::MenuPluginGroup;
use de_movement::MovementPluginGroup;
use de_objects::ObjectsPluginGroup;
use de_pathing::PathingPluginGroup;
use de_signs::SignsPluginGroup;
use de_spawner::SpawnerPluginGroup;
use de_terrain::TerrainPluginGroup;
use tracing::{span, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter};

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_SHA: &str = env!("GIT_SHA");

fn main() {
    // TODO: move log init to de_log crate
    // for file name
    let dt = chrono::Local::now();

    let file_name = dt.format("%Y-%m-%d %H:%M:%S.log").to_string();
    let file_appender = tracing_appender::rolling::never("logs", &*file_name);

    let (non_blocking_log_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let collector = tracing_subscriber::registry()
        .with(
            EnvFilter::from_default_env()
                .add_directive(Level::TRACE.into())
                .add_directive("wgpu_core=warn".parse().unwrap())
                .add_directive("bevy_ecs=info".parse().unwrap())
                .add_directive("async_io=info".parse().unwrap())
                .add_directive("polling=debug".parse().unwrap())
                .add_directive("wgpu_hal=info".parse().unwrap())
                .add_directive("naga=info".parse().unwrap()),
        )
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking_log_writer));
    tracing::subscriber::set_global_default(collector).expect("Unable to set a global collector");

    info!(
        "Starting Digital Extinction {{ \"Version\": \"{}\", \"GitSha\": \"{}\" }}",
        CARGO_PKG_VERSION, GIT_SHA
    );

    // so we can use it after span is closed
    let mut app: App;

    {
        let span = span!(Level::TRACE, "Startup");
        let _enter = span.enter();

        app = App::new();
        app.insert_resource(Msaa::Sample4)
            .add_plugins(LogPluginGroup)
            .add_plugins(
                DefaultPlugins
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: "Digital Extinction".to_string(),
                            mode: WindowMode::BorderlessFullscreen,
                            ..Default::default()
                        }),
                        ..default()
                    })
                    .disable::<LogPlugin>(),
            )
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
            .add_plugins(CombatPluginGroup)
            .add_plugins(ConstructionPluginGroup);
    }

    app.run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state_with_set::<AppState>();

        #[cfg(not(target_os = "macos"))]
        {
            app.add_system(cursor_grab_system.in_schedule(OnEnter(AppState::AppLoading)));
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn cursor_grab_system(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = window_query.single_mut();
    window.cursor.grab_mode = CursorGrabMode::Confined;
}
