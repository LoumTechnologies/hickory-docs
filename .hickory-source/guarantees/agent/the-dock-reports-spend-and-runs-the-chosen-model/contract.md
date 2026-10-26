<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Dock Reports Spend and Runs the Chosen Model

For: Authors using an agent to change documents while retaining review control.

Given a document's chat dock, when the user picks a provider and/or types a
model id and sends a prompt, then `POST /api/docs/:id/agent` runs that turn
on exactly that provider and model (an unknown provider selector is refused
with `400` naming the valid set — it never silently runs on another vendor),
the choice persists per document for the session and applies to subsequent
turns, and each turn records the provider and model it actually ran on so a
mid-conversation change stays visible per turn.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/the-dock-reports-spend-and-runs-the-chosen-model.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/components/ChatDock.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-57557cac78c8)
- [crates/hickory-cli/src/serve/agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/agent.rs.md#output-2ce16640df92)

### Tests and fixtures

- [apps/web/src/components/ChatDock.test.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-e7bd6d898a44)
- [crates/hickory-cli/tests/serve_agent.rs](../../../mechanisms/crates/hickory-cli/tests/serve_agent.rs.md#output-d24aa77e0987)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
