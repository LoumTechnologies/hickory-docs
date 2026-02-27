//! Capability type definitions and handler trait.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Namespace URI for the capabilities sub-app.
pub const CAPABILITY_NS: &str = "https://drift-signal.dev/ns/capability#";

/// A device capability that can be registered and invoked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDeviceCapability {
    pub id: String,
    pub device_id: String,
    pub cap_type: String,
    pub description: String,
}

/// A capability invocation received from a remote device, queued for local execution.
#[derive(Debug, Clone)]
pub struct PendingInvocation {
    pub id: String,
    pub cap_type: String,
    pub params: serde_json::Value,
}

/// Shared queue for pending remote invocations.
///
/// The `TagHandler` (sync) pushes invocations here; the async dispatcher drains them.
pub type PendingQueue = Arc<Mutex<VecDeque<PendingInvocation>>>;

/// Create a new empty pending queue.
pub fn new_pending_queue() -> PendingQueue {
    Arc::new(Mutex::new(VecDeque::new()))
}

/// Handler for executing a capability locally on a device.
#[async_trait]
pub trait AppCapabilityHandler: Send + Sync {
    /// The capability type identifier (e.g., "imessage", "filesystem", "desktop_automation").
    fn cap_type(&self) -> &str;

    /// Human-readable description of this capability.
    fn description(&self) -> &str {
        ""
    }

    /// Execute the capability with the given parameters.
    async fn execute(&self, params: serde_json::Value) -> anyhow::Result<serde_json::Value>;

    /// Whether the capability is currently available on this device.
    fn is_available(&self) -> bool;
}
