use core::sync::atomic::{AtomicBool, Ordering};

pub struct SpinLock {
    pub lock: AtomicBool,
}

impl SpinLock {
    pub const fn new() -> Self {
        SpinLock {
            lock: AtomicBool::new(false),
        }
    }
    pub fn lock(&mut self) {
        while self.lock.swap(true, Ordering::Acquire) {}
    }
    pub fn unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }
}

unsafe impl Sync for SpinLock {}