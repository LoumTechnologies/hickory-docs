<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Runtime observations retain their producing revision and occurrence

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a host using `hick-observe`, when a provider supplies runtime evidence and
the host saves it, then retrieval preserves its run/event identity, captured source
bytes and digest, optional block/invocation/thread/task identities, emission and
caller locations, precision, and typed payload. Receive order makes no claim
about causality between threads. Missing emission metadata means unknown.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/revision-bound-runtime-observations.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-fb97160c5f0d"></a>
## `crates/hick-observe/src/tests.rs`

Related contracts: [Runtime observations retain their producing revision and occurrence](contract.md).

<a id="f-fb97160c5f0d-run"></a>
### `run`

<storycode:copy id="f-fb97160c5f0d-run">
use super::*;
fn run() -> Run {
    Run::new(
        vec![Source::capture(
            "s".into(),
            "doc.md".into(),
            b"print(1)".to_vec(),
        )],
        1,
    )
}
</storycode:copy>
<a id="f-fb97160c5f0d-output"></a>
### `output`

<storycode:copy id="f-fb97160c5f0d-output">
fn output(id: &str, bytes: &[u8]) -> Event {
    Event {
        id: id.into(),
        provider: "fixture".into(),
        block: Some("cell".into()),
        invocation: Some(id.into()),
        parent_invocation: None,
        thread: Some(id.into()),
        task: None,
        time: None,
        emission: None,
        callers: vec![],
        observation: Observation::Output {
            stream: "stdout".into(),
            category: "console".into(),
            bytes: bytes.into(),
        },
    }
}
</storycode:copy>
<a id="f-fb97160c5f0d-concurrent_contributors_partial_unicode_crlf_and_controls"></a>
### `concurrent_contributors_partial_unicode_crlf_and_controls`

<storycode:copy id="f-fb97160c5f0d-concurrent_contributors_partial_unicode_crlf_and_controls">
// Guarantee: docs/guarantees/execution/revision-bound-runtime-observations.md
#[test]
fn concurrent_contributors_partial_unicode_crlf_and_controls() {
    let mut r = run();
    // Interleaved writers and repeated same-text invocations are distinct occurrences.
    for e in [
        output("a", &[0xe2]),
        output("b", &[0x82, 0xac, b'\r']),
        output("c", b"\nnext\n\x1b[31mpartial"),
        output("d", b"partial"),
    ] {
        r.append(e, 4096).unwrap();
    }
    let lines = output_lines(&r, "stdout", "console");
    assert_eq!(lines.len(), 3);
    assert_eq!(String::from_utf8(lines[0].bytes.clone()).unwrap(), "€\r\n");
    assert_eq!(
        lines[0]
            .contributors
            .iter()
            .map(|c| c.event.as_str())
            .collect::<Vec<_>>(),
        ["a", "b", "c"]
    );
    assert_eq!(lines[0].contributors[2].chunk_bytes, 0..1);
    assert_eq!(lines[1].bytes, b"next\n");
    assert_eq!(lines[1].contributors[0].chunk_bytes, 1..6);
    assert_eq!(lines[2].bytes, b"\x1b[31mpartialpartial");
    assert!(!lines[2].terminated);
    assert!(output_lines(&r, "stderr", "console").is_empty());
    assert_eq!(r.events[0].emission, None);
}
</storycode:copy>
<a id="f-fb97160c5f0d-revision_history_recovery_failure_retention_and_markdown_isolation"></a>
### `revision_history_recovery_failure_retention_and_markdown_isolation`

<storycode:copy id="f-fb97160c5f0d-revision_history_recovery_failure_retention_and_markdown_isolation">
// Guarantee: docs/guarantees/execution/revision-bound-runtime-observations.md
#[test]
fn revision_history_recovery_failure_retention_and_markdown_isolation() {
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("doc.md");
    std::fs::write(&doc, b"original <hick:output>kept</hick:output>").unwrap();
    let mut r = run();
    let id = r.id.clone();
    {
        let store = LocalStore::open(dir.path()).unwrap();
        assert!(LocalStore::open(dir.path()).is_err());
        r.append(output("1", b"before failure"), 4096).unwrap();
        store.save(&r).unwrap();
        let mut changed = r.clone();
        changed.sources[0] = Source::capture("s".into(), "doc.md".into(), b"new source".to_vec());
        assert!(store.save(&changed).is_err());
        assert!(!r.append(output("too-big", b"lost"), 1).unwrap());
        store.save(&r).unwrap();
    }
    let store = LocalStore::open(dir.path()).unwrap();
    let Retrieved::Present(recovered) = store.get(&id).unwrap() else {
        panic!()
    };
    assert_eq!(recovered.status, Status::Partial);
    assert!(recovered.truncated);
    assert_eq!(recovered.sources[0].bytes, b"print(1)");
    assert_eq!(recovered.events[0].id, "1");
    let mut failed = run();
    failed.finish(Status::Failed).unwrap();
    assert!(failed.append(output("late", b"x"), 4096).is_err());
    store.save(&failed).unwrap();
    store.pin(&id, true).unwrap();
    assert_eq!(
        store
            .retain(
                Retention {
                    max_unpinned_runs: 0,
                    max_unpinned_bytes: 0,
                    max_age_ms: 0
                },
                10
            )
            .unwrap(),
        [failed.id]
    );
    assert!(matches!(store.get(&id).unwrap(), Retrieved::Present(_)));
    assert!(store.delete(&id).unwrap());
    assert!(matches!(store.get(&id).unwrap(), Retrieved::Missing));
    assert!(store.get("../doc").is_err());
    assert_eq!(
        std::fs::read(&doc).unwrap(),
        b"original <hick:output>kept</hick:output>"
    );
}
</storycode:copy>
<a id="f-fb97160c5f0d-typed_evidence_preserves_wrapper_caller_and_profile_scope"></a>
### `typed_evidence_preserves_wrapper_caller_and_profile_scope`

