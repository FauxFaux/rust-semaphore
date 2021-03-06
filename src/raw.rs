use std::sync::atomic::{AtomicUsize, Ordering};

use parking_lot::{Condvar, Mutex};

pub struct RawSemaphore {
    active: AtomicUsize,
    capacity: usize,
    lock: Mutex<()>,
    cond: Condvar
}

impl RawSemaphore {
    pub fn new(capacity: usize) -> RawSemaphore {
        RawSemaphore {
            active: AtomicUsize::default(),
            capacity: capacity,
            lock: Mutex::new(()),
            cond: Condvar::new()
        }
    }

    #[inline]
    pub fn try_acquire(&self) -> bool {
        loop {
            let current_active = self.active.load(Ordering::SeqCst);
            assert!(current_active <= self.capacity);
            if current_active == self.capacity {
                return false;
            }
            let previous_active = self.active.compare_and_swap(
                current_active,
                current_active + 1,
                Ordering::SeqCst
            );
            if previous_active == current_active {
                return true;
            }
        }
    }

    #[inline]
    pub fn release(&self) {
        let previous_active = self.active.fetch_sub(1, Ordering::SeqCst);
        if previous_active == 1 {
            let guard = self.lock.lock();
            self.cond.notify_all();
            drop(guard)
        }
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst) > 0
    }

    #[inline]
    pub fn wait_until_inactive(&self) {
        let mut lock = self.lock.lock();

        while self.is_active() {
            self.cond.wait(&mut lock);
        }
    }
}
