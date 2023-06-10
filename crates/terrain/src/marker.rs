use bevy::{
    prelude::*,
    render::{
        primitives::{Aabb as BevyAabb, Frustum},
        view::VisibilitySystems,
    },
    utils::FloatOrd,
};
use de_core::{
    frustum, gamestate::GameState, objects::ObjectType, projection::ToFlat,
    visibility::VisibilityFlags,
};
use de_objects::SolidObjects;
use glam::Vec3A;
use parry2d::bounding_volume::Aabb;

use crate::shader::{Circle, Rectangle, TerrainMaterial, CIRCLE_CAPACITY, RECTANGLE_CAPACITY};

const RECTANGLE_MARKER_MARGIN: f32 = 1.;

pub(crate) struct MarkerPlugin;

impl Plugin for MarkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            update_markers::<CircleMarker>
                .in_base_set(CoreSet::PostUpdate)
                .run_if(in_state(GameState::Playing))
                .after(VisibilitySystems::CheckVisibility),
        )
        .add_system(
            update_markers::<RectangleMarker>
                .in_base_set(CoreSet::PostUpdate)
                .run_if(in_state(GameState::Playing))
                .after(VisibilitySystems::CheckVisibility),
        );
    }
}

/// A component representing the visibility of a terrain marker.
#[derive(Component, Default)]
pub struct MarkerVisibility(pub VisibilityFlags);

trait Marker {
    type Shape: Clone + Copy;
    const UNIFORM_CAPACITY: usize;

    fn as_shape(&self, position: Vec2) -> Self::Shape;
    fn apply_to_material(material: &mut TerrainMaterial, shapes: Vec<Self::Shape>);
}

/// This component configures a semi-transparent circle drawn on the terrain
/// surface below the entity.
#[derive(Component)]
pub struct CircleMarker {
    radius: f32,
}

impl CircleMarker {
    /// Crates a new circle marker.
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Marker for CircleMarker {
    type Shape = Circle;
    const UNIFORM_CAPACITY: usize = CIRCLE_CAPACITY;

    fn as_shape(&self, position: Vec2) -> Self::Shape {
        Circle::new(position, self.radius)
    }

    fn apply_to_material(material: &mut TerrainMaterial, shapes: Vec<Self::Shape>) {
        material.set_circle_markers(shapes);
    }
}

/// This component configures a semi-transparent rectangle drawn on the terrain
/// surface below the entity.
#[derive(Component)]
pub struct RectangleMarker {
    inverse_transform: Mat3,
    half_size: Vec2,
}

impl RectangleMarker {
    /// Creates a new rectangle marker.
    pub fn new(transform: &GlobalTransform, half_size: Vec2) -> Self {
        Self {
            inverse_transform: transform.compute_matrix().inverse().to_flat(),
            half_size,
        }
    }

    /// Creates a new rectangle marker with the size of the AABB plus a margin.
    pub fn from_aabb_transform(local_aabb: Aabb, transform: &GlobalTransform) -> Self {
        let half_extents: Vec2 = local_aabb.half_extents().into();
        Self::new(
            transform,
            half_extents + Vec2::ONE * RECTANGLE_MARKER_MARGIN,
        )
    }
}

impl Marker for RectangleMarker {
    type Shape = Rectangle;
    const UNIFORM_CAPACITY: usize = RECTANGLE_CAPACITY;

    fn as_shape(&self, _position: Vec2) -> Self::Shape {
        Rectangle::new(self.inverse_transform, self.half_size)
    }

    fn apply_to_material(material: &mut TerrainMaterial, shapes: Vec<Self::Shape>) {
        material.set_rectangle_markers(shapes);
    }
}

fn update_markers<M>(
    mut materials: ResMut<Assets<TerrainMaterial>>,
    solids: SolidObjects,
    camera: Query<(&Transform, &Frustum), With<Camera3d>>,
    terrains: Query<(&ComputedVisibility, &Handle<TerrainMaterial>)>,
    markers: Query<(
        &ObjectType,
        &ComputedVisibility,
        &GlobalTransform,
        &M,
        &MarkerVisibility,
    )>,
) where
    M: Marker + Component,
{
    let (eye, cam_frustum) = match camera.get_single() {
        Ok((transform, frustum)) => (transform.translation, frustum),
        Err(_) => return,
    };

    struct ShapeWithDist<S> {
        shape: S,
        distance_sq: FloatOrd,
    }

    let mut candidates = Vec::new();
    for (&object_type, circle_visibility, transform, marker, marker_visibility) in markers.iter() {
        if !circle_visibility.is_visible_in_hierarchy() {
            continue;
        }

        if !marker_visibility.0.visible() {
            continue;
        }

        let aabb = solids.get(object_type).collider().aabb();
        let aabb = BevyAabb {
            center: Vec3A::from(aabb.center()),
            half_extents: Vec3A::from(aabb.half_extents()),
        };

        if frustum::intersects_bevy(cam_frustum, transform, &aabb) {
            let translation = transform.translation();
            candidates.push(ShapeWithDist {
                shape: marker.as_shape(translation.to_flat()),
                distance_sq: FloatOrd(eye.distance_squared(translation)),
            });
        }
    }
    candidates.sort_unstable_by_key(|c| c.distance_sq);

    let shapes: Vec<M::Shape> = candidates
        .iter()
        .take(M::UNIFORM_CAPACITY)
        .map(|s| s.shape)
        .collect();

    for (terrain_visibility, material) in terrains.iter() {
        if !terrain_visibility.is_visible_in_hierarchy() {
            continue;
        }

        let material = materials.get_mut(material).unwrap();
        M::apply_to_material(material, shapes.clone());
    }
}
