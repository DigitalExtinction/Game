//! This module implements user input / user command handling, for example
//! keyboard shortcuts, mouse actions events, and so on.

use bevy::prelude::*;
use de_behaviour::ChaseTarget;
use de_combat::AttackEvent;
use de_core::{
    gconfig::GameConfig,
    objects::{BuildingType, MovableSolid, ObjectType, Playable, PLAYER_MAX_BUILDINGS},
    player::Player,
    projection::ToFlat,
    screengeom::ScreenRect,
    stages::GameStage,
    state::GameState,
};
use de_pathing::{PathQueryProps, PathTarget, UpdateEntityPath};
use de_spawner::{Draft, ObjectCounter};
use enum_map::enum_map;
use iyes_loopless::prelude::*;

use super::keyboard::KeyCondition;
use crate::{
    draft::{DiscardDraftsEvent, DraftLabels, NewDraftEvent, SpawnDraftsEvent},
    hud::{GameMenuLabel, ToggleGameMenu, UpdateSelectionBoxEvent},
    mouse::{
        DragUpdateType, MouseClicked, MouseDoubleClicked, MouseDragged, MouseLabels, Pointer,
        PointerLabels,
    },
    selection::{
        AreaSelectLabels, SelectEvent, SelectInRectEvent, Selected, SelectionLabels, SelectionMode,
    },
};

pub(super) struct HandlersPlugin;

impl HandlersPlugin {
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
                        .run_in_state(GameState::Playing)
                        .run_if(KeyCondition::single(key).build())
                        .before(DraftLabels::New)
                        .after(PointerLabels::Update),
                )
            })
    }
}

impl Plugin for HandlersPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            GameStage::Input,
            SystemSet::new()
                .with_system(
                    right_click_handler
                        .run_in_state(GameState::Playing)
                        .run_if(on_click(MouseButton::Right))
                        .after(PointerLabels::Update)
                        .after(MouseLabels::Buttons),
                )
                .with_system(
                    left_click_handler
                        .run_in_state(GameState::Playing)
                        .run_if(on_click(MouseButton::Left))
                        .label(CommandLabel::LeftClick)
                        .before(SelectionLabels::Update)
                        .before(DraftLabels::Spawn)
                        .after(PointerLabels::Update)
                        .after(MouseLabels::Buttons),
                )
                .with_system(
                    double_click_handler
                        .run_in_state(GameState::Playing)
                        .run_if(on_double_click(MouseButton::Left))
                        .before(SelectionLabels::Update)
                        .before(DraftLabels::Spawn)
                        .after(PointerLabels::Update)
                        .after(MouseLabels::Buttons)
                        .after(CommandLabel::LeftClick),
                )
                .with_system(
                    handle_escape
                        .run_in_state(GameState::Playing)
                        .run_if(KeyCondition::single(KeyCode::Escape).build())
                        .before(GameMenuLabel::Toggle)
                        .before(DraftLabels::Discard),
                )
                .with_system(
                    select_all
                        .run_in_state(GameState::Playing)
                        .run_if(KeyCondition::single(KeyCode::A).with_ctrl().build())
                        .before(SelectionLabels::Update),
                )
                .with_system(
                    select_all_visible
                        .run_in_state(GameState::Playing)
                        .run_if(
                            KeyCondition::single(KeyCode::A)
                                .with_ctrl()
                                .with_shift()
                                .build(),
                        )
                        .before(AreaSelectLabels::SelectInArea),
                )
                .with_system(
                    update_drags
                        .run_in_state(GameState::Playing)
                        .before(AreaSelectLabels::SelectInArea)
                        .after(MouseLabels::Buttons),
                ),
        )
        .add_system_set_to_stage(GameStage::Input, Self::place_draft_systems());
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum CommandLabel {
    LeftClick,
}

fn on_click(button: MouseButton) -> impl Fn(EventReader<MouseClicked>) -> bool {
    move |mut events: EventReader<MouseClicked>| {
        // It is desirable to exhaust the iterator, thus .filter().count() is
        // used instead of .any()
        events.iter().filter(|e| e.button() == button).count() > 0
    }
}

fn on_double_click(button: MouseButton) -> impl Fn(EventReader<MouseDoubleClicked>) -> bool {
    move |mut events: EventReader<MouseDoubleClicked>| {
        // It is desirable to exhaust the iterator, thus .filter().count() is
        // used instead of .any()
        events.iter().filter(|e| e.button() == button).count() > 0
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

fn double_click_handler(
    keys: Res<Input<KeyCode>>,
    pointer: Res<Pointer>,
    playable: Query<&ObjectType, With<Playable>>,
    drafts: Query<(), With<Draft>>,
    mut select_in_rect_events: EventWriter<SelectInRectEvent>,
) {
    if !drafts.is_empty() {
        return;
    }
    let selection_mode = if keys.pressed(KeyCode::LControl) {
        SelectionMode::Add
    } else {
        SelectionMode::Replace
    };

    let Some(targeted_entity_type) = pointer.entity().and_then(|entity| playable.get(entity).ok()) else {
        return;
    };

    // Select all the units visible of the same type as the targeted entity
    select_in_rect_events.send(SelectInRectEvent::new(
        ScreenRect::full(),
        selection_mode,
        Some(*targeted_entity_type),
    ));
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

fn handle_escape(
    mut toggle_menu_events: EventWriter<ToggleGameMenu>,
    mut discard_events: EventWriter<DiscardDraftsEvent>,
    drafts: Query<(), With<Draft>>,
) {
    if drafts.is_empty() {
        toggle_menu_events.send(ToggleGameMenu);
    } else {
        discard_events.send(DiscardDraftsEvent);
    }
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
    events.send(SelectEvent::many(entities, SelectionMode::AddToggle));
}

fn select_all_visible(mut events: EventWriter<SelectInRectEvent>) {
    events.send(SelectInRectEvent::new(
        ScreenRect::full(),
        SelectionMode::Replace,
        None,
    ));
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
                select_events.send(SelectInRectEvent::new(drag_event.rect(), mode, None));

                UpdateSelectionBoxEvent::none()
            }
        };

        ui_events.send(ui_event)
    }
}
