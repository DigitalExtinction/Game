use std::net::SocketAddr;

use ahash::AHashSet;
use anyhow::Context;
use async_std::task;
use de_net::{Network, MAX_DATAGRAM_SIZE};
use futures::future::try_join_all;
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

    let mut net = Network::bind(Some(PORT))
        .await
        .with_context(|| format!("Failed to bind on port {PORT}"))?;
    info!("Listening on port {}", PORT);

    let mut buffer = [0u8; MAX_DATAGRAM_SIZE];

    loop {
        let (n, source) = net
            .recv(&mut buffer)
            .await
            .context("Data receiving failed")?;
        clients.insert(source);
        let send_futures = clients.iter().filter_map(|&target| {
            if target == source {
                None
            } else {
                Some(net.send(target, &buffer[0..n]))
            }
        });
        try_join_all(send_futures).await.context("Sending failed")?;
    }
}
