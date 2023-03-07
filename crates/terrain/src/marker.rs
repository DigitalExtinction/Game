use bevy::{
    prelude::*,
    render::{
        primitives::{Aabb, Frustum},
        view::VisibilitySystems,
    },
    utils::FloatOrd,
};
use de_core::{
    frustum, gamestate::GameState, objects::ObjectType, projection::ToFlat,
    visibility::VisibilityFlags,
};
use de_objects::{ColliderCache, ObjectCache};
use glam::Vec3A;

use crate::shader::{Circle, TerrainMaterial, CIRCLE_CAPACITY};

pub(crate) struct MarkerPlugin;

impl Plugin for MarkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            update_markers
                .in_base_set(CoreSet::PostUpdate)
                .run_if(in_state(GameState::Playing))
                .after(VisibilitySystems::CheckVisibility),
        );
    }
}

/// This component configures a semi-transparent circle drawn on the terrain
/// surface below the entity.
#[derive(Component)]
pub struct CircleMarker {
    radius: f32,
    visibility: VisibilityFlags,
}

impl CircleMarker {
    /// Crates a new circle marker with default visibility flags.
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            visibility: VisibilityFlags::default(),
        }
    }

    pub fn visibility(&self) -> &VisibilityFlags {
        &self.visibility
    }

    pub fn visibility_mut(&mut self) -> &mut VisibilityFlags {
        &mut self.visibility
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
    let (eye, cam_frustum) = match camera.get_single() {
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

        if !marker.visibility().visible() {
            continue;
        }

        let aabb = cache.get_collider(object_type).aabb();
        let aabb = Aabb {
            center: Vec3A::from(aabb.center()),
            half_extents: Vec3A::from(aabb.half_extents()),
        };

        if frustum::intersects_bevy(cam_frustum, transform, &aabb) {
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
