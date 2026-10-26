<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Bisect keeps candidates isolated

For: Maintainers committing source and checking the staged project.

Given known good and bad commits, visual bisect uses Git's search in a private
worktree. Each candidate receives a fresh detached inspection worktree. Starting,
classifying, skipping, restoring, and ending never switch the original branch or
alter its tracked worktree or index. Earlier candidate windows remain at their
own commits when the search advances.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/git/bisect-keeps-candidates-isolated.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/views/BisectPane.tsx](../../../mechanisms/apps/web/src/views/BisectPane.tsx.md#output-73f5ba1f30b7)
- [crates/hickory-cli/src/serve/bisect.rs](../../../mechanisms/crates/hickory-cli/src/serve/bisect.rs.md#output-85364c468167)
- [crates/hickory-cli/src/up/mod.rs](../../../mechanisms/crates/hickory-cli/src/up/mod.rs.md#output-73148a3ab9db)

### Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../mechanisms/apps/web/e2e/literate-editor.spec.ts.md#output-648693644a28)
- [crates/hickory-cli/tests/literate_views.rs](../../../mechanisms/crates/hickory-cli/tests/literate_views.rs.md#output-3df513c37b3c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
