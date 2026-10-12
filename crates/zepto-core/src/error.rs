//! Unified, zero-panic error taxonomy for hot-path execution boundaries.

use core::fmt;

/// Unified error taxonomy for `zepto-core`.
///
/// Designed to be copyable and stack-allocated without standard heap allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ZeptoError {
    /// Ingress SPSC ring buffer is completely full.
    RingBufferFull = 1,
    /// Ingress SPSC ring buffer is empty.
    RingBufferEmpty = 2,
    /// Invalid transaction frame size or layout.
    InvalidFrameSize = 3,
    /// Base58 transaction signature decoding failed due to malformed characters.
    InvalidBase58Signature = 4,
    /// AVX-512 vector hardware instructions are unavailable or failed execution.
    SimdInstructionFault = 5,
    /// Target account key was not found in the zero-copy mmap index.
    AccountNotFound = 6,
    /// Memory offset requested is out of bounds for the current mmap allocation.
    BoundsViolation = 7,
    /// Memory region alignment error (must be aligned to 64/128-byte boundaries).
    AlignmentFault = 8,
}

impl fmt::Display for ZeptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RingBufferFull => f.write_str("SPSC ring buffer capacity reached"),
            Self::RingBufferEmpty => f.write_str("SPSC ring buffer contains no available elements"),
            Self::InvalidFrameSize => f.write_str("Transaction frame failed size validation"),
            Self::InvalidBase58Signature => f.write_str("Malformed Base58 encoding in transaction"),
            Self::SimdInstructionFault => f.write_str("AVX-512 SIMD vector decoding fault"),
            Self::AccountNotFound => f.write_str("Account key absent from memory-mapped index"),
            Self::BoundsViolation => f.write_str("Requested slice exceeds memory bounds"),
            Self::AlignmentFault => f.write_str("Pointer failed required cache-line alignment"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ZeptoError {}

/// Standard Result type alias bound to `ZeptoError`.
pub type Result<T> = core::result::Result<T, ZeptoError>;