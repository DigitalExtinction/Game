use std::time::Duration;

use bevy::{
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use de_core::state::GameState;
use iyes_loopless::prelude::*;
use parry3d::query::Ray;

use crate::AttackingLabels;

/// All but bottom vertex of a hexagon. The hexagon lies on plane perpendicular
/// to X axis. It starts with the bottom-right point.
const HEXAGON_VERTICES: [[f32; 3]; 5] = [
    [0., -0.25, 0.433],
    [0., 0.25, 0.433],
    [0., 0.5, 0.],
    [0., 0.25, -0.433],
    [0., -0.25, -0.433],
];

/// Outwards normals of a hexagon whose base is given by [`HEXAGON_VERTICES`].
/// The bottom two edges are / surfaces are not included. It starts with normal
/// of the right (largest Z coordinate) surface.
const HEXAGON_NORMALS: [[f32; 3]; 4] = [
    [0., 0., 1.],
    [0., 0.866_025_4, 0.5],
    [0., 0.866_025_4, -0.5],
    [0., 0., 1.],
];

const BEAM_COLOR: Color = Color::rgba(0.2, 0., 1., 0.4);
const BEAM_DURATION: Duration = Duration::from_millis(500);

pub(crate) struct BeamPlugin;

impl Plugin for BeamPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnBeamEvent>()
            .add_enter_system(GameState::Playing, setup)
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::new()
                    .with_system(
                        spawn
                            .run_in_state(GameState::Playing)
                            .label(AttackingLabels::Animate),
                    )
                    .with_system(
                        despawn
                            .run_in_state(GameState::Playing)
                            .label(AttackingLabels::Animate),
                    ),
            );
    }
}

pub(crate) struct SpawnBeamEvent(Ray);

impl SpawnBeamEvent {
    // TODO docs
    pub(crate) fn new(ray: Ray) -> Self {
        Self(ray)
    }

    fn ray(&self) -> &Ray {
        &self.0
    }
}

struct BeamHandles {
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
}

#[derive(Component)]
struct Beam {
    timer: Timer,
}

impl Beam {
    fn new() -> Self {
        Self {
            timer: Timer::new(BEAM_DURATION, false),
        }
    }

    fn tick(&mut self, duration: Duration) -> bool {
        self.timer.tick(duration);
        self.timer.finished()
    }
}

fn spawn(
    mut commands: Commands,
    handles: Res<BeamHandles>,
    mut events: EventReader<SpawnBeamEvent>,
) {
    for event in events.iter() {
        commands
            .spawn_bundle(PbrBundle {
                mesh: handles.mesh.clone(),
                material: handles.material.clone(),
                transform: Transform {
                    translation: event.ray().origin.into(),
                    rotation: Quat::from_rotation_arc(Vec3::X, event.ray().dir.normalize().into()),
                    scale: Vec3::new(event.ray().dir.norm(), 0.1, 0.1),
                },
                ..Default::default()
            })
            .insert(Beam::new());
    }
}

fn despawn(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Beam)>) {
    for (entity, mut beam) in query.iter_mut() {
        if beam.tick(time.delta()) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: BEAM_COLOR,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    });
    let mesh = meshes.add(generate_beam_mesh());
    commands.insert_resource(BeamHandles { material, mesh });
}

fn generate_beam_mesh() -> Mesh {
    let mut positions = Vec::with_capacity(16);
    let mut normals = Vec::with_capacity(positions.len());
    let mut uvs = Vec::with_capacity(positions.len());

    for i in 0..4 {
        positions.push(HEXAGON_VERTICES[i]);
        positions.push(HEXAGON_VERTICES[i + 1]);
        normals.push(HEXAGON_NORMALS[i]);
        normals.push(HEXAGON_NORMALS[i]);
        uvs.push([0., i as f32 / 4.]);
        uvs.push([0., (i + 1) as f32 / 4.]);
    }
    for i in 0..positions.len() {
        positions.push([1., positions[i][1], positions[i][2]]);
        normals.push(normals[i]);
        uvs.push([1., uvs[i][1]]);
    }
    let indices = Indices::U16(
        (0..8)
            .step_by(2)
            .flat_map(|i| [i, i + 8, i + 1, i + 1, i + 8, i + 9])
            .collect(),
    );

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(indices));
    mesh
}
