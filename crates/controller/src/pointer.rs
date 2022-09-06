use bevy::{ecs::system::SystemParam, prelude::*, window::Windows};
use de_core::{stages::GameStage, state::GameState};
use de_index::SpatialQuery;
use de_terrain::TerrainCollider;
use glam::{Vec2, Vec3};
use iyes_loopless::prelude::*;
use parry3d::query::Ray;

use crate::Labels;

pub(crate) struct PointerPlugin;

impl Plugin for PointerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pointer>().add_system_to_stage(
            GameStage::Input,
            mouse_move_handler
                .run_in_state(GameState::Playing)
                .label(Labels::PreInputUpdate),
        );
    }
}

#[derive(Default)]
pub(crate) struct Pointer {
    entity: Option<Entity>,
    terrain: Option<Vec3>,
}

impl Pointer {
    /// Pointed to entity or None if mouse is not over any entity.
    pub(crate) fn entity(&self) -> Option<Entity> {
        self.entity
    }

    /// Pointed to 3D position on the surface of the terrain. This can be below
    /// (occluded) another entity. It is None if the mouse is not over terrain
    /// at all.
    pub(crate) fn terrain_point(&self) -> Option<Vec3> {
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
    cameras: Query<'w, 's, (&'static GlobalTransform, &'static Camera), With<Camera3d>>,
}

impl<'w, 's> MouseInWorld<'w, 's> {
    fn mouse_ray(&self) -> Option<Ray> {
        let window = self.windows.get_primary().unwrap();

        // Normalized to values between -1.0 to 1.0 with (0.0, 0.0) in the
        // middle of the screen.
        let cursor_position = match window.cursor_position() {
            Some(position) => {
                let screen_size = Vec2::new(window.width(), window.height());
                (position / screen_size) * 2.0 - Vec2::ONE
            }
            None => return None,
        };

        let (camera_transform, camera) = self.cameras.single();
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
        let ray_origin = ndc_to_world.project_point3(cursor_position.extend(1.));
        let ray_direction = ray_origin - camera_transform.translation();
        Some(Ray::new(ray_origin.into(), ray_direction.into()))
    }
}

fn mouse_move_handler(
    mut resource: ResMut<Pointer>,
    mouse: MouseInWorld,
    entities: SpatialQuery<()>,
    terrain: TerrainCollider,
) {
    let ray = mouse.mouse_ray();

    let entity = ray
        .as_ref()
        .and_then(|ray| entities.cast_ray(ray, f32::INFINITY, None))
        .map(|intersection| intersection.entity());
    resource.set_entity(entity);

    let terrain_point = ray
        .and_then(|ray| terrain.cast_ray(&ray, f32::INFINITY))
        .map(|intersection| ray.unwrap().point_at(intersection.toi).into());
    resource.set_terrain_point(terrain_point);
}
