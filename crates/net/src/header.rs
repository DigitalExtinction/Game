use std::fmt;

use thiserror::Error;

/// Number of bytes (at the beginning of each datagram) used up by the header.
pub(crate) const HEADER_SIZE: usize = 4;
const SPECIAL_BIT: u32 = 1 << 31;
const ANONYMOUS: u32 = SPECIAL_BIT;
const CONFIRMATION: u32 = SPECIAL_BIT + 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DatagramHeader {
    /// Datagram to be reliably delivered.
    Reliable(u32),
    /// An anonymous datagram (without an ID) delivered unreliably.
    Anonymous,
    Confirmation,
}

impl DatagramHeader {
    /// Writes the header to the beginning of a bytes buffer.
    ///
    /// # Panics
    ///
    /// Panics if the buffer is smaller than the header.
    pub(crate) fn write(&self, buf: &mut [u8]) {
        assert!(buf.len() >= HEADER_SIZE);
        let bytes = match self {
            Self::Reliable(id) => id.to_be_bytes(),
            Self::Anonymous => ANONYMOUS.to_be_bytes(),
            Self::Confirmation => CONFIRMATION.to_be_bytes(),
        };

        buf[0..HEADER_SIZE].copy_from_slice(&bytes);
    }

    /// Reads the header from the beginning of a bytes buffer.
    ///
    /// # Panics
    ///
    /// Panics if the buffer is smaller than header.
    pub(crate) fn read(data: &[u8]) -> Result<Self, HeaderError> {
        assert!(data.len() >= 4);
        debug_assert!(u32::BITS == (HEADER_SIZE as u32) * 8);
        let value = u32::from_be_bytes(data[0..HEADER_SIZE].try_into().unwrap());

        if value >= SPECIAL_BIT {
            if value == ANONYMOUS {
                Ok(Self::Anonymous)
            } else if value == CONFIRMATION {
                Ok(Self::Confirmation)
            } else {
                Err(HeaderError::Invalid)
            }
        } else {
            Ok(Self::Reliable(value))
        }
    }
}

impl fmt::Display for DatagramHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reliable(id) => write!(f, "Reliable({})", id),
            Self::Anonymous => write!(f, "Anonymous"),
            Self::Confirmation => write!(f, "Confirmation"),
        }
    }
}

#[derive(Error, Debug)]
pub(crate) enum HeaderError {
    #[error("The header is invalid")]
    Invalid,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DatagramCounter(u32);

impl DatagramCounter {
    pub(crate) fn zero() -> Self {
        Self(0)
    }

    /// Increments the counter by one. It wraps around to zero before reaching
    /// 2^31 (i.e. most significant bit is reserved for special values).
    pub(crate) fn increment(&mut self) {
        self.0 = if self.0 >= SPECIAL_BIT { 0 } else { self.0 + 1 };
    }

    pub(crate) fn to_header(self) -> DatagramHeader {
        DatagramHeader::Reliable(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_header() {
        let mut buf = [0u8; 256];

        DatagramHeader::Anonymous.write(&mut buf);
        assert_eq![&buf[0..4], &[128, 0, 0, 0]];
        assert_eq![&buf[4..], &[0; 252]];

        DatagramHeader::Reliable(1033).write(&mut buf);
        assert_eq![&buf[0..4], &[0, 0, 4, 9]];
        assert_eq![&buf[4..], &[0; 252]];
    }

    #[test]
    fn test_read_header() {
        let mut buf = [88u8; 256];

        buf[0..4].copy_from_slice(&[0, 0, 0, 0]);
        assert_eq!(
            DatagramHeader::read(&buf).unwrap(),
            DatagramHeader::Reliable(0)
        );

        buf[0..4].copy_from_slice(&[0, 1, 0, 3]);
        assert_eq!(
            DatagramHeader::read(&buf).unwrap(),
            DatagramHeader::Reliable(65539)
        );

        buf[0..4].copy_from_slice(&[128, 0, 0, 0]);
        assert_eq!(
            DatagramHeader::read(&buf).unwrap(),
            DatagramHeader::Anonymous
        );
    }

    #[test]
    fn test_counter() {
        let mut counter = DatagramCounter::zero();

        assert!(matches!(counter.to_header(), DatagramHeader::Reliable(0)));
        assert!(matches!(counter.to_header(), DatagramHeader::Reliable(0)));

        counter.increment();
        assert!(matches!(counter.to_header(), DatagramHeader::Reliable(1)));

        for _ in 0..1000 {
            let first = counter.to_header();
            counter.increment();
            let second = counter.to_header();
            assert_ne!(first, second);
        }

        counter.0 = u32::MAX - 1;
        counter.increment();
        assert!(matches!(counter.to_header(), DatagramHeader::Reliable(0)));
    }
}
