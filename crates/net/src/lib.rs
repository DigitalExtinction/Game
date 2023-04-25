pub use net::{Network, RecvError, SendError, MAX_DATAGRAM_SIZE};
pub use processor::{setup_processor, InMessage, OutMessage};

mod net;
mod processor;
