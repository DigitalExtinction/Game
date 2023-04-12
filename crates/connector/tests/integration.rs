use async_std::{prelude::*, task};
use de_net::Network;
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
        assert_eq!(&buffer[0..n], &[5, 6, 7, 8]);

        client
            .send("127.0.0.1:8082".parse().unwrap(), &[22; 412])
            .await
            .unwrap();

        let mut buffer = [0u8; 1024];
        let (n, _) = client.recv(&mut buffer).await.unwrap();
        assert_eq!(&buffer[0..n], &[81; 82]);
    }

    async fn second(client: &mut Network) {
        let mut buffer = [0u8; 1024];
        let (n, _) = client.recv(&mut buffer).await.unwrap();
        assert_eq!(&buffer[0..n], &[22; 412]);

        client
            .send("127.0.0.1:8082".parse().unwrap(), &[81; 82])
            .await
            .unwrap();
    }

    task::block_on(task::spawn(async {
        let mut first_client = Network::bind(None).await.unwrap();
        let mut second_client = Network::bind(None).await.unwrap();

        first_client
            .send("127.0.0.1:8082".parse().unwrap(), &[1, 2, 3, 4])
            .await
            .unwrap();

        second_client
            .send("127.0.0.1:8082".parse().unwrap(), &[5, 6, 7, 8])
            .await
            .unwrap();

        first(&mut first_client)
            .join(second(&mut second_client))
            .await;
    }));

    term_and_wait(child);
}
