use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use async_std::{prelude::FutureExt, task};
use de_net::Network;
use futures::join;
use ntest::timeout;

use crate::common::{spawn_and_wait, term_and_wait};

mod common;

#[test]
#[timeout(5000)]
fn test() {
    const ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8082));
    let child = spawn_and_wait();

    async fn first(client: &mut Network) {
        let mut buffer = [0u8; 1024];
        let (n, _) = client.recv(&mut buffer).await.unwrap();
        assert_eq!(&buffer[4..n], &[5, 6, 7, 8]);

        let mut first_header = [0; 4];
        first_header.copy_from_slice(&buffer[..4]);

        let mut data = [22; 412];
        data[0] = 64; // Reliable
        client.send(ADDR, &data).await.unwrap();

        let mut buffer = [0u8; 1024];
        let (n, _) = client.recv(&mut buffer).await.unwrap();

        // Anonymous datagram (last header byte skipped)
        assert_eq!(&buffer[0..3], &[0, 0, 0]);
        assert_eq!(&buffer[4..n], &[82, 83, 84]);

        // Confirmation
        let (n, _) = client.recv(&mut buffer).await.unwrap();
        assert_eq!(&buffer[0..n], &[128, 0, 0, 0, 3, 3, 7, 22, 22, 22]);

        // Try to send invalid data -- wrong header
        client
            .send(ADDR, &[128, 255, 0, 1, 1, 2, 3, 4])
            .await
            .unwrap();
        // Try to send invalid data -- wrong ID
        client
            .send(ADDR, &[128, 0, 0, 1, 255, 2, 3, 4])
            .await
            .unwrap();

        // Two retries before we confirm.
        let (n, _) = client.recv(&mut buffer).await.unwrap();
        assert_eq!(&buffer[..4], &first_header);
        assert_eq!(&buffer[4..n], &[5, 6, 7, 8]);
        let (n, _) = client.recv(&mut buffer).await.unwrap();
        assert_eq!(&buffer[..4], &first_header);
        assert_eq!(&buffer[4..n], &[5, 6, 7, 8]);
        // And send a confirmation
        client
            .send(ADDR, &[128, 0, 0, 0, buffer[1], buffer[2], buffer[3]])
            .await
            .unwrap();

        // No more redeliveries expected.
        assert!(client
            .recv(&mut buffer)
            .timeout(Duration::from_secs(2))
            .await
            .is_err());
    }

    async fn second(client: &mut Network) {
        let mut buffer = [0u8; 1024];
        let (n, _) = client.recv(&mut buffer).await.unwrap();

        // First 4 bytes are interpreted as datagram ID.
        assert_eq!(&buffer[4..n], &[22; 408]);

        // Sending confirmation
        client
            .send(ADDR, &[128, 0, 0, 0, buffer[1], buffer[2], buffer[3]])
            .await
            .unwrap();

        client
            .send(
                ADDR,
                // Anonymous message
                &[0, 0, 0, 0, 82, 83, 84],
            )
            .await
            .unwrap();

        // Confirmation
        let (n, _) = client.recv(&mut buffer).await.unwrap();
        assert_eq!(&buffer[0..n], &[128, 0, 0, 0, 0, 8, 7]);

        assert!(client
            .recv(&mut buffer)
            .timeout(Duration::from_secs(2))
            .await
            .is_err());
    }

    task::block_on(task::spawn(async {
        let mut first_client = Network::bind(None).await.unwrap();
        let mut second_client = Network::bind(None).await.unwrap();

        first_client
            // Reliable
            .send(ADDR, &[64, 3, 3, 7, 1, 2, 3, 4])
            .await
            .unwrap();

        second_client
            // Reliable
            .send(ADDR, &[64, 0, 8, 7, 5, 6, 7, 8])
            .await
            .unwrap();

        join!(first(&mut first_client), second(&mut second_client));
    }));

    term_and_wait(child);
}
