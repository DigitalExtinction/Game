use bevy::{prelude::*, render::texture::TextureFormatPixelInfo};
use de_core::state::GameState;
use iyes_loopless::prelude::*;
use wgpu_types::{Extent3d, TextureDimension, TextureFormat};

use crate::hud::{interaction::InteractionBlocker, minimap::MapImageHandle, HUD_COLOR};

pub(super) struct NodesPlugin;

impl Plugin for NodesPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Playing, setup);
    }
}

#[derive(Component)]
pub(super) struct MinimapNode;

fn setup(mut commands: Commands, windows: Res<Windows>, mut images: ResMut<Assets<Image>>) {
    // Multiple of screen size
    let node_size = Vec2::new(0.2, 0.3);
    let padding = 0.01;

    let window = windows.get_primary().unwrap();
    let win_size = Vec2::new(window.width(), window.height());
    let minimap_resolution = ((node_size - Vec2::splat(2. * padding)) * win_size)
        .round()
        .as_uvec2();

    info!("Setting minimap resolution to {minimap_resolution:?}");

    let format = TextureFormat::Rgba8UnormSrgb;
    assert_eq!(format.pixel_size(), 4);
    let num_bytes = (minimap_resolution.x * minimap_resolution.y) as usize * 4;
    let data = vec![255; num_bytes];
    let image = Image::new(
        Extent3d {
            width: minimap_resolution.x,
            height: minimap_resolution.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        format,
    );

    let handle = images.add(image);
    commands.insert_resource(MapImageHandle::from(handle.clone()));

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(20.),
                    height: Val::Percent(30.),
                },
                position_type: PositionType::Absolute,
                position: UiRect::new(
                    Val::Percent(100. - 100. * node_size.x),
                    Val::Percent(100.),
                    Val::Percent(100. - 100. * node_size.y),
                    Val::Percent(100.),
                ),
                padding: UiRect::all(Val::Percent(100. * padding)),
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
