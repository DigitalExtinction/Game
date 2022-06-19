use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
    render::camera::Camera3d,
};
use de_core::{events::ResendEventPlugin, projection::ToMsl, state::GameState};
use de_map::size::MapBounds;
use de_terrain::TerrainCollider;
use iyes_loopless::prelude::*;
use parry3d::{
    na::{Unit, Vector3},
    query::{Ray, RayCast},
    shape::HalfSpace,
};

/// Horizontal camera movement is initiated if mouse cursor is within this
/// distance to window edge.
const MOVE_MARGIN_LOGICAL_PX: f32 = 40.;
/// Camera moves horizontally at speed `distance * CAMERA_HORIZONTAL_SPEED`
/// meters per second.
const CAMERA_HORIZONTAL_SPEED: f32 = 2.0;
/// Minimum camera distance from terrain achievable with zooming along.
const MIN_CAMERA_DISTANCE: f32 = 20.;
/// Maximum camera distance from terrain achievable with zooming alone.
const MAX_CAMERA_DISTANCE: f32 = 100.;
/// Minimum temporary distance from terrain. Forward/backward camera motion is
/// smooth within this range. Step adjustment is applied outside of this range.
const HARD_MIN_CAMERA_DISTANCE: f32 = 0.8 * MIN_CAMERA_DISTANCE;
/// Maximum temporary distance from terrain. Forward/backward camera motion is
/// smooth within this range. Step adjustment is applied outside of this range.
const HARD_MAX_CAMERA_DISTANCE: f32 = 1.1 * MAX_CAMERA_DISTANCE;
/// Camera moves along forward axis (zooming) at speed `distance *
/// CAMERA_VERTICAL_SPEED` meters per second.
const CAMERA_VERTICAL_SPEED: f32 = 2.0;
/// Do not zoom camera if it is within this distance of the desired distance.
const DISTANCE_TOLERATION: f32 = 0.001;
/// Scale factor (i.e `distance * factor`) applied after single mouse wheel
/// click.
const WHEEL_ZOOM_FACTOR: f32 = 1.1;
/// Scale factor (i.e. `distance * drag_size * factor`) applied after sliding
/// on touch pad.
const TOUCH_PAD_ZOOM_FACTOR: f32 = 1.01;
/// Minimum camera tilt in radians.
const MIN_OFF_NADIR: f32 = 0.;
/// Maximum camera tilt in radians.
const MAX_OFF_NADIR: f32 = 0.7 * FRAC_PI_2;
/// Mouse drag by `d` logical pixels will lead to rotation by `d *
/// ROTATION_SENSITIVITY` radians.
const ROTATION_SENSITIVITY: f32 = 0.008;
/// Never move camera focus point closer than this to a map edge.
const MAP_FOCUS_MARGIN: f32 = 1.0;

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MoveFocusEvent>()
            .add_plugin(ResendEventPlugin::<MoveFocusEvent>::default())
            .add_event::<FocusInvalidatedEvent>()
            .add_event::<PivotEvent>()
            .add_enter_system(GameState::Playing, setup)
            .add_system(
                update_focus
                    .run_in_state(GameState::Playing)
                    .label("update_focus"),
            )
            .add_system(
                zoom_event
                    .run_in_state(GameState::Playing)
                    .label("zoom_event"),
            )
            .add_system(
                pivot_event
                    .run_in_state(GameState::Playing)
                    .label("pivot_event"),
            )
            .add_system(
                move_horizontaly_event
                    .run_in_state(GameState::Playing)
                    .label("move_horizontaly_event"),
            )
            .add_system(
                process_move_focus_events
                    .run_in_state(GameState::Playing)
                    .label("process_move_focus_events")
                    .after("update_focus"),
            )
            .add_system(
                zoom.run_in_state(GameState::Playing)
                    .label("zoom")
                    .after("update_focus")
                    .after("zoom_event")
                    .after("process_move_focus_events"),
            )
            .add_system(
                pivot
                    .run_in_state(GameState::Playing)
                    .label("pivot")
                    .after("update_focus")
                    .after("pivot_event"),
            )
            .add_system(
                move_horizontaly
                    .run_in_state(GameState::Playing)
                    .label("move_horizontaly")
                    .after("update_focus")
                    .after("move_horizontaly_event")
                    .after("process_move_focus_events")
                    // Zooming changes camera focus point so do it
                    // after other types of camera movement.
                    .after("zoom")
                    .after("pivot"),
            );
    }
}

pub struct MoveFocusEvent {
    point: Vec2,
}

impl MoveFocusEvent {
    pub fn new(point: Vec2) -> Self {
        Self { point }
    }

