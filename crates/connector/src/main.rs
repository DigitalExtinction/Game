use de_connector_lib::start;
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<(), String> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let result = start();
    if let Err(message) = result.as_ref() {
        error!(message);
    }

    result
}
