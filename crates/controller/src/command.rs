use bevy::{
    input::{mouse::MouseButtonInput, ElementState},
    prelude::*,
};
use de_core::{
    objects::{ActiveObjectType, MovableSolid},
    projection::ToFlat,
};
use de_pathing::UpdateEntityPath;
use de_spawner::Draft;
use iyes_loopless::prelude::*;

use crate::{
    draft::{NewDraftEvent, SpawnDraftsEvent},
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
                )
                .with_system(
                    key_press_handler
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
    mut select_events: EventWriter<SelectEvent>,
    mut draft_events: EventWriter<SpawnDraftsEvent>,
    keys: Res<Input<KeyCode>>,
    pointer: Res<Pointer>,
    drafts: Query<(), With<Draft>>,
) {
    if drafts.is_empty() {
        let selection_mode = if keys.pressed(KeyCode::LControl) {
            SelectionMode::Add
        } else {
            SelectionMode::Replace
        };
        let event = match pointer.entity() {
            Some(entity) => SelectEvent::single(entity, selection_mode),
            None => SelectEvent::none(selection_mode),
        };
        select_events.send(event);
    } else {
        draft_events.send(SpawnDraftsEvent);
    }
}

fn key_press_handler(
    keys: Res<Input<KeyCode>>,
    pointer: Res<Pointer>,
    mut events: EventWriter<NewDraftEvent>,
) {
    let key = match keys.get_just_pressed().last() {
        Some(key) => key,
        None => return,
    };
    let object_type = match key {
        KeyCode::B => ActiveObjectType::Base,
        KeyCode::P => ActiveObjectType::PowerHub,
        _ => return,
    };
    let point = pointer.terrain_point().unwrap_or(Vec3::ZERO);
    events.send(NewDraftEvent::new(point, object_type));
}
