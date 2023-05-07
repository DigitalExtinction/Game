use bevy::prelude::*;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::layer::SubscriberExt;

pub(crate) struct LogPlugin {}

/// the handle for the guard (dropping it will disable the log writer)
#[derive(Resource)]
pub(crate) struct CurrentLogHandle{
    #[allow(dead_code)]
    guard: WorkerGuard
}

impl Plugin for LogPlugin {
    fn build(&self, app: &mut App) {
        // for file name
        let dt = chrono::Local::now();

        let file_name = dt.format("%Y-%m-%d %H:%M:%S.log").to_string();
        let file_appender = tracing_appender::rolling::never("logs", &*file_name);

        let (non_blocking_log_writer, _guard) = tracing_appender::non_blocking(file_appender);

        let collector = tracing_subscriber::registry()
            .with(
                EnvFilter::from_default_env()
                    .add_directive(Level::TRACE.into())
                    .add_directive("async_io=info".parse().unwrap())
                    .add_directive("bevy_ecs=info".parse().unwrap())
                    .add_directive("naga=info".parse().unwrap())
                    .add_directive("polling=debug".parse().unwrap())
                    .add_directive("wgpu_core=warn".parse().unwrap())
                    .add_directive("wgpu_hal=info".parse().unwrap()),
            )
            .with(fmt::layer().with_writer(std::io::stdout))
            .with(fmt::layer().with_writer(non_blocking_log_writer));
        tracing::subscriber::set_global_default(collector).expect("Unable to set a global collector");

        app.insert_resource(CurrentLogHandle {
            guard:_guard
        });
    }
}
