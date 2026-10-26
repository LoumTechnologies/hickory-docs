<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Verify Covers the Whole Editable Set

For: Authors using an agent to change documents while retaining review control.

Given an agent session whose primary document declares `hick:upstream` edges,
when the agent runs `verify`, then every document in that closure is executed
and its output files are written **in its own directory** — and the result says
how many upstream documents were re-woven. A `PASS` means the whole chain is
consistent, not just the document in front of the agent.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/verify-covers-the-whole-editable-set.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-2d9aacc8fb8b"></a>
## `crates/hick-literate/src/run_transcripts.rs`

Related contracts: [Verify Covers the Whole Editable Set](contract.md).

<a id="f-2d9aacc8fb8b-Scope"></a>
### `Scope`

A reused executor's prior runs are not this pipeline's runtime evidence.

<storycode:copy id="f-2d9aacc8fb8b-Scope">
//! A reused executor's prior runs are not this pipeline's runtime evidence.
use hickory_executor::{Executor, Transcripts};
use std::collections::HashMap;

pub(crate) struct Scope {
    previous: HashMap<String, usize>,
}
</storycode:copy>
<a id="f-2d9aacc8fb8b-impl-Scope"></a>
### `impl-Scope`

<storycode:copy id="f-2d9aacc8fb8b-impl-Scope">
impl Scope {
    pub fn capture(executor: &dyn Executor) -> Self {
        Self {
            previous: executor
                .transcripts()
                .into_iter()
                .map(|(name, entries)| (name, entries.len()))
                .collect(),
        }
    }

    /// Executor transcripts append for the lifetime of a run. Like execution
    /// itself, this requires a single caller per executor; it does not isolate
    /// concurrent pipelines or reset container files/process state.
    pub fn current_lines(
        &self,
        executor: &dyn Executor,
        lines: &HashMap<String, Vec<usize>>,
    ) -> Transcripts {
        let mut transcripts = executor.transcripts();
        for (name, entries) in &mut transcripts {
            let old = self
                .previous
                .get(name)
                .copied()
                .unwrap_or(0)
                .min(entries.len());
            entries.drain(..old);
            if let Some(lines) = lines.get(name) {
                for (entry, line) in entries.iter_mut().zip(lines) {
                    entry.source_line = Some(*line);
                }
            }
        }
        transcripts.retain(|_, entries| !entries.is_empty());
        transcripts
    }
}
</storycode:copy>

### Assemble `crates/hick-literate/src/run_transcripts.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/run_transcripts.rs"><storycode:paste select="#f-2d9aacc8fb8b-Scope"/><storycode:paste select="#f-2d9aacc8fb8b-impl-Scope"/></storycode:file>

<a id="output-8535390e9d97"></a>
## `crates/hickory-agent/tests/verify_publication.rs`

Related contracts: [Verify Covers the Whole Editable Set](contract.md).

<a id="f-8535390e9d97-document"></a>
### `document`

<storycode:copy id="f-8535390e9d97-document">
#![cfg(unix)]
//! Focused real-executor checks for provisional versus published verification.
use hickory_agent::{EditSession, ToolInvocation, execute_tool};
use hickory_executor::LocalExecutor;
use std::{path::Path, sync::Arc};

fn document(output: &str, value: &str, expected: &str) -> String {
    format!(
        r#"<hick:container name="c" image="alpine" />
<hick:file path="{output}"><hick:exec container="c" show="output">printf {value}<hick:expect>{expected}</hick:expect></hick:exec></hick:file>"#
    )
}

</storycode:copy>
<a id="f-8535390e9d97-verify"></a>
### `verify`

<storycode:copy id="f-8535390e9d97-verify">
async fn verify(path: &Path, guarded: bool) -> hickory_agent::ToolOutcome {
    let mut session = EditSession::open(path, &[]).await.unwrap();
    if guarded {
        session = session.require_verified_publication();
    }
    execute_tool(
        &mut session,
        Arc::new(LocalExecutor::new().unwrap()),
        &ToolInvocation::synthetic("verify", vec![], None).unwrap(),
    )
    .await
}

