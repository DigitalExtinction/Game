use bevy::prelude::*;
use de_core::{cleanup::DespawnOnGameExit, gamestate::GameState, screengeom::ScreenRect};

const SELECTION_BOX_COLOR: Color = Color::rgba(0., 0.5, 0.8, 0.2);

pub(crate) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateSelectionBoxEvent>().add_systems(
            PostUpdate,
            process_events.run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Event)]
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
                let width = Val::Percent(50. * size.x);
                let height = Val::Percent(50. * size.y);
                let left = Val::Percent(50. * (rect.left() + 1.));
                let top = Val::Percent(50. * (1. - rect.top()));

                match boxes.get_single_mut() {
                    Ok((_, mut style)) => {
                        style.width = width;
                        style.height = height;
                        style.left = left;
                        style.top = top;
                    }
                    Err(_) => {
                        assert!(boxes.is_empty());

                        commands.spawn((
                            NodeBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    width,
                                    height,
                                    left,
                                    top,
                                    ..Default::default()
                                },
                                background_color: BackgroundColor(SELECTION_BOX_COLOR),
                                ..Default::default()
                            },
                            SelectionBox,
                            DespawnOnGameExit,
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
