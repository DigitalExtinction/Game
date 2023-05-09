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
    /// Reusable buffer used for datagram receiving / sending.
    buf: [u8; MAX_DATAGRAM_SIZE],
}

impl Messages {
    pub(crate) fn new(network: Network) -> Self {
        Self {
            network,
            buf: [0; MAX_DATAGRAM_SIZE],
        }
    }

    /// Send a message (a datagram) to a list of targets.
    ///
    /// The sending is done in parallel.
    pub(crate) async fn send(
        &mut self,
        header: DatagramHeader,
        data: &[u8],
        targets: &[SocketAddr],
    ) -> Result<(), SendError> {
        trace!("Going to send datagram {}", header);

        header.write(&mut self.buf);

        let len = HEADER_SIZE + data.len();
        assert!(self.buf.len() >= len);
        self.buf[HEADER_SIZE..len].copy_from_slice(data);
        let data = &self.buf[..len];

        try_join_all(
            targets
                .iter()
                .map(|&target| self.network.send(target, data)),
        )
        .await?;

        Ok(())
    }

    /// Receive a single message.
    pub(crate) async fn recv(
        &mut self,
    ) -> Result<(SocketAddr, DatagramHeader, &[u8]), MsgRecvError> {
        let (stop, source) = self
            .network
            .recv(&mut self.buf)
            .await
            .map_err(MsgRecvError::from)?;

        let header = DatagramHeader::read(&self.buf[0..stop]).map_err(MsgRecvError::from)?;
        trace!("Received datagram with ID {header}");

        Ok((source, header, &self.buf[HEADER_SIZE..stop]))
    }
}

#[derive(Error, Debug)]
pub(crate) enum MsgRecvError {
    #[error(transparent)]
    InvalidHeader(#[from] HeaderError),
    #[error("error while receiving data from the socket")]
    RecvError(#[from] net::RecvError),
}
