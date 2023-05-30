use std::fmt;

use thiserror::Error;

/// Number of bytes (at the beginning of each datagram) used up by the header.
pub(crate) const HEADER_SIZE: usize = 4;

/// This bit is set in protocol control datagrams.
const CONTROL_BIT: u8 = 0b1000_0000;
/// This bit is set on datagrams which must be delivered reliably.
const RELIABLE_BIT: u8 = 0b0100_0000;
/// This bit is set on datagrams which are sent to the server instead of other
/// players.
const SERVER_DESTINATION_BIT: u8 = 0b0010_0000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DatagramHeader {
    Confirmation,
    Data(DataHeader),
}

impl DatagramHeader {
    pub(crate) fn new_data(reliable: bool, destination: Destination, id: DatagramId) -> Self {
        Self::Data(DataHeader {
            reliable,
            destination,
            id,
        })
    }

    /// Writes the header to the beginning of a bytes buffer.
    ///
    /// # Panics
    ///
    /// Panics if the buffer is smaller than the header.
    pub(crate) fn write(&self, buf: &mut [u8]) {
        assert!(buf.len() >= HEADER_SIZE);
        let (mask, id) = match self {
            Self::Confirmation => (CONTROL_BIT, [0, 0, 0]),
            Self::Data(data_header) => {
                let mut mask = 0;
                if data_header.reliable {
                    mask |= RELIABLE_BIT;
                }
                if matches!(data_header.destination, Destination::Server) {
                    mask |= SERVER_DESTINATION_BIT;
                }
                (mask, data_header.id.to_bytes())
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
            let reliable = mask & RELIABLE_BIT > 0;
            let destination = if mask & SERVER_DESTINATION_BIT > 0 {
                Destination::Server
            } else {
                Destination::Players
            };
            Ok(Self::Data(DataHeader {
                reliable,
                destination,
                id: DatagramId::from_bytes(&data[1..HEADER_SIZE]),
            }))
        }
    }
}

impl fmt::Display for DatagramHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Confirmation => write!(f, "Confirmation"),
            Self::Data(header) => {
                write!(
                    f,
                    "Data {{ reliable: {}, destination: {}, id: {} }}",
                    header.reliable, header.destination, header.id
                )
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DataHeader {
    /// True if the datagram is delivered reliably.
    reliable: bool,
    destination: Destination,
    /// ID of the datagram.
    id: DatagramId,
}

impl DataHeader {
    pub(crate) fn reliable(&self) -> bool {
        self.reliable
    }

    pub(crate) fn destination(&self) -> Destination {
        self.destination
    }

    pub(crate) fn id(&self) -> DatagramId {
        self.id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Destination {
    Server,
    Players,
}

impl fmt::Display for Destination {
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
pub(crate) struct DatagramId(u32);

impl DatagramId {
    pub(crate) const fn zero() -> Self {
        Self(0)
    }

    /// Increments the counter by one. It wraps around to zero after reaching
    /// maximum value.
    pub(crate) fn incremented(self) -> Self {
        if self.0 >= 0xffffff {
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

impl fmt::Display for DatagramId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u32> for DatagramId {
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

        DatagramHeader::new_data(false, Destination::Server, DatagramId::zero()).write(&mut buf);
        assert_eq![&buf[0..4], &[0b0010_0000, 0, 0, 0]];
        assert_eq![&buf[4..], &[0; 252]];
        DatagramHeader::new_data(true, Destination::Server, 256.try_into().unwrap())
            .write(&mut buf);
        assert_eq![&buf[0..4], &[0b0110_0000, 0, 1, 0]];
        assert_eq![&buf[4..], &[0; 252]];

        DatagramHeader::new_data(true, Destination::Players, 1033.try_into().unwrap())
            .write(&mut buf);
        assert_eq![&buf[0..4], &[0b0100_0000, 0, 4, 9]];
        assert_eq![&buf[4..], &[0; 252]];
    }

    #[test]
    fn test_read_header() {
        let mut buf = [88u8; 256];

        buf[0..4].copy_from_slice(&[64, 0, 0, 0]);
        assert_eq!(
            DatagramHeader::read(&buf).unwrap(),
            DatagramHeader::new_data(true, Destination::Players, 0.try_into().unwrap())
        );

        buf[0..4].copy_from_slice(&[64, 1, 0, 3]);
        assert_eq!(
            DatagramHeader::read(&buf).unwrap(),
            DatagramHeader::new_data(true, Destination::Players, 65539.try_into().unwrap())
        );

        buf[0..4].copy_from_slice(&[32, 0, 0, 2]);
        assert_eq!(
            DatagramHeader::read(&buf).unwrap(),
            DatagramHeader::new_data(false, Destination::Server, 2.try_into().unwrap())
        );
    }

    #[test]
    fn test_id() {
        let id = DatagramId::from_bytes(&[0, 1, 0]);
        assert_eq!(id.incremented().to_bytes(), [0, 1, 1]);

        let id: DatagramId = 0xffffff.try_into().unwrap();
        assert_eq!(id.incremented(), 0.try_into().unwrap());
    }
}
