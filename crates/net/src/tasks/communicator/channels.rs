use std::{net::SocketAddr, ops::Deref};

use async_std::channel::{Receiver, Sender};

use super::{decode::InPackage, encode::OutPackage};

/// Channel into networking stack tasks, used for data sending.
///
/// The data-sending components of the networking stack are halted when this
/// channel is closed (dropped).
pub struct PackageSender(pub(crate) Sender<OutPackage>);

impl Deref for PackageSender {
    type Target = Sender<OutPackage>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Channel into networking stack tasks, used for data receiving.
///
/// This is based on a bounded queue, so non-receiving of packages can
/// eventually block the networking stack.
///
/// The data-receiving components of the networking stack are halted when this
/// channel is closed or dropped.
pub struct PackageReceiver(pub(crate) Receiver<InPackage>);

impl Deref for PackageReceiver {
    type Target = Receiver<InPackage>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Channel into networking stack tasks, used for receiving connection errors.
///
/// This channel is based on a bounded queue; therefore, the non-receiving of
/// errors can eventually block the networking stack.
///
/// If the connection errors are not needed, this channel can be safely
/// dropped. Its closure does not stop or block any part of the networking
/// stack. Although it must be dropped for the networking stack to fully
/// terminate.
pub struct ConnErrorReceiver(pub(crate) Receiver<ConnectionError>);

impl Deref for ConnErrorReceiver {
    type Target = Receiver<ConnectionError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// This error indicates failure to deliver a package to the target.
pub struct ConnectionError {
    target: SocketAddr,
}

impl ConnectionError {
    pub(crate) fn new(target: SocketAddr) -> Self {
        Self { target }
    }

    pub fn target(&self) -> SocketAddr {
        self.target
    }
}
