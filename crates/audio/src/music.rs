use bevy::{asset::LoadState, prelude::*};
use bevy_kira_audio::{
    prelude::{Audio, AudioSource, Volume},
    AudioControl,
};
use de_conf::Configuration;
use de_core::state::AppState;
use iyes_progress::prelude::*;

pub(crate) struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::AppLoading), setup)
            .add_systems(
                Update,
                load.track_progress().run_if(in_state(AppState::AppLoading)),
            )
            .add_systems(OnExit(AppState::AppLoading), start);
    }
}

#[derive(Resource)]
struct Tracks(Handle<AudioSource>);

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(Tracks(server.load("audio/music/menu_loop.mp3")));
}

fn load(server: Res<AssetServer>, tracks: Res<Tracks>) -> Progress {
    match server.get_load_state(&tracks.0) {
        Some(LoadState::Loaded) => true.into(),
        Some(LoadState::NotLoaded) | Some(LoadState::Loading) => false.into(),
        _ => panic!("Unexpected loading state."),
    }
}

fn start(audio: Res<Audio>, tracks: Res<Tracks>, config: Res<Configuration>) {
    if !config.audio().music_enabled() {
        return;
    }

    audio
        .play(tracks.0.clone())
        .looped()
        .with_volume(Volume::Amplitude(config.audio().music_volume().into()));
}
