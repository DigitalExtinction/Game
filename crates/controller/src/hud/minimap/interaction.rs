use std::fmt;

use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseMotion},
        ButtonState,
    },
    prelude::*,
    window::PrimaryWindow,
};
use de_camera::MoveFocusEvent;
use de_core::{gamestate::GameState, schedule::InputSchedule};
use de_map::size::MapBounds;

use super::nodes::MinimapNode;
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
                    click_handler
                        .in_set(InteractionSet::ClickHandler)
                        .run_if(on_event::<MouseButtonInput>()),
                    drag_handler
                        .in_set(InteractionSet::DragHandler)
                        .after(InteractionSet::ClickHandler)
                        .run_if(on_event::<MouseMotion>()),
                    move_camera_system.after(InteractionSet::DragHandler),
                    send_units_system
                        .after(InteractionSet::ClickHandler)
                        .before(CommandsSet::SendSelected),
                    delivery_location_system
                        .after(InteractionSet::ClickHandler)
                        .before(CommandsSet::DeliveryLocation),
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum InteractionSet {
    ClickHandler,
    DragHandler,
}

#[derive(Event)]
struct MinimapPressEvent {
    button: MouseButton,
    position: Vec2,
}

impl MinimapPressEvent {
    fn new(button: MouseButton, position: Vec2) -> Self {
        Self { button, position }
    }

    fn button(&self) -> MouseButton {
        self.button
    }

    /// Position on the map in 2D flat coordinates (these are not minimap
    /// coordinates).
    fn position(&self) -> Vec2 {
        self.position
    }
}

impl fmt::Debug for MinimapPressEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} -> {:?}", self.button, self.position)
    }
}

#[derive(Event)]
struct MinimapDragEvent {
    button: MouseButton,
    position: Vec2,
}

impl MinimapDragEvent {
    fn new(button: MouseButton, position: Vec2) -> Self {
        Self { button, position }
    }

    fn button(&self) -> MouseButton {
        self.button
    }

    /// Position on the map in 2D flat coordinates (these are not minimap
    /// coordinates).
    fn position(&self) -> Vec2 {
        self.position
    }
}

#[derive(Resource)]
struct DraggingButtons(Vec<MouseButton>);

fn click_handler(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut input_events: EventReader<MouseButtonInput>,
    hud: HudNodes<With<MinimapNode>>,
    bounds: Res<MapBounds>,
    mut dragging: ResMut<DraggingButtons>,
    mut click_events: EventWriter<MinimapPressEvent>,
) {
    let Some(cursor) = window_query.single().cursor_position() else {
        return;
    };

    for event in input_events.iter() {
        if event.state != ButtonState::Pressed {
            dragging.0.retain(|b| *b != event.button);
            continue;
        }

        if let Some(mut relative) = hud.relative_position(cursor) {
            dragging.0.push(event.button);
            relative.y = 1. - relative.y;
            let event = MinimapPressEvent::new(event.button, bounds.rel_to_abs(relative));
            info!("Sending minimap click event {event:?}.");
            click_events.send(event);
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
    if dragging.0.is_empty() {
        return;
    }

    let Some(cursor) = window_query.single().cursor_position() else {
        return;
    };

    if let Some(relative) = hud.relative_position(cursor) {
        let proportional = Vec2::new(relative.x, 1. - relative.y);
        let world = bounds.rel_to_abs(proportional);

        for b in &dragging.0 {
            let event = MinimapDragEvent::new(*b, world);
            drag_events.send(event);
        }
    }
}

fn move_camera_system(
    mut click_events: EventReader<MinimapPressEvent>,
    mut drag_events: EventReader<MinimapDragEvent>,
    mut camera_events: EventWriter<MoveFocusEvent>,
) {
    for click in click_events.iter() {
        if click.button() != MouseButton::Left {
            continue;
        }

        let event = MoveFocusEvent::new(click.position());
        camera_events.send(event);
    }

    for drag in drag_events.iter() {
        if drag.button() != MouseButton::Left {
            continue;
        }

        let event = MoveFocusEvent::new(drag.position());
        camera_events.send(event);
    }
}

fn send_units_system(
    mut click_events: EventReader<MinimapPressEvent>,
    mut send_events: EventWriter<SendSelectedEvent>,
) {
    for click in click_events.iter() {
        if click.button() != MouseButton::Right {
            continue;
        }
        send_events.send(SendSelectedEvent::new(click.position()));
    }
}

fn delivery_location_system(
    mut click_events: EventReader<MinimapPressEvent>,
    mut location_events: EventWriter<DeliveryLocationSelectedEvent>,
) {
    for click in click_events.iter() {
        if click.button() != MouseButton::Right {
            continue;
        }
        location_events.send(DeliveryLocationSelectedEvent::new(click.position()));
    }
}