    fn point(&self) -> Vec2 {
        self.point
    }
}

struct CameraFocus {
    point: Vec3,
    distance: f32,
}

impl CameraFocus {
    fn point(&self) -> Vec3 {
        self.point
    }

    fn distance(&self) -> f32 {
        self.distance
    }

    fn update<V: Into<Vec3>>(&mut self, point: V, distance: f32) {
        self.point = point.into();
        self.distance = distance;
    }

    fn set_point(&mut self, point: Vec3) {
        self.point = point;
    }

    fn update_distance(&mut self, forward_move: f32) {
        self.distance -= forward_move;
    }
}

struct FocusInvalidatedEvent;

struct PivotEvent;

#[derive(Copy, Clone)]
enum HorizontalMovementDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Default)]
struct HorizontalMovement {
    movement: Option<HorizontalMovementDirection>,
}

impl HorizontalMovement {
    fn movement(&self) -> Option<HorizontalMovementDirection> {
        self.movement
    }

    fn start(&mut self, direction: HorizontalMovementDirection) {
        self.movement = Some(direction);
    }

    fn stop(&mut self) {
        self.movement = None;
    }
}

struct DesiredPoW {
    distance: f32,
    off_nadir: f32,
    azimuth: f32,
}

impl DesiredPoW {
    fn distance(&self) -> f32 {
        self.distance
    }

    fn off_nadir(&self) -> f32 {
        self.off_nadir
    }

    fn azimuth(&self) -> f32 {
        self.azimuth
    }

    fn zoom_clamped(&mut self, factor: f32) {
        self.distance = (self.distance * factor).clamp(MIN_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE);
    }

    fn tilt_clamped(&mut self, delta: f32) {
        self.off_nadir = (self.off_nadir + delta).clamp(MIN_OFF_NADIR, MAX_OFF_NADIR);
    }

    fn rotate(&mut self, delta: f32) {
        self.azimuth = (self.azimuth + delta).rem_euclid(2. * PI);
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(HorizontalMovement::default());
    commands.insert_resource(DesiredPoW {
        distance: MAX_CAMERA_DISTANCE,
        off_nadir: 0.,
        azimuth: 0.,
    });
    commands.insert_resource(CameraFocus {
        point: Vec3::ZERO,
        distance: MAX_CAMERA_DISTANCE,
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, MAX_CAMERA_DISTANCE, 0.0)
            .looking_at(Vec3::ZERO, -Vec3::Z),
        ..Default::default()
    });
}

fn update_focus(
    mut event: EventReader<FocusInvalidatedEvent>,
    mut focus: ResMut<CameraFocus>,
    terrain: TerrainCollider,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
) {
    if event.iter().count() == 0 {
        return;
    }

    let camera_transform = camera_query.single();
    let ray = Ray::new(
        camera_transform.translation.into(),
        camera_transform.forward().into(),
    );

    let intersection = terrain
        .cast_ray_bidir(&ray, f32::INFINITY)
        .or_else(|| {
            let below_msl = HalfSpace::new(Unit::new_unchecked(Vector3::new(0., -1., 0.)));
            below_msl.cast_local_ray_and_get_normal(&ray, f32::INFINITY, false)
        })
        .expect("Camera ray does not intersect MSL plane.");
    focus.update(ray.point_at(intersection.toi), intersection.toi);
}

fn process_move_focus_events(
    mut events: EventReader<MoveFocusEvent>,
    mut focus: ResMut<CameraFocus>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let event = match events.iter().last() {
        Some(event) => event,
        None => return,
    };

    let focus_msl = event.point().to_msl();
    focus.set_point(focus_msl);

    let mut camera_transform = camera_query.single_mut();
    camera_transform.translation = focus_msl + focus.distance() * camera_transform.back();
}

fn move_horizontaly(
    horizontal_movement: Res<HorizontalMovement>,
    focus: Res<CameraFocus>,
    map_bounds: Res<MapBounds>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    mut event: EventWriter<FocusInvalidatedEvent>,
) {
    let direction = match horizontal_movement.movement() {
        Some(direction) => direction,
        None => return,
    };

    let mut transform = camera_query.single_mut();
    let distance_factor = focus
        .distance()
        .clamp(MIN_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE);
    let delta_scalar = time.delta().as_secs_f32() * distance_factor * CAMERA_HORIZONTAL_SPEED;
    let delta_vec = match direction {
        HorizontalMovementDirection::Left => -transform.local_x() * delta_scalar,
        HorizontalMovementDirection::Right => transform.local_x() * delta_scalar,
        HorizontalMovementDirection::Up => -transform.local_y() * delta_scalar,
        HorizontalMovementDirection::Down => transform.local_y() * delta_scalar,
    };

    let margin = Vec3::new(MAP_FOCUS_MARGIN, 0., MAP_FOCUS_MARGIN);
    let focus_msl: Vec3 = focus.point().to_msl();
    let map_bounds = map_bounds.aabb().to_msl();
    let min_delta_vec = (Vec3::from(map_bounds.mins) - focus_msl + margin).min(Vec3::ZERO);
    let max_delta_vec = (Vec3::from(map_bounds.maxs) - focus_msl - margin).max(Vec3::ZERO);
    transform.translation += delta_vec.clamp(min_delta_vec, max_delta_vec);
    event.send(FocusInvalidatedEvent);
}

