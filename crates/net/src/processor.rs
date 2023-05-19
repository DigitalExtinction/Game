use std::{net::SocketAddr, time::Instant};

use async_std::channel::{
    bounded, Receiver, RecvError, SendError, Sender, TryRecvError, TrySendError,
};
use futures::FutureExt;
use thiserror::Error;
use tracing::{error, info, warn};

use crate::{
    header::{DatagramCounter, DatagramHeader},
    messages::{Messages, MsgRecvError},
    reliability::Reliability,
    Network, MAX_DATAGRAM_SIZE, MAX_MESSAGE_SIZE,
};

const CHANNEL_CAPACITY: usize = 1024;

/// A message / datagram to be delivered.
pub struct OutMessage {
    data: Vec<u8>,
    reliable: bool,
    targets: Vec<SocketAddr>,
}

impl OutMessage {
    /// # Arguments
    ///
    /// * `data` - data to be send.
    ///
    /// * `reliable` - whether to deliver the data reliably.
    ///
    /// * `targets` - list of message recipients.
    ///
    /// # Panics
    ///
    /// Panics if data is longer than [`MAX_MESSAGE_SIZE`].
    pub fn new(data: Vec<u8>, reliable: bool, targets: Vec<SocketAddr>) -> Self {
        assert!(data.len() < MAX_MESSAGE_SIZE);
        Self {
            data,
            reliable,
            targets,
        }
    }
}

/// A received message / datagram.
pub struct InMessage {
    data: Vec<u8>,
    reliable: bool,
    source: SocketAddr,
}

impl InMessage {
    pub fn data(self) -> Vec<u8> {
        self.data
    }

    /// Whether the datagram was delivered reliably.
    pub fn reliable(&self) -> bool {
        self.reliable
    }

    pub fn source(&self) -> SocketAddr {
        self.source
    }
}

pub struct ConnectionError {
    target: SocketAddr,
}

impl ConnectionError {
    pub fn target(&self) -> SocketAddr {
        self.target
    }
}

/// This struct handles communication with a side async loop with the network
/// communication.
pub struct Communicator {
    outputs: Sender<OutMessage>,
    inputs: Receiver<InMessage>,
    errors: Receiver<ConnectionError>,
}

impl Communicator {
    fn new(
        outputs: Sender<OutMessage>,
        inputs: Receiver<InMessage>,
        errors: Receiver<ConnectionError>,
    ) -> Self {
        Self {
            outputs,
            inputs,
            errors,
        }
    }

    pub async fn recv(&mut self) -> Result<InMessage, RecvError> {
        self.inputs.recv().await
    }

    pub async fn send(&mut self, message: OutMessage) -> Result<(), SendError<OutMessage>> {
        self.outputs.send(message).await
    }

    pub async fn errors(&mut self) -> Result<ConnectionError, RecvError> {
        self.errors.recv().await
    }
}

/// This struct implements an async loop which handles the network
/// communication.
pub struct Processor {
    buf: [u8; MAX_DATAGRAM_SIZE],
    messages: Messages,
    counter: DatagramCounter,
    reliability: Reliability,
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
            counter: DatagramCounter::zero(),
            reliability: Reliability::new(),
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
                .reliability
                .send_confirms(&mut self.buf, &mut self.messages, Instant::now())
                .await
            {
                error!("Message confirmation error: {err:?}");
                break;
            }

            if self.handle_resends().await {
                info!("Errors finished...");
                break;
            }
        }
    }

    async fn handle_output(&mut self) -> bool {
        match self.outputs.try_recv() {
            Ok(message) => {
                let header = if message.reliable {
                    self.counter.increment();
                    self.counter.to_header()
                } else {
                    DatagramHeader::Anonymous
                };

                match self
                    .messages
                    .send_separate(&mut self.buf, header, &message.data, &message.targets)
                    .await
                {
                    Ok(()) => {
                        if let DatagramHeader::Reliable(id) = header {
                            let time = Instant::now();
                            for target in message.targets {
                                self.reliability.sent(target, id, &message.data, time);
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

        let reliable = match header {
            DatagramHeader::Confirmation => {
                self.reliability.confirmed(source, data, Instant::now());
                return Ok(());
            }
            DatagramHeader::Anonymous => false,
            DatagramHeader::Reliable(id) => {
                self.reliability.received(source, id, Instant::now());
                true
            }
        };

        self.inputs
            .send(InMessage {
                data: data.to_vec(),
                reliable,
                source,
            })
            .await
            .map_err(InputHandlingError::from)?;

        Ok(())
    }

    async fn handle_resends(&mut self) -> bool {
        if let Err(err) = self
            .reliability
            .resend(&mut self.buf, &mut self.messages, Instant::now())
            .await
        {
            for &target in err.targets() {
                if let Err(send_err) = self.errors.try_send(ConnectionError { target }) {
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
