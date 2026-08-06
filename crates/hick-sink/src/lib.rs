//! API sink system for data flow control.
//!
//! Provides typed API sink definitions with slot-based classification policies,
//! mock backends for testing, and validation against capability tokens.

use std::collections::HashMap;
use std::sync::Mutex;

use hick_classify::Classification;
use hick_token::ContainerCapabilities;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum SinkError {
    #[error("sink type not allowed by token: {0}")]
    SinkNotAllowed(String),

    #[error("classification rejected by slot '{slot}': {reason}")]
    ClassificationRejected { slot: String, reason: String },

    #[error("max calls exceeded for sink '{sink}': limit {limit}, current {current}")]
    MaxCallsExceeded {
        sink: String,
        limit: u64,
        current: u64,
    },

    #[error("token expired")]
    Expired,

    #[error("recipient not allowed: {0}")]
    RecipientNotAllowed(String),

    #[error("backend error: {0}")]
    Backend(String),
}

// ---------------------------------------------------------------------------
// Sink definitions
// ---------------------------------------------------------------------------

/// A typed slot in an API sink, with accept/reject classification policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkSlot {
    pub name: String,
    pub accepted_classifications: Vec<Classification>,
    pub rejected_classifications: Vec<Classification>,
}

/// An API sink definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkDef {
    pub sink_type: String,
    pub slots: Vec<SinkSlot>,
    pub required_capabilities: Vec<String>,
}

// ---------------------------------------------------------------------------
// API call records
// ---------------------------------------------------------------------------

/// A recorded API call (from mock or live execution).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCallRecord {
    pub sink_type: String,
    pub slot_values: HashMap<String, serde_json::Value>,
    pub token_caveats: Vec<String>,
    pub classification_checks: Vec<String>,
    pub timestamp: String,
}

// ---------------------------------------------------------------------------
// Sink backend trait
// ---------------------------------------------------------------------------

/// Backend for executing API sink calls.
#[async_trait::async_trait]
pub trait SinkBackend: Send + Sync {
    async fn execute(
        &self,
        sink_type: &str,
        slot_values: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, SinkError>;
}

// ---------------------------------------------------------------------------
// Mock backend
// ---------------------------------------------------------------------------

/// Mock sink backend that records calls for verification.
pub struct MockSinkBackend {
    calls: Mutex<Vec<ApiCallRecord>>,
}

impl MockSinkBackend {
    pub fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
        }
    }

    pub fn calls(&self) -> Vec<ApiCallRecord> {
        self.calls.lock().unwrap().clone()
    }

    pub fn call_count(&self, sink_type: &str) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.sink_type == sink_type)
            .count()
    }
}

