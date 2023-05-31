use std::net::SocketAddr;

use async_std::channel::{Receiver, RecvError, SendError, Sender};

use crate::{header::Destination, messages::MAX_MESSAGE_SIZE};

/// A message / datagram to be delivered.
pub struct OutMessage {
    data: Vec<u8>,
    reliable: bool,
    destination: Destination,
    targets: Vec<SocketAddr>,
}

impl OutMessage {
    /// # Arguments
    ///
    /// * `data` - data to be send.
    ///
    /// * `reliable` - whether to deliver the data reliably.
    ///
    /// * `targets` - list of message recipients.
    ///
    /// # Panics
    ///
    /// Panics if data is longer than [`MAX_MESSAGE_SIZE`].

    pub fn new(
        data: Vec<u8>,
        reliable: bool,
        destination: Destination,
        targets: Vec<SocketAddr>,
    ) -> Self {
        assert!(data.len() < MAX_MESSAGE_SIZE);
        Self {
            data,
            reliable,
            destination,
            targets,
        }
    }

    pub(crate) fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub(crate) fn reliable(&self) -> bool {
        self.reliable
    }

    pub(crate) fn destination(&self) -> Destination {
        self.destination
    }

    pub(crate) fn targets(&self) -> &[SocketAddr] {
        self.targets.as_slice()
    }
}

/// A received message / datagram.
pub struct InMessage {
    data: Vec<u8>,
    reliable: bool,
    destination: Destination,
    source: SocketAddr,
}

impl InMessage {
    pub(crate) fn new(
        data: Vec<u8>,
        reliable: bool,
        destination: Destination,
        source: SocketAddr,
    ) -> Self {
        Self {
            data,
            reliable,
            destination,
            source,
        }
    }

    pub fn data(self) -> Vec<u8> {
        self.data
    }

    /// Whether the datagram was delivered reliably.
    pub fn reliable(&self) -> bool {
        self.reliable
    }

    pub fn source(&self) -> SocketAddr {
        self.source
    }

    pub fn destination(&self) -> Destination {
        self.destination
    }
}

pub struct ConnectionError {
    target: SocketAddr,
}

impl ConnectionError {
    pub(crate) fn new(target: SocketAddr) -> Self {
        Self { target }
    }

    pub fn target(&self) -> SocketAddr {
        self.target
    }
}

/// This struct handles communication with a side async loop with the network
/// communication.
pub struct Communicator {
    outputs: Sender<OutMessage>,
    inputs: Receiver<InMessage>,
    errors: Receiver<ConnectionError>,
}

impl Communicator {
    pub(crate) fn new(
        outputs: Sender<OutMessage>,
        inputs: Receiver<InMessage>,
        errors: Receiver<ConnectionError>,
    ) -> Self {
        Self {
            outputs,
            inputs,
            errors,
        }
    }

    pub async fn recv(&mut self) -> Result<InMessage, RecvError> {
        self.inputs.recv().await
    }

    pub async fn send(&mut self, message: OutMessage) -> Result<(), SendError<OutMessage>> {
        self.outputs.send(message).await
    }

    pub async fn errors(&mut self) -> Result<ConnectionError, RecvError> {
        self.errors.recv().await
    }
}
