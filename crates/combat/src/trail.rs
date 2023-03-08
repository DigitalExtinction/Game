use std::time::Duration;

use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey, NotShadowCaster, NotShadowReceiver},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};
use de_core::{
    baseset::GameSet, cleanup::DespawnOnGameExit, gamestate::GameState, state::AppState,
};
use parry3d::query::Ray;

const TRAIL_LIFESPAN: Duration = Duration::from_millis(500);
const TRAIL_THICKNESS: f32 = 0.1;

pub(crate) struct TrailPlugin;

impl Plugin for TrailPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<TrailMaterial>::default())
            .add_event::<TrailEvent>()
            .add_system(setup.in_schedule(OnEnter(AppState::InGame)))
            .add_system(cleanup.in_schedule(OnExit(AppState::InGame)))
            .add_system(
                spawn
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_system(
                update
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

pub(crate) struct TrailEvent(Ray);

impl TrailEvent {
    /// Send this event to spawn a new trail. The trail will automatically fade
    /// out and disappear.
    ///
    /// # Arguments
    ///
    /// * `ray` - the trail originates at the ray origin. The trail ends at the
    ///   `ray.origin + ray.dir`.
    pub(crate) fn new(ray: Ray) -> Self {
        Self(ray)
    }

    fn ray(&self) -> &Ray {
        &self.0
    }
}

#[derive(Resource)]
struct MeshHandle(Handle<Mesh>);

#[derive(Component, Default)]
struct Trail(Duration);

impl Trail {
    fn tick(&mut self, duration: Duration) {
        self.0 += duration;
    }

    fn finished(&self) -> bool {
        self.0 >= TRAIL_LIFESPAN
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "560ab431-1a54-48b3-87ea-8de8d94ceafb"]
struct TrailMaterial {
    #[uniform(0)]
    start_time: f32,
}

impl TrailMaterial {
    /// # Arguments
    ///
    /// `start_time` - wrapped time since the application startup.
    fn new(start_time: f32) -> Self {
        Self { start_time }
    }
}

impl Material for TrailMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/trail.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

fn spawn(
    mut commands: Commands,
    mut materials: ResMut<Assets<TrailMaterial>>,
    time: Res<Time>,
    mesh: Res<MeshHandle>,
    mut events: EventReader<TrailEvent>,
) {
    for event in events.iter() {
        let material = materials.add(TrailMaterial::new(time.elapsed_seconds_wrapped()));

        commands.spawn((
            MaterialMeshBundle::<TrailMaterial> {
                mesh: mesh.0.clone(),
                material,
                transform: Transform {
                    translation: event.ray().origin.into(),
                    rotation: Quat::from_rotation_arc(Vec3::X, event.ray().dir.normalize().into()),
                    scale: Vec3::new(event.ray().dir.norm(), 1., 1.),
                },
                ..Default::default()
            },
            Trail::default(),
            DespawnOnGameExit,
            NotShadowCaster,
            NotShadowReceiver,
        ));
    }
}

fn update(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Trail)>) {
    for (entity, mut trail) in query.iter_mut() {
        trail.tick(time.delta());
        if trail.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(generate_trail_mesh());
    commands.insert_resource(MeshHandle(mesh));
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<MeshHandle>();
}

/// This generates a trail mesh starting at (0, 0, 0) and pointing towards +X
/// axis. The trail cross section is a cross.
fn generate_trail_mesh() -> Mesh {
    let positions = vec![
        // Vertical
        [0., TRAIL_THICKNESS, 0.],
        [0., -TRAIL_THICKNESS, 0.],
        [1., -TRAIL_THICKNESS, 0.],
        [1., TRAIL_THICKNESS, 0.],
        // Horizontal
        [0., 0., TRAIL_THICKNESS],
        [0., 0., -TRAIL_THICKNESS],
        [1., 0., -TRAIL_THICKNESS],
        [1., 0., TRAIL_THICKNESS],
    ];
    let normals = vec![
        // Vertical
        [0., 0., 1.],
        [0., 0., 1.],
        [0., 0., 1.],
        [0., 0., 1.],
        // Horizontal
        [0., 1., 0.],
        [0., 1., 0.],
        [0., 1., 0.],
        [0., 1., 0.],
    ];
    let uvs = vec![
        // Vertical
        [0., 1.],
        [0., -1.],
        [1., -1.],
        [1., 1.],
        // Horizontal
        [0., 1.],
        [0., -1.],
        [1., -1.],
        [1., 1.],
    ];

    let indices = Indices::U16(vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7]);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(indices));
    mesh
}
