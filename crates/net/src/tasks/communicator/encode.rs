use std::net::SocketAddr;

use bincode::{encode_into_std_write, error::EncodeError};

use crate::{
    header::{Peers, Reliability, HEADER_SIZE},
    protocol::MAX_PACKAGE_SIZE,
    tasks::communicator::BINCODE_CONF,
    MAX_DATAGRAM_SIZE,
};

/// A package to be send.
pub struct OutPackage {
    /// First [`HEADER_SIZE`] bytes are reserved for the header. Payload must
    /// follow.
    data: Vec<u8>,
    reliability: Reliability,
    peers: Peers,
    target: SocketAddr,
}

impl OutPackage {
    /// Creates a package from a single message.
    pub fn encode_single<E>(
        message: &E,
        reliability: Reliability,
        peers: Peers,
        target: SocketAddr,
    ) -> Result<Self, EncodeError>
    where
        E: bincode::Encode,
    {
        let mut data = Vec::with_capacity(HEADER_SIZE + 1);
        data.extend([0; HEADER_SIZE]);
        encode_into_std_write(message, &mut data, BINCODE_CONF)?;
        Ok(Self::new(data, reliability, peers, target))
    }

    /// # Panics
    ///
    /// If `data` is longer than [`MAX_PACKAGE_SIZE`].
    pub fn from_slice(
        data: &[u8],
        reliability: Reliability,
        peers: Peers,
        target: SocketAddr,
    ) -> Self {
        assert!(data.len() <= MAX_PACKAGE_SIZE);

        let mut full_data = Vec::with_capacity(HEADER_SIZE + data.len());
        full_data.extend([0; HEADER_SIZE]);
        full_data.extend(data);
        Self::new(full_data, reliability, peers, target)
    }

    /// # Arguments
    ///
    /// * `data` - data to be send. The message data must start exactly at
    ///   [`HEADER_SIZE`]. The initial bytes are reserved for the header. The
    ///   header is not filled by the caller.
    ///
    /// * `reliability` - package delivery reliability mode.
    ///
    /// * `target` - package recipient.
    ///
    /// # Panics
    ///
    /// * If data length is smaller or equal to header size..
    ///
    /// * If data is longer than [`MAX_DATAGRAM_SIZE`].
    pub(super) fn new(
        data: Vec<u8>,
        reliability: Reliability,
        peers: Peers,
        target: SocketAddr,
    ) -> Self {
        assert!(data.len() > HEADER_SIZE);
        assert!(data.len() <= MAX_DATAGRAM_SIZE);
        Self {
            data,
            reliability,
            peers,
            target,
        }
    }

    /// Returns package data.
    ///
    /// The data start at [`HEADER_SIZE`] so that header may be written
    /// to the beginning of the vector.
    pub(crate) fn data(self) -> Vec<u8> {
        self.data
    }

    /// Returns slice to the payload part (without header) of the data.
    pub(crate) fn data_slice(&self) -> &[u8] {
        &self.data[HEADER_SIZE..]
    }

    pub(crate) fn reliability(&self) -> Reliability {
        self.reliability
    }

    pub(crate) fn peers(&self) -> Peers {
        self.peers
    }

    pub(crate) fn target(&self) -> SocketAddr {
        self.target
    }
}
