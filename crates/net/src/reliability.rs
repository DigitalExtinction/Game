use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use ahash::AHashMap;
use thiserror::Error;

use crate::{confirmbuf::ConfirmBuffer, header::HEADER_SIZE, MAX_MESSAGE_SIZE};
use crate::{header::DatagramHeader, SendError};
use crate::{messages::Messages, resend::ResendQueue};

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

    pub(crate) fn sent(&mut self, addr: SocketAddr, id: u32, data: &[u8], time: Instant) {
        let connection = self.update(time, addr);
        connection.resends.push(id, data, time);
    }

    /// This method marks a message with `id` from `addr` as received.
    ///
    /// This method should be called exactly once after each reliable message
    /// is delivered.
    pub(crate) fn received(&mut self, addr: SocketAddr, id: u32, time: Instant) {
        let connection = self.update(time, addr);
        connection.confirms.push(time, id);
    }

    /// Processes message with datagram confirmations.
    ///
    /// The data encode IDs of delivered (and confirmed) messages so that they
    /// can be forgotten.
    pub(crate) fn confirmed(&mut self, addr: SocketAddr, data: &[u8], time: Instant) {
        let connection = self.update(time, addr);

        let mut bytes = [0; 4];
        for i in 0..data.len() / 4 {
            let offset = i * 4;
            bytes.copy_from_slice(&data[offset..offset + 4]);
            let id = u32::from_be_bytes(bytes);
            connection.resends.resolve(id);
        }
    }

    /// Send message confirmation packets which are ready to be send.
    ///
    /// # Arguments
    ///
    /// * `buf` - buffer for message construction. Must be at least
    ///   [`crate::MAX_DATAGRAM_SIZE`] long.
    ///
    /// * `messages` - message connection to be used for delivery of the
    ///   confirmations.
    ///
    /// * `time` - current time.
    ///
    /// # Panics
    ///
    /// May panic if `buf` is not large enough.
    pub(crate) async fn send_confirms(
        &mut self,
        buf: &mut [u8],
        messages: &mut Messages,
        time: Instant,
    ) -> Result<(), SendError> {
        for (addr, connection) in self.connections.iter_mut() {
            if !connection.confirms.ready(time) {
                continue;
            }

            while let Some(data) = connection.confirms.flush(MAX_MESSAGE_SIZE) {
                messages
                    .send_separate(buf, DatagramHeader::Confirmation, data, &[*addr])
                    .await?;
            }
        }

        self.clean(time);
        Ok(())
    }

    /// Re-send all messages already due for re-sending.
    pub(crate) async fn resend(
        &mut self,
        buf: &mut [u8],
        messages: &mut Messages,
        time: Instant,
    ) -> Result<(), DeliveryErrors> {
        let mut errors = Vec::new();

        let mut index = 0;
        while index < self.addrs.len() {
            let addr = self.addrs[index];
            let connection = self.connections.get_mut(&addr).unwrap();

            let failed = loop {
                match connection.resends.reschedule(&mut buf[HEADER_SIZE..], time) {
                    Ok(Some((len, id))) => {
                        let result = messages
                            .send(
                                &mut buf[..len + HEADER_SIZE],
                                DatagramHeader::Reliable(id),
                                &[addr],
                            )
                            .await;

                        if result.is_err() {
                            break true;
                        }
                    }

                    Ok(None) => break false,
                    Err(_) => break true,
                }
            };

            if failed {
                self.remove(index);
                errors.push(addr);
            } else {
                index += 1;
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(DeliveryErrors(errors))
        }
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
                    resends: ResendQueue::new(),
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
                self.remove(index);
            } else {
                index += 1;
            }
        }
    }

    fn remove(&mut self, index: usize) {
        let addr = self.addrs.swap_remove(index);
        self.connections.remove(&addr).unwrap();
    }
}

/// This object represents an "open connection". It holds state of the
/// connection necessary for message delivery semi-reliability.
struct Connection {
    last_contact: Instant,
    confirms: ConfirmBuffer,
    resends: ResendQueue,
}

#[derive(Error, Debug)]
#[error("connection failed with {0:?}")]
pub(crate) struct DeliveryErrors(Vec<SocketAddr>);

impl DeliveryErrors {
    pub(crate) fn targets(&self) -> &[SocketAddr] {
        self.0.as_slice()
    }
}
