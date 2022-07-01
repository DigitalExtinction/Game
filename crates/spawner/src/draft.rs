#![allow(clippy::forget_non_drop)] // Needed because of #[derive(Bundle)]

//! This module implements a Bevy plugin for drafting new objects on the map.
//! An entity marked with a component [`Draft`] is automatically handled and
//! visualized by the plugin.

use bevy::prelude::*;
use de_core::{
    objects::{ActiveObjectType, BuildingType, MovableSolid, ObjectType, StaticSolid},
    projection::ToFlat,
    state::GameState,
};
use de_index::{ColliderWithCache, QueryCollider, SpatialQuery};
use de_map::size::MapBounds;
use de_objects::{ColliderCache, ObjectCache};
use iyes_loopless::prelude::*;
use parry2d::bounding_volume::BoundingVolume;
use parry3d::math::Isometry;

pub(crate) struct DraftPlugin;

impl Plugin for DraftPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new()
                .with_system(new_draft.run_in_state(GameState::Playing))
                .with_system(update_draft.run_in_state(GameState::Playing)),
        );
    }
}

/// Bundle to spawn a construction draft.
#[derive(Bundle)]
pub struct DraftBundle {
    object_type: ObjectType,
    transform: Transform,
    global_transform: GlobalTransform,
    draft: Draft,
}

impl DraftBundle {
    pub fn new(building_type: BuildingType, transform: Transform) -> Self {
        Self {
            object_type: ObjectType::Active(ActiveObjectType::Building(building_type)),
            transform,
            global_transform: transform.into(),
            draft: Draft::default(),
        }
    }
}

#[derive(Component, Default)]
pub struct Draft {
    allowed: bool,
}

impl Draft {
    pub fn allowed(&self) -> bool {
        self.allowed
    }
}

#[derive(Component)]
struct Ready;

type NonReadyDrafts<'w, 's> =
    Query<'w, 's, (Entity, &'static ObjectType), (With<Draft>, Without<Ready>)>;

type Solids<'w, 's> = SpatialQuery<'w, 's, Entity, Or<(With<StaticSolid>, With<MovableSolid>)>>;

fn new_draft(mut commands: Commands, drafts: NonReadyDrafts, cache: Res<ObjectCache>) {
    for (entity, object_type) in drafts.iter() {
        commands
            .entity(entity)
            .insert(Ready)
            .with_children(|parent| {
                parent.spawn_scene(cache.get(*object_type).scene());
            });
    }
}

fn update_draft(
    mut drafts: Query<(&Transform, &ObjectType, &mut Draft)>,
    solids: Solids,
    cache: Res<ObjectCache>,
    bounds: Res<MapBounds>,
) {
    for (transform, &object_type, mut draft) in drafts.iter_mut() {
        let collider = QueryCollider::new(
            cache.get_collider(object_type),
            Isometry::new(
                transform.translation.into(),
                transform.rotation.to_scaled_axis().into(),
            ),
        );

        let flat_aabb = collider.world_aabb().to_flat();
        let allowed = bounds.aabb().contains(&flat_aabb) && !solids.collides(&collider);
        if allowed != draft.allowed {
            // Access the component mutably only when really needed for optimal
            // Bevy change detection.
            draft.allowed = allowed
        }
    }
}
