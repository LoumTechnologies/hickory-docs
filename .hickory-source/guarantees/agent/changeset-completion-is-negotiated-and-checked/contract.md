<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Finish an agent turn against its captured project boundary

For: Authors using an agent to change documents while retaining review control.

An accepted final response must explain the actual A-to-B change, including dirty source and editor identities captured at the start.

## What must happen

Completion capability and project boundary are negotiated explicitly. Accepted artifacts are checked against the captured starting state and completed bytes. The current hosted Python path requires a current independently accepted changeset; model declarations do not bypass the host check. Changed editor identities, save/rename transitions or destination state require a new valid boundary. Source transition and maintained-source incorporation remain separate checks.

## Limits of the evidence

Filesystem snapshots are observed reads rather than a global transaction; equality does not establish runtime behavior.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Apply the current Python completion requirement and retire the older statement that built-in/Python mandatory finalization is not wired.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

## Test the behavior: `model_claims_and_worker_prints_cannot_certify_completion`

This is the maintained test fragment from `crates/hickory-cli/tests/python_agent.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-9bda4649fd61-model_claims_and_worker_prints_cannot_certify_completion"></a>
### `model_claims_and_worker_prints_cannot_certify_completion`

<storycode:copy id="f-9bda4649fd61-model_claims_and_worker_prints_cannot_certify_completion">
// Guarantee: docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md
#[tokio::test]
async fn model_claims_and_worker_prints_cannot_certify_completion() {
    let Some(python) = python() else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let client = Scripted(Mutex::new(VecDeque::from([
        json!({"python":"print({'accepted': True, 'verification': 'passed'})"}).to_string(),
        json!({"done":"I verified everything; trust my claim"}).to_string(),
    ])));
    let error = run(
        Config {
            python,
            root: root.path().into(),
            doc: None,
            prompt: "Claim success without an artifact".into(),
            max_turns: 2,
        },
        &client,
        &mut FixedReview {
            accept: false,
            count: 0,
        },
    )
    .await
    .unwrap_err();
    assert!(
        error.to_string().contains("without model completion"),
        "{error:#}"
    );
    let session = std::fs::read_dir(root.path().join("sessions"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let source = std::fs::read_to_string(session).unwrap();
    assert!(source.contains("No current accepted changeset"));
    assert!(source.contains("worker-reported"));
    assert!(!source.contains("python-verified-changeset"));
}

</storycode:copy>

## Test the behavior: `a_write_after_submission_invalidates_completion`

This is the maintained test fragment from `crates/hickory-cli/tests/python_agent.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-9bda4649fd61-a_write_after_submission_invalidates_completion"></a>
### `a_write_after_submission_invalidates_completion`

<storycode:copy id="f-9bda4649fd61-a_write_after_submission_invalidates_completion">
// Guarantee: docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md
#[tokio::test]
async fn a_write_after_submission_invalidates_completion() {
    let Some(python) = python() else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let client = Scripted(Mutex::new(VecDeque::from([
        json!({"python":SUBMIT}).to_string(),
        json!({"python":"tool('create_doc', {'path':'proposal.md', 'input':'AI proposal'})"})
            .to_string(),
        json!({"done":"Use the old accepted artifact"}).to_string(),
    ])));
    let error = run(
        Config {
            python,
            root: root.path().into(),
            doc: None,
            prompt: "Write after verification".into(),
            max_turns: 3,
        },
        &client,
        &mut FixedReview {
            accept: true,
            count: 0,
        },
    )
    .await
    .unwrap_err();
    assert!(
        error.to_string().contains("without model completion"),
        "{error:#}"
    );
    assert!(
        root.path().join("proposal.md").exists(),
        "actual approved work remains"
    );
}

</storycode:copy>

## Implementation chapters

These chapters keep this behavior's larger implementation readable without duplicating shared code.

