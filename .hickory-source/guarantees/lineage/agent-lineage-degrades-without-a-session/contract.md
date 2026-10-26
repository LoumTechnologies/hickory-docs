<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Agent-Authored Bytes Always Report Session, Turn, And A Git-Derived Author — Even When The Session Cannot Be Read

For: Authors editing generated text and reviewers tracing a result to its source.

Given a byte of generated output whose provenance origin is
`SourceOrigin::Agent { session, turn }`, when a reader asks for its lineage,
then the report always names the session id and the turn, and names the author
by deriving it from `git` rather than from any stored field — and when the
session file is missing, unreadable, unparseable, or does not record that turn,
the report says so with the path it looked for and why it failed, instead of
erroring, panicking, or degrading the origin to `synthetic`.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/agent-lineage-degrades-without-a-session.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-a790430ed0b2"></a>
## `crates/hick-flow/src/provenance.rs`

Related contracts: [Agent-Authored Bytes Always Report Session, Turn, And A Git-Derived Author — Even When The Session Cannot Be Read](contract.md).

<a id="f-a790430ed0b2-ProvenanceSpan"></a>
### `ProvenanceSpan`

Character-level provenance tracking for generated output.
Maps output byte ranges back to their source origins, enabling
tools like hick-lens to know which `.md` source positions
produced each region of a generated file.

<storycode:copy id="f-a790430ed0b2-ProvenanceSpan">
//! Character-level provenance tracking for generated output.
//!
//! Maps output byte ranges back to their source origins, enabling
//! tools like hick-lens to know which `.md` source positions
//! produced each region of a generated file.

use crate::node::SourceOrigin;

/// A single span in the output mapped to a source origin.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProvenanceSpan {
    /// Start byte offset in the output (inclusive).
    pub output_start: usize,
    /// End byte offset in the output (exclusive).
    pub output_end: usize,
    /// Where this output region came from.
    pub origin: SourceOrigin,
}

</storycode:copy>
<a id="f-a790430ed0b2-ProvenanceMap"></a>
### `ProvenanceMap`

Maps output byte ranges to their source origins.
Built by `converge_with_provenance()`
during file convergence. Spans are stored in output-order and do not overlap.

<storycode:copy id="f-a790430ed0b2-ProvenanceMap">
/// Maps output byte ranges to their source origins.
///
/// Built by [`converge_with_provenance()`](crate::converge::converge_with_provenance)
/// during file convergence. Spans are stored in output-order and do not overlap.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProvenanceMap {
    spans: Vec<ProvenanceSpan>,
}

</storycode:copy>
<a id="f-a790430ed0b2-impl-ProvenanceMap"></a>
### `impl-ProvenanceMap`

<storycode:copy id="f-a790430ed0b2-impl-ProvenanceMap">
impl ProvenanceMap {
    pub fn new() -> Self {
        Self { spans: Vec::new() }
    }

    /// Add a span to the map.
    pub fn push(&mut self, span: ProvenanceSpan) {
        self.spans.push(span);
    }

    /// Number of tracked spans.
    pub fn len(&self) -> usize {
        self.spans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    /// All spans, in output order.
    pub fn spans(&self) -> &[ProvenanceSpan] {
        &self.spans
    }

    /// Find the origin of the output at the given byte offset.
    pub fn origin_at(&self, byte_offset: usize) -> Option<&SourceOrigin> {
        self.spans
            .iter()
            .find(|s| s.output_start <= byte_offset && byte_offset < s.output_end)
            .map(|s| &s.origin)
    }

    /// Find all spans that overlap the given output range.
    pub fn spans_in_range(&self, start: usize, end: usize) -> Vec<&ProvenanceSpan> {
        self.spans
            .iter()
            .filter(|s| s.output_start < end && s.output_end > start)
            .collect()
    }
}

</storycode:copy>
<a id="f-a790430ed0b2-tests"></a>
### `tests`

<storycode:copy id="f-a790430ed0b2-tests">
#[cfg(test)]
mod tests {
    use super::*;
    use hick_lang::SourceSpan;
    use std::sync::Arc;

