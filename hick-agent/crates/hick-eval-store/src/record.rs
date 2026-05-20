use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A single eval result written by the agent after completing a query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResultRecord {
    pub machine_id: String,
    pub timestamp: DateTime<Utc>,
    pub config_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    /// Named metric scores (e.g. `{"accuracy": 0.9, "latency_ms": 420.0}`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scores: Option<HashMap<String, f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_cost: Option<TokenCost>,
}

/// A pruning decision appended on top of existing records.
///
/// Lines are never deleted — this record marks that older entries should be
/// excluded from aggregation by readers that honour prune semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruneRecord {
    pub machine_id: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Exclude records with `timestamp < pruned_before` when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pruned_before: Option<DateTime<Utc>>,
}

/// LLM token usage for a single eval turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCost {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cost_usd: Option<f64>,
}

/// A parsed NDJSON record from the eval store.
///
/// The `Unknown` variant preserves records whose `record_type` is not recognised by
/// this version of the crate, so that older readers skip new types without data loss.
/// TASK-45 and TASK-50 may add new variants here; existing `Unknown` entries will then
/// be re-parsed on the next schema update.
#[derive(Debug, Clone)]
pub enum EvalRecord {
    EvalResult(EvalResultRecord),
    Prune(PruneRecord),
    /// Any `record_type` not known to this version. Preserved verbatim on re-serialise.
    Unknown(Value),
}

impl EvalRecord {
    pub fn record_type(&self) -> &str {
        match self {
            EvalRecord::EvalResult(_) => "eval_result",
            EvalRecord::Prune(_) => "prune",
            EvalRecord::Unknown(v) => v
                .get("record_type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown"),
        }
    }
}

// ── Serialisation helpers ─────────────────────────────────────────────────────

/// Injects `"record_type"` at the top level without storing it in each inner struct.
#[derive(Serialize)]
struct Tagged<T: Serialize> {
    record_type: &'static str,
    #[serde(flatten)]
    inner: T,
}

impl Serialize for EvalRecord {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            EvalRecord::EvalResult(r) => Tagged { record_type: "eval_result", inner: r }.serialize(s),
            EvalRecord::Prune(r) => Tagged { record_type: "prune", inner: r }.serialize(s),
            EvalRecord::Unknown(v) => v.serialize(s),
        }
    }
}

impl<'de> Deserialize<'de> for EvalRecord {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        let record_type = v.get("record_type").and_then(|t| t.as_str()).unwrap_or("");
        Ok(match record_type {
            "eval_result" => EvalRecord::EvalResult(
                serde_json::from_value(v).map_err(serde::de::Error::custom)?,
            ),
            "prune" => EvalRecord::Prune(
                serde_json::from_value(v).map_err(serde::de::Error::custom)?,
            ),
            _ => EvalRecord::Unknown(v),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn eval_result(machine_id: &str) -> EvalRecord {
        EvalRecord::EvalResult(EvalResultRecord {
            machine_id: machine_id.to_string(),
            timestamp: Utc::now(),
            config_name: "test-config".to_string(),
            query: Some("what is 2+2?".to_string()),
            response: Some("4".to_string()),
            scores: None,
            token_cost: None,
        })
    }

    #[test]
    fn eval_result_round_trip() {
        let r = eval_result("m1");
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains(r#""record_type":"eval_result""#));
        let back: EvalRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.record_type(), "eval_result");
        if let EvalRecord::EvalResult(inner) = back {
            assert_eq!(inner.machine_id, "m1");
            assert_eq!(inner.config_name, "test-config");
        } else {
            panic!("expected EvalResult variant");
        }
    }

    #[test]
    fn prune_round_trip() {
        let r = EvalRecord::Prune(PruneRecord {
            machine_id: "m1".to_string(),
            timestamp: Utc::now(),
            reason: Some("old data".to_string()),
            pruned_before: None,
        });
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains(r#""record_type":"prune""#));
        let back: EvalRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.record_type(), "prune");
    }

    #[test]
    fn unknown_record_type_preserved_as_unknown_variant() {
        let json = r#"{"record_type":"future_v2_type","extra_field":99}"#;
        let r: EvalRecord = serde_json::from_str(json).unwrap();
        assert_eq!(r.record_type(), "future_v2_type");
        assert!(matches!(r, EvalRecord::Unknown(_)));
        // Re-serialising preserves the original data.
        let roundtrip = serde_json::to_string(&r).unwrap();
        assert!(roundtrip.contains("future_v2_type"));
        assert!(roundtrip.contains("extra_field"));
    }

    #[test]
    fn pruning_appends_not_deletes() {
        // Simulate writing two records to a string buffer then reading them back.
        // The original eval_result line must remain intact alongside the prune line.
        let eval = eval_result("m1");
        let prune = EvalRecord::Prune(PruneRecord {
            machine_id: "m1".to_string(),
            timestamp: Utc::now(),
            reason: Some("cleanup".to_string()),
            pruned_before: None,
        });
        let mut buf = String::new();
        buf.push_str(&serde_json::to_string(&eval).unwrap());
        buf.push('\n');
        buf.push_str(&serde_json::to_string(&prune).unwrap());
        buf.push('\n');

        let records: Vec<EvalRecord> = buf
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].record_type(), "eval_result");
        assert_eq!(records[1].record_type(), "prune");
    }

    #[test]
    fn scores_serialise_as_flat_object() {
        let mut scores = HashMap::new();
        scores.insert("accuracy".to_string(), 0.95);
        scores.insert("latency_ms".to_string(), 312.0);
        let r = EvalRecord::EvalResult(EvalResultRecord {
            machine_id: "m".to_string(),
            timestamp: Utc::now(),
            config_name: "c".to_string(),
            query: None,
            response: None,
            scores: Some(scores),
            token_cost: None,
        });
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("accuracy"));
        assert!(json.contains("0.95"));
    }

    #[test]
    fn token_cost_optional_fields_omitted_when_none() {
        let r = EvalRecord::EvalResult(EvalResultRecord {
            machine_id: "m".to_string(),
            timestamp: Utc::now(),
            config_name: "c".to_string(),
            query: None,
            response: None,
            scores: None,
            token_cost: Some(TokenCost {
                input_tokens: Some(100),
                output_tokens: None,
                total_cost_usd: None,
            }),
        });
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("input_tokens"));
        assert!(!json.contains("output_tokens"));
        assert!(!json.contains("total_cost_usd"));
    }
}
