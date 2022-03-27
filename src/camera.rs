use crate::intersection::{ray_mesh_intersection, ray_plane_intersection, Ray, RayIntersection};
use crate::terrain::components::Terrain;
use bevy::ecs::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::Quat;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use std::f32::consts::{FRAC_PI_2, PI};

const MOVE_MARGIN_PX: f32 = 40.;
const MIN_CAMERA_DISTANCE: f32 = 5.;
const MAX_CAMERA_DISTANCE: f32 = 25.;
const MIN_OFF_NADIR: f32 = 0.;
const MAX_OFF_NADIR: f32 = 0.8 * FRAC_PI_2;
/// Camera moves horizontally at distance * CAMERA_HORIZONTAL_SPEED meters per
/// second.
const CAMERA_HORIZONTAL_SPEED: f32 = 1.0;
/// Camera moves along forward axis at speed distance * CAMERA_VERTICAL_SPEED
/// meters per second.
const CAMERA_VERTICAL_SPEED: f32 = 1.5;
/// Scale factor (e.g. camera_distance * factor) applied after single mouse
/// wheel click.
const WHEEL_ZOOM_FACTOR: f32 = 1.1;
/// Mouse movement by d logical pixels will lead to rotation by d *
/// ROTATION_SENSITIVITY radians.
const ROTATION_SENSITIVITY: f32 = 0.008;

#[derive(Debug)]
enum HorizonalMovement {
    None,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Component)]
pub struct CameraMovement {
    horizontal: HorizonalMovement,
    distance: f32,
    off_nadir: f32,
    azimuth: f32,
    is_rotation_synced: bool,
}

impl CameraMovement {
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

impl Default for CameraMovement {
    fn default() -> Self {
        Self {
            horizontal: HorizonalMovement::None,
            distance: MIN_CAMERA_DISTANCE,
            off_nadir: MIN_OFF_NADIR,
            azimuth: 0.0,
            is_rotation_synced: false,
        }
    }
}

pub fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 5.0, 2.0).looking_at(Vec3::ZERO, -Vec3::Z),
            ..Default::default()
        })
        .insert(CameraMovement::default());
}

pub fn move_camera(
    mut queries: QuerySet<(
        QueryState<(&mut Transform, &mut CameraMovement)>,
        QueryState<(&Handle<Mesh>, &GlobalTransform, Option<&Aabb>), With<Terrain>>,
    )>,
    time: Res<Time>,
    meshes: Res<Assets<Mesh>>,
) {
    let (map_width, map_height) = match queries.q1().single().2 {
        Some(aabb) => (aabb.max().x, aabb.max().z),
        None => return,
    };

    let camera_transform = queries.q0().single().0;
    let camera_ray = Ray::new(camera_transform.translation, camera_transform.forward());
    let focus = get_focus(queries.q1(), meshes, &camera_ray);

    rotate_camera(queries.q0(), &focus);
    move_camera_horizontaly(queries.q0(), &time, &focus, map_width, map_height);
    zoom_camera(queries.q0(), &time, &focus);
}

fn get_focus(
    terrain_query: Query<(&Handle<Mesh>, &GlobalTransform, Option<&Aabb>), With<Terrain>>,
    meshes: Res<Assets<Mesh>>,
    camera_ray: &Ray,
) -> RayIntersection {
    let (terrain_mesh_handle, terrain_transform, _) = terrain_query.single();
    let terrain_mesh = meshes.get(terrain_mesh_handle).unwrap();
    if let Some(intersection) = ray_mesh_intersection(
        camera_ray,
        terrain_mesh,
        &terrain_transform.compute_matrix(),
    ) {
        return intersection;
    }
    ray_plane_intersection(&camera_ray, Vec3::ZERO, Vec3::Y)
        .expect("Camera ray does not intersect 0 elevation terrain plane.")
}

fn rotate_camera(
    mut camera_query: Query<(&mut Transform, &mut CameraMovement)>,
    focus: &RayIntersection,
) {
    let (mut camera_transform, mut camera_movement) = camera_query.single_mut();

    if camera_movement.is_rotation_synced {
        return;
    }

    let off_nadir_rotation = Quat::from_rotation_x(camera_movement.off_nadir - FRAC_PI_2);
    let azimuth_rotation = Quat::from_axis_angle(
        off_nadir_rotation.inverse().mul_vec3(Vec3::Y),
        -camera_movement.azimuth,
    );
    camera_transform.rotation = off_nadir_rotation.mul_quat(azimuth_rotation).normalize();
    camera_transform.translation = focus.position() - focus.distance() * camera_transform.forward();
    camera_movement.is_rotation_synced = true;
}

