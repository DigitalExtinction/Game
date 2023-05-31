pub use header::Destination;
pub use messages::MAX_MESSAGE_SIZE;
pub use net::{Network, RecvError, SendError, MAX_DATAGRAM_SIZE};
pub use processor::{setup_processor, Communicator, InMessage, OutMessage};

mod confirmbuf;
mod databuf;
mod header;
mod messages;
mod net;
mod processor;
mod reliability;
mod resend;
