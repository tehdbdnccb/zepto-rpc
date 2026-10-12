//! Application CLI argument parsing and runtime environment configuration.

use clap::Parser;
use std::net::SocketAddr;
use zepto_quant::EntropyConfig;
use zepto_quant::HawkesConfig;

/// Ultra-low latency RPC execution node and TPU multiplexing server.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct ServerConfig {
    /// Ingress JSON-RPC/binary transaction listening address.
    #[arg(long, env = "ZEPTO_RPC_BIND_ADDR", default_value = "0.0.0.0:8899")]
    pub rpc_bind_addr: SocketAddr,

    /// Outbound UDP fan-out multiplexer local binding address.
    #[arg(long, env = "ZEPTO_UDP_BIND_ADDR", default_value = "0.0.0.0:0")]
    pub udp_bind_addr: SocketAddr,

    /// Dedicated CPU core ID pinned for transaction ingestion and entropy evaluation.
    #[arg(long, env = "ZEPTO_INGRESS_CORE_ID", default_value_t = 0)]
    pub ingress_core_id: usize,

    /// Dedicated CPU core ID pinned for TPU network packet dispatch.
    #[arg(long, env = "ZEPTO_NET_CORE_ID", default_value_t = 1)]
    pub net_core_id: usize,

    /// Minimum payload entropy threshold (Q16 format).
    #[arg(long, env = "ZEPTO_MIN_ENTROPY_Q16", default_value_t = 98304)]
    pub min_entropy_q16: u32,

    /// Maximum payload entropy threshold (Q16 format).
    #[arg(long, env = "ZEPTO_MAX_ENTROPY_Q16", default_value_t = 511180)]
    pub max_entropy_q16: u32,

    /// Hawkes self-excitation intensity threshold for capacity scaling.
    #[arg(long, env = "ZEPTO_HAWKES_THRESHOLD", default_value_t = 1000)]
    pub hawkes_threshold: u64,
}

impl ServerConfig {
    /// Extracts entropy configuration settings for execution filtering.
    pub fn to_entropy_config(&self) -> EntropyConfig {
        EntropyConfig {
            min_entropy_q16: self.min_entropy_q16,
            max_entropy_q16: self.max_entropy_q16,
        }
    }

    /// Extracts Hawkes point process parameters for traffic surge prediction.
    pub fn to_hawkes_config(&self) -> HawkesConfig {
        HawkesConfig {
            intensity_threshold: self.hawkes_threshold,
            ..Default::default()
        }
    }
}