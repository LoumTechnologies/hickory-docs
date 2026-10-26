<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-dap/src/session.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3c41c22e0551"></a>
## `crates/hick-dap/src/session.rs`

Related contracts: [A Breakpoint Can Carry A Condition](../../../../guarantees/debugging/a-breakpoint-can-carry-a-condition/contract.md), [A Breakpoint That Is Not Bound Yet Is Not Refused](../../../../guarantees/debugging/a-breakpoint-that-is-not-bound-yet-is-not-refused/contract.md), [A Plain File Has The Same Debugger A Document Has](../../../../guarantees/debugging/a-plain-file-has-the-same-debugger/contract.md), [Use the adapter’s transport and the current measured language support](../../../../guarantees/debugging/an-adapter-that-listens-is-connected-to/contract.md), [An Adapter That Runs On A Second Connection Is Followed](../../../../guarantees/debugging/an-adapter-that-runs-on-a-second-connection-is-followed/contract.md), [A `hick:file` Is A Generated File However Deeply It Is Nested](../../../../guarantees/editor-intelligence/a-file-block-is-a-file-however-deeply-it-is-nested/contract.md).

<a id="f-3c41c22e0551-source_breakpoints"></a>
### `source_breakpoints`

A debug session over one cell: breakpoints, stepping, and evaluation.
Everything a caller needs is here, in document coordinates. The three
surfaces above this — the app, `<hick:capture>`, and the MCP tools — differ
only in who calls it and whether anything is written down.
## Coordinates
A breakpoint is set on a line of the `.md` document; the adapter needs a
line in the generated file. A stack frame comes back in the generated file
and has to be shown on the document's line. That mapping already exists —
it is what makes the language server work — so this uses it rather than a
second one that could disagree.
## What a session may not do
Write. Not to the document, not to the transcripts, not to the outputs.
The scratch workdir a session runs in is discarded when it ends. Stepping
is a read of a running program, and the only thing in this product that
writes a document is a run or a person.

<storycode:copy id="f-3c41c22e0551-source_breakpoints">
//! A debug session over one cell: breakpoints, stepping, and evaluation.
//!
//! Everything a caller needs is here, in document coordinates. The three
//! surfaces above this — the app, `<hick:capture>`, and the MCP tools — differ
//! only in who calls it and whether anything is written down.
//!
//! ## Coordinates
//!
//! A breakpoint is set on a line of the `.md` document; the adapter needs a
//! line in the generated file. A stack frame comes back in the generated file
//! and has to be shown on the document's line. That mapping already exists —
//! it is what makes the language server work — so this uses it rather than a
//! second one that could disagree.
//!
//! ## What a session may not do
//!
//! Write. Not to the document, not to the transcripts, not to the outputs.
//! The scratch workdir a session runs in is discarded when it ends. Stepping
//! is a read of a running program, and the only thing in this product that
//! writes a document is a run or a person.

use std::collections::HashMap;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::{broadcast, mpsc};

use crate::adapter::Adapter;
use crate::protocol::Event;
#[path = "source_breakpoints.rs"]
mod source_breakpoints;

</storycode:copy>
<a id="f-3c41c22e0551-private_capture_session"></a>
### `private_capture_session`

<storycode:copy id="f-3c41c22e0551-private_capture_session">
#[path = "private_capture_session.rs"]
mod private_capture_session;

</storycode:copy>
<a id="f-3c41c22e0551-Capabilities"></a>
### `Capabilities`

What this adapter can actually do, read from its `initialize` reply.
Every control the UI offers is gated on one of these. A button that is
present and silently does nothing is worse than one that is absent.

<storycode:copy id="f-3c41c22e0551-Capabilities">
/// What this adapter can actually do, read from its `initialize` reply.
///
/// Every control the UI offers is gated on one of these. A button that is
/// present and silently does nothing is worse than one that is absent.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capabilities {
    pub conditional_breakpoints: bool,
    pub hit_conditional_breakpoints: bool,
    pub log_points: bool,
    pub set_variable: bool,
    /// Drop frame — re-enter the current function from its first line. The
    /// honest 80% of "step back": it re-runs rather than rewinds, so side
    /// effects already performed stay performed.
    pub restart_frame: bool,
    pub step_in_targets: bool,
    /// Move the instruction pointer within the current frame — Python's
    /// `jump`, "set next statement" elsewhere. The OTHER way backwards, and
    /// the one debugpy has: where `restartFrame` re-enters a function from
    /// the top, this goes to any line in the frame, including an earlier one.
    /// Both re-run rather than rewind; neither undoes a side effect.
    pub goto_targets: bool,
    /// True reverse execution. Almost no adapter has it; the UI greys the
    /// control rather than offering one that fails.
    pub step_back: bool,
    pub terminate_threads: bool,
    pub exception_filters: Vec<ExceptionFilter>,
}

</storycode:copy>
<a id="f-3c41c22e0551-ExceptionFilter"></a>
### `ExceptionFilter`

<storycode:copy id="f-3c41c22e0551-ExceptionFilter">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExceptionFilter {
    pub id: String,
    pub label: String,
}

</storycode:copy>
<a id="f-3c41c22e0551-impl-Capabilities"></a>
### `impl-Capabilities`

