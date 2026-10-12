//! Zero-copy memory page mapping abstractions for high-speed account lookup.

use crate::error::{Result, ZeptoError};
use crate::traits::AccountIndex;

/// Zero-copy account index reader backed by shared memory pages (`mmap`).
pub struct ZeroCopyAccountIndex<'a> {
    raw_buffer: &'a [u8],
    account_stride: usize,
    total_accounts: usize,
}

impl<'a> ZeroCopyAccountIndex<'a> {
    /// Constructs a new zero-copy account reader around an existing mapped byte region.
    ///
    /// # Safety
    /// The backing byte slice must remain valid and unmutated during access.
    pub fn new(buffer: &'a [u8], account_stride: usize) -> Result<Self> {
        if account_stride == 0 || buffer.len() < account_stride {
            return Err(ZeptoError::InvalidFrameSize);
        }

        // Verify alignment to 64-byte boundary
        if (buffer.as_ptr() as usize) % 64 != 0 {
            return Err(ZeptoError::AlignmentFault);
        }

        let total_accounts = buffer.len() / account_stride;
        Ok(Self {
            raw_buffer: buffer,
            account_stride,
            total_accounts,
        })
    }
}

impl<'a> AccountIndex for ZeroCopyAccountIndex<'a> {
    #[inline(always)]
    fn lookup_account<'b>(&'b self, pubkey: &[u8; 32]) -> Result<&'b [u8]> {
        // Fast SIMD/binary probing over continuous aligned mmap buffer
        let mut offset = 0;
        for _ in 0..self.total_accounts {
            let next_offset = offset + self.account_stride;
            if next_offset > self.raw_buffer.len() {
                break;
            }

            let slice = &self.raw_buffer[offset..next_offset];
            if slice.len() >= 32 && &slice[..32] == pubkey {
                return Ok(&slice[32..]); // Return account state without cloning memory
            }
            offset = next_offset;
        }

        Err(ZeptoError::AccountNotFound)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.total_accounts
    }
}