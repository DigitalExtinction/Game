use std::time::Duration;

use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};
use de_core::{stages::GameStage, state::GameState};
use iyes_loopless::prelude::*;
use parry3d::query::Ray;

const TRAIL_LIFESPAN: Duration = Duration::from_millis(500);
const TRAIL_THICKNESS: f32 = 0.1;

pub(crate) struct TrailPlugin;

impl Plugin for TrailPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<TrailMaterial>::default())
            .add_event::<TrailEvent>()
            .add_enter_system(GameState::Loading, setup)
            .add_system_set_to_stage(
                GameStage::PostUpdate,
                SystemSet::new()
                    .with_system(spawn.run_in_state(GameState::Playing))
                    .with_system(update.run_in_state(GameState::Playing)),
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

    fn as_secs(&self) -> f32 {
        self.0.as_secs_f32()
    }

    fn finished(&self) -> bool {
        self.0 >= TRAIL_LIFESPAN
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Default)]
#[uuid = "560ab431-1a54-48b3-87ea-8de8d94ceafb"]
struct TrailMaterial {
    #[uniform(0)]
    time: f32,
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
    mesh: Res<MeshHandle>,
    mut events: EventReader<TrailEvent>,
) {
    for event in events.iter() {
        let material = materials.add(TrailMaterial::default());

        commands
            .spawn(MaterialMeshBundle::<TrailMaterial> {
                mesh: mesh.0.clone(),
                material,
                transform: Transform {
                    translation: event.ray().origin.into(),
                    rotation: Quat::from_rotation_arc(Vec3::X, event.ray().dir.normalize().into()),
                    scale: Vec3::new(event.ray().dir.norm(), 1., 1.),
                },
                ..Default::default()
            })
            .insert(Trail::default());
    }
}

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<TrailMaterial>>,
    mut query: Query<(Entity, &mut Trail, &Handle<TrailMaterial>)>,
) {
    for (entity, mut trail, handle) in query.iter_mut() {
        trail.tick(time.delta());
        if trail.finished() {
            commands.entity(entity).despawn();
        } else {
            materials.get_mut(handle).unwrap().time = trail.as_secs();
        }
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(generate_trail_mesh());
    commands.insert_resource(MeshHandle(mesh));
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
