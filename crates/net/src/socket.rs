use std::{
    io,
    net::{IpAddr, Ipv4Addr},
};

use async_std::net::{SocketAddr, UdpSocket};
use thiserror::Error;

/// Maximum size of a UDP datagram which might be sent by this crate.
///
/// This is the maximum datagram size "guaranteed" to be deliverable over any
/// reasonable network.
///
/// <https://stackoverflow.com/a/35697810/4448708>
pub const MAX_DATAGRAM_SIZE: usize = 508;

/// This struct represents a low level network socket. The socket is based on
/// UDP and thus provides unreliable and unordered means of data delivery.
pub struct Socket {
    socket: UdpSocket,
    port: u16,
}

impl Socket {
    /// Creates / binds a new IPv4 based connection (socket).
    ///
    /// # Arguments
    ///
    /// * `port` - if None, system assigned port is used.
    pub async fn bind(port: Option<u16>) -> io::Result<Self> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port.unwrap_or(0));
        let socket = UdpSocket::bind(addr).await?;

        let obtained_port = socket.local_addr().map(|addr| addr.port())?;
        if let Some(desired_port) = port {
            assert_eq!(obtained_port, desired_port);
        }

        Ok(Self {
            socket,
            port: obtained_port,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Receive a single datagram.
    ///
    /// The returned data are guaranteed to be at most [`MAX_DATAGRAM_SIZE`]
    /// bytes long.
    ///
    /// # Panics
    ///
    /// Panics if len of `buf` is smaller than [`MAX_DATAGRAM_SIZE`].
    pub async fn recv(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), RecvError> {
        assert!(buf.len() >= MAX_DATAGRAM_SIZE);

        self.socket
            .recv_from(buf)
            .await
            .map(|(len, source)| (len.min(MAX_DATAGRAM_SIZE), source))
            .map_err(RecvError::from)
    }

    /// Send data to a single target.
    ///
    /// # Panics
    ///
    /// This method panics if `data` have more than [`MAX_DATAGRAM_SIZE`]
    /// bytes.
    pub async fn send(&self, target: SocketAddr, data: &[u8]) -> Result<(), SendError> {
        if data.len() > MAX_DATAGRAM_SIZE {
            panic!(
                "Max datagram size is {} got {}.",
                MAX_DATAGRAM_SIZE,
                data.len()
            );
        }

        let n = self
            .socket
            .send_to(data, target)
            .await
            .map_err(SendError::from)?;

        if n < data.len() {
            Err(SendError::PartialSend(n, data.len()))
        } else {
            Ok(())
        }
    }
}

#[derive(Error, Debug)]
pub enum RecvError {
    #[error("an IO error occurred")]
    Io(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum SendError {
    #[error("an IO error occurred")]
    Io(#[from] io::Error),
    #[error("only {0} of {1} bytes sent")]
    PartialSend(usize, usize),
}
