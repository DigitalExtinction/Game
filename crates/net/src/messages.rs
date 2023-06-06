use std::{borrow::Cow, io, net::SocketAddr};

use async_std::sync::Arc;
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
#[derive(Clone)]
pub(crate) struct Messages {
    network: Arc<Network>,
}

impl Messages {
    pub(crate) fn new(network: Network) -> Self {
        Self {
            network: Arc::new(network),
        }
    }

    pub(crate) fn port(&self) -> io::Result<u16> {
        self.network.port()
    }

    /// Send message to a list of targets.
    ///
    /// The sending is done in parallel.
    ///
    /// # Arguments
    ///
    /// * `data` - to be modified data slice containing the data of the message
    ///   to be send. The data must start at position [`HEADER_SIZE`] and span
    ///   across the rest of the slice.
    ///
    /// * `header` - header of the message.
    ///
    /// * `targets` - recipients of the message.
    pub(crate) async fn send<'a, T>(
        &'a self,
        buf: &mut [u8],
        header: DatagramHeader,
        data: &[u8],
        targets: T,
    ) -> Result<(), SendError>
    where
        T: Into<Targets<'a>>,
    {
        let len = HEADER_SIZE + data.len();
        assert!(buf.len() >= len);
        let buf = &mut buf[..len];
        buf[HEADER_SIZE..len].copy_from_slice(data);

        trace!("Going to send datagram {}", header);
        header.write(buf);

        match targets.into() {
            Targets::Single(target) => {
                self.network.send(target, buf).await?;
            }
            Targets::Many(targets) => {
                try_join_all(targets.iter().map(|&target| self.network.send(target, buf))).await?;
            }
        }

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
        &self,
        buf: &'a mut [u8],
    ) -> Result<(SocketAddr, DatagramHeader, &'a [u8]), MsgRecvError> {
        let (stop, source) = self.network.recv(buf).await.map_err(MsgRecvError::from)?;

        let header = DatagramHeader::read(&buf[0..stop]).map_err(MsgRecvError::from)?;
        trace!("Received datagram with ID {header}");

        Ok((source, header, &buf[HEADER_SIZE..stop]))
    }
}

pub(crate) enum Targets<'a> {
    Single(SocketAddr),
    Many(Cow<'a, [SocketAddr]>),
}

impl<'a> From<SocketAddr> for Targets<'a> {
    fn from(addr: SocketAddr) -> Self {
        Self::Single(addr)
    }
}

impl<'a> From<&'a [SocketAddr]> for Targets<'a> {
    fn from(addrs: &'a [SocketAddr]) -> Self {
        Self::Many(addrs.into())
    }
}

impl<'a> From<Vec<SocketAddr>> for Targets<'a> {
    fn from(addrs: Vec<SocketAddr>) -> Self {
        Self::Many(addrs.into())
    }
}

#[derive(Error, Debug)]
pub(crate) enum MsgRecvError {
    #[error(transparent)]
    InvalidHeader(#[from] HeaderError),
    #[error("error while receiving data from the socket")]
    RecvError(#[from] net::RecvError),
}
