use bevy::{asset::LoadState, audio::Volume, prelude::*};
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
        LoadState::Loaded => true.into(),
        LoadState::NotLoaded | LoadState::Loading => false.into(),
        _ => panic!("Unexpected loading state."),
    }
}

fn start(mut commands: Commands, tracks: Res<Tracks>, config: Res<Configuration>) {
    if !config.audio().music_enabled() {
        return;
    }

    let volume = Volume::new_relative(config.audio().music_volume());
    commands.spawn(AudioBundle {
        source: tracks.0.clone(),
        settings: PlaybackSettings::LOOP.with_volume(volume),
    });
}
