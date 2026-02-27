//! Cross-device capability dispatch.
//!
//! Routes capability invocations to the appropriate device. When the
//! target capability is local, executes immediately. When remote,
//! emits a `CapabilityInvoked` event for waymark gossip delivery.
//!
//! Remote invocations arriving via sync are queued in the shared
//! `PendingQueue` by the `CapabilityInvokedHandler` tag handler.
//! Call `process_pending()` to drain and execute them.

use std::collections::BTreeMap;

use waymark_sdk::SyncApp;

use super::types::{AppCapabilityHandler, AppDeviceCapability, PendingQueue};

/// Routes capability invocations to local handlers or remote devices.
pub struct AppCapabilityDispatcher {
    local_handlers: Vec<Box<dyn AppCapabilityHandler>>,
    device_id: String,
    pending: PendingQueue,
}

impl AppCapabilityDispatcher {
    pub fn new(device_id: String, pending: PendingQueue) -> Self {
        Self {
            local_handlers: Vec::new(),
            device_id,
            pending,
        }
    }

    pub fn register(&mut self, handler: Box<dyn AppCapabilityHandler>) {
        self.local_handlers.push(handler);
    }

    /// List locally available capabilities.
    pub fn local_capabilities(&self) -> Vec<AppDeviceCapability> {
        self.local_handlers
            .iter()
            .filter(|h| h.is_available())
            .map(|h| AppDeviceCapability {
                id: format!("cap-{}-{}", self.device_id, h.cap_type()),
                device_id: self.device_id.clone(),
                cap_type: h.cap_type().to_string(),
                description: h.description().to_string(),
            })
            .collect()
    }

    /// Try to execute a capability locally. Returns `None` if no local handler
    /// exists for the requested type.
    pub async fn execute_local(
        &self,
        cap_type: &str,
        params: serde_json::Value,
    ) -> Option<anyhow::Result<serde_json::Value>> {
        let handler = self
            .local_handlers
            .iter()
            .find(|h| h.cap_type() == cap_type && h.is_available())?;
        Some(handler.execute(params).await)
    }

    /// Invoke a capability, dispatching locally if available or emitting a
    /// `CapabilityInvoked` event for remote execution.
    pub async fn invoke(
        &self,
        sync_app: &mut SyncApp,
        cap_type: &str,
        target_device: Option<&str>,
        params: serde_json::Value,
    ) -> anyhow::Result<String> {
        let invocation_id = format!("inv-{}", uuid::Uuid::now_v7());

        // If targeting this device (or no target and we have a local handler), run locally.
        let run_local = target_device.is_none_or(|d| d == self.device_id);

        if run_local {
            if let Some(result) = self.execute_local(cap_type, params.clone()).await {
                let (status, result_json, error) = match result {
                    Ok(val) => ("success".to_string(), Some(val.to_string()), None),
                    Err(e) => ("error".to_string(), None, Some(e.to_string())),
                };

                // Emit both invocation and result events
                let mut invoke_attrs = BTreeMap::new();
                invoke_attrs.insert("id".into(), invocation_id.clone());
                invoke_attrs.insert("cap_type".into(), cap_type.to_string());
                invoke_attrs.insert("params".into(), params.to_string());
                sync_app.execute_command(
                    "capabilities",
                    "InvokeCapability",
                    &invoke_attrs,
                    None,
                )?;

                let mut result_attrs = BTreeMap::new();
                result_attrs.insert("invocation_id".into(), invocation_id.clone());
                result_attrs.insert("status".into(), status);
                if let Some(r) = result_json {
                    result_attrs.insert("result".into(), r);
                }
                if let Some(e) = error {
                    result_attrs.insert("error".into(), e);
                }
                sync_app.execute_command(
                    "capabilities",
                    "ReportResult",
                    &result_attrs,
                    None,
                )?;

                return Ok(invocation_id);
            }
        }

        // Remote dispatch: emit CapabilityInvoked event for gossip delivery
        let mut attrs = BTreeMap::new();
        attrs.insert("id".into(), invocation_id.clone());
        attrs.insert("cap_type".into(), cap_type.to_string());
        attrs.insert("params".into(), params.to_string());
        if let Some(target) = target_device {
            attrs.insert("target_device".into(), target.to_string());
        }
        sync_app.execute_command("capabilities", "InvokeCapability", &attrs, None)?;

        Ok(invocation_id)
    }

    /// Process pending remote invocations queued by the tag handler.
    ///
    /// Drains the shared pending queue, executes each invocation locally,
    /// and emits `CapabilityResult` events with the outcomes.
    pub async fn process_pending(&self, sync_app: &mut SyncApp) -> anyhow::Result<usize> {
        let pending: Vec<_> = {
            let mut queue = self.pending.lock().unwrap_or_else(|e| e.into_inner());
            queue.drain(..).collect()
        };

        let count = pending.len();
        for inv in pending {
            let (status, result_json, error) =
                if let Some(result) = self.execute_local(&inv.cap_type, inv.params).await {
                    match result {
                        Ok(val) => ("success".to_string(), Some(val.to_string()), None),
                        Err(e) => ("error".to_string(), None, Some(e.to_string())),
                    }
                } else {
                    (
                        "error".to_string(),
                        None,
                        Some(format!(
                            "No local handler for capability type '{}'",
                            inv.cap_type
                        )),
                    )
                };

            let mut attrs = BTreeMap::new();
            attrs.insert("invocation_id".into(), inv.id);
            attrs.insert("status".into(), status);
            if let Some(r) = result_json {
                attrs.insert("result".into(), r);
            }
            if let Some(e) = error {
                attrs.insert("error".into(), e);
            }
            sync_app.execute_command("capabilities", "ReportResult", &attrs, None)?;
        }

        Ok(count)
    }
}
