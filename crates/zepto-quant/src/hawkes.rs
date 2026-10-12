//! Hawkes Self-Exciting Point Process Intensity Engine.
//!
//! Models incoming transaction arrival bursts using conditional intensity:
//! $$\lambda(t) = \mu(t) + \sum_{t_i < t} \alpha e^{-\beta (t - t_i)}$$
//!
//! Predicts traffic spike probabilities to trigger pre-emptive ring buffer scaling.

use arrayvec::ArrayVec;
use zepto_core::error::{Result, ZeptoError};

/// Maximum window of historical timestamp markers tracked in fixed ring buffer memory.
const HISTORY_WINDOW_SIZE: usize = 64;

/// Configuration bounds for the Hawkes process intensity predictor.
#[derive(Debug, Clone, Copy)]
pub struct HawkesConfig {
    /// Baseline background query arrival rate ($\mu$).
    pub baseline_mu: u64,
    /// Self-excitation multiplier per incoming packet arrival ($\alpha$).
    pub alpha_decay_scale: u64,
    /// Exponential decay coefficient parameter ($\beta$).
    pub beta_decay_rate: u64,
    /// Intensity threshold ($\lambda_{\text{threshold}}$) triggering queue scaling.
    pub intensity_threshold: u64,
}

impl Default for HawkesConfig {
    fn default() -> Self {
        Self {
            baseline_mu: 100,
            alpha_decay_scale: 50,
            beta_decay_rate: 10,
            intensity_threshold: 1_000,
        }
    }
}

/// Zero-allocation Hawkes point-process traffic intensity analyzer.
pub struct HawkesProcessModel {
    config: HawkesConfig,
    timestamps: ArrayVec<u64, HISTORY_WINDOW_SIZE>,
}

impl HawkesProcessModel {
    /// Constructs a new Hawkes intensity model handle.
    pub fn new(config: HawkesConfig) -> Self {
        Self {
            config,
            timestamps: ArrayVec::new(),
        }
    }

    /// Records a new frame arrival event timestamp (in nanoseconds) and calculates current intensity $\lambda(t)$.
    #[inline(always)]
    pub fn observe_arrival(&mut self, timestamp_ns: u64) -> u64 {
        if self.timestamps.is_full() {
            self.timestamps.remove(0);
        }
        self.timestamps.push(timestamp_ns);

        self.compute_intensity(timestamp_ns)
    }

    /// Calculates current intensity $\lambda(t)$ over active event window.
    #[inline(always)]
    pub fn compute_intensity(&self, current_time_ns: u64) -> u64 {
        let mut self_excitation_sum: u64 = 0;

        for &t_i in self.timestamps.iter().rev() {
            if current_time_ns <= t_i {
                continue;
            }
            let delta_t = current_time_ns - t_i;

            // Approximated exponential decay: alpha * e^(-beta * delta_t)
            // Linearized fast bit-shift approximation for microsecond accuracy
            let decay_factor = self.config.beta_decay_rate.saturating_mul(delta_t) >> 10;
            if decay_factor > 64 {
                break; // Decay term negligible beyond threshold
            }

            let excited_term = self.config.alpha_decay_scale >> decay_factor;
            self_excitation_sum = self_excitation_sum.saturating_add(excited_term);
        }

        self.config.baseline_mu.saturating_add(self_excitation_sum)
    }

    /// Checks if current traffic excitation requires pre-emptive queue capacity scaling.
    #[inline(always)]
    pub fn should_scale_capacity(&self, current_time_ns: u64) -> bool {
        self.compute_intensity(current_time_ns) >= self.config.intensity_threshold
    }
}