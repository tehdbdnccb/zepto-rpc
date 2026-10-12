//! Single-Producer Single-Consumer (SPSC) lock-free ring buffer.
//!
//! Features explicit 128-byte cache-line padding to prevent false sharing
//! and acquire-release atomic pointer synchronization.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::error::{Result, ZeptoError};
use crate::traits::IngressQueue;

/// Cache-line padding size (128 bytes) targeting modern x86_64 L1/L2 prefetchers.
const CACHE_LINE_SIZE: usize = 128;

#[repr(C, align(128))]
struct CachePaddedAtomic {
    value: AtomicUsize,
    _pad: [u8; CACHE_LINE_SIZE - core::mem::size_of::<AtomicUsize>()],
}

impl CachePaddedAtomic {
    const fn new(val: usize) -> Self {
        Self {
            value: AtomicUsize::new(val),
            _pad: [0u8; CACHE_LINE_SIZE - core::mem::size_of::<AtomicUsize>()],
        }
    }
}

/// Fixed-capacity, zero-allocation lock-free SPSC ring buffer.
pub struct LockFreeSpscRing<T, const N: usize> {
    head: CachePaddedAtomic, // Write index (owned by Producer)
    tail: CachePaddedAtomic, // Read index (owned by Consumer)
    buffer: [UnsafeCell<Option<T>>; N],
}

unsafe impl<T: Send, const N: usize> Sync for LockFreeSpscRing<T, N> {}
unsafe impl<T: Send, const N: usize> Send for LockFreeSpscRing<T, N> {}

impl<T, const N: usize> LockFreeSpscRing<T, N> {
    /// Creates a new `LockFreeSpscRing` instance.
    pub fn new() -> Self {
        assert!(N.is_power_of_two(), "Capacity N must be a power of two");
        
        // Initialize fixed UnsafeCell array
        let buffer = unsafe {
            let mut uninit: [UnsafeCell<Option<T>>; N] = core::mem::zeroed();
            for item in uninit.iter_mut() {
                core::ptr::write(item.get(), None);
            }
            uninit
        };

        Self {
            head: CachePaddedAtomic::new(0),
            tail: CachePaddedAtomic::new(0),
            buffer,
        }
    }
}

impl<T, const N: usize> Default for LockFreeSpscRing<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> IngressQueue<T> for LockFreeSpscRing<T, N> {
    #[inline(always)]
    fn try_push(&self, item: T) -> Result<()> {
        let head = self.head.value.load(Ordering::Relaxed);
        let tail = self.tail.value.load(Ordering::Acquire); // Synchronize read state

        if head.wrapping_sub(tail) >= N {
            return Err(ZeptoError::RingBufferFull);
        }

        let slot_idx = head & (N - 1); // Power of 2 bitwise mask
        unsafe {
            let slot_ptr = self.buffer[slot_idx].get();
            *slot_ptr = Some(item);
        }

        self.head.value.store(head.wrapping_add(1), Ordering::Release); // Publish write
        Ok(())
    }

    #[inline(always)]
    fn try_pop(&self) -> Result<T> {
        let tail = self.tail.value.load(Ordering::Relaxed);
        let head = self.head.value.load(Ordering::Acquire); // Synchronize write state

        if tail == head {
            return Err(ZeptoError::RingBufferEmpty);
        }

        let slot_idx = tail & (N - 1);
        let item = unsafe {
            let slot_ptr = self.buffer[slot_idx].get();
            (*slot_ptr).take().ok_or(ZeptoError::RingBufferEmpty)?
        };

        self.tail.value.store(tail.wrapping_add(1), Ordering::Release); // Publish release
        Ok(item)
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        N
    }
}