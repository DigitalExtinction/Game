use std::net::SocketAddr;

use ahash::AHashSet;
use anyhow::Context;
use async_std::task;
use de_net::{setup_processor, Network, OutMessage};
use tracing::info;

const PORT: u16 = 8082;

pub fn start() {
    info!("Starting...");

    task::block_on(task::spawn(async {
        if let Err(error) = main_loop().await {
            eprintln!("{:?}", error);
        }
    }));
}

async fn main_loop() -> anyhow::Result<()> {
    let mut clients: AHashSet<SocketAddr> = AHashSet::new();

    let net = Network::bind(Some(PORT))
        .await
        .with_context(|| format!("Failed to bind on port {PORT}"))?;
    info!("Listening on port {}", PORT);

    let (mut communicator, processor) = setup_processor(net);
    task::spawn(processor.run());

    loop {
        let input = communicator.recv().await.context("Data receiving failed")?;
        clients.insert(input.source());

        let targets = clients
            .iter()
            .cloned()
            .filter(|&target| target != input.source())
            .collect();

        communicator
            .send(OutMessage::new(input.data(), targets))
            .await
            .context("Data sending failed")?;
    }
}