<storycode:copy id="f-3c41c22e0551-impl-Capabilities">
impl Capabilities {
    fn from_initialize(body: &Value) -> Self {
        let flag = |name: &str| body.get(name).and_then(Value::as_bool).unwrap_or(false);
        Self {
            conditional_breakpoints: flag("supportsConditionalBreakpoints"),
            hit_conditional_breakpoints: flag("supportsHitConditionalBreakpoints"),
            log_points: flag("supportsLogPoints"),
            set_variable: flag("supportsSetVariable"),
            restart_frame: flag("supportsRestartFrame"),
            step_in_targets: flag("supportsStepInTargetsRequest"),
            goto_targets: flag("supportsGotoTargetsRequest"),
            step_back: flag("supportsStepBack"),
            terminate_threads: flag("supportsTerminateThreadsRequest"),
            exception_filters: body
                .get("exceptionBreakpointFilters")
                .and_then(Value::as_array)
                .map(|filters| {
                    filters
                        .iter()
                        .filter_map(|f| {
                            Some(ExceptionFilter {
                                id: f.get("filter")?.as_str()?.to_string(),
                                label: f.get("label")?.as_str()?.to_string(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

</storycode:copy>
<a id="f-3c41c22e0551-Breakpoint"></a>
### `Breakpoint`

A breakpoint, in the document's own coordinates.

<storycode:copy id="f-3c41c22e0551-Breakpoint">
/// A breakpoint, in the document's own coordinates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// 0-based line in the `.md` document.
    pub line: u32,
    /// Stop only when this is true, in the debuggee's language.
    #[serde(default)]
    pub condition: Option<String>,
    /// Stop only on the Nth hit (`">5"`, `"%3"` — the adapter's own syntax).
    #[serde(default)]
    pub hit_condition: Option<String>,
    /// Log and continue instead of stopping. What a capture uses when the
    /// adapter supports it: the same effect in one round trip rather than
    /// three.
    #[serde(default)]
    pub log_message: Option<String>,
}

</storycode:copy>
<a id="f-3c41c22e0551-SourceBreakpoints"></a>
### `SourceBreakpoints`

A breakpoint set in a source other than the session's entry document.

<storycode:copy id="f-3c41c22e0551-SourceBreakpoints">
/// A breakpoint set in a source other than the session's entry document.
pub struct SourceBreakpoints {
    pub source: String,
    pub document: PathBuf,
    pub breakpoints: Vec<Breakpoint>,
}

</storycode:copy>
<a id="f-3c41c22e0551-BreakpointStatus"></a>
### `BreakpointStatus`

<storycode:copy id="f-3c41c22e0551-BreakpointStatus">
/// What came back when a breakpoint was set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointStatus {
    pub line: u32,
    /// Whether the adapter has bound this line — see [`BindState`], and note
    /// that it has three answers rather than two.
    pub state: BindState,
    #[serde(default)]
    pub message: Option<String>,
    /// The document line the adapter actually bound it to, when that is not
    /// the line that was asked for.
    ///
    /// Adapters slide a breakpoint down to the nearest line that can hold one
    /// — a blank line, a comment, or a docstring becomes the statement after
    /// it. Silently keeping the requested line makes the gutter disagree with
    /// where the program stops, and makes a `<hick:capture>` on a docstring
    /// look like a breakpoint that never fires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moved_to: Option<u32>,
}

</storycode:copy>
<a id="f-3c41c22e0551-BindState"></a>
### `BindState`

Whether a breakpoint is bound, and the reason a bool was not enough.
`setBreakpoints` answers `verified: false` for two situations DAP does not
distinguish, and this product had been reading both as "will never bind":
* **debugpy** verifies while answering, so `false` really is a refusal.
* **netcoredbg** answers every breakpoint `false` with "The breakpoint is
pending and will be resolved when debugging starts", binds it when the
module loads, and says so in a `breakpoint` **event**. Verified on
2026-08-27 by stopping a real C# program on a breakpoint that had been
reported unverified (`crates/hick-dap/tests/live_session_csharp.rs`).
Reading the adapter's message text would tell them apart and is refused:
it is a string another project owns. What is used instead is what DAP
actually states — an unconfirmed breakpoint is unconfirmed, not doomed —
so the only certain refusal is the one made here, before the adapter is
asked at all.

<storycode:copy id="f-3c41c22e0551-BindState">
/// Whether a breakpoint is bound, and the reason a bool was not enough.
///
/// `setBreakpoints` answers `verified: false` for two situations DAP does not
/// distinguish, and this product had been reading both as "will never bind":
///
/// * **debugpy** verifies while answering, so `false` really is a refusal.
/// * **netcoredbg** answers every breakpoint `false` with "The breakpoint is
///   pending and will be resolved when debugging starts", binds it when the
///   module loads, and says so in a `breakpoint` **event**. Verified on
///   2026-08-27 by stopping a real C# program on a breakpoint that had been
///   reported unverified (`crates/hick-dap/tests/live_session_csharp.rs`).
///
/// Reading the adapter's message text would tell them apart and is refused:
/// it is a string another project owns. What is used instead is what DAP
/// actually states — an unconfirmed breakpoint is unconfirmed, not doomed —
/// so the only certain refusal is the one made here, before the adapter is
/// asked at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BindState {
    /// The adapter confirmed this line. The program will stop here.
    Bound,
    /// The adapter has not confirmed it yet. Usual for a compiled language,
    /// where nothing can be bound until the module is loaded — and the
    /// honest word for "we do not know", which is what DAP has said so far.
    Pending,
    /// There is nothing here to stop on, and no adapter was asked. The one
    /// case this product can be certain about: a document line that maps to
    /// no generated code at all.
    Refused,
}

</storycode:copy>
<a id="f-3c41c22e0551-impl-BindState"></a>
### `impl-BindState`

<storycode:copy id="f-3c41c22e0551-impl-BindState">
impl BindState {
    /// Whether the program will definitely not stop here.
    pub fn is_refused(self) -> bool {
        self == BindState::Refused
    }

    /// Whether the adapter has confirmed it.
    pub fn is_bound(self) -> bool {
        self == BindState::Bound
    }
}

</storycode:copy>
<a id="f-3c41c22e0551-stopped_from"></a>
### `stopped_from`

<storycode:copy id="f-3c41c22e0551-stopped_from">
/// Read a `stopped` event body.
fn stopped_from(body: &Value) -> Stopped {
    Stopped {
        reason: body
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("?")
            .into(),
        thread_id: body.get("threadId").and_then(Value::as_i64).unwrap_or(1),
        description: body
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_string),
        text: body.get("text").and_then(Value::as_str).map(str::to_string),
    }
}

</storycode:copy>
<a id="f-3c41c22e0551-stack_frame"></a>
### `stack_frame`

<storycode:copy id="f-3c41c22e0551-stack_frame">
#[path = "stack_frame.rs"]
mod stack_frame;
</storycode:copy>
<a id="f-3c41c22e0551-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-3c41c22e0551-use_declaration">
pub use stack_frame::Frame;

</storycode:copy>
<a id="f-3c41c22e0551-Variable"></a>
### `Variable`

<storycode:copy id="f-3c41c22e0551-Variable">
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub type_name: Option<String>,
    /// Non-zero when this value has children to expand, lazily.
    #[serde(default)]
    pub variables_reference: i64,
}

</storycode:copy>
<a id="f-3c41c22e0551-Exit"></a>
### `Exit`

How the debuggee ended.
DAP splits the end across two events: `exited` carries the exit code and
`terminated` merely says it is over. Adapters send either, both, in either
order — so the code stays optional even once the end itself is certain.

<storycode:copy id="f-3c41c22e0551-Exit">
/// How the debuggee ended.
///
/// DAP splits the end across two events: `exited` carries the exit code and
/// `terminated` merely says it is over. Adapters send either, both, in either
/// order — so the code stays optional even once the end itself is certain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exit {
    pub code: Option<i64>,
}

</storycode:copy>
<a id="f-3c41c22e0551-Stopped"></a>
### `Stopped`

<storycode:copy id="f-3c41c22e0551-Stopped">
/// Why the program stopped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stopped {
    pub reason: String,
    pub thread_id: i64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

</storycode:copy>
<a id="f-3c41c22e0551-Step"></a>
### `Step`

<storycode:copy id="f-3c41c22e0551-Step">
/// How to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    Over,
    In,
    Out,
    Continue,
    /// Re-enter the current function from its first line (`restartFrame`).
    DropFrame,
    /// True reverse execution, where an adapter has it.
    Back,
}

</storycode:copy>
<a id="f-3c41c22e0551-impl-Step"></a>
### `impl-Step`

<storycode:copy id="f-3c41c22e0551-impl-Step">
impl Step {
    fn command(self) -> &'static str {
        match self {
            Step::Over => "next",
            Step::In => "stepIn",
            Step::Out => "stepOut",
            Step::Continue => "continue",
            Step::DropFrame => "restartFrame",
            Step::Back => "stepBack",
        }
    }
}

</storycode:copy>
<a id="f-3c41c22e0551-Session"></a>
### `Session`

<storycode:copy id="f-3c41c22e0551-Session">
/// One cell, being debugged.
pub struct Session {
    pub observation_source: Option<hick_observe::Source>,
    /// The breakpoint statuses as they stand, promoted in place by the
    /// `breakpoint` event. A plain `std` mutex because the event pump is not
    /// async and only ever swaps a small vector.
    statuses: Arc<std::sync::Mutex<Vec<BreakpointStatus>>>,
    /// The adapter's own breakpoint ids, mapped to the document lines they
    /// belong to. Matching an event by id rather than by coordinates matters
    /// because an adapter may have slid the breakpoint to another line.
    bound_ids: Arc<std::sync::Mutex<HashMap<i64, u32>>>,
    adapter: Adapter,
    /// The supervising connection of a multi-session adapter, held open.
    ///
    /// js-debug's launching connection owns the server process and does not
    /// run the program; dropping it would take the server down with the
    /// session still using it. Shut down after the child, never used.
    parent: Option<Adapter>,
    capabilities: Capabilities,
    /// Maps the generated file back to the document, and the document to it.
    mapping: Arc<Mapping>,
    /// The most recent stop, so a caller that missed the event can still ask.
    last_stopped: tokio::sync::Mutex<Option<Stopped>>,
    /// The breakpoints the *caller* asked for, as opposed to whatever is
    /// momentarily set in the adapter.
    ///
    /// `setBreakpoints` replaces every breakpoint in a file, so a one-shot
    /// breakpoint for "run to cursor" cannot simply be added: sending it
    /// deletes the ones a person put there. Keeping the asked-for set here is
    /// what lets the temporary one be layered over it and then taken away.
    desired: tokio::sync::Mutex<Vec<Breakpoint>>,
    sources: Arc<source_breakpoints::Sources>,
    /// Set while a one-shot "run to" breakpoint is in the adapter, so the
    /// next stop can take it out again.
    running_to: tokio::sync::Mutex<Option<u32>>,
    /// Stops, queued from the moment the session exists.
    ///
    /// A caller subscribing only when it is ready to wait loses any stop that
    /// arrived first — and after a `continue`, the program can hit the next
    /// breakpoint before the request even returns. That race is not
    /// theoretical: it made a `<hick:capture>` record its first hit and no
    /// other. A queue filled by a task that starts with the session cannot
    /// miss one.
    stops: tokio::sync::Mutex<mpsc::UnboundedReceiver<Option<Stopped>>>,
    /// How the debuggee ended, once it has. Written by the same watch task
    /// that queues stops, so by the time a waiter hears "no more stops" the
    /// exit is already recorded.
    exit: Arc<std::sync::Mutex<Option<Exit>>>,
}

</storycode:copy>
<a id="f-3c41c22e0551-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-3c41c22e0551-use_declaration-2">
use crate::mapping::{Mapping, same_file};

</storycode:copy>
<a id="f-3c41c22e0551-Launch"></a>
### `Launch`

<storycode:copy id="f-3c41c22e0551-Launch">
/// How the program under test is started.
pub struct Launch {
    /// The adapter command, from discovery.
    pub adapter: Vec<String>,
    /// How that adapter is talked to. `Transport::Stdio` is the default,
    /// which is what every caller meant before some adapters turned out to
    /// be servers.
    pub transport: crate::adapter::Transport,
    /// Launch keys this ADAPTER requires, from discovery. Merged under
    /// `extra`, which is the caller's.
    pub adapter_extra: Value,
    /// Whether this adapter runs the program on a SECOND connection.
    pub multi_session: bool,
    /// The program the cell runs, as the adapter's `launch` wants it.
    pub program: PathBuf,
    /// Working directory — the session's own scratch clone, never the one a
    /// run would write outputs from.
    pub cwd: PathBuf,
    /// Language-specific launch arguments merged over the defaults.
    ///
    /// The adapter's OWN required keys come from discovery
    /// (`Discovered::launch_extra`) and are merged first, so a caller's
    /// `extra` can still override them.
    pub extra: Value,
}

</storycode:copy>
<a id="f-3c41c22e0551-wait_for_start_debugging"></a>
### `wait_for_start_debugging`

Wait for the adapter to ask us to start a child session, and answer it.
The answer matters: DAP is bidirectional and js-debug waits on this one
before it will do anything further. Returning the `configuration`
unchanged is deliberate — it carries `__pendingTargetId`, which is the
adapter's own name for the target, and inventing any part of it would
attach the child to nothing.

<storycode:copy id="f-3c41c22e0551-wait_for_start_debugging">
/// Wait for the adapter to ask us to start a child session, and answer it.
///
/// The answer matters: DAP is bidirectional and js-debug waits on this one
/// before it will do anything further. Returning the `configuration`
/// unchanged is deliberate — it carries `__pendingTargetId`, which is the
/// adapter's own name for the target, and inventing any part of it would
/// attach the child to nothing.
async fn wait_for_start_debugging(
    events: &mut broadcast::Receiver<Event>,
    adapter: &Adapter,
) -> Result<Value> {
    let deadline = Duration::from_secs(30);
    let found = tokio::time::timeout(deadline, async {
        loop {
            let event = match events.recv().await {
                Ok(event) => event,
                // Lagged: the burst of telemetry js-debug emits can overrun a
                // slow subscriber, and the request may have been in it.
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => return None,
            };
            if event.event == "hick/reverseRequest/startDebugging" {
                return Some(event);
            }
        }
    })
    .await
    .ok()
    .flatten();
    let event = found.context("no `startDebugging` request arrived")?;
    // `reverse_as_event` carries the request's own seq, which is what a
    // response has to name.
    adapter
        .respond(event.seq, "startDebugging", json!({}))
        .await?;
    Ok(event
        .body
        .get("configuration")
        .cloned()
        .unwrap_or_else(|| json!({})))
}

</storycode:copy>
<a id="f-3c41c22e0551-impl-Session"></a>
### `impl-Session`

<storycode:copy id="f-3c41c22e0551-impl-Session">
impl Session {
    pub async fn start_with_sources(
        launch: Launch,
        mapping: Arc<Mapping>,
        breakpoints: &[Breakpoint],
        sources: &[SourceBreakpoints],
    ) -> Result<(Self, Vec<BreakpointStatus>)> {
        let adapter = Adapter::start(&launch.adapter, launch.transport).await?;
        let extra = launch.extra.clone();
        Self::start_on(adapter, None, &launch, extra, mapping, breakpoints, sources).await
    }

    /// Start a session on one connection, following a child if the adapter
    /// asks for one.
    ///
    /// `parent` is the supervising connection of a multi-session adapter,
    /// held only so that it stays alive and can be shut down: js-debug's
    /// launching connection owns the server process, and the program runs on
    /// a sibling.
    #[allow(clippy::too_many_arguments)]
    async fn start_on(
        adapter: Adapter,
        parent: Option<Adapter>,
        launch: &Launch,
        extra: Value,
        mapping: Arc<Mapping>,
        breakpoints: &[Breakpoint],
        source_breakpoints: &[SourceBreakpoints],
    ) -> Result<(Self, Vec<BreakpointStatus>)> {
        // Subscribed BEFORE initialize: `initialized` arrives as an event and
        // routinely beats the response to the request that caused it.
        let mut events = adapter.events();
        // A separate subscription for the same reason, one step later:
        // `startDebugging` can arrive before `configurationDone` is answered,
        // and a receiver created after the fact would have missed it.
        let mut child_watch = adapter.events();

        let body = adapter
            .request(
                "initialize",
                json!({
                    "clientID": "hick",
                    "clientName": "Hickory Docs",
                    "adapterID": "hick",
                    "linesStartAt1": true,
                    "columnsStartAt1": true,
                    "pathFormat": "path",
                    "supportsVariableType": true,
                    "supportsRunInTerminalRequest": false,
                    // Declared, because it is now true: a `startDebugging`
                    // request is answered and followed. An adapter that
                    // multiplexes decides what to send from this.
                    "supportsStartDebuggingRequest": true,
                }),
            )
            .await
            .context("the adapter refused to initialize")?;
        let capabilities = Capabilities::from_initialize(&body);

        let mut launch_args = json!({
            "program": launch.program.to_string_lossy(),
            "cwd": launch.cwd.to_string_lossy(),
            "noDebug": false,
            // Stepping into the standard library is a way to lose ten minutes.
            "justMyCode": true,
            "console": "internalConsole",
        });
        merge(&mut launch_args, &launch.adapter_extra);
        // For a child this is the configuration the adapter handed back,
        // `__pendingTargetId` and all — which is what names the target this
        // connection is for.
        merge(&mut launch_args, &extra);

        // SENT, NOT AWAITED, and this is the one ordering that must be right.
        //
        // Most adapters do not answer `launch` until after
        // `configurationDone` — the launch is not finished until the client
        // has finished configuring it. Awaiting the response here therefore
        // deadlocks forever: we wait for a reply that is waiting for a
        // request we have not sent because we are waiting for the reply.
        //
        // The protocol's real sequence is: send launch, wait for the
        // `initialized` EVENT, set breakpoints, send configurationDone — and
        // only then does the launch response arrive.
        adapter
            .notify("launch", launch_args)
            .await
            .context("sending launch")?;

        // The adapter says when it is ready for breakpoints. Setting them
        // before this is the other classic DAP mistake: they are accepted and
        // silently dropped.
        wait_for_event(&mut events, "initialized", Duration::from_secs(30))
            .await
            .context("the adapter never became ready for breakpoints")?;

        let statuses: Arc<std::sync::Mutex<Vec<BreakpointStatus>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let bound_ids: Arc<std::sync::Mutex<HashMap<i64, u32>>> =
            Arc::new(std::sync::Mutex::new(HashMap::new()));
        let sources = Arc::new(source_breakpoints::Sources::default());
        let promoting_sources = sources.clone();
        let promoting = statuses.clone();
        let promoting_ids = bound_ids.clone();

        let (tx, stops) = mpsc::unbounded_channel();
        let exit: Arc<std::sync::Mutex<Option<Exit>>> = Arc::new(std::sync::Mutex::new(None));
        let ended = exit.clone();
        let mut watch = adapter.events();
        tokio::spawn(async move {
            while let Ok(event) = watch.recv().await {
                match event.event.as_str() {
                    "stopped" => {
                        if tx.send(Some(stopped_from(&event.body))).is_err() {
                            return;
                        }
                    }
                    // `None` is "there will be no more stops", which is what
                    // a waiting caller needs to hear. The exit is recorded
                    // FIRST, so whoever hears it can ask how it ended:
                    // `exited` carries the code, `terminated` does not, and
                    // whichever arrives first is the one we act on.
                    "exited" => {
                        let code = event.body.get("exitCode").and_then(Value::as_i64);
                        *ended.lock().unwrap() = Some(Exit { code });
                        let _ = tx.send(None);
                        return;
                    }
                    "terminated" => {
                        *ended.lock().unwrap() = Some(Exit { code: None });
                        let _ = tx.send(None);
                        return;
                    }
                    // How an adapter says a breakpoint it could not confirm
                    // at set time has now bound. Without this, every
                    // breakpoint in a compiled language stays "pending" for
                    // the life of the session while working perfectly.
                    "breakpoint" => {
                        promoting_sources.promote(&event.body);
                        let Some(reported) = event.body.get("breakpoint") else {
                            continue;
                        };
                        if reported.get("verified").and_then(Value::as_bool) != Some(true) {
                            continue;
                        }
                        let Some(id) = reported.get("id").and_then(Value::as_i64) else {
                            continue;
                        };
                        let line = promoting_ids.lock().unwrap().get(&id).copied();
                        let Some(line) = line else { continue };
                        let mut held = promoting.lock().unwrap();
                        if let Some(status) =
                            held.iter_mut().find(|status| status.line == line)
                            // A refusal made here, before any adapter was
                            // asked, is not something an adapter may overturn.
                            && !status.state.is_refused()
                        {
                            status.state = BindState::Bound;
                            status.message = None;
                        }
                    }
                    _ => {}
                }
            }
            // The adapter's stream ended without saying so: the process is
            // gone, and that is an end too — just one with nothing to report.
            let mut recorded = ended.lock().unwrap();
            if recorded.is_none() {
                *recorded = Some(Exit { code: None });
            }
            drop(recorded);
            let _ = tx.send(None);
        });

        let is_supervisor = launch.multi_session && parent.is_none();
        let observation_source = mapping.observation_source();
        let session = Self {
            observation_source,
            parent,
            adapter,
            capabilities,
            mapping: mapping.clone(),
            statuses,
            bound_ids,
            sources,
            last_stopped: tokio::sync::Mutex::new(None),
            desired: tokio::sync::Mutex::new(Vec::new()),
            running_to: tokio::sync::Mutex::new(None),
            stops: tokio::sync::Mutex::new(stops),
            exit,
        };
        let statuses = session.set_breakpoints(breakpoints).await?;
        for set in source_breakpoints {
            session
                .set_source_breakpoints(&set.source, &set.document, &set.breakpoints)
                .await?;
        }
        session
            .adapter
            .request("configurationDone", json!({}))
            .await?;

        // A multi-session adapter has not run anything yet. This connection
        // launched a supervisor, whose answer is a request back: "start
        // debugging this target". The breakpoints just set on it came back
        // `provisionalBreakpoint` and will never fire; the ones that matter
        // are set on the sibling below, by the recursion.
        if is_supervisor {
            let configuration = wait_for_start_debugging(&mut child_watch, &session.adapter)
                .await
                .context(
                    "this adapter runs the program on a second connection and never asked \
                     for one",
                )?;
            let sibling = session.adapter.sibling().await?;
            let parent = session.into_parent();
            return Box::pin(Self::start_on(
                sibling,
                Some(parent),
                launch,
                configuration,
                mapping.clone(),
                breakpoints,
                source_breakpoints,
            ))
            .await;
        }
        Ok((session, statuses))
    }

    /// Give up everything but the connection, so it can be held open.
    fn into_parent(self) -> Adapter {
        self.adapter
    }

    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    /// How the debuggee ended — `None` while it is still running.
    ///
    /// Set the moment the adapter reports `exited` or `terminated`, which is
    /// what lets a caller that just heard "no more stops" say whether that
    /// was a clean exit, a failure code, or an adapter that simply went away.
    pub fn exit(&self) -> Option<Exit> {
        *self.exit.lock().unwrap()
    }

    pub fn events(&self) -> broadcast::Receiver<Event> {
        self.adapter.events()
    }

    /// The breakpoint statuses as they stand now, including any promotion a
    /// `breakpoint` event has made since they were set.
    ///
    /// This is what a caller should show after the program has started: for a
    /// compiled language the set-time answer is "pending" for everything, and
    /// the truth arrives a moment later.
    pub fn breakpoint_statuses(&self) -> Vec<BreakpointStatus> {
        self.statuses.lock().unwrap().clone()
    }

    /// Wait for the next stop, or for the program to end.
    ///
    /// Reads the queue that has been filling since the session started, so a
    /// stop that arrived before this call is still waiting here.
    pub async fn wait_for_stop(&self, timeout: Duration) -> Result<Option<Stopped>> {
        let mut stops = self.stops.lock().await;
        match tokio::time::timeout(timeout, stops.recv()).await {
            Ok(Some(Some(stopped))) => {
                *self.last_stopped.lock().await = Some(stopped.clone());
                drop(stops);
                self.clear_run_to().await;
                Ok(Some(stopped))
            }
            // The program ended, or the adapter is gone: either way there
            // will be no more stops.
            Ok(Some(None)) | Ok(None) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    /// Take out the one-shot breakpoint a "run to" left, if there is one.
    ///
    /// Called on every stop rather than only the one it caused: a program
    /// that hits a real breakpoint on the way to the cursor has arrived
    /// somewhere the person is now looking at, and leaving an invisible
    /// breakpoint armed behind them is how a later `continue` stops for no
    /// reason anyone can see.
    async fn clear_run_to(&self) {
        let pending = self.running_to.lock().await.take();
        if pending.is_some() {
            let desired = self.desired.lock().await.clone();
            if let Err(error) = self.apply_breakpoints(&desired).await {
                tracing::warn!("could not remove the run-to breakpoint: {error:#}");
            }
        }
    }

    /// The call stack, in document coordinates where it can be.
    pub async fn stack(&self, thread_id: i64) -> Result<Vec<Frame>> {
        let body = self
            .adapter
            .request("stackTrace", json!({ "threadId": thread_id, "levels": 50 }))
            .await?;
        let frames = body
            .get("stackFrames")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        Ok(frames
            .iter()
            .map(|frame| {
                let path = frame
                    .pointer("/source/path")
                    .and_then(Value::as_str)
                    .map(PathBuf::from);
                // DAP counts from 1 when the client says so, which we did.
                let raw_line = frame
                    .get("line")
                    .and_then(Value::as_i64)
                    .unwrap_or(1)
                    .max(1) as u32
                    - 1;
                let source_location = path
                    .as_deref()
                    .and_then(|p| self.mapping.source_location(p, raw_line));
                let document_line = source_location
                    .as_ref()
                    .filter(|(p, _)| same_file(p, self.mapping.document()))
                    .map(|(_, line)| *line);
                Frame {
                    id: frame.get("id").and_then(Value::as_i64).unwrap_or(0),
                    name: frame
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("?")
                        .to_string(),
                    line: document_line,
                    source_line: source_location
                        .as_ref()
                        .map(|(_, line)| *line)
                        .or_else(|| path.as_ref().map(|_| raw_line)),
                    adapter_source: path.as_ref().map(|path| path.to_string_lossy().to_string()),
                    adapter_line: path.as_ref().map(|_| raw_line),
                    source: source_location
                        .map(|(p, _)| p)
                        .or(path)
                        .map(|p| p.to_string_lossy().to_string()),
                    in_document: document_line.is_some(),
                }
            })
            .collect())
    }

    /// The variables visible in a frame, flattened across its scopes.
    pub async fn variables(&self, frame_id: i64) -> Result<Vec<Variable>> {
        let scopes = self
            .adapter
            .request("scopes", json!({ "frameId": frame_id }))
            .await?;
        let mut out = Vec::new();
        for scope in scopes
            .get("scopes")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
        {
            // `expensive` scopes are things like "Globals" that an adapter
            // warns will cost real time to fetch. Skipped: an inline-value
            // display that stalls the editor is worse than one that shows
            // less.
            if scope
                .get("expensive")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                continue;
            }
            let Some(reference) = scope.get("variablesReference").and_then(Value::as_i64) else {
                continue;
            };
            out.extend(self.children(reference).await?);
        }
        Ok(out)
    }

    /// Expand one composite value, lazily.
    pub async fn children(&self, variables_reference: i64) -> Result<Vec<Variable>> {
        if variables_reference == 0 {
            return Ok(Vec::new());
        }
        let body = self
            .adapter
            .request(
                "variables",
                json!({ "variablesReference": variables_reference }),
            )
            .await?;
        Ok(body
            .get("variables")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|v| Variable {
                name: v
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("?")
                    .to_string(),
                value: v
                    .get("value")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                type_name: v.get("type").and_then(Value::as_str).map(str::to_string),
                variables_reference: v
                    .get("variablesReference")
                    .and_then(Value::as_i64)
                    .unwrap_or(0),
            })
            .collect())
    }

    /// Evaluate an expression in a frame.
    ///
    /// `context` is DAP's own: `watch` for something evaluated repeatedly (a
    /// capture, a watch pane), `hover` for a tooltip, `repl` for something a
    /// person typed. Adapters treat them differently — several refuse side
    /// effects in `watch` — which is exactly why a capture uses it.
    pub async fn evaluate(
        &self,
        expression: &str,
        frame_id: Option<i64>,
        context: &str,
    ) -> Result<Variable> {
        let mut args = json!({ "expression": expression, "context": context });
        if let Some(frame_id) = frame_id {
            args["frameId"] = json!(frame_id);
        }
        let body = self.adapter.request("evaluate", args).await?;
        Ok(Variable {
            name: expression.to_string(),
            value: body
                .get("result")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            type_name: body.get("type").and_then(Value::as_str).map(str::to_string),
            variables_reference: body
                .get("variablesReference")
                .and_then(Value::as_i64)
                .unwrap_or(0),
        })
    }

    /// Move.
    pub async fn step(&self, step: Step, thread_id: i64, frame_id: Option<i64>) -> Result<()> {
        match step {
            Step::DropFrame => {
                if !self.capabilities.restart_frame {
                    if self.capabilities.goto_targets {
                        bail!(
                            "this debug adapter cannot drop a frame, but it CAN move the \
                             instruction pointer: jump to an earlier line in this frame instead. \
                             Both re-run rather than rewind."
                        );
                    }
                    bail!(
                        "this debug adapter can neither drop a frame nor move the instruction \
                         pointer, so there is no way backwards short of starting again. The \
                         controls that do work are step over, in, out and continue."
                    );
                }
                let frame_id = frame_id.context("dropping a frame needs the frame to drop")?;
                self.adapter
                    .request("restartFrame", json!({ "frameId": frame_id }))
                    .await?;
            }
            Step::Back => {
                if !self.capabilities.step_back {
                    bail!(
                        "this debug adapter cannot step backwards — almost none can, because it \
                         requires recorded execution. Drop frame re-runs the current function \
                         from its first line, which is usually what was wanted."
                    );
                }
                self.adapter
                    .request("stepBack", json!({ "threadId": thread_id }))
                    .await?;
            }
            other => {
                self.adapter
                    .request(other.command(), json!({ "threadId": thread_id }))
                    .await?;
            }
        }
        Ok(())
    }

    /// Run to a document line without setting a permanent breakpoint.
    ///
    /// The step people reach for most, and DAP has it directly — doing it by
    /// hand (set breakpoint, continue, remove breakpoint) would race the
    /// program.
    pub async fn run_to(&self, line: u32, thread_id: i64) -> Result<()> {
        let (path, target) = self
            .mapping
            .to_generated(line)
            .context("that line is prose, not code — there is nothing there to run to")?;
        self.adapter
            .request(
                "gotoTargets",
                json!({ "source": { "path": path.to_string_lossy() }, "line": target + 1 }),
            )
            .await
            .ok();
        // `gotoTargets`/`goto` JUMPS, which is not what "run to cursor"
        // means. The portable form is a one-shot breakpoint, and DAP has no
        // flag for that — so it is set, continued to, and removed at the next
        // stop (see `wait_for_stop`).
        //
        // Layered over the asked-for set rather than sent on its own:
        // `setBreakpoints` replaces every breakpoint in a file, so sending
        // only this one would silently delete the ones a person put there —
        // and they would still be drawn in the gutter, which is worse than
        // losing them outright.
        let mut set = self.desired.lock().await.clone();
        let already_there = set.iter().any(|b| b.line == line);
        if !already_there {
            set.push(Breakpoint {
                line,
                condition: None,
                hit_condition: None,
                log_message: None,
            });
        }
        let statuses = self.apply_breakpoints(&set).await?;
        if statuses
            .iter()
            .find(|status| status.line == line)
            .is_some_and(|status| status.state.is_refused())
        {
            // Put back what was there before giving up, or the failure would
            // also have quietly changed the breakpoints.
            let desired = self.desired.lock().await.clone();
            let _ = self.apply_breakpoints(&desired).await;
            bail!("nothing to run to on that line — the adapter could not bind a breakpoint there");
        }
        if !already_there {
            *self.running_to.lock().await = Some(line);
        }
        self.step(Step::Continue, thread_id, None).await
    }

    /// Move the instruction pointer to a document line in the current frame.
    ///
    /// The backwards move that actually exists for most adapters. Jumping to
    /// an earlier line re-executes from there — it does not rewind, so
    /// anything already written stays written — but "I stepped one too far,
    /// do that again" is answered, which is what people want from stepping
    /// back nine times in ten.
    ///
    /// Constrained to the current frame by the protocol, which is also the
    /// only place it is meaningful: jumping into a different function would
    /// arrive with the wrong locals.
    pub async fn jump_to(&self, line: u32, thread_id: i64) -> Result<()> {
        if !self.capabilities.goto_targets {
            bail!(
                "this debug adapter cannot move the instruction pointer. Neither it nor drop \
                 frame is available here, so the only way back is to start the session again."
            );
        }
        let (path, target_line) = self
            .mapping
            .to_generated(line)
            .context("that line is prose, not code — there is nothing there to jump to")?;

        let body = self
            .adapter
            .request(
                "gotoTargets",
                json!({
                    "source": { "path": path.to_string_lossy() },
                    "line": target_line + 1,
                }),
            )
            .await?;
        let target = body
            .get("targets")
            .and_then(Value::as_array)
            .and_then(|targets| targets.first())
            .and_then(|target| target.get("id"))
            .and_then(Value::as_i64)
            .context(
                "the adapter offers no jump target on that line — a line inside a different \
                 function, or one the compiler removed, cannot be jumped to",
            )?;

        self.adapter
            .request("goto", json!({ "threadId": thread_id, "targetId": target }))
            .await
            .with_context(|| {
                // The adapter's own message for this is routinely empty, and
                // the cause is almost always the same one: the pointer can
                // only move within the frame you are stopped in. Arriving
                // somewhere else would arrive with the wrong locals, so the
                // constraint is the protocol's, not ours.
                format!(
                    "could not move the instruction pointer to line {line}. It has to be a line \
                     in the function you are stopped in — stepping out of that function first is \
                     the usual reason this fails."
                )
            })?;
        Ok(())
    }

    /// Change a variable's value in the running program.
    pub async fn set_variable(&self, container: i64, name: &str, value: &str) -> Result<Variable> {
        if !self.capabilities.set_variable {
            bail!("this debug adapter cannot change a variable's value");
        }
        let body = self
            .adapter
            .request(
                "setVariable",
                json!({ "variablesReference": container, "name": name, "value": value }),
            )
            .await?;
        Ok(Variable {
            name: name.to_string(),
            value: body
                .get("value")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            type_name: body.get("type").and_then(Value::as_str).map(str::to_string),
            variables_reference: body
                .get("variablesReference")
                .and_then(Value::as_i64)
                .unwrap_or(0),
        })
    }

    /// Break where an exception is raised rather than where it surfaced.
    pub async fn set_exception_breakpoints(&self, filters: &[String]) -> Result<()> {
        self.adapter
            .request("setExceptionBreakpoints", json!({ "filters": filters }))
            .await?;
        Ok(())
    }

    /// Which calls on this line could be stepped into.
    pub async fn step_in_targets(&self, frame_id: i64) -> Result<Vec<(i64, String)>> {
        if !self.capabilities.step_in_targets {
            return Ok(Vec::new());
        }
        let body = self
            .adapter
            .request("stepInTargets", json!({ "frameId": frame_id }))
            .await?;
        Ok(body
            .get("targets")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|t| {
                Some((
                    t.get("id")?.as_i64()?,
                    t.get("label")?.as_str()?.to_string(),
                ))
            })
            .collect())
    }

    pub async fn shutdown(&self) {
        // The child first, then the connection that owns the server process.
        // The other order kills the server while the session it is running
        // is still being told to stop.
        self.adapter.shutdown().await;
        if let Some(parent) = &self.parent {
            parent.shutdown().await;
        }
    }
}

</storycode:copy>
<a id="f-3c41c22e0551-wait_for_event"></a>
### `wait_for_event`

<storycode:copy id="f-3c41c22e0551-wait_for_event">
/// Wait for one named event.
async fn wait_for_event(
    events: &mut broadcast::Receiver<Event>,
    name: &str,
    timeout: Duration,
) -> Result<Event> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            bail!("timed out waiting for the adapter's `{name}` event");
        }
        match tokio::time::timeout(remaining, events.recv()).await {
            Ok(Ok(event)) if event.event == name => return Ok(event),
            Ok(Ok(_)) => {}
            Ok(Err(broadcast::error::RecvError::Lagged(_))) => {}
            _ => bail!("the adapter ended before sending `{name}`"),
        }
    }
}

