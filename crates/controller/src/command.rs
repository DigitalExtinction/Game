use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use de_attacking::AttackEvent;
use de_behaviour::ChaseTarget;
use de_core::{
    gconfig::GameConfig,
    objects::{BuildingType, MovableSolid, Playable, PLAYER_MAX_BUILDINGS},
    player::Player,
    projection::ToFlat,
    stages::GameStage,
};
use de_pathing::{PathQueryProps, PathTarget, UpdateEntityPath};
use de_spawner::{Draft, ObjectCounter};
use enum_map::enum_map;
use iyes_loopless::prelude::*;

use crate::{
    draft::{DiscardDraftsEvent, NewDraftEvent, SpawnDraftsEvent},
    keyboard::KeyCondition,
    pointer::Pointer,
    selection::{SelectEvent, Selected, SelectionMode},
    Labels,
};

pub(crate) struct CommandPlugin;

impl CommandPlugin {
    fn place_draft_systems() -> SystemSet {
        let key_map = enum_map! {
            BuildingType::Base => KeyCode::B,
            BuildingType::PowerHub => KeyCode::P,
        };
        key_map
            .iter()
            .fold(SystemSet::new(), |systems, (building_type, &key)| {
                systems.with_system(
                    place_draft(building_type)
                        .run_if(KeyCondition::single(key).build())
                        .label(Labels::InputUpdate)
                        .after(Labels::PreInputUpdate),
                )
            })
    }
}

impl Plugin for CommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            GameStage::Input,
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
                    discard_drafts
                        .run_if(KeyCondition::single(KeyCode::Escape).build())
                        .label(Labels::InputUpdate)
                        .after(Labels::PreInputUpdate),
                )
                .with_system(
                    select_all
                        .run_if(KeyCondition::with_ctrl(KeyCode::A).build())
                        .label(Labels::InputUpdate)
                        .after(Labels::PreInputUpdate),
                ),
        )
        .add_system_set_to_stage(GameStage::Input, Self::place_draft_systems());
    }
}

fn on_pressed(button: MouseButton) -> impl Fn(EventReader<MouseButtonInput>) -> bool {
    move |mut events: EventReader<MouseButtonInput>| {
        // It is desirable to exhaust the iterator, thus .filter().count() is
        // used instead of .any()
        events
            .iter()
            .filter(|e| e.button == button && e.state == ButtonState::Pressed)
            .count()
            > 0
    }
}

type SelectedQuery<'w, 's> =
    Query<'w, 's, (Entity, Option<&'static ChaseTarget>), (With<Selected>, With<MovableSolid>)>;

fn right_click_handler(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut path_events: EventWriter<UpdateEntityPath>,
    mut attack_events: EventWriter<AttackEvent>,
    selected: SelectedQuery,
    targets: Query<&Player>,
    pointer: Res<Pointer>,
) {
    match pointer.entity().filter(|&entity| {
        targets
            .get(entity)
            .map(|&player| !config.is_local_player(player))
            .unwrap_or(false)
    }) {
        Some(enemy) => {
            for (attacker, _) in selected.iter() {
                attack_events.send(AttackEvent::new(attacker, enemy));
            }
        }
        None => {
            let target = match pointer.terrain_point() {
                Some(point) => point.to_flat(),
                None => return,
            };

            for (entity, chase) in selected.iter() {
                if chase.is_some() {
                    commands.entity(entity).remove::<ChaseTarget>();
                }

                path_events.send(UpdateEntityPath::new(
                    entity,
                    PathTarget::new(target, PathQueryProps::exact(), false),
                ));
            }
        }
    }
}

fn left_click_handler(
    mut select_events: EventWriter<SelectEvent>,
    mut draft_events: EventWriter<SpawnDraftsEvent>,
    keys: Res<Input<KeyCode>>,
    pointer: Res<Pointer>,
    playable: Query<(), With<Playable>>,
    drafts: Query<(), With<Draft>>,
) {
    if drafts.is_empty() {
        let selection_mode = if keys.pressed(KeyCode::LControl) {
            SelectionMode::AddToggle
        } else {
            SelectionMode::Replace
        };

        let event = match pointer.entity().filter(|&e| playable.contains(e)) {
            Some(entity) => SelectEvent::single(entity, selection_mode),
            None => SelectEvent::none(selection_mode),
        };
        select_events.send(event);
    } else {
        draft_events.send(SpawnDraftsEvent);
    }
}

fn discard_drafts(mut events: EventWriter<DiscardDraftsEvent>) {
    events.send(DiscardDraftsEvent);
}

fn place_draft(
    building_type: BuildingType,
) -> impl Fn(Res<ObjectCounter>, Res<Pointer>, EventWriter<NewDraftEvent>) {
    move |counter: Res<ObjectCounter>,
          pointer: Res<Pointer>,
          mut events: EventWriter<NewDraftEvent>| {
        if counter.building_count() >= PLAYER_MAX_BUILDINGS {
            warn!("Maximum number of buildings reached.");
            return;
        }

        let point = match pointer.terrain_point() {
            Some(point) => point,
            None => return,
        };
        events.send(NewDraftEvent::new(point, building_type));
    }
}

fn select_all(
    playable: Query<Entity, (With<Playable>, Without<Selected>)>,
    mut events: EventWriter<SelectEvent>,
) {
    let entities = playable.iter().collect();
    events.send(SelectEvent::many(entities, SelectionMode::AddToggle))
}
