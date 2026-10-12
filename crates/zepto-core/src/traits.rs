//! Abstract contract boundaries enforcing Dependency Inversion (DIP).

use crate::error::Result;

/// Direct, zero-copy account state reader abstraction.
pub trait AccountIndex {
    /// Performs an $\mathcal{O}(1)$ lookup of account data given a 32-byte public key slice.
    ///
    /// Returns a direct zero-copy byte slice reference valid for lifetime `'a`.
    fn lookup_account<'a>(&'a self, pubkey: &[u8; 32]) -> Result<&'a [u8]>;

    /// Returns total active account records held in memory.
    fn len(&self) -> usize;

    /// Checks if the account index is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Hardware-accelerated transaction parser contract.
pub trait TransactionDecoder {
    /// Decodes a 64-byte Base58 raw transaction signature into a fixed binary array.
    fn decode_signature(&self, input: &[u8; 64], output: &mut [u8; 64]) -> Result<()>;

    /// Validates raw transaction header and account key layout in nanoseconds.
    fn validate_frame(&self, frame: &[u8]) -> Result<()>;
}

/// High-frequency lock-free queue contract.
pub trait IngressQueue<T> {
    /// Attempts to push a item to the tail of the queue without locking.
    fn try_push(&self, item: T) -> Result<()>;

    /// Attempts to pop an item from the head of the queue without locking.
    fn try_pop(&self) -> Result<T>;

    /// Returns the queue capacity.
    fn capacity(&self) -> usize;
}