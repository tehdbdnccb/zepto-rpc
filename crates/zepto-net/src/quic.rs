//! Sub-millisecond direct TPU QUIC client for Solana transaction submission.

use std::net::SocketAddr;
use std::sync::Arc;
use quinn::{ClientConfig, Endpoint, Connection};
use zepto_core::error::{Result, ZeptoError};

/// Network configuration settings for low-latency QUIC connection pools.
#[derive(Debug, Clone)]
pub struct QuicConfig {
    /// Local socket binding address for outbound QUIC streams.
    pub bind_addr: SocketAddr,
    /// Connection handshake timeout in milliseconds.
    pub timeout_ms: u64,
    /// Maximum concurrent stream channels per active leader connection.
    pub max_concurrent_streams: u32,
}

impl Default for QuicConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:0".parse().unwrap(),
            timeout_ms: 50,
            max_concurrent_streams: 128,
        }
    }
}

/// Ultra-low latency QUIC transport manager targeting direct validator TPU QUIC ports.
pub struct DirectTpuQuicClient {
    endpoint: Endpoint,
}

impl DirectTpuQuicClient {
    /// Initializes a new high-throughput QUIC endpoint instance.
    pub fn new(config: QuicConfig) -> Result<Self> {
        let mut endpoint = Endpoint::client(config.bind_addr)
            .map_err(|_| ZeptoError::AlignmentFault)?;

        // Disable standard TLS verification overhead for local testbeds/direct TPU IP routes
        let client_config = ClientConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![], rustls::pki_types::PrivateKeyDer::Pkcs8(vec![].into()))
            .map_err(|_| ZeptoError::AlignmentFault)?;

        endpoint.set_default_client_config(client_config);

        Ok(Self { endpoint })
    }

    /// Asynchronously establishes a QUIC connection stream to target leader socket address.
    pub async fn connect_leader(&self, tpu_addr: SocketAddr) -> Result<Connection> {
        let connection = self
            .endpoint
            .connect(tpu_addr, "solana-tpu")
            .map_err(|_| ZeptoError::AlignmentFault)?
            .await
            .map_err(|_| ZeptoError::BoundsViolation)?;

        Ok(connection)
    }

    /// Transmits a raw transaction payload directly over an open uni-directional QUIC stream.
    #[inline(always)]
    pub async fn send_transaction(&self, connection: &Connection, payload: &[u8]) -> Result<()> {
        if payload.is_empty() || payload.len() > 1232 {
            return Err(ZeptoError::InvalidFrameSize);
        }

        let mut send_stream = connection
            .open_uni()
            .await
            .map_err(|_| ZeptoError::RingBufferFull)?;

        send_stream
            .write_all(payload)
            .await
            .map_err(|_| ZeptoError::BoundsViolation)?;

        send_stream
            .finish()
            .map_err(|_| ZeptoError::BoundsViolation)?;

        Ok(())
    }
}