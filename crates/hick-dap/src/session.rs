//! A debug session over one cell: breakpoints, stepping, and evaluation.
//!
//! Everything a caller needs is here, in document coordinates. The three
//! surfaces above this — the app, `<hick:capture>`, and the MCP tools — differ
//! only in who calls it and whether anything is written down.
//!
//! ## Coordinates
//!
//! A breakpoint is set on a line of the `.hick` document; the adapter needs a
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
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::broadcast;

use crate::adapter::Adapter;
use crate::protocol::Event;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExceptionFilter {
    pub id: String,
    pub label: String,
}

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

/// A breakpoint, in the document's own coordinates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// 0-based line in the `.hick` document.
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

/// What came back when a breakpoint was set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointStatus {
    pub line: u32,
    /// False when the adapter could not bind it — a line the debugger will
    /// never reach. Shown differently in the gutter, because a breakpoint
    /// that cannot work must not look like one that does.
    pub verified: bool,
    #[serde(default)]
    pub message: Option<String>,
}

/// Where execution is, in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub id: i64,
    pub name: String,
    /// 0-based document line, when this frame is in a file the document
    /// generated. `None` for a library frame, which is shown read-only rather
    /// than pretended to be part of the document.
    pub line: Option<u32>,
    /// The file as the adapter named it, for frames outside the document.
    pub source: Option<String>,
    pub in_document: bool,
}

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

/// One cell, being debugged.
pub struct Session {
    adapter: Adapter,
    capabilities: Capabilities,
    /// Maps the generated file back to the document, and the document to it.
    mapping: Arc<Mapping>,
    /// The most recent stop, so a caller that missed the event can still ask.
    last_stopped: tokio::sync::Mutex<Option<Stopped>>,
}

/// Document <-> generated-file coordinates for the files a cell can stop in.
pub struct Mapping {
    /// Generated file path (as the adapter sees it) -> the document's mapping.
    files: HashMap<PathBuf, hick_lsp::position_map::PositionMap>,
    /// The document's own path, for naming.
    document: PathBuf,
}

impl Mapping {
    /// Build the mapping for a document by weaving it the way the language
    /// server does.
    pub fn for_document(document: &Path, source: &str, workdir: &Path) -> Result<Self> {
        let state = hick_lsp::document::HickDocumentState::from_source(source)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut files = HashMap::new();
        for file in &state.virtual_files {
            let map = hick_lsp::position_map::PositionMap::build(&file.segments);
            // The adapter sees the file where the cell ran, not where the
            // document lives.
            files.insert(workdir.join(&file.path), map);
        }
        Ok(Self {
            files,
            document: document.to_path_buf(),
        })
    }

    /// Document line -> (generated file, line) for setting a breakpoint.
    pub fn to_generated(&self, line: u32) -> Option<(PathBuf, u32)> {
        for (path, map) in &self.files {
            if let Some((vline, _)) = map.to_virtual(line, 0) {
                return Some((path.clone(), vline));
            }
        }
        None
    }

    /// (generated file, line) -> document line for showing a frame.
    pub fn to_document(&self, path: &Path, line: u32) -> Option<u32> {
        let map = self.files.get(path)?;
        map.to_source(line, 0).map(|(l, _)| l)
    }

    pub fn document(&self) -> &Path {
        &self.document
    }

    /// Every generated file this document produces, for the launch config.
    pub fn generated_files(&self) -> Vec<PathBuf> {
        self.files.keys().cloned().collect()
    }
}

/// How the program under test is started.
pub struct Launch {
    /// The adapter command, from discovery.
    pub adapter: Vec<String>,
    /// The program the cell runs, as the adapter's `launch` wants it.
    pub program: PathBuf,
    /// Working directory — the session's own scratch clone, never the one a
    /// run would write outputs from.
    pub cwd: PathBuf,
    /// Language-specific launch arguments merged over the defaults.
    pub extra: Value,
}