</storycode:copy>
<a id="f-8535390e9d97-failed_outputs_stay_provisional_until_unchanged_expectation_passes"></a>
### `failed_outputs_stay_provisional_until_unchanged_expectation_passes`

<storycode:copy id="f-8535390e9d97-failed_outputs_stay_provisional_until_unchanged_expectation_passes">
// Guarantee: docs/guarantees/agent/verify-covers-the-whole-editable-set.md
#[tokio::test]
async fn failed_outputs_stay_provisional_until_unchanged_expectation_passes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("doc.md");
    let output = dir.path().join("out.txt");
    std::fs::write(&output, "accepted earlier").unwrap();
    std::fs::write(&path, document("out.txt", "160", "100")).unwrap();
    let failed = verify(&path, true).await;
    assert!(!failed.ok, "{}", failed.text);
    assert!(failed.text.contains("no verified outputs published"));
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "accepted earlier"
    );
    std::fs::remove_file(&output).unwrap();
    assert!(!verify(&path, true).await.ok);
    assert!(
        !output.exists(),
        "a failed first run cannot create an accepted artifact"
    );
    std::fs::write(&path, document("out.txt", "100", "100")).unwrap();
    let passed = verify(&path, true).await;
    assert!(passed.ok, "{}", passed.text);
    assert!(std::fs::read_to_string(output).unwrap().contains("100"));
}

</storycode:copy>
<a id="f-8535390e9d97-upstream_failure_blocks_primary_and_upstream_publication"></a>
### `upstream_failure_blocks_primary_and_upstream_publication`

<storycode:copy id="f-8535390e9d97-upstream_failure_blocks_primary_and_upstream_publication">
// Guarantee: docs/guarantees/agent/verify-covers-the-whole-editable-set.md
#[tokio::test]
async fn upstream_failure_blocks_primary_and_upstream_publication() {
    let dir = tempfile::tempdir().unwrap();
    let upstream = dir.path().join("up");
    let primary = dir.path().join("primary");
    std::fs::create_dir_all(&upstream).unwrap();
    std::fs::create_dir_all(&primary).unwrap();
    std::fs::write(upstream.join("up.md"), document("up.txt", "160", "41")).unwrap();
    let path = primary.join("doc.md");
    std::fs::write(
        &path,
        format!(
            "<hick:upstream file=\"../up/up.md\" />\n{}",
            document("out.txt", "100", "100")
        ),
    )
    .unwrap();
    let failed = verify(&path, true).await;
    assert!(!failed.ok, "{}", failed.text);
    assert!(!upstream.join("up.txt").exists());
    assert!(!primary.join("out.txt").exists());
    std::fs::write(upstream.join("up.md"), document("up.txt", "41", "41")).unwrap();
    let passed = verify(&path, true).await;
    assert!(passed.ok, "{}", passed.text);
    assert!(upstream.join("up.txt").is_file());
    assert!(primary.join("out.txt").is_file());
    assert!(
        !std::fs::read_to_string(primary.join("out.txt"))
            .unwrap()
            .contains("41")
    );
}

</storycode:copy>
<a id="f-8535390e9d97-ordinary_verification_still_publishes_observed_output_on_mismatch"></a>
### `ordinary_verification_still_publishes_observed_output_on_mismatch`

<storycode:copy id="f-8535390e9d97-ordinary_verification_still_publishes_observed_output_on_mismatch">
#[tokio::test]
async fn ordinary_verification_still_publishes_observed_output_on_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("doc.md");
    std::fs::write(&path, document("out.txt", "160", "100")).unwrap();
    let failed = verify(&path, false).await;
    assert!(!failed.ok, "{}", failed.text);
    assert!(
        std::fs::read_to_string(dir.path().join("out.txt"))
            .unwrap()
            .contains("160")
    );
}

