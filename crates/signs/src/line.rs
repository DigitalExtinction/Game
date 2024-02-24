use ahash::AHashMap;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use de_core::cleanup::DespawnOnGameExit;
use de_core::objects::Active;
use de_core::state::AppState;

/// Width of the line that goes to the pole.
const LINE_WIDTH: f32 = 1.;
/// Offset above mean sea level of the line, stopping z-fighting with the floor.
const LINE_OFFSET: Vec3 = Vec3::new(0., 1e-3, 0.);

pub(crate) struct LinePlugin;

impl Plugin for LinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<LineMaterial>::default())
            .add_event::<UpdateLineLocationEvent>()
            .add_event::<UpdateLineEndEvent>()
            .add_event::<UpdateLineVisibilityEvent>()
            .add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(OnExit(AppState::InGame), cleanup)
            .add_systems(
                PostUpdate,
                (
                    update_line_end
                        .run_if(on_event::<UpdateLineEndEvent>())
                        .in_set(LinesSet::LineEnd),
                    update_line_location
                        .run_if(on_event::<UpdateLineLocationEvent>())
                        .in_set(LinesSet::LocationEvents)
                        .after(LinesSet::LineEnd),
                    update_line_visibility
                        .run_if(on_event::<UpdateLineVisibilityEvent>())
                        .in_set(LinesSet::VisibilityEvents)
                        .after(LinesSet::LocationEvents),
                    owner_despawn
                        .in_set(LinesSet::Despawn)
                        .after(LinesSet::VisibilityEvents),
                )
                    .run_if(in_state(AppState::InGame)),
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
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
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

    /// A transform matrix from a plane with points at `(-1, 0, -1), (1, 0,
    /// -1), (1, 0, 1), (1, 0, -1)` to the line start and end with the
    /// `LINE_WIDTH`.
    fn transform(&self) -> Transform {
        let half_dir = 0.5 * (self.end - self.start);
        let norm_perp_dir = Vec3::new(-half_dir.z, half_dir.y, half_dir.x).normalize();
        let half_perp_dir = 0.5 * LINE_WIDTH * norm_perp_dir;

        let x_axis = half_dir.extend(0.);
        let y_axis = Vec4::Y;
        let z_axis = half_perp_dir.extend(0.);
        let w_axis = (self.start + half_dir + LINE_OFFSET).extend(1.);

        Transform::from_matrix(Mat4::from_cols(x_axis, y_axis, z_axis, w_axis))
    }
}

#[derive(Event)]
pub struct UpdateLineVisibilityEvent {
    owner: Entity,
    visible: bool,
}

impl UpdateLineVisibilityEvent {
    pub fn new(owner: Entity, visible: bool) -> Self {
        Self { owner, visible }
    }
}

#[derive(Event)]
pub struct UpdateLineLocationEvent {
    owner: Entity,
    location: LineLocation,
}

impl UpdateLineLocationEvent {
    pub fn new(owner: Entity, location: LineLocation) -> Self {
        Self { owner, location }
    }
}

#[derive(Event)]
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
    let line_mesh = meshes.add(Plane3d::default());
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
    for event in events.read() {
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
    for event in events.read() {
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
    for event in events.read() {
        if event.visible && !lines.0.contains_key(&event.owner) {
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

fn owner_despawn(
    mut commands: Commands,
    mut lines: ResMut<LineEntities>,
    mut removed: RemovedComponents<Active>,
) {
    for owner in removed.read() {
        if let Some(line) = lines.0.remove(&owner) {
            commands.entity(line).despawn_recursive();
        }
    }
}
