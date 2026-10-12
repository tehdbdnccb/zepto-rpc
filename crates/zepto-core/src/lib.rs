//! # ZeptoRPC Core Engine
//!
//! High-performance, zero-allocation, `#![no_std]` compatible domain logic
//! for the ZeptoRPC engine. Enforces strict zero-panic policies, cache-line
//! padded lock-free ring buffers, zero-copy mmap lookups, and AVX-512 SIMD.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unreachable_pub)]
#![deny(missing_docs)]

pub mod error;
pub mod memory;
pub mod simd;
pub mod traits;

pub use error::{Result, ZeptoError};
pub use memory::{LockFreeSpscRing, ZeroCopyAccountIndex};
pub use traits::{AccountIndex, IngressQueue, TransactionDecoder};