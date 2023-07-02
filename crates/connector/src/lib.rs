use anyhow::Context;
use async_std::task;
use de_net::Socket;
use tracing::{error, info};

use crate::server::MainServer;

mod game;
mod server;

const PORT: u16 = 8082;

pub fn start() {
    info!("Starting...");

    task::block_on(task::spawn(async {
        if let Err(error) = start_inner().await {
            error!("{:?}", error);
        }
    }));
}

async fn start_inner() -> anyhow::Result<()> {
    let socket = Socket::bind(Some(PORT))
        .await
        .with_context(|| format!("Failed to open network on port {PORT}"))?;
    info!("Listening on port {PORT}");

    let server = MainServer::start(socket);
    server.run().await
}
