use std::{net::SocketAddr, time::Duration};

use async_std::{channel::Sender, future::timeout};
use tracing::{error, info, warn};

use crate::{
    header::{DatagramHeader, PackageHeader},
    protocol::{MsgRecvError, ProtocolSocket},
    MAX_DATAGRAM_SIZE, MAX_PACKAGE_SIZE,
};

pub(super) struct InSystemDatagram {
    pub(super) source: SocketAddr,
    pub(super) data: Vec<u8>,
}

pub(super) struct InPackageDatagram {
    pub(super) source: SocketAddr,
    pub(super) header: PackageHeader,
    pub(super) data: Vec<u8>,
}

/// Handler of input datagrams received with `socket`.
///
/// The handler runs a loop which finishes when `system_datagrams` and
/// `user_datagrams` channel are closed.
pub(super) async fn run(
    port: u16,
    system_datagrams: Sender<InSystemDatagram>,
    package_datagrams: Sender<InPackageDatagram>,
    socket: ProtocolSocket,
) {
    info!("Starting datagram receiver on port {port}...");
    let mut buffer = [0u8; MAX_DATAGRAM_SIZE];

    loop {
        if package_datagrams.is_closed() && system_datagrams.is_closed() {
            break;
        }

        let Ok(result) = timeout(Duration::from_millis(500), socket.recv(&mut buffer)).await else {
            continue;
        };

        let (addr, header, data) = match result {
            Ok(msg) => msg,
            Err(err @ MsgRecvError::InvalidHeader(_)) => {
                warn!("Invalid datagram received on port {port}: {err:?}");
                continue;
            }
            Err(err @ MsgRecvError::RecvError(_)) => {
                error!("Data receiving failed on port {port}: {err:?}");
                break;
            }
        };

        assert!(data.len() <= MAX_PACKAGE_SIZE);

        // Closed channel(s) are handled at the top part of the loop,
        // therefore errors from .send() are not treated below.
        match header {
            DatagramHeader::Confirmation => {
                let _ = system_datagrams
                    .send(InSystemDatagram {
                        source: addr,
                        data: data.to_vec(),
                    })
                    .await;
            }
            DatagramHeader::Package(package_header) => {
                let _ = package_datagrams
                    .send(InPackageDatagram {
                        source: addr,
                        header: package_header,
                        data: data.to_vec(),
                    })
                    .await;
            }
        };
    }

    info!("Datagram receiver on port {port} finished.");
}
