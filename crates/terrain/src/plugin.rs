use bevy::{
    asset::LoadState,
    pbr::ExtendedMaterial,
    prelude::*,
    render::texture::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
};
use de_core::{gamestate::GameState, state::AppState};
use iyes_progress::prelude::*;

use crate::{
    shader::{TerrainMaterial, UV_SCALE},
    terrain::Terrain,
};

const TERRAIN_TEXTURE: &str = "textures/terrain.png";

pub(crate) struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, TerrainMaterial>,
        >::default())
            .add_systems(OnEnter(AppState::InGame), load)
            .add_systems(OnExit(AppState::InGame), cleanup)
            .add_systems(
                Update,
                (
                    setup_textures
                        .track_progress()
                        .run_if(in_state(GameState::Loading)),
                    init.run_if(in_state(AppState::InGame)),
                ),
            );
    }
}

#[derive(Resource)]
struct Textures(Handle<Image>);

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Textures>();
}

fn load(mut commands: Commands, server: Res<AssetServer>) {
    let handle = server.load(TERRAIN_TEXTURE);
    commands.insert_resource(Textures(handle));
}

fn setup_textures(
    server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    textures: Option<Res<Textures>>,
) -> Progress {
    let textures = match textures {
        Some(textures) => textures,
        None => return false.into(),
    };

    match server.get_load_state(&textures.0) {
        Some(load_state) => match load_state {
            LoadState::NotLoaded => false.into(),
            LoadState::Loading => false.into(),
            LoadState::Failed => panic!("Texture loading has failed."),
            LoadState::Loaded => {
                // Ideally, this setup would happen in some kind of asset post
                // processing. This was not supported by Bevy at the time of
                // implementation.
                //
                // https://github.com/bevyengine/bevy/discussions/3972
                let image = images.get_mut(&textures.0).unwrap();
                image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..Default::default()
                });

                true.into()
            }
        },
        None => panic!("Terrain texture asset unknown."),
    }
}

fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, TerrainMaterial>>>,
    textures: Res<Textures>,
    uninitialized: Query<(Entity, &Terrain, &Transform), Without<Handle<Mesh>>>,
) {
    for (entity, terrain, transform) in uninitialized.iter() {
        commands.entity(entity).insert(MaterialMeshBundle {
            mesh: meshes.add(terrain.generate_mesh(transform.translation)),
            material: materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color_texture: Some(textures.0.clone()),
                    perceptual_roughness: 0.8,
                    metallic: 0.23,
                    reflectance: 0.06,
                    ..default()
                },
                extension: TerrainMaterial::new(UV_SCALE),
            }),
            transform: *transform,
            ..Default::default()
        });
    }
}
