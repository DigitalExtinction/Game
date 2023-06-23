use bevy::{asset::LoadState, prelude::*};
use de_conf::Configuration;
use de_core::state::AppState;
use iyes_progress::prelude::*;

pub(crate) struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(AppState::AppLoading)))
            .add_system(load.track_progress().run_if(in_state(AppState::AppLoading)))
            .add_system(start.in_schedule(OnExit(AppState::AppLoading)));
    }
}

#[derive(Resource)]
struct Tracks(Handle<AudioSource>);

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(Tracks(server.load("audio/music/menu_loop.mp3")));
}

fn load(server: Res<AssetServer>, tracks: Res<Tracks>) -> Progress {
    match server.get_load_state(&tracks.0) {
        LoadState::Loaded => true.into(),
        LoadState::NotLoaded | LoadState::Loading => false.into(),
        _ => panic!("Unexpected loading state."),
    }
}

fn start(audio: Res<Audio>, tracks: Res<Tracks>, config: Res<Configuration>) {
    if !config.audio().music_enabled() {
        return;
    }
    audio.play_with_settings(
        tracks.0.clone(),
        PlaybackSettings::LOOP.with_volume(config.audio().music_volume()),
    );
}
