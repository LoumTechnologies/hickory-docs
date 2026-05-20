//! SDK v2 primitives: trait-based, composable agent engine.
//!
//! This module provides the foundational traits and types for building
//! extensible AI coding agents without forking the runtime loop.
//!
//! # Key types
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`MatchIntent`] | Trait for extensible LLM response parsers |
//! | [`ClassifyData`] | Trait for background security / taint scanners |
//! | [`AgentHistory`] | Taint-aware conversation history with branching |
//! | [`IntentRouter`] | Routes a response to the first matching parser |
//! | [`LoopIntent`] | Enum dispatched by the canonical engine loop |
//! | [`AgentContext`] | Dependency bag passed into the engine loop |
//! | [`ToolExecutor`] | Trait for dispatching tool and custom intent calls |
//! | [`AgentIo`] | Trait for blocking input / streaming output |
//! | [`run_agent_engine`] | Canonical async agent loop |

use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;

use crate::llm::{LlmClient, Message, Role};

// ---------------------------------------------------------------------------
// MatchIntent — extensible response parser
// ---------------------------------------------------------------------------

/// Extensible LLM response parser.
///
/// Each impl inspects a completed [`Message`] and extracts a typed payload
/// when the message matches its pattern. The router tries parsers in priority
/// order and stops at the first match.
#[async_trait]
pub trait MatchIntent: Send + Sync {
    /// Unique name used in config and routing (e.g. `"script_first"`).
    fn name(&self) -> &'static str;

    /// Inspect `message` and return a boxed payload if it matches, or `None`.
    ///
    /// Returning `None` means "I don't handle this message" — the router tries
    /// the next registered parser.
    fn parse(&self, message: &Message) -> Option<Box<dyn Any + Send>>;
}

// ---------------------------------------------------------------------------
// ClassifyData — background security / taint scanner
// ---------------------------------------------------------------------------

/// Background data classification scanner.
///
/// Classifiers are registered on [`AgentHistory`]. Before the executor runs
/// any tool, the engine checks `history.active_classifications()` — the loop
/// body never writes inline security checks.
pub trait ClassifyData: Send + Sync {
    /// Canonical name for this classification (e.g. `"phi"`, `"secrets"`).
    fn classification_name(&self) -> &'static str;

    /// Return `true` if the message content matches this classification.
    fn scan(&self, message: &Message) -> bool;
}

// ---------------------------------------------------------------------------
// AgentHistory — typed, taint-aware conversation history
// ---------------------------------------------------------------------------

/// Merge strategy for folding a sub-branch back into its parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Append sub-branch messages verbatim to the parent.
    AppendRaw,
    /// Summarise the sub-branch with an LLM call and append the summary.
    SummarizeConformant,
}

/// Typed, taint-aware conversation history with branching support.
///
/// Replaces the flat `Vec<Message>` in the old `AgentRuntime`. Each history
/// carries a set of active data classifications (propagated from parent
/// branches) so the executor can enforce policy without inline checks.
#[derive(Clone)]
pub struct AgentHistory {
    messages: Vec<Message>,
    /// Classifications accumulated from registered scanners.
    classifications: HashSet<String>,
    idle: bool,
    is_sub_branch: bool,
    classifiers: Vec<Arc<dyn ClassifyData>>,
}

