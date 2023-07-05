use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use async_std::{prelude::FutureExt, task};
use de_net::Socket;
use futures::join;
use ntest::timeout;

use crate::common::{spawn_and_wait, term_and_wait};

mod common;

const SERVER_ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8082));

#[derive(Debug)]
struct ReceivedBuffer(Vec<Incomming>);

impl ReceivedBuffer {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn assert_confirmed(&self, id: u32) {
        assert!(
            self.0.iter().any(|incomming| {
                match incomming {
                    Incomming::Data { .. } => false,
                    Incomming::Confirm(confirmed) => id == *confirmed,
                }
            }),
            "datagram {id} was not confirmed"
        );
    }

    fn find_id(&self, filter_reliable: bool, filter_data: &[u8]) -> Option<u32> {
        self.0.iter().find_map(|incomming| match incomming {
            Incomming::Data { reliable, id, data } => {
                if *reliable == filter_reliable && data == filter_data {
                    Some(*id)
                } else {
                    None
                }
            }
            Incomming::Confirm(_) => None,
        })
    }

    async fn load(&mut self, net: &mut Socket, buf: &mut [u8; 1024]) {
        let (n, _) = net.recv(buf).await.unwrap();
        assert!(n >= 4);

        let mut id_bytes = [0u8; 4];

        if buf[0] & 128 > 0 {
            assert!(buf[0] == 128);
            assert!(buf[1] == 0);
            assert!(buf[2] == 0);
            assert!(buf[3] == 0);

            for i in (4..n - 2).step_by(3) {
                id_bytes[0] = 0;
                id_bytes[1] = buf[i];
                id_bytes[2] = buf[i + 1];
                id_bytes[3] = buf[i + 2];
                let id = u32::from_be_bytes(id_bytes);
                self.0.push(Incomming::Confirm(id));
            }
        } else {
            let reliable = buf[0] & 64 > 0;

            id_bytes[0] = 0;
            id_bytes[1] = buf[1];
            id_bytes[2] = buf[2];
            id_bytes[3] = buf[3];
            let id = u32::from_be_bytes(id_bytes);

            self.0.push(Incomming::Data {
                reliable,
                id,
                data: buf[4..n].to_vec(),
            });
        }
    }
}

#[derive(Debug)]
enum Incomming {
    Confirm(u32),
    Data {
        reliable: bool,
        id: u32,
        data: Vec<u8>,
    },
}

#[test]
#[timeout(5000)]
fn test() {
    let child = spawn_and_wait();

    async fn first(mut client: Socket, game_port: u16) {
        let server = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, game_port));

        let mut buffer = [0u8; 1024];

        let mut received = ReceivedBuffer::new();
        received.load(&mut client, &mut buffer).await;
        received.load(&mut client, &mut buffer).await;

        // [5, 1] -> FromGame::PeerJoined(1)
        let id = received.find_id(true, &[5, 1]).unwrap().to_be_bytes();
        // And send a confirmation
        client
            .send(server, &[128, 0, 0, 0, id[1], id[2], id[3]])
            .await
            .unwrap();

        let first_id = received.find_id(true, &[5, 6, 7, 8]).unwrap();

        let mut data = [22; 412];
        data[0] = 64; // Reliable
        client.send(server, &data).await.unwrap();

        let mut received = ReceivedBuffer::new();
        received.load(&mut client, &mut buffer).await;
        received.load(&mut client, &mut buffer).await;
        received.assert_confirmed(1447446);
        received.find_id(false, &[82, 83, 84]).unwrap();

        // Try to send invalid data -- wrong header
        client
            .send(server, &[128, 255, 0, 1, 1, 2, 3, 4])
            .await
            .unwrap();
        // Try to send invalid data -- wrong ID
        client
            .send(server, &[128, 0, 0, 1, 255, 2, 3, 4])
            .await
            .unwrap();

        // Two retries before we confirm.
        let mut received = ReceivedBuffer::new();
        received.load(&mut client, &mut buffer).await;
        assert_eq!(received.find_id(true, &[5, 6, 7, 8]).unwrap(), first_id);
        let mut received = ReceivedBuffer::new();
        received.load(&mut client, &mut buffer).await;
        assert_eq!(received.find_id(true, &[5, 6, 7, 8]).unwrap(), first_id);

        let id = first_id.to_be_bytes();
        // And send a confirmation
        client
            .send(server, &[128, 0, 0, 0, id[1], id[2], id[3]])
            .await
            .unwrap();

        client.send(server, &[64, 0, 0, 92, 16]).await.unwrap();
        client.send(server, &[64, 0, 0, 86, 23]).await.unwrap();
        let mut received = ReceivedBuffer::new();
        received.load(&mut client, &mut buffer).await;
        received.assert_confirmed(92);
        received.assert_confirmed(86);

        // No more redeliveries expected.
        assert!(client
            .recv(&mut buffer)
            .timeout(Duration::from_secs(2))
            .await
            .is_err());
    }

    async fn second(mut client: Socket, game_port: u16) {
        let server = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, game_port));

        let mut buffer = [0u8; 1024];

        client
            // Reliable
            .send(server, &[64, 0, 8, 7, 5, 6, 7, 8])
            .await
            .unwrap();

        let mut received = ReceivedBuffer::new();
        received.load(&mut client, &mut buffer).await;
        received.load(&mut client, &mut buffer).await;
        received.assert_confirmed(2055);
        let id = received.find_id(true, &[22; 408]).unwrap().to_be_bytes();
        // Sending confirmation

        client
            .send(server, &[128, 0, 0, 0, id[1], id[2], id[3]])
            .await
            .unwrap();

        client
            .send(
                server,
                // Anonymous message
                &[0, 0, 0, 0, 82, 83, 84],
            )
            .await
            .unwrap();

        let mut received = ReceivedBuffer::new();
        received.load(&mut client, &mut buffer).await;
        let id = received.find_id(true, &[16]).unwrap().to_be_bytes();
        client
            .send(server, &[128, 0, 0, 0, id[1], id[2], id[3]])
            .await
            .unwrap();

        let mut received = ReceivedBuffer::new();
        received.load(&mut client, &mut buffer).await;
        let id = received.find_id(true, &[23]).unwrap().to_be_bytes();
        client
            .send(server, &[128, 0, 0, 0, id[1], id[2], id[3]])
            .await
            .unwrap();

        assert!(client
            .recv(&mut buffer)
            .timeout(Duration::from_secs(2))
            .await
            .is_err());
    }

    task::block_on(task::spawn(async {
        let (first_client, game_port) = create_game().await;
        let second_client = join_game(game_port).await;
        join!(
            first(first_client, game_port),
            second(second_client, game_port)
        );
    }));

    term_and_wait(child);
}

