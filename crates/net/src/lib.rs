pub use header::Peers;
pub use messages::{FromGame, FromServer, JoinError, ToGame, ToServer};
pub use protocol::{Targets, MAX_PACKAGE_SIZE};
pub use socket::{RecvError, SendError, Socket, MAX_DATAGRAM_SIZE};
pub use tasks::{
    startup, ConnErrorReceiver, ConnectionError, InPackage, MessageDecoder, OutPackage,
    PackageBuilder, PackageReceiver, PackageSender,
};

mod connection;
mod header;
mod messages;
mod protocol;
mod socket;
mod tasks;