impl Default for MockSinkBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SinkBackend for MockSinkBackend {
    async fn execute(
        &self,
        sink_type: &str,
        slot_values: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, SinkError> {
        let record = ApiCallRecord {
            sink_type: sink_type.to_string(),
            slot_values: slot_values.clone(),
            token_caveats: Vec::new(),
            classification_checks: Vec::new(),
            timestamp: chrono_now_stub(),
        };
        self.calls.lock().unwrap().push(record);
        Ok(serde_json::json!({"status": "mock_ok"}))
    }
}

fn chrono_now_stub() -> String {
    "2026-02-17T00:00:00Z".to_string()
}

// ---------------------------------------------------------------------------
// Sink validator
// ---------------------------------------------------------------------------

/// Validates API sink calls against token capabilities and sink definitions.
pub struct SinkValidator;

impl SinkValidator {
    /// Validate a sink call against the token and sink definition.
    pub fn validate_call(
        capabilities: &ContainerCapabilities,
        sink_def: &SinkDef,
        slot_data_classifications: &HashMap<String, Classification>,
        current_call_count: u64,
        now: &str,
        recipient: Option<&str>,
    ) -> Result<(), SinkError> {
        // Check API sink permission
        if !capabilities.check_api_sink(&sink_def.sink_type) {
            return Err(SinkError::SinkNotAllowed(sink_def.sink_type.clone()));
        }

        // Check expiration
        if !capabilities.check_expires(now) {
            return Err(SinkError::Expired);
        }

        // Check max calls
        if !capabilities.check_max_calls(&sink_def.sink_type, current_call_count) {
            return Err(SinkError::MaxCallsExceeded {
                sink: sink_def.sink_type.clone(),
                limit: capabilities
                    .max_calls_rules
                    .iter()
                    .find(|r| r.sink_type == sink_def.sink_type)
                    .map(|r| r.max_calls)
                    .unwrap_or(0),
                current: current_call_count,
            });
        }

        // Check recipient constraints
        if let Some(recip) = recipient
            && !capabilities.check_recipient(&sink_def.sink_type, recip)
        {
            return Err(SinkError::RecipientNotAllowed(recip.to_string()));
        }

        // Check slot classification policies
        for slot in &sink_def.slots {
            if let Some(data_cls) = slot_data_classifications.get(&slot.name) {
                // Check rejected classifications
                for rejected in &slot.rejected_classifications {
                    for label in rejected.labels() {
                        if data_cls.labels().contains(label) {
                            return Err(SinkError::ClassificationRejected {
                                slot: slot.name.clone(),
                                reason: format!("data has rejected label: {}", label),
                            });
                        }
                    }
                }
                // If accepted list is non-empty, check data has at least one accepted label
                if !slot.accepted_classifications.is_empty() {
                    let has_accepted = slot.accepted_classifications.iter().any(|accepted| {
                        accepted
                            .labels()
                            .iter()
                            .any(|label| data_cls.labels().contains(label))
                    });
                    if !has_accepted {
                        return Err(SinkError::ClassificationRejected {
                            slot: slot.name.clone(),
                            reason: "data has no accepted classification labels".to_string(),
                        });
                    }
                }
            } else if !slot.accepted_classifications.is_empty() {
                // Slot requires classification data but caller omitted it — reject.
                // Slots with only rejected_classifications can pass without data
                // (absence of data can't contain a rejected label).
                return Err(SinkError::ClassificationRejected {
                    slot: slot.name.clone(),
                    reason: "slot requires classification data but none was provided".to_string(),
                });
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_sink_def() -> SinkDef {
        SinkDef {
            sink_type: "email".to_string(),
            slots: vec![
                SinkSlot {
                    name: "to".to_string(),
                    accepted_classifications: vec![Classification::new(vec![
                        json!({"trust": "internal"}),
                    ])],
                    rejected_classifications: vec![],
                },
                SinkSlot {
                    name: "body".to_string(),
                    accepted_classifications: vec![],
                    rejected_classifications: vec![Classification::new(vec![
                        json!({"sensitivity": "secret"}),
                    ])],
                },
            ],
            required_capabilities: vec!["api-sink = email".to_string()],
        }
    }

    #[tokio::test]
    async fn mock_records_calls() {
        let backend = MockSinkBackend::new();
        let mut values = HashMap::new();
        values.insert("to".to_string(), json!("alice@co.com"));

        let result = backend.execute("email", &values).await.unwrap();
        assert_eq!(result["status"], "mock_ok");
        assert_eq!(backend.call_count("email"), 1);
    }

    #[test]
    fn validator_accepts_matching_classifications() {
        let caps = ContainerCapabilities::new()
            .allow_api_sink("email")
            .with_expires("2027-01-01T00:00:00Z");

        let sink_def = test_sink_def();
        let mut slot_cls = HashMap::new();
        slot_cls.insert(
            "to".to_string(),
            Classification::new(vec![json!({"trust": "internal"})]),
        );
        slot_cls.insert(
            "body".to_string(),
            Classification::new(vec![json!({"sensitivity": "low"})]),
        );

        let result = SinkValidator::validate_call(
            &caps,
            &sink_def,
            &slot_cls,
            0,
            "2026-06-01T00:00:00Z",
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn validator_rejects_secret_data_in_body_slot() {
        let caps = ContainerCapabilities::new()
            .allow_api_sink("email")
            .with_expires("2027-01-01T00:00:00Z");

        let sink_def = test_sink_def();
        let mut slot_cls = HashMap::new();
        slot_cls.insert(
            "body".to_string(),
            Classification::new(vec![json!({"sensitivity": "secret"})]),
        );

        let result = SinkValidator::validate_call(
            &caps,
            &sink_def,
            &slot_cls,
            0,
            "2026-06-01T00:00:00Z",
            None,
        );
        assert!(matches!(
            result,
            Err(SinkError::ClassificationRejected { .. })
        ));
    }

    #[test]
    fn validator_max_calls_enforcement() {
        let caps = ContainerCapabilities::new()
            .allow_api_sink("email")
            .with_max_calls("email", 1)
            .with_expires("2027-01-01T00:00:00Z");

        let sink_def = test_sink_def();
        let mut slot_cls = HashMap::new();
        slot_cls.insert(
            "to".to_string(),
            Classification::new(vec![json!({"trust": "internal"})]),
        );

        // First call ok
        assert!(
            SinkValidator::validate_call(
                &caps,
                &sink_def,
                &slot_cls,
                0,
                "2026-06-01T00:00:00Z",
                None,
            )
            .is_ok()
        );

        // Second call rejected
        assert!(matches!(
            SinkValidator::validate_call(
                &caps,
                &sink_def,
                &slot_cls,
                1,
                "2026-06-01T00:00:00Z",
                None,
            ),
            Err(SinkError::MaxCallsExceeded { .. })
        ));
    }

    #[test]
    fn validator_expiration() {
        let caps = ContainerCapabilities::new()
            .allow_api_sink("email")
            .with_expires("2026-03-01T00:00:00Z");

        let sink_def = test_sink_def();
        let mut slot_cls = HashMap::new();
        slot_cls.insert(
            "to".to_string(),
            Classification::new(vec![json!({"trust": "internal"})]),
        );

        // Before expiry
        assert!(
            SinkValidator::validate_call(
                &caps,
                &sink_def,
                &slot_cls,
                0,
                "2026-02-15T00:00:00Z",
                None,
            )
            .is_ok()
        );

        // After expiry
        assert!(matches!(
            SinkValidator::validate_call(
                &caps,
                &sink_def,
                &slot_cls,
                0,
                "2026-04-01T00:00:00Z",
                None,
            ),
            Err(SinkError::Expired)
        ));
    }

    #[test]
    fn validator_recipient_constraint() {
        let caps = ContainerCapabilities::new()
            .allow_api_sink("email")
            .with_recipient("email", "alice@co.com")
            .with_expires("2027-01-01T00:00:00Z");

        let sink_def = test_sink_def();
        let mut slot_cls = HashMap::new();
        slot_cls.insert(
            "to".to_string(),
            Classification::new(vec![json!({"trust": "internal"})]),
        );

        // Correct recipient
        assert!(
            SinkValidator::validate_call(
                &caps,
                &sink_def,
                &slot_cls,
                0,
                "2026-06-01T00:00:00Z",
                Some("alice@co.com"),
            )
            .is_ok()
        );

        // Wrong recipient
        assert!(matches!(
            SinkValidator::validate_call(
                &caps,
                &sink_def,
                &slot_cls,
                0,
                "2026-06-01T00:00:00Z",
                Some("bob@co.com"),
            ),
            Err(SinkError::RecipientNotAllowed(_))
        ));
    }

    #[test]
    fn validator_sink_not_allowed() {
        let caps = ContainerCapabilities::new(); // no api-sink rules

        let sink_def = test_sink_def();
        let slot_cls = HashMap::new();

        assert!(matches!(
            SinkValidator::validate_call(
                &caps,
                &sink_def,
                &slot_cls,
                0,
                "2026-06-01T00:00:00Z",
                None,
            ),
            Err(SinkError::SinkNotAllowed(_))
        ));
    }

    #[test]
    fn validator_rejects_missing_slot_data_when_accepted_required() {
        let caps = ContainerCapabilities::new()
            .allow_api_sink("email")
            .with_expires("2027-01-01T00:00:00Z");

        let sink_def = test_sink_def();
        // Omit the "to" slot which has accepted_classifications
        let slot_cls = HashMap::new();

        let result = SinkValidator::validate_call(
            &caps,
            &sink_def,
            &slot_cls,
            0,
            "2026-06-01T00:00:00Z",
            None,
        );
        assert!(matches!(
            result,
            Err(SinkError::ClassificationRejected { .. })
        ));
    }

    #[test]
    fn validator_allows_missing_slot_data_when_only_rejected() {
        let caps = ContainerCapabilities::new()
            .allow_api_sink("email")
            .with_expires("2027-01-01T00:00:00Z");

        // Sink with only rejected_classifications (no accepted list)
        let sink_def = SinkDef {
            sink_type: "email".to_string(),
            slots: vec![SinkSlot {
                name: "body".to_string(),
                accepted_classifications: vec![],
                rejected_classifications: vec![Classification::new(vec![
                    json!({"sensitivity": "secret"}),
                ])],
            }],
            required_capabilities: vec!["api-sink = email".to_string()],
        };

        // Omit "body" slot data — should pass since only rejected list exists
        let slot_cls = HashMap::new();

        let result = SinkValidator::validate_call(
            &caps,
            &sink_def,
            &slot_cls,
            0,
            "2026-06-01T00:00:00Z",
            None,
        );
        assert!(result.is_ok());
    }
}
