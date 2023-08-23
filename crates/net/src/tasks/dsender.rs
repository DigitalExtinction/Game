use std::net::SocketAddr;

use async_std::channel::Receiver;
use tracing::{error, info};

use crate::{header::DatagramHeader, protocol::ProtocolSocket, MAX_DATAGRAM_SIZE};

pub(crate) struct OutDatagram {
    header: DatagramHeader,
    data: Vec<u8>,
    target: SocketAddr,
}

impl OutDatagram {
    pub(crate) fn new(header: DatagramHeader, data: Vec<u8>, target: SocketAddr) -> Self {
        Self {
            header,
            data,
            target,
        }
    }
}

pub(super) async fn run(port: u16, datagrams: Receiver<OutDatagram>, socket: ProtocolSocket) {
    info!("Starting datagram sender on port {port}...");
    let mut buffer = [0u8; MAX_DATAGRAM_SIZE];

    loop {
        let Ok(datagram) = datagrams.recv().await else {
            break;
        };
        if let Err(err) = socket
            .send(
                &mut buffer,
                datagram.header,
                &datagram.data,
                datagram.target,
            )
            .await
        {
            error!("Error while sending a datagram: {err:?}");
            break;
        }
    }

    info!("Datagram sender on port {port} finished.");
}
