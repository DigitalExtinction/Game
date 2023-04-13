use std::{
    net::SocketAddr,
    num::NonZeroU32,
    time::{Duration, Instant},
};

use async_std::{
    channel::{Receiver, TryRecvError},
    prelude::*,
};
use futures::future::FutureExt;

use self::{
    buffer::DatagramBuffer,
    pending::PendingDatagrams,
    queue::{DatagramQueue, RescheduleError},
};
use crate::Network;

mod buffer;
mod pending;
mod queue;

pub(crate) async fn start(mut network: Network, requests: Receiver<(SocketAddr, &[u8])>) {
    let mut pending = PendingDatagrams::new();
    let mut buff = [0u8; 1024]; // TODO

    'outer: loop {
        let now = Instant::now();

        'recv: loop {
            match requests.try_recv() {
                Ok((target, data)) => {
                    // TODO only if reliability was requested
                    let id = pending.push(data, now);
                    // TODO move to another method

                    buff[0..4].copy_from_slice(&id.get().to_be_bytes());
                    buff[4..4 + data.len()].copy_from_slice(data);

                    // TODO handle result
                    network.send(target, &buff).await.unwrap();
                }
                Err(err) => match err {
                    TryRecvError::Empty => break 'recv,
                    TryRecvError::Closed => break 'outer,
                },
            }
        }

        'pending: loop {
            match pending.reschedule(now) {
                Ok((id, data)) => {
                    // TODO
                }
                Err(err) => match err {
                    RescheduleError::None => break 'pending,
                    RescheduleError::DatagramFailed(id) => {
                        // TODO
                        panic!("TODO");
                    }
                },
            }
        }

        // TODO make sure that no data is skipped like this
        while let Some(result) = network.recv(&mut buff).now_or_never() {
            // TODO handle results
            let (n, source) = result.unwrap();

            // TODO
        }
    }
}
