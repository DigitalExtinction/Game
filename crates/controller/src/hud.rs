use bevy::{ecs::system::SystemParam, prelude::*};
use de_core::{
    screengeom::ScreenRect,
    stages::GameStage,
    state::{AppState, GameState},
};
use glam::Vec3Swizzles;
use iyes_loopless::prelude::*;

use crate::hud_components;

const SELECTION_BOX_COLOR: Color = Color::rgba(0., 0.5, 0.8, 0.2);

pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateSelectionBoxEvent>()
            .add_enter_system(GameState::Playing, spawn_hud)
            .add_system_set_to_stage(
                GameStage::PostUpdate,
                SystemSet::new().with_system(process_events.run_in_state(AppState::InGame)),
            );
    }
}

pub struct UpdateSelectionBoxEvent(Option<ScreenRect>);

impl UpdateSelectionBoxEvent {
    pub fn none() -> Self {
        Self(None)
    }

    pub fn from_rect(rect: ScreenRect) -> Self {
        Self(Some(rect))
    }
}

#[derive(Component)]
struct SelectionBox;

fn process_events(
    mut commands: Commands,
    mut boxes: Query<(Entity, &mut Style), With<SelectionBox>>,
    mut events: EventReader<UpdateSelectionBoxEvent>,
) {
    if let Some(event) = events.iter().last() {
        match event.0 {
            Some(rect) => {
                let size = rect.size();
                let ui_size = Size::new(Val::Percent(50. * size.x), Val::Percent(50. * size.y));
                let ui_rect = UiRect {
                    left: Val::Percent(50. * (rect.left() + 1.)),
                    top: Val::Percent(50. * (1. - rect.top())),
                    ..Default::default()
                };

                match boxes.get_single_mut() {
                    Ok((_, mut style)) => {
                        style.size = ui_size;
                        style.position = ui_rect;
                    }
                    Err(_) => {
                        assert!(boxes.is_empty());

                        commands.spawn((
                            NodeBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    size: ui_size,
                                    position: ui_rect,
                                    ..Default::default()
                                },
                                background_color: BackgroundColor(SELECTION_BOX_COLOR),
                                ..Default::default()
                            },
                            SelectionBox,
                        ));
                    }
                }
            }
            None => {
                for (entity, _) in boxes.iter() {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

pub fn spawn_hud(mut commands: Commands) {
    hud_components::spawn_details(&mut commands);
    hud_components::spawn_action_bar(&mut commands);
    hud_components::spawn_map(&mut commands);
}

/// Top-level non-transparent UI node. All such nodes are marked with this component and no descendants have it attached
#[derive(Component)]
pub struct HudTopVisibleNode;

#[derive(SystemParam)]
pub(crate) struct HudNodes<'w, 's> {
    hud: Query<'w, 's, (&'static GlobalTransform, &'static Node), With<HudTopVisibleNode>>,
    windows: Res<'w, Windows>,
}

impl<'w, 's> HudNodes<'w, 's> {
    pub(crate) fn contains_point(&mut self, point: &Vec2) -> bool {
        let window = self.windows.get_primary().unwrap();
        let win_size = Vec2::new(window.width(), window.height());
        self.hud.iter().any(|(box_transform, node)| {
            // WARNING: This is because mouse y starts on bottom, GlobalTransform on top
            let mouse_position = Vec2::new(point.x, win_size.y - point.y);

            let box_size = node.size();
            let box_transform: Vec3 = box_transform.translation();
            // WARNING: This is because GlobalTransform is centered, width/2 to left and to right, same on vertical
            let box_position = box_transform.xy() - box_size / 2.;

            mouse_position.cmpge(box_position).all()
                && mouse_position.cmple(box_position + box_size).all()
        })
    }
}
