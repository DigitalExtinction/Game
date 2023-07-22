use bevy::{asset::LoadState, prelude::*};
use bevy_kira_audio::{
    prelude::{Audio as KAudio, AudioSource as KAudioSource},
    AudioControl, AudioInstance,
};
use de_camera::CameraFocus;
use de_core::{baseset::GameSet, gamestate::GameState, state::AppState};
use enum_map::{enum_map, Enum, EnumMap};
use iyes_progress::{Progress, ProgressSystem};

/// The start of the linear falloff for sounds, in units of camera zoom distance
/// from the camera focus
const SOUND_FALLOFF_START_RATIO: f32 = 0.7;
/// The end of the linear falloff for sounds, in units of camera zoom distance
/// from the camera focus
const SOUND_FALLOFF_END_RATIO: f32 = 1.0;
/// This is added to the camera zoom distance for sound falloff to make the range
/// larger with a very close zoom.
const SOUND_FALLOFF_BIAS: f32 = 20.0;
/// The calculated length of the linear falloff
const SOUND_FALLOFF_LENGTH: f32 = SOUND_FALLOFF_END_RATIO - SOUND_FALLOFF_START_RATIO;

pub(crate) struct SpatialSoundPlugin;

impl Plugin for SpatialSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaySpatialAudioEvent>()
            .add_system(setup.in_schedule(OnEnter(AppState::AppLoading)))
            .add_system(load.track_progress().run_if(in_state(AppState::AppLoading)))
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

#[derive(Component, Default)]
struct SpatialSound;

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    use Sound::*;
    commands.insert_resource(Sounds(enum_map! {
        Construct => server.load("audio/sounds/construct.ogg"),
        Manufacture => server.load("audio/sounds/manufacture.ogg"),
        DestroyBuilding => server.load("audio/sounds/destruction_building.ogg"),
        DestroyUnit => server.load("audio/sounds/destruction_unit.ogg"),
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

/// Clamped linear function passing through (start, 1) and (end, 0).
/// `distance_ratio` is in units of camera zoom distance.
fn sound_falloff(distance_ratio: f32) -> f32 {
    ((SOUND_FALLOFF_END_RATIO - distance_ratio) / SOUND_FALLOFF_LENGTH).clamp(0.0, 1.0)
}

fn calculate_volume_and_pan(
    camera: &GlobalTransform,
    focus: &CameraFocus,
    sound_position: Vec3,
) -> (f64, f64) {
    let cam_right = camera.right();
    let sound_dir = (sound_position - camera.translation()).normalize();
    let pan = cam_right.dot(sound_dir) * 0.5 + 0.5;

    let distance_from_focus = focus.point().distance(sound_position);
    let biased_zoom_distance = focus.distance().inner() + SOUND_FALLOFF_BIAS;

    let distance_attenuation = sound_falloff(distance_from_focus / biased_zoom_distance);
    (distance_attenuation as f64, pan as f64)
}

fn play(
    mut commands: Commands,
    camera: Query<&GlobalTransform, With<Camera>>,
    focus: Res<CameraFocus>,
    audio: Res<KAudio>,
    sounds: Res<Sounds>,
    mut play_events: EventReader<PlaySpatialAudioEvent>,
) {
    let camera = camera.single();

    for PlaySpatialAudioEvent { sound, position } in &mut play_events {
        let (volume, pan) = calculate_volume_and_pan(camera, &focus, *position);
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
    camera: Query<&GlobalTransform, With<Camera>>,
    focus: Res<CameraFocus>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    let camera = camera.single();

    for (entity, audio, transform) in &spatial_audios {
        let Some(audio_instance) = audio_instances.get_mut(audio) else {
            commands.entity(entity).despawn();
            continue;
        };

        let (volume, pan) = calculate_volume_and_pan(camera, &focus, transform.translation());

        audio_instance.set_volume(volume, default());
        audio_instance.set_panning(pan, default());
    }
}
