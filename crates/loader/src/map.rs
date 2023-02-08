use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use de_camera::MoveFocusEvent;
use de_core::{
    assets::asset_path,
    cleanup::DespawnOnGameExit,
    gconfig::GameConfig,
    log_full_error,
    objects::{ActiveObjectType, BuildingType, ObjectType},
    state::{AppState, GameState},
};
use de_map::{
    content::InnerObject,
    io::{load_map, MapLoadingError},
    map::Map,
    size::MapBounds,
};
use de_spawner::SpawnBundle;
use de_terrain::TerrainBundle;
use futures_lite::future;
use iyes_loopless::prelude::*;
use iyes_progress::prelude::*;

pub(crate) struct MapLoaderPlugin;

impl Plugin for MapLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::InGame, load_map_system)
            .add_exit_system(AppState::InGame, cleanup)
            .add_system(spawn_map.track_progress().run_in_state(GameState::Loading));
    }
}

#[derive(Resource)]
struct MapLoadingTask(Task<Result<Map, MapLoadingError>>);

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<MapLoadingTask>();
    commands.remove_resource::<MapBounds>();
}

fn load_map_system(mut commands: Commands, game_config: Res<GameConfig>) {
    let map_path = if game_config.map_path().is_relative() {
        asset_path(game_config.map_path())
    } else {
        game_config.map_path().to_owned()
    };

    info!("Loading map from {}", map_path.display());
    let task = IoTaskPool::get().spawn(async { load_map(map_path).await });
    commands.insert_resource(MapLoadingTask(task));
}

fn spawn_map(
    mut commands: Commands,
    task: Option<ResMut<MapLoadingTask>>,
    mut move_focus_events: EventWriter<MoveFocusEvent>,
    game_config: Res<GameConfig>,
) -> Progress {
    let mut task = match task {
        Some(task) => task,
        None => return true.into(),
    };

    let loading_result = match future::block_on(future::poll_once(&mut task.0)) {
        Some(result) => result,
        None => return false.into(),
    };

    info!("Map loaded, spawning");
    commands.remove_resource::<MapLoadingTask>();

    let map = match loading_result {
        Ok(map) => map,
        Err(error) => {
            log_full_error!(error);
            panic!("{}", error);
        }
    };

    let initial_focus = map
        .content()
        .objects()
        .iter()
        .filter_map(|object| match object.inner() {
            InnerObject::Active(active_object) => {
                if game_config.is_local_player(active_object.player())
                    && active_object.object_type() == ActiveObjectType::Building(BuildingType::Base)
                {
                    Some(object.placement().position())
                } else {
                    None
                }
            }
            _ => None,
        })
        .next();
    if let Some(focus) = initial_focus {
        move_focus_events.send(MoveFocusEvent::new(focus));
    }

    setup_light(&mut commands);
    commands.spawn((
        TerrainBundle::flat(map.metadata().bounds()),
        DespawnOnGameExit,
    ));

    let players = game_config.players();
    for object in map.content().objects() {
        let (mut entity_commands, object_type) = match object.inner() {
            InnerObject::Active(object) => {
                let player = object.player();
                if !players.contains(player) {
                    continue;
                }

                (
                    commands.spawn(player),
                    ObjectType::Active(object.object_type()),
                )
            }
            InnerObject::Inactive(object) => (
                commands.spawn_empty(),
                ObjectType::Inactive(object.object_type()),
            ),
        };

        entity_commands.insert((
            SpawnBundle::new(object_type, object.placement().to_transform()),
            DespawnOnGameExit,
        ));
    }

    commands.insert_resource(map.metadata().bounds());
    true.into()
}

fn setup_light(commands: &mut Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.6,
    });

    let mut transform = Transform::IDENTITY;
    transform.look_at(Vec3::new(1., -1., 0.), Vec3::new(1., 1., 0.));
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::WHITE,
                illuminance: 30000.,
                ..Default::default()
            },
            transform,
            ..Default::default()
        },
        DespawnOnGameExit,
    ));
}
