use std::{cmp::Ordering, fmt};

use thiserror::Error;

/// Number of bytes (at the beginning of each datagram) used up by the header.
pub(crate) const HEADER_SIZE: usize = 4;

/// This bit is set in protocol control datagrams.
const CONTROL_BIT: u8 = 0b1000_0000;
/// This bit is set on datagrams which are sent to the server instead of other
/// players.
const SERVER_PEER_BIT: u8 = 0b0001_0000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DatagramHeader {
    Confirmation,
    Package(PackageHeader),
}

impl DatagramHeader {
    /// Writes the header to the beginning of a bytes buffer.
    ///
    /// # Panics
    ///
    /// Panics if the buffer is smaller than the header.
    pub(crate) fn write(&self, buf: &mut [u8]) {
        assert!(buf.len() >= HEADER_SIZE);
        let (mask, id) = match self {
            Self::Confirmation => (CONTROL_BIT, [0, 0, 0]),
            Self::Package(package_header) => {
                let mut mask = package_header.reliability().to_bits();
                if matches!(package_header.peers, Peers::Server) {
                    mask |= SERVER_PEER_BIT;
                }
                (mask, package_header.id.to_bytes())
            }
        };

        buf[0] = mask;
        buf[1..HEADER_SIZE].copy_from_slice(&id);
    }

    /// Reads the header from the beginning of a bytes buffer.
    ///
    /// # Panics
    ///
    /// Panics if the buffer is smaller than header.
    pub(crate) fn read(data: &[u8]) -> Result<Self, HeaderError> {
        assert!(data.len() >= 4);
        debug_assert!(u32::BITS == (HEADER_SIZE as u32) * 8);

        let mask = data[0];

        if mask & CONTROL_BIT > 0 {
            if mask == CONTROL_BIT {
                Ok(Self::Confirmation)
            } else {
                Err(HeaderError::Invalid)
            }
        } else {
            let reliability = Reliability::from_bits(mask)?;
            let peers = if mask & SERVER_PEER_BIT > 0 {
                Peers::Server
            } else {
                Peers::Players
            };
            Ok(Self::Package(PackageHeader {
                reliability,
                peers,
                id: PackageId::from_bytes(&data[1..HEADER_SIZE]),
            }))
        }
    }
}

