use ahash::AHashMap;
use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use de_core::{
    screengeom::ScreenRect,
    stages::GameStage,
    state::{AppState, GameState},
};
use iyes_loopless::prelude::*;

use crate::hud::HudNodes;

const DRAGGING_THRESHOLD: f32 = 0.02;
const DOUBLE_CLICK_TIME: f64 = 0.5;

pub(super) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MouseClicked>()
            .add_event::<MouseDoubleClicked>()
            .add_event::<MouseDragged>()
            .add_enter_system(AppState::InGame, setup)
            .add_exit_system(AppState::InGame, cleanup)
            .add_system_set_to_stage(
                GameStage::Input,
                SystemSet::new()
                    .with_system(
                        update_position
                            .run_in_state(GameState::Playing)
                            .label(MouseLabels::Position),
                    )
                    .with_system(
                        update_drags
                            .run_in_state(GameState::Playing)
                            .label(MouseLabels::Drags)
                            .after(MouseLabels::Position),
                    )
                    .with_system(
                        update_buttons
                            .run_in_state(GameState::Playing)
                            .label(MouseLabels::SingeButton)
                            .after(MouseLabels::Drags),
                    )
                    .with_system(
                        check_double_click
                            .run_in_state(GameState::Playing)
                            .label(MouseLabels::Buttons)
                            .after(MouseLabels::SingeButton),
                    ),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum MouseLabels {
    Position,
    Drags,
    SingeButton,
    Buttons,
}

pub(crate) struct MouseClicked {
    button: MouseButton,
    position: Vec2,
}

impl MouseClicked {
    fn new(button: MouseButton, position: Vec2) -> Self {
        Self { button, position }
    }

    pub(crate) fn button(&self) -> MouseButton {
        self.button
    }

    pub(crate) fn position(&self) -> Vec2 {
        self.position
    }
}

pub(crate) struct MouseDoubleClicked {
    button: MouseButton,
}

impl MouseDoubleClicked {
    fn new(button: MouseButton) -> Self {
        Self { button }
    }

    pub(crate) fn button(&self) -> MouseButton {
        self.button
    }
}

pub(crate) struct MouseDragged {
    button: MouseButton,
    rect: Option<ScreenRect>,
    update_type: DragUpdateType,
}

impl MouseDragged {
    fn new(button: MouseButton, rect: Option<ScreenRect>, update_type: DragUpdateType) -> Self {
        Self {
            button,
            rect,
            update_type,
        }
    }

    pub(crate) fn button(&self) -> MouseButton {
        self.button
    }

    /// Screen rectangle corresponding to the drag (i.e. its starting and
    /// ending points). It is None if the starting or ending point is not over
    /// the 3D world.
    pub(crate) fn rect(&self) -> Option<ScreenRect> {
        self.rect
    }

    pub(crate) fn update_type(&self) -> DragUpdateType {
        self.update_type
    }
}

#[derive(Clone, Copy)]
pub(crate) enum DragUpdateType {
    Moved,
    Released,
}

#[derive(Default, Resource)]
pub(crate) struct MousePosition(Option<Vec2>);

impl MousePosition {
    /// Returns position of the mouse on screen normalized to values between
    /// [-1., -1.] (bottom-left corner) and [1., 1.] (upper-right corner).
    pub(crate) fn ndc(&self) -> Option<Vec2> {
        self.0.map(|p| 2. * p - Vec2::ONE)
    }

    fn position(&self) -> Option<Vec2> {
        self.0
    }

    fn set_position(&mut self, position: Option<Vec2>) {
        self.0 = position;
    }
}

#[derive(Default, Resource)]
struct MouseDragStates(AHashMap<MouseButton, DragState>);

impl MouseDragStates {
    fn set(&mut self, button: MouseButton, position: Option<Vec2>) {
        self.0.insert(button, DragState::new(position));
    }

    fn resolve(&mut self, button: MouseButton) -> Option<DragResolution> {
        self.0.remove(&button).and_then(DragState::resolve)
    }

    /// Updates the end position of all opened drags. A map of mouse buttons to
    /// updated screen rectangle is returned for all changed drags.
    ///
    /// None means that the drag is (temporarily) canceled, Some means that the
    /// drag has been updated to this new rectangle.
    fn update(&mut self, position: Option<Vec2>) -> AHashMap<MouseButton, Option<ScreenRect>> {
        let mut updates = AHashMap::new();
        for (&button, drag) in self.0.iter_mut() {
            if let Some(update) = drag.update(position) {
                updates.insert(button, update);
            }
        }
        updates
    }
}

struct DragState {
    start: Option<Vec2>,
    stop: Option<Vec2>,
    active: bool,
}

impl DragState {
    fn new(start: Option<Vec2>) -> Self {
        Self {
            start,
            stop: start,
            active: false,
        }
    }

    fn resolve(self) -> Option<DragResolution> {
        match self.start {
            Some(start) => match (self.active, self.stop) {
                (true, Some(stop)) => Some(DragResolution::Rect(Some(ScreenRect::from_points(
                    start, stop,
                )))),
                (true, None) => Some(DragResolution::Rect(None)),
                (false, Some(stop)) => Some(DragResolution::Point(stop)),
                (false, None) => None,
            },
            None => None,
        }
    }

    fn update(&mut self, position: Option<Vec2>) -> Option<Option<ScreenRect>> {
        let changed = self.stop != position;
        self.stop = position;

        if let Some(start) = self.start {
            let rect = match self.stop {
                Some(stop) => {
                    self.active |= start.distance(stop) >= DRAGGING_THRESHOLD;
                    Some(ScreenRect::from_points(start, stop))
                }
                None => None,
            };

            if self.active && changed {
                return Some(rect);
            }
        }

        None
    }
}

enum DragResolution {
    Point(Vec2),
    Rect(Option<ScreenRect>),
}

fn setup(mut commands: Commands) {
    commands.init_resource::<MousePosition>();
    commands.init_resource::<MouseDragStates>();
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<MousePosition>();
    commands.remove_resource::<MouseDragStates>();
}

fn update_position(windows: Res<Windows>, hud: HudNodes, mut mouse: ResMut<MousePosition>) {
    let window = windows.get_primary().unwrap();
    let position = window
        .cursor_position()
        .filter(|&position| !hud.contains_point(position))
        .map(|position| position / Vec2::new(window.width(), window.height()))
        .map(|normalised_position| normalised_position.clamp(Vec2::ZERO, Vec2::ONE));

    // Avoid unnecessary change detection.
    if mouse.position() != position {
        mouse.set_position(position)
    }
}

fn update_drags(
    mouse_position: Res<MousePosition>,
    mut mouse_state: ResMut<MouseDragStates>,
    mut drags: EventWriter<MouseDragged>,
) {
    if mouse_position.is_changed() {
        let resolutions = mouse_state.update(mouse_position.ndc());
        for (&button, &rect) in resolutions.iter() {
            drags.send(MouseDragged::new(button, rect, DragUpdateType::Moved));
        }
    }
}

fn update_buttons(
    mouse_position: Res<MousePosition>,
    mut mouse_state: ResMut<MouseDragStates>,
    mut input_events: EventReader<MouseButtonInput>,
    mut clicks: EventWriter<MouseClicked>,
    mut drags: EventWriter<MouseDragged>,
) {
    for event in input_events.iter() {
        match event.state {
            ButtonState::Released => {
                if let Some(drag_resolution) = mouse_state.resolve(event.button) {
                    match drag_resolution {
                        DragResolution::Point(position) => {
                            clicks.send(MouseClicked::new(event.button, position));
                        }
                        DragResolution::Rect(rect) => {
                            drags.send(MouseDragged::new(
                                event.button,
                                rect,
                                DragUpdateType::Released,
                            ));
                        }
                    }
                }
            }
            ButtonState::Pressed => {
                mouse_state.set(event.button, mouse_position.ndc());
            }
        }
    }
}

fn check_double_click(
    mut clicks: EventReader<MouseClicked>,
    mut double_clicks: EventWriter<MouseDoubleClicked>,
    mut last_click_position: Local<Option<Vec2>>,
    mut last_click_time: Local<f64>,
    time: Res<Time>,
) {
    for mouse_clicked in clicks.iter() {
        let current_time = time.elapsed_seconds_f64();

        if last_click_position.map_or(true, |p| {
            p.distance(mouse_clicked.position()) < DRAGGING_THRESHOLD
        }) {
            // Check if double click using timer
            if (current_time - *last_click_time) < DOUBLE_CLICK_TIME {
                double_clicks.send(MouseDoubleClicked::new(mouse_clicked.button()));
            }
        }

        *last_click_time = time.elapsed_seconds_f64();
        *last_click_position = Some(mouse_clicked.position());
    }
}
