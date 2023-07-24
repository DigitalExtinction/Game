use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use de_core::{cleanup::DespawnOnGameExit, gamestate::GameState, schedule::PreMovement};
use de_map::size::MapBounds;

use crate::hud::{interaction::InteractionBlocker, HUD_COLOR};

const MINIMAP_WIDTH: Val = Val::Percent(20.);
const MINIMAP_PADDING: Val = Val::Percent(1.);

pub(super) struct NodesPlugin;

impl Plugin for NodesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup)
            .add_systems(
                PreMovement,
                update_resolution.run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub(super) struct MinimapNode;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, map_bounds: Res<MapBounds>) {
    let handle = images.add(new_image(UVec2::splat(128)));
    let map_size = map_bounds.size();
    let aspect = map_size.x / map_size.y;

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    padding: UiRect::all(MINIMAP_PADDING),
                    width: MINIMAP_WIDTH,
                    right: Val::Px(0.),
                    bottom: Val::Px(0.),
                    ..default()
                },
                background_color: HUD_COLOR.into(),
                ..default()
            },
            InteractionBlocker,
            DespawnOnGameExit,
        ))
        .with_children(|parent| {
            parent
                .spawn(ImageBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        aspect_ratio: Some(aspect),
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
    commands.entity(entity).insert(UiImage::new(image));
}

/// Creates a new minimap image.
fn new_image(resolution: UVec2) -> Image {
    info!("Creating new minimap image with resolution {resolution:?}");

    let format = TextureFormat::Rgba8UnormSrgb;
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
