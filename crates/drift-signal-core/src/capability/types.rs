//! Capability type definitions and handler trait.

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

/// Handler for executing a capability locally on a device.
#[async_trait]
pub trait AppCapabilityHandler: Send + Sync {
    /// The capability type identifier (e.g., "imessage", "filesystem", "desktop_automation").
    fn cap_type(&self) -> &str;

    /// Execute the capability with the given parameters.
    async fn execute(&self, params: serde_json::Value) -> anyhow::Result<serde_json::Value>;

    /// Whether the capability is currently available on this device.
    fn is_available(&self) -> bool;
}
