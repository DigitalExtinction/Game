//! This module implements user input / user command handling, for example
//! keyboard shortcuts, mouse actions events, and so on.

use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use de_camera::{
    CameraSet, MoveCameraHorizontallyEvent, RotateCameraEvent, TiltCameraEvent, ZoomCameraEvent,
};
use de_conf::Configuration;
use de_core::{
    gamestate::GameState,
    gconfig::GameConfig,
    objects::{ObjectTypeComponent, Playable},
    player::PlayerComponent,
    schedule::InputSchedule,
    screengeom::ScreenRect,
};
use de_spawner::{DraftAllowed, ObjectCounter};
use de_types::{
    objects::{BuildingType, PLAYER_MAX_BUILDINGS},
    projection::ToFlat,
};
use leafwing_input_manager::prelude::ActionState;

use super::{
    executor::DeliveryLocationSelectedEvent, CommandsSet, GroupAttackEvent, SendSelectedEvent,
};
use crate::actions::{action_pressed, Action};
use crate::{
    draft::{DiscardDraftsEvent, DraftSet, NewDraftEvent, SpawnDraftsEvent},
    hud::{GameMenuSet, ToggleGameMenuEvent, UpdateSelectionBoxEvent},
    mouse::{
        DragUpdateType, MouseDoubleClickedEvent, MouseDraggedEvent, MouseSet, Pointer, PointerSet,
    },
    selection::{
        AreaSelectSet, SelectEvent, SelectInRectEvent, Selected, SelectionMode, SelectionSet,
    },
};

/// Horizontal camera movement is initiated if mouse cursor is within this
/// distance to window edge.
const MOVE_MARGIN: f32 = 2.;

pub(super) struct HandlersPlugin;

impl HandlersPlugin {
    fn add_place_draft_systems(app: &mut App) {
        for (action, building) in Action::get_factory_actions() {
            app.add_systems(
                InputSchedule,
                place_draft(building)
                    .run_if(in_state(GameState::Playing))
                    .run_if(action_pressed(action))
                    .before(DraftSet::New)
                    .after(PointerSet::Update),
            );
        }
    }
}

impl Plugin for HandlersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            InputSchedule,
            (
                secondary_click_handler
                    .run_if(action_pressed(Action::SecondaryClick))
                    .after(PointerSet::Update)
                    .after(MouseSet::Buttons)
                    .before(CommandsSet::SendSelected)
                    .before(CommandsSet::DeliveryLocation)
                    .before(CommandsSet::Attack),
                primary_click_handler
                    .run_if(action_pressed(Action::PrimaryClick))
                    .in_set(HandlersSet::LeftClick)
                    .before(SelectionSet::Update)
                    .before(DraftSet::Spawn)
                    .after(PointerSet::Update)
                    .after(MouseSet::Buttons),
                double_click_handler
                    .run_if(on_double_click(Action::PrimaryClick))
                    .before(SelectionSet::Update)
                    .before(DraftSet::Spawn)
                    .after(PointerSet::Update)
                    .after(MouseSet::Buttons)
                    .after(HandlersSet::LeftClick),
                move_camera_arrows_system.before(CameraSet::MoveHorizontallEvent),
                move_camera_mouse_system.before(CameraSet::MoveHorizontallEvent),
                zoom_camera.before(CameraSet::ZoomEvent),
                pivot_camera
                    .before(CameraSet::RotateEvent)
                    .before(CameraSet::TiltEvent),
                handle_escape
                    .run_if(action_pressed(Action::Exit))
                    .before(GameMenuSet::Toggle)
                    .before(DraftSet::Discard),
                select_all
                    .run_if(action_pressed(Action::SelectAll))
                    .before(SelectionSet::Update),
                select_all_visible
                    .run_if(action_pressed(Action::SelectAllVisible))
                    .before(AreaSelectSet::SelectInArea),
                update_drags
                    .before(AreaSelectSet::SelectInArea)
                    .after(MouseSet::Buttons),
            )
                .run_if(in_state(GameState::Playing)),
        );

        Self::add_place_draft_systems(app);
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub(crate) enum HandlersSet {
    LeftClick,
}

fn on_double_click(button: Action) -> impl Fn(EventReader<MouseDoubleClickedEvent>) -> bool {
    move |mut events: EventReader<MouseDoubleClickedEvent>| {
        // It is desirable to exhaust the iterator, thus .filter().count() is
        // used instead of .any()
        events.iter().filter(|e| e.button() == button).count() > 0
    }
}

fn secondary_click_handler(
    config: Res<GameConfig>,
    mut send_events: EventWriter<SendSelectedEvent>,
    mut location_events: EventWriter<DeliveryLocationSelectedEvent>,
    mut attack_events: EventWriter<GroupAttackEvent>,
    targets: Query<&PlayerComponent>,
    pointer: Res<Pointer>,
) {
    match pointer.entity().filter(|&entity| {
        targets
            .get(entity)
            .map(|&player| !config.locals().is_playable(*player))
            .unwrap_or(false)
    }) {
        Some(enemy) => attack_events.send(GroupAttackEvent::new(enemy)),
        None => {
            let Some(target) = pointer.terrain_point().map(|p| p.to_flat()) else {
                return;
            };
            send_events.send(SendSelectedEvent::new(target));
            location_events.send(DeliveryLocationSelectedEvent::new(target));
        }
    }
}

