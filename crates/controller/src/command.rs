use bevy::{
    input::{mouse::MouseButtonInput, ElementState},
    prelude::*,
};
use de_core::{objects::MovableSolid, projection::ToFlat};
use de_pathing::UpdateEntityPath;
use iyes_loopless::prelude::*;

use crate::{
    pointer::Pointer,
    selection::{SelectEvent, Selected, SelectionMode},
    Labels,
};

pub(crate) struct CommandPlugin;

impl Plugin for CommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_system(
                    right_click_handler
                        .run_if(on_pressed(MouseButton::Right))
                        .label(Labels::InputUpdate)
                        .after(Labels::PreInputUpdate),
                )
                .with_system(
                    left_click_handler
                        .run_if(on_pressed(MouseButton::Left))
                        .label(Labels::InputUpdate)
                        .after(Labels::PreInputUpdate),
                ),
        );
    }
}

fn on_pressed(button: MouseButton) -> impl Fn(EventReader<MouseButtonInput>) -> bool {
    move |mut events: EventReader<MouseButtonInput>| {
        // It is desirable to exhaust the iterator, thus .filter().count() is
        // used instead of .any()
        events
            .iter()
            .filter(|e| e.button == button && e.state == ElementState::Pressed)
            .count()
            > 0
    }
}

fn right_click_handler(
    mut path_events: EventWriter<UpdateEntityPath>,
    selected: Query<Entity, (With<Selected>, With<MovableSolid>)>,
    pointer: Res<Pointer>,
) {
    let target = match pointer.terrain_point() {
        Some(point) => point.to_flat(),
        None => return,
    };

    for entity in selected.iter() {
        path_events.send(UpdateEntityPath::new(entity, target));
    }
}

fn left_click_handler(
    mut events: EventWriter<SelectEvent>,
    keys: Res<Input<KeyCode>>,
    pointer: Res<Pointer>,
) {
    let selection_mode = if keys.pressed(KeyCode::LControl) {
        SelectionMode::Add
    } else {
        SelectionMode::Replace
    };
    let event = match pointer.entity() {
        Some(entity) => SelectEvent::single(entity, selection_mode),
        None => SelectEvent::none(selection_mode),
    };
    events.send(event);
}
