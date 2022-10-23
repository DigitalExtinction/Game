use ahash::AHashMap;
use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use de_core::{screengeom::ScreenRect, stages::GameStage, state::GameState};
use iyes_loopless::condition::IntoConditionalExclusiveSystem;

const DRAGGING_THRESHOLD: f32 = 0.02;

pub(crate) struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MouseClicked>()
            .add_event::<MouseDragged>()
            .init_resource::<MousePosition>()
            .init_resource::<MouseDragStates>()
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
                            .label(MouseLabels::Buttons)
                            .after(MouseLabels::Drags),
                    ),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum MouseLabels {
    Position,
    Drags,
    Buttons,
}

pub(crate) struct MouseClicked {
    button: MouseButton,
}

impl MouseClicked {
    fn new(button: MouseButton) -> Self {
        Self { button }
    }

    pub(crate) fn button(&self) -> MouseButton {
        self.button
    }
}

pub(crate) struct MouseDragged {
    button: MouseButton,
    rect: ScreenRect,
    update_type: DragUpdateType,
}

impl MouseDragged {
    fn new(button: MouseButton, rect: ScreenRect, update_type: DragUpdateType) -> Self {
        Self {
            button,
            rect,
            update_type,
        }
    }

    pub(crate) fn button(&self) -> MouseButton {
        self.button
    }

    pub(crate) fn rect(&self) -> ScreenRect {
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

#[derive(Default)]
pub(crate) struct MousePosition(Option<Vec2>);

impl MousePosition {
    /// Returns position of the mouse on screen normalized to values between
    /// [-1., -1.] (bottom-left corner) and [1., 1.] (upper-right corner).
    pub(crate) fn ndc(&self) -> Option<Vec2> {
        self.0.map(|p| 2. * p - Vec2::ONE)
    }

    fn set_position(&mut self, position: Option<Vec2>) {
        self.0 = position;
    }
}

#[derive(Default)]
pub(crate) struct MouseDragStates(AHashMap<MouseButton, DragState>);

impl MouseDragStates {
    fn set(&mut self, button: MouseButton, position: Option<Vec2>) {
        self.0.insert(button, DragState::new(position));
    }

    fn resolve(&mut self, button: MouseButton) -> Option<DragResolution> {
        self.0.remove(&button).and_then(DragState::resolve)
    }

    fn update(&mut self, position: Option<Vec2>) -> AHashMap<MouseButton, ScreenRect> {
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
        if let Some(start) = self.start {
            if let Some(stop) = self.stop {
                if self.active {
                    return Some(DragResolution::Rect(ScreenRect::from_points(start, stop)));
                } else {
                    return Some(DragResolution::Point(stop));
                }
            }
        }
        None
    }

    fn update(&mut self, position: Option<Vec2>) -> Option<ScreenRect> {
        let changed = self.stop != position;
        self.stop = position;

        if let Some(start) = self.start {
            if let Some(stop) = position {
                self.active |= start.distance(stop) >= DRAGGING_THRESHOLD;

                if self.active && changed {
                    return Some(ScreenRect::from_points(start, stop));
                }
            }
        }

        None
    }
}

enum DragResolution {
    Point(Vec2),
    Rect(ScreenRect),
}

fn update_position(windows: Res<Windows>, mut mouse: ResMut<MousePosition>) {
    let window = windows.get_primary().unwrap();
    mouse.set_position(
        window
            .cursor_position()
            .map(|position| position / Vec2::new(window.width(), window.height())),
    );
}

fn update_drags(
    mouse_position: Res<MousePosition>,
    mut mouse_state: ResMut<MouseDragStates>,
    mut drags: EventWriter<MouseDragged>,
) {
    let resolutions = mouse_state.update(mouse_position.ndc());
    for (&button, &rect) in resolutions.iter() {
        drags.send(MouseDragged::new(button, rect, DragUpdateType::Moved));
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
                        DragResolution::Point(_) => {
                            clicks.send(MouseClicked::new(event.button));
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
