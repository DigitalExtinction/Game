use std::net::SocketAddr;

#[derive(Copy, Clone)]
pub(crate) struct Datagram<'a> {
    target: SocketAddr,
    reliable: bool, // TODO a better name
    data: &'a [u8],
}

impl<'a> Datagram<'a> {
    pub(crate) fn new(target: SocketAddr, reliable: bool, data: &'a [u8]) -> Self {
        Self {
            target,
            reliable,
            data,
        }
    }

    pub(super) fn target(&self) -> SocketAddr {
        self.target
    }

    pub(super) fn reliable(&self) -> bool {
        self.reliable
    }

    pub(super) fn data(&self) -> &[u8] {
        self.data
    }
}