</storycode:copy>
<a id="f-3c41c22e0551-merge"></a>
### `merge`

<storycode:copy id="f-3c41c22e0551-merge">
/// Merge `extra` over `base`, one level deep.
fn merge(base: &mut Value, extra: &Value) {
    let (Some(base), Some(extra)) = (base.as_object_mut(), extra.as_object()) else {
        return;
    };
    for (key, value) in extra {
        base.insert(key.clone(), value.clone());
    }
}

</storycode:copy>
<a id="f-3c41c22e0551-tests"></a>
### `tests`

<storycode:copy id="f-3c41c22e0551-tests">
#[cfg(test)]
#[path = "session/tests/tests.rs"]
mod tests;
</storycode:copy>

### Assemble `crates/hick-dap/src/session.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/src/session.rs"><storycode:paste select="#f-3c41c22e0551-source_breakpoints"/><storycode:paste select="#f-3c41c22e0551-private_capture_session"/><storycode:paste select="#f-3c41c22e0551-Capabilities"/><storycode:paste select="#f-3c41c22e0551-ExceptionFilter"/><storycode:paste select="#f-3c41c22e0551-impl-Capabilities"/><storycode:paste select="#f-3c41c22e0551-Breakpoint"/><storycode:paste select="#f-3c41c22e0551-SourceBreakpoints"/><storycode:paste select="#f-3c41c22e0551-BreakpointStatus"/><storycode:paste select="#f-3c41c22e0551-BindState"/><storycode:paste select="#f-3c41c22e0551-impl-BindState"/><storycode:paste select="#f-3c41c22e0551-stopped_from"/><storycode:paste select="#f-3c41c22e0551-stack_frame"/><storycode:paste select="#f-3c41c22e0551-use_declaration"/><storycode:paste select="#f-3c41c22e0551-Variable"/><storycode:paste select="#f-3c41c22e0551-Exit"/><storycode:paste select="#f-3c41c22e0551-Stopped"/><storycode:paste select="#f-3c41c22e0551-Step"/><storycode:paste select="#f-3c41c22e0551-impl-Step"/><storycode:paste select="#f-3c41c22e0551-Session"/><storycode:paste select="#f-3c41c22e0551-use_declaration-2"/><storycode:paste select="#f-3c41c22e0551-Launch"/><storycode:paste select="#f-3c41c22e0551-wait_for_start_debugging"/><storycode:paste select="#f-3c41c22e0551-impl-Session"/><storycode:paste select="#f-3c41c22e0551-wait_for_event"/><storycode:paste select="#f-3c41c22e0551-merge"/><storycode:paste select="#f-3c41c22e0551-tests"/></storycode:file>

</storycode:doc>
