use super::{
    config::GameConfig,
    mapdescr::{self, MapDescription, MapObject, MapSize},
    objects,
    spawner::ToBeSpawnedEvent,
    terrain::Terrain,
    GameStates,
};
use anyhow::{bail, Context};
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadState, LoadedAsset},
    pbr::{PbrBundle, StandardMaterial},
    prelude::{shape::Plane, *},
};
use std::io::Read;
use tar::Archive;

pub struct MapLoaderPlugin;

impl Plugin for MapLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<MapDescription>()
            .add_asset_loader(MapLoader)
            .add_state(MapStates::Waiting)
            .add_system_set(SystemSet::on_enter(GameStates::Loading).with_system(load_map))
            .add_system_set(SystemSet::on_update(MapStates::Loading).with_system(wait_for_map))
            .add_system_set(
                SystemSet::on_enter(MapStates::InitingRes).with_system(add_map_resources),
            )
            .add_system_set(
                SystemSet::on_enter(MapStates::Spawning)
                    .with_system(setup_light)
                    .with_system(spawn_terrain.label("spawn_terrain"))
                    .with_system(spawn_objects.label("spawn_objects"))
                    .with_system(finalize.after("spawn_terrain").after("spawn_objects")),
            );
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MapStates {
    Waiting,
    Loading,
    InitingRes,
    Spawning,
}

struct MapLoader;

impl AssetLoader for MapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let map = load_from_slice(bytes).context("Failed to load map")?;
            load_context.set_default_asset(LoadedAsset::new(map));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tar"]
    }
}

pub fn load_from_slice(bytes: &[u8]) -> anyhow::Result<MapDescription> {
    let mut map: Option<MapDescription> = None;

    let mut archive = Archive::new(bytes);
    for entry in archive.entries()? {
        let mut entry = entry?;
        if entry.path()?.to_str().map_or(false, |p| p == "map.json") {
            let mut buf: Vec<u8> = Vec::new();
            entry.read_to_end(&mut buf)?;
            map = Some(serde_json::from_slice(buf.as_slice()).context("Failed to parse map.json")?);
        }
    }

    let map = match map {
        Some(map_description) => map_description,
        None => bail!("map.json entry is not present"),
    };

    map.validate()?;
    Ok(map)
}

fn load_map(
    mut commands: Commands,
    server: Res<AssetServer>,
    game_config: Res<GameConfig>,
    mut map_state: ResMut<State<MapStates>>,
) {
    let handle: Handle<MapDescription> = server.load(game_config.map_path());
    commands.insert_resource(handle);
    map_state.set(MapStates::Loading).unwrap();
}

fn wait_for_map(
    server: Res<AssetServer>,
    map_handle: Res<Handle<MapDescription>>,
    mut map_state: ResMut<State<MapStates>>,
) {
    match server.get_load_state(map_handle.as_ref()) {
        LoadState::Failed => panic!("Map loading has failed."),
        LoadState::Loaded => map_state.set(MapStates::InitingRes).unwrap(),
        _ => (),
    }
}

fn add_map_resources(
    mut commands: Commands,
    map_handle: Res<Handle<MapDescription>>,
    map_assets: Res<Assets<MapDescription>>,
    mut map_state: ResMut<State<MapStates>>,
) {
    let map_size = map_assets.get(map_handle.as_ref()).unwrap().size;
    commands.insert_resource(map_size);
    map_state.set(MapStates::Spawning).unwrap();
}

fn setup_light(mut commands: Commands, map_size: Res<MapSize>) {
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
            shadow_projection: OrthographicProjection {
                left: 0.,
                right: map_size.0,
                bottom: 0.,
                top: map_size.0,
                near: -10.,
                far: 2. * map_size.0,
                ..Default::default()
            },
            shadow_depth_bias: 0.2,
            shadow_normal_bias: 0.2,
            shadows_enabled: true,
        },
        transform,
        ..Default::default()
    });
}

fn spawn_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_size: Res<MapSize>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(Plane { size: map_size.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform {
                translation: Vec3::new(map_size.0 / 2., 0., map_size.0 / 2.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Terrain);
}

fn spawn_objects(
    map_handle: Res<Handle<MapDescription>>,
    maps: Res<Assets<MapDescription>>,
    mut spawn_events: EventWriter<ToBeSpawnedEvent>,
) {
    let map_description = maps.get(map_handle.as_ref()).unwrap();

    // TODO inactive objects
    // for object in map_description.inactive_objects() {
    //     spawn_object(&mut commands, server.as_ref(), object);
    // }

    for object in map_description.active_objects() {
        let object_type = match object.object_type() {
            mapdescr::ActiveObjectType::Base => objects::ActiveObjectType::Base,
            mapdescr::ActiveObjectType::PowerHub => objects::ActiveObjectType::PowerHub,
            mapdescr::ActiveObjectType::Attacker => objects::ActiveObjectType::Attacker,
        };

        spawn_events.send(ToBeSpawnedEvent::new(
            object.position().position(),
            object.position().rotation(),
            object_type,
            object.player(),
        ));
    }
}

fn finalize(
    mut commands: Commands,
    mut maps: ResMut<Assets<MapDescription>>,
    map_handle: Res<Handle<MapDescription>>,
    mut game_state: ResMut<State<GameStates>>,
) {
    commands.remove_resource::<Handle<MapDescription>>();
    maps.remove(map_handle.as_ref()).unwrap();
    game_state.set(GameStates::Playing).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, path::PathBuf};

    #[test]
    fn test_map_parsing() {
        let mut map_bytes = Vec::new();
        let mut test_map = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_map.push("test_data/test-map.tar");
        File::open(test_map)
            .unwrap()
            .read_to_end(&mut map_bytes)
            .unwrap();
        let map = load_from_slice(map_bytes.as_slice()).unwrap();
        assert_eq!(map.size.0, 108.1);
    }
}
