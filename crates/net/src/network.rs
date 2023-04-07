use std::{io, net::IpAddr};

use ahash::AHashMap;
use async_std::net::{SocketAddr, UdpSocket};
use de_core::player::Player;
use local_ip_address::list_afinet_netifas;
use thiserror::Error;

const MAX_DATAGRAM_SIZE: usize = 1024;

pub(crate) struct Network {
    socket: UdpSocket,
    buffer: [u8; MAX_DATAGRAM_SIZE],
    addrs: AddrBook,
}

impl Network {
    pub(crate) async fn bind() -> io::Result<Self> {
        // TODO without parsing?
        let addr: SocketAddr = "0.0.0.0:0".parse().unwrap();
        let socket = UdpSocket::bind(addr).await?;

        Ok(Self {
            socket,
            buffer: [0; MAX_DATAGRAM_SIZE],
            addrs: Default::default(),
        })
    }

    pub(crate) fn port(&self) -> io::Result<u16> {
        self.socket.local_addr().map(|addr| addr.port())
    }

    pub(crate) fn addrs(&self) -> &AddrBook {
        &self.addrs
    }

    pub(crate) fn addrs_mut(&mut self) -> &mut AddrBook {
        &mut self.addrs
    }

    pub(crate) async fn recv(&mut self) -> Result<(Player, &[u8]), RecvError> {
        let (n, peer) = self
            .socket
            .recv_from(&mut self.buffer)
            .await
            .map_err(RecvError::from)?;

        let Some(player) = self.addrs.get_player(peer) else {
            // TODO add a log
            // warn!("Received data from an unknown source: {:?}", peer);
            return Err(RecvError::UnknownSource(peer));
        };

        Ok((player, &self.buffer[..n]))
    }

    // TODO document panic if player is not registered
    pub(crate) async fn send(&self, player: Player, data: &[u8]) -> Result<(), SendError> {
        if data.len() > MAX_DATAGRAM_SIZE {
            panic!(
                "Max datagram size is {} got {}.",
                MAX_DATAGRAM_SIZE,
                data.len()
            );
        }

        let addr = self.addrs.get_addr(player).unwrap();
        let n = self
            .socket
            .send_to(data, addr)
            .await
            .map_err(SendError::from)?;

        if n < data.len() {
            Err(SendError::PartialSend(n, data.len()))
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
pub(crate) struct AddrBook {
    addr_to_player: AHashMap<SocketAddr, Player>,
    player_to_addr: AHashMap<Player, SocketAddr>,
}

impl AddrBook {
    // TODO document panic
    pub(crate) fn add(&mut self, player: Player, addr: SocketAddr) {
        let result = self.addr_to_player.insert(addr, player);
        debug_assert!(result.is_none());
        let result = self.player_to_addr.insert(player, addr);
        debug_assert!(result.is_none());
    }

    fn get_addr(&self, player: Player) -> Option<SocketAddr> {
        self.player_to_addr.get(&player).copied()
    }

    fn get_player(&self, addr: SocketAddr) -> Option<Player> {
        self.addr_to_player.get(&addr).copied()
    }
}

#[derive(Error, Debug)]
pub(crate) enum RecvError {
    #[error("an IO error occurred")]
    Io(#[from] io::Error),
    #[error("data received from an unknown source")]
    UnknownSource(SocketAddr),
}

#[derive(Error, Debug)]
pub(crate) enum SendError {
    #[error("an IO error occurred")]
    Io(#[from] io::Error),
    #[error("only {0} of {1} bytes sent")]
    PartialSend(usize, usize),
}

// TODO document
pub(crate) fn interfaces() -> Vec<IpAddr> {
    list_afinet_netifas()
        .unwrap()
        .iter()
        .filter_map(|(_, ip)| {
            if matches!(ip, IpAddr::V4(_)) {
                Some(*ip)
            } else {
                None
            }
        })
        .collect()
}