    #[test]
    fn empty_map() {
        let map = ProvenanceMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        assert!(map.origin_at(0).is_none());
    }

    #[test]
    fn origin_at_finds_correct_span() {
        let mut map = ProvenanceMap::new();
        map.push(ProvenanceSpan {
            output_start: 0,
            output_end: 5,
            origin: SourceOrigin::Literal {
                file: Arc::from("a.md"),
                span: SourceSpan::new(0, 5, 1, 0),
            },
        });
        map.push(ProvenanceSpan {
            output_start: 5,
            output_end: 10,
            origin: SourceOrigin::Paste {
                selector: Arc::from("#ver"),
                file: None,
                span: None,
            },
        });

        match map.origin_at(2) {
            Some(SourceOrigin::Literal { file, .. }) => assert_eq!(&**file, "a.md"),
            other => panic!("Expected Literal, got {:?}", other),
        }
        match map.origin_at(7) {
            Some(SourceOrigin::Paste { selector, .. }) => assert_eq!(&**selector, "#ver"),
            other => panic!("Expected Paste, got {:?}", other),
        }
        assert!(map.origin_at(10).is_none());
    }

    #[test]
    fn spans_in_range_returns_overlapping() {
        let mut map = ProvenanceMap::new();
        map.push(ProvenanceSpan {
            output_start: 0,
            output_end: 5,
            origin: SourceOrigin::Synthetic,
        });
        map.push(ProvenanceSpan {
            output_start: 5,
            output_end: 10,
            origin: SourceOrigin::Synthetic,
        });
        map.push(ProvenanceSpan {
            output_start: 10,
            output_end: 15,
            origin: SourceOrigin::Synthetic,
        });

        let overlapping = map.spans_in_range(3, 12);
        assert_eq!(overlapping.len(), 3); // all three overlap [3, 12)

        let overlapping = map.spans_in_range(5, 10);
        assert_eq!(overlapping.len(), 1); // only the middle span
        assert_eq!(overlapping[0].output_start, 5);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip() {
        let mut map = ProvenanceMap::new();
        map.push(ProvenanceSpan {
            output_start: 0,
            output_end: 42,
            origin: SourceOrigin::Literal {
                file: Arc::from("main.md"),
                span: SourceSpan::new(100, 142, 5, 2),
            },
        });
        map.push(ProvenanceSpan {
            output_start: 42,
            output_end: 60,
            origin: SourceOrigin::Exec {
                container: Arc::from("codegen"),
                tag_line: 10,
            },
        });
        map.push(ProvenanceSpan {
            output_start: 60,
            output_end: 70,
            origin: SourceOrigin::Paste {
                selector: Arc::from("#ver"),
                file: None,
                span: None,
            },
        });
        map.push(ProvenanceSpan {
            output_start: 70,
            output_end: 80,
            origin: SourceOrigin::Variable {
                name: Arc::from("version"),
            },
        });
        map.push(ProvenanceSpan {
            output_start: 80,
            output_end: 90,
            origin: SourceOrigin::Synthetic,
        });

        let json = serde_json::to_string(&map).unwrap();
        let map2: ProvenanceMap = serde_json::from_str(&json).unwrap();

        assert_eq!(map2.len(), 5);
        assert_eq!(map2.spans()[0].output_start, 0);
        assert_eq!(map2.spans()[0].output_end, 42);
        match &map2.spans()[0].origin {
            SourceOrigin::Literal { file, span } => {
                assert_eq!(file.as_ref(), "main.md");
                assert_eq!(span.start, 100);
                assert_eq!(span.end, 142);
            }
            other => panic!("Expected Literal, got {:?}", other),
        }
        match &map2.spans()[4].origin {
            SourceOrigin::Synthetic => {}
            other => panic!("Expected Synthetic, got {:?}", other),
        }
    }

    // Protects docs/guarantees/lineage/agent-lineage-degrades-without-a-session.md
    #[cfg(feature = "serde")]
    #[test]
    fn agent_origin_round_trips_and_is_additive() {
        let origin = SourceOrigin::Agent {
            session: Arc::from("abc123"),
            turn: 7,
            file: Some(Arc::from("doc.md")),
            span: Some(SourceSpan::new(10, 20, 3, 0)),
        };
        let json = serde_json::to_string(&origin).unwrap();
        assert!(json.contains(r#""type":"Agent""#), "{json}");
        match serde_json::from_str::<SourceOrigin>(&json).unwrap() {
            SourceOrigin::Agent {
                session,
                turn,
                file,
                span,
            } => {
                assert_eq!(session.as_ref(), "abc123");
                assert_eq!(turn, 7);
                assert_eq!(file.as_deref(), Some("doc.md"));
                assert_eq!(span.unwrap().start, 10);
            }
            other => panic!("Expected Agent, got {other:?}"),
        }

        // The document span is optional: an origin serialized without it
        // still deserializes, so adding the variant cannot break a stored map.
        let minimal = r#"{"type":"Agent","session":"abc123","turn":7}"#;
        match serde_json::from_str::<SourceOrigin>(minimal).unwrap() {
            SourceOrigin::Agent {
                session,
                turn,
                file,
                span,
            } => {
                assert_eq!(session.as_ref(), "abc123");
                assert_eq!(turn, 7);
                assert!(file.is_none());
                assert!(span.is_none());
            }
            other => panic!("Expected Agent, got {other:?}"),
        }

        // And the pre-existing variants are undisturbed.
        let literal = SourceOrigin::Literal {
            file: Arc::from("main.md"),
            span: SourceSpan::new(0, 5, 1, 0),
        };
        let json = serde_json::to_string(&literal).unwrap();
        assert!(matches!(
            serde_json::from_str::<SourceOrigin>(&json).unwrap(),
            SourceOrigin::Literal { .. }
        ));
    }
}
</storycode:copy>

### Assemble `crates/hick-flow/src/provenance.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-flow/src/provenance.rs"><storycode:paste select="#f-a790430ed0b2-ProvenanceSpan"/><storycode:paste select="#f-a790430ed0b2-ProvenanceMap"/><storycode:paste select="#f-a790430ed0b2-impl-ProvenanceMap"/><storycode:paste select="#f-a790430ed0b2-tests"/></storycode:file>

<a id="output-79fd95e5a8bf"></a>
## `crates/hickory-cli/tests/agent_lineage.rs`

Related contracts: [Agent-Authored Bytes Always Report Session, Turn, And A Git-Derived Author — Even When The Session Cannot Be Read](contract.md).

<a id="f-79fd95e5a8bf-git"></a>
### `git`

End-to-end shape of agent-authored lineage, from a `ProvenanceMap` to the
line `hick lineage` prints.
Protects docs/guarantees/lineage/agent-lineage-degrades-without-a-session.md
Nothing produces `SourceOrigin::Agent` spans yet — the `hick:agent` DAG
vertex is issue #7 — so the spans here are constructed synthetically. That
is deliberate: the reporting path is what this issue owns, and it must be
correct before there is a cell to feed it.

<storycode:copy id="f-79fd95e5a8bf-git">
//! End-to-end shape of agent-authored lineage, from a `ProvenanceMap` to the
//! line `hick lineage` prints.
//!
//! Protects docs/guarantees/lineage/agent-lineage-degrades-without-a-session.md
//!
//! Nothing produces `SourceOrigin::Agent` spans yet — the `hick:agent` DAG
//! vertex is issue #7 — so the spans here are constructed synthetically. That
//! is deliberate: the reporting path is what this issue owns, and it must be
//! correct before there is a cell to feed it.

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use hick_flow::{ProvenanceMap, ProvenanceSpan, SourceOrigin};
use hick_lang::SourceSpan;
use hickory_cli::agent_lineage::{self, Authorship, Reasoning};

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git available");
    assert!(out.status.success(), "git {args:?}: {:?}", out);
}

</storycode:copy>
<a id="f-79fd95e5a8bf-repo_with_committed_doc"></a>
### `repo_with_committed_doc`

A repo containing a committed document whose bytes an agent authored.

<storycode:copy id="f-79fd95e5a8bf-repo_with_committed_doc">
/// A repo containing a committed document whose bytes an agent authored.
fn repo_with_committed_doc() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["config", "user.name", "Nate Loum"]);
    git(dir.path(), &["config", "user.email", "nate@example.com"]);
    std::fs::write(
        dir.path().join("doc.md"),
        "first line\nsecond line\nthird line\n",
    )
    .unwrap();
    git(dir.path(), &["add", "doc.md"]);
    git(dir.path(), &["commit", "-q", "-m", "agent work"]);
    dir
}

</storycode:copy>
<a id="f-79fd95e5a8bf-agent_span"></a>
### `agent_span`

<storycode:copy id="f-79fd95e5a8bf-agent_span">
fn agent_span(session: &str, turn: usize, doc_span: Option<SourceSpan>) -> ProvenanceMap {
    let mut map = ProvenanceMap::new();
    map.push(ProvenanceSpan {
        output_start: 0,
        output_end: doc_span.map(|s| s.len()).unwrap_or(4),
        origin: SourceOrigin::Agent {
            session: Arc::from(session),
            turn,
            file: doc_span.map(|_| Arc::from("doc.md")),
            span: doc_span,
        },
    });
    map
}

</storycode:copy>
<a id="f-79fd95e5a8bf-agent_bytes_render_session_turn_author_and_missing_reasoning"></a>
### `agent_bytes_render_session_turn_author_and_missing_reasoning`

<storycode:copy id="f-79fd95e5a8bf-agent_bytes_render_session_turn_author_and_missing_reasoning">
#[test]
fn agent_bytes_render_session_turn_author_and_missing_reasoning() {
    let dir = repo_with_committed_doc();
    // "second line" starts at byte 11 of the document, on line 2.
    let map = agent_span("abc123", 7, Some(SourceSpan::new(11, 22, 2, 0)));
    let provenance = hickory_lineage::from_provenance_map(&map);
    let (session, turn) = provenance[0].origin.agent().expect("agent origin");
    let (_, doc_start, _) = provenance[0].origin.location().expect("document span");

    let doc_source = std::fs::read_to_string(dir.path().join("doc.md")).unwrap();
    let entry = agent_lineage::describe(
        dir.path(),
        "foo.rs",
        42,
        session,
        turn,
        Some((
            Path::new("doc.md"),
            agent_lineage::line_of(&doc_source, doc_start),
        )),
    );

    let rendered = entry.to_string();
    assert!(
        rendered.starts_with("foo.rs:42 ← session abc123 turn 7 · committed by Nate Loum"),
        "{rendered}"
    );
    // The session was never written here, so the reasoning is unreadable —
    // and that is reported, not raised.
    assert!(
        rendered.contains("reasoning not available to you"),
        "{rendered}"
    );
    assert!(matches!(entry.authorship, Authorship::Committed { .. }));
    assert!(matches!(entry.reasoning, Reasoning::Unavailable { .. }));
}

</storycode:copy>
<a id="f-79fd95e5a8bf-an_agent_byte_with_no_document_span_still_names_its_session"></a>
### `an_agent_byte_with_no_document_span_still_names_its_session`

<storycode:copy id="f-79fd95e5a8bf-an_agent_byte_with_no_document_span_still_names_its_session">
#[test]
fn an_agent_byte_with_no_document_span_still_names_its_session() {
    let dir = tempfile::tempdir().unwrap();
    let map = agent_span("def456", 0, None);
    let provenance = hickory_lineage::from_provenance_map(&map);
    // Never degraded to `synthetic`: the session id is the point.
    assert_eq!(provenance[0].origin.agent(), Some(("def456", 0)));

    let entry = agent_lineage::describe(dir.path(), "bar.rs", 1, "def456", 0, None);
    let rendered = entry.to_string();
    assert!(rendered.contains("session def456 turn 0"), "{rendered}");
    assert!(rendered.contains("author unknown"), "{rendered}");
    assert!(
        rendered.contains("reasoning not available to you"),
        "{rendered}"
    );
}

</storycode:copy>
<a id="f-79fd95e5a8bf-uncommitted_agent_work_is_attributed_to_the_reader"></a>
### `uncommitted_agent_work_is_attributed_to_the_reader`

<storycode:copy id="f-79fd95e5a8bf-uncommitted_agent_work_is_attributed_to_the_reader">
#[test]
fn uncommitted_agent_work_is_attributed_to_the_reader() {
    let dir = repo_with_committed_doc();
    // The agent edited the document but nobody has committed it yet.
    std::fs::write(
        dir.path().join("doc.md"),
        "first line\nrewritten by the agent\nthird line\n",
    )
    .unwrap();
    let entry = agent_lineage::describe(
        dir.path(),
        "foo.rs",
        42,
        "abc123",
        7,
        Some((Path::new("doc.md"), 2)),
    );
    match &entry.authorship {
        Authorship::Uncommitted { who } => assert!(who.contains("Nate Loum"), "{who}"),
        other => panic!("expected Uncommitted, got {other:?}"),
    }
    assert!(
        entry.to_string().contains("not committed yet — yours"),
        "{entry}"
    );
}

</storycode:copy>
<a id="f-79fd95e5a8bf-a_readable_session_is_reported_as_readable"></a>
### `a_readable_session_is_reported_as_readable`

<storycode:copy id="f-79fd95e5a8bf-a_readable_session_is_reported_as_readable">
#[test]
fn a_readable_session_is_reported_as_readable() {
    let dir = repo_with_committed_doc();
    let path = agent_lineage::session_path(dir.path(), "abc123");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <hick:session xmlns:hick=\"http://www.hickorydocs.com/1.0\" start=\"2026-01-01T00:00:00Z\">\n\
         <hick:user>write the thing</hick:user>\n\
         <hick:assistant>turn zero</hick:assistant>\n\
         </hick:session>\n",
    )
    .unwrap();
    let entry = agent_lineage::describe(
        dir.path(),
        "foo.rs",
        42,
        "abc123",
        0,
        Some((Path::new("doc.md"), 2)),
    );
    assert!(matches!(entry.reasoning, Reasoning::Available { .. }));
    assert!(entry.to_string().contains("reasoning in "), "{entry}");
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/agent_lineage.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/agent_lineage.rs"><storycode:paste select="#f-79fd95e5a8bf-git"/><storycode:paste select="#f-79fd95e5a8bf-repo_with_committed_doc"/><storycode:paste select="#f-79fd95e5a8bf-agent_span"/><storycode:paste select="#f-79fd95e5a8bf-agent_bytes_render_session_turn_author_and_missing_reasoning"/><storycode:paste select="#f-79fd95e5a8bf-an_agent_byte_with_no_document_span_still_names_its_session"/><storycode:paste select="#f-79fd95e5a8bf-uncommitted_agent_work_is_attributed_to_the_reader"/><storycode:paste select="#f-79fd95e5a8bf-a_readable_session_is_reported_as_readable"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-flow/src/node.rs](../../../mechanisms/crates/hick-flow/src/node.rs.md#output-90c967a2b659)
- [crates/hick-flow/src/provenance.rs](contract.md#output-a790430ed0b2)
- [crates/hickory-cli/src/agent_lineage.rs](../../../mechanisms/crates/hickory-cli/src/agent_lineage.rs.md#output-7b65c6db8d7c)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-lineage/src/lib.rs](../../../mechanisms/crates/hickory-lineage/src/lib.rs.md#output-bc43b0352ff3)

### Tests and fixtures

- [crates/hickory-cli/tests/agent_lineage.rs](contract.md#output-79fd95e5a8bf)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
