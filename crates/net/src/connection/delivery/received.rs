use std::{cmp::Ordering, collections::BTreeSet};

use thiserror::Error;

use crate::header::{PackageId, PackageIdRange};

// This must be less than `u32::MAX / 2` due to ID ordering circularity issues.
const MAX_SKIPPED: usize = 1024;

/// Database of already received packages. It servers for duplicate and
/// out-of-order delivery detection.
pub(super) struct Received {
    highest_id: Option<PackageId>,
    holes: BTreeSet<PackageId>,
}

impl Received {
    pub(super) fn new() -> Self {
        Self {
            highest_id: None,
            holes: BTreeSet::new(),
        }
    }

    /// Registers package as received and returns delivery order continuity in
    /// respect with earlier sent packages.
    pub(super) fn process(&mut self, id: PackageId) -> Result<IdContinuity, ReceivedIdError> {
        let range_start = match self.highest_id {
            Some(highest) => match highest.cmp(&id) {
                Ordering::Less => highest.incremented(),
                Ordering::Greater => {
                    if self.holes.remove(&id) {
                        return Ok(match self.holes.first() {
                            Some(first) => {
                                if first.cmp(&id).is_ge() {
                                    IdContinuity::Continuous(*first)
                                } else {
                                    IdContinuity::Sparse
                                }
                            }
                            None => IdContinuity::Continuous(highest.incremented()),
                        });
                    } else {
                        return Err(ReceivedIdError::Duplicate);
                    }
                }
                Ordering::Equal => {
                    return Err(ReceivedIdError::Duplicate);
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

        Ok(if skipped == 0 {
            IdContinuity::Continuous(id.incremented())
        } else {
            IdContinuity::Sparse
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum IdContinuity {
    /// Some of the earlier packages has not yet been received.
    Sparse,
    /// All earlier packages has been received. Delivery discontinuity starts
    /// at the attached ID (i.e. first non yet received package).
    Continuous(PackageId),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub(crate) enum ReceivedIdError {
    #[error("Duplicate package")]
    Duplicate,
    #[error("Too many packages skipped: {0}")]
    TooManySkipped(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_received() {
        let mut received = Received::new();

        assert_eq!(
            received.process(PackageId::from_bytes(&[0, 0, 2])),
            Ok(IdContinuity::Sparse)
        );
        assert_eq!(
            received.process(PackageId::from_bytes(&[0, 0, 1])),
            Ok(IdContinuity::Sparse)
        );
        assert_eq!(
            received.process(PackageId::from_bytes(&[0, 0, 1])),
            Err(ReceivedIdError::Duplicate)
        );
        assert_eq!(
            received.process(PackageId::from_bytes(&[0, 0, 0])),
            Ok(IdContinuity::Continuous(PackageId::from_bytes(&[0, 0, 3])))
        );

        assert_eq!(
            received.process(PackageId::from_bytes(&[0, 0, 5])),
            Ok(IdContinuity::Sparse)
        );
        assert_eq!(
            received.process(PackageId::from_bytes(&[0, 0, 3])),
            Ok(IdContinuity::Continuous(PackageId::from_bytes(&[0, 0, 4])))
        );
        assert_eq!(
            received.process(PackageId::from_bytes(&[0, 0, 5])),
            Err(ReceivedIdError::Duplicate)
        );
        assert_eq!(
            received.process(PackageId::from_bytes(&[0, 0, 6])),
            Ok(IdContinuity::Sparse)
        );
        assert_eq!(
            received.process(PackageId::from_bytes(&[0, 0, 3])),
            Err(ReceivedIdError::Duplicate)
        );

        assert_eq!(
            received.process(PackageId::from_bytes(&[50, 0, 6])),
            Err(ReceivedIdError::TooManySkipped(3276800))
        );
    }
}
