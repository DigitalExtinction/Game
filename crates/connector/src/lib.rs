use anyhow::Context;
use async_std::task;
use de_net::Socket;
use tracing::info;

use crate::server::MainServer;

mod clients;
mod game;
mod server;

const PORT: u16 = 8082;

pub fn start() -> Result<(), String> {
    info!("Starting...");
    task::block_on(task::spawn(async {
        start_inner().await.map_err(|error| format!("{:?}", error))
    }))
}

async fn start_inner() -> anyhow::Result<()> {
    let socket = Socket::bind(Some(PORT))
        .await
        .with_context(|| format!("Failed to open network on port {PORT}"))?;
    info!("Listening on port {PORT}");

    let server = MainServer::start(socket);
    server.run().await
}
