//! High-throughput RPC ingress handler and speculative transaction submission execution route.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::Mutex;
use tracing::{debug, warn};
use zepto_core::error::{Result, ZeptoError};
use zepto_net::leader_schedule::LeaderScheduleTable;
use zepto_net::udp::MultiPathUdpMultiplexer;
use zepto_quant::entropy::ThermodynamicEntropyFilter;
use zepto_quant::hawkes::HawkesProcessModel;

/// Core RPC Ingestion state machine handling quantitative pre-filtering and packet forwarding.
pub struct RpcIngressPipeline {
    entropy_filter: ThermodynamicEntropyFilter,
    hawkes_model: Mutex<HawkesProcessModel>,
    leader_schedule: Arc<LeaderScheduleTable>,
    udp_multiplexer: Arc<MultiPathUdpMultiplexer>,
}

impl RpcIngressPipeline {
    /// Constructs a new `RpcIngressPipeline` execution handle.
    pub fn new(
        entropy_filter: ThermodynamicEntropyFilter,
        hawkes_model: HawkesProcessModel,
        leader_schedule: Arc<LeaderScheduleTable>,
        udp_multiplexer: Arc<MultiPathUdpMultiplexer>,
    ) -> Self {
        Self {
            entropy_filter,
            hawkes_model: Mutex::new(hawkes_model),
            leader_schedule,
            udp_multiplexer,
        }
    }

    /// Processes an incoming raw wire transaction frame.
    ///
    /// # Pipeline Execution Stages
    /// 1. Low-overhead thermodynamic payload entropy drop check (<150ns).
    /// 2. Hawkes self-exciting point process burst intensity tracking.
    /// 3. Zero-copy multi-path UDP speculative fan-out to upcoming TPU leaders.
    #[inline(always)]
    pub fn submit_raw_transaction(&self, payload: &[u8]) -> Result<usize> {
        // Stage 1: Thermodynamic entropy drop filter
        self.entropy_filter.validate_payload(payload)?;

        // Stage 2: Hawkes intensity update
        let now_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ZeptoError::BoundsViolation)?
            .as_nanos() as u64;

        let intensity = {
            let mut hawkes = self.hawkes_model.lock();
            hawkes.observe_arrival(now_ns)
        };

        debug!(target: "zepto_rpc", "Calculated Hawkes intensity: {}", intensity);

        // Stage 3: Fetch active target TPU leaders
        let targets = self.leader_schedule.get_upcoming_leaders();
        if targets.is_empty() {
            warn!(target: "zepto_rpc", "No active leader targets found in schedule cache");
            return Err(ZeptoError::BoundsViolation);
        }

        // Stage 4: Multi-path UDP dispatch
        self.udp_multiplexer.speculative_fanout(&targets, payload)
    }
}