use std::f32::consts::PI;

use bevy::{asset::LoadState, input::keyboard::KeyboardInput, prelude::*};
use bevy_kira_audio::{
    prelude::{Audio as KAudio, AudioSource as KAudioSource},
    AudioControl, AudioInstance,
};
use de_camera::CameraFocus;
use de_core::{baseset::GameSet, gamestate::GameState, state::AppState};
use enum_map::{enum_map, Enum, EnumMap};
use iyes_progress::{Progress, ProgressSystem};

// Linear falloff parameters

/// The start of the linear falloff for sounds, in units of camera zoom distance
/// from the camera focus
const SOUND_LINEAR_FALLOFF_START_RATIO: f32 = 0.7;
/// The end of the linear falloff for sounds, in units of camera zoom distance
/// from the camera focus
const SOUND_LINEAR_FALLOFF_END_RATIO: f32 = 1.0;
/// This is added to the camera zoom distance for sound falloff to make the range
/// larger with a very close zoom.
const SOUND_LINEAR_FALLOFF_BIAS: f32 = 20.0;
/// The calculated length of the linear falloff
const SOUND_LINEAR_FALLOFF_LENGTH: f32 =
    SOUND_LINEAR_FALLOFF_END_RATIO - SOUND_LINEAR_FALLOFF_START_RATIO;

// Angle occlusion parameters

/// The angle is calculated from a point this many units behind the camera
const SOUND_VIEW_ANGLE_OFFSET: f32 = 32.0;

pub(crate) struct SpatialSoundPlugin;

