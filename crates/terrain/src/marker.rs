use bevy::{
    prelude::*,
    render::{
        primitives::{Aabb, Frustum, Sphere},
        view::VisibilitySystems,
    },
    utils::FloatOrd,
};
use de_core::{objects::ObjectType, projection::ToFlat, state::GameState};
use de_objects::{ColliderCache, ObjectCache};
use glam::Vec3A;
use iyes_loopless::prelude::*;

use crate::shader::{Circle, TerrainMaterial, CIRCLE_CAPACITY};

pub(crate) struct MarkerPlugin;

impl Plugin for MarkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            update_markers
                .run_in_state(GameState::Playing)
                .after(VisibilitySystems::CheckVisibility),
        );
    }
}

/// A semi-transparent circle is drawn on the terrain surface below every
/// entity with this component.
#[derive(Component)]
pub struct CircleMarker {
    radius: f32,
}

impl CircleMarker {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }

    pub(crate) fn radius(&self) -> f32 {
        self.radius
    }
}

fn update_markers(
    mut materials: ResMut<Assets<TerrainMaterial>>,
    cache: Res<ObjectCache>,
    camera: Query<(&Transform, &Frustum), With<Camera3d>>,
    terrains: Query<(&ComputedVisibility, &Handle<TerrainMaterial>)>,
    markers: Query<(
        &ObjectType,
        &ComputedVisibility,
        &GlobalTransform,
        &CircleMarker,
    )>,
) {
    let (eye, frustum) = match camera.get_single() {
        Ok((transform, frustum)) => (transform.translation, frustum),
        Err(_) => return,
    };

    struct CircleWithDist {
        circle: Circle,
        distance_sq: FloatOrd,
    }

    let mut candidates = Vec::new();
    for (&object_type, circle_visibility, transform, marker) in markers.iter() {
        if !circle_visibility.is_visible_in_hierarchy() {
            continue;
        }

        let aabb = cache.get_collider(object_type).aabb();
        let aabb = Aabb {
            center: Vec3A::from(aabb.center()),
            half_extents: Vec3A::from(aabb.half_extents()),
        };

        if intersects_frustum(frustum, transform, &aabb) {
            let translation = transform.translation();
            candidates.push(CircleWithDist {
                circle: Circle::new(translation.to_flat(), marker.radius()),
                distance_sq: FloatOrd(eye.distance_squared(translation)),
            });
        }
    }
    candidates.sort_unstable_by_key(|c| c.distance_sq);

    let circles: Vec<Circle> = candidates
        .iter()
        .take(CIRCLE_CAPACITY)
        .map(|c| c.circle)
        .collect();

    for (terrain_visibility, material) in terrains.iter() {
        if !terrain_visibility.is_visible_in_hierarchy() {
            continue;
        }

        let material = materials.get_mut(material).unwrap();
        material.set_markers(circles.clone());
    }
}

fn intersects_frustum(frustum: &Frustum, transform: &GlobalTransform, aabb: &Aabb) -> bool {
    let model = transform.compute_matrix();
    let model_sphere = Sphere {
        center: model.transform_point3a(aabb.center),
        radius: transform.radius_vec3a(aabb.half_extents),
    };

    frustum.intersects_sphere(&model_sphere, false) && frustum.intersects_obb(aabb, &model, false)
}
