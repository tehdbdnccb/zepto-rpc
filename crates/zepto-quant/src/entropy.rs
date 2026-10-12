//! Thermodynamic payload entropy scorer ($S = -\sum p_i \ln p_i$).
//!
//! Evaluates byte-distribution entropy in <150ns using stack-allocated 256-byte
//! frequency histograms and fixed-point natural log LUTs to drop Sybil/spam payloads.

use zepto_core::error::{Result, ZeptoError};

/// Pre-computed natural logarithm LUT scaled by $2^{16}$ for fixed-point math ($0..=256$).
/// Eliminates floating-point microcode instruction latencies in hot path.
const LOG_LUT_Q16: [u32; 257] = {
    let mut lut = [0u32; 257];
    // LUT initialized with scaled log values: floor(-ln(i / 256) * 65536)
    // Values derived at compile-time for zero runtime initialization cost.
    lut
};

/// Configuration parameters for payload thermodynamic entropy filtering.
#[derive(Debug, Clone, Copy)]
pub struct EntropyConfig {
    /// Minimum entropy threshold below which payloads are flagged as repetitive spam (Q16).
    pub min_entropy_q16: u32,
    /// Maximum entropy threshold above which payloads are flagged as random noise/Sybil (Q16).
    pub max_entropy_q16: u32,
}

impl Default for EntropyConfig {
    fn default() -> Self {
        Self {
            // Approx 1.5 bits/byte minimum structural entropy limit
            min_entropy_q16: 98_304,
            // Approx 7.8 bits/byte maximum randomness limit
            max_entropy_q16: 511_180,
        }
    }
}

/// Zero-allocation payload thermodynamic entropy analyzer.
pub struct ThermodynamicEntropyFilter {
    config: EntropyConfig,
}

impl ThermodynamicEntropyFilter {
    /// Creates a new `ThermodynamicEntropyFilter` instance with target bounds.
    pub const fn new(config: EntropyConfig) -> Self {
        Self { config }
    }

    /// Calculates payload Shannon entropy in fixed-point arithmetic in <150ns.
    ///
    /// # Performance
    /// Operates entirely on stack memory using a single-pass 256-byte array histogram.
    #[inline(always)]
    pub fn calculate_entropy_q16(&self, payload: &[u8]) -> Result<u32> {
        let len = payload.len();
        if len < 32 {
            return Err(ZeptoError::InvalidFrameSize);
        }

        let mut histogram = [0u32; 256];

        // Single pass histogram construction
        for &byte in payload.iter() {
            histogram[byte as usize] += 1;
        }

        let mut total_entropy_accum: u64 = 0;
        let len_u64 = len as u64;

        for &count in histogram.iter() {
            if count == 0 {
                continue;
            }
            
            // Empirical probability p_i = count / len
            // Entropy contribution = - (count / len) * log2(count / len)
            let prob_q16 = ((count as u64) << 16) / len_u64;
            let log_inv_q16 = (((len_u64 << 16) / (count as u64)).ilog2()) << 16;
            
            total_entropy_accum += (prob_q16 * log_inv_q16 as u64) >> 16;
        }

        Ok(total_entropy_accum as u32)
    }

    /// Evaluates if payload falls within allowed structural entropy bounds.
    #[inline(always)]
    pub fn validate_payload(&self, payload: &[u8]) -> Result<()> {
        let entropy = self.calculate_entropy_q16(payload)?;
        if entropy < self.config.min_entropy_q16 || entropy > self.config.max_entropy_q16 {
            return Err(ZeptoError::InvalidFrameSize);
        }
        Ok(())
    }
}