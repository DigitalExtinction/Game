use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use bincode::error::EncodeError;
use de_net::{OutPackage, PackageBuilder, PackageIterator, Peers, Reliability};

const UNRELIABLE_TIMEOUT: Duration = Duration::from_millis(10);
const RELIABLE_TIMEOUT: Duration = Duration::from_millis(50);

/// Buffers of player messages and package builder.
pub(super) struct PlayerBuffer {
    unreliable: PackageBuilder,
    unordered: PackageBuilder,
    semi_ordered: PackageBuilder,
}

impl PlayerBuffer {
    pub(super) fn new(target: SocketAddr) -> Self {
        Self {
            unreliable: PackageBuilder::new(Reliability::Unreliable, Peers::Players, target),
            unordered: PackageBuilder::new(Reliability::Unordered, Peers::Players, target),
            semi_ordered: PackageBuilder::new(Reliability::SemiOrdered, Peers::Players, target),
        }
    }

    /// Pushes a single message to an appropriate buffer.
    ///
    /// # Arguments
    ///
    /// * `reliability` - reliability mode of the message.
    ///
    /// * `message` - the message to be buffered.
    ///
    /// * `time` - time of the message arrival.
    pub(super) fn push<E>(
        &mut self,
        reliability: Reliability,
        message: &E,
        time: Instant,
    ) -> Result<(), EncodeError>
    where
        E: bincode::Encode,
    {
        self.builder_mut(reliability).push(message, time)
    }

    /// Builds packages from old enough messages and removes the packages from
    /// the buffer.
    ///
    /// # Arguments
    ///
    /// * `time` - current time.
    pub(super) fn build(&mut self, time: Instant) -> PlayerPackageIterator<'_> {
        let unreliable_threshodl = self.order_time_limit(time - UNRELIABLE_TIMEOUT);
        let unordered_threshodl = self.order_time_limit(time - RELIABLE_TIMEOUT);
        let semi_ordered_threshodl = time - RELIABLE_TIMEOUT;

        PlayerPackageIterator {
            index: 0,
            iterators: [
                self.unreliable.build_old(unreliable_threshodl),
                self.unordered.build_old(unordered_threshodl),
                self.semi_ordered.build_old(semi_ordered_threshodl),
            ],
        }
    }

    /// Builds packages from all buffered messages and removes the packages
    /// from the buffer.
    pub(super) fn build_all(&mut self) -> PlayerPackageIterator<'_> {
        PlayerPackageIterator {
            index: 0,
            iterators: [
                self.unreliable.build_all(),
                self.unordered.build_all(),
                self.semi_ordered.build_all(),
            ],
        }
    }

    fn builder_mut(&mut self, reliability: Reliability) -> &mut PackageBuilder {
        match reliability {
            Reliability::Unreliable => &mut self.unreliable,
            Reliability::Unordered => &mut self.unordered,
            Reliability::SemiOrdered => &mut self.semi_ordered,
        }
    }

    fn order_time_limit(&self, time: Instant) -> Instant {
        match self.semi_ordered.latest() {
            Some(latest_semi_ordered) => time.max(latest_semi_ordered),
            None => time,
        }
    }
}

pub(super) struct PlayerPackageIterator<'a> {
    index: usize,
    iterators: [PackageIterator<'a>; 3],
}

impl<'a> Iterator for PlayerPackageIterator<'a> {
    type Item = OutPackage;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.iterators.len() {
            let item = self.iterators[self.index].next();
            if item.is_some() {
                return item;
            }
            self.index += 1;
        }

        None
    }
}
