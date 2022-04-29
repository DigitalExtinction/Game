use bevy::{
    ecs::system::SystemParam,
    input::mouse::MouseMotion,
    prelude::{
        App, Camera, Entity, EventReader, GlobalTransform, Plugin, Query, Res, ResMut, With,
    },
    window::Windows,
};
use de_core::objects::Playable;
use glam::{Vec2, Vec3};
use iyes_loopless::prelude::*;

use super::{collisions::Intersector, terrain::Terrain, GameState, Labels};
use crate::math::ray::Ray;

pub struct PointerPlugin;

impl Plugin for PointerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pointer>().add_system(
            mouse_move_handler
                .run_in_state(GameState::Playing)
                .label(Labels::PreInputUpdate),
        );
    }
}

#[derive(Default)]
pub struct Pointer {
    entity: Option<Entity>,
    terrain: Option<Vec3>,
}

impl Pointer {
    /// Pointed to playable entity or None if mouse is not over any playable
    /// entity.
    pub fn entity(&self) -> Option<Entity> {
        self.entity
    }

    /// Pointed to 3D position on the surface of the terrain. This can be below
    /// (occluded) another entity. It is None if the mouse is not over terrain
    /// at all.
    pub fn terrain_point(&self) -> Option<Vec3> {
        self.terrain
    }

    fn set_entity(&mut self, entity: Option<Entity>) {
        self.entity = entity;
    }

    fn set_terrain_point(&mut self, point: Option<Vec3>) {
        self.terrain = point;
    }
}

#[derive(SystemParam)]
struct MouseInWorld<'w, 's> {
    windows: Res<'w, Windows>,
    cameras: Query<'w, 's, (&'static GlobalTransform, &'static Camera)>,
}

impl<'w, 's> MouseInWorld<'w, 's> {
    fn mouse_ray(&self) -> Option<Ray> {
        let window = self.windows.get_primary().unwrap();

        // Normalized to values between -1.0 to 1.0 with (0.0, 0.0) in the
        // middle of the screen.
        let cursor_position = match window.cursor_position() {
            Some(position) => {
                let screen_size = Vec2::new(window.width() as f32, window.height() as f32);
                (position / screen_size) * 2.0 - Vec2::ONE
            }
            None => return None,
        };

        let (camera_transform, camera) = self.cameras.single();
        let camera_transform_mat = camera_transform.compute_matrix();
        let camera_projection = camera.projection_matrix;

        let screen_to_world = camera_transform_mat * camera_projection.inverse();
        let world_to_screen = camera_projection * camera_transform_mat;

        // Depth of camera near plane in screen coordinates.
        let near_plane = world_to_screen.transform_point3(-Vec3::Z * camera.near).z;
        let ray_origin = screen_to_world.transform_point3(cursor_position.extend(near_plane));
        let ray_direction = ray_origin - camera_transform.translation;
        Some(Ray::new(ray_origin, ray_direction))
    }
}

fn mouse_move_handler(
    mut resource: ResMut<Pointer>,
    event: EventReader<MouseMotion>,
    mouse: MouseInWorld,
    playable: Intersector<With<Playable>>,
    terrain: Intersector<With<Terrain>>,
) {
    if event.is_empty() {
        return;
    }

    let ray = mouse.mouse_ray();

    let entity = ray
        .as_ref()
        .and_then(|ray| playable.ray_intersection(ray))
        .map(|(entity, _)| entity);
    resource.set_entity(entity);

    let terrain_point = ray
        .as_ref()
        .and_then(|ray| terrain.ray_intersection(ray))
        .map(|(_, intersection)| intersection.position().into());
    resource.set_terrain_point(terrain_point);
}
