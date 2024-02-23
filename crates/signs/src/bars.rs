use std::time::Duration;

use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey, NotShadowCaster, NotShadowReceiver},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{Indices, MeshVertexAttribute, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, PrimitiveTopology, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError, VertexFormat,
        },
    },
};
use de_camera::{CameraDistance, DistanceSet};
use de_core::{
    objects::{Active, ObjectTypeComponent},
    state::AppState,
    visibility::{VisibilityFlags, VisibilitySet},
};
use de_objects::SolidObjects;

use crate::{DISTANCE_FLAG_BIT, MAX_VISIBILITY_DISTANCE, UPDATE_TIMER_FLAG_BIT};

/// Vertical distance in meters between the bar center and the top of the
/// parent entity collider.
const BAR_HEIGHT: f32 = 2.;

/// Duration that a bar is visible when its value is updated.
const UPDATE_VISIBILITY_DURATION: Duration = Duration::from_secs(3);

const ATTRIBUTE_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("Position", 732918835, VertexFormat::Float32x2);

pub(crate) struct BarsPlugin;

impl Plugin for BarsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<BarMaterial>::default())
            .add_event::<UpdateBarValueEvent>()
            .add_event::<UpdateBarVisibilityEvent>()
            .add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(OnExit(AppState::InGame), cleanup)
            .add_systems(
                PostUpdate,
                (
                    spawn,
                    update_value,
                    (
                        update_visibility_events,
                        update_visibility_distance.after(DistanceSet::Update),
                        update_visibility_timer,
                    )
                        .before(VisibilitySet::Update),
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

/// An event which changes value displayed on the entity bar.
#[derive(Event)]
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

#[derive(Event)]
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

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
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

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

#[derive(Component)]
struct BarChild(Entity);

#[derive(Component)]
struct BarUpdateTimer(Timer);

impl Default for BarUpdateTimer {
    fn default() -> Self {
        let mut timer = Timer::new(UPDATE_VISIBILITY_DURATION, TimerMode::Once);
        // Avoid triggering visibility after spawning.
        timer.tick(UPDATE_VISIBILITY_DURATION);
        Self(timer)
    }
}

fn setup(mut commans: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commans.insert_resource(BarMesh(meshes.add(bar_mesh(1.5, 0.3))));
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<BarMesh>();
}

fn spawn(
    mut commands: Commands,
    solids: SolidObjects,
    mesh: Res<BarMesh>,
    mut materials: ResMut<Assets<BarMaterial>>,
    entities: Query<(Entity, &ObjectTypeComponent), Added<Active>>,
) {
    for (entity, &object_type) in entities.iter() {
        let height = solids.get(*object_type).collider().aabb().maxs.y + BAR_HEIGHT;
        let transform = Transform::from_translation(height * Vec3::Y);

        let material = materials.add(BarMaterial::default());

        let bar_entity = commands
            .spawn((
                MaterialMeshBundle::<BarMaterial> {
                    mesh: mesh.mesh(),
                    material,
                    transform,
                    visibility: Visibility::Hidden,
                    ..Default::default()
                },
                NotShadowCaster,
                NotShadowReceiver,
                VisibilityFlags::default(),
                BarUpdateTimer::default(),
            ))
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
    mut bars: Query<(&Handle<BarMaterial>, &mut BarUpdateTimer)>,
    mut events: EventReader<UpdateBarValueEvent>,
) {
    for event in events.read() {
        if let Ok(child) = parents.get(event.entity()) {
            let (handle, mut timer) = bars.get_mut(child.0).unwrap();
            let material = materials.get_mut(handle).unwrap();
            material.value = event.value();

            timer.0.reset();
        }
    }
}

fn update_visibility_events(
    parents: Query<&BarChild, With<Active>>,
    mut bars: Query<&mut VisibilityFlags>,
    mut events: EventReader<UpdateBarVisibilityEvent>,
) {
    for event in events.read() {
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

fn update_visibility_timer(
    mut bars: Query<(&mut VisibilityFlags, &mut BarUpdateTimer)>,
    time: Res<Time>,
) {
    for (mut flags, mut timer) in bars.iter_mut() {
        if timer.0.elapsed().is_zero() && !flags.visible_value(UPDATE_TIMER_FLAG_BIT) {
            flags.update_visible(UPDATE_TIMER_FLAG_BIT, true);
        }

        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            flags.update_visible(UPDATE_TIMER_FLAG_BIT, false);
        }
    }
}

fn bar_mesh(width: f32, height: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
        ATTRIBUTE_POSITION,
        vec![
            [-0.5 * width, 0.5 * height],
            [-0.5 * width, -0.5 * height],
            [0.5 * width, -0.5 * height],
            [0.5 * width, 0.5 * height],
        ],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0., 0.], [0., 1.], [1., 1.], [1., 0.]],
    );

    mesh.insert_indices(Indices::U16(vec![0, 1, 2, 0, 2, 3]));
    mesh
}
