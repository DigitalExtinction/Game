use std::{marker::PhantomData, net::SocketAddr, time::Instant};

use bincode::{decode_from_slice, error::DecodeError};

use crate::{tasks::communicator::BINCODE_CONF, Peers, Reliability};

/// A received message / datagram.
pub struct InPackage {
    data: Vec<u8>,
    reliability: Reliability,
    peers: Peers,
    source: SocketAddr,
    time: Instant,
}

impl InPackage {
    pub(crate) fn new(
        data: Vec<u8>,
        reliability: Reliability,
        peers: Peers,
        source: SocketAddr,
        time: Instant,
    ) -> Self {
        Self {
            data,
            reliability,
            peers,
            source,
            time,
        }
    }

    pub fn data(self) -> Vec<u8> {
        self.data
    }

    /// Interpret the data as a sequence of encoded messages.
    pub fn decode<E>(&self) -> MessageDecoder<E>
    where
        E: bincode::Decode,
    {
        MessageDecoder {
            data: self.data.as_slice(),
            offset: 0,
            _marker: PhantomData,
        }
    }

    pub fn reliability(&self) -> Reliability {
        self.reliability
    }

    pub fn source(&self) -> SocketAddr {
        self.source
    }

    pub fn peers(&self) -> Peers {
        self.peers
    }

    /// Package arrival time.
    pub fn time(&self) -> Instant {
        self.time
    }
}

/// An iterator which decodes binary input data item by item.
pub struct MessageDecoder<'a, E>
where
    E: bincode::Decode,
{
    data: &'a [u8],
    offset: usize,
    _marker: PhantomData<E>,
}

impl<'a, E> Iterator for MessageDecoder<'a, E>
where
    E: bincode::Decode,
{
    type Item = Result<E, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.data.len() {
            return None;
        }

        match decode_from_slice(&self.data[self.offset..], BINCODE_CONF) {
            Ok((item, len)) => {
                self.offset += len;
                Some(Ok(item))
            }
            Err(err) => Some(Err(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use bincode::Decode;

    use super::*;

    #[test]
    fn test_decoding() {
        #[derive(Decode, Debug, Eq, PartialEq)]
        enum Message {
            One(u16),
            Two([u32; 2]),
        }

        let package = InPackage {
            // Message::Two([3, 4]), Message::One(1286)
            data: vec![1, 3, 4, 0, 251, 5, 6],
            reliability: Reliability::Unreliable,
            peers: Peers::Players,
            source: "127.0.0.1:1111".parse().unwrap(),
            time: Instant::now(),
        };

        let mut items: MessageDecoder<Message> = package.decode();
        let first = items.next().unwrap().unwrap();
        assert_eq!(first, Message::Two([3, 4]));
        let second = items.next().unwrap().unwrap();
        assert_eq!(second, Message::One(1286));
        assert!(items.next().is_none());
    }
}
