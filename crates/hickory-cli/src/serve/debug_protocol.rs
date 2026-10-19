//! Debug wire types, shared by the socket bridge and its clients.
use hick_dap::{Breakpoint, Step};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// What the app asks for.
#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    /// Start a session over a document, with breakpoints in document lines.
    Start {
        doc: String,
        #[serde(default)]
        breakpoints: Vec<Breakpoint>,
        /// Which generated file to run. Omitted means "the first one that can
        /// be debugged", which is only ever right by accident once a document
        /// generates two.
        #[serde(default)]
        program: Option<String>,
        #[serde(default)]
        project: Option<String>,
        #[serde(default)]
        profile: Option<crate::debug_sessions::ProfileSelection>,
        #[serde(default)]
        sources: std::collections::HashMap<String, Vec<Breakpoint>>,
    },
    /// Replace the breakpoint set while a session is running.
    Breakpoints {
        session: String,
        breakpoints: Vec<Breakpoint>,
    },
    SourceBreakpoints {
        session: String,
        source: String,
        breakpoints: Vec<Breakpoint>,
    },
    /// Revision/stop-bound action; conveys no evaluation permission.
    Located {
        session: String,
        location: String,
        scope: hick_observe::reading::Scope,
        how: LocatedAction,
    },
    /// Where are we, and what is in scope.
    State {
        session: String,
    },
    /// Evaluate in a frame. `context` is DAP's: `hover`, `watch` or `repl`.
    Eval {
        session: String,
        expression: String,
        #[serde(default)]
        frame: Option<i64>,
        #[serde(default = "repl")]
        context: String,
    },
    /// Move. `how` is the session API's own verb.
    Step {
        session: String,
        how: Step,
        #[serde(default)]
        frame: Option<i64>,
    },
    /// Move the instruction pointer to a document line in this frame.
    Jump {
        session: String,
        line: u32,
    },
    /// Run to a line without leaving a breakpoint behind.
    RunTo {
        session: String,
        line: u32,
    },
    /// Expand one composite value.
    Children {
        session: String,
        reference: i64,
    },
    /// Stop when an exception is thrown, by the adapter's own filter ids.
    ///
    /// The ids are never invented here: they come from the adapter's
    /// `exceptionBreakpointFilters`, which is how a Python "raised" differs
    /// from a Java "caught". A client sends back the subset it wants on.
    SetExceptionBreakpoints {
        session: String,
        #[serde(default)]
        filters: Vec<String>,
    },
    /// Change a value in the running program.
    SetVariable {
        session: String,
        container: i64,
        name: String,
        value: String,
    },
    Stop {
        session: String,
    },
}

impl Request {
    pub fn session_id(&self) -> Option<&str> {
        match self {
            Self::Start { .. } => None,
            Self::Located { session, .. }
            | Self::Breakpoints { session, .. }
            | Self::SourceBreakpoints { session, .. }
            | Self::State { session }
            | Self::Eval { session, .. }
            | Self::Step { session, .. }
            | Self::Jump { session, .. }
            | Self::RunTo { session, .. }
            | Self::Children { session, .. }
            | Self::SetExceptionBreakpoints { session, .. }
            | Self::SetVariable { session, .. }
            | Self::Stop { session } => Some(session),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocatedAction {
    RunTo,
    Jump,
}

fn repl() -> String {
    "repl".to_string()
}

/// What the app is told.
#[derive(Debug, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Response {
    Reading {
        session: String,
        reading: hick_observe::reading::Reading,
    },
    Started {
        session: String,
        /// The document (or plain file) this session runs, exactly as the
        /// client named it in `start`. The workspace socket is shared by
        /// every plain-file pane, so an answer must say which pane it is
        /// for; a document's own socket carries it too, harmlessly.
        doc: String,
        capabilities: hick_dap::Capabilities,
        breakpoints: Vec<hick_dap::BreakpointStatus>,
    },
    /// Where execution is now — the paused line the gutter marks, and the
    /// values the editor shows inline.
    Stopped {
        observation_scope: Option<hick_observe::reading::Scope>,
        session: String,
        reason: String,
        frames: Vec<hick_dap::Frame>,
        variables: Vec<hick_dap::Variable>,
        /// The top frame's document line, hoisted so the editor need not
        /// hunt for it.
        line: Option<u32>,
    },
    Breakpoints {
        session: String,
        breakpoints: Vec<hick_dap::BreakpointStatus>,
    },
    SourceBreakpoints {
        session: String,
        source: String,
        breakpoints: Vec<hick_dap::BreakpointStatus>,
    },
    Value {
        session: String,
        expression: String,
        value: String,
        #[serde(rename = "type")]
        type_name: Option<String>,
        reference: i64,
    },
    Children {
        session: String,
        reference: i64,
        variables: Vec<hick_dap::Variable>,
    },
    /// The program ended. Not an error: it is what `continue` usually does.
    ///
    /// The session is reaped WITH it — adapter process killed, scratch clone
    /// deleted, registry entry gone — because a debugger over a program that
    /// no longer exists is a process holding a workdir open. The id in here
    /// no longer answers; the client keeps its breakpoints and starts a new
    /// session to run again.
    Finished {
        session: String,
        /// The debuggee's exit code, when the adapter reported one.
        exit_code: Option<i64>,
    },
    Ended {
        session: String,
    },
    /// A build that had to happen before there was a program to launch.
    ///
    /// Carried as `TranscriptEvent`-shaped events on purpose: the client
    /// already has a terminal that renders exactly those
    /// (`terminal/WatchingTerminal.tsx`), and a build tool's ANSI colour and
    /// carriage-return rewriting are the whole reason it is a terminal and
    /// not a text card. This is NOT a transcript: nothing here is recorded
    /// under a cache key, woven, or compared. The terminal is the run
    /// happening; the transcript is the record.
    ///
    /// It arrives BEFORE `started` on success and before `failed` on
    /// failure, and the failing case is the one it exists for — a build that
    /// fails says why in MSBuild's own words, and "build failed" throws all
    /// of that away.
    Build {
        /// Which start this build belongs to, as the client named it.
        doc: String,
        events: Vec<Value>,
    },
    /// Something did not work, in words a person can act on.
    Failed {
        session: Option<String>,
        /// The document a failed START was for, so a pane on a shared
        /// socket can tell its own refusal from a neighbour's. Absent when
        /// the failure has a session to name instead.
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
        message: String,
        /// Which request failed, so the app can put the message where it
        /// belongs. A failure with no home becomes a banner that outlives its
        /// cause and attaches itself to whatever the person does next.
        #[serde(skip_serializing_if = "Option::is_none")]
        about: Option<String>,
        /// The document lines the failed request was about, when it was about
        /// lines: a breakpoint that could not be set is a fact about that
        /// breakpoint, shown on it.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        lines: Vec<u32>,
        /// The one failure a person can fix from here: no debug adapter for
        /// this language, and hick knows how to fetch one.
        ///
        /// Carried as a FIELD rather than left in the message, because a
        /// button is the difference between "go and type this in a terminal"
        /// and "yes, do that". Absent when the language is one hick cannot
        /// install — the sentence then names the ecosystem's own way and is
        /// all there is to say.
        #[serde(skip_serializing_if = "Option::is_none")]
        offer_install: Option<InstallOffer>,
    },
}

/// A missing tool the app can offer to fetch.
#[derive(Debug, Clone, Serialize)]
pub struct InstallOffer {
    /// What to install — always `"dap"` today; the shape is here because
    /// `hick lsp install` and `hick index install` are the same gesture.
    pub kind: String,
    pub language: String,
}
