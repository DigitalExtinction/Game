use anyhow::{bail, Result};
use async_std::path::PathBuf;
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use de_core::{log_full_error, state::AppState};
use de_gui::ToastEvent;
use futures_lite::future;
use iyes_progress::prelude::*;

use crate::{io::load_conf, Configuration};

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
struct LoadingTask(Task<Result<Configuration>>);

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<LoadingTask>();
}

fn start_loading(mut commands: Commands) {
    let task = IoTaskPool::get().spawn(async {
        let Some(base_conf_dir) = dirs::config_dir() else {
            bail!(
                "User's configuration directory cannot be established."
            )
        };
        let path = PathBuf::from(base_conf_dir)
            .join("DigitalExtinction")
            .join("conf.yaml");
        load_conf(path.as_path()).await
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
                    toasts.send(ToastEvent::new(format!(
                        "Configuration loading failed: {err}"
                    )));
                    let error: &dyn Error = err.as_ref();
                    log_full_error!(error);

                    commands.init_resource::<Configuration>();
                    true.into()
                }
            },
            None => false.into(),
        },
        None => false.into(),
    }
}
