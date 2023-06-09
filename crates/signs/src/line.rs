use std::collections::hash_map::Entry;

use ahash::AHashMap;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use de_core::baseset::GameSet;
use de_core::state::AppState;

/// Width of the line that goes to the pole.
const LINE_WIDTH: f32 = 1.;
/// Offset above mean sea level of the line, stopping z-fighting with the floor.
const LINE_OFFSET: Vec3 = Vec3::new(0., 1e-3, 0.);
/// Material configuration used for the lines to the factory spawn point
const LINE_MATERIAL: LineMaterial = LineMaterial {
    color: Color::rgba(0.0, 0.5, 0.0, 0.8),
    pointiness: 2.,
    speed: 3.,
    length: 1.,
    spacing: 0.5,
    fade: 3.,
    alpha_mode: AlphaMode::Blend,
};

pub(crate) struct LinePlugin;

impl Plugin for LinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<LineMaterial>::default())
            .add_event::<UpdateLineLocationEvent>()
            .add_event::<UpdateLineEndEvent>()
            .add_event::<UpdateLineVisibilityEvent>()
            .add_event::<SpawnLineEvent>()
            .add_event::<DespawnLineEvent>()
            .add_system(setup.in_schedule(OnEnter(AppState::InGame)))
            .add_system(cleanup.in_schedule(OnExit(AppState::InGame)))
            .add_system(
                update_line_end
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame))
                    .run_if(on_event::<UpdateLineEndEvent>())
                    .in_set(LinesSet::LineEnd),
            )
            .add_system(
                update_line_location
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame))
                    .run_if(on_event::<UpdateLineLocationEvent>())
                    .in_set(LinesSet::LocationEvents)
                    .after(LinesSet::LineEnd),
            )
            .add_system(
                update_line_visibility
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame))
                    .run_if(on_event::<UpdateLineVisibilityEvent>())
                    .in_set(LinesSet::VisibilityEvents),
            )
            .add_system(
                spawn_line
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame))
                    .run_if(on_event::<SpawnLineEvent>())
                    .in_set(LinesSet::SpawnLines)
                    .after(LinesSet::VisibilityEvents),
            )
            .add_system(
                despawn_line
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame))
                    .run_if(on_event::<DespawnLineEvent>())
                    .in_set(LinesSet::SpawnLines)
                    .after(LinesSet::VisibilityEvents),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum LinesSet {
    LineEnd,
    LocationEvents,
    VisibilityEvents,
    SpawnLines,
}

#[derive(Resource)]
struct LineMesh(Handle<Mesh>);

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.init_resource::<LineEntities>();
    commands.init_resource::<LineTransforms>();
    let line_mesh = meshes.add(shape::Plane::from_size(1.0).into());
    commands.insert_resource(LineMesh(line_mesh));
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<LineEntities>();
    commands.remove_resource::<LineTransforms>();
    commands.remove_resource::<LineMesh>();
}

pub struct UpdateLineVisibilityEvent {
    owner: Entity,
    visible: bool,
}

impl UpdateLineVisibilityEvent {
    pub fn new(owner: Entity, visible: bool) -> Self {
        Self { owner, visible }
    }
}

pub struct UpdateLineLocationEvent {
    owner: Entity,
    start: Vec3,
    end: Vec3,
}

impl UpdateLineLocationEvent {
    pub fn new(owner: Entity, start: Vec3, end: Vec3) -> Self {
        Self { owner, start, end }
    }
}

pub struct UpdateLineEndEvent {
    owner: Entity,
    end: Vec3,
}

impl UpdateLineEndEvent {
    pub fn new(owner: Entity, end: Vec3) -> Self {
        Self { owner, end }
    }
}

struct DespawnLineEvent {
    owner: Entity,
}

struct SpawnLineEvent(Transform, Entity);

#[derive(Resource, Default)]
struct LineEntities(AHashMap<Entity, Entity>);

#[derive(Resource, Default)]
struct LineTransforms(AHashMap<Entity, [Vec3; 2]>);

