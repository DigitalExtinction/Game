use std::time::Instant;

use async_std::channel::{Receiver, TryRecvError};
use futures::future::FutureExt;

use self::{pending::PendingRouter, types::Datagram};
use crate::Network;

mod buffer;
mod pending;
mod queue;
mod types;

pub(crate) async fn start(mut network: Network, requests: Receiver<Datagram<'_>>) {
    let mut pending = PendingRouter::new();
    let mut buff = [0u8; 1024]; // TODO

    'outer: loop {
        let now = Instant::now();

        'recv: loop {
            match requests.try_recv() {
                Ok(datagram) => {
                    // TODO only if reliability was requested
                    let id = pending.push(datagram, now);
                    // TODO move to another method

                    buff[0..4].copy_from_slice(&id.get().to_be_bytes());
                    buff[4..4 + datagram.data().len()].copy_from_slice(datagram.data());

                    // TODO handle result
                    network.send(datagram.target(), &buff).await.unwrap();
                }
                Err(err) => match err {
                    TryRecvError::Empty => break 'recv,
                    TryRecvError::Closed => break 'outer,
                },
            }
        }

        let mut reschedules = pending.reschedule(now);
        while let Some(result) = reschedules.next() {
            // TODO resend or fail the connection
        }

        pending.cleanup(now);

        // TODO make sure that no data is skipped like this
        while let Some(result) = network.recv(&mut buff).now_or_never() {
            // TODO handle results
            let (n, source) = result.unwrap();

            // TODO
        }
    }
}
