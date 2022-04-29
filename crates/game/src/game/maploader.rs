use bevy::{
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    tasks::{IoTaskPool, Task},
};
use de_core::{gconfig::GameConfig, log_full_error, utils::ToMsl};
use de_map::{
    description::Map,
    io::{load_map, MapLoadingError},
    size::MapBounds,
};
use futures_lite::future;
use iyes_loopless::prelude::*;

use super::{terrain::Terrain, GameState};
use crate::{assets::asset_path, game::spawner::SpawnEvent};

pub struct MapLoaderPlugin;

impl Plugin for MapLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Loading, load_map_system)
            .add_system(spawn_map.run_in_state(GameState::Loading));
    }
}

struct MapLoadingTask(Task<Result<Map, MapLoadingError>>);

fn load_map_system(
    mut commands: Commands,
    thread_pool: Res<IoTaskPool>,
    game_config: Res<GameConfig>,
) {
    let map_path = if game_config.map_path().is_relative() {
        asset_path(game_config.map_path())
    } else {
        game_config.map_path().to_owned()
    };

    info!("Loading map from {}", map_path.display());
    let task = thread_pool.spawn(async { load_map(map_path).await });
    commands.insert_resource(MapLoadingTask(task));
}

fn spawn_map(
    mut commands: Commands,
    mut task: ResMut<MapLoadingTask>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events: EventWriter<SpawnEvent>,
) {
    let loading_result = match future::block_on(future::poll_once(&mut task.0)) {
        Some(result) => result,
        None => return,
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

    setup_light(&mut commands);
    setup_terrain(&mut commands, &mut meshes, &mut materials, map.bounds());
    events.send_batch(map.objects().iter().cloned().map(SpawnEvent::new));
    commands.insert_resource(map.bounds());
    commands.insert_resource(NextState(GameState::Playing));
}

fn setup_light(commands: &mut Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.6,
    });

    let mut transform = Transform::identity();
    transform.look_at(Vec3::new(1., -1., 0.), Vec3::new(1., 1., 0.));
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 30000.,
            ..Default::default()
        },
        transform,
        ..Default::default()
    });
}

fn setup_terrain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    bounds: MapBounds,
) {
    let map_size = bounds.size();

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(terrain_mesh(map_size)),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform {
                translation: bounds.min().to_msl(),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Terrain);
}

fn terrain_mesh(size: Vec2) -> Mesh {
    let vertices = [
        ([0., 0., 0.], [0., 1., 0.], [0., 0.]),
        ([0., 0., size.y], [0., 1., 0.], [0., 1.]),
        ([size.x, 0., size.y], [0., 1., 0.], [1., 1.]),
        ([size.x, 0., 0.], [0., 1., 0.], [1., 0.]),
    ];

    let indices = Indices::U32(vec![0, 1, 2, 0, 2, 3]);

    let mut positions = Vec::<[f32; 3]>::new();
    let mut normals = Vec::<[f32; 3]>::new();
    let mut uvs = Vec::<[f32; 2]>::new();
    for (position, normal, uv) in &vertices {
        positions.push(*position);
        normals.push(*normal);
        uvs.push(*uv);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}
