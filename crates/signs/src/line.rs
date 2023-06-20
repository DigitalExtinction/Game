use std::collections::hash_map::Entry;

use ahash::AHashMap;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use de_core::baseset::GameSet;
use de_core::cleanup::DespawnOnGameExit;
use de_core::state::AppState;

/// Width of the line that goes to the pole.
const LINE_WIDTH: f32 = 1.;
/// Offset above mean sea level of the line, stopping z-fighting with the floor.
const LINE_OFFSET: Vec3 = Vec3::new(0., 1e-3, 0.);

pub(crate) struct LinePlugin;

impl Plugin for LinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<LineMaterial>::default())
            .add_event::<UpdateLineLocationEvent>()
            .add_event::<UpdateLineEndEvent>()
            .add_event::<UpdateLineVisibilityEvent>()
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
                    .in_set(LinesSet::VisibilityEvents)
                    .after(LinesSet::LocationEvents),
            )
            .add_system(
                owner_despawn
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame))
                    .in_set(LinesSet::Despawn)
                    .after(LinesSet::VisibilityEvents),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum LinesSet {
    LineEnd,
    LocationEvents,
    VisibilityEvents,
    Despawn,
}

// Passed to the `rally_point.wgsl` shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "d0fae52d-f398-4416-9b72-9039093a6c34"]
pub struct LineMaterial {}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/rally_point.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

#[derive(Clone, Copy)]
pub struct LineLocation {
    start: Vec3,
    end: Vec3,
}

impl LineLocation {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Self { start, end }
    }

    /// A transform matrix from a plane with points at `(-0.5, 0. -0.5), (0.5, 0. -0.5),
    /// (0.5, 0., 0.5), (-0.5, 0., -0.5)` to the line start and end with the `LINE_WIDTH`.
    fn transform(&self) -> Transform {
        let line_direction = self.end - self.start;
        let perpendicular_direction =
            Vec3::new(-line_direction.z, line_direction.y, line_direction.x).normalize()
                * LINE_WIDTH;
        let x_axis = line_direction.extend(0.);
        let z_axis = perpendicular_direction.extend(0.);
        let w_axis = (self.start + line_direction / 2. + LINE_OFFSET).extend(1.);
        Transform::from_matrix(Mat4::from_cols(x_axis, Vec4::Y, z_axis, w_axis))
    }
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
    location: LineLocation,
}

impl UpdateLineLocationEvent {
    pub fn new(owner: Entity, location: LineLocation) -> Self {
        Self { owner, location }
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

#[derive(Resource)]
struct LineMesh(Handle<Mesh>, Handle<LineMaterial>);

#[derive(Resource, Default)]
struct LineEntities(AHashMap<Entity, Entity>);

#[derive(Resource, Default)]
struct LineLocations(AHashMap<Entity, LineLocation>);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    commands.init_resource::<LineEntities>();
    commands.init_resource::<LineLocations>();
    let line_mesh = meshes.add(shape::Plane::from_size(1.0).into());
    let line_material = materials.add(LineMaterial {});
    commands.insert_resource(LineMesh(line_mesh, line_material));
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<LineEntities>();
    commands.remove_resource::<LineLocations>();
    commands.remove_resource::<LineMesh>();
}

fn update_line_end(
    mut events: EventReader<UpdateLineEndEvent>,
    lines: Res<LineLocations>,
    mut line_location: EventWriter<UpdateLineLocationEvent>,
) {
    for event in &mut events {
        if let Some(old_location) = lines.0.get(&event.owner) {
            let location = LineLocation::new(old_location.start, event.end);
            line_location.send(UpdateLineLocationEvent::new(event.owner, location));
        }
    }
}

fn update_line_location(
    lines: Res<LineEntities>,
    mut events: EventReader<UpdateLineLocationEvent>,
    mut transforms: Query<&mut Transform>,
    mut line_locations: ResMut<LineLocations>,
) {
    for event in &mut events {
        line_locations.0.insert(event.owner, event.location);
        if let Some(line_entity) = lines.0.get(&event.owner) {
            let mut current_transform = transforms.get_mut(*line_entity).unwrap();
            *current_transform = event.location.transform()
        }
    }
}

fn update_line_visibility(
    mut events: EventReader<UpdateLineVisibilityEvent>,
    mut lines: ResMut<LineEntities>,
    line_locations: Res<LineLocations>,
    mut commands: Commands,
    line_mesh: Res<LineMesh>,
) {
    for event in &mut events {
        let line_entity = lines.0.entry(event.owner);
        if event.visible && matches!(line_entity, Entry::Vacant(_)) {
            let transform = line_locations
                .0
                .get(&event.owner)
                .map(|location| location.transform());
            let line_id = commands
                .spawn((
                    MaterialMeshBundle {
                        mesh: line_mesh.0.clone(),
                        material: line_mesh.1.clone(),
                        transform: transform.unwrap_or_default(),
                        ..default()
                    },
                    DespawnOnGameExit,
                ))
                .id();
            lines.0.insert(event.owner, line_id);
        } else if !event.visible {
            if let Some(line_entity) = lines.0.remove(&event.owner) {
                commands.entity(line_entity).despawn_recursive();
            }
        }
    }
}

fn owner_despawn(mut commands: Commands, mut lines: ResMut<LineEntities>) {
    lines.0.retain(|&owner, &mut line| {
        if commands.get_entity(owner).is_some() {
            return true;
        }

        commands.entity(line).despawn();
        false
    });
}