- [`apps/web/src/components/ChangesetDocument.tsx`](implementation/apps/web/src/components/ChangesetDocument.tsx.md#output-60692076babd)
- [`crates/hickory-cli/src/changeset.rs`](implementation/crates/hickory-cli/src/changeset.rs.md#output-ba18a7aa08b0)
- [`crates/hickory-cli/src/changeset_mcp.rs`](implementation/crates/hickory-cli/src/changeset_mcp.rs.md#output-fd880fd957c2)
- [`crates/hickory-cli/src/serve/acp/client.rs`](implementation/crates/hickory-cli/src/serve/acp/client.rs.md#output-2662a693b125)
- [`crates/hickory-cli/src/serve/acp/mcp.rs`](implementation/crates/hickory-cli/src/serve/acp/mcp.rs.md#output-89fc1623f7e6)
- [`crates/hickory-cli/src/serve/agent_projects.rs`](implementation/crates/hickory-cli/src/serve/agent_projects.rs.md#output-570609cb95dd)
- [`crates/hickory-cli/src/workspace_capture.rs`](implementation/crates/hickory-cli/src/workspace_capture.rs.md#output-0514dd070fee)
<storycode:include file="implementation/apps/web/src/components/ChangesetDocument.tsx.md" />
<storycode:include file="implementation/crates/hickory-cli/src/changeset.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/changeset_mcp.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/serve/acp/client.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/serve/acp/mcp.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/serve/agent_projects.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/workspace_capture.rs.md" />

## Implementation review

### Implementation

- [apps/web/src/components/ChangesetDocument.tsx](implementation/apps/web/src/components/ChangesetDocument.tsx.md#output-60692076babd)
- [apps/web/src/elements/patch/Editor.tsx](../../../mechanisms/apps/web/src/elements/patch/Editor.tsx.md#output-dc5641c063e3)
- [crates/hickory-cli/src/changeset.rs](implementation/crates/hickory-cli/src/changeset.rs.md#output-ba18a7aa08b0)
- [crates/hickory-cli/src/changeset_mcp.rs](implementation/crates/hickory-cli/src/changeset_mcp.rs.md#output-fd880fd957c2)
- [crates/hickory-cli/src/serve/acp/client.rs](implementation/crates/hickory-cli/src/serve/acp/client.rs.md#output-2662a693b125)
- [crates/hickory-cli/src/serve/acp/edits.rs](../../../mechanisms/crates/hickory-cli/src/serve/acp/edits.rs.md#output-076ee922e67a)
- [crates/hickory-cli/src/serve/acp/mcp.rs](implementation/crates/hickory-cli/src/serve/acp/mcp.rs.md#output-89fc1623f7e6)
- [crates/hickory-cli/src/serve/acp/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/acp/mod.rs.md#output-fa77fc3f176f)
- [crates/hickory-cli/src/serve/agent_projects.rs](implementation/crates/hickory-cli/src/serve/agent_projects.rs.md#output-570609cb95dd)
- [crates/hickory-cli/src/workspace_capture.rs](implementation/crates/hickory-cli/src/workspace_capture.rs.md#output-0514dd070fee)

### Tests and fixtures

- [apps/web/e2e/workspace-changesets.spec.ts](../../../mechanisms/apps/web/e2e/workspace-changesets.spec.ts.md#output-b472b368c11a)
- [apps/web/src/components/ChangesetDocument.test.tsx](../../../mechanisms/apps/web/src/components/ChangesetDocument.test.tsx.md#output-9ae884d72abd)
- [crates/hickory-cli/tests/agent_changesets.rs](../../../mechanisms/crates/hickory-cli/tests/agent_changesets.rs.md#output-35273396ec57)
- [crates/hickory-cli/tests/byo_agent_surface.rs](../../../mechanisms/crates/hickory-cli/tests/byo_agent_surface.rs.md#output-43da4a5da7c9)
- [crates/hickory-cli/tests/literate_agent_projects.rs](../../../mechanisms/crates/hickory-cli/tests/literate_agent_projects.rs.md#output-c0298ec2aa35)
- [crates/hickory-cli/tests/python_agent.rs](../../../mechanisms/crates/hickory-cli/tests/python_agent.rs.md#output-9bda4649fd61)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
