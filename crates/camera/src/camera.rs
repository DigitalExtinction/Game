use std::f32::consts::FRAC_PI_2;

use bevy::{
    ecs::query::Has,
    pbr::{CascadeShadowConfig, CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};
use de_conf::{CameraConf, Configuration};
use de_core::{
    cleanup::DespawnOnGameExit,
    events::ResendEventPlugin,
    gamestate::GameState,
    schedule::{InputSchedule, Movement, PreMovement},
    state::AppState,
};
use de_map::size::MapBounds;
use de_terrain::{TerrainCollider, MAX_ELEVATION};
use de_types::projection::ToAltitude;
use de_uom::{InverseSecond, Metre, Quantity, Radian, Second};
use iyes_progress::prelude::*;
use parry3d::{math::Vector, query::Ray};

/// Camera moves horizontally at speed `distance * CAMERA_HORIZONTAL_SPEED`.
const CAMERA_HORIZONTAL_SPEED: InverseSecond = Quantity::new_unchecked(2.0);
/// Minimum camera distance multiplied by this gives minimum temporary distance
/// from terrain. Forward/backward camera motion is smooth within this range.
/// Step adjustment is applied outside of this range.
const HARD_MIN_CAMERA_DISTANCE_FACTOR: f32 = 0.8;
/// Maximum camera distance multiplied by this gives maximum temporary distance
/// from terrain. Forward/backward camera motion is smooth within this range.
/// Step adjustment is applied outside of this range.
const HARD_MAX_CAMERA_DISTANCE_FACTOR: f32 = 1.1;
/// Camera moves along forward axis (zooming) at speed `distance *
/// CAMERA_VERTICAL_SPEED`.
const CAMERA_VERTICAL_SPEED: InverseSecond = Quantity::new_unchecked(2.0);
/// Do not zoom camera if it is within this distance of the desired distance.
const DISTANCE_TOLERATION: Metre = Quantity::new_unchecked(0.001);
/// Minimum camera tilt in radians.
const MIN_OFF_NADIR: Radian = Quantity::new_unchecked(0.001);
/// Maximum camera tilt in radians.
const MAX_OFF_NADIR: Radian = Quantity::new_unchecked(0.7 * FRAC_PI_2);
/// Never move camera focus point closer than this to a map edge.
const MAP_FOCUS_MARGIN: Metre = Quantity::new_unchecked(1.);
const SHADOWS_NUM_CASCADES: usize = 4;

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MoveFocusEvent>()
            .add_event::<MoveCameraHorizontallyEvent>()
            .add_event::<ZoomCameraEvent>()
            .add_event::<RotateCameraEvent>()
            .add_event::<TiltCameraEvent>()
            .add_plugins(ResendEventPlugin::<MoveFocusEvent>::default())
            .add_event::<FocusInvalidatedEvent>()
            .add_event::<UpdateTranslationEvent>()
            .add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(OnExit(AppState::InGame), cleanup)
            .add_systems(
                InputSchedule,
                (
                    handle_horizontal_events.in_set(CameraSet::MoveHorizontallEvent),
                    handle_zoom_events.in_set(CameraSet::ZoomEvent),
                    handle_rotate_events.in_set(CameraSet::RotateEvent),
                    handle_tilt_events.in_set(CameraSet::TiltEvent),
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PreMovement,
                (
                    update_focus
                        .run_if(on_event::<FocusInvalidatedEvent>())
                        .in_set(InternalCameraSet::UpdateFocus),
                    process_move_focus_events
                        .in_set(InternalCameraSet::MoveFocus)
                        .after(InternalCameraSet::UpdateFocus),
                    update_translation_handler
                        .run_if(on_event::<UpdateTranslationEvent>())
                        .after(InternalCameraSet::MoveFocus),
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Movement,
                (
                    zoom.in_set(InternalCameraSet::Zoom),
                    update_shadows
                        .after(InternalCameraSet::Zoom)
                        .run_if(resource_exists_and_changed::<CameraFocus>()),
                    pivot
                        .run_if(
                            resource_exists_and_changed::<DesiredOffNadir>()
                                .or_else(resource_exists_and_changed::<DesiredAzimuth>()),
                        )
                        .in_set(InternalCameraSet::Pivot),
                    move_horizontaly
                        // Zooming changes camera focus point so do it
                        // after other types of camera movement.
                        .after(InternalCameraSet::Zoom)
                        .after(InternalCameraSet::Pivot),
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                setup_shadows
                    .track_progress()
                    .run_if(in_state(GameState::Loading)),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub enum CameraSet {
    MoveHorizontallEvent,
    RotateEvent,
    TiltEvent,
    ZoomEvent,
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum InternalCameraSet {
    UpdateFocus,
    Zoom,
    Pivot,
    MoveFocus,
}

#[derive(Event)]
pub struct MoveFocusEvent {
    point: Vec2,
}

impl MoveFocusEvent {
    pub fn new(point: Vec2) -> Self {
        Self { point }
    }

    fn point(&self) -> Vec2 {
        self.point
    }
}

/// Send this event to (re)set camera horizontal movement.
#[derive(Event)]
pub struct MoveCameraHorizontallyEvent(Vec2);

impl MoveCameraHorizontallyEvent {
    /// # Arguments
    ///
    /// * `direction` - camera will move along the XZ plane in this direction.
    ///   This can be any vector, the longer the vector, the faster the
    ///   movement. Vector coordinates should preferably be -1, 0, or 1.
    pub fn new(direction: Vec2) -> Self {
        Self(direction)
    }

    fn direction(&self) -> Vec2 {
        self.0
    }
}

/// Send this event to rotate camera around the vertical (Y) axis.
#[derive(Event)]
pub struct RotateCameraEvent(f32);

impl RotateCameraEvent {
    /// # Arguments
    ///
    /// * `delta` - this value will be added to the camera azimuth angle.
    pub fn new(delta: f32) -> Self {
        Self(delta)
    }

    fn delta(&self) -> f32 {
        self.0
    }
}

/// Send this event to tilt the camera, i.e. to change elevation / off nadir.
#[derive(Event)]
pub struct TiltCameraEvent(f32);

impl TiltCameraEvent {
    /// # Arguments
    ///
    /// * `delta` - this value will be added to the camera off-nadir angle.
    pub fn new(delta: f32) -> Self {
        Self(delta)
    }

    fn delta(&self) -> f32 {
        self.0
    }
}

/// Send this event to zoom the camera.
#[derive(Event)]
pub struct ZoomCameraEvent(f32);

impl ZoomCameraEvent {
    /// # Arguments
    ///
    /// * `factor` - desired camera to terrain distance will be multiplied with
    ///   this factor.
    pub fn new(factor: f32) -> Self {
        Self(factor)
    }

    fn factor(&self) -> f32 {
        self.0
    }
}

#[derive(Resource)]
pub struct CameraFocus {
    point: Vec3,
    distance: Metre,
}

impl CameraFocus {
    pub fn point(&self) -> Vec3 {
        self.point
    }

    pub fn distance(&self) -> Metre {
        self.distance
    }

    fn update<V: Into<Vec3>>(&mut self, point: V, distance: Metre) {
        self.point = point.into();
        self.distance = distance;
    }

    fn set_point(&mut self, point: Vec3) {
        self.point = point;
    }

    fn update_distance(&mut self, forward_move: Metre) {
        self.distance -= forward_move;
    }
}

#[derive(Event)]
struct FocusInvalidatedEvent;

/// Send this event to (re)set camera translation based on focus point.
#[derive(Event)]
struct UpdateTranslationEvent;

#[derive(Default, Resource)]
struct HorizontalMovement {
    movement: Vec2,
}

impl HorizontalMovement {
    fn movement(&self) -> Vec2 {
        self.movement
    }

    fn set(&mut self, movement: Vec2) {
        self.movement = movement;
    }
}

#[derive(Resource)]
struct DesiredDistance(Metre);

impl DesiredDistance {
    fn distance(&self) -> Metre {
        self.0
    }

    fn zoom_clamped(&mut self, conf: &CameraConf, factor: f32) {
        self.0 = (if conf.scroll_inverted() {
            self.0 / factor
        } else {
            self.0 * factor
        })
        .clamp(conf.min_distance(), conf.max_distance())
    }
}

#[derive(Resource)]
struct DesiredOffNadir(Radian);

impl DesiredOffNadir {
    fn off_nadir(&self) -> Radian {
        self.0
    }

    fn tilt_clamped(&mut self, delta: Radian) {
        self.0 = (self.0 + delta).clamp(MIN_OFF_NADIR, MAX_OFF_NADIR);
    }
}

#[derive(Resource)]
struct DesiredAzimuth(Radian);

impl DesiredAzimuth {
    fn azimuth(&self) -> Radian {
        self.0
    }

    fn rotate(&mut self, delta: Radian) {
        self.0 = (self.0 + delta).normalized();
    }
}

fn setup(mut commands: Commands, conf: Res<Configuration>) {
    let conf = conf.camera();
    let distance = 0.6 * conf.min_distance() + 0.4 * conf.max_distance();

    commands.insert_resource(HorizontalMovement::default());
    commands.insert_resource(DesiredDistance(distance));
    commands.insert_resource(DesiredOffNadir(Radian::ZERO));
    commands.insert_resource(DesiredAzimuth(Radian::ZERO));
    commands.insert_resource(CameraFocus {
        point: Vec3::ZERO,
        distance,
    });
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, distance.into(), 0.0)
                .looking_at(Vec3::ZERO, -Vec3::Z),
            ..Default::default()
        },
        DespawnOnGameExit,
    ));

    commands.insert_resource(DirectionalLightShadowMap { size: 4096 });
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<HorizontalMovement>();
    commands.remove_resource::<DesiredDistance>();
    commands.remove_resource::<DesiredOffNadir>();
    commands.remove_resource::<DesiredAzimuth>();
    commands.remove_resource::<CameraFocus>();
}

fn setup_shadows(
    mut commands: Commands,
    focus: Res<CameraFocus>,
    light: Query<(Entity, Has<CascadeShadowConfig>), With<DirectionalLight>>,
) -> Progress {
    let Ok((entity, has_config)) = light.get_single() else {
        return false.into();
    };

    if has_config {
        return true.into();
    }

    let mut config = CascadeShadowConfigBuilder {
        num_cascades: SHADOWS_NUM_CASCADES,
        minimum_distance: 1.,
        ..default()
    }
    .build();
    update_shadows_config(focus.as_ref(), &mut config);
    commands.entity(entity).insert(config);
    true.into()
}

fn update_focus(
    mut focus: ResMut<CameraFocus>,
    terrain: TerrainCollider,
    camera_query: Query<&Transform, With<Camera3d>>,
) {
    let camera_transform = camera_query.single();
    let ray = Ray::new(
        camera_transform.translation.into(),
        camera_transform.forward().into(),
    );

    let intersection = terrain
        .cast_ray_bidir_msl(&ray, f32::INFINITY)
        .expect("Camera ray does not intersect MSL plane.");
    focus.update(
        ray.point_at(intersection.toi),
        Metre::try_from(intersection.toi).unwrap(),
    );
}

fn process_move_focus_events(
    mut in_events: EventReader<MoveFocusEvent>,
    mut focus: ResMut<CameraFocus>,
    terrain: TerrainCollider,
    mut out_events: EventWriter<UpdateTranslationEvent>,
) {
    let event = match in_events.iter().last() {
        Some(event) => event,
        None => return,
    };

    let origin = event.point().to_altitude(MAX_ELEVATION);
    let ray = Ray::new(origin.into(), Vector::new(0., -1., 0.));
    let intersection = terrain.cast_ray_msl(&ray, f32::INFINITY).unwrap();
    let focused_point = ray.origin + intersection.toi * ray.dir;
    focus.set_point(focused_point.into());
    out_events.send(UpdateTranslationEvent);
}

fn update_translation_handler(
    focus: Res<CameraFocus>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let mut transform = camera_query.single_mut();
    transform.translation = focus.point() + f32::from(focus.distance()) * transform.back();
}

fn move_horizontaly(
    conf: Res<Configuration>,
    horizontal_movement: Res<HorizontalMovement>,
    focus: Res<CameraFocus>,
    map_bounds: Res<MapBounds>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    mut event: EventWriter<FocusInvalidatedEvent>,
) {
    let direction = horizontal_movement.movement();
    if direction == Vec2::ZERO {
        return;
    }

    let mut transform = camera_query.single_mut();
    let conf = conf.camera();
    let distance_factor = focus
        .distance()
        .clamp(conf.min_distance(), conf.max_distance());
    let time_delta = Second::try_from(time.delta().as_secs_f32()).unwrap();
    let delta_scalar: f32 = (time_delta * CAMERA_HORIZONTAL_SPEED * distance_factor).into();
    let delta_vec = (transform.rotation * direction.extend(0.)) * delta_scalar;

    let margin = Vec3::new(MAP_FOCUS_MARGIN.into(), 0., MAP_FOCUS_MARGIN.into());
    let focus_msl: Vec3 = focus.point().to_msl();
    let map_bounds = map_bounds.aabb().to_msl();
    let min_delta_vec = (Vec3::from(map_bounds.mins) - focus_msl + margin).min(Vec3::ZERO);
    let max_delta_vec = (Vec3::from(map_bounds.maxs) - focus_msl - margin).max(Vec3::ZERO);
    transform.translation += delta_vec.clamp(min_delta_vec, max_delta_vec);
    event.send(FocusInvalidatedEvent);
}

fn zoom(
    conf: Res<Configuration>,
    desired_distance: Res<DesiredDistance>,
    time: Res<Time>,
    mut focus: ResMut<CameraFocus>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let conf = conf.camera();
    let hard_min_distance = conf.min_distance() * HARD_MIN_CAMERA_DISTANCE_FACTOR;
    let hard_max_distance = conf.max_distance() * HARD_MAX_CAMERA_DISTANCE_FACTOR;

    let mut delta_scalar = focus.distance() - hard_max_distance;
    if delta_scalar <= Metre::ZERO {
        // Camera is not further than hard_max_distance => zoom out to
        // hard_min_distance (if necessary).
        delta_scalar = (focus.distance() - hard_min_distance).min(Metre::ZERO);
    }
    if delta_scalar == Metre::ZERO {
        // Camera is within hard_min_distance and hard_max_distance => move
        // smoothly to desired distance.

        let error = focus.distance() - desired_distance.distance();
        if error.abs() <= DISTANCE_TOLERATION {
            return;
        }

        let time_delta = Second::try_from(time.delta().as_secs_f32()).unwrap();
        let max_delta = time_delta * CAMERA_VERTICAL_SPEED * focus.distance();
        delta_scalar = error.clamp(-max_delta, max_delta);
    }

    let mut transform = camera_query.single_mut();
    let delta_vec = f32::from(delta_scalar) * transform.forward();
    transform.translation += delta_vec;
    focus.update_distance(delta_scalar);
}

fn update_shadows(focus: Res<CameraFocus>, mut shadows: Query<&mut CascadeShadowConfig>) {
    update_shadows_config(focus.as_ref(), shadows.single_mut().as_mut());
}

fn update_shadows_config(focus: &CameraFocus, shadows: &mut CascadeShadowConfig) {
    let distnace = focus.distance().inner();

    shadows.bounds[0] = 0.8 * distnace;

    let base = (4f32).powf(1. / (shadows.bounds.len() - 1) as f32);
    for i in 1..shadows.bounds.len() {
        shadows.bounds[i] = shadows.bounds[i - 1] * base.powi(i as i32);
    }
}

fn pivot(
    desired_off_nadir: Res<DesiredOffNadir>,
    desired_azimuth: Res<DesiredAzimuth>,
    focus: Res<CameraFocus>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let mut transform = camera_query.single_mut();
    transform.rotation = Quat::from_euler(
        EulerRot::YXZ,
        (-desired_azimuth.azimuth()).into(),
        (desired_off_nadir.off_nadir() - Radian::FRAC_PI_2).into(),
        0.,
    );
    transform.translation = focus.point() - f32::from(focus.distance()) * transform.forward();
}

fn handle_horizontal_events(
    mut movement: ResMut<HorizontalMovement>,
    mut events: EventReader<MoveCameraHorizontallyEvent>,
) {
    if let Some(event) = events.iter().last() {
        movement.set(event.direction());
    }
}

fn handle_zoom_events(
    conf: Res<Configuration>,
    mut events: EventReader<ZoomCameraEvent>,
    mut desired: ResMut<DesiredDistance>,
) {
    for event in events.iter() {
        desired.zoom_clamped(conf.camera(), event.factor());
    }
}

fn handle_tilt_events(
    mut events: EventReader<TiltCameraEvent>,
    mut desired: ResMut<DesiredOffNadir>,
) {
    for event in events.iter() {
        desired.tilt_clamped(Radian::ONE * event.delta());
    }
}

fn handle_rotate_events(
    mut events: EventReader<RotateCameraEvent>,
    mut desired: ResMut<DesiredAzimuth>,
) {
    for event in events.iter() {
        desired.rotate(Radian::ONE * event.delta());
    }
}