async fn create_game() -> (Socket, u16) {
    let mut buffer = [0u8; 1024];

    let mut client = Socket::bind(None).await.unwrap();

    // [64 + 32] -> reliable + Peers::Server
    // [0, 0, 7] -> datagram ID = 7
    // [1 3] -> ToGame::OpenGame { max_players: 3 }
    client
        .send(SERVER_ADDR, &[64 + 32, 0, 0, 7, 1, 3])
        .await
        .unwrap();

    let mut received = ReceivedBuffer::new();
    received.load(&mut client, &mut buffer).await;

    assert_eq!(received.0.len(), 1);

    let port = {
        let Incomming::Data { reliable, id, data } = &(received.0)[0] else {
            panic!("Unexpected data received: {:?}", received);
        };

        assert!(reliable);

        // Confirm
        let id = id.to_be_bytes();
        client
            .send(SERVER_ADDR, &[128, 0, 0, 0, id[1], id[2], id[3]])
            .await
            .unwrap();

        // Decode bincode encoded port:
        // [1] -> FromServer::GameOpened
        // [p] or [261 p p] -> { port: p }
        assert_eq!(data[0], 1);
        if data.len() == 2 {
            data[1] as u16
        } else {
            assert_eq!(data.len(), 4);
            assert_eq!(data[1], 251);
            u16::from_be_bytes([data[2], data[3]])
        }
    };

    let mut received = ReceivedBuffer::new();
    received.load(&mut client, &mut buffer).await;
    received.assert_confirmed(7);

    let server = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));

    let mut received = ReceivedBuffer::new();
    received.load(&mut client, &mut buffer).await;

    // [2, 0] -> FromGame::Joined(0)
    let id = received.find_id(true, &[2, 0]).unwrap().to_be_bytes();
    client
        .send(server, &[128, 0, 0, 0, id[1], id[2], id[3]])
        .await
        .unwrap();

    (client, port)
}

async fn join_game(game_port: u16) -> Socket {
    let mut buffer = [0u8; 1024];

    let server = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, game_port));
    let mut client = Socket::bind(None).await.unwrap();

    // [64 + 32] -> reliable + Peers::Server
    // [0, 0, 3] -> datagram ID = 3
    // [1] -> ToGame::Join
    client.send(server, &[64 + 32, 0, 0, 3, 1]).await.unwrap();

    let mut received = ReceivedBuffer::new();
    received.load(&mut client, &mut buffer).await;
    received.load(&mut client, &mut buffer).await;
    received.assert_confirmed(3);

    // [2, 1] -> FromGame::Joined(1)
    let id = received.find_id(true, &[2, 1]).unwrap().to_be_bytes();
    client
        .send(server, &[128, 0, 0, 0, id[1], id[2], id[3]])
        .await
        .unwrap();

    client
}
