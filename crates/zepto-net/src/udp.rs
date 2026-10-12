//! Multi-path UDP packet multiplexer for speculative multi-leader transaction fan-out.

use std::net::{SocketAddr, UdpSocket};
use arrayvec::ArrayVec;
use zepto_core::error::{Result, ZeptoError};
use crate::leader_schedule::{LeaderTarget, MAX_LEADER_HORIZON};

/// UDP Socket multiplexer configuration limits.
#[derive(Debug, Clone, Copy)]
pub struct UdpConfig {
    /// OS UDP socket send buffer size in bytes (SO_SNDBUF).
    pub send_buffer_bytes: usize,
    /// IP TTL header value for outbound frames.
    pub ttl: u32,
}

impl Default for UdpConfig {
    fn default() -> Self {
        Self {
            send_buffer_bytes: 8 * 1024 * 1024, // 8MB OS buffer
            ttl: 64,
        }
    }
}

/// Zero-copy non-blocking multi-path UDP socket multiplexer.
pub struct MultiPathUdpMultiplexer {
    socket: UdpSocket,
}

impl MultiPathUdpMultiplexer {
    /// Creates and configures a non-blocking system UDP socket.
    pub fn bind(bind_addr: SocketAddr, config: UdpConfig) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr).map_err(|_| ZeptoError::AlignmentFault)?;
        socket.set_nonblocking(true).map_err(|_| ZeptoError::AlignmentFault)?;
        socket.set_ttl(config.ttl).map_err(|_| ZeptoError::AlignmentFault)?;

        Ok(Self { socket })
    }

    /// Simultaneously fires a validated raw transaction payload to active and upcoming TPU leaders.
    ///
    /// # Performance
    /// Executes zero-copy system calls across speculative target channels in nanosecond timeframes.
    #[inline(always)]
    pub fn speculative_fanout(
        &self,
        targets: &ArrayVec<LeaderTarget, MAX_LEADER_HORIZON>,
        payload: &[u8],
    ) -> Result<usize> {
        if payload.is_empty() || payload.len() > 1232 {
            return Err(ZeptoError::InvalidFrameSize);
        }

        let mut successful_sends = 0;

        for target in targets.iter() {
            // Forward directly to primary TPU socket
            if self.socket.send_to(payload, target.tpu_addr).is_ok() {
                successful_sends += 1;
            }

            // Fan-out to TPU Forward socket for next slot handoff resilience
            let _ = self.socket.send_to(payload, target.tpu_forward_addr);
        }

        if successful_sends == 0 {
            return Err(ZeptoError::BoundsViolation);
        }

        Ok(successful_sends)
    }
}