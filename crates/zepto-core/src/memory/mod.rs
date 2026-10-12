//! Zero-copy memory management and cache-line padded lock-free concurrency structures.

pub mod mmap;
pub mod ring;

pub use mmap::ZeroCopyAccountIndex;
pub use ring::LockFreeSpscRing;