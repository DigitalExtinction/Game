use bevy::{
    asset::LoadState,
    core_pipeline::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};
use de_core::{gamestate::GameState, state::AppState};
use iyes_progress::prelude::*;

pub(crate) struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::AppLoading), load)
            .add_systems(
                Update,
                (
                    configure_cubemap
                        .track_progress()
                        .run_if(in_state(AppState::AppLoading)),
                    setup_camera.run_if(in_state(GameState::Loading)),
                ),
            );
    }
}

#[derive(Resource)]
struct SkyboxSource {
    handle: Handle<Image>,
    configured: bool,
}

fn load(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(SkyboxSource {
        handle: server.load("textures/skybox.png"),
        configured: false,
    });
}

fn configure_cubemap(
    server: Res<AssetServer>,
    mut source: ResMut<SkyboxSource>,
    mut images: ResMut<Assets<Image>>,
) -> Progress {
    if source.configured {
        return true.into();
    }

    match server.get_load_state(&source.handle) {
        Some(LoadState::Loaded) => (),
        Some(LoadState::NotLoaded) | Some(LoadState::Loading) => return false.into(),
        _ => panic!("Unexpected loading state."),
    }

    let image = images.get_mut(&source.handle).unwrap();
    image.reinterpret_stacked_2d_as_array(
        image.texture_descriptor.size.height / image.texture_descriptor.size.width,
    );
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });

    source.configured = true;
    true.into()
}

fn setup_camera(
    mut commands: Commands,
    skybox: ResMut<SkyboxSource>,
    camera_query: Query<(Entity, Has<Skybox>), With<Camera>>,
) {
    let (entity, configured) = camera_query.single();
    if configured {
        return;
    }
    commands.entity(entity).insert(Skybox {
        image: skybox.handle.clone(),
        brightness: 300.,
    });
}
