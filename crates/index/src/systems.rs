//! Module with systems and a Bevy plugin for automatic entity indexing of
//! solid entities.

use bevy::{prelude::*, transform::TransformSystem};
use de_core::{
    objects::{MovableSolid, ObjectType, StaticSolid},
    state::GameState,
};
use de_objects::{ColliderCache, ObjectCache};
use iyes_loopless::prelude::*;
use parry3d::math::Isometry;

use super::index::EntityIndex;

type SolidEntityQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static ObjectType, &'static GlobalTransform),
    (
        Without<Indexed>,
        Or<(With<StaticSolid>, With<MovableSolid>)>,
    ),
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
    cache: Res<ObjectCache>,
    query: SolidEntityQuery,
) {
    for (entity, object_type, transform) in query.iter() {
        let position = Isometry::new(
            transform.translation.into(),
            transform.rotation.to_scaled_axis().into(),
        );
        let collider = cache.get_collider(*object_type).clone();
        index.insert(entity, collider, position);
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
