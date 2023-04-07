// TODO rename the module
use std::net::SocketAddr;

use ahash::AHashMap;
use bevy::prelude::*;
use de_core::player::Player;

use crate::network::Network;

pub(crate) struct IoPlugin;

impl Plugin for IoPlugin {
    fn build(&self, app: &mut App) {
        // TODO
    }
}

struct Client {
    network: Network,
    players: AHashMap<Player, SocketAddr>,
}

impl Client {
    fn new(network: Network) -> Self {
        Self {
            network,
            // TODO
            players: AHashMap::new(),
        }
    }

    // TODO listen
    // TODO send
}

fn parse(player: Player, data: &[u8]) {
    // TODO change to trace
    info!("Parsing data from {}", player);
}

enum Message {
    Move { entity: u64, location: Vec3 },
}

#[cfg(test)]
mod tests {
    use async_std::task;

    use super::*;

    // TODO rename
    // TODO integration test?
    #[test]
    fn test_x() {
        async fn test() -> Result<(), ()> {
            let network = Network::bind().await.unwrap();

            println!("Port: {}", network.port().unwrap());

            let mut client = Client::new(network);

            //client.listen().await;

            Ok(())
        }

        let conf = task::block_on(test()).unwrap();

        assert!(false);
    }
}
