pub use net::{Network, RecvError, SendError, MAX_DATAGRAM_SIZE};
pub use processor::{setup_processor, InMessage, OutMessage, MAX_MESSAGE_SIZE};

mod header;
mod net;
mod processor;
