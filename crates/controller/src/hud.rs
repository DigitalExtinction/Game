use bevy::prelude::*;
use iyes_loopless::prelude::*;

use de_core::{
    screengeom::ScreenRect,
    stages::GameStage,
    state::{AppState, GameState},
};
use de_gui::TextProps;

use crate::{
    hud_components,
    hud_interaction::hud_button_system,
};

const SELECTION_BOX_COLOR: Color = Color::rgba(0., 0.5, 0.8, 0.2);

pub(crate) struct HudPlugin;

impl HudPlugin {
    fn place_draft_systems() -> SystemSet {
        SystemSet::new()
            .with_system(
                hud_button_system
                    .run_in_state(GameState::Playing),
            )
    }
}

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Playing, spawn_hud)
            .add_system_set_to_stage(GameStage::Input, Self::place_draft_systems());
        app.add_event::<UpdateSelectionBoxEvent>()
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

pub fn spawn_hud(
    mut commands: Commands,
    text_props: Res<TextProps>,
) {
    let font = text_props.as_ref();
    commands.spawn(
        NodeBundle {
            style: Style {
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd,
                position_type: PositionType::Absolute,
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                ..Style::default()
            },
            background_color: BackgroundColor::from(Color::NONE),
            ..default()
        }
    )
        .with_children(|commands| {
            hud_components::spawn_details(commands, font);
            hud_components::spawn_action_bar(commands, font);
            hud_components::spawn_map(commands, font);
        });
}
