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

## Choose subscription-backed Codex

In the built-in Hickory Python agent, the provider selector offers **Codex —
ChatGPT subscription** alongside API-key providers. Codex manages sign-in and
reuses its existing ChatGPT session. Hickory never reads credential files or
extracts tokens. An API-key Codex login does not qualify. Authentication, network
and usage-limit failures remain on this backend; Hickory never switches to API
billing or purchases credits. Reopening a retained subscription conversation
keeps this provider; an empty provider field cannot switch it to API billing.

To use it on macOS with Codex CLI 0.166 or later installed, open the Agent pane's
**Agent settings**, choose **Hickory Python** as the agent and **Codex — ChatGPT
subscription** as the provider. If sign-in is ready, keep using that Codex login.
Otherwise choose **Sign in with ChatGPT**, follow **Continue ChatGPT sign-in**,
complete the browser flow and choose **Check sign-in**. Leave **Model** empty for
the default, then send a request and review edits normally.

The transport supplies model text to the existing Python loop. Hickory's editing,
review, tool allowlist, cancellation and ordinary prediction gates remain in that
loop. Native execution and integration tools are disabled; other native tool items are
refused. Process confinement blocks filesystem
content outside the selected runtime, credentials and scratch. Private prediction
inference and captured image input are refused until supported safely. The first
implementation requires macOS sandbox-exec and Codex CLI 0.166 or later.

Reported token totals are subscription observations, potentially partial, rather
than API-price estimates. Missing counters, balances and credit costs are unknown.
Cache-write and reasoning counters are not exposed by this CLI event contract.

Selection and failure fixtures live with the [transport construction](../../../mechanisms/crates/hickory-cli/src/serve/codex_subscription.rs.md)
and [sign-in controls](../../../mechanisms/apps/web/src/components/ConversationModelChoice.test.tsx.md).
Live sign-in, real account entitlement, exhausted limits and consumption of existing
credits still require live verification; fixture success does not establish them.

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
