//! End-to-end shape of agent-authored lineage, from a `ProvenanceMap` to the
//! line `hickory lineage` prints.
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

/// A repo containing a committed document whose bytes an agent authored.
fn repo_with_committed_doc() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["config", "user.name", "Nate Loum"]);
    git(dir.path(), &["config", "user.email", "nate@example.com"]);
    std::fs::write(
        dir.path().join("doc.hick"),
        "first line\nsecond line\nthird line\n",
    )
    .unwrap();
    git(dir.path(), &["add", "doc.hick"]);
    git(dir.path(), &["commit", "-q", "-m", "agent work"]);
    dir
}

fn agent_span(session: &str, turn: usize, doc_span: Option<SourceSpan>) -> ProvenanceMap {
    let mut map = ProvenanceMap::new();
    map.push(ProvenanceSpan {
        output_start: 0,
        output_end: doc_span.map(|s| s.len()).unwrap_or(4),
        origin: SourceOrigin::Agent {
            session: Arc::from(session),
            turn,
            file: doc_span.map(|_| Arc::from("doc.hick")),
            span: doc_span,
        },
    });
    map
}

#[test]
fn agent_bytes_render_session_turn_author_and_missing_reasoning() {
    let dir = repo_with_committed_doc();
    // "second line" starts at byte 11 of the document, on line 2.
    let map = agent_span("abc123", 7, Some(SourceSpan::new(11, 22, 2, 0)));
    let provenance = hickory_lineage::from_provenance_map(&map);
    let (session, turn) = provenance[0].origin.agent().expect("agent origin");
    let (_, doc_start, _) = provenance[0].origin.location().expect("document span");

    let doc_source = std::fs::read_to_string(dir.path().join("doc.hick")).unwrap();
    let entry = agent_lineage::describe(
        dir.path(),
        "foo.rs",
        42,
        session,
        turn,
        Some((
            Path::new("doc.hick"),
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

#[test]
fn uncommitted_agent_work_is_attributed_to_the_reader() {
    let dir = repo_with_committed_doc();
    // The agent edited the document but nobody has committed it yet.
    std::fs::write(
        dir.path().join("doc.hick"),
        "first line\nrewritten by the agent\nthird line\n",
    )
    .unwrap();
    let entry = agent_lineage::describe(
        dir.path(),
        "foo.rs",
        42,
        "abc123",
        7,
        Some((Path::new("doc.hick"), 2)),
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
        Some((Path::new("doc.hick"), 2)),
    );
    assert!(matches!(entry.reasoning, Reasoning::Available { .. }));
    assert!(entry.to_string().contains("reasoning in "), "{entry}");
}
