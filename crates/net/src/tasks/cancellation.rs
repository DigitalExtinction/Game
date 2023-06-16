use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub(super) struct CancellationSender(Arc<AtomicBool>);

impl Drop for CancellationSender {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

pub(super) struct CancellationRecv(Arc<AtomicBool>);

impl CancellationRecv {
    pub(super) fn cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

/// Creates a cancellation sender / receiver pair. Once the sender gets
/// dropped, the receiver signals cancellation.
pub(super) fn cancellation() -> (CancellationSender, CancellationRecv) {
    let flag = Arc::new(AtomicBool::new(false));
    (CancellationSender(flag.clone()), CancellationRecv(flag))
}
