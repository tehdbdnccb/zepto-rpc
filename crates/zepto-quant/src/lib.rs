//! # ZeptoRPC Quantitative Traffic & Entropy Engine
//!
//! Sub-microsecond predictive quantitative execution models:
//! - Thermodynamic payload entropy scoring (<150ns payload spam drop)
//! - Hawkes self-exciting point process intensity estimation for burst traffic
//! - Navier-Stokes laminar flow queue scale controller

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unreachable_pub)]
#![deny(missing_docs)]

pub mod entropy;
pub mod hawkes;
pub mod queue;

pub use entropy::{ThermodynamicEntropyFilter, EntropyConfig};
pub use hawkes::{HawkesProcessModel, HawkesConfig};
pub use queue::{LaminarQueueController, QueueState};