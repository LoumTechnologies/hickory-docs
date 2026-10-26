<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Stepping A Table Replays The Order The Host Chose

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a table whose formulas can be evaluated, when the author steps through
them, then they see every formula cell's turn in the order it actually ran —
what the cell read, what those references were worth at that moment, which
batch it went out in, and what it came to — and those steps are the same
evaluation the grid is showing, not a second one.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/stepping-a-table-replays-the-order-the-host-chose.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/api/client.ts](../../../mechanisms/apps/web/src/api/client.ts.md#output-1e909407d4cd)
- [apps/web/src/components/FormulaDebugger.tsx](../../../mechanisms/apps/web/src/components/FormulaDebugger.tsx.md#output-3a3b3fc26b23)
- [apps/web/src/components/TablePanel.tsx](../../../mechanisms/apps/web/src/components/TablePanel.tsx.md#output-38d5395d7736)
- [apps/web/src/styles.css](../../../mechanisms/apps/web/src/styles.css.md#output-8695905e2358)
- [crates/hick-formula/src/evaluate.rs](../../../mechanisms/crates/hick-formula/src/evaluate.rs.md#output-61e8dc986852)
- [crates/hickory-cli/src/serve/formula.rs](../../../mechanisms/crates/hickory-cli/src/serve/formula.rs.md#output-c63d0db349cf)

### Tests and fixtures

- [apps/web/src/components/FormulaDebugger.test.tsx](../../../mechanisms/apps/web/src/components/FormulaDebugger.tsx.md#output-1acac206a244)
- [apps/web/src/components/TablePanel.test.tsx](../../../mechanisms/apps/web/src/components/TablePanel.test.tsx.md#output-694220ace684)
- [crates/hick-formula/tests/backends.rs](../../../mechanisms/crates/hick-formula/tests/backends.rs.md#output-6d63eb1fec85)
- [crates/hickory-cli/tests/serve_formula.rs](../../../mechanisms/crates/hickory-cli/tests/serve_formula.rs.md#output-6af46f552231)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
