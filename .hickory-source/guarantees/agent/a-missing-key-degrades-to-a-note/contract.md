<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Missing Key Degrades to a Note, Never a Crash

For: Authors using an agent to change documents while retaining review control.

Given a local session (the desktop app's server) on a machine with no LLM
provider key in the environment, when the user sends a prompt to the in-app
agent, then `POST /api/docs/:id/agent` answers `503` with an error message
that **starts with** `agent not available` and goes on to name every
environment variable that would fix it — and nothing else about the session
degrades: documents still open, edit, weave, and run.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/a-missing-key-degrades-to-a-note.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `DOC`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_agent.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-d24aa77e0987-DOC"></a>
### `DOC`

The in-app agent, driven the way the desktop app's chat dock drives it:
`POST /api/docs/:id/agent`, a WebSocket on the run channel, and
`GET /api/docs/:id/agent/turns`.
Protects docs/guarantees/agent/a-missing-key-degrades-to-a-note.md.
No test here spends a token: the full-turn tests inject a
[`hickory_agent::ScriptedLlmClient`] through the serve state's test seam,
and the no-key test scrubs the provider variables from this test binary's
environment. The other tests never read those variables (the scripted
client bypasses provider resolution), so the scrub cannot race them.

<storycode:copy id="f-d24aa77e0987-DOC">
//! The in-app agent, driven the way the desktop app's chat dock drives it:
//! `POST /api/docs/:id/agent`, a WebSocket on the run channel, and
//! `GET /api/docs/:id/agent/turns`.
//!
//! Protects docs/guarantees/agent/a-missing-key-degrades-to-a-note.md.
//!
//! No test here spends a token: the full-turn tests inject a
//! [`hickory_agent::ScriptedLlmClient`] through the serve state's test seam,
//! and the no-key test scrubs the provider variables from this test binary's
//! environment. The other tests never read those variables (the scripted
//! client bypasses provider resolution), so the scrub cannot race them.

use std::sync::Arc;

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="reading.md">
# Demo

<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
</hick:doc>
"##;

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/components/ChatDock.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-57557cac78c8)
- [crates/hickory-agent/src/provider.rs](../../../mechanisms/crates/hickory-agent/src/provider.rs.md#output-2be2d28e151b)
- [crates/hickory-cli/src/serve/agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/agent.rs.md#output-2ce16640df92)

### Tests and fixtures

- [crates/hickory-cli/tests/serve_agent.rs](../../../mechanisms/crates/hickory-cli/tests/serve_agent.rs.md#output-d24aa77e0987)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
