use std::net::SocketAddr;

use async_std::channel::{bounded, Receiver, RecvError, SendError, Sender, TryRecvError};
use futures::{future::try_join_all, FutureExt};
use tracing::info;

use crate::{Network, MAX_DATAGRAM_SIZE};

const CHANNEL_CAPACITY: usize = 1024;

/// A message / datagram to be delivered.
pub struct OutMessage {
    data: Vec<u8>,
    targets: Vec<SocketAddr>,
}

impl OutMessage {
    /// # Panics
    ///
    /// Panics if data is longer than [`MAX_DATAGRAM_SIZE`].
    pub fn new(data: Vec<u8>, targets: Vec<SocketAddr>) -> Self {
        assert!(data.len() < MAX_DATAGRAM_SIZE);
        Self { data, targets }
    }
}

/// A received message / datagram.
pub struct InMessage {
    data: Vec<u8>,
    source: SocketAddr,
}

impl InMessage {
    pub fn data(self) -> Vec<u8> {
        self.data
    }

    pub fn source(&self) -> SocketAddr {
        self.source
    }
}

/// This struct handles communication with a side async loop with the network
/// communication.
pub struct Communicator {
    outputs: Sender<OutMessage>,
    inputs: Receiver<InMessage>,
}

impl Communicator {
    fn new(outputs: Sender<OutMessage>, inputs: Receiver<InMessage>) -> Self {
        Self { outputs, inputs }
    }

    pub async fn recv(&mut self) -> Result<InMessage, RecvError> {
        self.inputs.recv().await
    }

    pub async fn send(&mut self, message: OutMessage) -> Result<(), SendError<OutMessage>> {
        self.outputs.send(message).await
    }
}

/// This struct implements an async loop which handles the network
/// communication.
pub struct Processor {
    network: Network,
    outputs: Receiver<OutMessage>,
    inputs: Sender<InMessage>,
}

impl Processor {
    fn new(network: Network, outputs: Receiver<OutMessage>, inputs: Sender<InMessage>) -> Self {
        Self {
            network,
            outputs,
            inputs,
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

        let mut buf = [0; crate::MAX_DATAGRAM_SIZE];

        loop {
            if self.handle_output().await {
                info!("Output finished...");
                break;
            }
            if self.handle_input(&mut buf).await {
                info!("Input finished...");
                break;
            }
        }
    }

    async fn handle_output(&mut self) -> bool {
        match self.outputs.try_recv() {
            Ok(message) => {
                let result = try_join_all(
                    message
                        .targets
                        .iter()
                        .map(|&target| self.network.send(target, &message.data)),
                )
                .await;

                if let Err(err) = result {
                    panic!("Send error: {:?}", err);
                }

                false
            }
            Err(err) => match err {
                TryRecvError::Empty => false,
                TryRecvError::Closed => true,
            },
        }
    }

    async fn handle_input(&mut self, buf: &mut [u8]) -> bool {
        match self.network.recv(buf).now_or_never() {
            Some(recv_result) => match recv_result {
                Ok((n, source)) => self
                    .inputs
                    .send(InMessage {
                        data: buf[0..n].to_vec(),
                        source,
                    })
                    .await
                    .is_err(),
                Err(err) => {
                    panic!("Receive error to: {:?}", err);
                }
            },
            None => false,
        }
    }
}

/// Setups a communicator and network processor couple.
pub fn setup_processor(network: Network) -> (Communicator, Processor) {
    let (outputs_sender, outputs_receiver) = bounded(CHANNEL_CAPACITY);
    let (inputs_sender, inputs_receiver) = bounded(CHANNEL_CAPACITY);
    let communicator = Communicator::new(outputs_sender, inputs_receiver);
    let processor = Processor::new(network, outputs_receiver, inputs_sender);
    (communicator, processor)
}