fn update_line_end(
    mut events: EventReader<UpdateLineEndEvent>,
    lines: Res<LineTransforms>,
    mut line_location: EventWriter<UpdateLineLocationEvent>,
) {
    for event in &mut events {
        if let Some([start, _end]) = lines.0.get(&event.owner) {
            line_location.send(UpdateLineLocationEvent::new(event.owner, *start, event.end));
        }
    }
}

fn update_line_location(
    lines: Res<LineEntities>,
    mut events: EventReader<UpdateLineLocationEvent>,
    mut transforms: Query<&mut Transform>,
    mut line_transforms: ResMut<LineTransforms>,
) {
    for event in &mut events {
        let transform = Transform::from_matrix(compute_line_transform(event.start, event.end));
        let positions = [event.start, event.end];
        line_transforms.0.insert(event.owner, positions);
        if let Some(line_entity) = lines.0.get(&event.owner) {
            let mut current_transform = transforms.get_mut(*line_entity).unwrap();
            *current_transform = transform
        }
    }
}

fn update_line_visibility(
    mut events: EventReader<UpdateLineVisibilityEvent>,
    mut lines: ResMut<LineEntities>,
    line_transforms: Res<LineTransforms>,
    mut spawn_line_events: EventWriter<SpawnLineEvent>,
    mut despawn_line_events: EventWriter<DespawnLineEvent>,
) {
    for event in &mut events {
        let line_entity = lines.0.entry(event.owner);
        if event.visible && matches!(line_entity, Entry::Vacant(_)) {
            if let Some([start, end]) = line_transforms.0.get(&event.owner) {
                let transform = Transform::from_matrix(compute_line_transform(*start, *end));
                spawn_line_events.send(SpawnLineEvent(transform, event.owner));
            }
        } else if !event.visible {
            let owner = event.owner;
            despawn_line_events.send(DespawnLineEvent { owner });
        }
    }
}

fn spawn_line(
    mut lines: ResMut<LineEntities>,
    mut events: EventReader<SpawnLineEvent>,
    mut commands: Commands,
    line_mesh: Res<LineMesh>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    for SpawnLineEvent(transform, owner) in &mut events {
        let line_id = commands
            .spawn(MaterialMeshBundle {
                mesh: line_mesh.0.clone(),
                transform: *transform,
                material: materials.add(LINE_MATERIAL),
                ..default()
            })
            .id();
        lines.0.insert(*owner, line_id);
    }
}

fn despawn_line(
    mut commands: Commands,
    mut events: EventReader<DespawnLineEvent>,
    mut lines: ResMut<LineEntities>,
) {
    for event in &mut events {
        if let Some(line_entity) = lines.0.remove(&event.owner) {
            commands.entity(line_entity).despawn_recursive();
        }
    }
}

/// A transform matrix from a plane with points at `(-0.5, 0. -0.5),(0.5, 0. -0.5),(0.5, 0. 0.5),(-0.5, 0.-0.5)` to the line start and end with the desired width
fn compute_line_transform(line_start: Vec3, end: Vec3) -> Mat4 {
    let line_direction = end - line_start;
    let perpendicular_direction =
        Vec3::new(-line_direction.z, line_direction.y, line_direction.x).normalize() * LINE_WIDTH;
    let x_axis = line_direction.extend(0.);
    let z_axis = perpendicular_direction.extend(0.);
    let w_axis = (line_start + line_direction / 2. + LINE_OFFSET).extend(1.);
    Mat4::from_cols(x_axis, Vec4::Y, z_axis, w_axis)
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/rally_point.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

// Passed to the `rally_point.wgsl` shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "d0fae52d-f398-4416-9b72-9039093a6c34"]
pub struct LineMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(0)]
    pointiness: f32,
    #[uniform(0)]
    speed: f32,
    #[uniform(0)]
    length: f32,
    #[uniform(0)]
    spacing: f32,
    #[uniform(0)]
    fade: f32,
    alpha_mode: AlphaMode,
}
