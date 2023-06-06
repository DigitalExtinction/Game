use std::net::SocketAddr;

use async_std::channel::Sender;
use tracing::{error, info, log::warn};

use crate::{
    header::DatagramHeader,
    messages::{Messages, MsgRecvError},
    MAX_DATAGRAM_SIZE,
};

pub(crate) struct InDatagram {
    pub(crate) source: SocketAddr,
    pub(crate) header: DatagramHeader,
    pub(crate) data: Vec<u8>,
}

pub(crate) async fn run(datagrams: Sender<InDatagram>, messages: Messages) {
    let port = match messages.port() {
        Ok(port) => port,
        Err(err) => {
            error!("Cannot obtain port: {:?}", err);
            return;
        }
    };

    info!("Starting datagram receiver on port {port}...");
    let mut buffer = [0u8; MAX_DATAGRAM_SIZE];

    loop {
        let (addr, header, data) = match messages.recv(&mut buffer).await {
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

        let result = datagrams
            .send(InDatagram {
                source: addr,
                header,
                data: data.to_vec(),
            })
            .await;
        if result.is_err() {
            break;
        }
    }

    info!("Datagram receiver on port {port} finished.");
}
