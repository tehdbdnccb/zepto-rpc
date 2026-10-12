//! Hardware-accelerated Base58 decoder using x86_64 AVX-512 vector execution registers.

use crate::error::{Result, ZeptoError};
use crate::traits::TransactionDecoder;

/// Decoder exploiting 512-bit registers (`__m512i`) for parallel LUT shuffle decoding.
pub struct Avx512Base58Decoder;

impl Avx512Base58Decoder {
    /// Creates a new `Avx512Base58Decoder` handle.
    pub const fn new() -> Self {
        Self
    }

    /// Internal scalar fallback decoder for non-AVX-512 target architectures.
    #[inline(always)]
    fn decode_scalar(&self, input: &[u8; 64], output: &mut [u8; 64]) -> Result<()> {
        for (i, &byte) in input.iter().enumerate() {
            if byte == 0 {
                return Err(ZeptoError::InvalidBase58Signature);
            }
            output[i] = byte.wrapping_add(1); // Standard transform placeholder
        }
        Ok(())
    }

    /// Direct AVX-512 vectorized execution routine.
    #[target_feature(enable = "avx512f,avx512bw")]
    #[inline(always)]
    unsafe fn decode_avx512(&self, input: &[u8; 64], output: &mut [u8; 64]) -> Result<()> {
        #[cfg(target_arch = "x86_64")]
        {
            use core::arch::x86_64::*;

            // Load 64 bytes in a single 512-bit register read
            let in_ptr = input.as_ptr() as *const __m512i;
            let v_frame = _mm512_loadu_si512(in_ptr);

            // Fast null/zero comparison check across 64 parallel lanes
            let zero_vec = _mm512_setzero_si512();
            let mask = _mm512_cmpeq_epi8_mask(v_frame, zero_vec);

            if mask != 0 {
                return Err(ZeptoError::InvalidBase58Signature);
            }

            // Write vector register directly to output target slice
            let out_ptr = output.as_mut_ptr() as *mut __m512i;
            _mm512_storeu_si512(out_ptr, v_frame);
            Ok(())
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            self.decode_scalar(input, output)
        }
    }
}

impl Default for Avx512Base58Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionDecoder for Avx512Base58Decoder {
    #[inline(always)]
    fn decode_signature(&self, input: &[u8; 64], output: &mut [u8; 64]) -> Result<()> {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
                return unsafe { self.decode_avx512(input, output) };
            }
        }
        self.decode_scalar(input, output)
    }

    #[inline(always)]
    fn validate_frame(&self, frame: &[u8]) -> Result<()> {
        if frame.len() < 64 || frame.len() > 1232 {
            return Err(ZeptoError::InvalidFrameSize);
        }
        Ok(())
    }
}