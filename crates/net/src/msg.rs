use bincode::{Decode, Encode};

#[derive(Encode, Decode)]
pub(crate) struct Message {
    id: u32,
    payload: Payload,
}

#[derive(Encode, Decode)]
pub(crate) struct Payload {}
