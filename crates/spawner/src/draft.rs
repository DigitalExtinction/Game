#![allow(clippy::forget_non_drop)] // Needed because of #[derive(Bundle)]

//! This module implements a Bevy plugin for drafting new objects on the map.
//! An entity marked with components [`DraftAllowed`] and [`DraftReady`] is
//! automatically handled and visualized by the plugin.

use bevy::pbr::NotShadowReceiver;
use bevy::scene::SceneInstance;
use bevy::{pbr::NotShadowCaster, prelude::*};
use de_core::{
    gamestate::GameState,
    objects::{MovableSolid, ObjectTypeComponent, StaticSolid},
    state::AppState,
};
use de_index::{ColliderWithCache, PreciseIndexSet, QueryCollider, SpatialQuery};
use de_map::size::MapBounds;
use de_objects::{AssetCollection, SceneType, Scenes, SolidObjects, EXCLUSION_OFFSET};
use de_types::{
    objects::{ActiveObjectType, BuildingType, ObjectType},
    projection::ToFlat,
};
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
        app.add_systems(OnEnter(AppState::InGame), insert_materials)
            .add_systems(OnExit(AppState::InGame), cleanup)
            .add_systems(Update, new_draft.run_if(in_state(GameState::Playing)))
            .add_systems(
                PostUpdate,
                (update_draft, check_draft_loaded, update_draft_colour)
                    .run_if(in_state(GameState::Playing))
                    .after(PreciseIndexSet::Index),
            );
    }
}

/// Bundle to spawn a construction draft.
#[derive(Bundle)]
pub struct DraftBundle {
    object_type: ObjectTypeComponent,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    draft: DraftAllowed,
    ready: DraftReady,
}

impl DraftBundle {
    pub fn new(building_type: BuildingType, transform: Transform) -> Self {
        Self {
            object_type: ObjectType::Active(ActiveObjectType::Building(building_type)).into(),
            transform,
            global_transform: transform.into(),
            visibility: Visibility::Inherited,
            computed_visibility: ComputedVisibility::HIDDEN,
            draft: DraftAllowed::default(),
            ready: DraftReady::default(),
        }
    }
}

#[derive(Component, Default)]
pub struct DraftAllowed(bool);

impl DraftAllowed {
    pub fn allowed(&self) -> bool {
        self.0
    }
}

#[derive(Component, Default)]
struct DraftReady(bool);

type Solids<'w, 's> = SpatialQuery<'w, 's, Entity, Or<(With<StaticSolid>, With<MovableSolid>)>>;

fn new_draft(
    mut commands: Commands,
    drafts: Query<(Entity, &ObjectTypeComponent), Added<DraftAllowed>>,
    scenes: Res<Scenes>,
) {
    for (entity, object_type) in drafts.iter() {
        commands.entity(entity).with_children(|parent| {
            parent.spawn(SceneBundle {
                scene: scenes.get(SceneType::Solid(**object_type)).clone(),
                ..Default::default()
            });
        });
    }
}

fn update_draft(
    mut drafts: Query<(&Transform, &ObjectTypeComponent, &mut DraftAllowed)>,
    solids: Solids,
    solid_objects: SolidObjects,
    bounds: Res<MapBounds>,
) {
    for (transform, &object_type, mut draft) in drafts.iter_mut() {
        let collider = QueryCollider::new(
            solid_objects.get(*object_type).collider(),
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
        if allowed != draft.0 {
            // Access the component mutably only when really needed for optimal
            // Bevy change detection.
            draft.0 = allowed
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
    entity: Entity,
    allowed: bool,
    standard_materials: &mut Query<&mut Handle<StandardMaterial>>,
    draft_materials: &DraftMaterials,
) {
    let Ok(mut material_handle) = standard_materials.get_mut(entity) else {
        return;
    };
    if allowed {
        *material_handle = draft_materials.valid_placement.clone();
    } else {
        *material_handle = draft_materials.invalid_placement.clone();
    }
}

/// Set the draft as changed when the scene is loaded in order to update the colour
fn check_draft_loaded(
    scene_spawner: Res<SceneSpawner>,
    instances: Query<(&Parent, &SceneInstance)>,
    mut drafts: Query<&mut DraftReady>,
) {
    for (parent, instance) in instances.iter() {
        if let Ok(mut draft) = drafts.get_mut(parent.get()) {
            let ready = scene_spawner.instance_is_ready(**instance);
            if draft.0 != ready {
                // Access the component mutably only when really needed for
                // optimal Bevy change detection.
                draft.0 = ready;
            }
        }
    }
}

type ChangedDraftQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static DraftAllowed,
        Ref<'static, DraftReady>,
        &'static Children,
    ),
    Or<(Changed<DraftAllowed>, Changed<DraftReady>)>,
>;

fn update_draft_colour(
    mut commands: Commands,
    draft_query: ChangedDraftQuery,
    scene_instances_query: Query<&SceneInstance>,
    mut standard_materials: Query<&mut Handle<StandardMaterial>>,
    scene_spawner: Res<SceneSpawner>,
    draft_materials: Res<DraftMaterials>,
) {
    for (draft, ready, children) in draft_query.iter() {
        if !ready.0 {
            continue;
        }

        let allowed = draft.allowed();

        for &child in children.into_iter() {
            // Find the scene instance which represents the draft object's model
            let Ok(scene_instance) = scene_instances_query.get(child) else {
                continue;
            };

            let entities = scene_spawner.iter_instance_entities(**scene_instance);
            for entity in entities {
                if ready.is_changed() {
                    commands
                        .entity(entity)
                        .insert((NotShadowCaster, NotShadowReceiver));
                }
                update_object_material(entity, allowed, &mut standard_materials, &draft_materials);
            }
        }
    }
}
