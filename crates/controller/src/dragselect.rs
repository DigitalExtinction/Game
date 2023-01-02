use bevy::prelude::*;
use de_core::{stages::GameStage, state::AppState};
use de_hud::UpdateSelectionBoxEvent;
use iyes_loopless::prelude::*;

use crate::{
    areaselect::{AreaSelectLabels, SelectInRectEvent},
    mouse::{DragUpdateType, MouseDragged, MouseLabels},
    selection::SelectionMode,
};

pub(crate) struct DragSelectPlugin;

impl Plugin for DragSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            GameStage::Input,
            SystemSet::new().with_system(
                update_drags
                    .run_in_state(AppState::InGame)
                    .before(AreaSelectLabels::SelectInArea)
                    .after(MouseLabels::Buttons),
            ),
        );
    }
}

fn update_drags(
    keys: Res<Input<KeyCode>>,
    mut drag_events: EventReader<MouseDragged>,
    mut ui_events: EventWriter<UpdateSelectionBoxEvent>,
    mut select_events: EventWriter<SelectInRectEvent>,
) {
    for drag_event in drag_events.iter() {
        if drag_event.button() != MouseButton::Left {
            continue;
        }

        let ui_event = match drag_event.update_type() {
            DragUpdateType::Moved => UpdateSelectionBoxEvent::from_rect(drag_event.rect()),
            DragUpdateType::Released => {
                let mode = if keys.pressed(KeyCode::LControl) || keys.pressed(KeyCode::RControl) {
                    SelectionMode::Add
                } else {
                    SelectionMode::Replace
                };
                select_events.send(SelectInRectEvent::new(drag_event.rect(), mode));

                UpdateSelectionBoxEvent::none()
            }
        };

        ui_events.send(ui_event)
    }
}
