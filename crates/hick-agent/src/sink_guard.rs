//! API sink guard for validating external API calls from agent scripts.
//!
//! Provides a [`SinkGuard`] that wraps capability token validation and sink
//! definitions to gate outbound API calls made by agent-executed scripts.
//! Scripts use the `hickory` SDK's `hick.api_call()` function, which routes
//! through this guard before the actual call is made.

use std::collections::HashMap;
use std::sync::Mutex;

use hick_classify::Classification;
use hick_sink::{SinkDef, SinkError, SinkValidator};
use hick_token::ContainerCapabilities;

/// Tracks API sink call counts and validates calls against capabilities.
pub struct SinkGuard {
    capabilities: ContainerCapabilities,
    sink_defs: HashMap<String, SinkDef>,
    call_counts: Mutex<HashMap<String, u64>>,
}

impl SinkGuard {
    /// Create a new sink guard with the given capabilities and sink definitions.
    pub fn new(
        capabilities: ContainerCapabilities,
        sink_defs: Vec<SinkDef>,
    ) -> Self {
        let defs_map = sink_defs
            .into_iter()
            .map(|d| (d.sink_type.clone(), d))
            .collect();
        Self {
            capabilities,
            sink_defs: defs_map,
            call_counts: Mutex::new(HashMap::new()),
        }
    }

    /// Validate an API sink call. Returns `Ok(())` if allowed, or a
    /// [`SinkError`] describing why the call was denied.
    pub fn validate_call(
        &self,
        sink_type: &str,
        slot_classifications: &HashMap<String, Classification>,
        recipient: Option<&str>,
    ) -> Result<(), SinkError> {
        let sink_def = self
            .sink_defs
            .get(sink_type)
            .ok_or_else(|| SinkError::SinkNotAllowed(sink_type.to_string()))?;

        let current_count = {
            let counts = self.call_counts.lock().unwrap();
            *counts.get(sink_type).unwrap_or(&0)
        };

        let now = chrono_now();
        SinkValidator::validate_call(
            &self.capabilities,
            sink_def,
            slot_classifications,
            current_count,
            &now,
            recipient,
        )?;

        // Increment call count on success.
        {
            let mut counts = self.call_counts.lock().unwrap();
            *counts.entry(sink_type.to_string()).or_insert(0) += 1;
        }

        Ok(())
    }

    /// Get the current call count for a sink type.
    pub fn call_count(&self, sink_type: &str) -> u64 {
        let counts = self.call_counts.lock().unwrap();
        *counts.get(sink_type).unwrap_or(&0)
    }

    /// Register a sink definition.
    pub fn register_sink(&mut self, sink_def: SinkDef) {
        self.sink_defs.insert(sink_def.sink_type.clone(), sink_def);
    }
}

fn chrono_now() -> String {
    // Use a simple UTC timestamp. In production, use chrono::Utc::now().
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Approximate ISO 8601 — sufficient for string comparison in ExpiresRule.
    let (year, rem) = (1970 + secs / 31557600, secs % 31557600);
    let (month, rem) = (1 + rem / 2629800, rem % 2629800);
    let (day, rem) = (1 + rem / 86400, rem % 86400);
    let (hour, rem) = (rem / 3600, rem % 3600);
    let (min, sec) = (rem / 60, rem % 60);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_classify::Classification;
    use hick_sink::SinkSlot;
    use serde_json::json;

    fn test_email_sink() -> SinkDef {
        SinkDef {
            sink_type: "email".to_string(),
            slots: vec![SinkSlot {
                name: "body".to_string(),
                accepted_classifications: vec![],
                rejected_classifications: vec![Classification::new(vec![
                    json!({"sensitivity": "secret"}),
                ])],
            }],
            required_capabilities: vec!["api-sink = email".to_string()],
        }
    }

    #[test]
    fn guard_allows_valid_call() {
        let caps = ContainerCapabilities::new()
            .allow_api_sink("email")
            .with_expires("2027-01-01T00:00:00Z");

        let guard = SinkGuard::new(caps, vec![test_email_sink()]);
        let slot_cls = HashMap::from([(
            "body".to_string(),
            Classification::new(vec![json!({"sensitivity": "low"})]),
        )]);

        assert!(guard.validate_call("email", &slot_cls, None).is_ok());
        assert_eq!(guard.call_count("email"), 1);
    }

    #[test]
    fn guard_blocks_rejected_classification() {
        let caps = ContainerCapabilities::new()
            .allow_api_sink("email")
            .with_expires("2027-01-01T00:00:00Z");

        let guard = SinkGuard::new(caps, vec![test_email_sink()]);
        let slot_cls = HashMap::from([(
            "body".to_string(),
            Classification::new(vec![json!({"sensitivity": "secret"})]),
        )]);

        assert!(guard.validate_call("email", &slot_cls, None).is_err());
        assert_eq!(guard.call_count("email"), 0);
    }

    #[test]
    fn guard_blocks_unregistered_sink() {
        let caps = ContainerCapabilities::new();
        let guard = SinkGuard::new(caps, vec![]);

        assert!(guard
            .validate_call("unknown", &HashMap::new(), None)
            .is_err());
    }

    #[test]
    fn guard_tracks_call_counts() {
        let caps = ContainerCapabilities::new()
            .allow_api_sink("email")
            .with_max_calls("email", 2)
            .with_expires("2027-01-01T00:00:00Z");

        let guard = SinkGuard::new(caps, vec![test_email_sink()]);
        let empty = HashMap::new();

        assert!(guard.validate_call("email", &empty, None).is_ok());
        assert!(guard.validate_call("email", &empty, None).is_ok());
        // Third call exceeds max_calls=2
        assert!(guard.validate_call("email", &empty, None).is_err());
    }
}
