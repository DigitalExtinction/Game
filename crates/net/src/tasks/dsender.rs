use async_std::channel::Receiver;
use tracing::{error, info};

use crate::{
    header::DatagramHeader,
    messages::{Messages, Targets},
    MAX_DATAGRAM_SIZE,
};

pub(crate) struct OutDatagram {
    header: DatagramHeader,
    data: Vec<u8>,
    targets: Targets<'static>,
}

impl OutDatagram {
    pub(crate) fn new<T: Into<Targets<'static>>>(
        header: DatagramHeader,
        data: Vec<u8>,
        targets: T,
    ) -> Self {
        Self {
            header,
            data,
            targets: targets.into(),
        }
    }
}

pub(super) async fn run(port: u16, datagrams: Receiver<OutDatagram>, messages: Messages) {
    info!("Starting datagram sender on port {port}...");
    let mut buffer = [0u8; MAX_DATAGRAM_SIZE];

    loop {
        let Ok(datagram) = datagrams.recv().await else {
            break;
        };
        if let Err(err) = messages
            .send(
                &mut buffer,
                datagram.header,
                &datagram.data,
                datagram.targets,
            )
            .await
        {
            error!("Error while sending a datagram: {err:?}");
            break;
        }
    }

    info!("Datagram sender on port {port} finished.");
}
