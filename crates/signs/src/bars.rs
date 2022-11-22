use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::Indices,
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef},
    },
};
use de_camera::{CameraDistance, DistanceLabels};
use de_core::{
    objects::{Active, ObjectType},
    stages::GameStage,
    state::{AppState, GameState},
    visibility::{VisibilityFlags, VisibilityLabels},
};
use de_objects::{ColliderCache, ObjectCache};
use iyes_loopless::prelude::*;

use crate::{DISTANCE_FLAG_BIT, MAX_VISIBILITY_DISTANCE};

/// Vertical distance in meters between the bar center and the top of the
/// parent entity collider.
const BAR_HEIGHT: f32 = 2.;

pub(crate) struct BarsPlugin;

impl Plugin for BarsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<BarMaterial>::default())
            .add_event::<UpdateBarValueEvent>()
            .add_event::<UpdateBarVisibilityEvent>()
            .add_enter_system(GameState::Loading, setup)
            .add_system_set_to_stage(
                GameStage::PostUpdate,
                SystemSet::new()
                    .with_system(spawn.run_in_state(AppState::InGame))
                    .with_system(update_value.run_in_state(AppState::InGame))
                    .with_system(
                        update_visibility_events
                            .run_in_state(AppState::InGame)
                            .before(VisibilityLabels::Update),
                    )
                    .with_system(
                        update_visibility_distance
                            .run_in_state(AppState::InGame)
                            .before(VisibilityLabels::Update)
                            .after(DistanceLabels::Update),
                    ),
            );
    }
}

/// An event which changes value displayed on the entity bar.
pub struct UpdateBarValueEvent {
    entity: Entity,
    value: f32,
}

impl UpdateBarValueEvent {
    /// Crates new update event.
    ///
    /// # Panics
    ///
    /// May panic if the value is not between 0. and 1. (inclusive).
    pub fn new(entity: Entity, value: f32) -> Self {
        debug_assert!((0. ..=1.).contains(&value));
        Self { entity, value }
    }

    fn entity(&self) -> Entity {
        self.entity
    }

    fn value(&self) -> f32 {
        self.value
    }
}

pub struct UpdateBarVisibilityEvent {
    entity: Entity,
    id: u32,
    value: bool,
}

impl UpdateBarVisibilityEvent {
    /// Crates a new event which updates visibility of the entity bar.
    ///
    /// The bar is visible `value` of at least one `id` is `true`.
    ///
    /// # Arguments
    ///
    /// * `entity` - entity whose bar is to be updated
    ///
    /// * `id` - a number between 0 and 31 (inclusive).
    ///
    /// * `value` - whether to make the entity visible.
    ///
    /// # Panics
    ///
    /// May panic if `id` is larger or equal to 32.
    pub fn new(entity: Entity, id: u32, value: bool) -> Self {
        debug_assert!(id < 32);
        Self { entity, id, value }
    }

    fn entity(&self) -> Entity {
        self.entity
    }

    fn id(&self) -> u32 {
        self.id
    }

    fn value(&self) -> bool {
        self.value
    }
}

#[derive(Resource)]
struct BarMesh(Handle<Mesh>);

impl BarMesh {
    fn mesh(&self) -> Handle<Mesh> {
        self.0.clone()
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "66547498-fb0d-4fb6-a8e6-c792367e53d6"]
struct BarMaterial {
    #[uniform(0)]
    value: f32,
}

impl Default for BarMaterial {
    fn default() -> Self {
        Self { value: 1. }
    }
}

impl Material for BarMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/bar.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/bar.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

#[derive(Component)]
struct BarChild(Entity);

fn setup(mut commans: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commans.insert_resource(BarMesh(meshes.add(bar_mesh(1.5, 0.3))));
}

fn spawn(
    mut commands: Commands,
    cache: Option<Res<ObjectCache>>,
    mesh: Res<BarMesh>,
    mut materials: ResMut<Assets<BarMaterial>>,
    entities: Query<(Entity, &ObjectType), Added<Active>>,
) {
    for (entity, &object_type) in entities.iter() {
        let height = cache
            .as_ref()
            .unwrap()
            .get_collider(object_type)
            .aabb()
            .maxs
            .y
            + BAR_HEIGHT;
        let transform = Transform::from_translation(height * Vec3::Y);

        let material = materials.add(BarMaterial::default());

        let bar_entity = commands
            .spawn(MaterialMeshBundle::<BarMaterial> {
                mesh: mesh.mesh(),
                material,
                transform,
                visibility: Visibility { is_visible: false },
                ..Default::default()
            })
            .insert(NotShadowCaster)
            .insert(NotShadowReceiver)
            .insert(VisibilityFlags::default())
            .id();

        commands
            .entity(entity)
            .add_child(bar_entity)
            .insert(BarChild(bar_entity));
    }
}

fn update_value(
    mut materials: ResMut<Assets<BarMaterial>>,
    parents: Query<&BarChild, With<Active>>,
    bars: Query<&Handle<BarMaterial>>,
    mut events: EventReader<UpdateBarValueEvent>,
) {
    for event in events.iter() {
        if let Ok(child) = parents.get(event.entity()) {
            let handle = bars.get(child.0).unwrap();
            let material = materials.get_mut(handle).unwrap();
            material.value = event.value();
        }
    }
}

fn update_visibility_events(
    parents: Query<&BarChild, With<Active>>,
    mut bars: Query<&mut VisibilityFlags>,
    mut events: EventReader<UpdateBarVisibilityEvent>,
) {
    for event in events.iter() {
        if let Ok(child) = parents.get(event.entity()) {
            bars.get_mut(child.0)
                .unwrap()
                .update_visible(event.id(), event.value());
        }
    }
}

fn update_visibility_distance(
    parents: Query<(&BarChild, &CameraDistance), Changed<CameraDistance>>,
    mut bars: Query<&mut VisibilityFlags>,
) {
    for (child, distance) in parents.iter() {
        let invisible = distance.distance() > MAX_VISIBILITY_DISTANCE;
        let mut flags = bars.get_mut(child.0).unwrap();

        // Do not trigger change detection unnecessarily.
        if flags.invisible_value(DISTANCE_FLAG_BIT) != invisible {
            flags.update_invisible(DISTANCE_FLAG_BIT, invisible);
        }
    }
}

fn bar_mesh(width: f32, height: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-0.5 * width, 0.5 * height, 0.],
            [-0.5 * width, -0.5 * height, 0.],
            [0.5 * width, -0.5 * height, 0.],
            [0.5 * width, 0.5 * height, 0.],
        ],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![[0., 0., 1.], [0., 0., 1.], [0., 0., 1.], [0., 0., 1.]],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0., 0.], [0., 1.], [1., 1.], [1., 0.]],
    );

    mesh.set_indices(Some(Indices::U16(vec![0, 1, 2, 0, 2, 3])));
    mesh
}
