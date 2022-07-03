use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ElementState},
    prelude::*,
};
use de_behaviour::AttackTarget;
use de_core::{
    gconfig::GameConfig,
    objects::{BuildingType, MovableSolid},
    player::Player,
    projection::ToFlat,
};
use de_pathing::{PathTarget, UpdateEntityPath};
use de_spawner::Draft;
use enum_map::enum_map;
use iyes_loopless::prelude::*;

use crate::{
    draft::{DiscardDraftsEvent, NewDraftEvent, SpawnDraftsEvent},
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
    mut commands: Commands,
    config: Res<GameConfig>,
    mut path_events: EventWriter<UpdateEntityPath>,
    selected: Query<(Entity, Option<&AttackTarget>), (With<Selected>, With<MovableSolid>)>,
    targets: Query<&Player>,
    pointer: Res<Pointer>,
) {
    match pointer.entity().filter(|&entity| {
        targets
            .get(entity)
            .map(|&player| config.is_local_player(player))
            .unwrap_or(false)
    }) {
        Some(enemy) => {
            info!("Commanding selected entities to attack {:?}", enemy);
            for (entity, _) in selected.iter() {
                // TODO what if the the entity does not have attack capability?
                // TODO what if the entity gets destroyed in the meantime?
                commands.entity(entity).insert(AttackTarget::new(enemy));
            }
        }
        None => {
            let target = match pointer.terrain_point() {
                Some(point) => point.to_flat(),
                None => return,
            };

            for (entity, attack) in selected.iter() {
                if attack.is_some() {
                    // TODO what if the entity gets destroyed in the meantime?
                    commands.entity(entity).remove::<AttackTarget>();
                }
                path_events.send(UpdateEntityPath::new(entity, PathTarget::exact(target)));
            }
        }
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
    mut key_events: EventReader<KeyboardInput>,
    pointer: Res<Pointer>,
    mut new_draft_events: EventWriter<NewDraftEvent>,
    mut discard_drafts_events: EventWriter<DiscardDraftsEvent>,
) {
    let key = match key_events
        .iter()
        .filter(|e| e.state == ElementState::Pressed)
        .last()
    {
        Some(event) => match event.key_code {
            Some(key) => key,
            None => return,
        },
        None => return,
    };

    if key == KeyCode::Escape {
        discard_drafts_events.send(DiscardDraftsEvent);
        return;
    }

    let point = match pointer.terrain_point() {
        Some(point) => point,
        None => return,
    };

    let key_map = enum_map! {
        BuildingType::Base => KeyCode::B,
        BuildingType::PowerHub => KeyCode::P,
    };

    if let Some(building_type) = key_map.iter().find_map(|(building_type, &associated_key)| {
        if associated_key == key {
            Some(building_type)
        } else {
            None
        }
    }) {
        new_draft_events.send(NewDraftEvent::new(point, building_type));
    }
}