fn zoom_camera(
    mut camera_query: Query<(&mut Transform, &mut CameraMovement)>,
    time: &Res<Time>,
    focus: &RayIntersection,
) {
    let (mut camera_transform, camera_movement) = camera_query.single_mut();
    let error = focus.distance() - camera_movement.distance;
    let abs_error = error.abs();
    if abs_error <= f32::EPSILON {
        return;
    }

    let sign = error / abs_error;
    let mut delta = sign * focus.distance() * time.delta().as_secs_f32() * CAMERA_VERTICAL_SPEED;
    if delta.abs() > abs_error {
        delta = error;
    }
    let displacement = delta * camera_transform.forward();
    camera_transform.translation += displacement;
}

fn move_camera_horizontaly(
    mut camera_query: Query<(&mut Transform, &mut CameraMovement)>,
    time: &Res<Time>,
    focus: &RayIntersection,
    map_width: f32,
    map_height: f32,
) {
    let delta = time.delta().as_secs_f32() * focus.distance() * CAMERA_HORIZONTAL_SPEED;
    let (mut camera_transform, camera_movement) = camera_query.single_mut();

    let right = camera_transform.local_x();
    let down = camera_transform.local_y();
    let displacement = match camera_movement.horizontal {
        HorizonalMovement::Left => -right * delta,
        HorizonalMovement::Right => right * delta,
        HorizonalMovement::Up => -down * delta,
        HorizonalMovement::Down => down * delta,
        HorizonalMovement::None => Vec3::ZERO,
    };

    let min_displacement = Vec3::new(-focus.position().x.max(0.), 0., -focus.position().z.max(0.));
    let max_displacement = Vec3::new(
        (map_width - focus.position().x).max(0.),
        0.,
        (map_height - focus.position().z).max(0.),
    );
    camera_transform.translation += displacement.clamp(min_displacement, max_displacement);
}

pub fn mouse_movement(
    mut event_reader: EventReader<MouseMotion>,
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut query: Query<&mut CameraMovement>,
) {
    let delta = event_reader.iter().fold(Vec2::ZERO, |sum, e| sum + e.delta);
    if delta == Vec2::ZERO {
        return;
    }

    let mut camera_movement = query.single_mut();
    if buttons.pressed(MouseButton::Middle) {
        rotate_event(delta, &mut camera_movement);
    } else {
        move_horizontaly_event(windows, &mut camera_movement);
    }
}

fn rotate_event(delta: Vec2, camera_movement: &mut CameraMovement) {
    camera_movement.rotate(ROTATION_SENSITIVITY * delta.x);
    camera_movement.tilt_clamped(-ROTATION_SENSITIVITY * delta.y);
    camera_movement.is_rotation_synced = false;
}

fn move_horizontaly_event(windows: Res<Windows>, camera_movement: &mut CameraMovement) {
    let window = windows.get_primary().unwrap();
    let (x, y) = match window.cursor_position() {
        Some(position) => (position.x, position.y),
        None => {
            camera_movement.horizontal = HorizonalMovement::None;
            return;
        }
    };

    let width = window.width();
    let height = window.height();

    if x < MOVE_MARGIN_PX {
        camera_movement.horizontal = HorizonalMovement::Left;
    } else if x > (width - MOVE_MARGIN_PX) {
        camera_movement.horizontal = HorizonalMovement::Right;
    } else if y < MOVE_MARGIN_PX {
        camera_movement.horizontal = HorizonalMovement::Up;
    } else if y > (height - MOVE_MARGIN_PX) {
        camera_movement.horizontal = HorizonalMovement::Down;
    } else {
        camera_movement.horizontal = HorizonalMovement::None;
    }
}

pub fn mouse_wheel(
    mut query: Query<&mut CameraMovement>,
    mut mouse_wheel: EventReader<MouseWheel>,
) {
    let displacement: f32 = mouse_wheel.iter().map(|e| e.y as f32).sum();
    if displacement == 0. {
        return;
    }
    // TODO: touchpad scrolling
    query
        .single_mut()
        .zoom_clamped(WHEEL_ZOOM_FACTOR.powf(displacement));
}
