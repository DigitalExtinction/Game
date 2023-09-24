use std::net::SocketAddr;

use async_std::channel::Receiver;
use tracing::{error, info};

use crate::{
    header::{DatagramHeader, HEADER_SIZE},
    protocol::ProtocolSocket,
    MAX_DATAGRAM_SIZE, MAX_PACKAGE_SIZE,
};

pub(crate) struct OutDatagram {
    header: DatagramHeader,
    data: Vec<u8>,
    target: SocketAddr,
}

impl OutDatagram {
    /// # Panics
    ///
    /// * If `data` is empty.
    ///
    /// * If `data` is larger than [`MAX_PACKAGE_SIZE`].
    pub(crate) fn from_slice(header: DatagramHeader, data: &[u8], target: SocketAddr) -> Self {
        assert!(!data.is_empty());
        assert!(data.len() <= MAX_PACKAGE_SIZE);

        let mut full_data = Vec::with_capacity(HEADER_SIZE + data.len());
        full_data.extend([0; HEADER_SIZE]);
        full_data.extend(data);
        Self::new(header, full_data, target)
    }

    /// # Argument
    ///
    /// * `header`
    ///
    /// * `data` - data of the datagram. First [`HEADER_SIZE`] is reserved for
    ///   to-be-written header.
    ///
    /// * `target` - datagram recipient.
    ///
    /// # Panics
    ///
    /// * If `data` length is smaller or equal to [`HEADER_SIZE`].
    ///
    /// * If `data` is larger than [`MAX_DATAGRAM_SIZE`].
    pub(crate) fn new(header: DatagramHeader, data: Vec<u8>, target: SocketAddr) -> Self {
        assert!(data.len() > HEADER_SIZE);
        assert!(data.len() <= MAX_DATAGRAM_SIZE);

        Self {
            header,
            data,
            target,
        }
    }
}

pub(super) async fn run(port: u16, datagrams: Receiver<OutDatagram>, socket: ProtocolSocket) {
    info!("Starting datagram sender on port {port}...");

    loop {
        let Ok(mut datagram) = datagrams.recv().await else {
            break;
        };
        if let Err(err) = socket
            .send(datagram.header, &mut datagram.data, datagram.target)
            .await
        {
            error!("Error while sending a datagram: {err:?}");
            break;
        }
    }

    info!("Datagram sender on port {port} finished.");
}
