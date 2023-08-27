use std::{mem, net::SocketAddr};

use bincode::{encode_into_slice, error::EncodeError};

use crate::{
    header::{Peers, Reliability, HEADER_SIZE},
    tasks::communicator::BINCODE_CONF,
    OutPackage, MAX_DATAGRAM_SIZE,
};

/// It cumulatively builds output packages from individual messages.
pub struct PackageBuilder {
    reliability: Reliability,
    peers: Peers,
    target: SocketAddr,
    buffer: Vec<u8>,
    used: usize,
    packages: Vec<OutPackage>,
}

impl PackageBuilder {
    pub fn new(reliability: Reliability, peers: Peers, target: SocketAddr) -> Self {
        Self {
            reliability,
            peers,
            target,
            buffer: vec![0; MAX_DATAGRAM_SIZE],
            used: HEADER_SIZE,
            packages: Vec::new(),
        }
    }

    /// Build output packages from all pushed messages.
    ///
    /// The messages are distributed among the packages in a sequential order.
    /// Each package is filled with as many messages as it can accommodate.
    pub fn build(mut self) -> Vec<OutPackage> {
        if self.used > HEADER_SIZE {
            self.build_package(false);
        }
        self.packages
    }

    /// Push another message to the builder so that it is included in one of
    /// the resulting packages.
    pub fn push<E>(&mut self, message: &E) -> Result<(), EncodeError>
    where
        E: bincode::Encode,
    {
        match self.push_inner(message) {
            Err(EncodeError::UnexpectedEnd) => {
                self.build_package(true);
                self.push_inner(message)
            }
            Err(err) => Err(err),
            Ok(()) => Ok(()),
        }
    }

    fn push_inner<E>(&mut self, message: &E) -> Result<(), EncodeError>
    where
        E: bincode::Encode,
    {
        let len = encode_into_slice(message, &mut self.buffer[self.used..], BINCODE_CONF)?;
        self.used += len;
        Ok(())
    }

    /// Build and store another package from already buffered data.
    ///
    /// # Arguments
    ///
    /// * `reusable` - if false, newly created buffer for further messages will
    ///   be empty.
    fn build_package(&mut self, reusable: bool) {
        let (mut data, used) = if reusable {
            (vec![0; MAX_DATAGRAM_SIZE], HEADER_SIZE)
        } else {
            (Vec::new(), 0)
        };

        mem::swap(&mut data, &mut self.buffer);
        data.truncate(self.used);
        self.used = used;

        self.packages.push(OutPackage::new(
            data,
            self.reliability,
            self.peers,
            self.target,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_out_message_builder() {
        #[derive(bincode::Encode)]
        struct TestData {
            values: [u64; 16], // up to 128 bytes
        }

        let mut builder = PackageBuilder::new(
            Reliability::Unordered,
            Peers::Players,
            "127.0.0.1:1111".parse::<SocketAddr>().unwrap(),
        );

        for i in 0..10 {
            builder
                .push(&TestData {
                    // Use large u64 so that the value cannot be shrunk.
                    values: [u64::MAX - (i as u64); 16],
                })
                .unwrap();
        }

        let packages = builder.build();
        assert_eq!(packages.len(), 4);
        // 3 items + something extra for the encoding
        assert!(packages[0].data_slice().len() >= 128 * 3);
        // less then 4 items
        assert!(packages[0].data_slice().len() < 128 * 4);

        assert!(packages[1].data_slice().len() >= 128 * 3);
        assert!(packages[1].data_slice().len() < 128 * 4);
        assert!(packages[2].data_slice().len() >= 128 * 3);
        assert!(packages[2].data_slice().len() < 128 * 4);
        // last one contains only one leftover item
        assert!(packages[3].data_slice().len() >= 128);
        assert!(packages[3].data_slice().len() < 128 * 2);
    }
}
