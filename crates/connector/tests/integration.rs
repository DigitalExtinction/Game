use async_std::task;
use de_net::Network;
use futures::join;
use ntest::timeout;

use crate::common::{spawn_and_wait, term_and_wait};

mod common;

#[test]
#[timeout(2000)]
fn test() {
    let child = spawn_and_wait();

    async fn first(client: &mut Network) {
        let mut buffer = [0u8; 1024];
        let (n, _) = client.recv(&mut buffer).await.unwrap();
        assert_eq!(&buffer[4..n], &[5, 6, 7, 8]);

        client
            .send("127.0.0.1:8082".parse().unwrap(), &[22; 412])
            .await
            .unwrap();

        let mut buffer = [0u8; 1024];
        let (n, _) = client.recv(&mut buffer).await.unwrap();

        // First 4 bytes are interpreted as datagram ID.
        assert_eq!(&buffer[4..n], &[81; 78]);
    }

    async fn second(client: &mut Network) {
        let mut buffer = [0u8; 1024];
        let (n, _) = client.recv(&mut buffer).await.unwrap();

        // First 4 bytes are interpreted as datagram ID.
        assert_eq!(&buffer[4..n], &[22; 408]);

        client
            .send("127.0.0.1:8082".parse().unwrap(), &[81; 82])
            .await
            .unwrap();
    }

    task::block_on(task::spawn(async {
        let mut first_client = Network::bind(None).await.unwrap();
        let mut second_client = Network::bind(None).await.unwrap();

        first_client
            .send("127.0.0.1:8082".parse().unwrap(), &[1, 3, 3, 7, 1, 2, 3, 4])
            .await
            .unwrap();

        second_client
            .send("127.0.0.1:8082".parse().unwrap(), &[7, 0, 8, 7, 5, 6, 7, 8])
            .await
            .unwrap();

        join!(first(&mut first_client), second(&mut second_client));
    }));

    term_and_wait(child);
}
