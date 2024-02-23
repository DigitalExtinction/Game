use std::time::Duration;

use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey, NotShadowCaster, NotShadowReceiver},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{Indices, MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};
use de_audio::spatial::{PlaySpatialAudioEvent, Sound};
use de_core::{
    cleanup::DespawnOnGameExit, gamestate::GameState, gconfig::GameConfig, state::AppState,
};
use de_messages::{NetProjectile, ToPlayers};
use de_multiplayer::{MessagesSet, NetRecvProjectileEvent, ToPlayersEvent};
use parry3d::query::Ray;

const TRAIL_LIFESPAN: Duration = Duration::from_millis(500);
const TRAIL_THICKNESS: f32 = 0.1;

pub(crate) struct TrailPlugin;

impl Plugin for TrailPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<TrailMaterial>::default())
            .add_event::<LocalLaserTrailEvent>()
            .add_event::<LaserTrailEvent>()
            .add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(OnExit(AppState::InGame), cleanup)
            .add_systems(
                PostUpdate,
                (
                    local_laser_trail
                        .before(MessagesSet::SendMessages)
                        .before(TrailSet::Trail),
                    remote_laser_trail.before(TrailSet::Trail),
                    laser_trail.in_set(TrailSet::Trail),
                    laser_sound.in_set(TrailSet::Trail),
                    update,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum TrailSet {
    Trail,
}

#[derive(Event)]
pub(crate) struct LocalLaserTrailEvent(Ray);

impl LocalLaserTrailEvent {
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
}

#[derive(Event)]
struct LaserTrailEvent(Ray);

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

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
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

fn local_laser_trail(
    config: Res<GameConfig>,
    mut in_events: EventReader<LocalLaserTrailEvent>,
    mut out_events: EventWriter<LaserTrailEvent>,
    mut net_events: EventWriter<ToPlayersEvent>,
) {
    for event in in_events.read() {
        out_events.send(LaserTrailEvent(event.0));

        if config.multiplayer() {
            net_events.send(ToPlayersEvent::new(ToPlayers::Projectile(
                NetProjectile::Laser {
                    origin: event.0.origin.into(),
                    direction: event.0.dir.into(),
                },
            )));
        }
    }
}

fn remote_laser_trail(
    mut in_events: EventReader<NetRecvProjectileEvent>,
    mut out_events: EventWriter<LaserTrailEvent>,
) {
    for event in in_events.read() {
        match **event {
            NetProjectile::Laser { origin, direction } => {
                out_events.send(LaserTrailEvent(Ray::new(origin.into(), direction.into())));
            }
        }
    }
}

fn laser_trail(
    mut commands: Commands,
    mut materials: ResMut<Assets<TrailMaterial>>,
    time: Res<Time>,
    mesh: Res<MeshHandle>,
    mut events: EventReader<LaserTrailEvent>,
) {
    for event in events.read() {
        let material = materials.add(TrailMaterial::new(time.elapsed_seconds_wrapped()));

        commands.spawn((
            MaterialMeshBundle::<TrailMaterial> {
                mesh: mesh.0.clone(),
                material,
                transform: Transform {
                    translation: event.0.origin.into(),
                    rotation: Quat::from_rotation_arc(Vec3::X, event.0.dir.normalize().into()),
                    scale: Vec3::new(event.0.dir.norm(), 1., 1.),
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

fn laser_sound(
    mut events: EventReader<LaserTrailEvent>,
    mut sound_events: EventWriter<PlaySpatialAudioEvent>,
) {
    for event in events.read() {
        sound_events.send(PlaySpatialAudioEvent::new(
            Sound::LaserFire,
            event.0.origin.into(),
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
    mesh.insert_indices(indices);
    mesh
}