impl fmt::Display for DatagramHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Confirmation => write!(f, "Confirmation"),
            Self::Package(header) => {
                write!(
                    f,
                    "Package {{ reliability: {}, peers: {}, id: {} }}",
                    header.reliability, header.peers, header.id
                )
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PackageHeader {
    /// True if the package is delivered reliably.
    reliability: Reliability,
    peers: Peers,
    id: PackageId,
}

impl PackageHeader {
    pub(crate) fn new(reliability: Reliability, peers: Peers, id: PackageId) -> Self {
        Self {
            reliability,
            peers,
            id,
        }
    }

    pub(crate) fn reliability(&self) -> Reliability {
        self.reliability
    }

    pub(crate) fn peers(&self) -> Peers {
        self.peers
    }

    pub(crate) fn id(&self) -> PackageId {
        self.id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reliability {
    /// There are no guarantees on reliability, ordering or duplicate delivery
    /// of the package.
    Unreliable,
    /// Non-duplicate delivery of the package is guaranteed. There are no
    /// guarantees on ordering of the package with respect to other packages.
    Unordered,
    /// Non-duplicate delivery of the package is guaranteed. The package is
    /// guaranteed to be delivered after all other previously reliably sent
    /// packages. There are no guarantees on ordering of the package with
    /// respect to other packages sent after this one.
    SemiOrdered,
}

impl Reliability {
    fn to_bits(self) -> u8 {
        let bits = match self {
            Self::Unreliable => 0,
            Self::Unordered => 1,
            Self::SemiOrdered => 2,
        };
        bits << 5
    }

    fn from_bits(bits: u8) -> Result<Self, HeaderError> {
        let bits = (bits >> 5) & 3;
        match bits {
            0 => Ok(Self::Unreliable),
            1 => Ok(Self::Unordered),
            2 => Ok(Self::SemiOrdered),
            _ => Err(HeaderError::Invalid),
        }
    }

    /// Returns true if the package is delivered reliably, independently on
    /// ordering.
    pub fn is_reliable(&self) -> bool {
        match self {
            Self::SemiOrdered | Self::Unordered => true,
            Self::Unreliable => false,
        }
    }

    /// Returns true if there are any guarantees on ordering of the package.
    pub fn is_ordered(&self) -> bool {
        match self {
            Self::SemiOrdered => true,
            Self::Unordered | Self::Unreliable => false,
        }
    }
}

impl fmt::Display for Reliability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreliable => write!(f, "unreliable"),
            Self::Unordered => write!(f, "unordered"),
            Self::SemiOrdered => write!(f, "semi-ordered"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Peers {
    /// Communication between networking server and a player/client.
    Server,
    /// Communication between a players (one-to-all).
    Players,
}

impl fmt::Display for Peers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Server => write!(f, "Server"),
            Self::Players => write!(f, "Players"),
        }
    }
}

#[derive(Error, Debug)]
pub(crate) enum HeaderError {
    #[error("The header is invalid")]
    Invalid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PackageId(u32);

impl PackageId {
    const MAX: u32 = 0xffffff;

    pub(crate) const fn zero() -> Self {
        Self(0)
    }

    /// Increments the counter by one. It wraps around to zero after reaching
    /// maximum value.
    pub(crate) fn incremented(self) -> Self {
        if self.0 >= Self::MAX {
            Self(0)
        } else {
            Self(self.0 + 1)
        }
    }

    /// # Panics
    ///
    /// If not exactly 3 bytes are passed.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), 3);
        let a = (bytes[0] as u32) << 16;
        let b = (bytes[1] as u32) << 8;
        let c = bytes[2] as u32;
        Self(a + b + c)
    }

    pub(crate) fn to_bytes(self) -> [u8; 3] {
        [
            ((self.0 >> 16) & 0xff) as u8,
            ((self.0 >> 8) & 0xff) as u8,
            (self.0 & 0xff) as u8,
        ]
    }
}

impl Ord for PackageId {
    /// Returns probable relative ordering of two package IDs.
    ///
    /// Note that the implementation is circular due to wrapping around maximum
    /// value and thus the ordering is not transitive.
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.cmp(&other.0) {
            Ordering::Greater => {
                if self.0.abs_diff(other.0) < Self::MAX / 2 {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            Ordering::Less => {
                if self.0.abs_diff(other.0) < Self::MAX / 2 {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            Ordering::Equal => Ordering::Equal,
        }
    }
}

impl PartialOrd for PackageId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub(crate) struct PackageIdRange {
    current: PackageId,
    stop: Option<PackageId>,
}

impl PackageIdRange {
    pub(crate) fn counter() -> Self {
        Self {
            current: PackageId::zero(),
            stop: None,
        }
    }

    /// # Arguments
    ///
    /// * `start` - inclusive start.
    ///
    /// * `stop` - exclusive stop.
    pub(crate) fn range(start: PackageId, stop: PackageId) -> Self {
        Self {
            current: start,
            stop: Some(stop),
        }
    }
}

impl Iterator for PackageIdRange {
    type Item = PackageId;

    fn next(&mut self) -> Option<Self::Item> {
        if Some(self.current) == self.stop {
            return None;
        }

        let current = self.current;
        self.current = current.incremented();
        Some(current)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let Some(stop) = self.stop else {
            return (usize::MAX, None);
        };

        let exact = match self.current.0.cmp(&stop.0) {
            Ordering::Less => stop.0 - self.current.0,
            Ordering::Equal => 0,
            Ordering::Greater => stop.0 + (PackageId::MAX - self.current.0),
        } as usize;

        (exact, Some(exact))
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u32> for PackageId {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > 0xffffff {
            Err("ID is too large")
        } else {
            Ok(Self(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_header() {
        let mut buf = [0u8; 256];

        DatagramHeader::Package(PackageHeader::new(
            Reliability::SemiOrdered,
            Peers::Server,
            PackageId::zero(),
        ))
        .write(&mut buf);
        assert_eq![&buf[0..4], &[0b0101_0000, 0, 0, 0]];
        assert_eq![&buf[4..], &[0; 252]];

        DatagramHeader::Package(PackageHeader::new(
            Reliability::Unordered,
            Peers::Server,
            256.try_into().unwrap(),
        ))
        .write(&mut buf);
        assert_eq![&buf[0..4], &[0b0011_0000, 0, 1, 0]];
        assert_eq![&buf[4..], &[0; 252]];

        DatagramHeader::Package(PackageHeader::new(
            Reliability::Unreliable,
            Peers::Players,
            1033.try_into().unwrap(),
        ))
        .write(&mut buf);
        assert_eq![&buf[0..4], &[0b0000_0000, 0, 4, 9]];
        assert_eq![&buf[4..], &[0; 252]];
    }

    #[test]
    fn test_read_header() {
        let mut buf = [88u8; 256];

        buf[0..4].copy_from_slice(&[64, 0, 0, 0]);
        assert_eq!(
            DatagramHeader::read(&buf).unwrap(),
            DatagramHeader::Package(PackageHeader::new(
                Reliability::SemiOrdered,
                Peers::Players,
                0.try_into().unwrap()
            ))
        );

        buf[0..4].copy_from_slice(&[32, 1, 0, 3]);
        assert_eq!(
            DatagramHeader::read(&buf).unwrap(),
            DatagramHeader::Package(PackageHeader::new(
                Reliability::Unordered,
                Peers::Players,
                65539.try_into().unwrap()
            ))
        );

        buf[0..4].copy_from_slice(&[16, 0, 0, 2]);
        assert_eq!(
            DatagramHeader::read(&buf).unwrap(),
            DatagramHeader::Package(PackageHeader::new(
                Reliability::Unreliable,
                Peers::Server,
                2.try_into().unwrap()
            ))
        );
    }

    #[test]
    fn test_incremented() {
        let id = PackageId::from_bytes(&[0, 1, 0]);
        assert_eq!(id.incremented().to_bytes(), [0, 1, 1]);

        let id: PackageId = 0xffffff.try_into().unwrap();
        assert_eq!(id.incremented(), 0.try_into().unwrap());
    }

    #[test]
    fn test_ordering() {
        assert_eq!(
            PackageId::from_bytes(&[0, 1, 1]).cmp(&PackageId::from_bytes(&[0, 1, 2])),
            Ordering::Less
        );
        assert_eq!(
            PackageId::from_bytes(&[0, 1, 1]).cmp(&PackageId::from_bytes(&[0, 1, 0])),
            Ordering::Greater
        );
        assert_eq!(
            PackageId::from_bytes(&[0, 1, 1]).cmp(&PackageId::from_bytes(&[0, 1, 1])),
            Ordering::Equal
        );

        assert_eq!(
            PackageId::from_bytes(&[0, 1, 2]).cmp(&PackageId::from_bytes(&[255, 255, 1])),
            Ordering::Greater
        );
        assert_eq!(
            PackageId::from_bytes(&[255, 255, 1]).cmp(&PackageId::from_bytes(&[0, 1, 2])),
            Ordering::Less
        );
    }

    #[test]
    fn test_iter() {
        let mut counter = PackageIdRange::counter();
        assert_eq!(counter.next().unwrap(), PackageId::zero());
        assert_eq!(counter.next().unwrap(), PackageId::zero().incremented());
        assert_eq!(
            counter.next().unwrap(),
            PackageId::zero().incremented().incremented()
        );
        assert_eq!(counter.next().unwrap(), PackageId::from_bytes(&[0, 0, 3]));

        let mut counter = PackageIdRange::range(
            PackageId::from_bytes(&[0, 1, 2]),
            PackageId::from_bytes(&[0, 1, 4]),
        );
        assert_eq!(counter.next().unwrap(), PackageId::from_bytes(&[0, 1, 2]));
        assert_eq!(counter.next().unwrap(), PackageId::from_bytes(&[0, 1, 3]));
        assert!(counter.next().is_none());
    }
}
