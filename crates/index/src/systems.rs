//! Module with systems and a Bevy plugin for automatic entity indexing of
//! solid entities.

use bevy::prelude::*;
use de_core::{
    objects::{MovableSolid, ObjectType, StaticSolid},
    stages::GameStage,
    state::{AppState, GameState},
};
use de_objects::{ColliderCache, ObjectCache};
use iyes_loopless::prelude::*;
use parry3d::math::Isometry;

use super::index::EntityIndex;
use crate::collider::LocalCollider;

type SolidEntityQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static ObjectType, &'static Transform),
    (
        Without<Indexed>,
        Or<(With<StaticSolid>, With<MovableSolid>)>,
    ),
>;

type MovedQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Transform), (With<Indexed>, Changed<Transform>)>;

/// Bevy plugin which adds systems necessary for spatial indexing of solid
/// entities.
///
/// Only entities with marker component [`de_core::objects::StaticSolid`] or
/// [`de_core::objects::MovableSolid`] are indexed.
///
/// The systems are executed only in state
/// [`de_core::state::GameState::Playing`]. The systems automatically insert
/// newly spawned solid entities to the index, update their position when
/// [`bevy::prelude::Transform`] is changed and remove the entities from the
/// index when they are de-spawned.
pub(crate) struct IndexPlugin;

impl Plugin for IndexPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::InGame, setup)
            .add_exit_system(AppState::InGame, cleanup)
            .add_system_to_stage(
                GameStage::PostUpdate,
                insert
                    .run_in_state(GameState::Playing)
                    .label(IndexLabel::Index),
            )
            .add_system_to_stage(
                GameStage::PostUpdate,
                remove
                    .run_in_state(GameState::Playing)
                    .label(IndexLabel::Index),
            )
            .add_system_to_stage(
                GameStage::PostMovement,
                update
                    .run_in_state(GameState::Playing)
                    .label(IndexLabel::Index),
            );
    }
}

#[derive(SystemLabel)]
pub enum IndexLabel {
    Index,
}

#[derive(Component)]
struct Indexed;

fn setup(mut commands: Commands) {
    commands.insert_resource(EntityIndex::new());
}

fn cleanup(mut commands: Commands) {
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
        let collider = LocalCollider::new(cache.get_collider(*object_type).clone(), position);
        index.insert(entity, collider);
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
