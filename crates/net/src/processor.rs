use std::time::Instant;

use async_std::channel::{bounded, Receiver, SendError, Sender, TryRecvError, TrySendError};
use futures::FutureExt;
use thiserror::Error;
use tracing::{error, info, warn};

use crate::{
    communicator::{Communicator, ConnectionError, InMessage, OutMessage},
    connection::{Confirmations, Resends},
    header::{DatagramHeader, DatagramId},
    messages::{Messages, MsgRecvError},
    Network, MAX_DATAGRAM_SIZE,
};

const CHANNEL_CAPACITY: usize = 1024;

/// This struct implements an async loop which handles the network
/// communication.
pub struct Processor {
    buf: [u8; MAX_DATAGRAM_SIZE],
    messages: Messages,
    counter: DatagramId,
    confirms: Confirmations,
    resends: Resends,
    outputs: Receiver<OutMessage>,
    inputs: Sender<InMessage>,
    errors: Sender<ConnectionError>,
}

impl Processor {
    fn new(
        messages: Messages,
        outputs: Receiver<OutMessage>,
        inputs: Sender<InMessage>,
        errors: Sender<ConnectionError>,
    ) -> Self {
        Self {
            buf: [0; MAX_DATAGRAM_SIZE],
            messages,
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
    pub async fn run(mut self) {
        info!("Starting network loop...");

        loop {
            if self.handle_output().await {
                info!("Output finished...");
                break;
            }

            if let Err(err) = self.handle_input().await {
                match err {
                    InputHandlingError::InputsError(err) => {
                        info!("Input finished: {err:?}");
                        break;
                    }
                    InputHandlingError::MsgRecvError(MsgRecvError::RecvError(err)) => {
                        error!("Message receiving error: {err:?}");
                        break;
                    }
                    InputHandlingError::MsgRecvError(MsgRecvError::InvalidHeader(err)) => {
                        warn!("Invalid header received: {err:?}");
                        // Do not break the loop for all because just of a
                        // single malformed datagram.
                    }
                }
            }

            if let Err(err) = self
                .confirms
                .send_confirms(Instant::now(), &mut self.buf, &mut self.messages)
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

                match self
                    .messages
                    .send_separate(&mut self.buf, header, message.data(), message.targets())
                    .await
                {
                    Ok(()) => {
                        if let DatagramHeader::Data(data_header) = header {
                            if data_header.reliable() {
                                let time = Instant::now();
                                for &target in message.targets() {
                                    self.resends.sent(
                                        time,
                                        target,
                                        data_header.id(),
                                        data_header.peers(),
                                        message.data(),
                                    );
                                }
                            }
                        }
                    }
                    Err(err) => panic!("Send error: {:?}", err),
                }

                false
            }
            Err(err) => match err {
                TryRecvError::Empty => false,
                TryRecvError::Closed => true,
            },
        }
    }

    async fn handle_input(&mut self) -> Result<(), InputHandlingError> {
        let Some(recv_result) = self.messages.recv(&mut self.buf).now_or_never() else { return Ok(()) };
        let (source, header, data) = recv_result.map_err(InputHandlingError::from)?;

        let data_header = match header {
            DatagramHeader::Confirmation => {
                self.resends.confirmed(Instant::now(), source, data);
                return Ok(());
            }
            DatagramHeader::Data(data_header) => data_header,
        };

        let reliable = if data_header.reliable() {
            self.confirms
                .received(Instant::now(), source, data_header.id());
            true
        } else {
            false
        };

        self.inputs
            .send(InMessage::new(
                data.to_vec(),
                reliable,
                data_header.peers(),
                source,
            ))
            .await
            .map_err(InputHandlingError::from)?;

        Ok(())
    }

    async fn handle_resends(&mut self) -> bool {
        let failures = match self
            .resends
            .resend(Instant::now(), &mut self.buf, &mut self.messages)
            .await
        {
            Ok(failures) => failures,
            Err(err) => {
                error!("Resend error: {err:?}");
                return true;
            }
        };

        for target in failures {
            if let Err(send_err) = self.errors.try_send(ConnectionError::new(target)) {
                match send_err {
                    TrySendError::Closed(_) => {
                        return true;
                    }
                    TrySendError::Full(_) => {
                        warn!("Connection error channel is full.");
                        continue;
                    }
                }
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

/// Setups a communicator and network processor couple.
pub fn setup_processor(network: Network) -> (Communicator, Processor) {
    let (outputs_sender, outputs_receiver) = bounded(CHANNEL_CAPACITY);
    let (inputs_sender, inputs_receiver) = bounded(CHANNEL_CAPACITY);
    let (errors_sender, errors_receiver) = bounded(CHANNEL_CAPACITY);

    let communicator = Communicator::new(outputs_sender, inputs_receiver, errors_receiver);
    let messages = Messages::new(network);
    let processor = Processor::new(messages, outputs_receiver, inputs_sender, errors_sender);
    (communicator, processor)
}
