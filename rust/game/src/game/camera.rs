use super::{collisions::Intersector, mapdescr::MapSize, terrain::Terrain, GameStates};
use crate::math::ray::{ray_plane_intersection, Ray};
use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};
use glam::Vec3A;
use std::f32::consts::{FRAC_PI_2, PI};

/// Horizontal camera movement is initiated if mouse cursor is within this
/// distance to window edge.
const MOVE_MARGIN_LOGICAL_PX: f32 = 40.;
/// Camera moves horizontally at speed `distance * CAMERA_HORIZONTAL_SPEED`
/// meters per second.
const CAMERA_HORIZONTAL_SPEED: f32 = 1.0;
/// Minimum camera distance to terrain.
const MIN_CAMERA_DISTANCE: f32 = 8.;
/// Maximum camera distance from terrain.
const MAX_CAMERA_DISTANCE: f32 = 100.;
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

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FocusInvalidatedEvent>()
            .add_event::<PivotEvent>()
            .add_system_set(SystemSet::on_enter(GameStates::Playing).with_system(setup))
            .add_system_set(
                SystemSet::on_update(GameStates::Playing)
                    .with_system(update_focus.label("update_focus"))
                    .with_system(zoom_event.label("zoom_event"))
                    .with_system(pivot_event.label("pivot_event"))
                    .with_system(move_horizontaly_event.label("move_horizontaly_event"))
                    .with_system(zoom.label("zoom").after("zoom_event").after("update_focus"))
                    .with_system(
                        pivot
                            .label("pivot")
                            .after("pivot_event")
                            .after("update_focus"),
                    )
                    .with_system(
                        move_horizontaly
                            .label("move_horizontaly")
                            .after("move_horizontaly_event")
                            .after("update_focus")
                            // Zooming changes camera focus point so do it
                            // after other types of camera movement.
                            .after("zoom")
                            .after("pivot"),
                    ),
            );
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
    let initial_camera_distance = (MIN_CAMERA_DISTANCE * MAX_CAMERA_DISTANCE).sqrt();
    commands.insert_resource(HorizontalMovement::default());
    commands.insert_resource(DesiredPoW {
        distance: initial_camera_distance,
        off_nadir: 0.,
        azimuth: 0.,
    });
    commands.insert_resource(CameraFocus {
        point: Vec3::ZERO,
        distance: initial_camera_distance,
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, initial_camera_distance, 0.0)
            .looking_at(Vec3::ZERO, -Vec3::Z),
        ..Default::default()
    });
}

fn update_focus(
    mut event: EventReader<FocusInvalidatedEvent>,
    mut focus: ResMut<CameraFocus>,
    terrain: Intersector<With<Terrain>>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
) {
    if event.iter().next().is_none() {
        return;
    }

    let camera_transform = camera_query.single();
    let ray = Ray::new(camera_transform.translation, camera_transform.forward());
    let intersection = match terrain.ray_intersection(&ray) {
        Some((_, intersection)) => intersection,
        None => ray_plane_intersection(&ray, Vec3A::ZERO, Vec3A::Y)
            .expect("Camera ray does not intersect base ground plane."),
    };
    focus.update(intersection.position(), intersection.distance());
}

fn move_horizontaly(
    horizontal_movement: Res<HorizontalMovement>,
    focus: Res<CameraFocus>,
    map_size: Res<MapSize>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut event: EventWriter<FocusInvalidatedEvent>,
) {
    let direction = match horizontal_movement.movement() {
        Some(direction) => direction,
        None => return,
    };

    let mut transform = camera_query.single_mut();
    let delta_scalar = time.delta().as_secs_f32() * focus.distance() * CAMERA_HORIZONTAL_SPEED;
    let delta_vec = match direction {
        HorizontalMovementDirection::Left => -transform.local_x() * delta_scalar,
        HorizontalMovementDirection::Right => transform.local_x() * delta_scalar,
        HorizontalMovementDirection::Up => -transform.local_y() * delta_scalar,
        HorizontalMovementDirection::Down => transform.local_y() * delta_scalar,
    };

    let min_delta_vec = Vec3::new(
        -(focus.point().x - MAP_FOCUS_MARGIN).max(0.),
        0.,
        -(focus.point().z - MAP_FOCUS_MARGIN).max(0.),
    );
    let max_delta_vec = Vec3::new(
        (map_size.0 - focus.point().x - MAP_FOCUS_MARGIN).max(0.),
        0.,
        (map_size.0 - focus.point().z - MAP_FOCUS_MARGIN).max(0.),
    );

    transform.translation += delta_vec.clamp(min_delta_vec, max_delta_vec);
    event.send(FocusInvalidatedEvent);
}

fn zoom(
    desired_pow: Res<DesiredPoW>,
    time: Res<Time>,
    mut focus: ResMut<CameraFocus>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    let error = focus.distance() - desired_pow.distance();
    if error.abs() <= DISTANCE_TOLERATION {
        return;
    }

    let mut transform = camera_query.single_mut();
    let max_delta = focus.distance() * time.delta().as_secs_f32() * CAMERA_VERTICAL_SPEED;
    let delta_scalar = error.clamp(-max_delta, max_delta);
    let delta_vec = delta_scalar * transform.forward();
    transform.translation += delta_vec;
    focus.update_distance(delta_scalar);
}

fn pivot(
    mut event: EventReader<PivotEvent>,
    desired_pow: Res<DesiredPoW>,
    focus: Res<CameraFocus>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
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
