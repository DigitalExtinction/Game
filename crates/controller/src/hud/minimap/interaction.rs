use std::fmt;

use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    window::PrimaryWindow,
};
use de_camera::MoveFocusEvent;
use de_core::{baseset::GameSet, gamestate::GameState};
use de_map::size::MapBounds;

use super::nodes::MinimapNode;
use crate::{
    commands::{CommandsSet, DeliveryLocationSelectedEvent, SendSelectedEvent},
    hud::HudNodes,
};

pub(super) struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MinimapClickEvent>()
            .add_system(
                click_handler
                    .in_base_set(GameSet::Input)
                    .run_if(in_state(GameState::Playing))
                    .in_set(InteractionSet::ClickHandler),
            )
            .add_system(
                move_camera_system
                    .in_base_set(GameSet::Input)
                    .run_if(in_state(GameState::Playing))
                    .after(InteractionSet::ClickHandler),
            )
            .add_system(
                send_units_system
                    .in_base_set(GameSet::Input)
                    .run_if(in_state(GameState::Playing))
                    .after(InteractionSet::ClickHandler)
                    .before(CommandsSet::SendSelected),
            )
            .add_system(
                delivery_location_system
                    .in_base_set(GameSet::Input)
                    .run_if(in_state(GameState::Playing))
                    .after(InteractionSet::ClickHandler)
                    .before(CommandsSet::DeliveryLocation),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum InteractionSet {
    ClickHandler,
}

struct MinimapClickEvent {
    button: MouseButton,
    position: Vec2,
}

impl MinimapClickEvent {
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

impl fmt::Debug for MinimapClickEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} -> {:?}", self.button, self.position)
    }
}

fn click_handler(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut input_events: EventReader<MouseButtonInput>,
    hud: HudNodes<With<MinimapNode>>,
    bounds: Res<MapBounds>,
    mut click_events: EventWriter<MinimapClickEvent>,
) {
    let Some(cursor) = window_query.single().cursor_position() else { return };
    for event in input_events.iter() {
        if event.state != ButtonState::Released {
            continue;
        }

        if let Some(mut relative) = hud.relative_position(cursor) {
            relative.y = 1. - relative.y;
            let event = MinimapClickEvent::new(event.button, bounds.rel_to_abs(relative));
            info!("Sending minimap click event {event:?}.");
            click_events.send(event);
        }
    }
}

fn move_camera_system(
    mut click_events: EventReader<MinimapClickEvent>,
    mut camera_events: EventWriter<MoveFocusEvent>,
) {
    for click in click_events.iter() {
        if click.button() != MouseButton::Left {
            continue;
        }
        camera_events.send(MoveFocusEvent::new(click.position()));
    }
}

fn send_units_system(
    mut click_events: EventReader<MinimapClickEvent>,
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
    mut click_events: EventReader<MinimapClickEvent>,
    mut location_events: EventWriter<DeliveryLocationSelectedEvent>,
) {
    for click in click_events.iter() {
        if click.button() != MouseButton::Right {
            continue;
        }
        location_events.send(DeliveryLocationSelectedEvent::new(click.position()));
    }
}