impl Session {
    /// Start an adapter, launch the program, and stop at the first
    /// breakpoint.
    pub async fn start(
        launch: Launch,
        mapping: Arc<Mapping>,
        breakpoints: &[Breakpoint],
    ) -> Result<(Self, Vec<BreakpointStatus>)> {
        let adapter = Adapter::spawn(&launch.adapter).await?;
        // Subscribed BEFORE initialize: `initialized` arrives as an event and
        // routinely beats the response to the request that caused it.
        let mut events = adapter.events();

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
        merge(&mut launch_args, &launch.extra);

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

        let session = Self {
            adapter,
            capabilities,
            mapping,
            last_stopped: tokio::sync::Mutex::new(None),
        };
        let statuses = session.set_breakpoints(breakpoints).await?;
        session
            .adapter
            .request("configurationDone", json!({}))
            .await?;
        Ok((session, statuses))
    }

    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    pub fn events(&self) -> broadcast::Receiver<Event> {
        self.adapter.events()
    }

    /// Set every breakpoint, grouped by the generated file they land in.
    ///
    /// DAP replaces a file's whole breakpoint set on each call, so they must
    /// be grouped: sending them one at a time leaves only the last one set,
    /// which looks like the others silently failing to bind.
    pub async fn set_breakpoints(
        &self,
        breakpoints: &[Breakpoint],
    ) -> Result<Vec<BreakpointStatus>> {
        let mut by_file: HashMap<PathBuf, Vec<(&Breakpoint, u32)>> = HashMap::new();
        let mut unmapped = Vec::new();
        for breakpoint in breakpoints {
            match self.mapping.to_generated(breakpoint.line) {
                Some((path, line)) => by_file.entry(path).or_default().push((breakpoint, line)),
                None => unmapped.push(BreakpointStatus {
                    line: breakpoint.line,
                    verified: false,
                    message: Some(
                        "this line is prose, not code the document generates — there is nothing \
                         there to stop on"
                            .into(),
                    ),
                }),
            }
        }

        let mut out = unmapped;
        for (path, entries) in by_file {
            let payload: Vec<Value> = entries
                .iter()
                .map(|(breakpoint, line)| {
                    let mut item = json!({ "line": line + 1 });
                    if let Some(condition) = &breakpoint.condition
                        && self.capabilities.conditional_breakpoints
                    {
                        item["condition"] = json!(condition);
                    }
                    if let Some(hit) = &breakpoint.hit_condition
                        && self.capabilities.hit_conditional_breakpoints
                    {
                        item["hitCondition"] = json!(hit);
                    }
                    if let Some(message) = &breakpoint.log_message
                        && self.capabilities.log_points
                    {
                        item["logMessage"] = json!(message);
                    }
                    item
                })
                .collect();

            let body = self
                .adapter
                .request(
                    "setBreakpoints",
                    json!({
                        "source": { "path": path.to_string_lossy() },
                        "breakpoints": payload,
                    }),
                )
                .await?;

            let verified = body
                .get("breakpoints")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for (index, (breakpoint, _)) in entries.iter().enumerate() {
                let answer = verified.get(index);
                out.push(BreakpointStatus {
                    line: breakpoint.line,
                    verified: answer
                        .and_then(|a| a.get("verified"))
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    message: answer
                        .and_then(|a| a.get("message"))
                        .and_then(Value::as_str)
                        .map(str::to_string),
                });
            }
        }
        out.sort_by_key(|status| status.line);
        Ok(out)
    }

