//! Agent Log XML session logging trait and event types.
//!
//! # Format
//!
//! Agent Log XML uses the namespace `xmlns:log="http://www.hickorydocs.com/agent-log/1.0"`.
//! This is a **purpose-built** namespace distinct from the Hickory pipeline namespace
//! (`http://www.hickorydocs.com/1.0`) — do not mix them.
//!
//! # Bundle directory layout
//!
//! A session log is a **directory bundle**:
//! - `<bundle-dir>/session.xml` — the root XML document with all event elements
//! - `<bundle-dir>/<filename>` — optional attachment files referenced by relative path
//!   from `LogEvent::Custom { attachments, .. }` records
//!
//! The bundle directory is created by [`XmlSessionLog`][crate::XmlSessionLog] at startup.
//! Extensions can append binary artifacts (images, generated files, etc.) as attachments
//! without encoding them in XML.
//!
//! # Element schema
//!
//! Every element carries a `time` attribute (RFC 3339). Session root carries `start`:
//!
//! ```xml
//! <log:session xmlns:log="http://www.hickorydocs.com/agent-log/1.0" start="2026-01-01T00:00:00Z">
//!   <log:user time="...">user message</log:user>
//!   <log:state-transition from="Idle" to="Thinking" time="..."/>
//!   <log:assistant time="...">llm response</log:assistant>
//!   <log:action lang="python" time="...">code here</log:action>
//!   <log:observation exit="0" time="...">stdout here</log:observation>
//!   <log:mcp-call server="github" method="tools/call" time="..."><![CDATA[...params json...]]></log:mcp-call>
//!   <log:mcp-response server="github" method="tools/call" time="..."><![CDATA[...result json...]]></log:mcp-response>
//!   <log:security-violation time="...">description</log:security-violation>
//!   <log:diagnostic level="info" target="hick_agent_sdk" time="...">message</log:diagnostic>
//!   <log:session-end time="..."/>
//! </log:session>
//! ```

use std::sync::Arc;

// ---------------------------------------------------------------------------
// Log level
// ---------------------------------------------------------------------------

/// Severity level for a [`LogEvent::DiagnosticLog`] record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

// ---------------------------------------------------------------------------
// Log attachment
// ---------------------------------------------------------------------------

/// A binary file to write into the session bundle directory alongside `session.xml`.
///
/// `filename` is a relative path within the bundle directory (e.g. `"image.png"`).
/// Extensions that produce binary artifacts pass these via [`LogEvent::Custom`].
pub struct LogAttachment {
    /// Relative path within the session bundle directory.
    pub filename: String,
    /// File content.
    pub data: Arc<[u8]>,
}

// ---------------------------------------------------------------------------
// Log event
// ---------------------------------------------------------------------------

/// A single event to be recorded into a session log.
///
/// All borrowed variants use `'a` so callers can pass references without cloning.
///
/// # State names
///
/// `StateTransition` accepts `&str` state names rather than the concrete
/// `AgentState` enum, allowing `hick-agent-sdk` to remain free of dependencies
/// on the engine crates.  Callers should format the state name with `format!("{:?}", state)`.
#[non_exhaustive]
pub enum LogEvent<'a> {
    /// Agent session started (first `run()` call).
    SessionStart {
        /// LLM model identifier.
        model: &'a str,
        /// Sandbox container image, or `None` for host execution.
        sandbox_image: Option<&'a str>,
        /// Maximum number of turns configured for this session.
        max_turns: usize,
    },

    /// User message appended to history.
    UserTurn { text: &'a str },

    /// Agent state machine transitioned between states.
    StateTransition { from: &'a str, to: &'a str },

    /// LLM produced a complete (accepted) response.
    LlmResponse { text: &'a str },

    /// Script execution about to begin.
    ActionStart { lang: &'a str, code: &'a str },

    /// Script execution completed.
    ActionEnd {
        stdout: &'a str,
        stderr: &'a str,
        exit_code: Option<i32>,
    },

    /// MCP tool call about to be sent to a server.
    McpCall {
        server: &'a str,
        method: &'a str,
        params: &'a serde_json::Value,
    },

    /// MCP tool call response received from a server.
    McpResponse {
        server: &'a str,
        method: &'a str,
        result: &'a serde_json::Value,
    },

    /// Security policy denied an action.
    SecurityViolation { description: &'a str },

    /// A `log::Record` forwarded from the diagnostic logger.
    DiagnosticLog {
        level: LogLevel,
        target: &'a str,
        message: &'a str,
    },

    /// Session ended (final `run()` return).
    SessionEnd,

    /// Escape hatch for extension crates: append an arbitrary XML fragment and
    /// optional binary attachment files to the bundle.
    ///
    /// The caller is responsible for the well-formedness of `xml_fragment`.
    /// Attachments are written to the bundle directory at the specified relative paths.
    Custom {
        xml_fragment: &'a str,
        attachments: &'a [LogAttachment],
    },
}

// ---------------------------------------------------------------------------
// SessionLog trait
// ---------------------------------------------------------------------------

/// Sink for structured agent session events.
///
/// Implementations must be `Send + Sync` so the log handle can be shared
/// across async tasks via `Arc<dyn SessionLog>`.
///
/// The only required method is [`record`][SessionLog::record]; implementations
/// are free to buffer, write synchronously, or discard events.
pub trait SessionLog: Send + Sync {
    fn record(&self, event: LogEvent<'_>);
}

// ---------------------------------------------------------------------------
// NullSessionLog
// ---------------------------------------------------------------------------

/// A no-op [`SessionLog`] that discards every event.
///
/// Used as the default when no log path is configured, and in unit tests
/// that don't need session output.
pub struct NullSessionLog;

impl SessionLog for NullSessionLog {
    fn record(&self, _event: LogEvent<'_>) {}
}
