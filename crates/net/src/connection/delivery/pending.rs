use std::collections::BTreeMap;

use crate::{connection::databuf::DataBuf, header::PackageId, record::DeliveryRecord};

/// Buffer for packages received out-of-order and thus awaiting the right
/// moment to be delivered.
pub(super) struct Pending {
    ids: BTreeMap<PackageId, DeliveryRecord>,
    buf: DataBuf,
}

impl Pending {
    pub(super) fn new() -> Self {
        Self {
            ids: BTreeMap::new(),
            buf: DataBuf::new(),
        }
    }

    /// Stores a package as pending (i.e. cannot be delivered right away).
    ///
    /// # Panics
    ///
    /// * When there already is a pending package with the given `id`.
    ///
    /// * It is not a (semi-)ordered package.
    pub(super) fn store(&mut self, record: DeliveryRecord, data: &[u8]) {
        assert!(record.header().reliability().is_ordered());
        let id = record.header().id();
        let result = self.ids.insert(id, record);
        assert!(result.is_none());
        self.buf.push(id, data);
    }

    /// Finds oldest (smallest) pending package older (smaller) than the given
    /// bound. Returns None if there is no such package. Otherwise, stores the
    /// package to the given buffer and returns original package delivery
    /// record and length of the package data (as stored to the buffer).
    ///
    /// # Arguments
    ///
    /// * `bound` - exclusive ID bound.
    ///
    /// # Panics
    ///
    /// Panics if `buf` len is smaller than length of found data.
    pub(super) fn pop_first(
        &mut self,
        bound: PackageId,
        buf: &mut [u8],
    ) -> Option<(DeliveryRecord, usize)> {
        match self.ids.first_entry() {
            Some(entry) => {
                if entry.key().cmp(&bound).is_lt() {
                    let id = *entry.key();
                    let record = entry.remove();
                    Some((record, self.buf.get_and_remove(id, buf).unwrap()))
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{header::PackageHeader, Peers, Reliability, MAX_PACKAGE_SIZE};

    #[test]
    fn test_pending() {
        let pkg_a = DeliveryRecord::now(PackageHeader::new(
            Reliability::SemiOrdered,
            Peers::Server,
            PackageId::from_bytes(&[0, 0, 18]),
        ));
        let pkg_b = DeliveryRecord::now(PackageHeader::new(
            Reliability::SemiOrdered,
            Peers::Server,
            PackageId::from_bytes(&[0, 0, 14]),
        ));
        let pkg_c = DeliveryRecord::now(PackageHeader::new(
            Reliability::SemiOrdered,
            Peers::Server,
            PackageId::from_bytes(&[0, 0, 22]),
        ));

        let mut buf = [0u8; MAX_PACKAGE_SIZE];
        let mut pending = Pending::new();

        assert!(pending
            .pop_first(PackageId::from_bytes(&[0, 0, 10]), &mut buf)
            .is_none());

        pending.store(pkg_a.clone(), &[4, 5, 6, 7]);
        pending.store(pkg_b.clone(), &[1, 2, 3]);
        pending.store(pkg_c.clone(), &[92, 86]);

        assert!(pending
            .pop_first(PackageId::from_bytes(&[0, 0, 10]), &mut buf)
            .is_none());

        assert_eq!(
            pending
                .pop_first(PackageId::from_bytes(&[0, 0, 20]), &mut buf)
                .unwrap(),
            (pkg_b, 3)
        );
        assert_eq!(&buf[..3], &[1, 2, 3]);

        assert_eq!(
            pending
                .pop_first(PackageId::from_bytes(&[0, 0, 20]), &mut buf)
                .unwrap(),
            (pkg_a, 4)
        );
        assert_eq!(&buf[..4], &[4, 5, 6, 7]);

        assert!(pending
            .pop_first(PackageId::from_bytes(&[0, 0, 20]), &mut buf)
            .is_none());

        assert_eq!(
            pending
                .pop_first(PackageId::from_bytes(&[0, 0, 30]), &mut buf)
                .unwrap(),
            (pkg_c, 2)
        );
        assert_eq!(&buf[..2], &[92, 86]);

        assert!(pending
            .pop_first(PackageId::from_bytes(&[0, 0, 30]), &mut buf)
            .is_none());
    }
}
