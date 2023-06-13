pub use header::Peers;
pub use messages::MAX_MESSAGE_SIZE;
pub use net::{Network, RecvError, SendError, MAX_DATAGRAM_SIZE};
pub use protocol::{FromGame, FromServer, ToGame, ToServer};
pub use tasks::{startup, Communicator, InMessage, OutMessage, OutMessageBuilder};

mod connection;
mod header;
mod messages;
mod net;
mod protocol;
mod tasks;
