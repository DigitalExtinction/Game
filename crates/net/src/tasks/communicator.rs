use std::{marker::PhantomData, mem, net::SocketAddr, ops::Deref};

use async_std::channel::{Receiver, Sender};
use bincode::{
    config::{BigEndian, Configuration, Limit, Varint},
    decode_from_slice, encode_into_slice, encode_to_vec,
    error::{DecodeError, EncodeError},
};

use crate::{
    header::Peers,
    protocol::{Targets, MAX_PACKAGE_SIZE},
};

const BINCODE_CONF: Configuration<BigEndian, Varint, Limit<MAX_PACKAGE_SIZE>> =
    bincode::config::standard()
        .with_big_endian()
        .with_variable_int_encoding()
        .with_limit::<MAX_PACKAGE_SIZE>();

/// It cumulatively builds output packages from individual messages.
pub struct PackageBuilder {
    reliable: bool,
    peers: Peers,
    targets: Targets<'static>,
    buffer: Vec<u8>,
    used: usize,
    packages: Vec<OutPackage>,
}

impl PackageBuilder {
    pub fn new<T>(reliable: bool, peers: Peers, targets: T) -> Self
    where
        T: Into<Targets<'static>>,
    {
        Self {
            reliable,
            peers,
            targets: targets.into(),
            buffer: vec![0; MAX_PACKAGE_SIZE],
            used: 0,
            packages: Vec::new(),
        }
    }

    /// Build output packages from all pushed messages.
    ///
    /// The messages are distributed among the packages in a sequential order.
    /// Each package is filled with as many messages as it can accommodate.
    pub fn build(mut self) -> Vec<OutPackage> {
        let mut packages = self.packages;

        if self.used > 0 {
            self.buffer.truncate(self.used);
            let package =
                OutPackage::new(self.buffer, self.reliable, self.peers, self.targets.clone());
            packages.push(package);
        }

        packages
    }

    /// Push another message to the builder so that it is included in one of
    /// the resulting packages.
    pub fn push<E>(&mut self, message: &E) -> Result<(), EncodeError>
    where
        E: bincode::Encode,
    {
        match self.push_inner(message) {
            Err(EncodeError::UnexpectedEnd) => {
                let mut data = vec![0; MAX_PACKAGE_SIZE];
                mem::swap(&mut data, &mut self.buffer);
                data.truncate(self.used);
                self.used = 0;

                let package =
                    OutPackage::new(data, self.reliable, self.peers, self.targets.clone());
                self.packages.push(package);

                self.push_inner(message)
            }
            Err(err) => Err(err),
            Ok(()) => Ok(()),
        }
    }

    fn push_inner<E>(&mut self, message: &E) -> Result<(), EncodeError>
    where
        E: bincode::Encode,
    {
        let len = encode_into_slice(message, &mut self.buffer[self.used..], BINCODE_CONF)?;
        self.used += len;
        Ok(())
    }
}

/// A package to be send.
pub struct OutPackage {
    pub(super) data: Vec<u8>,
    reliable: bool,
    peers: Peers,
    pub(super) targets: Targets<'static>,
}

impl OutPackage {
    /// Creates a package from a single message.
    ///
    /// See also [`Self::new`].
    pub fn encode_single<E, T>(
        message: &E,
        reliable: bool,
        peers: Peers,
        targets: T,
    ) -> Result<Self, EncodeError>
    where
        E: bincode::Encode,
        T: Into<Targets<'static>>,
    {
        let data = encode_to_vec(message, BINCODE_CONF)?;
        Ok(Self::new(data, reliable, peers, targets))
    }

    /// # Arguments
    ///
    /// * `data` - data to be send.
    ///
    /// * `reliable` - whether to deliver the data reliably.
    ///
    /// * `targets` - package recipients.
    ///
    /// # Panics
    ///
    /// Panics if data is longer than [`MAX_PACKAGE_SIZE`].
    pub fn new<T>(data: Vec<u8>, reliable: bool, peers: Peers, targets: T) -> Self
    where
        T: Into<Targets<'static>>,
    {
        assert!(data.len() < MAX_PACKAGE_SIZE);
        Self {
            data,
            reliable,
            peers,
            targets: targets.into(),
        }
    }

