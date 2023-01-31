use bevy::{ecs::system::SystemParam, prelude::*};
use de_core::{stages::GameStage, state::GameState};
use de_index::SpatialQuery;
use de_signs::UpdateBarVisibilityEvent;
use de_terrain::TerrainCollider;
use glam::{Vec2, Vec3};
use iyes_loopless::prelude::*;
use parry3d::query::Ray;

use crate::{
    mouse::{MouseLabels, MousePosition},
    POINTER_BAR_ID,
};

pub(super) struct PointerPlugin;

impl Plugin for PointerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pointer>()
            .add_system_to_stage(
                GameStage::Input,
                pointer_update_system
                    .run_in_state(GameState::Playing)
                    .label(PointerLabels::Update)
                    .after(MouseLabels::Position),
            )
            .add_system_to_stage(
                GameStage::Input,
                update_bar_visibility
                    .run_in_state(GameState::Playing)
                    .after(PointerLabels::Update),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum PointerLabels {
    Update,
}

#[derive(Default, Resource)]
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
struct ScreenRay<'w, 's> {
    cameras: Query<'w, 's, (&'static Transform, &'static Camera), With<Camera3d>>,
}

impl<'w, 's> ScreenRay<'w, 's> {
    /// Returns line of sight of a point on the screen.
    ///
    /// The ray originates on the near plane of the projection frustum.
    ///
    /// # Arguments
    ///
    /// * `point` - normalized coordinates (between [-1., -1.] and [1., 1.]) of
    ///   a point on the screen.
    fn ray(&self, point: Vec2) -> Ray {
        let (camera_transform, camera) = self.cameras.single();
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
        let ray_origin = ndc_to_world.project_point3(point.extend(1.));
        let ray_direction = (ray_origin - camera_transform.translation).normalize();
        Ray::new(ray_origin.into(), ray_direction.into())
    }
}

fn pointer_update_system(
    mut resource: ResMut<Pointer>,
    mouse: Res<MousePosition>,
    screen_ray: ScreenRay,
    entities: SpatialQuery<()>,
    terrain: TerrainCollider,
) {
    let ray = mouse.ndc().map(|cursor| screen_ray.ray(cursor));

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

fn update_bar_visibility(
    pointer: Res<Pointer>,
    mut previous: Local<Option<Entity>>,
    mut events: EventWriter<UpdateBarVisibilityEvent>,
) {
    if pointer.entity() == *previous {
        return;
    }

    if let Some(entity) = *previous {
        events.send(UpdateBarVisibilityEvent::new(entity, POINTER_BAR_ID, false));
    }
    if let Some(entity) = pointer.entity() {
        events.send(UpdateBarVisibilityEvent::new(entity, POINTER_BAR_ID, true));
    }

    *previous = pointer.entity();
}
