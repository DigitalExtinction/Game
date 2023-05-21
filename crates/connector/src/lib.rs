use std::{net::SocketAddr, time::Duration};

use ahash::AHashSet;
use anyhow::Context;
use async_std::{prelude::FutureExt as StdFutureExt, task};
use de_net::{setup_processor, Network, OutMessage};
use futures::FutureExt;
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
        if let Ok(input_result) = communicator
            .recv()
            .timeout(Duration::from_millis(1000))
            .await
        {
            let input = input_result.context("Data receiving failed")?;
            clients.insert(input.source());
            let reliable = input.reliable();

            let targets = clients
                .iter()
                .cloned()
                .filter(|&target| target != input.source())
                .collect();

            communicator
                .send(OutMessage::new(input.data(), reliable, targets))
                .await
                .context("Data sending failed")?;
        };

        if let Some(result) = communicator.errors().now_or_never() {
            let error = result.context("Errors receiving failed")?;
            clients.remove(&error.target());
        }
    }
}
