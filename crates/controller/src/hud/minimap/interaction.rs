use std::fmt;

use bevy::{prelude::*, window::PrimaryWindow};
use de_camera::MoveFocusEvent;
use de_core::{gamestate::GameState, schedule::InputSchedule};
use de_map::size::MapBounds;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::Actionlike;

use super::nodes::MinimapNode;
use crate::actions::{mouse_input_pressed, MouseAction};
use crate::{
    commands::{CommandsSet, DeliveryLocationSelectedEvent, SendSelectedEvent},
    hud::HudNodes,
};

pub(super) struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MinimapPressEvent>()
            .add_event::<MinimapDragEvent>()
            .insert_resource(DraggingButtons(Vec::new()))
            .add_systems(
                InputSchedule,
                (
                    press_handler
                        .in_set(InteractionSet::PressHandler)
                        .run_if(mouse_input_pressed)
                        .in_set(InteractionSet::PressHandler),
                    drag_handler
                        .in_set(InteractionSet::DragHandler)
                        .run_if(mouse_input_pressed)
                        .after(InteractionSet::PressHandler),
                    move_camera_system
                        .after(InteractionSet::PressHandler)
                        .after(InteractionSet::DragHandler),
                    send_units_system
                        .after(InteractionSet::PressHandler)
                        .before(CommandsSet::SendSelected),
                    delivery_location_system
                        .after(InteractionSet::PressHandler)
                        .before(CommandsSet::DeliveryLocation),
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum InteractionSet {
    PressHandler,
    DragHandler,
}

#[derive(Event)]
struct MinimapPressEvent {
    action: MouseAction,
    position: Vec2,
}

impl MinimapPressEvent {
    fn new(action: MouseAction, position: Vec2) -> Self {
        Self { action, position }
    }

    fn button(&self) -> MouseAction {
        self.action
    }

    /// Position on the map in 2D flat coordinates (these are not minimap
    /// coordinates).
    fn position(&self) -> Vec2 {
        self.position
    }
}

impl fmt::Debug for MinimapPressEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} -> {:?}", self.action, self.position)
    }
}

#[derive(Event)]
struct MinimapDragEvent {
    action: MouseAction,
    position: Vec2,
}

impl MinimapDragEvent {
    fn new(action: MouseAction, position: Vec2) -> Self {
        Self { action, position }
    }

    fn button(&self) -> MouseAction {
        self.action
    }

    /// Position on the map in 2D flat coordinates (these are not minimap
    /// coordinates).
    fn position(&self) -> Vec2 {
        self.position
    }
}

#[derive(Resource, Deref, DerefMut)]
struct DraggingButtons(Vec<MouseAction>);

fn press_handler(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mouse_action_state: Res<ActionState<MouseAction>>,
    hud: HudNodes<With<MinimapNode>>,
    bounds: Res<MapBounds>,
    mut dragging: ResMut<DraggingButtons>,
    mut press_events: EventWriter<MinimapPressEvent>,
) {
    let cursor = window_query.single().cursor_position();

    for mouse_action in MouseAction::variants() {
        if mouse_action_state.just_released(mouse_action) {
            println!("released drag point {:?}", mouse_action);
            dragging.retain(|b| *b != mouse_action);
            continue;
        } else if mouse_action_state.just_pressed(mouse_action) {
            let Some(cursor) = cursor else {
                continue;
            };

            if let Some(mut relative) = hud.relative_position(cursor) {
                dragging.push(mouse_action);
                relative.y = 1. - relative.y;
                let event = MinimapPressEvent::new(mouse_action, bounds.rel_to_abs(relative));
                info!("Sending minimap press event {event:?}.");
                press_events.send(event);
            }
        }
    }
}

fn drag_handler(
    window_query: Query<&Window, With<PrimaryWindow>>,
    hud: HudNodes<With<MinimapNode>>,
    bounds: Res<MapBounds>,
    dragging: Res<DraggingButtons>,
    mut drag_events: EventWriter<MinimapDragEvent>,
) {
    if dragging.is_empty() {
        return;
    }

    let Some(cursor) = window_query.single().cursor_position() else {
        return;
    };

    if let Some(relative) = hud.relative_position(cursor) {
        let proportional = Vec2::new(relative.x, 1. - relative.y);
        let world = bounds.rel_to_abs(proportional);

        for button in &**dragging {
            let event = MinimapDragEvent::new(*button, world);
            drag_events.send(event);
        }
    }
}

fn move_camera_system(
    mut press_events: EventReader<MinimapPressEvent>,
    mut drag_events: EventReader<MinimapDragEvent>,
    mut camera_events: EventWriter<MoveFocusEvent>,
) {
    for press in press_events.iter() {
        if press.button() != MouseAction::PrimaryClick {
            continue;
        }

        let event = MoveFocusEvent::new(press.position());
        camera_events.send(event);
    }

    for drag in drag_events.iter() {
        if drag.button() != MouseAction::PrimaryClick {
            continue;
        }

        let event = MoveFocusEvent::new(drag.position());
        camera_events.send(event);
    }
}

fn send_units_system(
    mut press_events: EventReader<MinimapPressEvent>,
    mut send_events: EventWriter<SendSelectedEvent>,
) {
    for press in press_events.iter() {
        if press.button() != MouseAction::SecondaryClick {
            continue;
        }
        send_events.send(SendSelectedEvent::new(press.position()));
    }
}

fn delivery_location_system(
    mut press_events: EventReader<MinimapPressEvent>,
    mut location_events: EventWriter<DeliveryLocationSelectedEvent>,
) {
    for press in press_events.iter() {
        if press.button() != MouseAction::SecondaryClick {
            continue;
        }
        location_events.send(DeliveryLocationSelectedEvent::new(press.position()));
    }
}