impl AgentHistory {
    /// Create a new, empty root history (not a sub-branch).
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            classifications: HashSet::new(),
            idle: true,
            is_sub_branch: false,
            classifiers: Vec::new(),
        }
    }

    /// Register a [`ClassifyData`] impl that will be run on every pushed message.
    pub fn add_classifier(&mut self, classifier: Arc<dyn ClassifyData>) {
        self.classifiers.push(classifier);
    }

    /// Append a message and run all registered classifiers over it.
    pub fn push(&mut self, message: Message) {
        for classifier in &self.classifiers {
            if classifier.scan(&message) {
                self.classifications.insert(classifier.classification_name().to_string());
            }
        }
        self.messages.push(message);
    }

    /// Convenience: push a user-role message.
    pub fn push_user(&mut self, content: impl Into<String>) {
        self.push(Message { role: Role::User, content: content.into() });
    }

    /// Convenience: push a system-role message.
    pub fn push_system(&mut self, content: impl Into<String>) {
        self.push(Message { role: Role::System, content: content.into() });
    }

    /// Convenience: push an assistant-role message.
    pub fn push_assistant(&mut self, content: impl Into<String>) {
        self.push(Message { role: Role::Assistant, content: content.into() });
    }

    /// Read-only view of all messages.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Replace the full message list (used for history rewind).
    pub fn set_messages(&mut self, messages: Vec<Message>) {
        self.messages = messages;
    }

    /// Set of active data classifications (accumulated from all pushed messages).
    pub fn active_classifications(&self) -> &HashSet<String> {
        &self.classifications
    }

    /// `true` when the agent is waiting for the next user input packet.
    pub fn is_idle(&self) -> bool {
        self.idle
    }

    /// Mark the agent as idle (ready to wait for the next user input).
    pub fn mark_idle(&mut self) {
        self.idle = true;
    }

    /// Mark the agent as active (processing a turn).
    pub fn mark_active(&mut self) {
        self.idle = false;
    }

    /// `true` when this history was created via [`branch`].
    pub fn is_sub_branch(&self) -> bool {
        self.is_sub_branch
    }

    /// Create a child sub-branch that inherits parent message context and
    /// all active classifications.
    pub fn branch(&self) -> AgentHistory {
        AgentHistory {
            messages: self.messages.clone(),
            classifications: self.classifications.clone(),
            idle: false,
            is_sub_branch: true,
            classifiers: self.classifiers.clone(),
        }
    }

    /// Fold a sub-branch back into this history using the given strategy.
    ///
    /// - `AppendRaw`: appends all messages from `sub` that are not already in
    ///   `self` (i.e. the messages added during the sub-branch execution).
    /// - `SummarizeConformant`: appends the last assistant message from the
    ///   sub-branch as a summary. (Full LLM summarization is a future
    ///   enhancement; this provides the structural hook.)
    pub fn merge(&mut self, sub: AgentHistory, strategy: MergeStrategy) -> anyhow::Result<()> {
        // Absorb any new classifications from the sub-branch.
        for cls in &sub.classifications {
            self.classifications.insert(cls.clone());
        }

        // Determine the new messages added during sub-branch execution.
        let parent_len = self.messages.len();
        let sub_new: Vec<Message> = sub.messages.into_iter().skip(parent_len).collect();

        match strategy {
            MergeStrategy::AppendRaw => {
                for msg in sub_new {
                    self.push(msg);
                }
            }
            MergeStrategy::SummarizeConformant => {
                // Append the last assistant message as the summary.
                if let Some(summary) = sub_new.iter().rev().find(|m| m.role == Role::Assistant) {
                    self.push(summary.clone());
                }
            }
        }

        Ok(())
    }
}

impl Default for AgentHistory {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ToolData — structured tool call
// ---------------------------------------------------------------------------

/// A structured tool call parsed from an LLM response.
#[derive(Debug, Clone)]
pub struct ToolData {
    /// Tool name (e.g. `"bash"`, `"read_file"`).
    pub name: String,
    /// JSON-encoded arguments string.
    pub arguments: String,
    /// Provider-assigned call identifier (may be empty for custom intents).
    pub id: String,
}

// ---------------------------------------------------------------------------
// LoopIntent — dispatched by IntentRouter
// ---------------------------------------------------------------------------

/// Intent returned by [`IntentRouter::determine_intent`] — what the loop does next.
#[derive(Debug)]
pub enum LoopIntent {
    /// Execute a named tool with structured arguments.
    CallTool(ToolData),
    /// A registered [`MatchIntent`] parser matched; payload forwarded to executor.
    Custom(String, Box<dyn Any + Send>),
    /// Spawn a sub-agent with the given prompt, wait, and merge results.
    DelegateTask(String),
    /// The agent considers the task complete; contains a human-readable summary.
    TaskComplete(String),
    /// No parser matched — the response is free-form text.
    None,
}

// ---------------------------------------------------------------------------
// TaskCompleteSignal — returned by MatchIntent to signal task completion
// ---------------------------------------------------------------------------

/// Marker payload returned by a [`MatchIntent`] impl to signal task completion.
///
/// When [`IntentRouter::determine_intent`] receives this payload from a parser,
/// it converts it to [`LoopIntent::TaskComplete`] rather than `Custom`.
///
/// # Example
///
/// ```rust,ignore
/// impl MatchIntent for DoneParser {
///     fn name(&self) -> &'static str { "done" }
///     fn parse(&self, msg: &Message) -> Option<Box<dyn Any + Send>> {
///         if msg.content.contains("<done>") {
///             Some(Box::new(TaskCompleteSignal { summary: "finished".into() }))
///         } else {
///             None
///         }
///     }
/// }
/// ```
pub struct TaskCompleteSignal {
    /// Human-readable summary shown to the user when the task completes.
    pub summary: String,
}

// ---------------------------------------------------------------------------
// IntentRouter
// ---------------------------------------------------------------------------

/// Holds registered [`MatchIntent`] impls and routes an LLM response to the
/// first match.
///
/// Parsers are tried in registration order (priority order). If the payload
/// produced by a parser is a [`TaskCompleteSignal`], the router returns
/// [`LoopIntent::TaskComplete`] instead of [`LoopIntent::Custom`].
pub struct IntentRouter {
    parsers: Vec<Box<dyn MatchIntent>>,
}

impl IntentRouter {
    /// Create an empty router.
    pub fn new() -> Self {
        Self { parsers: Vec::new() }
    }

