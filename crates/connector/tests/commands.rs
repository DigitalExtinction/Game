use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use async_std::{
    future::{timeout, TimeoutError},
    task,
};
use de_net::{
    self, ConnErrorReceiver, FromGame, FromServer, JoinError, OutPackage, PackageReceiver,
    PackageSender, Peers, Socket, ToGame, ToServer,
};
use ntest::timeout;

use crate::common::{spawn_and_wait, term_and_wait};

mod common;

macro_rules! check_response {
    ($comms:expr, $expect:pat) => {
        let mut response = $comms.recv().await;
        if response.len() != 1 {
            panic!("Unexpected number of messages: {response:?}");
        }

        let response = response.pop().unwrap();
        match response {
            $expect => (),
            _ => panic!("Unexpected response: {response:?}"),
        }
    };
}

#[test]
#[timeout(10_000)]
fn test() {
    let child = spawn_and_wait();

    task::block_on(task::spawn(async {
        let mut comms_a = Comms::init().await;
        let mut comms_b = Comms::init().await;
        let mut comms_c = Comms::init().await;
        let mut comms_d = Comms::init().await;

        comms_a.send(ToServer::OpenGame { max_players: 3 }).await;
        let mut response = comms_a.recv::<FromServer>().await;
        assert_eq!(response.len(), 1);
        let response = response.pop().unwrap();
        let game_port = match response {
            FromServer::GameOpened { port } => port,
            _ => panic!("Unexpected message: {response:?}"),
        };

        comms_a.port = game_port;
        comms_b.port = game_port;
        comms_c.port = game_port;
        comms_d.port = game_port;

        check_response!(comms_a, FromGame::Joined(1));

        comms_b.send(ToGame::Join).await;
        check_response!(comms_b, FromGame::Joined(2));
        check_response!(comms_a, FromGame::PeerJoined(2));

        comms_a.send(ToGame::Start).await;
        check_response!(comms_a, FromGame::Starting);
        check_response!(comms_b, FromGame::Starting);

        comms_c.send(ToGame::Join).await;
        check_response!(comms_c, FromGame::JoinError(JoinError::GameNotOpened));

        comms_a.send(ToGame::Initialized).await;
        // The other player is not yet initialized -> no message should be received.
        assert!(matches!(
            timeout(Duration::from_secs(1), comms_a.recv::<FromGame>()).await,
            Err(TimeoutError)
        ));
        assert!(matches!(
            timeout(Duration::from_secs(1), comms_b.recv::<FromGame>()).await,
            Err(TimeoutError)
        ));

        comms_b.send(ToGame::Initialized).await;
        check_response!(comms_a, FromGame::Started);
        check_response!(comms_b, FromGame::Started);

        comms_d.send(ToGame::Join).await;
        check_response!(comms_d, FromGame::JoinError(JoinError::GameNotOpened));

        assert!(comms_a.errors.is_empty());
        assert!(comms_b.errors.is_empty());
        assert!(comms_c.errors.is_empty());
    }));

    term_and_wait(child);
}

struct Comms {
    host: IpAddr,
    port: u16,
    sender: PackageSender,
    receiver: PackageReceiver,
    errors: ConnErrorReceiver,
}

impl Comms {
    async fn init() -> Self {
        let socket = Socket::bind(None).await.unwrap();
        let (sender, receiver, errors) = de_net::startup(
            |t| {
                task::spawn(t);
            },
            socket,
        );

        Self {
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 8082,
            sender,
            receiver,
            errors,
        }
    }

    async fn send<E>(&self, message: E)
    where
        E: bincode::Encode,
    {
        let addr = SocketAddr::new(self.host, self.port);
        let package = OutPackage::encode_single(&message, true, Peers::Server, addr).unwrap();
        self.sender.send(package).await.unwrap();
    }

    async fn recv<P>(&self) -> Vec<P>
    where
        P: bincode::Decode,
    {
        let package = self.receiver.recv().await.unwrap();
        let mut messages = Vec::new();
        for message in package.decode::<P>() {
            messages.push(message.unwrap());
        }
        messages
    }
}
