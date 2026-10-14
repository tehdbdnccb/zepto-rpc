//! Sub-millisecond direct TPU QUIC client for Solana transaction submission.

use std::net::SocketAddr;
use std::sync::Arc;
use quinn::crypto::rustls::QuicClientConfig;
use quinn::{ClientConfig, Endpoint, Connection};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use zepto_core::error::{Result, ZeptoError};

/// Certificate verifier that accepts any server certificate.
///
/// Solana validators present self-signed certificates on their TPU QUIC ports,
/// so standard chain validation cannot succeed for direct TPU connections.
#[derive(Debug)]
struct SkipServerVerification;

impl ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

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
        let rustls_config = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .map_err(|_| ZeptoError::AlignmentFault)?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
        .with_no_client_auth();

        let client_config = ClientConfig::new(Arc::new(
            QuicClientConfig::try_from(rustls_config).map_err(|_| ZeptoError::AlignmentFault)?,
        ));

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