use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use de_camera::MoveFocusEvent;
use de_core::{
    assets::asset_path,
    gconfig::GameConfig,
    log_full_error,
    objects::{ActiveObjectType, BuildingType, ObjectType},
    state::GameState,
};
use de_map::{
    description::{InnerObject, Map},
    io::{load_map, MapLoadingError},
};
use de_spawner::SpawnBundle;
use de_terrain::TerrainBundle;
use futures_lite::future;
use iyes_loopless::prelude::*;
use iyes_progress::prelude::*;

pub(crate) struct MapLoaderPlugin;

impl Plugin for MapLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Loading, load_map_system)
            .add_system(spawn_map.track_progress().run_in_state(GameState::Loading));
    }
}

struct MapLoadingTask(Task<Result<Map, MapLoadingError>>);

#[derive(Component, Reflect)]
pub struct RemoveBeforeLoad;

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
    entities_to_remove: Query<Entity, Or<(With<Handle<Scene>>, With<RemoveBeforeLoad>)>>,
) -> Progress {
    let mut task = match task {
        Some(task) => task,
        None => return true.into(),
    };

    let loading_result = match future::block_on(future::poll_once(&mut task.0)) {
        Some(result) => result,
        None => return false.into(),
    };

    info!("Map loaded, removing old one");
    for to_remove in entities_to_remove.iter() {
        commands.entity(to_remove).despawn_recursive();
    }
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
    commands
        .spawn_bundle(TerrainBundle::flat(map.bounds()))
        .insert(RemoveBeforeLoad);

    for object in map.objects() {
        let mut entity_commands = commands.spawn();
        let object_type = match object.inner() {
            InnerObject::Active(object) => {
                entity_commands.insert(object.player());
                ObjectType::Active(object.object_type())
            }
            InnerObject::Inactive(object) => ObjectType::Inactive(object.object_type()),
        };
        entity_commands.insert_bundle(SpawnBundle::new(
            object_type,
            object.placement().to_transform(),
        ));
    }

    commands.insert_resource(map.bounds());
    true.into()
}

fn setup_light(commands: &mut Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.6,
    });

    let mut transform = Transform::identity();
    transform.look_at(Vec3::new(1., -1., 0.), Vec3::new(1., 1., 0.));
    commands
        .spawn_bundle(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::WHITE,
                illuminance: 30000.,
                ..Default::default()
            },
            transform,
            ..Default::default()
        })
        .insert(RemoveBeforeLoad);
}
