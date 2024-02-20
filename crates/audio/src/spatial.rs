use std::f32::consts::PI;

use bevy::{asset::LoadState, prelude::*};
use bevy_kira_audio::{prelude::AudioSource, Audio, AudioControl, AudioInstance};
use de_camera::CameraFocus;
use de_conf::Configuration;
use de_core::{gamestate::GameState, state::AppState};
use enum_map::{enum_map, Enum, EnumMap};
use iyes_progress::{Progress, ProgressSystem};

// Angle occlusion parameters

/// The angle is calculated from a point this many units behind the camera
const SOUND_VIEW_ANGLE_OFFSET: f32 = 32.;

pub(crate) struct SpatialSoundPlugin;

impl Plugin for SpatialSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaySpatialAudioEvent>()
            .add_systems(OnEnter(AppState::AppLoading), setup)
            .add_systems(
                Update,
                load.track_progress().run_if(in_state(AppState::AppLoading)),
            )
            .add_systems(
                PostUpdate,
                (
                    play.run_if(on_event::<PlaySpatialAudioEvent>()),
                    update_spatial.after(play),
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Clone, Copy, Enum)]
pub enum Sound {
    Construct,
    Manufacture,
    DestroyBuilding,
    DestroyUnit,
    LaserFire,
}

#[derive(Event)]
pub struct PlaySpatialAudioEvent {
    pub sound: Sound,
    pub position: Vec3,
}

impl PlaySpatialAudioEvent {
    pub fn new(sound: Sound, position: Vec3) -> Self {
        Self { sound, position }
    }
}

#[derive(Resource)]
struct Sounds(EnumMap<Sound, Handle<AudioSource>>);

#[derive(Component, Default)]
struct SpatialSound;

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    use Sound::*;
    commands.insert_resource(Sounds(enum_map! {
        Construct => server.load("audio/sounds/construct.ogg"),
        Manufacture => server.load("audio/sounds/manufacture.ogg"),
        DestroyBuilding => server.load("audio/sounds/destruction_building.ogg"),
        DestroyUnit => server.load("audio/sounds/destruction_unit.ogg"),
        LaserFire => server.load("audio/sounds/laser.ogg"),
    }));
}

fn load(server: Res<AssetServer>, sounds: Res<Sounds>) -> Progress {
    Progress {
        done: sounds
            .0
            .values()
            .map(|handle| match server.get_load_state(handle) {
                Some(LoadState::Loaded) => 1,
                Some(LoadState::NotLoaded) | Some(LoadState::Loading) => 0,
                _ => panic!("Unexpected loading state."),
            })
            .sum(),
        total: sounds
            .0
            .len()
            .try_into()
            .expect("Trying to load an ungodly number of sounds"),
    }
}

fn calculate_volume_and_pan(
    camera: &GlobalTransform,
    focus: &CameraFocus,
    sound_position: Vec3,
) -> (f64, f64) {
    let cam_right = camera.right();
    let sound_dir = (sound_position - camera.translation()).normalize();
    let pan = cam_right.dot(sound_dir) * 0.5 + 0.5;

    // Simulates sounds becoming quieter when further away
    let attenuation_factor = {
        let distance_from_camera_squared = camera.translation().distance_squared(sound_position);
        // Anything closer than 70% of zoom distance is at full volume.
        let min_distance_squared = (0.7 * focus.distance().inner()).powi(2);

        min_distance_squared / distance_from_camera_squared
    };

    // Silences sounds whose sources are not in view
    let occlusion_factor = {
        // Volume is 1 from 0-25.5 degrees from the view direction, then linearly goes to 0 from 25.5-45 degrees.
        // Angle is calculated from a point slightly behind the camera.
        let camera_offset = camera.translation() + camera.back() * SOUND_VIEW_ANGLE_OFFSET;
        let angle = (sound_position - camera_offset).angle_between(camera.forward());

        // Slope parameter of linear function (-1 unit in 25.5 degrees or pi/8)
        const SLOPE: f32 = -1.0 / (PI / 8.);
        // Constant parameter of linear function
        const CONSTANT: f32 = 2.0;
        // Let's limit this to non-negative to avoid the weirdness that
        // occurs if both factors end up being negative at some point
        (SLOPE * angle + CONSTANT).max(0.0)
    };

    let volume = (attenuation_factor * occlusion_factor).clamp(0., 1.);

    (volume as f64, pan as f64)
}

fn play(
    mut commands: Commands,
    camera: Query<&GlobalTransform, With<Camera>>,
    focus: Res<CameraFocus>,
    audio: Res<Audio>,
    sounds: Res<Sounds>,
    config: Res<Configuration>,
    mut play_events: EventReader<PlaySpatialAudioEvent>,
) {
    if !config.audio().sound_enabled() {
        play_events.clear();
    }

    let camera = camera.single();
    let sound_volume = config.audio().sound_volume() as f64;

    for PlaySpatialAudioEvent { sound, position } in play_events.read() {
        let (volume, pan) = calculate_volume_and_pan(camera, &focus, *position);
        let handle = audio
            .play(sounds.0[*sound].clone())
            .with_volume(volume * sound_volume)
            .with_panning(pan)
            .handle();

        commands.spawn((
            TransformBundle::from_transform(Transform::from_translation(*position)),
            handle,
            SpatialSound,
        ));
    }
}

type InitializedSound<'s> = (Entity, &'s Handle<AudioInstance>, &'s GlobalTransform);

fn update_spatial(
    mut commands: Commands,
    spatial_audios: Query<InitializedSound, With<SpatialSound>>,
    camera: Query<&GlobalTransform, With<Camera>>,
    focus: Res<CameraFocus>,
    config: Res<Configuration>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    let camera = camera.single();
    let sound_volume = config.audio().sound_volume() as f64;

    for (entity, audio, transform) in &spatial_audios {
        let Some(audio_instance) = audio_instances.get_mut(audio) else {
            commands.entity(entity).despawn_recursive();
            continue;
        };

        let (volume, pan) = calculate_volume_and_pan(camera, &focus, transform.translation());

        audio_instance.set_volume(volume * sound_volume, default());
        audio_instance.set_panning(pan, default());
    }
}
