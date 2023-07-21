use bevy::prelude::*;
use bevy_kira_audio::{
    prelude::{Audio as KAudio, AudioSource as KAudioSource},
    AudioControl, AudioInstance,
};
use de_camera::CameraFocus;
use de_core::gamestate::GameState;

pub(crate) struct SpatialSoundPlugin;

impl Plugin for SpatialSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(start.run_if(in_state(GameState::Playing)))
            .add_system(
                update_spatial
                    .in_base_set(CoreSet::PostUpdate)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component, Default)]
pub struct SpatialSound;

#[derive(Component)]
pub struct Volume(pub f32);

impl Default for Volume {
    fn default() -> Self {
        Self(1.)
    }
}

trait VolumeOptionExt {
    fn volume(&self) -> f32;
}

impl VolumeOptionExt for Option<&Volume> {
    fn volume(&self) -> f32 {
        match self {
            Some(volume) => volume.0,
            None => 1.,
        }
    }
}

#[derive(Bundle, Default)]
pub struct SpatialSoundBundle {
    pub sound: Handle<KAudioSource>,
    pub spatial_sound: SpatialSound,
    pub volume: Volume,
}

fn calculate_volume_and_pan(
    camera: &GlobalTransform,
    focus: &CameraFocus,
    sound: &GlobalTransform,
) -> (f64, f64) {
    let cam_right = camera.right();
    let sound_dir = (sound.translation() - camera.translation()).normalize();
    let pan = cam_right.dot(sound_dir) * 0.5 + 0.5;

    let distance_from_camera = camera.translation().distance(sound.translation());
    let camera_zoom_distance = focus.distance().inner();
    let distance_attenuation =
        (1.0 - distance_from_camera / (camera_zoom_distance + 32.) + 0.5).clamp(0.0, 1.0);
    (distance_attenuation as f64, pan as f64)
}

type UninitializedSound<'s> = (
    Entity,
    &'s Handle<KAudioSource>,
    &'s GlobalTransform,
    Option<&'s Volume>,
);

fn start(
    mut commands: Commands,
    starts: Query<UninitializedSound, (With<SpatialSound>, Without<Handle<AudioInstance>>)>,
    camera: Query<&GlobalTransform, With<Camera>>,
    focus: Res<CameraFocus>,
    audio: Res<KAudio>,
) {
    let camera = camera.single();

    for (entity, sound, transform, sound_volume) in &starts {
        let (volume, pan) = calculate_volume_and_pan(camera, &focus, transform);
        let handle = audio
            .play(sound.clone())
            .with_volume(volume * sound_volume.volume() as f64)
            .with_panning(pan)
            .handle();

        commands.entity(entity).insert(handle);
    }
}

type InitializedSound<'s> = (
    Entity,
    &'s Handle<AudioInstance>,
    &'s GlobalTransform,
    Option<&'s Volume>,
);

fn update_spatial(
    mut commands: Commands,
    spatial_audios: Query<InitializedSound, With<SpatialSound>>,
    camera: Query<&GlobalTransform, With<Camera>>,
    focus: Res<CameraFocus>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    let camera = camera.single();

    for (entity, audio, transform, sound_volume) in &spatial_audios {
        let Some(audio_instance) = audio_instances.get_mut(audio) else {
            commands.entity(entity).despawn();
            continue;
        };

        let (volume, pan) = calculate_volume_and_pan(camera, &focus, transform);

        audio_instance.set_volume(volume * sound_volume.volume() as f64, default());
        audio_instance.set_panning(pan, default());
    }
}
