use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use de_core::fs::conf_dir;
use de_core::state::AppState;
use de_gui::ToastEvent;
use futures_lite::future;
use iyes_progress::prelude::*;
use tracing::error;

use crate::macros::ConfigLoadError;
use crate::Configuration;

pub(super) struct ConfPlugin;

impl Plugin for ConfPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(start_loading.in_schedule(OnEnter(AppState::AppLoading)))
            .add_system(cleanup.in_schedule(OnExit(AppState::AppLoading)))
            .add_system(
                poll_conf
                    .track_progress()
                    .run_if(in_state(AppState::AppLoading)),
            );
    }
}

#[derive(Resource)]
struct LoadingTask(Task<Result<Configuration, ConfigLoadError>>);

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<LoadingTask>();
}

fn start_loading(mut commands: Commands) {
    let task = IoTaskPool::get().spawn(async {
        let path = conf_dir().map_err(ConfigLoadError::from)?.join("conf.yaml");
        Configuration::load(path.as_path()).await
    });
    commands.insert_resource(LoadingTask(task));
}

fn poll_conf(
    mut commands: Commands,
    task: Option<ResMut<LoadingTask>>,
    conf: Option<Res<Configuration>>,
    mut toasts: EventWriter<ToastEvent>,
) -> Progress {
    if conf.is_some() {
        return true.into();
    }

    match task {
        Some(mut task) => match future::block_on(future::poll_once(&mut task.0)) {
            Some(result) => match result {
                Ok(configuration) => {
                    commands.insert_resource(configuration);
                    true.into()
                }
                Err(err) => {
                    error!("{err}");
                    toasts.send(ToastEvent::new("Configuration loading failed."));
                    commands.init_resource::<Configuration>();
                    true.into()
                }
            },
            None => false.into(),
        },
        None => false.into(),
    }
}
