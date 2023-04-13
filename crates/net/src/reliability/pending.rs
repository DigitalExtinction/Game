use std::{num::NonZeroU32, time::Instant};

use super::{
    buffer::DatagramBuffer,
    queue::{DatagramQueue, RescheduleError},
};

pub(super) struct PendingDatagrams {
    last_id: u32,
    // TODO better solve the const generics
    buffer: DatagramBuffer<1024, 512>,
    queue: DatagramQueue,
}

impl PendingDatagrams {
    pub(super) fn new() -> Self {
        Self {
            last_id: 0,
            buffer: DatagramBuffer::new(),
            queue: DatagramQueue::new(),
        }
    }

    pub(super) fn push(&mut self, data: &[u8], now: Instant) -> NonZeroU32 {
        let id = self.next_id();
        // TODO handle error
        self.buffer.push(id, data).unwrap();
        self.queue.push(id, now);
        id
    }

    pub(super) fn remove(&mut self, id: NonZeroU32) {
        self.buffer.remove(id);
        self.queue.remove(id);
    }

    pub(super) fn reschedule(
        &mut self,
        now: Instant,
    ) -> Result<(NonZeroU32, &[u8]), RescheduleError> {
        let id = self.queue.reschedule(now)?;
        Ok((id, self.buffer.get(id).unwrap()))
    }

    fn next_id(&mut self) -> NonZeroU32 {
        self.last_id = self.last_id.wrapping_add(1);
        if self.last_id == 0 {
            self.last_id = 1;
        }
        NonZeroU32::new(self.last_id).unwrap()
    }
}
