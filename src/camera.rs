use crate::intersection::{ray_mesh_intersection, ray_plane_intersection, Ray};
use crate::terrain::components::Terrain;
use bevy::ecs::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

const MOVE_MARGIN_PX: f32 = 40.0;

#[derive(Debug)]
enum HorizonalMovement {
    None,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Component)]
pub struct Movement {
    horizontal: HorizonalMovement,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            horizontal: HorizonalMovement::None,
        }
    }
}

pub fn setup(mut commands: Commands) {
    commands.spawn().insert(Movement::default());
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 5.0, 2.0).looking_at(Vec3::ZERO, -Vec3::Z),
        ..Default::default()
    });
}

pub fn mouse_movement(
    mut event_reader: EventReader<CursorMoved>,
    windows: Res<Windows>,
    mut query: Query<&mut Movement>,
) {
    if let Some(event) = event_reader.iter().last() {
        let x = event.position.x;
        let y = event.position.y;

        let window = windows.get_primary().unwrap();
        let width = window.width();
        let height = window.height();

        let mut movement = query.single_mut();
        if x < MOVE_MARGIN_PX {
            movement.horizontal = HorizonalMovement::Left;
        } else if x > (width - MOVE_MARGIN_PX) {
            movement.horizontal = HorizonalMovement::Right;
        } else if y < MOVE_MARGIN_PX {
            movement.horizontal = HorizonalMovement::Up;
        } else if y > (height - MOVE_MARGIN_PX) {
            movement.horizontal = HorizonalMovement::Down;
        } else {
            movement.horizontal = HorizonalMovement::None;
        }
    }
}

pub fn move_horizontaly(query: Query<&Movement>, mut camera: Query<&mut Transform, With<Camera>>) {
    let movement = query.single();
    if let HorizonalMovement::None = movement.horizontal {
        return;
    }

    let mut transform = camera.single_mut();
    let right = transform.local_x();
    let down = transform.local_y();
    match movement.horizontal {
        HorizonalMovement::Left => transform.translation -= right * 0.1,
        HorizonalMovement::Right => transform.translation += right * 0.1,
        HorizonalMovement::Up => transform.translation -= down * 0.1,
        HorizonalMovement::Down => transform.translation += down * 0.1,
        HorizonalMovement::None => (),
    };
}

pub fn zoom(
    mut queries: QuerySet<(
        QueryState<&mut Transform, With<Camera>>,
        QueryState<(&Handle<Mesh>, &Transform), With<Terrain>>,
    )>,
    meshes: Res<Assets<Mesh>>,
    mut mouse_wheel: EventReader<MouseWheel>,
) {
    let displacement: f32 = mouse_wheel.iter().map(|e| e.y as f32).sum();
    if displacement == 0. {
        return;
    }

    let camera_query = queries.q0();
    let camera_transform = camera_query.single();
    let camera_ray = Ray::new(camera_transform.translation, camera_transform.forward());

    let terrain_query = queries.q1();
    let (terrain_mesh_handle, terrain_transform) = terrain_query.single();
    let terrain = meshes.get(terrain_mesh_handle).unwrap();
    let terrain_intersection =
        ray_mesh_intersection(&camera_ray, terrain, &terrain_transform.compute_matrix())
            .unwrap_or_else(|| {
                ray_plane_intersection(&camera_ray, Vec3::ZERO, Vec3::Y)
                    .expect("Camera ray does not intersect 0 elevation terrain plane.")
            });

    let distance_delta = terrain_intersection.distance() * displacement * 0.1;
    let mut camera_query = queries.q0();
    let mut camera_transform = camera_query.single_mut();
    camera_transform.translation -= camera_ray.direction() * distance_delta;
}
