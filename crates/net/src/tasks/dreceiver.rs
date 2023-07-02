use std::{net::SocketAddr, time::Duration};

use async_std::{channel::Sender, future::timeout};
use tracing::{error, info, warn};

use crate::{
    header::{DataHeader, DatagramHeader},
    messages::{Messages, MsgRecvError},
    MAX_DATAGRAM_SIZE,
};

pub(super) struct InSystemDatagram {
    pub(super) source: SocketAddr,
    pub(super) data: Vec<u8>,
}

pub(super) struct InUserDatagram {
    pub(super) source: SocketAddr,
    pub(super) header: DataHeader,
    pub(super) data: Vec<u8>,
}

/// Handler of input datagrams received with `messages`.
///
/// The handler runs a loop which finishes when `system_datagrams` or
/// `user_datagrams` channel is closed.
pub(super) async fn run(
    port: u16,
    system_datagrams: Sender<InSystemDatagram>,
    user_datagrams: Sender<InUserDatagram>,
    messages: Messages,
) {
    info!("Starting datagram receiver on port {port}...");
    let mut buffer = [0u8; MAX_DATAGRAM_SIZE];

    loop {
        if user_datagrams.is_closed() || system_datagrams.is_closed() {
            break;
        }

        let Ok(result) = timeout(Duration::from_millis(500), messages.recv(&mut buffer)).await
        else {
            continue;
        };

        let (addr, header, data) = match result {
            Ok(msg) => msg,
            Err(err @ MsgRecvError::InvalidHeader(_)) => {
                warn!("Invalid message received on port {port}: {err:?}");
                continue;
            }
            Err(err @ MsgRecvError::RecvError(_)) => {
                error!("Data receiving failed on port {port}: {err:?}");
                break;
            }
        };

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
            DatagramHeader::Data(data_header) => {
                let _ = user_datagrams
                    .send(InUserDatagram {
                        source: addr,
                        header: data_header,
                        data: data.to_vec(),
                    })
                    .await;
            }
        };
    }

    info!("Datagram receiver on port {port} finished.");
}
