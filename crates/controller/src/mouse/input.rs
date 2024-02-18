use ahash::AHashMap;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::{prelude::*, window::PrimaryWindow};
use de_core::{
    gamestate::GameState, schedule::InputSchedule, screengeom::ScreenRect, state::AppState,
};

use crate::hud::HudNodes;

const DRAGGING_THRESHOLD: f32 = 0.02;
const DOUBLE_CLICK_TIME: f64 = 0.5;

pub(super) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MouseClickedEvent>()
            .add_event::<MouseDoubleClickedEvent>()
            .add_event::<MouseDraggedEvent>()
            .add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(OnExit(AppState::InGame), cleanup)
            .add_systems(
                InputSchedule,
                (
                    update_position.in_set(MouseSet::Position),
                    update_drags
                        .run_if(resource_exists_and_changed::<MousePosition>())
                        .in_set(MouseSet::Drags)
                        .after(MouseSet::Position),
                    update_buttons
                        .in_set(MouseSet::SingeButton)
                        .after(MouseSet::Drags),
                    check_double_click
                        .in_set(MouseSet::Buttons)
                        .after(MouseSet::SingeButton),
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub(crate) enum MouseSet {
    Position,
    Drags,
    SingeButton,
    Buttons,
}

#[derive(Event)]
pub(crate) struct MouseClickedEvent {
    action: MouseButton,
    position: Vec2,
}

impl MouseClickedEvent {
    fn new(action: MouseButton, position: Vec2) -> Self {
        Self { action, position }
    }

    pub(crate) fn button(&self) -> MouseButton {
        self.action
    }

    pub(crate) fn position(&self) -> Vec2 {
        self.position
    }
}

#[derive(Event)]
pub(crate) struct MouseDoubleClickedEvent {
    action: MouseButton,
}

impl MouseDoubleClickedEvent {
    fn new(action: MouseButton) -> Self {
        Self { action }
    }

    pub(crate) fn button(&self) -> MouseButton {
        self.action
    }
}

#[derive(Event)]
pub(crate) struct MouseDraggedEvent {
    action: MouseButton,
    rect: Option<ScreenRect>,
    update_type: DragUpdateType,
}

impl MouseDraggedEvent {
    fn new(action: MouseButton, rect: Option<ScreenRect>, update_type: DragUpdateType) -> Self {
        Self {
            action,
            rect,
            update_type,
        }
    }

    pub(crate) fn button(&self) -> MouseButton {
        self.action
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
        self.0.map(|p| Vec2::new(2. * p.x - 1., 1. - 2. * p.y))
    }

    /// Top-left corner is (0, 0), bottom-right corner is (1, 1).
    fn position(&self) -> Option<Vec2> {
        self.0
    }

    fn set_position(&mut self, position: Option<Vec2>) {
        self.0 = position;
    }
}

#[derive(Default, Resource, Debug)]
struct MouseDragStates(AHashMap<MouseButton, DragState>);

impl MouseDragStates {
    fn set(&mut self, action: MouseButton, position: Option<Vec2>) {
        self.0.insert(action, DragState::new(position));
    }

    fn resolve(&mut self, action: MouseButton) -> Option<DragResolution> {
        self.0.remove(&action).and_then(DragState::resolve)
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

#[derive(Debug)]
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

fn update_position(
    window_query: Query<&Window, With<PrimaryWindow>>,
    hud: HudNodes,
    mut mouse: ResMut<MousePosition>,
) {
    let window = window_query.single();
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
    mut drags: EventWriter<MouseDraggedEvent>,
) {
    let resolutions = mouse_state.update(mouse_position.ndc());
    for (&button, &rect) in resolutions.iter() {
        drags.send(MouseDraggedEvent::new(button, rect, DragUpdateType::Moved));
    }
}

fn update_buttons(
    mouse_position: Res<MousePosition>,
    mut mouse_state: ResMut<MouseDragStates>,
    mut input_events: EventReader<MouseButtonInput>,
    mut clicks: EventWriter<MouseClickedEvent>,
    mut drags: EventWriter<MouseDraggedEvent>,
) {
    for event in input_events.iter() {
        match event.state {
            ButtonState::Released => {
                if let Some(drag_resolution) = mouse_state.resolve(event.button) {
                    match drag_resolution {
                        DragResolution::Point(position) => {
                            clicks.send(MouseClickedEvent::new(event.button, position));
                        }
                        DragResolution::Rect(rect) => {
                            drags.send(MouseDraggedEvent::new(
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
    mut clicks: EventReader<MouseClickedEvent>,
    mut double_clicks: EventWriter<MouseDoubleClickedEvent>,
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
                double_clicks.send(MouseDoubleClickedEvent::new(mouse_clicked.button()));
            }
        }

        *last_click_time = time.elapsed_seconds_f64();
        *last_click_position = Some(mouse_clicked.position());
    }
}

pub(crate) fn pressed_mouse_button(
    mouse_button: MouseButton,
) -> impl Fn(EventReader<MouseClickedEvent>) -> bool {
    move |mut click: EventReader<MouseClickedEvent>| {
        click.iter().any(|button| button.action == mouse_button)
    }
}