impl Plugin for SpatialSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaySpatialAudioEvent>()
            .insert_resource(SoundAttenuation {
                attenuation: None,
                occlusion: None,
            })
            .add_system(setup.in_schedule(OnEnter(AppState::AppLoading)))
            .add_system(load.track_progress().run_if(in_state(AppState::AppLoading)))
            .add_system(
                change_attenuation
                    .in_base_set(GameSet::Input)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_system(
                play.in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(GameState::Playing))
                    .run_if(on_event::<PlaySpatialAudioEvent>()),
            )
            .add_system(
                update_spatial
                    .after(play)
                    .in_base_set(GameSet::PostUpdate)
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
struct Sounds(EnumMap<Sound, Handle<KAudioSource>>);

#[derive(Clone, Copy)]
enum Attenuation {
    /// Linear attenuation based on the camera's focal point
    LinearFromFocus,
    /// Inverse square attenuation
    InverseSquare,
    /// Inverse square attenuation based on the camera's focal point
    InverseSquareFromFocus,
}

#[derive(Clone, Copy)]
enum Occlusion {
    /// Only play sounds inside the view
    HardFrustum,
    /// Adjust volume based on the angle from the camera
    Angle,
}

#[derive(Resource)]
struct SoundAttenuation {
    attenuation: Option<Attenuation>,
    occlusion: Option<Occlusion>,
}

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
                LoadState::Loaded => 1,
                LoadState::NotLoaded | LoadState::Loading => 0,
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

fn change_attenuation(
    mut key_events: EventReader<KeyboardInput>,
    mut attenuation: ResMut<SoundAttenuation>,
) {
    for key_code in key_events.iter().filter_map(|e| e.key_code) {
        match key_code {
            KeyCode::Key1 => attenuation.attenuation = None,
            KeyCode::Key2 => attenuation.attenuation = Some(Attenuation::LinearFromFocus),
            KeyCode::Key3 => attenuation.attenuation = Some(Attenuation::InverseSquare),
            KeyCode::Key4 => attenuation.attenuation = Some(Attenuation::InverseSquareFromFocus),

            KeyCode::Key8 => attenuation.occlusion = None,
            KeyCode::Key9 => attenuation.occlusion = Some(Occlusion::HardFrustum),
            KeyCode::Key0 => attenuation.occlusion = Some(Occlusion::Angle),

            _ => (),
        }
    }
}

fn calculate_volume_and_pan(
    camera: &GlobalTransform,
    focus: &CameraFocus,
    sound_position: Vec3,
    attenuation: &SoundAttenuation,
    camera_config: &Camera,
) -> (f64, f64) {
    let cam_right = camera.right();
    let sound_dir = (sound_position - camera.translation()).normalize();
    let pan = cam_right.dot(sound_dir) * 0.5 + 0.5;

    let attenuation_factor = match attenuation.attenuation {
        Some(Attenuation::LinearFromFocus) => {
            let distance_from_focus = focus.point().distance(sound_position);
            let biased_zoom_distance = focus.distance().inner() + SOUND_LINEAR_FALLOFF_BIAS;

            let distance_ratio = distance_from_focus / biased_zoom_distance;
            // Clamped linear function passing through (start, 1) and (end, 0) in X units of camera zoom distance.
            ((SOUND_LINEAR_FALLOFF_END_RATIO - distance_ratio) / SOUND_LINEAR_FALLOFF_LENGTH)
                .clamp(0.0, 1.0)
        }
        Some(Attenuation::InverseSquare) => {
            let distance_from_camera_squared =
                camera.translation().distance_squared(sound_position);
            // Anything closer than 70% of zoom distance is at full volume.
            let min_distance_squared = (0.7 * focus.distance().inner()).powi(2);

            (min_distance_squared / distance_from_camera_squared).min(1.)
        }
        Some(Attenuation::InverseSquareFromFocus) => {
            let distance_from_focus_squared = focus.point().distance_squared(sound_position);
            // Anything closer than 70% of zoom distance is at full volume.
            let min_distance_squared = (0.7 * focus.distance().inner()).powi(2);

            (min_distance_squared / distance_from_focus_squared).min(1.)
        }
        None => 1.,
    };

    let occlusion_factor = match attenuation.occlusion {
        Some(Occlusion::HardFrustum) => {
            if let Some(ndc) = camera_config.world_to_ndc(camera, sound_position) {
                if (-1.0..=1.0).contains(&ndc.x)
                    && (-1.0..=1.0).contains(&ndc.y)
                    && (0.0..=1.0).contains(&ndc.z)
                {
                    1.
                } else {
                    0.
                }
            } else {
                0.
            }
        }
        Some(Occlusion::Angle) => {
            // Volume is 1 from 0-25.5 degrees from the view direction, then linearly goes to 0 from 25.5-45 degrees.
            // Angle is calculated from a point slightly behind the camera.
            let camera_offset = camera.translation() + camera.back() * SOUND_VIEW_ANGLE_OFFSET;
            let angle = (sound_position - camera_offset).angle_between(camera.forward());

            // Slope parameter of linear function (-1 unit in 25.5 degrees or pi/8)
            const SLOPE: f32 = -1.0 / (PI / 8.);
            // Constant parameter of linear function
            const CONSTANT: f32 = 2.0;
            (SLOPE * angle + CONSTANT).clamp(0.0, 1.0)
        }
        None => 1.,
    };

    let volume = attenuation_factor * occlusion_factor;

    (volume as f64, pan as f64)
}

fn play(
    mut commands: Commands,
    camera: Query<(&GlobalTransform, &Camera)>,
    focus: Res<CameraFocus>,
    audio: Res<KAudio>,
    sounds: Res<Sounds>,
    attenuation: Res<SoundAttenuation>,
    mut play_events: EventReader<PlaySpatialAudioEvent>,
) {
    let (camera, camera_config) = camera.single();

    for PlaySpatialAudioEvent { sound, position } in &mut play_events {
        let (volume, pan) =
            calculate_volume_and_pan(camera, &focus, *position, &attenuation, camera_config);
        let handle = audio
            .play(sounds.0[*sound].clone())
            .with_volume(volume)
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
    camera: Query<(&GlobalTransform, &Camera)>,
    focus: Res<CameraFocus>,
    attenuation: Res<SoundAttenuation>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    let (camera, camera_config) = camera.single();

    for (entity, audio, transform) in &spatial_audios {
        let Some(audio_instance) = audio_instances.get_mut(audio) else {
            commands.entity(entity).despawn();
            continue;
        };

        let (volume, pan) = calculate_volume_and_pan(
            camera,
            &focus,
            transform.translation(),
            &attenuation,
            camera_config,
        );

        audio_instance.set_volume(volume, default());
        audio_instance.set_panning(pan, default());
    }
}
