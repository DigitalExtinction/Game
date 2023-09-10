use std::net::SocketAddr;

use de_net::Reliability;

pub(super) struct InMessage<M> {
    meta: MessageMeta,
    message: M,
}

impl<M> InMessage<M> {
    pub(super) fn new(source: SocketAddr, reliability: Reliability, message: M) -> Self {
        Self {
            meta: MessageMeta {
                source,
                reliability,
            },
            message,
        }
    }

    pub(super) fn meta(&self) -> MessageMeta {
        self.meta
    }

    pub(super) fn message(&self) -> &M {
        &self.message
    }
}

#[derive(Clone, Copy)]
pub(super) struct MessageMeta {
    pub(super) source: SocketAddr,
    pub(super) reliability: Reliability,
}
