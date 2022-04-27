use super::{
    mapdescr::{ActiveObjectType, MapDescription, MapObject},
    terrain::Terrain,
    GameState,
};
use anyhow::{bail, Context};
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadState, LoadedAsset},
    ecs::system::EntityCommands,
    pbr::{PbrBundle, StandardMaterial},
    prelude::{shape::Plane, *},
};
use de_core::{
    gconfig::GameConfig,
    objects::{Active, Movable, Playable, SolidObject},
};
use iyes_loopless::prelude::*;
use std::io::Read;
use tar::Archive;

pub struct MapLoaderPlugin;

impl Plugin for MapLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<MapDescription>()
            .add_asset_loader(MapLoader)
            .add_enter_system(GameState::Loading, load_map)
            .add_system(
                setup_map
                    .run_in_state(GameState::Loading)
                    .run_if_resource_exists::<Handle<MapDescription>>(),
            );
    }
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

fn load_map(mut commands: Commands, server: Res<AssetServer>, game_config: Res<GameConfig>) {
    let handle: Handle<MapDescription> = server.load(game_config.map_path());
    commands.insert_resource(handle);
}

fn setup_map(
    mut commands: Commands,
    server: Res<AssetServer>,
    map_handle: Res<Handle<MapDescription>>,
    mut map_assets: ResMut<Assets<MapDescription>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_config: Res<GameConfig>,
) {
    let map = match server.get_load_state(map_handle.as_ref()) {
        LoadState::Failed => panic!("Map loading has failed."),
        LoadState::Loaded => map_assets.get(map_handle.as_ref()).unwrap(),
        _ => return,
    };

    commands.insert_resource(map.size);

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
                right: map.size.0,
                bottom: 0.,
                top: map.size.0,
                near: -10.,
                far: 2. * map.size.0,
                ..Default::default()
            },
            shadow_depth_bias: 0.2,
            shadow_normal_bias: 0.2,
            shadows_enabled: true,
        },
        transform,
        ..Default::default()
    });

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(Plane { size: map.size.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform {
                translation: Vec3::new(map.size.0 / 2., 0., map.size.0 / 2.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Terrain);

    for object in map.inactive_objects() {
        spawn_object(&mut commands, server.as_ref(), object);
    }
    for object in map.active_objects() {
        let mut entity_commands = spawn_object(&mut commands, server.as_ref(), object);
        entity_commands.insert(Active);
        if object.player() == game_config.player() {
            entity_commands.insert(Playable);
        }
        if object.object_type() == ActiveObjectType::Attacker {
            entity_commands.insert(Movable);
        }
    }

    commands.remove_resource::<Handle<MapDescription>>();
    map_assets.remove(map_handle.as_ref()).unwrap();
    commands.insert_resource(NextState(GameState::Playing));
}

fn spawn_object<'w, 's, 'a, 'b, O>(
    commands: &'a mut Commands<'w, 's>,
    server: &AssetServer,
    object: &O,
) -> EntityCommands<'w, 's, 'a>
where
    O: 'b + MapObject,
{
    let bundle = (
        GlobalTransform::identity(),
        object.position().transform(),
        SolidObject,
    );
    let gltf = server.load(&format!("{}.glb#Scene0", object.model_name()));
    let mut entity_commands = commands.spawn_bundle(bundle);
    entity_commands.with_children(|parent| {
        parent.spawn_scene(gltf);
    });
    entity_commands
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