</storycode:copy>
<a id="f-8535390e9d97-source_changes_during_execution_cannot_publish_a_stale_verification"></a>
### `source_changes_during_execution_cannot_publish_a_stale_verification`

<storycode:copy id="f-8535390e9d97-source_changes_during_execution_cannot_publish_a_stale_verification">
// Guarantee: docs/guarantees/agent/verify-covers-the-whole-editable-set.md
#[cfg(unix)]
#[tokio::test]
async fn source_changes_during_execution_cannot_publish_a_stale_verification() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("doc.md");
    let quoted = path.to_string_lossy().replace('\'', "'\"'\"'");
    let text = document("out.txt", "100", "100").replace(
        "printf 100",
        &format!("printf changed > '{quoted}'; printf 100"),
    );
    std::fs::write(&path, text).unwrap();
    let refused = verify(&path, true).await;
    assert!(!refused.ok, "{}", refused.text);
    assert!(refused.text.contains("Source closure changed"));
    assert!(!dir.path().join("out.txt").exists());
}

</storycode:copy>
<a id="f-8535390e9d97-publication_refuses_existing_symlink_without_modifying_its_target"></a>
### `publication_refuses_existing_symlink_without_modifying_its_target`

<storycode:copy id="f-8535390e9d97-publication_refuses_existing_symlink_without_modifying_its_target">
// Guarantee: docs/guarantees/agent/verify-covers-the-whole-editable-set.md
#[cfg(unix)]
#[tokio::test]
async fn publication_refuses_existing_symlink_without_modifying_its_target() {
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let target = outside.path().join("secret");
    std::fs::write(&target, "preserve").unwrap();
    std::os::unix::fs::symlink(&target, dir.path().join("out.txt")).unwrap();
    let path = dir.path().join("doc.md");
    std::fs::write(&path, document("out.txt", "100", "100")).unwrap();
    let refused = verify(&path, true).await;
    assert!(!refused.ok, "{}", refused.text);
    assert_eq!(std::fs::read_to_string(target).unwrap(), "preserve");
}
</storycode:copy>

### Assemble `crates/hickory-agent/tests/verify_publication.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/tests/verify_publication.rs"><storycode:paste select="#f-8535390e9d97-document"/><storycode:paste select="#f-8535390e9d97-verify"/><storycode:paste select="#f-8535390e9d97-failed_outputs_stay_provisional_until_unchanged_expectation_passes"/><storycode:paste select="#f-8535390e9d97-upstream_failure_blocks_primary_and_upstream_publication"/><storycode:paste select="#f-8535390e9d97-ordinary_verification_still_publishes_observed_output_on_mismatch"/><storycode:paste select="#f-8535390e9d97-source_changes_during_execution_cannot_publish_a_stale_verification"/><storycode:paste select="#f-8535390e9d97-publication_refuses_existing_symlink_without_modifying_its_target"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-literate/src/run_transcripts.rs](contract.md#output-2d9aacc8fb8b)
- [crates/hick-project-env/src/tree.rs](../../../mechanisms/crates/hick-project-env/src/tree.rs.md#output-c594be7b78fc)
- [crates/hickory-agent/src/tools/mod.rs](../../../mechanisms/crates/hickory-agent/src/tools/mod.rs.md#output-54c7fa39e3da)
- [crates/hickory-agent/src/tools/verify.rs](../../../mechanisms/crates/hickory-agent/src/tools/verify.rs.md#output-913f8d4d5670)
- [crates/hickory-agent/src/tools/verify_publish.rs](../../../mechanisms/crates/hickory-agent/src/tools/verify_publish.rs.md#output-4f425a9f3483)

### Tests and fixtures

- [crates/hickory-agent/tests/verify_publication.rs](contract.md#output-8535390e9d97)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</storycode:doc>