    pub(super) fn reliable(&self) -> bool {
        self.reliable
    }

    pub(super) fn peers(&self) -> Peers {
        self.peers
    }
}

/// A received message / datagram.
pub struct InPackage {
    data: Vec<u8>,
    reliable: bool,
    peers: Peers,
    source: SocketAddr,
}

impl InPackage {
    pub(super) fn new(data: Vec<u8>, reliable: bool, peers: Peers, source: SocketAddr) -> Self {
        Self {
            data,
            reliable,
            peers,
            source,
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
            _marker: PhantomData::default(),
        }
    }

    /// Whether the datagram was delivered reliably.
    pub fn reliable(&self) -> bool {
        self.reliable
    }

    pub fn source(&self) -> SocketAddr {
        self.source
    }

    pub fn peers(&self) -> Peers {
        self.peers
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

/// This error indicates failure to deliver a package to the target.
pub struct ConnectionError {
    target: SocketAddr,
}

impl ConnectionError {
    pub(super) fn new(target: SocketAddr) -> Self {
        Self { target }
    }

    pub fn target(&self) -> SocketAddr {
        self.target
    }
}

/// Channel into networking stack tasks, used for data sending.
///
/// The data-sending components of the networking stack are halted when this
/// channel is closed (dropped).
pub struct PackageSender(pub(crate) Sender<OutPackage>);

impl Deref for PackageSender {
    type Target = Sender<OutPackage>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Channel into networking stack tasks, used for data receiving.
///
/// This is based on a bounded queue, so non-receiving of packages can
/// eventually block the networking stack.
///
/// The data-receiving components of the networking stack are halted when this
/// channel is closed or dropped.
pub struct PackageReceiver(pub(crate) Receiver<InPackage>);

impl Deref for PackageReceiver {
    type Target = Receiver<InPackage>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Channel into networking stack tasks, used for receiving connection errors.
///
/// This channel is based on a bounded queue; therefore, the non-receiving of
/// errors can eventually block the networking stack.
///
/// If the connection errors are not needed, this channel can be safely
/// dropped. Its closure does not stop or block any part of the networking
/// stack. Although it must be dropped for the networking stack to fully
/// terminate.
pub struct ConnErrorReceiver(pub(crate) Receiver<ConnectionError>);

impl Deref for ConnErrorReceiver {
    type Target = Receiver<ConnectionError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use bincode::Decode;

    use super::*;

    #[test]
    fn test_out_message_builder() {
        #[derive(bincode::Encode)]
        struct TestData {
            values: [u64; 16], // up to 128 bytes
        }

        let mut builder = PackageBuilder::new(
            true,
            Peers::Players,
            "127.0.0.1:1111".parse::<SocketAddr>().unwrap(),
        );

        for i in 0..10 {
            builder
                .push(&TestData {
                    // Use large u64 so that the value cannot be shrunk.
                    values: [u64::MAX - (i as u64); 16],
                })
                .unwrap();
        }

        let packages = builder.build();
        assert_eq!(packages.len(), 4);
        // 3 items + something extra for the encoding
        assert!(packages[0].data.len() >= 128 * 3);
        // less then 4 items
        assert!(packages[0].data.len() < 128 * 4);

        assert!(packages[1].data.len() >= 128 * 3);
        assert!(packages[1].data.len() < 128 * 4);
        assert!(packages[2].data.len() >= 128 * 3);
        assert!(packages[2].data.len() < 128 * 4);
        // last one contains only one leftover item
        assert!(packages[3].data.len() >= 128);
        assert!(packages[3].data.len() < 128 * 2);
    }

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
            reliable: false,
            peers: Peers::Players,
            source: "127.0.0.1:1111".parse().unwrap(),
        };

        let mut items: MessageDecoder<Message> = package.decode();
        let first = items.next().unwrap().unwrap();
        assert_eq!(first, Message::Two([3, 4]));
        let second = items.next().unwrap().unwrap();
        assert_eq!(second, Message::One(1286));
        assert!(items.next().is_none());
    }
}
