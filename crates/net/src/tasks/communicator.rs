use std::{marker::PhantomData, mem, net::SocketAddr, ops::Deref};

use async_std::channel::{Receiver, Sender};
use bincode::{
    config::{BigEndian, Configuration, Limit, Varint},
    decode_from_slice, encode_into_slice, encode_to_vec,
    error::{DecodeError, EncodeError},
};

use crate::{
    header::Peers,
    messages::{Targets, MAX_MESSAGE_SIZE},
};

const BINCODE_CONF: Configuration<BigEndian, Varint, Limit<MAX_MESSAGE_SIZE>> =
    bincode::config::standard()
        .with_big_endian()
        .with_variable_int_encoding()
        .with_limit::<MAX_MESSAGE_SIZE>();

/// It cumulatively builds output messages from encodable data items.
pub struct OutMessageBuilder {
    reliable: bool,
    peers: Peers,
    targets: Targets<'static>,
    buffer: Vec<u8>,
    used: usize,
    messages: Vec<OutMessage>,
}

impl OutMessageBuilder {
    pub fn new<T>(reliable: bool, peers: Peers, targets: T) -> Self
    where
        T: Into<Targets<'static>>,
    {
        Self {
            reliable,
            peers,
            targets: targets.into(),
            buffer: vec![0; MAX_MESSAGE_SIZE],
            used: 0,
            messages: Vec::new(),
        }
    }

    /// Build output messages from all pushed data items. Data items are split
    /// among the messages in order.
    pub fn build(mut self) -> Vec<OutMessage> {
        let mut messages = self.messages;

        if self.used > 0 {
            self.buffer.truncate(self.used);
            let message =
                OutMessage::new(self.buffer, self.reliable, self.peers, self.targets.clone());
            messages.push(message);
        }

        messages
    }

    /// Push another data item to the builder so that it is included in one of
    /// the resulting messages.
    pub fn push<E>(&mut self, payload: &E) -> Result<(), EncodeError>
    where
        E: bincode::Encode,
    {
        match self.push_inner(payload) {
            Err(EncodeError::UnexpectedEnd) => {
                let mut data = vec![0; MAX_MESSAGE_SIZE];
                mem::swap(&mut data, &mut self.buffer);
                data.truncate(self.used);
                self.used = 0;

                let message =
                    OutMessage::new(data, self.reliable, self.peers, self.targets.clone());
                self.messages.push(message);

                self.push_inner(payload)
            }
            Err(err) => Err(err),
            Ok(()) => Ok(()),
        }
    }

    fn push_inner<E>(&mut self, payload: &E) -> Result<(), EncodeError>
    where
        E: bincode::Encode,
    {
        let len = encode_into_slice(payload, &mut self.buffer[self.used..], BINCODE_CONF)?;
        self.used += len;
        Ok(())
    }
}

/// A message / datagram to be delivered.
pub struct OutMessage {
    pub(super) data: Vec<u8>,
    reliable: bool,
    peers: Peers,
    pub(super) targets: Targets<'static>,
}

impl OutMessage {
    /// Creates datagram message from a single encodable item.
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
    /// * `targets` - message recipients.
    ///
    /// # Panics
    ///
    /// Panics if data is longer than [`MAX_MESSAGE_SIZE`].
    pub fn new<T>(data: Vec<u8>, reliable: bool, peers: Peers, targets: T) -> Self
    where
        T: Into<Targets<'static>>,
    {
        assert!(data.len() < MAX_MESSAGE_SIZE);
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
pub struct InMessage {
    data: Vec<u8>,
    reliable: bool,
    peers: Peers,
    source: SocketAddr,
}

impl InMessage {
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

    /// Interpret the data as a sequence of encoded items.
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

/// This error indicates failure to deliver a message to the target.
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
pub struct MessageSender(pub(crate) Sender<OutMessage>);

impl Deref for MessageSender {
    type Target = Sender<OutMessage>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Channel into networking stack tasks, used for data receiving.
///
/// This is based on a bounded queue, so non-receiving of messages can
/// eventually block the networking stack.
///
/// The data-receiving components of the networking stack are halted when this
/// channel is closed or dropped.
pub struct MessageReceiver(pub(crate) Receiver<InMessage>);

impl Deref for MessageReceiver {
    type Target = Receiver<InMessage>;

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

        let mut builder = OutMessageBuilder::new(
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

        let messages = builder.build();
        assert_eq!(messages.len(), 4);
        // 3 items + something extra for the encoding
        assert!(messages[0].data.len() >= 128 * 3);
        // less then 4 items
        assert!(messages[0].data.len() < 128 * 4);

        assert!(messages[1].data.len() >= 128 * 3);
        assert!(messages[1].data.len() < 128 * 4);
        assert!(messages[2].data.len() >= 128 * 3);
        assert!(messages[2].data.len() < 128 * 4);
        // last one contains only one leftover item
        assert!(messages[3].data.len() >= 128);
        assert!(messages[3].data.len() < 128 * 2);
    }

    #[test]
    fn test_decoding() {
        #[derive(Decode, Debug, Eq, PartialEq)]
        enum Message {
            One(u16),
            Two([u32; 2]),
        }

        let message = InMessage {
            // Message::Two([3, 4]), Message::One(1286)
            data: vec![1, 3, 4, 0, 251, 5, 6],
            reliable: false,
            peers: Peers::Players,
            source: "127.0.0.1:1111".parse().unwrap(),
        };

        let mut items: MessageDecoder<Message> = message.decode();
        let first = items.next().unwrap().unwrap();
        assert_eq!(first, Message::Two([3, 4]));
        let second = items.next().unwrap().unwrap();
        assert_eq!(second, Message::One(1286));
        assert!(items.next().is_none());
    }
}
