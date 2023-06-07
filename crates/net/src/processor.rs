use std::time::Instant;

use async_std::{
    channel::{bounded, Receiver, SendError, Sender, TryRecvError},
    io, task,
};
use thiserror::Error;
use tracing::{error, info};

use crate::{
    communicator::{Communicator, ConnectionError, InMessage, OutMessage},
    connection::{Confirmations, Resends},
    header::{DatagramHeader, DatagramId},
    messages::{Messages, MsgRecvError},
    tasks::{
        dreceiver,
        dsender::{self, OutDatagram},
        sreceiver, ureceiver,
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
    confirms: Confirmations,
    resends: Resends,
    outputs: Receiver<OutMessage>,
    errors: Sender<ConnectionError>,
}

impl Processor {
    fn new(
        confirms: Confirmations,
        resends: Resends,
        out_datagrams: Sender<OutDatagram>,
        outputs: Receiver<OutMessage>,
        errors: Sender<ConnectionError>,
    ) -> Self {
        Self {
            buf: [0; MAX_DATAGRAM_SIZE],
            out_datagrams,
            counter: DatagramId::zero(),
            confirms,
            resends,
            outputs,
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
            self.resends.clean(time).await;
            self.confirms.clean(time).await;
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

    let communicator = Communicator::new(outputs_sender, inputs_receiver, errors_receiver);
    let processor = Processor::new(
        confirms,
        resends,
        out_datagrams_sender,
        outputs_receiver,
        errors_sender,
    );

    task::spawn(processor.run());

    Ok(communicator)
}
