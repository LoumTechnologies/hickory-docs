//! Sealed prediction vocabulary; no execution or storage authority.
use super::digest;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Actor {
    pub kind: String,
    pub id: String,
}
impl Actor {
    pub(super) fn validate(&self) -> Result<()> {
        ensure!(
            ["human", "agent", "host"].contains(&self.kind.as_str())
                && !self.id.is_empty()
                && self.id.len() <= 256,
            "Invalid host actor"
        );
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identity {
    pub source: String,
    pub inputs: String,
    pub dependencies: BTreeMap<String, String>,
    pub environment: BTreeMap<String, String>,
}
impl Identity {
    pub(super) fn validate(&self) -> Result<()> {
        for hash in std::iter::once(&self.source)
            .chain(std::iter::once(&self.inputs))
            .chain(self.dependencies.values())
            .chain(self.environment.values())
        {
            ensure!(
                hash.len() == 64
                    && hash
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
                "Exact SHA-256 identities are required"
            );
        }
        ensure!(
            !self.environment.is_empty(),
            "A captured execution environment identity is required"
        );
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Predicate {
    Equals(Value),
    NumberAtMost(f64),
    /// Every named target must match; missing/extra fields cannot pass.
    Fields(BTreeMap<String, Predicate>),
}
impl Predicate {
    pub(super) fn validate(&self) -> Result<()> {
        self.validate_depth(0)?;
        ensure!(
            serde_json::to_vec(self)?.len() <= 65536,
            "Predicate is too large"
        );
        Ok(())
    }
    fn validate_depth(&self, depth: usize) -> Result<()> {
        ensure!(depth <= 8, "Predicate nesting exceeds the supported bound");
        match self {
            Self::NumberAtMost(number) => ensure!(number.is_finite(), "Predicate must be finite"),
            Self::Fields(fields) => {
                ensure!(
                    !fields.is_empty() && fields.len() <= 16,
                    "A bounded field predicate is required"
                );
                for (key, predicate) in fields {
                    ensure!(!key.is_empty() && key.len() <= 128, "Invalid target field");
                    predicate.validate_depth(depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub fn matches(&self, actual: &Value) -> bool {
        match self {
            Self::Equals(value) => value == actual,
            Self::NumberAtMost(limit) => actual
                .as_f64()
                .is_some_and(|value| value.is_finite() && value <= *limit),
            Self::Fields(fields) => actual.as_object().is_some_and(|values| {
                values.len() == fields.len()
                    && fields.iter().all(|(key, predicate)| {
                        values
                            .get(key)
                            .is_some_and(|value| predicate.matches(value))
                    })
            }),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scenario {
    /// A host-recognized semantic family, not a filename, cell position or session.
    pub family: String,
    pub inputs: Value,
    pub requirement_reference: String,
    pub requirement: Predicate,
    /// False when prior scenario correspondence/exposure cannot be established.
    pub identity_known: bool,
}
impl Scenario {
    pub fn key(&self) -> Result<String> {
        ensure!(
            !self.family.is_empty()
                && self.family.len() <= 256
                && !self.requirement_reference.is_empty(),
            "Scenario family and requirement reference are required"
        );
        self.requirement.validate()?;
        // Source, filenames, requirement edits and predictions cannot mint a new
        // scenario. Different concrete inputs remain independent scenarios.
        digest(&(&self.family, &self.inputs))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Target {
    Output {
        execution: String,
    },
    /// One admission and one promotion gate for both observations of the same
    /// source. Both arms must be concrete targets, never nested combinations.
    Combined {
        output: Box<Target>,
        debugger: Box<Target>,
    },
    Debugger {
        source: String,
        source_revision: String,
        start_byte: u64,
        end_byte: u64,
        expression: String,
        hit: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        frame: Option<String>,
    },
}
impl Target {
    pub(super) fn validate(&self, identity: &Identity) -> Result<()> {
        match self {
            Self::Combined { output, debugger } => {
                ensure!(
                    matches!(output.as_ref(), Self::Output { .. })
                        && matches!(debugger.as_ref(), Self::Debugger { .. }),
                    "Combined prediction requires one output and one debugger target"
                );
                output.validate(identity)?;
                debugger.validate(identity)?;
            }
            Self::Output { execution } => ensure!(
                !execution.is_empty() && execution.len() <= 256,
                "Output prediction requires an execution identity"
            ),
            Self::Debugger {
                source,
                source_revision,
                start_byte,
                end_byte,
                expression,
                hit,
                frame,
            } => {
                ensure!(
                    !source.is_empty()
                        && start_byte < end_byte
                        && !expression.trim().is_empty()
                        && expression.len() <= 8192
                        && *hit > 0
                        && frame
                            .as_ref()
                            .is_none_or(|name| !name.trim().is_empty() && name.len() <= 512),
                    "Debugger prediction requires an exact source span, expression and positive hit"
                );
                let captured = if source == "root" {
                    Some(&identity.source)
                } else {
                    identity.dependencies.get(source)
                };
                ensure!(
                    captured == Some(source_revision),
                    "Debugger prediction source is not in the captured source/dependency identities"
                );
            }
        }
        Ok(())
    }
    pub(super) fn same_question(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Combined {
                    output: left_output,
                    debugger: left_debugger,
                },
                Self::Combined {
                    output: right_output,
                    debugger: right_debugger,
                },
            ) => {
                left_output.same_question(right_output)
                    && left_debugger.same_question(right_debugger)
            }
            (Self::Output { .. }, Self::Output { .. }) => true,
            (
                Self::Debugger {
                    expression: left,
                    hit: left_hit,
                    frame: left_frame,
                    ..
                },
                Self::Debugger {
                    expression: right,
                    hit: right_hit,
                    frame: right_frame,
                    ..
                },
            ) => left == right && left_hit == right_hit && left_frame == right_frame,
            _ => false,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObservedResult {
    pub identity: Identity,
    pub target: Target,
    pub actual: Value,
    pub evidence: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PriorObservation {
    pub sequence: u64,
    pub scenario: String,
    pub actor: Actor,
    pub surface: String,
    pub identity: Identity,
    pub delivered_to_predictor: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Seal {
    pub scenario: Scenario,
    pub identity: Identity,
    pub target: Target,
    pub predicted: Predicate,
    pub rationale: String,
}
