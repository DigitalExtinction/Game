use bincode::{Decode, Encode};
use thiserror::Error;

// This must be short enough so that it fits with all overhead into a single
// de_net package.
/// Maximum text length of a chat message;
pub const MAX_CHAT_LEN: usize = 140;

#[derive(Debug, Encode, Decode)]
pub struct ChatMessage(String);

impl TryFrom<String> for ChatMessage {
    type Error = ChatMessageError;

    fn try_from(message: String) -> Result<Self, Self::Error> {
        if message.is_empty() {
            Err(ChatMessageError::Empty)
        } else if message.len() > MAX_CHAT_LEN {
            Err(ChatMessageError::TooLong {
                len: message.len(),
                max_len: MAX_CHAT_LEN,
            })
        } else {
            Ok(Self(message))
        }
    }
}

#[derive(Debug, Error)]
pub enum ChatMessageError {
    #[error("The chat message is empty")]
    Empty,
    #[error("The chat message is too long: {len} > {max_len}")]
    TooLong { len: usize, max_len: usize },
}
