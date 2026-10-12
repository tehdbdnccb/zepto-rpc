//! Application entry point for the ZeptoRPC low-latency server.

mod config;
mod daemon;
mod rpc_interface;

use clap::Parser;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use config::ServerConfig;
use daemon::DaemonSupervisor;
use rpc_interface::RpcIngressPipeline;
use zepto_net::leader_schedule::LeaderScheduleTable;
use zepto_net::udp::{MultiPathUdpMultiplexer, UdpConfig};
use zepto_quant::entropy::ThermodynamicEntropyFilter;
use zepto_quant::hawkes::HawkesProcessModel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing log output
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().or_else(|_| EnvFilter::new("info")))
        .init();

    info!(target: "zepto_main", "Starting ZeptoRPC Engine v{}", env!("CARGO_PKG_VERSION"));

    // Parse CLI arguments & Environment variables
    let config = ServerConfig::parse();
    info!(target: "zepto_main", "Server configured with RPC bind: {}", config.rpc_bind_addr);

    // Initialize core net components
    let leader_schedule = Arc::new(LeaderScheduleTable::default());
    let udp_multiplexer = Arc::new(MultiPathUdpMultiplexer::bind(
        config.udp_bind_addr,
        UdpConfig::default(),
    )?);

    // Initialize quantitative execution components
    let entropy_filter = ThermodynamicEntropyFilter::new(config.to_entropy_config());
    let hawkes_model = HawkesProcessModel::new(config.to_hawkes_config());

    // Construct pipeline orchestrator
    let pipeline = Arc::new(RpcIngressPipeline::new(
        entropy_filter,
        hawkes_model,
        leader_schedule,
        udp_multiplexer,
    ));

    // Spawn pinned daemon threads
    let mut daemon = DaemonSupervisor::new();

    let _ingress_pipeline = Arc::clone(&pipeline);
    daemon.spawn_pinned_worker("ingress_worker", config.ingress_core_id, move |running| {
        info!(target: "zepto_worker", "Ingress worker loop running");
        while running.load(std::sync::atomic::Ordering::Acquire) {
            std::thread::hint::spin_loop();
        }
    });

    let _net_pipeline = Arc::clone(&pipeline);
    daemon.spawn_pinned_worker("net_worker", config.net_core_id, move |running| {
        info!(target: "zepto_worker", "Network dispatch worker loop running");
        while running.load(std::sync::atomic::Ordering::Acquire) {
            std::thread::hint::spin_loop();
        }
    });

    info!(target: "zepto_main", "ZeptoRPC Engine operational. Awaiting termination signal...");

    // Wait for shutdown signals (SIGINT / SIGTERM)
    match signal::ctrl_c().await {
        Ok(()) => info!(target: "zepto_main", "Received SIGINT signal"),
        Err(err) => error!(target: "zepto_main", "Failed to listen for shutdown signal: {}", err),
    }

    // Terminate worker processes cleanly
    daemon.shutdown();

    Ok(())
}