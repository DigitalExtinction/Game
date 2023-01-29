use anyhow::{bail, Result};
use async_std::path::PathBuf;
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use de_core::{log_full_error, state::AppState};
use futures_lite::future;
use iyes_loopless::prelude::*;
use iyes_progress::prelude::*;

use crate::{io::load_conf, Configuration};

pub(super) struct ConfPlugin;

impl Plugin for ConfPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::AppLoading, start_loading)
            .add_system(
                poll_conf
                    .track_progress()
                    .run_in_state(AppState::AppLoading),
            );
    }
}

#[derive(Resource)]
struct LoadingTask(Task<Result<Configuration>>);

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
                Err(error) => {
                    let error = error.context("Failed to load game configuration");
                    let error: &dyn Error = error.as_ref();
                    log_full_error!(error);
                    panic!("{}", error);
                }
            },
            None => false.into(),
        },
        None => false.into(),
    }
}
