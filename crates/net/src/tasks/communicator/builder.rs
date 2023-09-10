use std::{collections::VecDeque, mem, net::SocketAddr, time::Instant};

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
    buffer: Buffer,
    latest: Option<Instant>,
    packages: VecDeque<OutPackage>,
}

impl PackageBuilder {
    pub fn new(reliability: Reliability, peers: Peers, target: SocketAddr) -> Self {
        Self {
            reliability,
            peers,
            target,
            latest: None,
            buffer: Buffer::new(),
            packages: VecDeque::new(),
        }
    }

    /// Time of newest message in the buffer.
    pub fn latest(&self) -> Option<Instant> {
        self.latest
    }

    /// Build packages from all messages pushed before a given threshold. The
    /// last yielded package may contain newer data.
    ///
    /// See [`Self::build_all`].
    pub fn build_old(&mut self, threshold: Instant) -> PackageIterator<'_> {
        if self.buffer.birth().map_or(false, |t| t <= threshold) {
            self.build_package(false);
        }

        // Threshold is used only in the condition to build package from
        // currently buffered messages. It makes little sense to make yielding
        // of already build packages conditional because the packages cannot
        // change in the future.
        PackageIterator::new(&mut self.latest, &mut self.packages)
    }

    /// Build packages from all pushed messages.
    ///
    /// The messages are distributed among the packages in a sequential order.
    /// Each package except the last one is filled with as many messages as it
    /// can accommodate.
    pub fn build_all(&mut self) -> PackageIterator<'_> {
        self.build_package(true);
        PackageIterator::new(&mut self.latest, &mut self.packages)
    }

    /// Push another message to the builder so that it is included in one of
    /// the resulting packages.
    ///
    /// It is assumed that messages are pushed in the order of their time.
    ///
    /// # Arguments
    ///
    /// * `message` - message to be pushed to the buffer.
    ///
    /// * `time` - time of creation of the message.
    pub fn push<E>(&mut self, message: &E, time: Instant) -> Result<(), EncodeError>
    where
        E: bincode::Encode,
    {
        self.latest = Some(time);
        match self.push_inner(message, time) {
            Err(EncodeError::UnexpectedEnd) => {
                self.build_package(false);
                self.push_inner(message, time)
            }
            Err(err) => Err(err),
            Ok(()) => Ok(()),
        }
    }

    fn push_inner<E>(&mut self, message: &E, time: Instant) -> Result<(), EncodeError>
    where
        E: bincode::Encode,
    {
        let len = encode_into_slice(message, self.buffer.unused_mut(), BINCODE_CONF)?;
        self.buffer.forward(len, time);
        Ok(())
    }

    /// Build and store another package from already buffered data (if there is
    /// any).
    ///
    /// # Arguments
    ///
    /// * `empty` - if true, newly created buffer for further messages will
    ///   be empty.
    fn build_package(&mut self, empty: bool) {
        let Some(data) = self.buffer.consume(empty) else {
            return;
        };

        self.packages.push_back(OutPackage::new(
            data,
            self.reliability,
            self.peers,
            self.target,
        ));
    }
}

struct Buffer {
    /// Time of the first piece of data.
    birth: Option<Instant>,
    data: Vec<u8>,
    used: usize,
}

impl Buffer {
    fn new() -> Self {
        Self {
            birth: None,
            data: vec![0; MAX_DATAGRAM_SIZE],
            used: HEADER_SIZE,
        }
    }

    /// Returns true if no data was pushed to the buffer.
    fn empty(&self) -> bool {
        self.used <= HEADER_SIZE
    }

    fn birth(&self) -> Option<Instant> {
        self.birth
    }

    /// Resets the buffer and returns the old data (before the reset). If there
    /// was no data pushed, it returns None.
    ///
    /// # Arguments
    ///
    /// * `empty` - if true, the new buffer may be created with zero capacity
    ///   as an optimization.
    fn consume(&mut self, empty: bool) -> Option<Vec<u8>> {
        if self.empty() {
            return None;
        }

        let (mut data, used) = if empty {
            (Vec::new(), 0)
        } else {
            (vec![0; MAX_DATAGRAM_SIZE], HEADER_SIZE)
        };

        mem::swap(&mut data, &mut self.data);
        data.truncate(self.used);
        self.used = used;
        self.birth = None;

        Some(data)
    }

    /// Returns mutable slice to the unused part of the buffer.
    fn unused_mut(&mut self) -> &mut [u8] {
        &mut self.data[self.used..]
    }

    /// Moves used data pointer forward and sets birth time to now if it is not
    /// set already.
    ///
    /// # Panics
    ///
    /// May panic if the pointer is moved beyond the buffer capacity.
    fn forward(&mut self, amount: usize, time: Instant) {
        if self.birth.is_none() {
            self.birth = Some(time);
        }

        self.used += amount;
        debug_assert!(self.used <= self.data.len());
    }
}

/// Iterator over already build packages.
///
/// Not consumed packages stay in the buffer.
pub struct PackageIterator<'a> {
    latest: &'a mut Option<Instant>,
    packages: &'a mut VecDeque<OutPackage>,
}

impl<'a> PackageIterator<'a> {
    /// # Arguments
    ///
    /// * `latest` - mutable reference to a timestamp which will be reset by
    ///   this iterator (set to None) after it is fully consumed.
    ///
    /// * `packages` - packages to be pop_font by this iterator as it is
    ///   consumed.
    fn new(latest: &'a mut Option<Instant>, packages: &'a mut VecDeque<OutPackage>) -> Self {
        Self { latest, packages }
    }
}

impl<'a> Iterator for PackageIterator<'a> {
    type Item = OutPackage;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.packages.pop_front();
        if result.is_none() {
            *self.latest = None;
        }
        result
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
                .push(
                    &TestData {
                        // Use large u64 so that the value cannot be shrunk.
                        values: [u64::MAX - (i as u64); 16],
                    },
                    Instant::now(),
                )
                .unwrap();
        }

        let packages: Vec<OutPackage> = builder.build_all().collect();
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
