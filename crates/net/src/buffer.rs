use ahash::AHashMap;
use async_std::prelude::*;
use bincode::{
    config::{self, Configuration},
    decode_from_slice, encode_into_slice,
    error::DecodeError,
    Encode,
};
use de_core::player::{Player, PlayerRange};

use crate::{
    msg::Message,
    net::{Network, RecvError, SendError, MAX_DATAGRAM_SIZE},
};

// TODO benchmark this
// TODO document
pub(crate) struct BufferedNetwork {
    net: Network,
    buffers: AHashMap<Player, OutputBuffer>,
}

impl BufferedNetwork {
    pub(crate) fn new(net: Network) -> Self {
        let buffers = AHashMap::from_iter(PlayerRange::all().map(|p| (p, OutputBuffer::new())));
        Self { net, buffers }
    }

    pub(crate) async fn recv(&mut self) -> Result<RecvMessages, RecvError> {
        let (player, data) = self.net.recv().await?;
        Ok(RecvMessages::new(player, data))
    }

    // TODO: not generic?
    pub(crate) async fn send<E>(&mut self, player: Player, message: E) -> Result<(), SendError>
    where
        E: Encode,
    {
        let buffer = self.buffers.get_mut(&player).unwrap();
        if let Some(data) = buffer.put(message) {
            self.net.send(player, data).await?;
        }
        Ok(())
    }

    pub(crate) async fn flush(&mut self) -> Result<(), SendError> {
        for (&player, buffer) in self.buffers.iter_mut() {
            if let Some(data) = buffer.flush() {
                self.net.send(player, data).await?;
            }
        }
        Ok(())
    }
}

pub(crate) struct RecvMessages<'a> {
    config: Configuration,
    player: Player,
    offset: usize,
    data: &'a [u8],
}

impl<'a> RecvMessages<'a> {
    fn new(player: Player, data: &'a [u8]) -> Self {
        Self {
            config: config::standard(),
            player,
            offset: 0,
            data,
        }
    }

    pub(crate) fn player(&self) -> Player {
        self.player
    }
}

impl<'a> Iterator for RecvMessages<'a> {
    type Item = Result<Message, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.data.len() {
            return None;
        }

        let size_bytes = [self.data[self.offset], self.data[self.offset + 1]];
        let size = u16::from_be_bytes(size_bytes) as usize;

        let from = self.offset + 2;
        self.offset = from + size;
        Some(
            decode_from_slice(&self.data[from..self.offset], self.config)
                .map(|(message, _)| message),
        )
    }
}

struct OutputBuffer {
    config: Configuration,
    used: usize,
    data: [u8; 2 * MAX_DATAGRAM_SIZE],
}

impl OutputBuffer {
    fn new() -> Self {
        Self {
            // TODO take as param?
            config: config::standard(),
            used: 0,
            data: [0; 2 * MAX_DATAGRAM_SIZE],
        }
    }

    /// Encodes the given message to the buffer. If the buffer overflows, a
    /// slice of data to be send is returned.
    fn put<E>(&mut self, message: E) -> Option<&[u8]>
    where
        E: Encode,
    {
        let used = self.used;
        // TODO handle result
        let size = encode_into_slice(message, &mut self.data[used + 2..], self.config).unwrap();
        self.data[used..used + 2].copy_from_slice(&(size as u16).to_be_bytes());
        self.used += 2 + size;

        if self.used > MAX_DATAGRAM_SIZE {
            self.data.rotate_left(used);
            self.used -= used;
            Some(&self.data[self.data.len() - used..])
        } else {
            None
        }
    }

    fn flush(&mut self) -> Option<&[u8]> {
        if self.used > 0 {
            let used = self.used;
            self.used = 0;
            Some(&self.data[0..used])
        } else {
            None
        }
    }
}
