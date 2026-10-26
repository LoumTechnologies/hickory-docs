<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Persist local document edits in ordinary files

For: Authors switching between the desktop editor, their other editor, and Git.

After a desktop edit is persisted, reading the Markdown file from another editor or git diff must show that edit without an export step.

## What must happen

Given a local document room, persisted source edits land in the Markdown file on that machine. The room uses the shared hickory-collab machinery and file-backed store. Document writes use a sibling temporary file and rename; closing the last socket requests a final persist. The CRDT sidecar supports room reconstruction and is not a replacement for the source file. The local service boundary and optional authenticated peer transport have separate contracts.


## Limits of the evidence

A local document store does not imply that optional fleet transport is absent. Socket-close ordering, crash behavior and frontend acceptance require their own witnesses; this rewrite does not rerun the HTTP suite.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/local-session-writes-your-files.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Remove the global no-sharing/no-relay claim from the local-persistence contract. Optional peer transport remains governed by its existing grant and disclosure guarantees.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Test the behavior: `DOC`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_local.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-2b0b4fa19188-DOC"></a>
### `DOC`

The local document server, over a real socket.
Protects docs/guarantees/collaboration/local-session-writes-your-files.md.
These drive the session the way the desktop app's window does — the same
Yjs protocol frames, the same REST calls — because the claim worth testing
is "an edit made in the app's editor reaches the file on disk", and that is
not observable from inside a unit test of a handler.
There is nothing here about tokens, scopes, or share links: this server
binds loopback and answers one person. See
`docs/specs/freeform/local-only.md`.

<storycode:copy id="f-2b0b4fa19188-DOC">
//! The local document server, over a real socket.
//!
//! Protects docs/guarantees/collaboration/local-session-writes-your-files.md.
//!
//! These drive the session the way the desktop app's window does — the same
//! Yjs protocol frames, the same REST calls — because the claim worth testing
//! is "an edit made in the app's editor reaches the file on disk", and that is
//! not observable from inside a unit test of a handler.
//!
//! There is nothing here about tokens, scopes, or share links: this server
//! binds loopback and answers one person. See
//! `docs/specs/freeform/local-only.md`.

use std::path::PathBuf;
use std::time::Duration;

use futures::{SinkExt as _, StreamExt as _};
use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message as TtMessage;
use yrs::sync::{Message, SyncMessage};
use yrs::updates::decoder::Decode as _;
use yrs::updates::encoder::Encode as _;
use yrs::{GetString as _, ReadTxn as _, Text as _, Transact as _, Update};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="demo.md">
# Demo

<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
</hick:doc>
"##;

</storycode:copy>

## Implementation review

The shared room coalesces changes and asks FileDocStore to persist them. The socket-close path flushes before dropping the room. The directory lock prevents hick up and the desktop process from independently owning the same directory.

### Implementation

- [crates/hickory-cli/src/serve/socket.rs](../../../mechanisms/crates/hickory-cli/src/serve/socket.rs.md#output-ce0f0e119d3c)
- [crates/hickory-cli/src/serve/store.rs](../../../mechanisms/crates/hickory-cli/src/serve/store.rs.md#output-60e6f71bfd5c)
- [crates/hickory-collab/src/lib.rs](../../../mechanisms/crates/hickory-collab/src/lib.rs.md#output-08bf73f4dffb)

### Tests and fixtures

- [crates/hickory-cli/tests/serve_local.rs](../../../mechanisms/crates/hickory-cli/tests/serve_local.rs.md#output-2b0b4fa19188)

### Focused witnesses

- `an_edit_in_one_editor_reaches_the_other_and_the_file_on_disk` in `crates/hickory-cli/tests/serve_local.rs`.

### Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
