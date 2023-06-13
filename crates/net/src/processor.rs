use std::time::Instant;

use async_std::{
    channel::{bounded, Receiver, SendError, Sender, TryRecvError},
    io, task,
};
use thiserror::Error;
use tracing::{error, info};

use crate::{
    communicator::{Communicator, InMessage, OutMessage},
    connection::{Confirmations, Resends},
    header::{DatagramHeader, DatagramId},
    messages::{Messages, MsgRecvError},
    tasks::{
        confirmer, dreceiver,
        dsender::{self, OutDatagram},
        resender, sreceiver, ureceiver,
    },
    Network,
};

const CHANNEL_CAPACITY: usize = 1024;

/// This struct implements an async loop which handles the network
/// communication.
struct Processor {
    counter: DatagramId,
    out_datagrams: Sender<OutDatagram>,
    resends: Resends,
    outputs: Receiver<OutMessage>,
}

impl Processor {
    fn new(
        resends: Resends,
        out_datagrams: Sender<OutDatagram>,
        outputs: Receiver<OutMessage>,
    ) -> Self {
        Self {
            out_datagrams,
            counter: DatagramId::zero(),
            resends,
            outputs,
        }
    }

    /// Start the infinite async loop.
    ///
    /// The loop terminates once the input or output communication channels is
    /// closed.
    ///
    /// # Panics
    ///
    /// Panics on IO errors.
    async fn run(mut self) {
        info!("Starting network loop...");

        loop {
            if self.handle_output().await {
                info!("Output finished...");
                break;
            }
        }
    }

    async fn handle_output(&mut self) -> bool {
        match self.outputs.try_recv() {
            Ok(message) => {
                let header =
                    DatagramHeader::new_data(message.reliable(), message.peers(), self.counter);
                self.counter = self.counter.incremented();

                if let DatagramHeader::Data(data_header) = header {
                    if data_header.reliable() {
                        let time = Instant::now();
                        for &target in &message.targets {
                            self.resends
                                .sent(
                                    time,
                                    target,
                                    data_header.id(),
                                    data_header.peers(),
                                    &message.data,
                                )
                                .await;
                        }
                    }
                }

                let closed = self
                    .out_datagrams
                    .send(OutDatagram::new(header, message.data, message.targets))
                    .await
                    .is_err();

                if closed {
                    error!("Datagram output channel is unexpectedly closed.");
                }

                closed
            }
            Err(err) => match err {
                TryRecvError::Empty => false,
                TryRecvError::Closed => true,
            },
        }
    }
}

#[derive(Error, Debug)]
enum InputHandlingError {
    #[error(transparent)]
    MsgRecvError(#[from] MsgRecvError),
    #[error("inputs channel error")]
    InputsError(#[from] SendError<InMessage>),
}

/// Setups and starts communication stack tasks.
pub fn startup(network: Network) -> io::Result<Communicator> {
    let port = network.port()?;
    let messages = Messages::new(network);

    let (out_datagrams_sender, out_datagrams_receiver) = bounded(16);
    task::spawn(dsender::run(port, out_datagrams_receiver, messages.clone()));

    let (in_system_datagrams_sender, in_system_datagrams_receiver) = bounded(16);
    let (in_user_datagrams_sender, in_user_datagrams_receiver) = bounded(16);
    task::spawn(dreceiver::run(
        port,
        in_system_datagrams_sender,
        in_user_datagrams_sender,
        messages,
    ));

    let resends = Resends::new();
    task::spawn(sreceiver::run(
        port,
        in_system_datagrams_receiver,
        resends.clone(),
    ));

    let (inputs_sender, inputs_receiver) = bounded(CHANNEL_CAPACITY);
    let confirms = Confirmations::new();
    task::spawn(ureceiver::run(
        port,
        in_user_datagrams_receiver,
        inputs_sender,
        confirms.clone(),
    ));

    let (outputs_sender, outputs_receiver) = bounded(CHANNEL_CAPACITY);
    let (errors_sender, errors_receiver) = bounded(CHANNEL_CAPACITY);
    task::spawn(resender::run(
        port,
        out_datagrams_sender.clone(),
        errors_sender,
        resends.clone(),
    ));

    task::spawn(confirmer::run(port, out_datagrams_sender.clone(), confirms));

    let communicator = Communicator::new(outputs_sender, inputs_receiver, errors_receiver);
    let processor = Processor::new(resends, out_datagrams_sender, outputs_receiver);

    task::spawn(processor.run());

    Ok(communicator)
}