fn double_click_handler(
    keys: Res<Input<KeyCode>>,
    pointer: Res<Pointer>,
    playable: Query<&ObjectTypeComponent, With<Playable>>,
    drafts: Query<(), With<DraftAllowed>>,
    mut select_in_rect_events: EventWriter<SelectInRectEvent>,
) {
    if !drafts.is_empty() {
        return;
    }
    let selection_mode = if keys.pressed(KeyCode::ControlLeft) {
        SelectionMode::Add
    } else {
        SelectionMode::Replace
    };

    let Some(targeted_entity_type) = pointer
        .entity()
        .and_then(|entity| playable.get(entity).ok())
    else {
        return;
    };

    // Select all the units visible of the same type as the targeted entity
    select_in_rect_events.send(SelectInRectEvent::new(
        ScreenRect::full(),
        selection_mode,
        Some(**targeted_entity_type),
    ));
}

fn move_camera_arrows_system(
    action_state: Res<ActionState<Action>>,
    mut move_events: EventWriter<MoveCameraHorizontallyEvent>,
    mut current_direction: Local<Vec2>,
) {
    let old_direction = *current_direction;

    let mut stopping: bool = false;

    if action_state.just_pressed(Action::Left) {
        current_direction.x = -1.;
    } else if action_state.just_released(Action::Left) {
        current_direction.x = 0.;
        stopping = true;
    }

    if action_state.just_pressed(Action::Right) {
        current_direction.x = 1.;
    } else if action_state.just_released(Action::Right) {
        current_direction.x = 0.;
        stopping = true;
    }

    if action_state.just_pressed(Action::Up) {
        current_direction.y = 1.;
    } else if action_state.just_released(Action::Up) {
        current_direction.y = 0.;
        stopping = true;
    }

    if action_state.just_pressed(Action::Down) {
        current_direction.y = -1.;
    } else if action_state.just_released(Action::Down) {
        current_direction.y = 0.;
        stopping = true;
    }

    if *current_direction == Vec2::ZERO && !stopping || *current_direction == old_direction {
        return;
    }

    move_events.send(MoveCameraHorizontallyEvent::new(*current_direction));
}

fn move_camera_mouse_system(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut was_moving: Local<bool>,
    mut move_events: EventWriter<MoveCameraHorizontallyEvent>,
) {
    let window = window_query.single();
    let Some(cursor) = window.cursor_position() else {
        if *was_moving {
            *was_moving = false;
            move_events.send(MoveCameraHorizontallyEvent::new(Vec2::ZERO));
        }
        return;
    };

    let mut movement = Vec2::ZERO;
    if cursor.x < MOVE_MARGIN {
        movement.x -= 1.;
    } else if cursor.x > (window.width() - MOVE_MARGIN) {
        movement.x += 1.;
    }
    if cursor.y < MOVE_MARGIN {
        movement.y += 1.;
    } else if cursor.y > (window.height() - MOVE_MARGIN) {
        movement.y -= 1.;
    }

    if (movement != Vec2::ZERO) == *was_moving {
        return;
    }
    *was_moving = movement != Vec2::ZERO;
    move_events.send(MoveCameraHorizontallyEvent::new(movement));
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
    action_state: Res<ActionState<Action>>,
    mut mouse_event: EventReader<MouseMotion>,
    mut rotate_event: EventWriter<RotateCameraEvent>,
    mut tilt_event: EventWriter<TiltCameraEvent>,
) {
    if !action_state.pressed(Action::Pivot) {
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

fn primary_click_handler(
    mut select_events: EventWriter<SelectEvent>,
    mut draft_events: EventWriter<SpawnDraftsEvent>,
    action_state: Res<ActionState<Action>>,
    pointer: Res<Pointer>,
    playable: Query<(), With<Playable>>,
    drafts: Query<(), With<DraftAllowed>>,
) {
    if drafts.is_empty() {
        let selection_mode = if action_state.pressed(Action::AddToSelection) {
            SelectionMode::AddToggle
        } else if action_state.pressed(Action::ReplaceSelection) {
            SelectionMode::Replace
        } else {
            return;
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
    mut toggle_menu_events: EventWriter<ToggleGameMenuEvent>,
    mut discard_events: EventWriter<DiscardDraftsEvent>,
    drafts: Query<(), With<DraftAllowed>>,
) {
    if drafts.is_empty() {
        toggle_menu_events.send(ToggleGameMenuEvent);
    } else {
        discard_events.send(DiscardDraftsEvent);
    }
}

fn place_draft(
    building_type: BuildingType,
) -> impl Fn(Res<GameConfig>, Res<ObjectCounter>, Res<Pointer>, EventWriter<NewDraftEvent>) {
    move |conf: Res<GameConfig>,
          counter: Res<ObjectCounter>,
          pointer: Res<Pointer>,
          mut events: EventWriter<NewDraftEvent>| {
        if counter
            .player(conf.locals().playable())
            .map_or(0, |c| c.building_count())
            >= PLAYER_MAX_BUILDINGS
        {
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
    action_state: Res<ActionState<Action>>,
    mut drag_events: EventReader<MouseDraggedEvent>,
    mut ui_events: EventWriter<UpdateSelectionBoxEvent>,
    mut select_events: EventWriter<SelectInRectEvent>,
) {
    for drag_event in drag_events.iter() {
        if drag_event.button() != Action::PrimaryClick {
            continue;
        }

        let ui_event = match drag_event.update_type() {
            DragUpdateType::Moved => match drag_event.rect() {
                Some(rect) => UpdateSelectionBoxEvent::from_rect(rect),
                None => UpdateSelectionBoxEvent::none(),
            },
            DragUpdateType::Released => {
                if let Some(rect) = drag_event.rect() {
                    let mode = if action_state.pressed(Action::AddToSelection) {
                        SelectionMode::Add
                    } else if action_state.pressed(Action::ReplaceSelection) {
                        SelectionMode::Replace
                    } else {
                        continue;
                    };

                    select_events.send(SelectInRectEvent::new(rect, mode, None));
                }

                UpdateSelectionBoxEvent::none()
            }
        };

        ui_events.send(ui_event)
    }
}
