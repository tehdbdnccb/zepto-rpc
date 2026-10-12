//! # ZeptoRPC Network Protocol & TPU Multiplexer Engine
//!
//! Sub-millisecond transaction dispatch and TPU leader forwarding infrastructure:
//! - Dynamic TPU leader schedule cache and slot-indexed lookups
//! - Sub-millisecond QUIC client optimized for direct Solana leader TPU sockets
//! - Multi-path UDP packet multiplexer for speculative multi-leader fan-out

#![deny(unreachable_pub)]
#![deny(missing_docs)]

pub mod leader_schedule;
pub mod quic;
pub mod udp;

pub use leader_schedule::{LeaderScheduleTable, LeaderTarget};
pub use quic::{DirectTpuQuicClient, QuicConfig};
pub use udp::{MultiPathUdpMultiplexer, UdpConfig};