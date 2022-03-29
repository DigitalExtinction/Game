use super::file::{load_from_slice, MapDescription};
use crate::{game::GameConfig, states::GameStates};
use anyhow::Context;
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadState, LoadedAsset},
    pbr::{PbrBundle, StandardMaterial},
    prelude::{shape::Plane, *},
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameConfig::new("map.tar"))
            .add_asset::<MapDescription>()
            .add_asset_loader(MapLoader)
            .add_system_set(SystemSet::on_enter(GameStates::MapLoading).with_system(load_map))
            .add_system_set(SystemSet::on_update(GameStates::MapLoading).with_system(spawn_map));
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

fn load_map(mut commands: Commands, server: Res<AssetServer>, game_config: Res<GameConfig>) {
    let handle: Handle<MapDescription> = server.load(game_config.map_path());
    commands.insert_resource(handle);
}

fn spawn_map(
    mut commands: Commands,
    server: Res<AssetServer>,
    map_handle: Res<Handle<MapDescription>>,
    mut maps: ResMut<Assets<MapDescription>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut app_state: ResMut<State<GameStates>>,
) {
    match server.get_load_state(map_handle.as_ref()) {
        LoadState::Failed => panic!("Map loading has failed."),
        LoadState::Loaded => {
            commands.remove_resource::<Handle<MapDescription>>();

            let map_description = maps.remove(map_handle.as_ref()).unwrap();
            commands.insert_resource(map_description.size);

            let map_size = map_description.size.0;
            commands.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(Plane { size: map_size })),
                material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                transform: Transform {
                    translation: Vec3::new(map_size / 2., 0., map_size / 2.),
                    ..Default::default()
                },
                ..Default::default()
            });

            app_state.set(GameStates::InGame).unwrap();
        }
        _ => (),
    }
}
