use bevy::{
    prelude::*,
    tasks::{futures_lite::future, IoTaskPool, Task},
};
use de_camera::MoveFocusEvent;
use de_core::{
    assets::asset_path, cleanup::DespawnOnGameExit, gamestate::GameState, gconfig::GameConfig,
    log_full_error, state::AppState,
};
use de_map::{
    content::InnerObject,
    io::{load_map, MapLoadingError},
    map::Map,
    size::MapBounds,
};
use de_spawner::{SpawnInactiveEvent, SpawnLocalActiveEvent, SpawnerSet};
use de_terrain::TerrainBundle;
use de_types::objects::{ActiveObjectType, BuildingType};
use iyes_progress::prelude::*;

pub(crate) struct MapLoaderPlugin;

impl Plugin for MapLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), load_map_system)
            .add_systems(OnExit(AppState::InGame), cleanup)
            .add_systems(
                Update,
                spawn_map
                    .track_progress()
                    .run_if(in_state(GameState::Loading))
                    .before(SpawnerSet::Spawner),
            );
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
    mut spawn_active_events: EventWriter<SpawnLocalActiveEvent>,
    mut spawn_inactive_events: EventWriter<SpawnInactiveEvent>,
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
        Err(err) => {
            log_full_error!(err);
            panic!("{}", err);
        }
    };

    let initial_focus = map
        .content()
        .objects()
        .iter()
        .filter_map(|object| match object.inner() {
            InnerObject::Active(active_object) => {
                if game_config.locals().is_playable(active_object.player())
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

    let locals = game_config.locals();
    for object in map.content().objects() {
        let transform = object.placement().to_transform();

        match object.inner() {
            InnerObject::Active(object) => {
                let player = object.player();
                if !locals.is_local(player) {
                    continue;
                }

                spawn_active_events.send(SpawnLocalActiveEvent::stationary(
                    object.object_type(),
                    transform,
                    player,
                ));
            }
            InnerObject::Inactive(object) => {
                spawn_inactive_events
                    .send(SpawnInactiveEvent::new(object.object_type(), transform));
            }
        }
    }

    commands.insert_resource(map.metadata().bounds());
    true.into()
}

fn setup_light(commands: &mut Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 180.,
    });

    let mut transform = Transform::IDENTITY;
    transform.look_at(Vec3::new(1., -1., 0.), Vec3::new(1., 1., 0.));

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::WHITE,
                illuminance: 10_000.,
                shadows_enabled: true,
                ..Default::default()
            },
            transform,
            ..Default::default()
        },
        DespawnOnGameExit,
    ));
}
