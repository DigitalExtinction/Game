use std::fmt;

use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use de_camera::MoveFocusEvent;
use de_core::{stages::GameStage, state::GameState};
use de_map::size::MapBounds;
use iyes_loopless::prelude::*;

use super::nodes::MinimapNode;
use crate::hud::HudNodes;

pub(super) struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MinimapClickEvent>()
            .add_system_set_to_stage(
                GameStage::Input,
                SystemSet::new()
                    .with_system(
                        click_handler
                            .run_in_state(GameState::Playing)
                            .label(InteractionLabel::ClickHandler),
                    )
                    .with_system(
                        move_camera_system
                            .run_in_state(GameState::Playing)
                            .after(InteractionLabel::ClickHandler),
                    ),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
enum InteractionLabel {
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
    windows: Res<Windows>,
    mut input_events: EventReader<MouseButtonInput>,
    hud: HudNodes<With<MinimapNode>>,
    bounds: Res<MapBounds>,
    mut click_events: EventWriter<MinimapClickEvent>,
) {
    let Some(cursor) = windows.get_primary().unwrap().cursor_position() else { return };
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
