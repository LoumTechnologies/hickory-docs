<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A final changeset revision applies B to C

For: Authors using an agent to change documents while retaining review control.

Given a finalized agent artifact constructing accepted B from captured A,
when the user chooses Revise New, then the accepted closure and snapshots remain
immutable. A separately labeled authored draft permits New code edits with Old
fixed. Editing uses ordinary patch regeneration with captured A, re-evaluates the
entire supplied closure, and writes no destination. A downstream-breaking edit
remains unchecked and disables Apply. Discard removes the draft without writes.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/final-changeset-revisions-apply-b-to-c.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `accepted_process_result_b_can_be_revised_to_c_without_replaying_a_over_b`

This is the maintained test fragment from `crates/hickory-cli/tests/agent_changesets.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-35273396ec57-accepted_process_result_b_can_be_revised_to_c_without_replaying_a_over_b"></a>
### `accepted_process_result_b_can_be_revised_to_c_without_replaying_a_over_b`

<storycode:copy id="f-35273396ec57-accepted_process_result_b_can_be_revised_to_c_without_replaying_a_over_b">
// Guarantee: docs/guarantees/agent/final-changeset-revisions-apply-b-to-c.md
#[tokio::test]
async fn accepted_process_result_b_can_be_revised_to_c_without_replaying_a_over_b() {
    let host = host(true).await;
    let turn = run(&host, "contract-repair").await;
    assert_eq!(finished(&host, &turn).await["status"], "ok");
    let path = format!("/api/docs/{}/agent/projects/{turn}", host.doc);
    let (_, capture) = request(&host, &path, None).await;
    let (status, resolved) =
        request(&host, &path, Some(json!({"resolve":capture["ambiguous"]}))).await;
    assert_eq!(status, 200, "{resolved}");
    let (_, accepted) = request(&host, &path, None).await;
    let a: hick_lang::ProjectSnapshot =
        serde_json::from_value(accepted["starting"].clone()).unwrap();
    let b: hick_lang::ProjectSnapshot =
        serde_json::from_value(accepted["completed"].clone()).unwrap();
    let a = a.restore().unwrap();
    let b = b.restore().unwrap();
    assert!(!a.compare(&b).is_empty());
    let mut c = b.clone();
    c.insert(
        "app.ts",
        hick_lang::ProjectEntry::file(b"revision C\n".to_vec()),
    )
    .unwrap();
    let source = hick_lang::transition_response(&a, &c, "Authored revision").unwrap();
    let (status, result) = request(
        &host,
        &format!("{path}/draft/apply"),
        Some(json!({"sources":{"response.md":source},"context":{"buffers":[]}})),
    )
    .await;
    assert_eq!(status, 200, "{result}");
    assert_eq!(result["error"], Value::Null, "{result}");
    assert_eq!(
        std::fs::read_to_string(host.root.join("app.ts")).unwrap(),
        "revision C\n"
    );
    assert_eq!(request(&host, &path, None).await.1, accepted);
}
</storycode:copy>

## Test the behavior: `revision`

This is the maintained test fragment from `crates/hickory-cli/tests/literate_agent_projects.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-c0298ec2aa35-revision"></a>
### `revision`

<storycode:copy id="f-c0298ec2aa35-revision">
// Guarantee: docs/guarantees/agent/final-changeset-revisions-apply-b-to-c.md
async fn revision(host: &Host, turn: &str, changes: &[(&str, &str)]) -> (String, Value) {
    let path = format!("/api/docs/{}/agent/projects/{turn}", host.doc);
    let (_, capture) = request(host, &path, None).await;
    let starting: hick_lang::ProjectSnapshot =
        serde_json::from_value(capture["starting"].clone()).unwrap();
    let completed: hick_lang::ProjectSnapshot =
        serde_json::from_value(capture["completed"].clone()).unwrap();
    let a = starting.restore().unwrap();
    let mut c = completed.restore().unwrap();
    for (name, text) in changes {
        let mut entry = c
            .get(name)
            .map(|e| (**e).clone())
            .unwrap_or_else(|| hick_lang::ProjectEntry::file(Vec::new()));
        entry.bytes = text.as_bytes().to_vec().into();
        entry.origins.clear();
        c.insert(name, entry).unwrap();
    }
    let response = hick_lang::transition_response(&a, &c, "Authored revision").unwrap();
    (
        format!("{path}/draft/apply"),
        json!({"sources":{"response.md":response},"context":{"buffers":[]}}),
    )
}
</storycode:copy>

## Test the behavior: `final_revision_preserves_the_accepted_executable_mode`

This is the maintained test fragment from `crates/hickory-cli/tests/literate_agent_projects.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-c0298ec2aa35-final_revision_preserves_the_accepted_executable_mode"></a>
### `final_revision_preserves_the_accepted_executable_mode`

<storycode:copy id="f-c0298ec2aa35-final_revision_preserves_the_accepted_executable_mode">
// Guarantee: docs/guarantees/agent/final-changeset-revisions-apply-b-to-c.md
#[cfg(unix)]
#[tokio::test]
async fn final_revision_preserves_the_accepted_executable_mode() {
    use std::os::unix::fs::PermissionsExt;
    let host = host().await;
    std::fs::set_permissions(
        host.root.join("app.ts"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let turn = finalized(&host).await;
    let (path, body) = revision(&host, &turn, &[("app.ts", "C\n")]).await;
    let (status, result) = request(&host, &path, Some(body)).await;
    assert_eq!(status, 200, "{result}");
    assert_eq!(result["error"], Value::Null, "{result}");
    assert_eq!(
        std::fs::metadata(host.root.join("app.ts"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o755
    );
}
</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-cli/src/serve/changeset_drafts.rs](../../../mechanisms/crates/hickory-cli/src/serve/changeset_drafts.rs.md#output-21353eb88ac0)

### Tests and fixtures

- [apps/web/e2e/workspace-changesets.spec.ts](../../../mechanisms/apps/web/e2e/workspace-changesets.spec.ts.md#output-b472b368c11a)
- [apps/web/src/components/ChangesetDocument.test.tsx](../../../mechanisms/apps/web/src/components/ChangesetDocument.test.tsx.md#output-9ae884d72abd)
- [apps/web/src/lib/agentEdit.test.ts](../../../mechanisms/apps/web/src/lib/agentEdit.ts.md#output-ebba6809ee8c)
- [crates/hickory-cli/tests/agent_changesets.rs](../../../mechanisms/crates/hickory-cli/tests/agent_changesets.rs.md#output-35273396ec57)
- [crates/hickory-cli/tests/literate_agent_projects.rs](../../../mechanisms/crates/hickory-cli/tests/literate_agent_projects.rs.md#output-c0298ec2aa35)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
