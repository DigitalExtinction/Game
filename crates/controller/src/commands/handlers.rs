//! This module implements user input / user command handling, for example
//! keyboard shortcuts, mouse actions events, and so on.

use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};
use de_camera::{CameraLabel, RotateCameraEvent, TiltCameraEvent, ZoomCameraEvent};
use de_conf::Configuration;
use de_core::{
    gconfig::GameConfig,
    objects::{BuildingType, ObjectType, Playable, PLAYER_MAX_BUILDINGS},
    player::Player,
    projection::ToFlat,
    screengeom::ScreenRect,
    stages::GameStage,
    state::GameState,
};
use de_spawner::{Draft, ObjectCounter};
use enum_map::enum_map;
use iyes_loopless::prelude::*;

use super::{keyboard::KeyCondition, CommandsLabel, GroupAttackEvent, SendSelectedEvent};
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
                        .after(MouseLabels::Buttons)
                        .before(CommandsLabel::SendSelected)
                        .before(CommandsLabel::Attack),
                )
                .with_system(
                    left_click_handler
                        .run_in_state(GameState::Playing)
                        .run_if(on_click(MouseButton::Left))
                        .label(HandlersLabel::LeftClick)
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
                        .after(HandlersLabel::LeftClick),
                )
                .with_system(
                    zoom_camera
                        .run_in_state(GameState::Playing)
                        .before(CameraLabel::ZoomEvent),
                )
                .with_system(
                    pivot_camera
                        .run_in_state(GameState::Playing)
                        .before(CameraLabel::RotateEvent)
                        .before(CameraLabel::TiltEvent),
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
pub(crate) enum HandlersLabel {
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

fn right_click_handler(
    config: Res<GameConfig>,
    mut send_events: EventWriter<SendSelectedEvent>,
    mut attack_events: EventWriter<GroupAttackEvent>,
    targets: Query<&Player>,
    pointer: Res<Pointer>,
) {
    match pointer.entity().filter(|&entity| {
        targets
            .get(entity)
            .map(|&player| !config.is_local_player(player))
            .unwrap_or(false)
    }) {
        Some(enemy) => attack_events.send(GroupAttackEvent::new(enemy)),
        None => {
            let Some(target) = pointer.terrain_point().map(|p| p.to_flat()) else { return };
            send_events.send(SendSelectedEvent::new(target));
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

fn zoom_camera(
    conf: Res<Configuration>,
    mut wheel_events: EventReader<MouseWheel>,
    mut zoom_events: EventWriter<ZoomCameraEvent>,
) {
    let conf = conf.camera();
    let factor = wheel_events
        .iter()
        .fold(1.0, |factor, event| match event.unit {
            MouseScrollUnit::Line => factor * conf.wheel_zoom_sensitivity().powf(event.y),
            MouseScrollUnit::Pixel => factor * conf.touchpad_zoom_sensitivity().powf(event.y),
        });
    zoom_events.send(ZoomCameraEvent::new(factor));
}

fn pivot_camera(
    conf: Res<Configuration>,
    buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut mouse_event: EventReader<MouseMotion>,
    mut rotate_event: EventWriter<RotateCameraEvent>,
    mut tilt_event: EventWriter<TiltCameraEvent>,
) {
    if !buttons.pressed(MouseButton::Middle) && !keys.pressed(KeyCode::LShift) {
        return;
    }

    let delta = mouse_event.iter().fold(Vec2::ZERO, |sum, e| sum + e.delta);
    let sensitivity = conf.camera().rotation_sensitivity();
    if delta.x != 0. {
        rotate_event.send(RotateCameraEvent::new(sensitivity * delta.x));
    }
    if delta.y != 0. {
        tilt_event.send(TiltCameraEvent::new(-sensitivity * delta.y));
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
            DragUpdateType::Moved => match drag_event.rect() {
                Some(rect) => UpdateSelectionBoxEvent::from_rect(rect),
                None => UpdateSelectionBoxEvent::none(),
            },
            DragUpdateType::Released => {
                if let Some(rect) = drag_event.rect() {
                    let mode = if keys.pressed(KeyCode::LControl) || keys.pressed(KeyCode::RControl)
                    {
                        SelectionMode::Add
                    } else {
                        SelectionMode::Replace
                    };
                    select_events.send(SelectInRectEvent::new(rect, mode, None));
                }

                UpdateSelectionBoxEvent::none()
            }
        };

        ui_events.send(ui_event)
    }
}
