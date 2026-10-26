<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Check the agent's transition separately from maintained-source reconstruction

For: Maintainers reorganizing source while preserving the complete generated program.

An agent finishes B from captured A. Its explanation must reproduce that transition, while the maintained source must reproduce B from an empty tree.

## What must happen

Finalization checks apply(explanation, A) equals B. Incorporation separately checks generate(maintained source, empty tree) equals B. Both proposed and reviewed candidates are rechecked before publication, including destination source identity and completed code. Already-present B is not applied a second time. Removing source retires its live rooms so late persistence cannot recreate it. Captured editor and dirty-source identities remain part of the boundary.

## Limits of the evidence

Observed filesystem snapshots are not global transactions. Equal source trees do not prove that the explanation is accurate or that B behaves correctly.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/finalization-and-incorporation-check-distinct-equalities.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Use the implemented built-in/Python/ACP capture and review-queue continuation. Remove obsolete pending-integration language while retaining the distinct equalities.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-bf6f560e06c6"></a>
## `crates/hick-lang/src/project/reconstruct.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

Related contracts: [Check the agent's transition separately from maintained-source reconstruction](contract.md), [Keep a patch edit as a draft until the whole construction still works](../patch-edits-revalidate-downstream-construction/contract.md).

<a id="f-bf6f560e06c6-transition_response"></a>
### `transition_response`

<storycode:copy id="f-bf6f560e06c6-transition_response">
use super::{EntryKind, ProjectDifference, ProjectTree, evaluate_project};
use std::collections::{BTreeMap, BTreeSet};

/// Deterministic textual reconstruction of a successful A-to-B transition.
/// Kept apart from the actual session. Metadata/binary transitions refuse by
/// name until an exact supported transition operation exists for them.
pub fn transition_response(
    before: &ProjectTree,
    after: &ProjectTree,
    explanation: &str,
) -> Result<String, String> {
    let diff = unified_transition(before, after)?;
    let mut prefix = "hick".to_owned();
    let mut number = 0;
    while explanation.contains(&format!("<{prefix}:"))
        || explanation.contains(&format!("</{prefix}:"))
        || diff.contains(&format!("</{prefix}:patch>"))
    {
        number += 1;
        prefix = format!("hick{number}");
    }
    let patch = if diff.is_empty() {
        String::new()
    } else {
        format!("\n<{prefix}:patch id=\"completed\">\n{diff}</{prefix}:patch>\n")
    };
    let response = format!(
        "<{prefix}:doc xmlns:{prefix}=\"http://www.hickorydocs.com/1.0\">\n{explanation}\n\n<{prefix}:project/>\n{patch}</{prefix}:doc>\n"
    );
    let mut entry = "response.md".to_owned();
    while before.get(&entry).is_some() || after.get(&entry).is_some() {
        entry.insert(0, '_');
    }
    let sources = BTreeMap::from([(entry.clone(), response.clone())]);
    let evaluated = evaluate_project(&entry, &sources, before)?;
    if !evaluated.new.compare(after).is_empty() {
        return Err("reconstructed response did not reproduce the completed state".into());
    }
    Ok(response)
}
</storycode:copy>
<a id="f-bf6f560e06c6-unified_transition"></a>
### `unified_transition`

<storycode:copy id="f-bf6f560e06c6-unified_transition">
pub fn unified_transition(before: &ProjectTree, after: &ProjectTree) -> Result<String, String> {
    let paths: BTreeSet<_> = before
        .entries()
        .keys()
        .chain(after.entries().keys())
        .collect();
    let mut diff = String::new();
    for path in paths {
        let old = before.get(path);
        let new = after.get(path);
        if matches!((old,new),(Some(a),Some(b)) if a.same_output(b)) {
            continue;
        }
        for entry in [old, new].into_iter().flatten() {
            if entry.kind != EntryKind::File
                || entry.executable != old.is_some_and(|o| o.executable)
            {
                return Err(format!("unsupported textual transition metadata: {path}"));
            }
        }
        let old_text = old
            .map(|e| std::str::from_utf8(&e.bytes))
            .transpose()
            .map_err(|_| format!("unsupported binary transition: {path}"))?
            .unwrap_or("");
        let new_text = new
            .map(|e| std::str::from_utf8(&e.bytes))
            .transpose()
            .map_err(|_| format!("unsupported binary transition: {path}"))?
            .unwrap_or("");
        let old_lines: Vec<_> = old_text.split_inclusive('\n').collect();
        let new_lines: Vec<_> = new_text.split_inclusive('\n').collect();
        diff.push_str(&format!(
            "--- {}\n+++ {}\n@@ -{},{} +{},{} @@\n",
            if old.is_some() {
                format!("a/{path}")
            } else {
                "/dev/null".into()
            },
            if new.is_some() {
                format!("b/{path}")
            } else {
                "/dev/null".into()
            },
            usize::from(!old_lines.is_empty()),
            old_lines.len(),
            usize::from(!new_lines.is_empty()),
            new_lines.len()
        ));
        for (prefix, lines) in [('-', old_lines), ('+', new_lines)] {
            for line in lines {
                diff.push(prefix);
                diff.push_str(line);
                if !line.ends_with('\n') {
                    diff.push_str("\n\\ No newline at end of file\n");
                }
            }
        }
    }
    Ok(diff)
}

</storycode:copy>
<a id="f-bf6f560e06c6-IncorporationError"></a>
### `IncorporationError`

<storycode:copy id="f-bf6f560e06c6-IncorporationError">
#[derive(Debug, Clone)]
pub enum IncorporationError {
    Construction(String),
    Outputs(Vec<ProjectDifference>),
}

</storycode:copy>
<a id="f-bf6f560e06c6-IncorporationCandidate"></a>
### `IncorporationCandidate`

A reviewed candidate is pinned to both destination documents and completed
code. Validation is pure; writing it is the existing host review's job.

<storycode:copy id="f-bf6f560e06c6-IncorporationCandidate">
/// A reviewed candidate is pinned to both destination documents and completed
/// code. Validation is pure; writing it is the existing host review's job.
#[derive(Debug, Clone)]
pub struct IncorporationCandidate {
    entry: String,
    baseline_documents: BTreeMap<String, String>,
    documents: BTreeMap<String, String>,
    completed: ProjectTree,
}
</storycode:copy>
<a id="f-bf6f560e06c6-impl-IncorporationCandidate"></a>
### `impl-IncorporationCandidate`

<storycode:copy id="f-bf6f560e06c6-impl-IncorporationCandidate">
impl IncorporationCandidate {
    pub fn verify(
        entry: &str,
        baseline_documents: BTreeMap<String, String>,
        documents: BTreeMap<String, String>,
        completed: ProjectTree,
    ) -> Result<Self, IncorporationError> {
        let evaluated = evaluate_project(entry, &documents, &ProjectTree::default())
            .map_err(IncorporationError::Construction)?;
        if documents
            .keys()
            .any(|path| !evaluated.inventory.documents.contains(path))
        {
            return Err(IncorporationError::Construction(
                "candidate contains documents outside its declared Markdown closure".into(),
            ));
        }
        for path in &evaluated.inventory.read_only {
            if baseline_documents.get(path) != documents.get(path) {
                return Err(IncorporationError::Construction(format!(
                    "Upstream source is read-only: {path}"
                )));
            }
        }
        let generated = evaluated.new;
        let differences = generated.compare(&completed);
        if !differences.is_empty() {
            return Err(IncorporationError::Outputs(differences));
        }
        Ok(Self {
            entry: entry.into(),
            baseline_documents,
            documents,
            completed,
        })
    }
    pub fn entry(&self) -> &str {
        &self.entry
    }
    pub fn documents(&self) -> &BTreeMap<String, String> {
        &self.documents
    }
    pub fn completed(&self) -> &ProjectTree {
        &self.completed
    }
    pub fn recheck_destinations(
        &self,
        documents: &BTreeMap<String, String>,
        code: &ProjectTree,
    ) -> Result<(), String> {
        if &self.baseline_documents != documents {
            return Err(
                "literate destinations changed; generate and review a new candidate".into(),
            );
        }
        if !self.completed.compare(code).is_empty() {
            return Err("completed code changed; generate and review a new candidate".into());
        }
        Ok(())
    }
}

</storycode:copy>
<a id="f-bf6f560e06c6-AgentProjectCapture"></a>
### `AgentProjectCapture`

Shared capture contract for built-in, Python and ACP harnesses. Capturing
native writes establishes actual B, but cannot establish their authorship.

<storycode:copy id="f-bf6f560e06c6-AgentProjectCapture">
/// Shared capture contract for built-in, Python and ACP harnesses. Capturing
/// native writes establishes actual B, but cannot establish their authorship.
#[derive(Debug, Clone)]
pub struct AgentProjectCapture {
    pub starting: ProjectTree,
    ambiguous_paths: BTreeSet<String>,
    complete: bool,
}
</storycode:copy>
<a id="f-bf6f560e06c6-impl-AgentProjectCapture"></a>
### `impl-AgentProjectCapture`

<storycode:copy id="f-bf6f560e06c6-impl-AgentProjectCapture">
impl AgentProjectCapture {
    pub fn new(starting: ProjectTree) -> Self {
        Self {
            starting,
            ambiguous_paths: BTreeSet::new(),
            complete: true,
        }
    }
    pub fn incomplete(&mut self) {
        self.complete = false;
    }
    pub fn concurrent_write(&mut self, path: &str) {
        self.ambiguous_paths.insert(path.into());
    }
    pub fn resolve_concurrent(&mut self, path: &str) {
        self.ambiguous_paths.remove(path);
    }
    pub fn finalize(&self, completed: &ProjectTree, explanation: &str) -> Result<String, String> {
        if !self.complete {
            return Err(
                "incomplete agent project capture; finalization requires the actual output state"
                    .into(),
            );
        }
        if !self.ambiguous_paths.is_empty() {
            return Err(format!(
                "resolve concurrent writes before attributing the transition: {}",
                self.ambiguous_paths
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        transition_response(&self.starting, completed, explanation)
    }
}
</storycode:copy>

### Assemble `crates/hick-lang/src/project/reconstruct.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/src/project/reconstruct.rs"><storycode:paste select="#f-bf6f560e06c6-transition_response"/><storycode:paste select="#f-bf6f560e06c6-unified_transition"/><storycode:paste select="#f-bf6f560e06c6-IncorporationError"/><storycode:paste select="#f-bf6f560e06c6-IncorporationCandidate"/><storycode:paste select="#f-bf6f560e06c6-impl-IncorporationCandidate"/><storycode:paste select="#f-bf6f560e06c6-AgentProjectCapture"/><storycode:paste select="#f-bf6f560e06c6-impl-AgentProjectCapture"/></storycode:file>

## Test the behavior: `final_response_uses_dirty_start_and_incorporation_rechecks_destinations`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-final_response_uses_dirty_start_and_incorporation_rechecks_destinations"></a>
### `final_response_uses_dirty_start_and_incorporation_rechecks_destinations`

<storycode:copy id="f-005c1e9344c7-final_response_uses_dirty_start_and_incorporation_rechecks_destinations">
// Guarantee: docs/guarantees/literate-projects/finalization-and-incorporation-check-distinct-equalities.md
#[test]
fn final_response_uses_dirty_start_and_incorporation_rechecks_destinations() {
    let before = tree(&[
        ("app.ts", b"user's existing uncommitted work\n"),
        ("asset", &[0, 255]),
    ]);
    let completed = tree(&[
        ("app.ts", b"successful implementation\n"),
        ("asset", &[0, 255]),
        ("empty", b""),
    ]);
    let mut capture = AgentProjectCapture::new(before.clone());
    capture.concurrent_write("app.ts");
    assert!(
        capture
            .finalize(&completed, "The successful result.")
            .unwrap_err()
            .contains("concurrent")
    );
    capture.resolve_concurrent("app.ts");
    let response = capture
        .finalize(&completed, "The successful result.")
        .unwrap();
    let replay = evaluate_project(
        "response.md",
        &BTreeMap::from([("response.md".into(), response)]),
        &before,
    )
    .unwrap();
    assert!(replay.new.compare(&completed).is_empty());
    capture.incomplete();
    assert!(
        capture
            .finalize(&completed, "The result.")
            .unwrap_err()
            .contains("incomplete")
    );
    let baseline = BTreeMap::from([("project.md".into(), "previous explanation".into())]);
    let documents = BTreeMap::from([("project.md".into(),"<hick:project/><hick:file path=\"app.ts\">successful implementation\n</hick:file><hick:entry path=\"asset\" encoding=\"hex\">00ff</hick:entry><hick:file path=\"empty\"></hick:file>".into())]);
    let candidate = IncorporationCandidate::verify(
        "project.md",
        baseline.clone(),
        documents,
        completed.clone(),
    )
    .unwrap();
    assert!(
        candidate
            .recheck_destinations(&baseline, &completed)
            .is_ok()
    );
    assert!(
        candidate
            .recheck_destinations(&BTreeMap::new(), &completed)
            .is_err()
    );
    assert!(candidate.recheck_destinations(&baseline, &before).is_err());
    assert!(
        IncorporationCandidate::verify(
            "project.md",
            baseline,
            BTreeMap::from([("project.md".into(), "<hick:project/>".into())]),
            completed
        )
        .is_err()
    );
}
</storycode:copy>

## Test the behavior: `dirty_start_is_finalized_then_ai_source_is_verified_reviewed_and_published_without_reapplying_code`

This is the maintained test fragment from `crates/hickory-cli/tests/literate_agent_projects.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-c0298ec2aa35-dirty_start_is_finalized_then_ai_source_is_verified_reviewed_and_published_without_reapply"></a>
### `dirty_start_is_finalized_then_ai_source_is_verified_reviewed_and_published_without_reapplying_code`

<storycode:copy id="f-c0298ec2aa35-dirty_start_is_finalized_then_ai_source_is_verified_reviewed_and_published_without_reapply">
// Guarantee: docs/guarantees/literate-projects/finalization-and-incorporation-check-distinct-equalities.md
#[tokio::test]
async fn dirty_start_is_finalized_then_ai_source_is_verified_reviewed_and_published_without_reapplying_code()
 {
    let host = host().await;
    let turn = finalized(&host).await;
    let path = format!("/api/docs/{}/agent/projects/{turn}/incorporate", host.doc);
    let (status, value) = request(&host, &path, Some(json!({}))).await;
    assert_eq!(status, 200, "{value}");
    let review = until(
        &host,
        &format!("/api/docs/{}/agent/review", host.doc),
        |v| {
            v["edits"]["changes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["status"] == "pending")
        },
    )
    .await;
    let change = &review["edits"]["changes"][0];
    assert_eq!(
        std::fs::read_to_string(host.root.join("app.ts")).unwrap(),
        "dirty A\n"
    );
    let revised = format!(
        "Reviewed explanation.\n{}",
        change["newText"].as_str().unwrap()
    );
    let (status, value) = request(
        &host,
        &format!("/api/docs/{}/agent/review", host.doc),
        Some(json!({"id":change["id"],"accepted":true,"new_text":revised})),
    )
    .await;
    assert_eq!(status, 200, "{value}");
    let result = until(&host, &path, |v| {
        matches!(v["status"].as_str(), Some("applied" | "failed"))
    })
    .await;
    assert_eq!(result["status"], "applied", "{result}");
    assert_eq!(
        std::fs::read_to_string(host.root.join("project.md")).unwrap(),
        revised
    );
    assert_eq!(
        std::fs::read_to_string(host.root.join("app.ts")).unwrap(),
        "dirty A\n"
    );
    let log = std::fs::read_dir(host.root.join("sessions"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert!(
        std::fs::read_to_string(log)
            .unwrap()
            .contains("literate-incorporation-applied")
    );
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hick-lang/src/project/reconstruct.rs](contract.md#output-bf6f560e06c6)
- [crates/hickory-cli/src/serve/agent_projects.rs](../../agent/changeset-completion-is-negotiated-and-checked/implementation/crates/hickory-cli/src/serve/agent_projects.rs.md#output-570609cb95dd)
- [crates/hickory-cli/src/serve/project_incorporation.rs](../../../mechanisms/crates/hickory-cli/src/serve/project_incorporation.rs.md#output-37c249049c6e)
- [crates/hickory-cli/src/serve/project_publish.rs](../../../mechanisms/crates/hickory-cli/src/serve/project_publish.rs.md#output-4faa99fa96cf)

### Tests and fixtures

- [crates/hick-lang/tests/project.rs](../../../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7)
- [crates/hickory-cli/tests/literate_agent_projects.rs](../../../mechanisms/crates/hickory-cli/tests/literate_agent_projects.rs.md#output-c0298ec2aa35)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