<storycode:copy id="f-fb97160c5f0d-typed_evidence_preserves_wrapper_caller_and_profile_scope">
// Guarantee: docs/guarantees/execution/revision-bound-runtime-observations.md
#[test]
fn typed_evidence_preserves_wrapper_caller_and_profile_scope() {
    let mut r = run();
    let exact = Location {
        source: "s".into(),
        block: Some("cell".into()),
        bytes: Some(0..5),
        precision: Precision::Exact,
    };
    let caller = Location {
        bytes: Some(6..8),
        precision: Precision::AdapterReported,
        ..exact.clone()
    };
    let mut e = output("exception", b"");
    e.emission = Some(exact.clone());
    e.callers = vec![caller.clone()];
    e.observation = Observation::Exception {
        type_name: "ValueError".into(),
        message: "bad".into(),
        handling: Handling::Caught,
        throwing: Some(exact),
        catching: Some(caller),
    };
    r.append(e.clone(), 4096).unwrap();
    e.id = "profile".into();
    e.observation = Observation::Profile {
        metric: "cpu".into(),
        value: 1.2,
        unit: "ms".into(),
        interval: Interval {
            start: Clock {
                domain: "process-monotonic".into(),
                unit: "ns".into(),
                value: 0.,
            },
            end: Clock {
                domain: "process-monotonic".into(),
                unit: "ns".into(),
                value: 100.,
            },
            scope: "invocation".into(),
        },
        aggregation: "sum".into(),
        samples: Some(2),
        function: None,
    };
    r.append(e, 4096).unwrap();
    let mut err = output("stderr", b"Traceback is just text");
    if let Observation::Output { stream, .. } = &mut err.observation {
        *stream = "stderr".into();
    }
    r.append(err, 4096).unwrap();
    let decoded: Run = serde_json::from_slice(&serde_json::to_vec(&r).unwrap()).unwrap();
    assert_eq!(r, decoded);
    assert!(matches!(
        decoded.events[2].observation,
        Observation::Output { .. }
    ));
    assert_eq!(decoded.events[0].callers[0].bytes, Some(6..8));
    let mut invalid = output("invalid", b"x");
    invalid.emission = Some(Location {
        source: "s".into(),
        block: None,
        bytes: Some(0..999),
        precision: Precision::Exact,
    });
    assert!(r.append(invalid, 4096).is_err());
    assert_eq!(r.events.len(), 3);
}
</storycode:copy>
<a id="f-fb97160c5f0d-corrupt_artifacts_are_visible_and_not_overwritten"></a>
### `corrupt_artifacts_are_visible_and_not_overwritten`

<storycode:copy id="f-fb97160c5f0d-corrupt_artifacts_are_visible_and_not_overwritten">
// Guarantee: docs/guarantees/execution/revision-bound-runtime-observations.md
#[test]
fn corrupt_artifacts_are_visible_and_not_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let store = LocalStore::open(dir.path()).unwrap();
    let r = run();
    store.save(&r).unwrap();
    std::fs::write(
        dir.path().join(".hick/runs").join(format!("{}.json", r.id)),
        b"broken",
    )
    .unwrap();
    assert!(matches!(store.get(&r.id).unwrap(), Retrieved::Corrupt(_)));
    assert!(store.save(&r).is_err());
    assert!(
        store
            .retain(Retention::default(), u64::MAX)
            .unwrap()
            .is_empty()
    );
}

</storycode:copy>
<a id="f-fb97160c5f0d-provider_repeated_nested_calls_budget_and_errors"></a>
### `provider_repeated_nested_calls_budget_and_errors`

