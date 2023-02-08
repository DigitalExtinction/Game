use bevy::{
    asset::LoadState,
    prelude::*,
    render::{
        render_resource::{AddressMode, SamplerDescriptor},
        texture::ImageSampler,
    },
};
use de_core::{gamestate::GameState, stages::GameStage, state::AppState};
use iyes_loopless::prelude::*;
use iyes_progress::prelude::*;

use crate::{shader::TerrainMaterial, terrain::Terrain};

const TERRAIN_TEXTURE: &str = "textures/terrain.png";

pub(crate) struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<TerrainMaterial>::default())
            .add_enter_system(AppState::InGame, load)
            .add_exit_system(AppState::InGame, cleanup)
            .add_system(
                setup_textures
                    .track_progress()
                    .run_in_state(GameState::Loading),
            )
            .add_system_to_stage(GameStage::Update, init.run_in_state(AppState::InGame));
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
        LoadState::NotLoaded => false.into(),
        LoadState::Loading => false.into(),
        LoadState::Failed => panic!("Texture loading has failed."),
        LoadState::Unloaded => unreachable!(),
        LoadState::Loaded => {
            // Ideally, this setup would happen in some kind of asset post
            // processing. This is however not yet supported by Bevy.
            //
            // https://github.com/bevyengine/bevy/discussions/3972
            let mut image = images.get_mut(&textures.0).unwrap();
            image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                ..Default::default()
            });

            true.into()
        }
    }
}

fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    textures: Res<Textures>,
    uninitialized: Query<(Entity, &Terrain, &Transform), Without<Handle<Mesh>>>,
) {
    for (entity, terrain, transform) in uninitialized.iter() {
        commands.entity(entity).insert(MaterialMeshBundle {
            mesh: meshes.add(terrain.generate_mesh(transform.translation)),
            material: materials.add(TerrainMaterial::new(textures.0.clone())),
            transform: *transform,
            ..Default::default()
        });
    }
}
