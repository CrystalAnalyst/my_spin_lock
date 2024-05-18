// 实现一个我自己的自旋锁SpinLock
#![allow(unused)]
#![allow(dead_code)]

use std::sync::atomic::AtomicBool;

pub struct SpinLock {
    // Using a boolean value to indicate
    // whether it is being locked.
    //
    // Using Atomic to make sure it can be
    // accessed simultaneously by multiple threads.
    locked: AtomicBool,
}

impl SpinLock {
    /// const fn are functions that can be called at compile-time
    /// (by being used as the value of a const, or static),
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    /// locked starts as false，lock() try to change it to true and keep trying.
    pub fn lock_with_swap(&self) {
        while self.locked.swap(true, std::sync::atomic::Ordering::Acquire) {
            // use hint::spin_lock to tell cpu that we're self-spinning
            // and is wating for a change.
            // spin_loop() differs largely from `thread::sleep()` and `thread::park()`
            // it will NOT leads to syscall that makes our thread fall asleep.
            std::hint::spin_loop();
        }
    }

    /// Besides use `swap`, we can also use `CAS`(compare and exchange) ops
    /// to automatically check whether the boolean value is false.
    /// If it is, then we set it to true. This method is more understanable.
    pub fn lock_with_cas(&self) {
        while self
            .locked
            .compare_exchange_weak(
                false,
                true,
                std::sync::atomic::Ordering::Acquire,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_err()
        {}
    }

    /// unlock method only trans it to false.
    pub fn unlock(&self) {
        self.locked
            .store(false, std::sync::atomic::Ordering::Release);
    }
}
