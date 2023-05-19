use std::net::SocketAddr;

use futures::future::try_join_all;
use thiserror::Error;
use tracing::{error, trace};

use crate::{
    header::{DatagramHeader, HeaderError, HEADER_SIZE},
    net, Network, SendError, MAX_DATAGRAM_SIZE,
};

/// Maximum number of bytes of a single message.
pub const MAX_MESSAGE_SIZE: usize = MAX_DATAGRAM_SIZE - HEADER_SIZE;

/// A thin layer over UDP datagram based network translating UDP datagrams to
/// messages with headers.
pub(crate) struct Messages {
    network: Network,
}

impl Messages {
    pub(crate) fn new(network: Network) -> Self {
        Self { network }
    }

    /// Send a message (a datagram) to a list of targets.
    ///
    /// The sending is done in parallel.
    ///
    /// # Arguments
    ///
    /// * `buf` - buffer to be used for datagram construction. The buffer must
    ///   be at least [`MAX_DATAGRAM_SIZE`] long.
    ///
    /// # Panics
    ///
    /// Panics if `buf` is smaller than the constructed datagram size.
    pub(crate) async fn send(
        &mut self,
        buf: &mut [u8],
        header: DatagramHeader,
        data: &[u8],
        targets: &[SocketAddr],
    ) -> Result<(), SendError> {
        trace!("Going to send datagram {}", header);

        let len = HEADER_SIZE + data.len();
        assert!(buf.len() >= len);

        header.write(buf);
        buf[HEADER_SIZE..len].copy_from_slice(data);
        let data = &buf[..len];

        try_join_all(
            targets
                .iter()
                .map(|&target| self.network.send(target, data)),
        )
        .await?;

        Ok(())
    }

    /// Receive a single message.
    ///
    /// # Arguments
    ///
    /// * `buf` - the message is written to this buffer. The buffer must be at
    ///   least [`MAX_DATAGRAM_SIZE`] long.
    ///
    /// # Returns
    ///
    /// Return source address, datagram header and number of bytes of the
    /// message.
    ///
    /// # Panics
    ///
    /// Panics if len of `buf` is smaller than [`MAX_DATAGRAM_SIZE`].
    pub(crate) async fn recv<'a>(
        &mut self,
        buf: &'a mut [u8],
    ) -> Result<(SocketAddr, DatagramHeader, &'a [u8]), MsgRecvError> {
        let (stop, source) = self.network.recv(buf).await.map_err(MsgRecvError::from)?;

        let header = DatagramHeader::read(&buf[0..stop]).map_err(MsgRecvError::from)?;
        trace!("Received datagram with ID {header}");

        Ok((source, header, &buf[HEADER_SIZE..stop]))
    }
}

#[derive(Error, Debug)]
pub(crate) enum MsgRecvError {
    #[error(transparent)]
    InvalidHeader(#[from] HeaderError),
    #[error("error while receiving data from the socket")]
    RecvError(#[from] net::RecvError),
}
