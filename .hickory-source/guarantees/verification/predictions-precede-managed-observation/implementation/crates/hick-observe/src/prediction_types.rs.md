<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-observe/src/prediction_types.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ca9436bd58e3"></a>
## `crates/hick-observe/src/prediction_types.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../contract.md).

<a id="f-ca9436bd58e3-Actor"></a>
### `Actor`

Sealed prediction vocabulary; no execution or storage authority.

<storycode:copy id="f-ca9436bd58e3-Actor">
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
</storycode:copy>
<a id="f-ca9436bd58e3-impl-Actor"></a>
### `impl-Actor`

<storycode:copy id="f-ca9436bd58e3-impl-Actor">
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
</storycode:copy>
<a id="f-ca9436bd58e3-Identity"></a>
### `Identity`

<storycode:copy id="f-ca9436bd58e3-Identity">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identity {
    pub source: String,
    pub inputs: String,
    pub dependencies: BTreeMap<String, String>,
    pub environment: BTreeMap<String, String>,
}
</storycode:copy>
<a id="f-ca9436bd58e3-impl-Identity"></a>
### `impl-Identity`

<storycode:copy id="f-ca9436bd58e3-impl-Identity">
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
</storycode:copy>
<a id="f-ca9436bd58e3-Predicate"></a>
### `Predicate`

<storycode:copy id="f-ca9436bd58e3-Predicate">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Predicate {
    Equals(Value),
    NumberAtMost(f64),
    /// Every named target must match; missing/extra fields cannot pass.
    Fields(BTreeMap<String, Predicate>),
}
</storycode:copy>
<a id="f-ca9436bd58e3-impl-Predicate"></a>
### `impl-Predicate`

<storycode:copy id="f-ca9436bd58e3-impl-Predicate">
impl Predicate {
    /// Tool-schema definition; install under the root's `$defs.predicate`.
    /// Runtime admission additionally enforces UTF-8 byte, depth and size bounds.
    pub fn schema_definition() -> Value {
        serde_json::json!({"oneOf":[
            {"type":"object","properties":{"kind":{"const":"equals"},"value":{}},"required":["kind","value"],"additionalProperties":false},
            {"type":"object","properties":{"kind":{"const":"number_at_most"},"value":{"type":"number"}},"required":["kind","value"],"additionalProperties":false},
            {"type":"object","properties":{"kind":{"const":"fields"},"value":{"type":"object","minProperties":1,"maxProperties":16,"propertyNames":{"minLength":1,"maxLength":128},"additionalProperties":{"$ref":"#/$defs/predicate"}}},"required":["kind","value"],"additionalProperties":false}
        ]})
    }

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
</storycode:copy>
<a id="f-ca9436bd58e3-Scenario"></a>
### `Scenario`

<storycode:copy id="f-ca9436bd58e3-Scenario">
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
</storycode:copy>
<a id="f-ca9436bd58e3-impl-Scenario"></a>
### `impl-Scenario`

<storycode:copy id="f-ca9436bd58e3-impl-Scenario">
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
</storycode:copy>
<a id="f-ca9436bd58e3-Target"></a>
### `Target`

<storycode:copy id="f-ca9436bd58e3-Target">
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
</storycode:copy>
<a id="f-ca9436bd58e3-impl-Target"></a>
### `impl-Target`

<storycode:copy id="f-ca9436bd58e3-impl-Target">
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
</storycode:copy>
<a id="f-ca9436bd58e3-ObservedResult"></a>
### `ObservedResult`

<storycode:copy id="f-ca9436bd58e3-ObservedResult">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObservedResult {
    pub identity: Identity,
    pub target: Target,
    pub actual: Value,
    pub evidence: Value,
}
</storycode:copy>
<a id="f-ca9436bd58e3-PriorObservation"></a>
### `PriorObservation`

<storycode:copy id="f-ca9436bd58e3-PriorObservation">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PriorObservation {
    pub sequence: u64,
    pub scenario: String,
    pub actor: Actor,
    pub surface: String,
    pub identity: Identity,
    pub delivered_to_predictor: bool,
}
</storycode:copy>
<a id="f-ca9436bd58e3-Seal"></a>
### `Seal`

<storycode:copy id="f-ca9436bd58e3-Seal">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Seal {
    pub scenario: Scenario,
    pub identity: Identity,
    pub target: Target,
    pub predicted: Predicate,
    pub rationale: String,
}

</storycode:copy>
<a id="f-ca9436bd58e3-FutureStop"></a>
### `FutureStop`

Host-captured prior stop. Contains no variable values or evaluation text.

<storycode:copy id="f-ca9436bd58e3-FutureStop">
/// Host-captured prior stop. Contains no variable values or evaluation text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FutureStop {
    pub run: String,
    pub thread: i64,
    pub frame: i64,
    pub hit: u64,
    pub prior_visible_digest: String,
}
</storycode:copy>
<a id="f-ca9436bd58e3-impl-FutureStop"></a>
### `impl-FutureStop`

<storycode:copy id="f-ca9436bd58e3-impl-FutureStop">
impl FutureStop {
    pub(super) fn validate(&self) -> Result<()> {
        ensure!(
            !self.run.is_empty()
                && self.run.len() <= 256
                && self.thread > 0
                && self.frame > 0
                && (1..100).contains(&self.hit),
            "Invalid retained stop identity"
        );
        ensure!(
            self.prior_visible_digest.len() == 64
                && self
                    .prior_visible_digest
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
            "Invalid prior visible context digest"
        );
        Ok(())
    }
}
</storycode:copy>

### Assemble `crates/hick-observe/src/prediction_types.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-observe/src/prediction_types.rs"><storycode:paste select="#f-ca9436bd58e3-Actor"/><storycode:paste select="#f-ca9436bd58e3-impl-Actor"/><storycode:paste select="#f-ca9436bd58e3-Identity"/><storycode:paste select="#f-ca9436bd58e3-impl-Identity"/><storycode:paste select="#f-ca9436bd58e3-Predicate"/><storycode:paste select="#f-ca9436bd58e3-impl-Predicate"/><storycode:paste select="#f-ca9436bd58e3-Scenario"/><storycode:paste select="#f-ca9436bd58e3-impl-Scenario"/><storycode:paste select="#f-ca9436bd58e3-Target"/><storycode:paste select="#f-ca9436bd58e3-impl-Target"/><storycode:paste select="#f-ca9436bd58e3-ObservedResult"/><storycode:paste select="#f-ca9436bd58e3-PriorObservation"/><storycode:paste select="#f-ca9436bd58e3-Seal"/><storycode:paste select="#f-ca9436bd58e3-FutureStop"/><storycode:paste select="#f-ca9436bd58e3-impl-FutureStop"/></storycode:file>

</storycode:doc>
