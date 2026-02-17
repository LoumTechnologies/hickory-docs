//! Python-based policy evaluation for data flow control.
//!
//! Calls a Python subprocess to evaluate data flow policies.
//! The Python function receives JSON input and returns allow/deny decisions.

use std::time::Duration;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("policy denied: {0}")]
    Denied(String),

    #[error("subprocess error: {0}")]
    Subprocess(String),

    #[error("invalid output from policy script: {0}")]
    InvalidOutput(String),

    #[error("policy evaluation timed out after {0:?}")]
    Timeout(Duration),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

// ---------------------------------------------------------------------------
// Policy types
// ---------------------------------------------------------------------------

/// Result of a policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision")]
pub enum PolicyDecision {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "deny")]
    Deny { reason: String },
}

/// Input for a flow policy check.
#[derive(Debug, Serialize)]
pub struct FlowPolicyInput {
    pub column_classifications: serde_json::Value,
    pub sink_type: String,
    pub slot_name: String,
    pub data_classification: serde_json::Value,
    pub token_caveats: Vec<String>,
}

/// Input for a data access policy check.
#[derive(Debug, Serialize)]
pub struct AccessPolicyInput {
    pub column_classifications: serde_json::Value,
    pub requested_columns: Vec<String>,
    pub token_caveats: Vec<String>,
}

// ---------------------------------------------------------------------------
// Policy evaluator
// ---------------------------------------------------------------------------

/// Evaluates policies by calling a Python subprocess.
pub struct PolicyEvaluator {
    pub python_path: String,
    pub script_path: String,
    pub function_name: String,
    pub timeout: Duration,
}

impl PolicyEvaluator {
    pub fn new(script_path: &str) -> Self {
        Self {
            python_path: "python3".to_string(),
            script_path: script_path.to_string(),
            function_name: "evaluate".to_string(),
            timeout: Duration::from_secs(5),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_function(mut self, name: &str) -> Self {
        self.function_name = name.to_string();
        self
    }

    /// Evaluate a data-to-sink flow against the policy.
    pub async fn evaluate_flow(
        &self,
        input: &FlowPolicyInput,
    ) -> Result<PolicyDecision, PolicyError> {
        let json_input = serde_json::to_string(input)?;
        self.run_python(&json_input).await
    }

    /// Evaluate a data access request against the policy.
    pub async fn evaluate_access(
        &self,
        input: &AccessPolicyInput,
    ) -> Result<PolicyDecision, PolicyError> {
        let json_input = serde_json::to_string(input)?;
        self.run_python(&json_input).await
    }

    async fn run_python(&self, json_input: &str) -> Result<PolicyDecision, PolicyError> {
        use tokio::process::Command;

        let script = format!(
            "import json, sys; exec(open('{}').read()); print(json.dumps({}(json.load(sys.stdin))))",
            self.script_path, self.function_name,
        );

        let mut child = Command::new(&self.python_path)
            .arg("-c")
            .arg(&script)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| PolicyError::Subprocess(format!("failed to spawn python: {e}")))?;

        // Write input to stdin
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(json_input.as_bytes())
                .await
                .map_err(|e| PolicyError::Subprocess(format!("stdin write failed: {e}")))?;
            drop(stdin);
        }

        // Wait with timeout
        let output = tokio::time::timeout(self.timeout, child.wait_with_output())
            .await
            .map_err(|_| PolicyError::Timeout(self.timeout))?
            .map_err(|e| PolicyError::Subprocess(format!("wait failed: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PolicyError::Subprocess(format!(
                "python exited with {}: {}",
                output.status, stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let decision: PolicyDecision = serde_json::from_str(stdout.trim())
            .map_err(|e| PolicyError::InvalidOutput(format!("{e}: {stdout}")))?;

        Ok(decision)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_policy_script(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[tokio::test]
    async fn evaluate_with_allow_all_script() {
        let script = write_policy_script(
            r#"
def evaluate(data):
    return {"decision": "allow"}
"#,
        );

        let evaluator = PolicyEvaluator::new(script.path().to_str().unwrap());
        let input = FlowPolicyInput {
            column_classifications: serde_json::json!({}),
            sink_type: "email".to_string(),
            slot_name: "to".to_string(),
            data_classification: serde_json::json!([]),
            token_caveats: vec![],
        };

        let result = evaluator.evaluate_flow(&input).await.unwrap();
        assert_eq!(result, PolicyDecision::Allow);
    }

    #[tokio::test]
    async fn evaluate_with_deny_script() {
        let script = write_policy_script(
            r#"
def evaluate(data):
    return {"decision": "deny", "reason": "policy violation"}
"#,
        );

        let evaluator = PolicyEvaluator::new(script.path().to_str().unwrap());
        let input = FlowPolicyInput {
            column_classifications: serde_json::json!({}),
            sink_type: "email".to_string(),
            slot_name: "to".to_string(),
            data_classification: serde_json::json!([]),
            token_caveats: vec![],
        };

        let result = evaluator.evaluate_flow(&input).await.unwrap();
        assert_eq!(
            result,
            PolicyDecision::Deny {
                reason: "policy violation".to_string()
            }
        );
    }

    #[tokio::test]
    async fn evaluate_malformed_output_errors() {
        let script = write_policy_script(
            r#"
def evaluate(data):
    return "not json object"
"#,
        );

        let evaluator = PolicyEvaluator::new(script.path().to_str().unwrap());
        let input = FlowPolicyInput {
            column_classifications: serde_json::json!({}),
            sink_type: "email".to_string(),
            slot_name: "to".to_string(),
            data_classification: serde_json::json!([]),
            token_caveats: vec![],
        };

        let result = evaluator.evaluate_flow(&input).await;
        assert!(matches!(result, Err(PolicyError::InvalidOutput(_))));
    }

    #[tokio::test]
    async fn evaluate_timeout_errors() {
        let script = write_policy_script(
            r#"
import time
def evaluate(data):
    time.sleep(10)
    return {"decision": "allow"}
"#,
        );

        let evaluator = PolicyEvaluator::new(script.path().to_str().unwrap())
            .with_timeout(Duration::from_millis(100));
        let input = FlowPolicyInput {
            column_classifications: serde_json::json!({}),
            sink_type: "email".to_string(),
            slot_name: "to".to_string(),
            data_classification: serde_json::json!([]),
            token_caveats: vec![],
        };

        let result = evaluator.evaluate_flow(&input).await;
        assert!(matches!(result, Err(PolicyError::Timeout(_))));
    }
}
