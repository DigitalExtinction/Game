use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use ahash::AHashMap;

use crate::messages::Messages;
use crate::{confirmbuf::ConfirmBuffer, MAX_MESSAGE_SIZE};
use crate::{header::DatagramHeader, SendError};

/// Connection info should be tossed away after this time.
const MAX_CONN_AGE: Duration = Duration::from_secs(600);

/// This struct handles reliable delivery of messages.
pub(crate) struct Reliability {
    addrs: Vec<SocketAddr>,
    connections: AHashMap<SocketAddr, Connection>,
}

impl Reliability {
    pub(super) fn new() -> Self {
        Self {
            addrs: Vec::new(),
            connections: AHashMap::new(),
        }
    }

    /// This method marks a message with `id` from `addr` as received.
    ///
    /// This method should be called exactly once after each reliable message
    /// is delivered.
    pub(crate) fn received(&mut self, addr: SocketAddr, id: u32) {
        let time = Instant::now();
        let connection = self.update(time, addr);
        connection.confirms.push(time, id);
    }

    /// Send message confirmation packets which are ready to be send.
    pub(crate) async fn send_confirms(&mut self, messages: &mut Messages) -> Result<(), SendError> {
        let time = Instant::now();

        for (addr, connection) in self.connections.iter_mut() {
            if !connection.confirms.ready(time) {
                continue;
            }

            while let Some(data) = connection.confirms.flush(MAX_MESSAGE_SIZE) {
                messages
                    .send(DatagramHeader::Confirmation, data, &[*addr])
                    .await?;
            }
        }

        self.clean(time);
        Ok(())
    }

    /// Ensure that a connection object exists and set its last contact time to
    /// `time`. Mutable reference to the connection object is returned.
    fn update(&mut self, time: Instant, addr: SocketAddr) -> &mut Connection {
        self.connections
            .entry(addr)
            .and_modify(|c| c.last_contact = time)
            .or_insert_with(|| {
                self.addrs.push(addr);
                Connection {
                    last_contact: time,
                    confirms: ConfirmBuffer::new(),
                }
            })
    }

    /// Forget all connections with last contact older than [`MAX_CONN_AGE`].
    fn clean(&mut self, time: Instant) {
        let mut index = 0;

        while index < self.addrs.len() {
            let addr = self.addrs[index];
            let connection = self.connections.get_mut(&addr).unwrap();

            if time - connection.last_contact > MAX_CONN_AGE {
                self.addrs.swap_remove(index);
                self.connections.remove(&addr).unwrap();
            } else {
                index += 1;
            }
        }
    }
}

/// This object represents an "open connection". It holds state of the
/// connection necessary for message delivery semi-reliability.
struct Connection {
    last_contact: Instant,
    confirms: ConfirmBuffer,
}
