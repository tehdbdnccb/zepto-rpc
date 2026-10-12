//! Shared User-Space Memory (UMEM) region management for zero-copy AF_XDP frame access.

use core::alloc::Layout;
use zepto_core::error::{Result, ZeptoError};

/// Default frame allocation size (4096 bytes per hardware packet buffer slot).
pub const UMEM_FRAME_SIZE: usize = 4096;

/// Total UMEM frame capacity per socket ring buffer.
pub const DEFAULT_UMEM_FRAME_COUNT: usize = 4096;

/// Page-aligned UMEM shared memory block.
pub struct UmemRingBuffer {
    memory_ptr: *mut u8,
    layout: Layout,
    frame_count: usize,
    frame_size: usize,
}

unsafe impl Send for UmemRingBuffer {}
unsafe impl Sync for UmemRingBuffer {}

impl UmemRingBuffer {
    /// Allocates page-aligned (4096-byte boundary) memory region for AF_XDP UMEM registration.
    pub fn allocate(frame_count: usize, frame_size: usize) -> Result<Self> {
        if frame_count == 0 || frame_size == 0 || frame_size % 2048 != 0 {
            return Err(ZeptoError::InvalidFrameSize);
        }

        let total_bytes = frame_count
            .checked_mul(frame_size)
            .ok_or(ZeptoError::BoundsViolation)?;

        let layout = Layout::from_size_align(total_bytes, 4096)
            .map_err(|_| ZeptoError::AlignmentFault)?;

        let memory_ptr = unsafe { std::alloc::alloc_zeroed(layout) };
        if memory_ptr.is_null() {
            return Err(ZeptoError::BoundsViolation);
        }

        Ok(Self {
            memory_ptr,
            layout,
            frame_count,
            frame_size,
        })
    }

    /// Obtains a direct zero-copy byte slice for an ingress frame at `frame_index`.
    #[inline(always)]
    pub fn get_frame<'a>(&'a self, frame_index: usize) -> Result<&'a [u8]> {
        if frame_index >= self.frame_count {
            return Err(ZeptoError::BoundsViolation);
        }

        let offset = frame_index * self.frame_size;
        let slice = unsafe {
            core::slice::from_raw_parts(self.memory_ptr.add(offset), self.frame_size)
        };

        Ok(slice)
    }

    /// Returns memory pointer address required by Linux kernel `setsockopt(SOL_XDP, XDP_UMEM_REG)`.
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut u8 {
        self.memory_ptr
    }

    /// Returns total size in bytes of allocated UMEM region.
    #[inline(always)]
    pub fn total_bytes(&self) -> usize {
        self.layout.size()
    }
}

impl Drop for UmemRingBuffer {
    fn drop(&mut self) {
        if !self.memory_ptr.is_null() {
            unsafe {
                std::alloc::dealloc(self.memory_ptr, self.layout);
            }
        }
    }
}