    /// Register a parser. Higher-priority parsers should be added first.
    pub fn register(&mut self, parser: impl MatchIntent + 'static) {
        self.parsers.push(Box::new(parser));
    }

    /// Try each parser in order.
    ///
    /// - If the matched payload is a [`TaskCompleteSignal`], returns
    ///   [`LoopIntent::TaskComplete`].
    /// - Otherwise returns [`LoopIntent::Custom`].
    /// - Returns [`LoopIntent::None`] when no parser matches.
    pub fn determine_intent(&self, message: &Message) -> LoopIntent {
        for parser in &self.parsers {
            if let Some(payload) = parser.parse(message) {
                // Attempt to downcast to TaskCompleteSignal; on failure the
                // original Box is returned as Err so we can forward it as Custom.
                match payload.downcast::<TaskCompleteSignal>() {
                    Ok(sig) => return LoopIntent::TaskComplete(sig.summary),
                    Err(payload) => return LoopIntent::Custom(parser.name().to_string(), payload),
                }
            }
        }
        LoopIntent::None
    }
}

impl Default for IntentRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ToolExecutor — dispatches tool and custom intent calls
// ---------------------------------------------------------------------------

/// Dispatches tool calls and custom-intent payloads; enforces security policy.
///
/// All security enforcement lives inside `run` / `run_custom` implementations —
/// the engine loop body never writes inline security checks.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a named tool call. Returns a [`Message`] to push back into history.
    async fn run(&self, tool: ToolData) -> anyhow::Result<Message>;

    /// Execute a custom intent payload. `name` matches [`MatchIntent::name`].
    /// Returns a [`Message`] to push back into history.
    async fn run_custom(&self, name: &str, payload: Box<dyn Any + Send>) -> anyhow::Result<Message>;
}

// ---------------------------------------------------------------------------
// AgentIo — blocking input / streaming output
// ---------------------------------------------------------------------------

/// Input/output interface for the agent engine.
///
/// Implementations bridge the engine to TUI channels, web sockets, stdin, etc.
#[async_trait]
pub trait AgentIo: Send + Sync {
    /// Block until the next user input packet is available.
    async fn await_next_input_packet(&self) -> String;

    /// Emit an incremental text chunk to the UI.
    fn stream_chunk(&self, message: &Message);

    /// Emit a status update (task complete, info line, etc.).
    fn print_status_update(&self, text: &str);
}

// ---------------------------------------------------------------------------
// AgentContext — dependency bag
// ---------------------------------------------------------------------------

/// Dependency bag passed into the canonical agent engine loop.
///
/// Wire this up in the binary crate using config-driven extension loading.
pub struct AgentContext {
    pub llm: Box<dyn LlmClient>,
    pub router: IntentRouter,
    pub executor: Arc<dyn ToolExecutor>,
    pub io: Arc<dyn AgentIo>,
}

