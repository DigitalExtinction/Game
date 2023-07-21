use bevy::{ecs::system::SystemParam, prelude::*};
use de_core::{
    schedule::PostMovement, gamestate::GameState, gconfig::GameConfig, objects::ObjectType,
    player::Player, projection::ToFlat,
};
use de_map::size::MapBounds;
use de_objects::SolidObjects;
use de_terrain::TerrainCollider;
use parry2d::{
    bounding_volume::Aabb,
    math::Point,
    query::{Ray, RayCast},
};

use super::draw::DrawingParam;
use crate::ray::ScreenRay;

const TERRAIN_COLOR: Color = Color::rgb(0.61, 0.46, 0.32);
const PLAYER_COLOR: Color = Color::rgb(0.1, 0.1, 0.9);
const ENEMY_COLOR: Color = Color::rgb(0.9, 0.1, 0.1);
const MIN_ENTITY_SIZE: Vec2 = Vec2::splat(0.02);
const CAMERA_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

pub(super) struct FillPlugin;

impl Plugin for FillPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostMovement,
            (
                clear_system
                    .run_if(in_state(GameState::Playing))
                    .in_set(FillSet::Clear),
                draw_entities_system
                    .run_if(in_state(GameState::Playing))
                    .in_set(FillSet::DrawEntities)
                    .after(FillSet::Clear),
                draw_camera_system
                    .run_if(in_state(GameState::Playing))
                    .after(FillSet::DrawEntities),
            ),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum FillSet {
    Clear,
    DrawEntities,
}

#[derive(SystemParam)]
struct UiCoords<'w> {
    bounds: Res<'w, MapBounds>,
}

impl<'w> UiCoords<'w> {
    /// Transforms 2D flat position (in meters from origin) to relative UI
    /// position (from 0 to 1 from top-right corner).
    fn flat_to_rel(&self, point: Vec2) -> Vec2 {
        Vec2::new(point.x - self.bounds.min().x, self.bounds.max().y - point.y) / self.bounds.size()
    }

    /// Transforms 2D flat position (in meters from origin) to relative UI
    /// position (from 0 to 1 from top-right corner).
    fn size_to_rel(&self, size: Vec2) -> Vec2 {
        size / self.bounds.size()
    }
}

fn clear_system(mut drawing: DrawingParam) {
    let mut drawing = drawing.drawing();
    drawing.fill(TERRAIN_COLOR);
}

fn draw_entities_system(
    mut drawing: DrawingParam,
    ui_coords: UiCoords,
    solids: SolidObjects,
    game: Res<GameConfig>,
    entities: Query<(&Transform, &Player, &ObjectType)>,
) {
    let mut drawing = drawing.drawing();

    for (transform, &player, &object_type) in entities.iter() {
        let minimap_position = ui_coords.flat_to_rel(transform.translation.to_flat());
        let color = if game.locals().is_playable(player) {
            PLAYER_COLOR
        } else {
            ENEMY_COLOR
        };

        let radius = solids.get(object_type).ichnography().radius();
        let rect_size = MIN_ENTITY_SIZE.max(ui_coords.size_to_rel(Vec2::splat(radius)));
        drawing.rect(minimap_position, rect_size, color);
    }
}

#[derive(SystemParam)]
struct CameraPoint<'w, 's> {
    ray: ScreenRay<'w, 's>,
    terrain: TerrainCollider<'w, 's>,
    ui_coords: UiCoords<'w>,
}

impl<'w, 's> CameraPoint<'w, 's> {
    fn point(&self, ndc: Vec2) -> Option<Vec2> {
        let ray = self.ray.ray(ndc);
        let Some(intersection) = self.terrain.cast_ray_msl(&ray, f32::INFINITY) else {
            return None;
        };
        let point = ray.origin + ray.dir * intersection.toi;
        Some(self.ui_coords.flat_to_rel(point.to_flat()))
    }
}

fn draw_camera_system(mut drawing: DrawingParam, camera: CameraPoint) {
    let mut drawing = drawing.drawing();

    let corner_a = camera.point(Vec2::new(-1., -1.));
    let corner_b = camera.point(Vec2::new(-1., 1.));
    let corner_c = camera.point(Vec2::new(1., 1.));
    let corner_d = camera.point(Vec2::new(1., -1.));

    if let Some((start, end)) = endpoints_to_line(corner_a, corner_b) {
        drawing.line(start, end, CAMERA_COLOR);
    }
    if let Some((start, end)) = endpoints_to_line(corner_b, corner_c) {
        drawing.line(start, end, CAMERA_COLOR);
    }
    if let Some((start, end)) = endpoints_to_line(corner_c, corner_d) {
        drawing.line(start, end, CAMERA_COLOR);
    }
    if let Some((start, end)) = endpoints_to_line(corner_d, corner_a) {
        drawing.line(start, end, CAMERA_COLOR);
    }
}

/// Converts optional line endpoints to a minimap compatible line segment.
///
/// The returned line segment is the longest line segment which a) is fully
/// contained by rectangle (0, 0) -> (1, 1), b) is fully contained by the
/// original line segment.
fn endpoints_to_line(start: Option<Vec2>, end: Option<Vec2>) -> Option<(Vec2, Vec2)> {
    let Some(start) = start else { return None };
    let Some(end) = end else { return None };

    let mut start: Point<f32> = start.into();
    let mut end: Point<f32> = end.into();

    let aabb = Aabb::new(Point::new(0., 0.), Point::new(1., 1.));
    if !aabb.contains_local_point(&start) {
        let ray = Ray::new(start, end - start);
        let Some(toi) = aabb.cast_local_ray(&ray, 1., false) else {
            return None;
        };
        start = ray.origin + toi * ray.dir;
    }
    if !aabb.contains_local_point(&end) {
        let ray = Ray::new(end, start - end);
        let Some(toi) = aabb.cast_local_ray(&ray, 1., false) else {
            return None;
        };
        end = ray.origin + toi * ray.dir;
    }

    // Clamp to avoid rounding error issues.
    Some((
        Vec2::from(start).clamp(Vec2::ZERO, Vec2::ONE),
        Vec2::from(end).clamp(Vec2::ZERO, Vec2::ONE),
    ))
}
