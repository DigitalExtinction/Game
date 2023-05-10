use std::path::PathBuf;

use bevy::prelude::*;
use de_core::fs::logs_dir;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter};

pub(crate) struct LogPlugin;

/// the handle for the guard (dropping it will disable the log writer)
#[derive(Resource)]
pub(crate) struct CurrentLogHandle {
    #[allow(dead_code)]
    guard: WorkerGuard,
}

impl Plugin for LogPlugin {
    fn build(&self, app: &mut App) {
        let dir: PathBuf = logs_dir()
            .map(|d| d.into())
            .unwrap_or(PathBuf::from("logs"));

        // for file name
        let dt = chrono::Local::now();
        let path: PathBuf = dt.format("%Y-%m-%d_%H-%M-%S.log").to_string().into();

        let file_appender = tracing_appender::rolling::never(dir.as_path(), path.as_path());

        let (non_blocking_log_writer, _guard) = tracing_appender::non_blocking(file_appender);

        let collector = tracing_subscriber::registry()
            .with(
                EnvFilter::builder()
                    // defaults to INFO if RUST_LOG not set
                    .with_default_directive(Level::INFO.into())
                    .from_env_lossy(),
            )
            .with(fmt::layer().with_writer(std::io::stdout))
            .with(fmt::layer().with_writer(non_blocking_log_writer));
        tracing::subscriber::set_global_default(collector)
            .expect("Unable to set a global collector");

        app.insert_resource(CurrentLogHandle { guard: _guard });

        info!("Logging to file {path:?} in directory: {dir:?}");
    }
}
