use std::net::SocketAddr;

use async_std::channel::{bounded, Receiver, RecvError, SendError, Sender, TryRecvError};
use futures::{future::try_join_all, FutureExt};
use tracing::{info, trace};

use crate::{
    header::{DatagramCounter, DatagramHeader, HEADER_SIZE},
    Network, MAX_DATAGRAM_SIZE,
};

/// Maximum number of bytes of a single message.
pub const MAX_MESSAGE_SIZE: usize = MAX_DATAGRAM_SIZE - HEADER_SIZE;
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
    counter: DatagramCounter,
    outputs: Receiver<OutMessage>,
    inputs: Sender<InMessage>,
}

impl Processor {
    fn new(network: Network, outputs: Receiver<OutMessage>, inputs: Sender<InMessage>) -> Self {
        Self {
            network,
            counter: DatagramCounter::zero(),
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
            if self.handle_output(&mut buf).await {
                info!("Output finished...");
                break;
            }
            if self.handle_input(&mut buf).await {
                info!("Input finished...");
                break;
            }
        }
    }

    async fn handle_output(&mut self, buf: &mut [u8]) -> bool {
        match self.outputs.try_recv() {
            Ok(message) => {
                let header = if message.reliable {
                    self.counter.increment();
                    self.counter.to_header()
                } else {
                    DatagramHeader::Anonymous
                };

                trace!("Going to send datagram {}", header);

                header.write(buf);

                let len = HEADER_SIZE + message.data.len();
                assert!(buf.len() >= len);
                buf[HEADER_SIZE..len].copy_from_slice(&message.data);
                let data = &buf[..len];

                let result = try_join_all(
                    message
                        .targets
                        .iter()
                        .map(|&target| self.network.send(target, data)),
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
                Ok((stop, source)) => {
                    let header = match DatagramHeader::read(&buf[0..stop]) {
                        Ok(header) => header,
                        Err(err) => panic!("Header parsing failed: {err:?}"),
                    };

                    trace!("Received datagram with ID {header}");

                    let reliable = match header {
                        DatagramHeader::Anonymous => false,
                        DatagramHeader::Reliable(_) => true,
                    };

                    self.inputs
                        .send(InMessage {
                            data: buf[HEADER_SIZE..stop].to_vec(),
                            reliable,
                            source,
                        })
                        .await
                        .is_err()
                }
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
