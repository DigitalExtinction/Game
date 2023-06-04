use async_std::task;
use tracing::{error, info};

use crate::game::GameProcessor;

mod game;

const PORT: u16 = 8082;

pub fn start() {
    info!("Starting...");

    task::block_on(task::spawn(async {
        if let Err(error) = GameProcessor::start(PORT).await {
            error!("{:?}", error);
        }
    }));
}
