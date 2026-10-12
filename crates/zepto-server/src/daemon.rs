//! System thread pinning, CPU core affinity pinning, and lifecycle supervisor.

use core_affinity::CoreId;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tracing::{error, info, warn};
use zepto_core::error::{Result, ZeptoError};

/// Multi-threaded runtime execution lifecycle controller.
pub struct DaemonSupervisor {
    running: Arc<AtomicBool>,
    handles: Vec<JoinHandle<()>>,
}

impl DaemonSupervisor {
    /// Instantiates a new `DaemonSupervisor`.
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
            handles: Vec::new(),
        }
    }

    /// Returns an atomic reference to the system execution flag.
    pub fn is_running_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.running)
    }

    /// Spawns a dedicated worker thread pinned directly to a specified CPU core.
    pub fn spawn_pinned_worker<F>(&mut self, name: &'static str, core_id: usize, worker_fn: F)
    where
        F: FnOnce(Arc<AtomicBool>) + Send + 'static,
    {
        let running = Arc::clone(&self.running);

        let handle = thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let core = CoreId { id: core_id };
                if core_affinity::set_for_current(core) {
                    info!(target: "zepto_daemon", "Successfully pinned worker '{}' to CPU core {}", name, core_id);
                } else {
                    warn!(target: "zepto_daemon", "Failed to pin worker '{}' to CPU core {}", name, core_id);
                }

                worker_fn(running);
            })
            .expect("Failed to spawn pinned worker thread");

        self.handles.push(handle);
    }

    /// Signals all managed threads to stop and joins worker handles.
    pub fn shutdown(mut self) {
        info!(target: "zepto_daemon", "Initiating graceful daemon worker shutdown");
        self.running.store(false, Ordering::Release);

        for handle in self.handles.drain(..) {
            if let Err(e) = handle.join() {
                error!(target: "zepto_daemon", "Error joining worker thread: {:?}", e);
            }
        }
        info!(target: "zepto_daemon", "Daemon shutdown complete");
    }
}

impl Default for DaemonSupervisor {
    fn default() -> Self {
        Self::new()
    }
}