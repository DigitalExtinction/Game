use bevy::prelude::*;
use de_core::{screengeom::ScreenRect, stages::GameStage, state::AppState};
use iyes_loopless::prelude::*;

const SELECTION_BOX_COLOR: Color = Color::rgba(0., 0.5, 0.8, 0.2);

pub(crate) struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
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

                        commands
                            .spawn(NodeBundle {
                                style: Style {
                                    size: ui_size,
                                    position: ui_rect,
                                    ..Default::default()
                                },
                                background_color: BackgroundColor(SELECTION_BOX_COLOR),
                                ..Default::default()
                            })
                            .insert(SelectionBox);
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
