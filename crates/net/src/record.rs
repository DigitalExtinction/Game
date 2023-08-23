use std::time::Instant;

use crate::header::PackageHeader;

#[derive(Debug, PartialEq, Clone)]
pub(super) struct DeliveryRecord {
    time: Instant,
    header: PackageHeader,
}

impl DeliveryRecord {
    pub(crate) fn now(header: PackageHeader) -> Self {
        Self {
            time: Instant::now(),
            header,
        }
    }

    /// Original package receive time.
    pub(crate) fn time(&self) -> Instant {
        self.time
    }

    /// Package header.
    pub(crate) fn header(&self) -> PackageHeader {
        self.header
    }
}
