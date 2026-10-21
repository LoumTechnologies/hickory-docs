//! Host-owned predict/observe ordering. This ledger never executes a test.
//! Callers supply independently captured identities/results; transport adapters
//! must obtain actors from their host, never from model-supplied arguments.
use anyhow::{Context, Result, ensure};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Actor {
    pub kind: String,
    pub id: String,
}
impl Actor {
    fn validate(&self) -> Result<()> {
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
    fn validate(&self) -> Result<()> {
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
}
impl Predicate {
    fn validate(&self) -> Result<()> {
        if let Self::NumberAtMost(number) = self {
            ensure!(number.is_finite(), "Predicate must be finite");
        }
        ensure!(
            serde_json::to_vec(self)?.len() <= 65536,
            "Predicate is too large"
        );
        Ok(())
    }
    pub fn matches(&self, actual: &Value) -> bool {
        match self {
            Self::Equals(value) => value == actual,
            Self::NumberAtMost(limit) => actual
                .as_f64()
                .is_some_and(|value| value.is_finite() && value <= *limit),
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
    Debugger {
        source: String,
        source_revision: String,
        start_byte: u64,
        end_byte: u64,
        expression: String,
        hit: u64,
    },
}
impl Target {
    fn validate(&self, identity: &Identity) -> Result<()> {
        match self {
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
            } => {
                ensure!(
                    !source.is_empty()
                        && start_byte < end_byte
                        && !expression.trim().is_empty()
                        && expression.len() <= 8192
                        && *hit > 0,
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
    fn same_question(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Output { .. }, Self::Output { .. }) => true,
            (
                Self::Debugger {
                    expression: left,
                    hit: left_hit,
                    ..
                },
                Self::Debugger {
                    expression: right,
                    hit: right_hit,
                    ..
                },
            ) => left == right && left_hit == right_hit,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Event {
    Sealed {
        id: String,
        actor: Actor,
        seal: Seal,
        context: Vec<PriorObservation>,
    },
    Started {
        id: String,
        seal: String,
        key: String,
        replay: bool,
        actor: Actor,
    },
    Finished {
        attempt: String,
        actual: Value,
        evidence: Value,
    },
    Interrupted {
        attempt: String,
        actor: Actor,
        reason: Interruption,
    },
    Observed {
        attempt: Option<String>,
        scenario: String,
        actor: Actor,
        surface: String,
        identity: Identity,
    },
    Promoted {
        attempt: String,
        actor: Actor,
        identity: Identity,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Ordered {
    sequence: u64,
    previous: String,
    nonce: String,
    digest: String,
    event: Event,
}
#[derive(Default, Serialize, Deserialize)]
struct Journal {
    version: u32,
    entries: Vec<Ordered>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Interruption {
    Cancelled,
    HostInterrupted,
    EvidenceUnavailable,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    NotRun,
    Running,
    Mismatch,
    RequirementFailed,
    RequirementChanged,
    FirstAttemptVerified,
    VerifiedAfterCorrection,
    VerifiedAfterPriorAttempt,
    AdjustedAfterObservation,
    PredictionChanged,
    Stale,
    Unknown,
    Replayed,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub attempt: String,
    pub scenario: String,
    pub seal: String,
    pub classification: Classification,
    pub prediction_matched: Option<bool>,
    pub original_requirement_satisfied: Option<bool>,
    pub observed: bool,
    /// Historical event; current certification additionally requires current identities.
    pub promoted: bool,
    pub current_certification: bool,
}
/// Summaries deliberately exclude actual values and raw diagnostic/evidence data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub generation: u64,
    pub attempts: Vec<Summary>,
    pub ordering: Vec<Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub id: String,
    pub generation: u64,
}

pub fn digest(value: &impl Serialize) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(value)?)))
}

pub struct Ledger {
    root: PathBuf,
}
impl Ledger {
    /// The host chooses a durable private directory outside session/cache eviction.
    /// Absence is not evidence of no historical exposure: admissions must mark
    /// identity_known=false when correspondence/earlier history is unavailable.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        fs::create_dir_all(root.as_ref())?;
        Ok(Self {
            root: root.as_ref().to_owned(),
        })
    }
    fn transaction<T>(&self, apply: impl FnOnce(&mut Journal) -> Result<T>) -> Result<T> {
        let mut options = OpenOptions::new();
        options.create(true).truncate(false).read(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let lease = options.open(self.root.join("prediction.lock"))?;
        lease.lock_exclusive()?;
        let path = self.root.join("prediction.json");
        let mut journal: Journal = match fs::read(&path) {
            Ok(bytes) => {
                ensure!(
                    bytes.len() <= 128 * 1024 * 1024,
                    "Prediction ledger capacity exceeded"
                );
                serde_json::from_slice(&bytes)
                    .context("Prediction history is corrupt; refusing to reset it")?
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Journal {
                version: 1,
                entries: vec![],
            },
            Err(error) => return Err(error.into()),
        };
        ensure!(journal.version == 1, "Unknown prediction ledger version");
        let mut previous = String::new();
        for (index, entry) in journal.entries.iter().enumerate() {
            ensure!(
                entry.sequence == index as u64 + 1
                    && entry.previous == previous
                    && entry.digest
                        == digest(&(entry.sequence, &entry.previous, &entry.nonce, &entry.event))?,
                "Prediction event ordering/hash mismatch; history is unavailable"
            );
            previous = entry.digest.clone();
        }
        let before = journal.entries.len();
        let result = apply(&mut journal)?;
        if journal.entries.len() != before {
            let bytes = serde_json::to_vec(&journal)?;
            ensure!(
                bytes.len() <= 128 * 1024 * 1024,
                "Prediction ledger capacity exceeded"
            );
            let mut temporary = tempfile::NamedTempFile::new_in(&self.root)?;
            temporary.write_all(&bytes)?;
            temporary.as_file().sync_all()?;
            temporary.persist(path)?;
            #[cfg(unix)]
            File::open(&self.root)?.sync_all()?;
        }
        Ok(result)
    }
    pub fn seal(&self, expected_generation: u64, seal: Seal, actor: Actor) -> Result<Receipt> {
        actor.validate()?;
        seal.identity.validate()?;
        seal.target.validate(&seal.identity)?;
        seal.predicted.validate()?;
        ensure!(
            !seal.rationale.trim().is_empty() && seal.rationale.len() <= 8192,
            "A bounded prediction rationale is required"
        );
        seal.scenario.key()?;
        ensure!(
            seal.identity.inputs == digest(&seal.scenario.inputs)?,
            "Prediction inputs do not match their captured identity"
        );
        self.transaction(|journal| {
            generation(journal, expected_generation)?;
            let context = journal
                .entries
                .iter()
                .filter_map(|entry| match &entry.event {
                    Event::Observed {
                        scenario,
                        actor: observer,
                        surface,
                        identity,
                        ..
                    } => Some(PriorObservation {
                        sequence: entry.sequence,
                        scenario: scenario.clone(),
                        actor: observer.clone(),
                        surface: surface.clone(),
                        identity: identity.clone(),
                        delivered_to_predictor: observer == &actor,
                    }),
                    _ => None,
                })
                .collect();
            let id = Uuid::new_v4().to_string();
            append(
                journal,
                Event::Sealed {
                    id: id.clone(),
                    actor,
                    seal,
                    context,
                },
            )?;
            Ok(Receipt {
                id,
                generation: journal.entries.len() as u64,
            })
        })
    }
    pub fn begin(
        &self,
        seal_id: &str,
        key: &str,
        identity: &Identity,
        replay: bool,
        actor: Actor,
    ) -> Result<Receipt> {
        actor.validate()?;
        identity.validate()?;
        ensure!(
            !key.is_empty() && key.len() <= 128,
            "A bounded execution request id is required"
        );
        self.transaction(|journal| {
            let (_, seal) = find_seal(journal, seal_id)?;
            ensure!(&seal.identity == identity, "Prediction is stale; seal the current captured inputs before execution");
            for entry in &journal.entries {
                if let Event::Started { id, seal, key: prior, replay: old_replay, actor: owner } = &entry.event
                    && prior == key {
                    ensure!(seal == seal_id && owner == &actor && *old_replay == replay, "Execution request id was reused for a different operation");
                    return Ok(Receipt { id: id.clone(), generation: journal.entries.len() as u64 });
                }
            }
            let scenario = seal.scenario.key()?;
            for entry in &journal.entries {
                if let Event::Started { id, seal: old, .. } = &entry.event
                    && find_seal(journal, old)?.1.scenario.key()? == scenario {
                    ensure!((finished(journal, id).is_some() || interrupted(journal, id)), "This scenario has an unresolved execution; inspect/recover its attempt before starting another");
                }
            }
            let id = Uuid::new_v4().to_string();
            append(journal, Event::Started { id: id.clone(), seal: seal_id.into(), key: key.into(), replay, actor })?;
            Ok(Receipt { id, generation: journal.entries.len() as u64 })
        })
    }
    /// Host-only actual result intake. No actual values are returned in feedback.
    pub fn finish(&self, attempt: &str, observation: ObservedResult) -> Result<Summary> {
        let ObservedResult {
            identity,
            target,
            actual,
            evidence,
        } = observation;
        identity.validate()?;
        target.validate(&identity)?;
        ensure!(
            serde_json::to_vec(&(&actual, &evidence))?.len() <= 1048576,
            "Attempt evidence exceeds the private payload limit"
        );
        self.transaction(|journal| {
            let (_, seal, _) = find_attempt(journal, attempt)?;
            ensure!(!interrupted(journal, attempt), "The attempt was interrupted; a late result cannot replace that outcome");
            ensure!(identity == seal.identity && target == seal.target, "Observed source, expression/location or hit differs from the sealed prediction; no certification was recorded");
            if let Some((old_actual, old_evidence)) = finished(journal, attempt) {
                ensure!(old_actual == &actual && old_evidence == &evidence, "A settled attempt cannot be replaced");
            } else { append(journal, Event::Finished { attempt: attempt.into(), actual, evidence })?; }
            summarize(journal, attempt, None)
        })
    }
    /// Called after the authoritative executor confirms cancellation/interruption.
    /// It releases a pending attempt without inventing an observation or success.
    pub fn interrupt(&self, attempt: &str, actor: Actor, reason: Interruption) -> Result<Summary> {
        actor.validate()?;
        ensure!(
            actor.kind == "host",
            "Only the authoritative executor can settle an interrupted attempt"
        );
        self.transaction(|journal| {
            find_attempt(journal, attempt)?;
            ensure!(
                finished(journal, attempt).is_none(),
                "An executed result cannot be replaced by interruption"
            );
            if !interrupted(journal, attempt) {
                append(
                    journal,
                    Event::Interrupted {
                        attempt: attempt.into(),
                        actor,
                        reason,
                    },
                )?;
            }
            summarize(journal, attempt, None)
        })
    }
    /// Human-only inspection. Agent feedback must never contain the held-out answer,
    /// even on explicit request. Host adapters must keep human evidence out of agent
    /// context, cached transcripts and tool responses.
    pub fn reveal(&self, attempt: &str, actor: Actor, surface: &str) -> Result<Value> {
        actor.validate()?;
        ensure!(
            actor.kind == "human",
            "Held-out answers cannot be revealed to an agent or host tool response"
        );
        ensure!(
            !surface.is_empty() && surface.len() <= 256,
            "A bounded exposure surface is required"
        );
        self.transaction(|journal| {
            let (_, seal, _) = find_attempt(journal, attempt)?;
            let scenario = seal.scenario.key()?;
            let identity = seal.identity.clone();
            let (actual, evidence) =
                finished(journal, attempt).context("Attempt has no retained result to reveal")?;
            let result = serde_json::json!({ "actual": actual, "evidence": evidence });
            append(
                journal,
                Event::Observed {
                    attempt: Some(attempt.into()),
                    scenario,
                    identity,
                    actor,
                    surface: surface.into(),
                },
            )?;
            Ok(result)
        })
    }
    /// Debugger/cached-output/context adapters call this before releasing values,
    /// including observations made before a prediction or run exists.
    pub fn observe(
        &self,
        scenario: &Scenario,
        identity: Identity,
        actor: Actor,
        surface: &str,
    ) -> Result<Receipt> {
        actor.validate()?;
        identity.validate()?;
        ensure!(
            !surface.is_empty() && surface.len() <= 256,
            "A bounded exposure surface is required"
        );
        let scenario = scenario.key()?;
        self.transaction(|journal| {
            append(
                journal,
                Event::Observed {
                    attempt: None,
                    scenario: scenario.clone(),
                    identity,
                    actor,
                    surface: surface.into(),
                },
            )?;
            Ok(Receipt {
                id: scenario,
                generation: journal.entries.len() as u64,
            })
        })
    }
    pub fn history(&self, current: Option<&Identity>) -> Result<History> {
        self.transaction(|journal| {
            let attempts = journal.entries.iter().filter_map(|entry| if let Event::Started { id, .. } = &entry.event { Some(id) } else { None })
                .map(|id| summarize(journal, id, current)).collect::<Result<Vec<_>>>()?;
            let ordering = journal.entries.iter().map(|entry| {
                let event = match &entry.event {
                    Event::Sealed { id, actor, seal, context } => serde_json::json!({"kind":"sealed", "id":id, "actor":actor, "seal":seal, "context":context}),
                    Event::Started { id, seal, replay, actor, .. } => serde_json::json!({"kind":"started", "id":id, "seal":seal, "replay":replay, "actor":actor}),
                    Event::Finished { attempt, .. } => serde_json::json!({"kind":"finished", "attempt":attempt, "actual":"withheld from agents"}),
                    Event::Interrupted { attempt, actor, reason } => serde_json::json!({"kind":"interrupted", "attempt":attempt, "actor":actor, "reason":reason}),
                    Event::Observed { attempt, scenario, actor, surface, identity } => serde_json::json!({"kind":"observed", "attempt":attempt, "scenario":scenario, "actor":actor, "surface":surface, "identity":identity}),
                    Event::Promoted { attempt, actor, identity } => serde_json::json!({"kind":"promoted", "attempt":attempt, "actor":actor, "identity":identity}),
                };
                serde_json::json!({"sequence":entry.sequence, "previous":entry.previous, "digest":entry.digest, "event":event})
            }).collect();
            Ok(History { generation: journal.entries.len() as u64, attempts, ordering })
        })
    }
    pub fn promote(&self, attempt: &str, current: &Identity, actor: Actor) -> Result<Summary> {
        actor.validate()?;
        current.validate()?;
        self.transaction(|journal| {
            let summary = summarize(journal, attempt, Some(current))?;
            ensure!(matches!(summary.classification, Classification::FirstAttemptVerified | Classification::VerifiedAfterCorrection | Classification::VerifiedAfterPriorAttempt), "Promotion blocked: {:?}; inspect prediction/requirement/history and reverify the current source", summary.classification);
            if !summary.promoted { append(journal, Event::Promoted { attempt: attempt.into(), identity: current.clone(), actor })?; }
            summarize(journal, attempt, Some(current))
        })
    }
}

fn generation(journal: &Journal, expected: u64) -> Result<()> {
    ensure!(
        journal.entries.len() as u64 == expected,
        "Prediction history changed; inspect it before submitting a current seal"
    );
    Ok(())
}
fn append(journal: &mut Journal, event: Event) -> Result<()> {
    ensure!(
        journal.entries.len() < 100000,
        "Prediction event capacity exceeded"
    );
    let sequence = journal.entries.len() as u64 + 1;
    let previous = journal
        .entries
        .last()
        .map(|entry| entry.digest.clone())
        .unwrap_or_default();
    let nonce = Uuid::new_v4().to_string();
    let hash = digest(&(sequence, &previous, &nonce, &event))?;
    journal.entries.push(Ordered {
        sequence,
        previous,
        nonce,
        digest: hash,
        event,
    });
    Ok(())
}
fn find_seal<'a>(journal: &'a Journal, id: &str) -> Result<(u64, &'a Seal)> {
    journal
        .entries
        .iter()
        .find_map(|entry| {
            if let Event::Sealed {
                id: found, seal, ..
            } = &entry.event
            {
                (found == id).then_some((entry.sequence, seal))
            } else {
                None
            }
        })
        .context("No host-sealed prediction exists")
}
fn find_attempt<'a>(journal: &'a Journal, id: &str) -> Result<(u64, &'a Seal, bool)> {
    let (sequence, seal, replay) = journal
        .entries
        .iter()
        .find_map(|entry| {
            if let Event::Started {
                id: found,
                seal,
                replay,
                ..
            } = &entry.event
            {
                (found == id).then_some((entry.sequence, seal, *replay))
            } else {
                None
            }
        })
        .context("Unknown prediction attempt")?;
    Ok((sequence, find_seal(journal, seal)?.1, replay))
}
fn finished<'a>(journal: &'a Journal, id: &str) -> Option<(&'a Value, &'a Value)> {
    journal.entries.iter().find_map(|entry| {
        if let Event::Finished {
            attempt,
            actual,
            evidence,
        } = &entry.event
        {
            (attempt == id).then_some((actual, evidence))
        } else {
            None
        }
    })
}
fn interrupted(journal: &Journal, id: &str) -> bool {
    journal
        .entries
        .iter()
        .any(|entry| matches!(&entry.event, Event::Interrupted { attempt, .. } if attempt == id))
}
fn summarize(journal: &Journal, attempt: &str, current: Option<&Identity>) -> Result<Summary> {
    let (sequence, seal, replay) = find_attempt(journal, attempt)?;
    let key = seal.scenario.key()?;
    let (first_sequence, original) = journal
        .entries
        .iter()
        .find_map(|entry| {
            if let Event::Sealed { seal, .. } = &entry.event {
                (seal.scenario.key().ok().as_ref() == Some(&key)).then_some((entry.sequence, seal))
            } else {
                None
            }
        })
        .context("Scenario has no original prediction")?;
    let (seal_id, sealed_at) = journal
        .entries
        .iter()
        .find_map(|entry| {
            if let Event::Started { id, seal, .. } = &entry.event {
                (id == attempt).then_some((seal.clone(), find_seal(journal, seal).ok()?.0))
            } else {
                None
            }
        })
        .context("Missing attempt seal")?;
    let observations: Vec<_> = journal
        .entries
        .iter()
        .filter(
            |entry| matches!(&entry.event, Event::Observed { scenario, .. } if scenario == &key),
        )
        .collect();
    let observed_before_seal = observations.iter().any(|entry| entry.sequence < sealed_at);
    let unknown_origin = observations
        .iter()
        .any(|entry| entry.sequence < first_sequence);
    let requirement_changed = original.scenario.requirement != seal.scenario.requirement
        || original.scenario.requirement_reference != seal.scenario.requirement_reference;
    let question_changed = !original.target.same_question(&seal.target);
    let adjusted =
        (original.predicted != seal.predicted || question_changed || requirement_changed)
            && observed_before_seal;
    let actual = finished(journal, attempt).map(|(actual, _)| actual);
    let matched = actual.map(|value| seal.predicted.matches(value));
    let requirement = actual.map(|value| original.scenario.requirement.matches(value));
    let prior_attempt = journal.entries.iter().any(|entry| entry.sequence < sequence && matches!(&entry.event, Event::Started { seal, .. } if find_seal(journal, seal).ok().and_then(|(_, seal)| seal.scenario.key().ok()).as_ref() == Some(&key)));
    let corrected = journal.entries.iter().any(|entry| {
        entry.sequence < sequence
            && match &entry.event {
                Event::Started {
                    id, seal: previous, ..
                } => find_seal(journal, previous).ok().is_some_and(|(_, old)| {
                    old.scenario.key().ok().as_ref() == Some(&key)
                        && old.identity.source != seal.identity.source
                        && finished(journal, id).is_some_and(|(actual, _)| {
                            !old.predicted.matches(actual)
                                || !original.scenario.requirement.matches(actual)
                        })
                }),
                _ => false,
            }
    });
    let classification = if current.is_some_and(|current| current != &seal.identity) {
        Classification::Stale
    } else if !seal.scenario.identity_known || !original.scenario.identity_known || unknown_origin {
        Classification::Unknown
    } else if replay {
        Classification::Replayed
    } else if interrupted(journal, attempt) {
        Classification::Unknown
    } else if actual.is_none() {
        Classification::Running
    } else if adjusted {
        Classification::AdjustedAfterObservation
    } else if requirement_changed {
        Classification::RequirementChanged
    } else if question_changed {
        Classification::Unknown
    } else if original.predicted != seal.predicted {
        Classification::PredictionChanged
    } else if matched != Some(true) {
        Classification::Mismatch
    } else if requirement != Some(true) {
        Classification::RequirementFailed
    } else if corrected {
        Classification::VerifiedAfterCorrection
    } else if prior_attempt {
        Classification::VerifiedAfterPriorAttempt
    } else {
        Classification::FirstAttemptVerified
    };
    let promoted = journal
        .entries
        .iter()
        .any(|entry| matches!(&entry.event, Event::Promoted { attempt: id, .. } if id == attempt));
    let current_certification = current.is_some()
        && promoted
        && matches!(
            classification,
            Classification::FirstAttemptVerified
                | Classification::VerifiedAfterCorrection
                | Classification::VerifiedAfterPriorAttempt
        );
    Ok(Summary {
        current_certification,
        attempt: attempt.into(),
        scenario: key,
        seal: seal_id,
        classification,
        prediction_matched: matched,
        original_requirement_satisfied: requirement,
        observed: !observations.is_empty(),
        promoted,
    })
}

#[cfg(test)]
#[path = "prediction_tests.rs"]
mod tests;
