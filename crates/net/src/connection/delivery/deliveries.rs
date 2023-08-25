use std::mem;

use super::pending::Pending;
use crate::{header::PackageId, record::DeliveryRecord};

/// Iterator over packages ready to be delivered to the user. It may include
/// just received package and postponed out-of-order packages received in the
/// past.
///
/// The packages are yielded in the order of their IDs.
pub(crate) struct Deliveries<'a, 'b> {
    pending: Option<PendingDeliveries<'a>>,
    current: Option<(DeliveryRecord, Vec<u8>)>,
    buf: &'b mut [u8],
}

impl<'a, 'b> Deliveries<'a, 'b> {
    /// Creates iterator which yields 0 items.
    pub(super) fn empty(buf: &'b mut [u8]) -> Self {
        Self {
            pending: None,
            current: None,
            buf,
        }
    }

    /// Create iterator yielding solely one package.
    pub(super) fn current(
        current_record: DeliveryRecord,
        current_data: Vec<u8>,
        buf: &'b mut [u8],
    ) -> Self {
        Self {
            pending: None,
            current: Some((current_record, current_data)),
            buf,
        }
    }

    /// Constructs deliveries of pending packages and the current package.
    pub(super) fn drain(
        pending: PendingDeliveries<'a>,
        current_record: DeliveryRecord,
        current_data: Vec<u8>,
        buf: &'b mut [u8],
    ) -> Self {
        Self {
            pending: Some(pending),
            current: Some((current_record, current_data)),
            buf,
        }
    }
}

impl<'a, 'b> Iterator for Deliveries<'a, 'b> {
    type Item = (DeliveryRecord, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        let buf = &mut self.buf;

        let mut item = self.pending.as_mut().and_then(|p| p.pop(buf));

        let current_id = self.current.as_ref().map(|c| c.0.header().id());
        let item_id = item.as_ref().map(|c| c.0.header().id());
        let current_first = match (current_id, item_id) {
            (Some(current), Some(item)) => {
                assert!(current != item);
                current < item
            }
            (Some(_), None) => true,
            _ => false,
        };

        if current_first {
            mem::swap(&mut item, &mut self.current);
        }

        item
    }
}

pub(super) struct PendingDeliveries<'a> {
    bound: PackageId,
    pending: &'a mut Pending,
}

impl<'a> PendingDeliveries<'a> {
    pub(super) fn new(bound: PackageId, pending: &'a mut Pending) -> Self {
        Self { bound, pending }
    }

    fn pop(&mut self, buf: &mut [u8]) -> Option<(DeliveryRecord, Vec<u8>)> {
        self.pending
            .pop_first(self.bound, buf)
            .map(|(record, len)| (record, (buf[..len]).to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{header::PackageHeader, Peers, Reliability};

    #[test]
    fn test_empty() {
        let mut buf = [0u8; 4];
        let mut iter = Deliveries::empty(&mut buf);
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_current() {
        let current_record = DeliveryRecord::now(PackageHeader::new(
            Reliability::SemiOrdered,
            Peers::Players,
            PackageId::from_bytes(&[0, 0, 5]),
        ));
        let current_data = vec![11, 7];

        let mut buf = [0u8; 4];
        let mut iter = Deliveries::current(current_record, current_data, &mut buf);

        let (first_record, first_data) = iter.next().unwrap();
        assert_eq!(
            first_record.header().reliability(),
            Reliability::SemiOrdered
        );
        assert_eq!(
            first_record.header().id(),
            PackageId::from_bytes(&[0, 0, 5])
        );
        assert_eq!(first_data, vec![11, 7]);

        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_drain() {
        let mut pending = Pending::new();
        pending.store(
            DeliveryRecord::now(PackageHeader::new(
                Reliability::SemiOrdered,
                Peers::Players,
                PackageId::from_bytes(&[0, 0, 6]),
            )),
            &[12],
        );
        pending.store(
            DeliveryRecord::now(PackageHeader::new(
                Reliability::SemiOrdered,
                Peers::Players,
                PackageId::from_bytes(&[0, 0, 7]),
            )),
            &[13, 14],
        );
        pending.store(
            DeliveryRecord::now(PackageHeader::new(
                Reliability::SemiOrdered,
                Peers::Players,
                PackageId::from_bytes(&[0, 0, 4]),
            )),
            &[10, 7, 3],
        );

        let current_record = DeliveryRecord::now(PackageHeader::new(
            Reliability::SemiOrdered,
            Peers::Players,
            PackageId::from_bytes(&[0, 0, 5]),
        ));
        let current_data = vec![11, 7];

        let mut buf = [0u8; 4];
        let mut iter = Deliveries::drain(
            PendingDeliveries::new(PackageId::from_bytes(&[0, 0, 7]), &mut pending),
            current_record,
            current_data,
            &mut buf,
        );

        let (next_record, next_data) = iter.next().unwrap();
        assert_eq!(next_record.header().id(), PackageId::from_bytes(&[0, 0, 4]));
        assert_eq!(next_data, vec![10, 7, 3]);

        let (next_record, next_data) = iter.next().unwrap();
        assert_eq!(next_record.header().id(), PackageId::from_bytes(&[0, 0, 5]));
        assert_eq!(next_data, vec![11, 7]);

        let (next_record, next_data) = iter.next().unwrap();
        assert_eq!(next_record.header().id(), PackageId::from_bytes(&[0, 0, 6]));
        assert_eq!(next_data, vec![12]);

        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }
}
