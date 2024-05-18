// 实现一个我自己的自旋锁SpinLock
#![allow(unused)]
#![allow(dead_code)]

use std::sync::atomic::AtomicBool;

pub struct SpinLock {
    locked: AtomicBool,
}
