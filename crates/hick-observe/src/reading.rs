//! Derived reading surfaces. Captured source and evidence remain in the existing Run.
use crate::{Event, Location, Observation, Precision, Run, Source};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub session: Option<String>,
    pub run: String,
    pub stop: String,
    pub frame: Option<i64>,
    /// Only supplied when the runtime actually identifies an invocation.
    pub invocation: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DebugAnnotation {
    BreakpointRequested {
        breakpoint: String,
        state: String,
    },
    BreakpointBound {
        breakpoint: String,
    },
    Paused {
        reason: String,
    },
    Frame {
        name: String,
        depth: usize,
        reported_source: Option<String>,
        reported_line: Option<u32>,
    },
    Values {
        values: Vec<(String, String)>,
    },
    Expression {
        expression: String,
        value: String,
        receipt: String,
    },
    Exception {
        description: Option<String>,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    pub evidence: String,
    pub provider: String,
    pub payload_schema: String,
    pub location: Option<String>,
    pub scope: Scope,
    pub payload: DebugAnnotation,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationRef {
    pub id: String,
    pub revision: String,
    pub location: Location,
    /// Already mapped document coordinate, never a projected display row.
    pub line: u32,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reading {
    #[serde(default)]
    pub source_aliases: Vec<String>,
    pub schema_version: u32,
    pub source: Source,
    pub scope: Scope,
    pub locations: Vec<LocationRef>,
    pub annotations: Vec<Annotation>,
    pub unsupported: Vec<String>,
}
impl Reading {
    pub fn new(source: Source, scope: Scope) -> Self {
        Self {
            schema_version: 1,
            source_aliases: vec![],
            source,
            scope,
            locations: vec![],
            annotations: vec![],
            unsupported: vec!["invocation identity".into(), "exception details".into()],
        }
    }
    pub fn location(&mut self, line: u32) -> Option<String> {
        if let Some(l) = self.locations.iter().find(|l| l.line == line) {
            return Some(l.id.clone());
        }
        let text = std::str::from_utf8(&self.source.bytes).ok()?;
        let mut start = 0;
        let mut found = None;
        for (index, part) in text.split_inclusive('\n').enumerate() {
            if index == line as usize {
                found = Some(start..start + part.trim_end_matches(['\r', '\n']).len());
                break;
            }
            start += part.len();
        }
        if text.ends_with('\n') && line as usize == text.bytes().filter(|b| *b == b'\n').count() {
            found = Some(text.len()..text.len());
        }
        let bytes = found?;
        let id = format!("{}:L{}", self.source.revision, line);
        self.locations.push(LocationRef {
            id: id.clone(),
            revision: self.source.revision.clone(),
            location: Location {
                source: self.source.id.clone(),
                block: None,
                bytes: Some(bytes),
                precision: Precision::AdapterReported,
            },
            line,
        });
        Some(id)
    }
    pub fn annotate(
        &mut self,
        run: &mut Run,
        line: Option<u32>,
        frame: Option<i64>,
        payload: DebugAnnotation,
    ) -> Result<()> {
        let location = line.and_then(|line| self.location(line));
        let mut scope = self.scope.clone();
        scope.frame = frame;
        let id = format!("{}:E{}", run.id, run.events.len());
        let annotation = Annotation {
            evidence: id.clone(),
            provider: "dap".into(),
            payload_schema: "hick.debug-annotation.v1".into(),
            location: location.clone(),
            scope,
            payload,
        };
        run.append(
            Event {
                id,
                provider: annotation.provider.clone(),
                block: None,
                invocation: annotation.scope.invocation.clone(),
                parent_invocation: None,
                thread: None,
                task: None,
                time: None,
                emission: location.and_then(|id| {
                    self.locations
                        .iter()
                        .find(|l| l.id == id)
                        .map(|l| l.location.clone())
                }),
                callers: vec![],
                observation: Observation::Debugger {
                    reason: "annotation".into(),
                    details: serde_json::to_value(&annotation)?,
                },
            },
            usize::MAX,
        )?;
        self.annotations.push(annotation);
        Ok(())
    }
    /// Action resolution is a lookup in a captured revision, never label parsing.
    pub fn resolve(
        &self,
        location: &str,
        scope: &Scope,
        current: &[u8],
        frame: Option<i64>,
    ) -> Result<u32> {
        ensure!(
            scope == &self.scope && scope.frame == frame,
            "stale stop or wrong frame"
        );
        ensure!(current == self.source.bytes, "source changed");
        let l = self
            .locations
            .iter()
            .find(|l| l.id == location)
            .ok_or_else(|| anyhow::anyhow!("unknown location handle"))?;
        ensure!(l.revision == self.source.revision, "stale source revision");
        Ok(l.line)
    }
    /// Explicit selection only. No automatic inclusion of every captured event.
    pub fn selected(&self, evidence: &[String]) -> Self {
        let mut result = self.clone();
        result
            .annotations
            .retain(|a| evidence.contains(&a.evidence));
        result.locations.retain(|l| {
            result
                .annotations
                .iter()
                .any(|a| a.location.as_ref() == Some(&l.id))
        });
        result
    }
}

/// Non-runtime providers do not fabricate a run or stop identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EvidenceOrigin {
    Runtime {
        session: Option<String>,
        run: String,
        stop: Option<String>,
        frame: Option<i64>,
        invocation: Option<String>,
    },
    Review {
        repository: String,
        pull_request: u64,
        thread: String,
        base_revision: String,
        head_revision: String,
        side: String,
        author: String,
        resolved: bool,
        outdated: bool,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderAnnotation {
    pub evidence: String,
    pub provider: String,
    pub provider_version: String,
    pub payload_schema: String,
    pub origin: EvidenceOrigin,
    /// None is honestly unmapped; source precision is not measurement uncertainty.
    pub location: Option<LocationRef>,
    pub source_precision: Precision,
    pub measurement_semantics: Option<String>,
    pub uncertainty: Option<String>,
    pub payload: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Run, Reading) {
        let source = Source::capture(
            "source".into(),
            "demo.md".into(),
            "prose\nBREAKPOINT HERE\nx += 1;\n".as_bytes().to_vec(),
        );
        let run = Run::new(vec![source.clone()], 1);
        let reading = Reading::new(
            source,
            Scope {
                session: Some("session".into()),
                run: run.id.clone(),
                stop: "S1".into(),
                frame: Some(7),
                invocation: None,
            },
        );
        (run, reading)
    }
    // Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
    #[test]
    fn source_markers_do_not_create_evidence_and_relocated_breakpoints_stay_distinct() {
        let (mut run, mut reading) = fixture();
        assert!(reading.annotations.is_empty());
        reading
            .annotate(
                &mut run,
                Some(1),
                None,
                DebugAnnotation::BreakpointRequested {
                    breakpoint: "B1".into(),
                    state: "bound".into(),
                },
            )
            .unwrap();
        reading
            .annotate(
                &mut run,
                Some(2),
                None,
                DebugAnnotation::BreakpointBound {
                    breakpoint: "B1".into(),
                },
            )
            .unwrap();
        reading
            .annotate(
                &mut run,
                Some(2),
                Some(7),
                DebugAnnotation::Paused {
                    reason: "breakpoint".into(),
                },
            )
            .unwrap();
        assert_ne!(
            reading.annotations[0].location,
            reading.annotations[1].location
        );
        assert_eq!(
            reading.annotations[1].location,
            reading.annotations[2].location
        );
        assert_eq!(run.sources[0].bytes, reading.source.bytes);
        for (annotation, event) in reading.annotations.iter().zip(&run.events) {
            assert_eq!(annotation.evidence, event.id);
        }
        let selected = reading.selected(&[reading.annotations[2].evidence.clone()]);
        assert_eq!(selected.annotations.len(), 1);
        let json = serde_json::to_string(&selected).unwrap();
        assert_eq!(serde_json::from_str::<Reading>(&json).unwrap(), selected);
    }
    // Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
    #[test]
    fn handles_reject_changed_source_repeated_stops_wrong_frames_and_fabrications() {
        let (_, mut reading) = fixture();
        let id = reading.location(2).unwrap();
        assert_eq!(
            reading
                .resolve(&id, &reading.scope, &reading.source.bytes, Some(7))
                .unwrap(),
            2
        );
        let mut stale = reading.scope.clone();
        stale.stop = "S0".into();
        assert!(
            reading
                .resolve(&id, &stale, &reading.source.bytes, Some(7))
                .is_err()
        );
        assert!(
            reading
                .resolve(&id, &reading.scope, &reading.source.bytes, Some(8))
                .is_err()
        );
        assert!(
            reading
                .resolve(&id, &reading.scope, b"edited", Some(7))
                .is_err()
        );
        assert!(
            reading
                .resolve(
                    "BREAKPOINT HERE",
                    &reading.scope,
                    &reading.source.bytes,
                    Some(7)
                )
                .is_err()
        );
        let mut other_run = reading.scope.clone();
        other_run.run = "other".into();
        assert!(
            reading
                .resolve(&id, &other_run, &reading.source.bytes, Some(7))
                .is_err()
        );
    }
    #[test]
    fn logs_profiles_and_review_payloads_keep_their_own_semantics() {
        use crate::{Clock, Interval};
        for payload in [
            Observation::Log {
                level: "info".into(),
                message: "hello".into(),
                fields: Default::default(),
            },
            Observation::Profile {
                metric: "cpu".into(),
                value: 0.5,
                unit: "ms".into(),
                interval: Interval {
                    start: Clock {
                        domain: "sampler".into(),
                        unit: "ms".into(),
                        value: 0.0,
                    },
                    end: Clock {
                        domain: "sampler".into(),
                        unit: "ms".into(),
                        value: 1.0,
                    },
                    scope: "function".into(),
                },
                aggregation: "sample estimate".into(),
                samples: Some(2),
                function: None,
            },
        ] {
            let bytes = serde_json::to_vec(&payload).unwrap();
            assert_eq!(
                serde_json::from_slice::<Observation>(&bytes).unwrap(),
                payload
            );
        }
        let review = ProviderAnnotation {
            evidence: "review:thread1".into(),
            provider: "github".into(),
            provider_version: "1".into(),
            payload_schema: "review.v1".into(),
            location: None,
            origin: EvidenceOrigin::Review {
                repository: "owner/repo".into(),
                pull_request: 66,
                thread: "thread1".into(),
                base_revision: "base".into(),
                head_revision: "head".into(),
                side: "right".into(),
                author: "reviewer".into(),
                resolved: false,
                outdated: true,
            },
            source_precision: Precision::Unknown,
            measurement_semantics: None,
            uncertainty: None,
            payload: serde_json::json!({"body":"untrusted review text"}),
        };
        let json = serde_json::to_value(&review).unwrap();
        assert!(json["origin"].get("run").is_none());
        assert_eq!(
            serde_json::from_value::<ProviderAnnotation>(json).unwrap(),
            review
        );
    }
}

impl Reading {
    /// Source-adjacent textual equivalent with immutable byte lineage. Payload text is data.
    pub fn agent_projection(&self) -> serde_json::Value {
        let excerpts: Vec<_> = self
            .locations
            .iter()
            .map(|l| {
                let text = l
                    .location
                    .bytes
                    .as_ref()
                    .and_then(|r| self.source.bytes.get(r.clone()))
                    .and_then(|b| std::str::from_utf8(b).ok());
                serde_json::json!({"location":l,"text":text})
            })
            .collect();
        serde_json::json!({"schema_version":self.schema_version,"source":{"id":self.source.id,"uri":self.source.uri,"revision":self.source.revision},"scope":self.scope,"excerpts":excerpts,"annotations":self.annotations,"unsupported":self.unsupported})
    }
}

impl Run {
    /// Adapt every existing provider record to the common envelope, without a second store.
    pub fn provider_annotations(&self) -> Result<Vec<ProviderAnnotation>> {
        self.validate()?;
        let metadata = self
            .detail
            .as_deref()
            .and_then(|detail| serde_json::from_str::<serde_json::Value>(detail).ok());
        let version = metadata
            .as_ref()
            .and_then(|m| m.get("provider_version"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        self.events
            .iter()
            .map(|event| {
                let location = event.emission.as_ref().and_then(|location| {
                    let source = self.sources.iter().find(|s| s.id == location.source)?;
                    let start = location.bytes.as_ref()?.start;
                    Some(LocationRef {
                        id: format!(
                            "{}:{}:bytes:{}:{}",
                            source.id,
                            source.revision,
                            start,
                            location.bytes.as_ref()?.end
                        ),
                        revision: source.revision.clone(),
                        location: location.clone(),
                        line: source.bytes[..start]
                            .iter()
                            .filter(|b| **b == b'\n')
                            .count() as u32,
                    })
                });
                let mut origin = EvidenceOrigin::Runtime {
                    session: None,
                    run: self.id.clone(),
                    stop: None,
                    frame: None,
                    invocation: event.invocation.clone(),
                };
                let (schema, semantics) = match &event.observation {
                    Observation::Log { .. } => ("hick.log.v1", None),
                    Observation::Output { .. } => ("hick.output.v1", None),
                    Observation::Profile { aggregation, .. } => {
                        ("hick.profile.v1", Some(aggregation.clone()))
                    }
                    Observation::Exception { .. } => ("hick.exception.v1", None),
                    Observation::Debugger { details, .. } => {
                        if let Ok(annotation) =
                            serde_json::from_value::<Annotation>(details.clone())
                        {
                            origin = EvidenceOrigin::Runtime {
                                session: annotation.scope.session,
                                run: self.id.clone(),
                                stop: Some(annotation.scope.stop),
                                frame: annotation.scope.frame,
                                invocation: annotation.scope.invocation,
                            };
                        }
                        ("hick.debugger.v1", None)
                    }
                };
                Ok(ProviderAnnotation {
                    evidence: event.id.clone(),
                    provider: event.provider.clone(),
                    provider_version: version.into(),
                    payload_schema: schema.into(),
                    origin,
                    source_precision: event
                        .emission
                        .as_ref()
                        .map(|l| l.precision.clone())
                        .unwrap_or(Precision::Unknown),
                    location,
                    measurement_semantics: semantics,
                    uncertainty: matches!(event.observation, Observation::Profile { .. }).then(
                        || "Profiler overhead and timing variability are not estimated".into(),
                    ),
                    payload: serde_json::to_value(&event.observation)?,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod shared_fixture_tests {
    use super::*;
    // Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
    #[test]
    fn browser_fixture_is_the_rust_reading_and_agent_projection_contract() {
        let reading: Reading = serde_json::from_str(include_str!(
            "../../../apps/web/src/debug/fixtures/reading.json"
        ))
        .unwrap();
        Run::new(vec![reading.source.clone()], 0)
            .validate()
            .unwrap();
        let projection = reading.agent_projection();
        assert_eq!(projection["excerpts"][0]["text"], "value += 1;");
        assert_eq!(projection["annotations"][0]["evidence"], "opaque-E");
        let json = serde_json::to_value(&reading).unwrap();
        assert_eq!(serde_json::from_value::<Reading>(json).unwrap(), reading);
    }
}

#[cfg(test)]
mod provider_projection_tests {
    use super::*;
    // Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
    #[test]
    fn existing_run_records_share_the_provider_envelope_without_debugger_scope() {
        let source = Source::capture("doc".into(), "a.md".into(), b"x\n".to_vec());
        let mut run = Run::new(vec![source], 0);
        run.append(
            Event {
                id: "log-1".into(),
                provider: "logger".into(),
                block: None,
                invocation: None,
                parent_invocation: None,
                thread: None,
                task: None,
                time: None,
                emission: Some(Location {
                    source: "doc".into(),
                    block: None,
                    bytes: Some(0..1),
                    precision: Precision::Exact,
                }),
                callers: vec![],
                observation: Observation::Log {
                    level: "info".into(),
                    message: "hello".into(),
                    fields: Default::default(),
                },
            },
            10000,
        )
        .unwrap();
        let annotations = run.provider_annotations().unwrap();
        assert_eq!(annotations[0].evidence, "log-1");
        assert_eq!(annotations[0].source_precision, Precision::Exact);
        assert!(matches!(
            &annotations[0].origin,
            EvidenceOrigin::Runtime {
                stop: None,
                frame: None,
                ..
            }
        ));
        let json = serde_json::to_value(&annotations).unwrap();
        assert_eq!(
            serde_json::from_value::<Vec<ProviderAnnotation>>(json).unwrap(),
            annotations
        );
    }
}