fn zoom(
    desired_pow: Res<DesiredPoW>,
    time: Res<Time>,
    mut focus: ResMut<CameraFocus>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let mut delta_scalar = focus.distance() - HARD_MAX_CAMERA_DISTANCE;
    if delta_scalar <= 0. {
        // Camera is not further than HARD_MAX_CAMERA_DISTANCE => zoom out to
        // HARD_MIN_CAMERA_DISTANCE.
        delta_scalar = (focus.distance() - HARD_MIN_CAMERA_DISTANCE).min(0.);
    }
    if delta_scalar == 0. {
        // Camera is within HARD_MIN_CAMERA_DISTANCE and
        // HARD_MAX_CAMERA_DISTANCE => move smoothly to desired distance.

        let error = focus.distance() - desired_pow.distance();
        if error.abs() <= DISTANCE_TOLERATION {
            return;
        }

        let max_delta = focus.distance() * time.delta().as_secs_f32() * CAMERA_VERTICAL_SPEED;
        delta_scalar = error.clamp(-max_delta, max_delta);
    }

    let mut transform = camera_query.single_mut();
    let delta_vec = delta_scalar * transform.forward();
    transform.translation += delta_vec;
    focus.update_distance(delta_scalar);
}

fn pivot(
    mut event: EventReader<PivotEvent>,
    desired_pow: Res<DesiredPoW>,
    focus: Res<CameraFocus>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    if event.iter().next().is_none() {
        return;
    }

    let mut transform = camera_query.single_mut();
    transform.rotation = Quat::from_euler(
        EulerRot::YXZ,
        -desired_pow.azimuth(),
        desired_pow.off_nadir() - FRAC_PI_2,
        0.,
    );
    transform.translation = focus.point() - focus.distance() * transform.forward();
}

fn move_horizontaly_event(
    mut horizontal_movement: ResMut<HorizontalMovement>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();
    let (x, y) = match window.cursor_position() {
        Some(position) => (position.x, position.y),
        None => {
            horizontal_movement.stop();
            return;
        }
    };

    let width = window.width();
    let height = window.height();

    if x < MOVE_MARGIN_LOGICAL_PX {
        horizontal_movement.start(HorizontalMovementDirection::Left);
    } else if x > (width - MOVE_MARGIN_LOGICAL_PX) {
        horizontal_movement.start(HorizontalMovementDirection::Right);
    } else if y < MOVE_MARGIN_LOGICAL_PX {
        horizontal_movement.start(HorizontalMovementDirection::Up);
    } else if y > (height - MOVE_MARGIN_LOGICAL_PX) {
        horizontal_movement.start(HorizontalMovementDirection::Down);
    } else {
        horizontal_movement.stop();
    }
}

fn zoom_event(mut desired_pow: ResMut<DesiredPoW>, mut events: EventReader<MouseWheel>) {
    let factor = events.iter().fold(1.0, |factor, event| match event.unit {
        MouseScrollUnit::Line => factor * WHEEL_ZOOM_FACTOR.powf(event.y),
        MouseScrollUnit::Pixel => factor * TOUCH_PAD_ZOOM_FACTOR.powf(event.y),
    });
    desired_pow.zoom_clamped(factor);
}

fn pivot_event(
    mut desired_pow: ResMut<DesiredPoW>,
    mut pivot_event: EventWriter<PivotEvent>,
    buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut mouse_event: EventReader<MouseMotion>,
) {
    if !buttons.pressed(MouseButton::Middle) && !keys.pressed(KeyCode::LShift) {
        return;
    }

    let delta = mouse_event.iter().fold(Vec2::ZERO, |sum, e| sum + e.delta);
    if delta == Vec2::ZERO {
        return;
    }

    desired_pow.rotate(ROTATION_SENSITIVITY * delta.x);
    desired_pow.tilt_clamped(-ROTATION_SENSITIVITY * delta.y);
    pivot_event.send(PivotEvent);
}