<storycode:copy id="f-fb97160c5f0d-provider_repeated_nested_calls_budget_and_errors">
// Guarantee: docs/guarantees/execution/revision-bound-runtime-observations.md
#[test]
fn provider_repeated_nested_calls_budget_and_errors() {
    struct Provider(std::collections::VecDeque<Event>);
    impl ObservationProvider for Provider {
        fn id(&self) -> &str {
            "fixture"
        }
        fn poll(&mut self) -> anyhow::Result<Option<Event>> {
            Ok(self.0.pop_front())
        }
    }
    let mut outer = output("outer", b"same");
    outer.invocation = Some("call-1".into());
    let mut recursive = output("recursive", b"same");
    recursive.invocation = Some("call-2".into());
    recursive.parent_invocation = Some("call-1".into());
    let mut repeated = output("again", b"same");
    repeated.invocation = Some("call-3".into());
    let budget =
        serde_json::to_vec(&outer).unwrap().len() + serde_json::to_vec(&recursive).unwrap().len();
    let mut provider = Provider([outer, recursive, repeated].into());
    let mut r = run();
    assert_eq!(r.collect(&mut provider, budget).unwrap(), 2);
    assert!(r.truncated);
    assert_eq!(r.events[1].parent_invocation.as_deref(), Some("call-1"));
    assert_ne!(r.events[0].invocation, r.events[1].invocation);
    let mut duplicate = r.events[0].clone();
    assert!(r.append(duplicate.clone(), 4096).is_err());
    duplicate.id = "different".into();
    duplicate.provider = "pretending".into();
    assert!(r.collect(&mut Provider([duplicate].into()), 99999).is_err());
    struct Broken;
    impl ObservationProvider for Broken {
        fn id(&self) -> &str {
            "broken"
        }
        fn poll(&mut self) -> anyhow::Result<Option<Event>> {
            anyhow::bail!("adapter disconnected")
        }
    }
    assert!(r.collect(&mut Broken, 99999).is_err());
    assert_eq!(r.events.len(), 2);
    r.finish(Status::Cancelled).unwrap();
    assert!(r.finish(Status::Completed).is_err());
}

</storycode:copy>
<a id="f-fb97160c5f0d-retention_age_bytes_and_count_are_independent"></a>
### `retention_age_bytes_and_count_are_independent`

<storycode:copy id="f-fb97160c5f0d-retention_age_bytes_and_count_are_independent">
// Guarantee: docs/guarantees/execution/revision-bound-runtime-observations.md
#[test]
fn retention_age_bytes_and_count_are_independent() {
    for policy in [
        Retention {
            max_unpinned_runs: 0,
            max_unpinned_bytes: u64::MAX,
            max_age_ms: u64::MAX,
        },
        Retention {
            max_unpinned_runs: usize::MAX,
            max_unpinned_bytes: 0,
            max_age_ms: u64::MAX,
        },
        Retention {
            max_unpinned_runs: usize::MAX,
            max_unpinned_bytes: u64::MAX,
            max_age_ms: 1,
        },
    ] {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let mut r = run();
        r.finish(Status::Completed).unwrap();
        store.save(&r).unwrap();
        let active = run();
        store.save(&active).unwrap();
        assert_eq!(store.retain(policy, 10).unwrap(), [r.id]);
        assert!(matches!(
            store.get(&active.id).unwrap(),
            Retrieved::Present(_)
        ));
    }
}
</storycode:copy>

### Assemble `crates/hick-observe/src/tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-observe/src/tests.rs"><storycode:paste select="#f-fb97160c5f0d-run"/><storycode:paste select="#f-fb97160c5f0d-output"/><storycode:paste select="#f-fb97160c5f0d-concurrent_contributors_partial_unicode_crlf_and_controls"/><storycode:paste select="#f-fb97160c5f0d-revision_history_recovery_failure_retention_and_markdown_isolation"/><storycode:paste select="#f-fb97160c5f0d-typed_evidence_preserves_wrapper_caller_and_profile_scope"/><storycode:paste select="#f-fb97160c5f0d-corrupt_artifacts_are_visible_and_not_overwritten"/><storycode:paste select="#f-fb97160c5f0d-provider_repeated_nested_calls_budget_and_errors"/><storycode:paste select="#f-fb97160c5f0d-retention_age_bytes_and_count_are_independent"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-observe/examples/local_run.rs](../../../mechanisms/crates/hick-observe/examples/local_run.rs.md#output-490351fb838b)
- [crates/hick-observe/src/lib.rs](../../../mechanisms/crates/hick-observe/src/lib.rs.md#output-022e74f036b0)
- [crates/hick-observe/src/reading.rs](../../debugging/source-anchored-observation-reading/contract.md#output-a6f0ea9e8feb)
- [crates/hick-observe/src/tests.rs](contract.md#output-fb97160c5f0d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
