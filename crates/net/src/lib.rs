pub use header::{Peers, Reliability};
pub use protocol::MAX_PACKAGE_SIZE;
pub use socket::{RecvError, SendError, Socket, MAX_DATAGRAM_SIZE};
pub use tasks::{
    startup, ConnErrorReceiver, ConnectionError, InPackage, MessageDecoder, OutPackage,
    PackageBuilder, PackageReceiver, PackageSender,
};

mod connection;
mod header;
mod protocol;
mod record;
mod socket;
mod tasks;