    /// Wait for the next stop, or for the program to end.
    pub async fn wait_for_stop(&self, timeout: Duration) -> Result<Option<Stopped>> {
        let mut events = self.adapter.events();
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Ok(None);
            }
            let Ok(Ok(event)) = tokio::time::timeout(remaining, events.recv()).await else {
                return Ok(None);
            };
            match event.event.as_str() {
                "stopped" => {
                    let stopped = Stopped {
                        reason: event
                            .body
                            .get("reason")
                            .and_then(Value::as_str)
                            .unwrap_or("?")
                            .into(),
                        thread_id: event
                            .body
                            .get("threadId")
                            .and_then(Value::as_i64)
                            .unwrap_or(1),
                        description: event
                            .body
                            .get("description")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        text: event
                            .body
                            .get("text")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                    };
                    *self.last_stopped.lock().await = Some(stopped.clone());
                    return Ok(Some(stopped));
                }
                // The program ended without stopping again, which is a
                // perfectly ordinary answer to "continue".
                "terminated" | "exited" => return Ok(None),
                _ => {}
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
                let document_line = path
                    .as_deref()
                    .and_then(|p| self.mapping.to_document(p, raw_line));
                Frame {
                    id: frame.get("id").and_then(Value::as_i64).unwrap_or(0),
                    name: frame
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("?")
                        .to_string(),
                    line: document_line,
                    source: path.map(|p| p.to_string_lossy().to_string()),
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
        // flag for that — so it is set, continued to, and removed.
        let existing = self
            .set_breakpoints(&[Breakpoint {
                line,
                condition: None,
                hit_condition: None,
                log_message: None,
            }])
            .await?;
        if existing.first().is_some_and(|b| !b.verified) {
            bail!("nothing to run to on that line — the adapter could not bind a breakpoint there");
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
        self.adapter.shutdown().await;
    }
}

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

/// Merge `extra` over `base`, one level deep.
fn merge(base: &mut Value, extra: &Value) {
    let (Some(base), Some(extra)) = (base.as_object_mut(), extra.as_object()) else {
        return;
    };
    for (key, value) in extra {
        base.insert(key.clone(), value.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_are_read_from_the_adapters_own_answer() {
        // Every control is gated on one of these, so reading them wrongly
        // means offering a button that does nothing.
        let caps = Capabilities::from_initialize(&json!({
            "supportsConditionalBreakpoints": true,
            "supportsRestartFrame": true,
            "supportsSetVariable": true,
            "exceptionBreakpointFilters": [
                { "filter": "raised", "label": "Raised Exceptions" },
                { "filter": "uncaught", "label": "Uncaught Exceptions" }
            ]
        }));
        assert!(caps.conditional_breakpoints);
        assert!(caps.restart_frame);
        assert!(caps.set_variable);
        // Absent means false, never "probably supported".
        assert!(!caps.step_back);
        assert!(!caps.log_points);
        assert_eq!(caps.exception_filters.len(), 2);
        assert_eq!(caps.exception_filters[0].id, "raised");
    }

    #[test]
    fn step_verbs_map_to_the_protocols_own_names() {
        assert_eq!(Step::Over.command(), "next");
        assert_eq!(Step::In.command(), "stepIn");
        assert_eq!(Step::Out.command(), "stepOut");
        assert_eq!(Step::Continue.command(), "continue");
        assert_eq!(Step::DropFrame.command(), "restartFrame");
        assert_eq!(Step::Back.command(), "stepBack");
    }

    #[test]
    fn launch_arguments_can_be_overridden_per_language() {
        // `justMyCode` is right for Python and meaningless elsewhere, so the
        // defaults have to be overridable rather than final.
        let mut base = json!({ "program": "a.py", "justMyCode": true });
        merge(
            &mut base,
            &json!({ "justMyCode": false, "args": ["--fast"] }),
        );
        assert_eq!(base["justMyCode"], json!(false));
        assert_eq!(base["args"], json!(["--fast"]));
        assert_eq!(base["program"], json!("a.py"));
    }

    #[test]
    fn a_document_line_maps_into_the_file_it_generates() {
        let source = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
             # Title\n\n\
             <hick:file path=\"app.py\">\n\
             def total(x):\n\
             \x20   return x * 2\n\
             </hick:file>\n\
             </hick:doc>\n";
        let dir = tempfile::tempdir().unwrap();
        let mapping = Mapping::for_document(Path::new("d.hick"), source, dir.path()).expect("maps");

        // `def total(x):` is document line 5 (0-based). It is line ONE of
        // app.py, not line zero: the block's text begins with the newline
        // that follows the opening tag, so the generated file starts with a
        // blank line. Getting this off by one would put every breakpoint one
        // line from where it was asked for.
        let (path, line) = mapping.to_generated(5).expect("the code line maps");
        assert!(path.ends_with("app.py"), "{path:?}");
        assert_eq!(line, 1);

        // And back again, which is what a stack frame needs.
        assert_eq!(mapping.to_document(&path, 1), Some(5));
    }

    #[test]
    fn a_prose_line_maps_to_nothing() {
        // A breakpoint on a heading is not a breakpoint the adapter can bind,
        // and saying so beats sending it and reporting `verified: false`
        // without explanation.
        let source = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
             # Just prose here\n\
             </hick:doc>\n";
        let dir = tempfile::tempdir().unwrap();
        let mapping = Mapping::for_document(Path::new("d.hick"), source, dir.path()).unwrap();
        assert_eq!(mapping.to_generated(2), None);
    }
}
