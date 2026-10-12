//! Dynamic TPU leader schedule lookup index and active slot tracking.

use core::sync::atomic::{AtomicU64, Ordering};
use std::net::SocketAddr;
use arrayvec::ArrayVec;
use parking_lot::RwLock;
use solana_sdk::pubkey::Pubkey;
use zepto_core::error::{Result, ZeptoError};

/// Maximum look-ahead horizon for upcoming TPU leaders (current + next 3 leaders).
pub const MAX_LEADER_HORIZON: usize = 4;

/// Destination endpoint descriptor for an active or upcoming leader TPU socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaderTarget {
    /// Slot height for which this leader is scheduled.
    pub slot: u64,
    /// Solana identity public key of the leader node.
    pub identity: Pubkey,
    /// TPU transaction ingress socket address (UDP or QUIC).
    pub tpu_addr: SocketAddr,
    /// TPU forward transaction ingress socket address.
    pub tpu_forward_addr: SocketAddr,
}

/// Thread-safe, lock-optimized leader schedule cache indexed by slot height.
pub struct LeaderScheduleTable {
    current_slot: AtomicU64,
    // Slot-indexed round-robin schedule ring
    schedule_ring: RwLock<ArrayVec<LeaderTarget, 64>>,
}

impl LeaderScheduleTable {
    /// Constructs a new `LeaderScheduleTable` with zero-allocation backing buffers.
    pub fn new(initial_slot: u64) -> Self {
        Self {
            current_slot: AtomicU64::new(initial_slot),
            schedule_ring: RwLock::new(ArrayVec::new()),
        }
    }

    /// Updates the current slot height atomically.
    #[inline(always)]
    pub fn update_slot(&self, slot: u64) {
        self.current_slot.fetch_max(slot, Ordering::Release);
    }

    /// Obtains the currently active slot height.
    #[inline(always)]
    pub fn get_current_slot(&self) -> u64 {
        self.current_slot.load(Ordering::Acquire)
    }

    /// Bulk updates the leader schedule ring buffer.
    pub fn update_schedule(&self, targets: &[LeaderTarget]) -> Result<()> {
        if targets.is_empty() {
            return Err(ZeptoError::InvalidFrameSize);
        }

        let mut lock = self.schedule_ring.write();
        lock.clear();
        for &target in targets.iter().take(64) {
            lock.push(target);
        }

        Ok(())
    }

    /// Fetches the active TPU leader target alongside the next `N` upcoming speculative leaders.
    #[inline(always)]
    pub fn get_upcoming_leaders(&self) -> ArrayVec<LeaderTarget, MAX_LEADER_HORIZON> {
        let current_slot = self.get_current_slot();
        let lock = self.schedule_ring.read();
        let mut result = ArrayVec::new();

        for target in lock.iter() {
            if target.slot >= current_slot {
                result.push(*target);
                if result.is_full() {
                    break;
                }
            }
        }

        result
    }
}

impl Default for LeaderScheduleTable {
    fn default() -> Self {
        Self::new(0)
    }
}