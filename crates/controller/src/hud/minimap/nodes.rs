use bevy::{prelude::*, render::texture::TextureFormatPixelInfo};
use de_core::{stages::GameStage, state::GameState};
use iyes_loopless::prelude::*;
use wgpu_types::{Extent3d, TextureDimension, TextureFormat};

use crate::hud::{interaction::InteractionBlocker, HUD_COLOR};

pub(super) struct NodesPlugin;

impl Plugin for NodesPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Playing, setup)
            .add_system_to_stage(
                GameStage::PreMovement,
                update_resolution.run_in_state(GameState::Playing),
            );
    }
}

#[derive(Component)]
pub(super) struct MinimapNode;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let handle = images.add(new_image(UVec2::splat(128)));

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(20.),
                    height: Val::Percent(30.),
                },
                position_type: PositionType::Absolute,
                position: UiRect::new(
                    Val::Percent(80.),
                    Val::Percent(100.),
                    Val::Percent(70.),
                    Val::Percent(100.),
                ),
                padding: UiRect::all(Val::Percent(1.)),
                ..default()
            },
            background_color: HUD_COLOR.into(),
            ..default()
        })
        .insert(InteractionBlocker)
        .with_children(|parent| {
            parent
                .spawn(ImageBundle {
                    style: Style {
                        position: UiRect::all(Val::Percent(0.)),
                        size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                        ..default()
                    },
                    background_color: Color::WHITE.into(),
                    image: handle.into(),
                    ..default()
                })
                .insert(MinimapNode);
        });
}

type ChangedMinimap = (Changed<Node>, With<MinimapNode>);

fn update_resolution(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    query: Query<(Entity, &Node), ChangedMinimap>,
) {
    if query.is_empty() {
        return;
    }

    let (entity, node) = query.single();
    let resolution = node.size().round().as_uvec2();
    if resolution == UVec2::ZERO {
        return;
    }

    let image = images.add(new_image(resolution));
    commands.entity(entity).insert(UiImage(image));
}

/// Creates a new minimap image.
fn new_image(resolution: UVec2) -> Image {
    info!("Creating new minimap image with resolution {resolution:?}");

    let format = TextureFormat::Rgba8UnormSrgb;
    assert_eq!(format.pixel_size(), 4);
    let num_bytes = resolution.x as usize * resolution.y as usize * 4;
    let data = vec![255; num_bytes];
    Image::new(
        Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        format,
    )
}
