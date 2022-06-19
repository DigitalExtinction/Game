//! Module with systems and a Bevy plugin for automatic entity indexing of
//! solid entities.

use bevy::{
    hierarchy::{Children, Parent},
    prelude::*,
    transform::TransformSystem,
};
use de_core::{
    objects::{MovableSolid, StaticSolid},
    state::GameState,
};
use iyes_loopless::prelude::*;
use parry3d::{
    bounding_volume::{BoundingVolume, AABB},
    math::Isometry,
    shape::Cuboid,
};

use super::index::EntityIndex;
use crate::shape::EntityShape;

type SolidEntityQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GlobalTransform,
        Option<&'static Handle<Mesh>>,
        Option<&'static Children>,
    ),
    (
        Without<Indexed>,
        Or<(With<StaticSolid>, With<MovableSolid>)>,
    ),
>;

type ChildQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        Option<&'static Handle<Mesh>>,
        Option<&'static Children>,
    ),
    With<Parent>,
>;

type MovedQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static GlobalTransform), (With<Indexed>, Changed<GlobalTransform>)>;

/// Bevy plugin which adds systems necessary for spatial indexing of solid
/// entities.
///
/// Only entities with marker component [`de_core::objects::StaticSolid`] or
/// [`de_core::objects::MovableSolid`] are indexed.
///
/// The systems are executed only in state
/// [`de_core::state::GameState::Playing`]. The systems automatically insert
/// newly spawned solid entities to the index, update their position when
/// [`bevy::prelude::GlobalTransform`] is changed and remove the entities from
/// the index when they are de-spawned.
///
/// Entity removal is done during stage
/// [`bevy::prelude::CoreStage::PostUpdate`], thus entities removed during or
/// after this stage might be missed and kept in the index even after their
/// de-spawning.
pub struct IndexPlugin;

impl Plugin for IndexPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Loading, setup)
            .add_exit_system(GameState::Playing, destruct)
            .add_system(insert.run_in_state(GameState::Playing))
            .add_system_to_stage(
                CoreStage::PostUpdate,
                remove.run_in_state(GameState::Playing),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                update
                    .run_in_state(GameState::Playing)
                    .after(TransformSystem::TransformPropagate),
            );
    }
}

#[derive(Component)]
struct Indexed;

fn setup(mut commands: Commands) {
    commands.insert_resource(EntityIndex::new());
}

fn destruct(mut commands: Commands) {
    commands.remove_resource::<EntityIndex>();
}

/// This system iterates over all not yet indexed entities, computes their
/// shape and adds them to the index.
///
/// Shape of the entities is minimum axis-aligned bounding box of the entity
/// mesh and all descendant entity meshes. The shape is represented with
/// [`parry3d::shape::Cuboid`].
fn insert(
    mut commands: Commands,
    mut index: ResMut<EntityIndex>,
    meshes: Res<Assets<Mesh>>,
    root_query: SolidEntityQuery,
    child_query: ChildQuery,
) {
    for (entity, transform, mesh_handle, children) in root_query.iter() {
        let shape = compute_entity_shape(&meshes, &child_query, mesh_handle, children);
        let position = Isometry::new(
            transform.translation.into(),
            transform.rotation.to_scaled_axis().into(),
        );
        index.insert(entity, shape, position);
        commands.entity(entity).insert(Indexed);
    }
}

fn remove(mut index: ResMut<EntityIndex>, removed: RemovedComponents<Indexed>) {
    for entity in removed.iter() {
        index.remove(entity);
    }
}

fn update(mut index: ResMut<EntityIndex>, moved: MovedQuery) {
    for (entity, transform) in moved.iter() {
        let position = Isometry::new(
            transform.translation.into(),
            transform.rotation.to_scaled_axis().into(),
        );
        index.update(entity, position);
    }
}

fn compute_entity_shape(
    meshes: &Assets<Mesh>,
    query: &ChildQuery,
    mesh_handle: Option<&Handle<Mesh>>,
    children: Option<&Children>,
) -> EntityShape {
    let identity = Transform::identity();

    let mut aabb = mesh_handle.and_then(|mesh| aabb_from_mesh(&identity, mesh, meshes));

    if let Some(children) = children {
        for &child in children.iter() {
            aabb = aabb_recursive(child, &identity, aabb, meshes, query);
        }
    }

    let aabb = aabb.expect("Solid entity with an empty AABB.");
    let translation = aabb.center();
    EntityShape::new(
        Cuboid::new(aabb.half_extents()),
        Isometry::translation(translation.x, translation.y, translation.z),
    )
}

fn aabb_recursive(
    entity: Entity,
    parent_transform: &Transform,
    mut aabb: Option<AABB>,
    meshes: &Assets<Mesh>,
    query: &ChildQuery,
) -> Option<AABB> {
    let (transform, mesh, children) = query
        .get(entity)
        .expect("Child entity could not be retrieved.");

    let transform = Transform::from_matrix(
        parent_transform
            .compute_matrix()
            .mul_mat4(&transform.compute_matrix()),
    );

    if let Some(mesh) = mesh {
        let child_aabb = aabb_from_mesh(&transform, mesh, meshes);
        aabb = match (aabb, child_aabb) {
            (Some(a), Some(b)) => Some(a.merged(&b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
    }

    if let Some(children) = children {
        for &child in children.iter() {
            aabb = aabb_recursive(child, &transform, aabb, meshes, query);
        }
    }

    aabb
}

fn aabb_from_mesh(
    transform: &Transform,
    mesh_handle: &Handle<Mesh>,
    meshes: &Assets<Mesh>,
) -> Option<AABB> {
    let mesh = meshes.get(mesh_handle).unwrap();
    mesh.compute_aabb().map(|aabb| {
        let position = Isometry::new(
            transform.translation.into(),
            transform.rotation.to_scaled_axis().into(),
        );
        AABB::new(aabb.min().into(), aabb.max().into())
            .scaled(&transform.scale.into())
            .transform_by(&position)
    })
}
