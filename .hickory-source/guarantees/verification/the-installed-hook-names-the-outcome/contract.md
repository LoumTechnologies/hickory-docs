<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Pre-Commit Hook `hick init` Installs Names Which Outcome Blocked The Commit

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given a repository set up by `hick init`, when the installed pre-commit
hook blocks a commit because `hick test` exited non-zero, then the message
names the specific outcome that occurred — drift (exit `1`), not verified
(exit `2`), or a failed expectation (exit `3`) — and states the fix for that
outcome and only that outcome.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/the-installed-hook-names-the-outcome.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `hook_checks_document_names_with_spaces_and_newlines`

This is the maintained test fragment from `crates/hickory-cli/tests/init_tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-7418ab26c01b-hook_checks_document_names_with_spaces_and_newlines"></a>
### `hook_checks_document_names_with_spaces_and_newlines`

Protects docs/guarantees/verification/the-installed-hook-names-the-outcome.md.

<storycode:copy id="f-7418ab26c01b-hook_checks_document_names_with_spaces_and_newlines">
// Protects docs/guarantees/verification/the-installed-hook-names-the-outcome.md.
#[test]
fn hook_checks_document_names_with_spaces_and_newlines() {
    let repo = init_repo();
    run_init(repo.path());
    let name = if cfg!(windows) {
        "a note.md"
    } else {
        "a note\nwith a newline.md"
    };
    std::fs::write(repo.path().join(name), false_claim_doc()).unwrap();
    git(repo.path(), &["add", name]);
    let out = run_hook(repo.path());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "a false claim passed: {out:?}");
    assert!(stderr.contains("FAILED EXPECTATION (exit 3)"), "{stderr}");
}

</storycode:copy>

## Test the behavior: `hook_names_an_unrecorded_frozen_cell`

This is the maintained test fragment from `crates/hickory-cli/tests/init_tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-7418ab26c01b-hook_names_an_unrecorded_frozen_cell"></a>
### `hook_names_an_unrecorded_frozen_cell`

Protects docs/guarantees/verification/the-installed-hook-names-the-outcome.md.

<storycode:copy id="f-7418ab26c01b-hook_names_an_unrecorded_frozen_cell">
// Protects docs/guarantees/verification/the-installed-hook-names-the-outcome.md.
#[test]
fn hook_names_an_unrecorded_frozen_cell() {
    let repo = init_repo();
    run_init(repo.path());
    let source = passing_doc().replace(
        "<hick:exec container=\"c\">",
        "<hick:exec container=\"c\" freeze=\"true\">",
    );
    std::fs::write(repo.path().join("frozen.md"), source).unwrap();
    git(repo.path(), &["add", "frozen.md"]);
    let out = run_hook(repo.path());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "an unrecorded frozen cell passed: {out:?}"
    );
    assert!(stderr.contains("could NOT VERIFY (exit 2)"), "{stderr}");
}
</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-cli/src/init.rs](../../../mechanisms/crates/hickory-cli/src/init.rs.md#output-a329ec22d0b8)

### Tests and fixtures

- [crates/hickory-cli/tests/init_tests.rs](../../../mechanisms/crates/hickory-cli/tests/init_tests.rs.md#output-7418ab26c01b)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
