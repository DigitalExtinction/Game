use std::time::Instant;

use async_std::{
    channel::{bounded, Receiver, SendError, Sender, TryRecvError},
    task,
};
use futures::FutureExt;
use thiserror::Error;
use tracing::{error, info};

use crate::{
    communicator::{Communicator, ConnectionError, InMessage, OutMessage},
    connection::{Confirmations, Resends},
    header::{DatagramHeader, DatagramId},
    messages::{Messages, MsgRecvError},
    tasks::{
        dreceiver::{self, InDatagram},
        dsender::{self, OutDatagram},
    },
    Network, MAX_DATAGRAM_SIZE,
};

const CHANNEL_CAPACITY: usize = 1024;

/// This struct implements an async loop which handles the network
/// communication.
struct Processor {
    buf: [u8; MAX_DATAGRAM_SIZE],
    counter: DatagramId,
    out_datagrams: Sender<OutDatagram>,
    in_datagrams: Receiver<InDatagram>,
    confirms: Confirmations,
    resends: Resends,
    outputs: Receiver<OutMessage>,
    inputs: Sender<InMessage>,
    errors: Sender<ConnectionError>,
}

impl Processor {
    fn new(
        out_datagrams: Sender<OutDatagram>,
        in_datagrams: Receiver<InDatagram>,
        outputs: Receiver<OutMessage>,
        inputs: Sender<InMessage>,
        errors: Sender<ConnectionError>,
    ) -> Self {
        Self {
            buf: [0; MAX_DATAGRAM_SIZE],
            out_datagrams,
            in_datagrams,
            counter: DatagramId::zero(),
            confirms: Confirmations::new(),
            resends: Resends::new(),
            outputs,
            inputs,
            errors,
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

            if self.handle_input().await {
                info!("Input finished...");
                break;
            }

            if let Err(err) = self
                .confirms
                .send_confirms(Instant::now(), &mut self.out_datagrams)
                .await
            {
                error!("Message confirmation error: {err:?}");
                break;
            }

            if self.handle_resends().await {
                info!("Errors finished...");
                break;
            }

            let time = Instant::now();
            self.resends.clean(time);
            self.confirms.clean(time);
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
                            self.resends.sent(
                                time,
                                target,
                                data_header.id(),
                                data_header.peers(),
                                &message.data,
                            );
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

    async fn handle_input(&mut self) -> bool {
        let Some(recv_result) = self.in_datagrams.recv().now_or_never() else {
            return false;
        };

        let Ok(datagram) = recv_result else {
            error!("Datagram input channel is unexpectedly closed.");
            return true;
        };

        let data_header = match datagram.header {
            DatagramHeader::Confirmation => {
                self.resends
                    .confirmed(Instant::now(), datagram.source, &datagram.data);
                return false;
            }
            DatagramHeader::Data(data_header) => data_header,
        };

        let reliable = if data_header.reliable() {
            self.confirms
                .received(Instant::now(), datagram.source, data_header.id());
            true
        } else {
            false
        };

        self.inputs
            .send(InMessage::new(
                datagram.data,
                reliable,
                data_header.peers(),
                datagram.source,
            ))
            .await
            .is_err()
    }

    async fn handle_resends(&mut self) -> bool {
        let failures = match self
            .resends
            .resend(Instant::now(), &mut self.buf, &mut self.out_datagrams)
            .await
        {
            Ok(failures) => failures,
            Err(err) => {
                error!("Resend error: {err:?}");
                return true;
            }
        };

        for target in failures {
            let result = self.errors.send(ConnectionError::new(target)).await;
            if result.is_err() {
                return true;
            }
        }

        false
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
pub fn startup(network: Network) -> Communicator {
    let messages = Messages::new(network);

    let (out_datagrams_sender, out_datagrams_receiver) = bounded(16);
    task::spawn(dsender::run(out_datagrams_receiver, messages.clone()));

    let (in_datagrams_sender, in_datagrams_receiver) = bounded(16);
    task::spawn(dreceiver::run(in_datagrams_sender, messages));

    let (outputs_sender, outputs_receiver) = bounded(CHANNEL_CAPACITY);
    let (inputs_sender, inputs_receiver) = bounded(CHANNEL_CAPACITY);
    let (errors_sender, errors_receiver) = bounded(CHANNEL_CAPACITY);

    let communicator = Communicator::new(outputs_sender, inputs_receiver, errors_receiver);
    let processor = Processor::new(
        out_datagrams_sender,
        in_datagrams_receiver,
        outputs_receiver,
        inputs_sender,
        errors_sender,
    );

    task::spawn(processor.run());

    communicator
}