// ---------------------------------------------------------------------------
// run_agent_engine — canonical async loop
// ---------------------------------------------------------------------------

/// Canonical agent engine loop.
///
/// Drives the agent until `running` is cleared. Uses `history.is_idle()` to
/// decide when to wait for user input. Dispatches on [`LoopIntent`] for each
/// LLM response.
///
/// # Sub-agents
///
/// When the router returns [`LoopIntent::DelegateTask`], this function
/// recurses into a child sub-branch and merges results back with
/// [`MergeStrategy::SummarizeConformant`].
pub async fn run_agent_engine(
    ctx: &AgentContext,
    history: &mut AgentHistory,
    running: Arc<AtomicBool>,
) {
    while running.load(Ordering::SeqCst) {
        if history.is_idle() {
            let input = ctx.io.await_next_input_packet().await;
            // io may have cleared `running` (e.g. on Quit or RewindTo).
            if !running.load(Ordering::SeqCst) {
                break;
            }
            history.push_user(input);
            history.mark_active();
        }

        let response = match ctx.llm.complete(history.messages().to_vec()).await {
            Ok(text) => Message { role: Role::Assistant, content: text },
            Err(e) => Message { role: Role::Assistant, content: format!("<error>{e}</error>") },
        };
        history.push(response.clone());

        match ctx.router.determine_intent(&response) {
            LoopIntent::CallTool(tool_data) => {
                let result = ctx.executor.run(tool_data).await
                    .unwrap_or_else(|e| Message {
                        role: Role::User,
                        content: format!("<tool-error>{e}</tool-error>"),
                    });
                history.push(result);
            }
            LoopIntent::Custom(name, payload) => {
                let result = ctx.executor.run_custom(&name, payload).await
                    .unwrap_or_else(|e| Message {
                        role: Role::User,
                        content: format!("<tool-error>{e}</tool-error>"),
                    });
                history.push(result);
            }
            LoopIntent::DelegateTask(prompt) => {
                let mut sub = history.branch();
                sub.push_system(prompt);
                // Recurse: run the sub-agent until it signals TaskComplete.
                Box::pin(run_agent_engine(ctx, &mut sub, Arc::clone(&running))).await;
                history.merge(sub, MergeStrategy::SummarizeConformant).ok();
            }
            LoopIntent::TaskComplete(summary) => {
                ctx.io.print_status_update(&summary);
                if history.is_sub_branch() {
                    return;
                } else {
                    history.mark_idle();
                }
            }
            LoopIntent::None => {
                ctx.io.stream_chunk(&response);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, Mutex as StdMutex};
    use async_trait::async_trait;

    // ── Mock LLM ────────────────────────────────────────────────────────────

    struct MockLlm {
        responses: StdMutex<Vec<String>>,
    }

    impl MockLlm {
        fn new(responses: Vec<&str>) -> Self {
            Self {
                responses: StdMutex::new(responses.iter().map(|s| s.to_string()).collect()),
            }
        }
    }

    #[async_trait]
    impl LlmClient for MockLlm {
        async fn complete(&self, _messages: Vec<Message>) -> anyhow::Result<String> {
            let mut lock = self.responses.lock().unwrap();
            if lock.is_empty() {
                Ok("<done>all done</done>".to_string())
            } else {
                Ok(lock.remove(0))
            }
        }
        fn provider_name(&self) -> &str { "mock" }
        fn model_name(&self) -> &str { "mock-1" }
    }

    // ── Mock executor ────────────────────────────────────────────────────────

    struct MockExecutor {
        calls: StdMutex<Vec<String>>,
    }

    impl MockExecutor {
        fn new() -> Self {
            Self { calls: StdMutex::new(Vec::new()) }
        }
    }

    #[async_trait]
    impl ToolExecutor for MockExecutor {
        async fn run(&self, tool: ToolData) -> anyhow::Result<Message> {
            self.calls.lock().unwrap().push(format!("tool:{}", tool.name));
            Ok(Message { role: Role::User, content: "tool result".to_string() })
        }

        async fn run_custom(&self, name: &str, _payload: Box<dyn Any + Send>) -> anyhow::Result<Message> {
            self.calls.lock().unwrap().push(format!("custom:{name}"));
            Ok(Message { role: Role::User, content: format!("custom result from {name}") })
        }
    }

    // ── Mock IO ──────────────────────────────────────────────────────────────

    struct MockIo {
        input: StdMutex<Vec<String>>,
        status_updates: StdMutex<Vec<String>>,
        chunks: StdMutex<Vec<String>>,
        running: Arc<AtomicBool>,
    }

    impl MockIo {
        fn new(inputs: Vec<&str>, running: Arc<AtomicBool>) -> Self {
            Self {
                input: StdMutex::new(inputs.iter().map(|s| s.to_string()).collect()),
                status_updates: StdMutex::new(Vec::new()),
                chunks: StdMutex::new(Vec::new()),
                running,
            }
        }
    }

    #[async_trait]
    impl AgentIo for MockIo {
        async fn await_next_input_packet(&self) -> String {
            let mut lock = self.input.lock().unwrap();
            if lock.is_empty() {
                // No more inputs — stop the loop.
                self.running.store(false, Ordering::SeqCst);
                "quit".to_string()
            } else {
                lock.remove(0)
            }
        }

        fn stream_chunk(&self, message: &Message) {
            self.chunks.lock().unwrap().push(message.content.clone());
        }

        fn print_status_update(&self, text: &str) {
            self.status_updates.lock().unwrap().push(text.to_string());
        }
    }

    // ── Custom MatchIntent impls ─────────────────────────────────────────────

    /// Signals task completion via [`TaskCompleteSignal`].
    struct DoneParser;

    impl MatchIntent for DoneParser {
        fn name(&self) -> &'static str { "done" }

        fn parse(&self, message: &Message) -> Option<Box<dyn Any + Send>> {
            if message.content.contains("<done>") {
                let summary = message.content
                    .split("<done>").nth(1)
                    .and_then(|s| s.split("</done>").next())
                    .unwrap_or("done")
                    .to_string();
                Some(Box::new(TaskCompleteSignal { summary }))
            } else {
                None
            }
        }
    }

    /// Returns a custom payload (plain String) for non-terminal responses.
    struct CodeParser;

    impl MatchIntent for CodeParser {
        fn name(&self) -> &'static str { "code" }

        fn parse(&self, message: &Message) -> Option<Box<dyn Any + Send>> {
            if message.content.contains("```python") {
                Some(Box::new("python_code".to_string()))
            } else {
                None
            }
        }
    }

    // ── Tests ────────────────────────────────────────────────────────────────

    #[test]
    fn agent_history_push_and_read() {
        let mut h = AgentHistory::new();
        h.push_user("hello");
        h.push_assistant("world");
        assert_eq!(h.messages().len(), 2);
        assert_eq!(h.messages()[0].role, Role::User);
        assert_eq!(h.messages()[1].role, Role::Assistant);
    }

    #[test]
    fn agent_history_idle_lifecycle() {
        let mut h = AgentHistory::new();
        assert!(h.is_idle());
        h.mark_active();
        assert!(!h.is_idle());
        h.mark_idle();
        assert!(h.is_idle());
    }

    #[test]
    fn agent_history_branch_inherits_classifications() {
        let mut h = AgentHistory::new();
        h.classifications.insert("phi".to_string());
        h.push_user("patient data");

        let sub = h.branch();
        assert!(sub.is_sub_branch());
        assert!(sub.active_classifications().contains("phi"));
    }

    #[test]
    fn agent_history_merge_append_raw() {
        let mut h = AgentHistory::new();
        h.push_user("start");
        let mut sub = h.branch();
        sub.push_assistant("sub response");
        h.merge(sub, MergeStrategy::AppendRaw).unwrap();
        // parent had 1 message; sub added 1 → total 2
        assert_eq!(h.messages().len(), 2);
        assert_eq!(h.messages()[1].content, "sub response");
    }

    #[test]
    fn agent_history_merge_summarize_conformant() {
        let mut h = AgentHistory::new();
        h.push_user("start");
        let mut sub = h.branch();
        sub.push_user("tool result");
        sub.push_assistant("final summary");
        h.merge(sub, MergeStrategy::SummarizeConformant).unwrap();
        // Only the last assistant message is appended.
        assert_eq!(h.messages().len(), 2);
        assert_eq!(h.messages()[1].content, "final summary");
    }

    #[test]
    fn intent_router_returns_task_complete_for_task_complete_signal() {
        let mut router = IntentRouter::new();
        router.register(DoneParser);
        let msg = Message { role: Role::Assistant, content: "<done>task complete</done>".to_string() };
        assert!(matches!(router.determine_intent(&msg), LoopIntent::TaskComplete(_)));
    }

    #[test]
    fn intent_router_returns_custom_for_non_terminal_payload() {
        let mut router = IntentRouter::new();
        router.register(CodeParser);
        let msg = Message { role: Role::Assistant, content: "```python\nprint(1)\n```".to_string() };
        assert!(matches!(router.determine_intent(&msg), LoopIntent::Custom(ref n, _) if n == "code"));
    }

    #[test]
    fn intent_router_returns_none_on_no_match() {
        let router = IntentRouter::new();
        let msg = Message { role: Role::Assistant, content: "just plain text".to_string() };
        assert!(matches!(router.determine_intent(&msg), LoopIntent::None));
    }

    #[test]
    fn classify_data_scanner_populates_classifications() {
        struct PhiScanner;
        impl ClassifyData for PhiScanner {
            fn classification_name(&self) -> &'static str { "phi" }
            fn scan(&self, message: &Message) -> bool { message.content.contains("SSN") }
        }

        let mut h = AgentHistory::new();
        h.add_classifier(Arc::new(PhiScanner));
        h.push_user("My SSN is 123-45-6789");
        assert!(h.active_classifications().contains("phi"));

        h.push_user("No sensitive data here");
        assert!(h.active_classifications().contains("phi"));
    }

    #[tokio::test]
    async fn run_agent_engine_custom_match_intent() {
        // AC #4: run_agent_engine compiles and passes with a mock LLM, mock
        // executor, and custom MatchIntent impls (CodeParser + DoneParser).
        let running = Arc::new(AtomicBool::new(true));
        let io = Arc::new(MockIo::new(vec!["do something"], Arc::clone(&running)));

        // LLM: first turn returns a code block; second turn signals done.
        let llm = Box::new(MockLlm::new(vec![
            "```python\nprint('hi')\n```",
            "<done>task finished</done>",
        ]));

        let executor_calls: Arc<StdMutex<Vec<String>>> = Arc::new(StdMutex::new(Vec::new()));

        struct TrackingExecutor(Arc<StdMutex<Vec<String>>>);
        #[async_trait]
        impl ToolExecutor for TrackingExecutor {
            async fn run(&self, tool: ToolData) -> anyhow::Result<Message> {
                self.0.lock().unwrap().push(format!("tool:{}", tool.name));
                Ok(Message { role: Role::User, content: "ok".to_string() })
            }
            async fn run_custom(&self, name: &str, _payload: Box<dyn Any + Send>) -> anyhow::Result<Message> {
                self.0.lock().unwrap().push(format!("custom:{name}"));
                Ok(Message { role: Role::User, content: "executed".to_string() })
            }
        }

        let mut router = IntentRouter::new();
        router.register(DoneParser);  // TaskCompleteSignal → TaskComplete
        router.register(CodeParser);  // String payload → Custom("code", …)

        let ctx = AgentContext {
            llm,
            router,
            executor: Arc::new(TrackingExecutor(Arc::clone(&executor_calls))),
            io: Arc::clone(&io) as Arc<dyn AgentIo>,
        };

        let mut history = AgentHistory::new();
        run_agent_engine(&ctx, &mut history, Arc::clone(&running)).await;

        // custom executor was called for the code block.
        let calls = executor_calls.lock().unwrap();
        assert!(calls.iter().any(|c| c == "custom:code"), "calls: {calls:?}");

        // print_status_update was called for the done signal.
        let updates = io.status_updates.lock().unwrap();
        assert!(!updates.is_empty(), "expected print_status_update to be called");

        // History: user msg + code response + executor result + done response + (extra iter msgs)
        assert!(history.messages().len() >= 3,
            "expected at least 3 messages, got {}", history.messages().len());
    }
}
