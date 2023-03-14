#![allow(clippy::forget_non_drop)] // Needed because of #[derive(Bundle)]

//! This module implements a Bevy plugin for drafting new objects on the map.
//! An entity marked with a component [`Draft`] is automatically handled and
//! visualized by the plugin.

use bevy::prelude::*;
use bevy::scene::SceneInstance;
use de_core::{
    baseset::GameSet,
    gamestate::GameState,
    objects::{ActiveObjectType, BuildingType, MovableSolid, ObjectType, StaticSolid},
    projection::ToFlat,
    state::AppState,
};
use de_index::{ColliderWithCache, IndexSet, QueryCollider, SpatialQuery};
use de_map::size::MapBounds;
use de_objects::{ColliderCache, ObjectCache, EXCLUSION_OFFSET};
use parry2d::{
    bounding_volume::{Aabb, BoundingVolume},
    math::Vector,
};
use parry3d::math::Isometry;

const MAP_PADDING: f32 = 2. * EXCLUSION_OFFSET + 0.1;
const MAP_OFFSET: Vector<f32> = Vector::new(MAP_PADDING, MAP_PADDING);

const VALID_PLACEMENT: Color = Color::rgba(0.2, 0.8, 0.2, 0.7);
const INVALID_PLACEMENT: Color = Color::rgba(0.86, 0.08, 0.24, 0.7);

pub(crate) struct DraftPlugin;

impl Plugin for DraftPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(insert_materials.in_schedule(OnEnter(AppState::InGame)))
            .add_system(cleanup.in_schedule(OnExit(AppState::InGame)))
            .add_system(
                new_draft
                    .in_base_set(GameSet::Update)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_system(
                update_draft
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(GameState::Playing))
                    .after(IndexSet::Index),
            )
            .add_system(
                check_draft_loaded
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(GameState::Playing))
                    .after(IndexSet::Index),
            )
            .add_system(
                update_draft_colour
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(GameState::Playing))
                    .after(IndexSet::Index),
            );
    }
}

/// Bundle to spawn a construction draft.
#[derive(Bundle)]
pub struct DraftBundle {
    object_type: ObjectType,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    draft: Draft,
}

impl DraftBundle {
    pub fn new(building_type: BuildingType, transform: Transform) -> Self {
        Self {
            object_type: ObjectType::Active(ActiveObjectType::Building(building_type)),
            transform,
            global_transform: transform.into(),
            visibility: Visibility::Inherited,
            computed_visibility: ComputedVisibility::HIDDEN,
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

type Solids<'w, 's> = SpatialQuery<'w, 's, Entity, Or<(With<StaticSolid>, With<MovableSolid>)>>;

fn new_draft(
    mut commands: Commands,
    drafts: Query<(Entity, &ObjectType), Added<Draft>>,
    cache: Res<ObjectCache>,
) {
    for (entity, object_type) in drafts.iter() {
        commands.entity(entity).with_children(|parent| {
            parent.spawn(SceneBundle {
                scene: cache.get(*object_type).scene(),
                ..Default::default()
            });
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
        let shrinked_map = {
            let aabb = bounds.aabb();
            Aabb::new(aabb.mins + MAP_OFFSET, aabb.maxs - MAP_OFFSET)
        };
        let allowed = shrinked_map.contains(&flat_aabb) && !solids.collides(&collider);
        if allowed != draft.allowed {
            // Access the component mutably only when really needed for optimal
            // Bevy change detection.
            draft.allowed = allowed
        }
    }
}

/// Materials for the invalid and valid placing states
#[derive(Clone, Resource)]
struct DraftMaterials {
    valid_placement: Handle<StandardMaterial>,
    invalid_placement: Handle<StandardMaterial>,
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<DraftMaterials>();
}

fn insert_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.insert_resource(DraftMaterials {
        valid_placement: materials.add(VALID_PLACEMENT.into()),
        invalid_placement: materials.add(INVALID_PLACEMENT.into()),
    });
}

// Assign the appropriate allowed to all entities in the spawned glb scene
fn update_object_material(
    allowed: bool,
    entities: impl Iterator<Item = Entity>,
    standard_materials: &mut Query<&mut Handle<StandardMaterial>>,
    draft_materials: &DraftMaterials,
) {
    for entity in entities {
        let Ok(mut material_handle) = standard_materials.get_mut(entity) else {
            continue;
        };
        if allowed {
            *material_handle = draft_materials.valid_placement.clone();
        } else {
            *material_handle = draft_materials.invalid_placement.clone();
        }
    }
}

/// Set the draft as changed when the scene is loaded in order to update the colour
fn check_draft_loaded(
    new_instances_query: Query<&Parent, Added<SceneInstance>>,
    mut draft_query: Query<&mut Draft>,
) {
    for parent in &new_instances_query {
        if let Ok(mut draft) = draft_query.get_mut(parent.get()) {
            draft.set_changed();
        }
    }
}

fn update_draft_colour(
    mut draft_query: Query<(&Draft, &Children), Changed<Draft>>,
    scene_instances_query: Query<&SceneInstance>,
    mut standard_materials: Query<&mut Handle<StandardMaterial>>,
    scene_spawner: Res<SceneSpawner>,
    draft_materials: Res<DraftMaterials>,
) {
    for (draft, children) in &mut draft_query {
        let allowed = draft.allowed();

        for &child in children.into_iter() {
            // Find the scene instance which represents the draft object's model
            let Ok(scene_instance) = scene_instances_query.get(child) else {
                continue;
            };

            let entities = scene_spawner.iter_instance_entities(**scene_instance);

            update_object_material(allowed, entities, &mut standard_materials, &draft_materials);
        }
    }
}
