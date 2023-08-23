use std::net::SocketAddr;

use async_std::sync::Arc;
use thiserror::Error;
use tracing::{error, trace};

use crate::{
    header::{DatagramHeader, HeaderError, HEADER_SIZE},
    socket, SendError, Socket, MAX_DATAGRAM_SIZE,
};

/// Maximum number of bytes of a single package payload.
pub const MAX_PACKAGE_SIZE: usize = MAX_DATAGRAM_SIZE - HEADER_SIZE;

/// A thin layer over a UDP socket translating between UDP datagrams and
/// header-payload pairs.
#[derive(Clone)]
pub(crate) struct ProtocolSocket {
    socket: Arc<Socket>,
}

impl ProtocolSocket {
    pub(crate) fn new(socket: Socket) -> Self {
        Self {
            socket: Arc::new(socket),
        }
    }

    /// Send data to a list of targets.
    ///
    /// The sending is done in parallel.
    ///
    /// # Arguments
    ///
    /// * `buf` - binary data buffer used during datagram construction.
    ///
    /// * `header` - header of the datagram.
    ///
    /// * `data` - datagram payload.
    ///
    /// * `target` - recipient of the datagram.
    pub(crate) async fn send(
        &self,
        buf: &mut [u8],
        header: DatagramHeader,
        data: &[u8],
        target: SocketAddr,
    ) -> Result<(), SendError> {
        let len = HEADER_SIZE + data.len();
        assert!(buf.len() >= len);
        let buf = &mut buf[..len];
        buf[HEADER_SIZE..len].copy_from_slice(data);

        trace!("Going to send datagram {}", header);
        header.write(buf);
        self.socket.send(target, buf).await?;
        Ok(())
    }

    /// Receive a single datagram.
    ///
    /// # Arguments
    ///
    /// * `buf` - the data is written to this buffer. The buffer must be at
    ///   least [`MAX_DATAGRAM_SIZE`] long.
    ///
    /// # Returns
    ///
    /// Return source address, datagram header and a slice with the payload.
    /// Header data are not included in the payload slice.
    ///
    /// # Panics
    ///
    /// Panics if len of `buf` is smaller than [`MAX_DATAGRAM_SIZE`].
    pub(crate) async fn recv<'a>(
        &self,
        buf: &'a mut [u8],
    ) -> Result<(SocketAddr, DatagramHeader, &'a [u8]), MsgRecvError> {
        let (stop, source) = self.socket.recv(buf).await.map_err(MsgRecvError::from)?;

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
    RecvError(#[from] socket::RecvError),
}
