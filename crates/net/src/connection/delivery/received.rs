use std::cmp::Ordering;

use ahash::AHashSet;
use thiserror::Error;

use crate::header::{PackageId, PackageIdRange};

pub(super) const MAX_SKIPPED: usize = 1024;

/// Database of already received packages.
pub(super) struct Received {
    highest_id: Option<PackageId>,
    holes: AHashSet<PackageId>,
}

impl Received {
    pub(super) fn new() -> Self {
        Self {
            highest_id: None,
            holes: AHashSet::new(),
        }
    }

    /// Registers package as delivered and returns true if it was already
    /// delivered in the past.
    pub(super) fn process(&mut self, id: PackageId) -> Result<bool, ReceivedIdError> {
        let range_start = match self.highest_id {
            Some(highest) => match highest.ordering(id) {
                Ordering::Less => highest.incremented(),
                Ordering::Greater => {
                    return Ok(!self.holes.remove(&id));
                }
                Ordering::Equal => {
                    return Ok(true);
                }
            },
            None => PackageId::zero(),
        };

        let range = PackageIdRange::range(range_start, id);
        let skipped = range.size_hint().1.unwrap() + self.holes.len();
        if skipped > MAX_SKIPPED {
            return Err(ReceivedIdError::TooManySkipped(skipped));
        }

        self.highest_id = Some(id);
        for hole in range {
            self.holes.insert(hole);
        }

        Ok(false)
    }
}

#[derive(Error, Debug)]
pub(crate) enum ReceivedIdError {
    #[error("Too many packages skipped: {0}")]
    TooManySkipped(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_received() {
        let mut received = Received::new();

        assert!(!received.process(PackageId::from_bytes(&[0, 0, 2])).unwrap());
        assert!(!received.process(PackageId::from_bytes(&[0, 0, 1])).unwrap());
        assert!(received.process(PackageId::from_bytes(&[0, 0, 1])).unwrap());
        assert!(!received.process(PackageId::from_bytes(&[0, 0, 0])).unwrap());

        assert!(!received.process(PackageId::from_bytes(&[0, 0, 5])).unwrap());
        assert!(!received.process(PackageId::from_bytes(&[0, 0, 3])).unwrap());
        assert!(received.process(PackageId::from_bytes(&[0, 0, 5])).unwrap());
        assert!(!received.process(PackageId::from_bytes(&[0, 0, 6])).unwrap());
        assert!(received.process(PackageId::from_bytes(&[0, 0, 3])).unwrap());

        assert!(matches!(
            received.process(PackageId::from_bytes(&[50, 0, 6])),
            Err(ReceivedIdError::TooManySkipped(3276800))
        ));
    }
}
