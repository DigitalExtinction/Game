use crate::buffer::BufferedNetwork;

pub(crate) struct AccountedNetwork {
    net: BufferedNetwork,
    counter: u32,
}

impl AccountedNetwork {
    pub(crate) fn new(net: BufferedNetwork) -> Self {
        Self { net, counter: 0 }
    }

    pub(crate) async fn start(&mut self) {
        // TODO
    }
}
