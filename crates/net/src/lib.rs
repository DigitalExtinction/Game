pub use header::Peers;
pub use protocol::{Targets, MAX_PACKAGE_SIZE};
pub use socket::{RecvError, SendError, Socket, MAX_DATAGRAM_SIZE};
pub use messages::{FromGame, FromServer, JoinError, ToGame, ToServer};
pub use tasks::{
    startup, ConnErrorReceiver, ConnectionError, InPackage, MessageDecoder, PackageReceiver,
    PackageSender, OutPackage, PackageBuilder,
};

mod connection;
mod header;
mod protocol;
mod socket;
mod messages;
mod tasks;
