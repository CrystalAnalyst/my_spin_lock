//  实现一个我自己的自旋锁SpinLock
#![allow(unused)]
#![allow(dead_code)]

use std::{
    cell::UnsafeCell,
    error,
    ops::{Deref, DerefMut},
    sync::atomic::AtomicBool,
};

/// 'a保证了Guard不会比SpinLock生命周期长。
pub struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
}

/// Deref trait 用于重载`不可变解引用`操作。
impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

/// DerefMut trait 用于重载`可变解引用`操作。
impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock
            .locked
            .store(false, std::sync::atomic::Ordering::Release);
    }
}

pub struct SpinLock<T> {
    // Using a boolean value to indicate
    // whether it is being locked.
    //
    // Using Atomic to make sure it can be
    // accessed simultaneously by multiple threads.
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

/// Impl Sync to ensures that our data can be shared between threads.
/// `Sync` is more strict than `Send`, `Send` meaning can be transfered.
/// But may not be shared simultaneously by many threads, unlike `RwLock`.
unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> SpinLock<T> {
    /// const fn are functions that can be called at compile-time
    /// (by being used as the value of a const, or static),
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    /// locked starts as false，lock() try to change it to true and keep trying.
    pub fn lock_with_swap(&self) -> Guard<T> {
        while self.locked.swap(true, std::sync::atomic::Ordering::Acquire) {
            // use hint::spin_lock to tell cpu that we're self-spinning
            // and is wating for a change.
            // spin_loop() differs largely from `thread::sleep()` and `thread::park()`
            // it will NOT leads to syscall that makes our thread fall asleep.
            std::hint::spin_loop();
        }
        Guard { lock: self }
    }

    /// Besides use `swap`, we can also use `CAS`(compare and exchange) ops
    /// to automatically check whether the boolean value is false.
    /// If it is, then we set it to true. This method is more understanable.
    pub fn lock_with_cas(&self) -> Guard<T> {
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
        Guard { lock: self }
    }

    /// Safety: The &mut T from lock() must be gone!
    /// (And no cheating by keeping reference to fields of that T around!)
    /// unlock method only trans it to false.
    pub unsafe fn unlock(&self) {
        self.locked
            .store(false, std::sync::atomic::Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[test]
    fn it_works_with_swap() {
        let x = SpinLock::new(Vec::new());
        thread::scope(|s| {
            s.spawn(|| x.lock_with_swap().push(42));
            s.spawn(|| {
                let mut g = x.lock_with_swap();
                g.push(21);
                g.push(21);
            });
        });
        let g = x.lock_with_swap();
        assert!(g.as_slice() == [42, 21, 21] || g.as_slice() == [21, 21, 42]);
    }

    #[test]
    fn it_works_with_cas() {
        let x = SpinLock::new(Vec::new());
        thread::scope(|s| {
            s.spawn(|| x.lock_with_cas().push(1));
            s.spawn(|| {
                let mut g = x.lock_with_cas();
                g.push(2);
                g.push(2);
            });
        });
        let g = x.lock_with_cas();
        assert!(g.as_slice() == [1, 2, 2] || g.as_slice() == [2, 2, 1]);
    }
}
