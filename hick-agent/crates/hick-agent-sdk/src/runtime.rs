//! Agent runtime — the handle for writing an agent while loop.
//!
//! [`AgentRuntime`] manages conversation history, streaming LLM calls, sandbox
//! execution, and project context loading. It intentionally does **not** parse
//! LLM responses or decide what to do with them — that is the while loop body's
//! responsibility.
//!
//! # Example
//!
//! ```rust,no_run
//! use hick_agent_sdk::runtime::{AgentRuntime, Step};
//! use hick_agent_sdk::llm::{Message, Role};
//! use futures::StreamExt;
//!
//! async fn run(mut rt: AgentRuntime) {
//!     rt.add_user_message("Write a hello world script".into());
//!
//!     // complete_stream is 'static — stream and rt can coexist.
//!     let mut stream = rt.complete_stream().await.unwrap();
//!     let mut assembled = String::new();
//!     while let Some(step) = stream.next().await {
//!         match step.unwrap() {
//!             Step::AssistantChunk(chunk) => assembled.push_str(&chunk.delta),
//!             Step::AssistantDone(_) => break,
//!             _ => {}
//!         }
//!     }
//!     drop(stream);
//!
//!     // Now add the assembled response to history.
//!     rt.add_message(Message { role: Role::Assistant, content: assembled });
//! }
//! ```

use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;

use crate::command::{CommandExecutor, CommandOutput};
use crate::llm::{ChatChunk, LlmClient, Message, Role};
use crate::project_context::ProjectContextLoader;

// ---------------------------------------------------------------------------
// Step enum
// ---------------------------------------------------------------------------

/// A streaming token chunk from the LLM.
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Incremental text content.
    pub delta: String,
    /// Set when the stream is finishing (e.g. "stop", "end_turn").
    pub finish_reason: Option<String>,
}

impl From<ChatChunk> for StreamChunk {
    fn from(c: ChatChunk) -> Self {
        Self {
            delta: c.delta,
            finish_reason: c.finish_reason,
        }
    }
}

/// A structured tool call requested by the LLM.
#[derive(Debug, Clone)]
pub struct ToolCall {
    /// Provider-assigned call identifier.
    pub id: String,
    /// Tool name (e.g. "bash", "read_file").
    pub name: String,
    /// JSON-encoded arguments string.
    pub arguments: String,
}

/// Event emitted by [`AgentRuntime::complete_stream`] — what the while loop dispatches on.
///
/// The runtime streams chunks as they arrive, then emits `AssistantDone` with
/// the fully assembled message. The loop body decides what to do next.
#[derive(Debug)]
pub enum Step {
    /// A user message was added to the conversation.
    UserMessage(String),
    /// An incremental token chunk from the LLM.
    AssistantChunk(StreamChunk),
    /// The LLM response is complete; contains the fully assembled message.
    AssistantDone(Message),
    /// The LLM requested a tool call.
    ToolCall(ToolCall),
    /// Result from a sandbox command execution.
    SandboxResult(CommandOutput),
    /// A `! cmd` direct command from the user (bypasses LLM).
    DirectCommand(String),
    /// The context window is full and conversation must be compacted.
    ContextWindowFull,
    /// The agent has not made progress for `repeated_turns` consecutive turns.
    Stuck {
        repeated_turns: usize,
    },
    /// The agent loop is done; the caller should exit its while loop.
    Done,
}

// ---------------------------------------------------------------------------
// AgentRuntime
// ---------------------------------------------------------------------------

/// The handle for writing an agent while loop.
///
/// Manages conversation history, streaming LLM calls, sandbox lifecycle,
/// and project context loading. Multiple agent behaviors can be implemented
/// on top of this runtime by interpreting [`Step`] events differently.
pub struct AgentRuntime {
    llm: Box<dyn LlmClient>,
    sandbox: Box<dyn CommandExecutor>,
    history: Vec<Message>,
    context_loader: ProjectContextLoader,
    token_budget: Option<usize>,
    tokens_used: usize,
}

impl AgentRuntime {
    /// Create a new runtime.
    ///
    /// - `llm`: The LLM client to use for completions.
    /// - `sandbox`: The sandboxed command executor.
    /// - `project_dir`: Root of the project (used for context loading).
    pub fn new(
        llm: Box<dyn LlmClient>,
        sandbox: Box<dyn CommandExecutor>,
        project_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            llm,
            sandbox,
            history: Vec::new(),
            context_loader: ProjectContextLoader::new(project_dir),
            token_budget: None,
            tokens_used: 0,
        }
    }

    /// Set a soft token budget. When the estimated usage exceeds this the
    /// runtime will emit [`Step::ContextWindowFull`] on the next completion.
    pub fn set_token_budget(&mut self, budget: usize) {
        self.token_budget = Some(budget);
    }

    /// Estimated tokens used so far (character count / 4, a rough heuristic).
    pub fn tokens_used(&self) -> usize {
        self.tokens_used
    }

    // -----------------------------------------------------------------------
    // History management
    // -----------------------------------------------------------------------

    /// Append a message to conversation history.
    pub fn add_message(&mut self, message: Message) {
        self.tokens_used += message.content.len() / 4;
        self.history.push(message);
    }

    /// Append a user-role message.
    pub fn add_user_message(&mut self, content: String) {
        self.add_message(Message {
            role: Role::User,
            content,
        });
    }

    /// Append a system-role message.
    pub fn add_system_message(&mut self, content: String) {
        self.add_message(Message {
            role: Role::System,
            content,
        });
    }

    /// Read-only view of the current conversation history.
    pub fn history(&self) -> &[Message] {
        &self.history
    }

    /// Replace the entire history with a new set of messages.
    pub fn set_history(&mut self, messages: Vec<Message>) {
        self.history = messages;
    }

    /// Clear all history and reset the token counter.
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.tokens_used = 0;
    }

    /// Compact history by keeping the first `keep_system` system messages and
    /// the last `keep_tail` non-system messages, discarding the rest.
    ///
    /// Use this to prevent context-window overflow while retaining the original
    /// system prompt and recent context.
    pub fn compact_history(&mut self, keep_system: usize, keep_tail: usize) {
        let system: Vec<Message> = self
            .history
            .iter()
            .filter(|m| m.role == Role::System)
            .take(keep_system)
            .cloned()
            .collect();

        let non_system: Vec<Message> = self
            .history
            .iter()
            .filter(|m| m.role != Role::System)
            .cloned()
            .collect();

        let tail_start = non_system.len().saturating_sub(keep_tail);
        let tail = non_system[tail_start..].to_vec();

        self.history = system.into_iter().chain(tail).collect();
        self.tokens_used = self
            .history
            .iter()
            .map(|m| m.content.len() / 4)
            .sum();
    }

    // -----------------------------------------------------------------------
    // LLM streaming
    // -----------------------------------------------------------------------

    /// Stream a completion against the current history.
    ///
    /// Returns a stream of [`Step`] events. The stream emits
    /// [`Step::AssistantChunk`] for each incremental token, then
    /// [`Step::AssistantDone`] with the fully assembled response.
    ///
    /// The caller is responsible for adding the assembled message back to
    /// history via [`add_message`] if desired.
    ///
    /// Emits [`Step::ContextWindowFull`] immediately (without calling the LLM)
    /// when the token budget is exceeded.
    pub async fn complete_stream(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<Step>> + Send>>> {
        if let Some(budget) = self.token_budget {
            if self.tokens_used >= budget {
                let stream = futures::stream::once(async { Ok(Step::ContextWindowFull) });
                return Ok(Box::pin(stream));
            }
        }

        // Clone history so the returned stream is 'static and does not borrow self.
        let history = self.history.clone();
        let raw = self.llm.complete_stream(history).await?;
        let adapted = StepStream::new(raw);
        Ok(Box::pin(adapted))
    }

    // -----------------------------------------------------------------------
    // Sandbox
    // -----------------------------------------------------------------------

    /// Run a shell command in the sandbox and return its output.
    pub async fn run_command(
        &mut self,
        container: &str,
        command: &str,
        image: &str,
        timeout: Option<u64>,
    ) -> anyhow::Result<CommandOutput> {
        self.sandbox.execute(container, command, image, timeout).await
    }

    /// Inject a file into the sandbox container.
    pub async fn inject_file(
        &mut self,
        container: &str,
        image: &str,
        content: &[u8],
        dest_path: &str,
    ) -> anyhow::Result<()> {
        self.sandbox
            .inject_file(container, image, content, dest_path)
            .await
    }

    // -----------------------------------------------------------------------
    // Project context
    // -----------------------------------------------------------------------

    /// Load and return the project context string (AGENTS.md, CLAUDE.md, etc.).
    pub fn load_project_context(&self) -> String {
        self.context_loader.load()
    }

    /// Reference to the underlying context loader.
    pub fn context_loader(&self) -> &ProjectContextLoader {
        &self.context_loader
    }

    // -----------------------------------------------------------------------
    // Provider info
    // -----------------------------------------------------------------------

    /// Provider name of the underlying LLM client.
    pub fn provider_name(&self) -> &str {
        self.llm.provider_name()
    }

    /// Model name of the underlying LLM client.
    pub fn model_name(&self) -> &str {
        self.llm.model_name()
    }
}

// ---------------------------------------------------------------------------
// StepStream — wraps a ChatChunk stream into a Step stream
// ---------------------------------------------------------------------------

struct StepStream<S> {
    inner: S,
    accumulated: String,
    done_emitted: bool,
}

impl<S> StepStream<S> {
    fn new(inner: S) -> Self {
        Self {
            inner,
            accumulated: String::new(),
            done_emitted: false,
        }
    }
}

impl<S> Stream for StepStream<S>
where
    S: Stream<Item = anyhow::Result<ChatChunk>> + Unpin + Send,
{
    type Item = anyhow::Result<Step>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.done_emitted {
            return Poll::Ready(None);
        }

        match Pin::new(&mut this.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                this.accumulated.push_str(&chunk.delta);

                if chunk.finish_reason.is_some() {
                    // Emit AssistantDone and mark that the next poll returns None.
                    let msg = Message {
                        role: Role::Assistant,
                        content: std::mem::take(&mut this.accumulated),
                    };
                    this.done_emitted = true;
                    Poll::Ready(Some(Ok(Step::AssistantDone(msg))))
                } else if !chunk.delta.is_empty() {
                    Poll::Ready(Some(Ok(Step::AssistantChunk(StreamChunk {
                        delta: chunk.delta,
                        finish_reason: None,
                    }))))
                } else {
                    // Empty delta, no finish — skip.
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => {
                // Stream ended without a finish_reason marker — emit AssistantDone anyway.
                if !this.accumulated.is_empty() {
                    let msg = Message {
                        role: Role::Assistant,
                        content: std::mem::take(&mut this.accumulated),
                    };
                    this.done_emitted = true;
                    Poll::Ready(Some(Ok(Step::AssistantDone(msg))))
                } else {
                    Poll::Ready(None)
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandOutput;
    use async_trait::async_trait;
    use futures::{stream, StreamExt};

    struct MockLlm {
        response: String,
    }

    #[async_trait]
    impl LlmClient for MockLlm {
        async fn complete(&self, _messages: Vec<Message>) -> anyhow::Result<String> {
            Ok(self.response.clone())
        }

        fn provider_name(&self) -> &str {
            "mock"
        }
        fn model_name(&self) -> &str {
            "mock-1"
        }
    }

    struct NoopSandbox;

    #[async_trait]
    impl CommandExecutor for NoopSandbox {
        async fn execute(
            &mut self,
            _container: &str,
            _command: &str,
            _image: &str,
            _timeout: Option<u64>,
        ) -> anyhow::Result<CommandOutput> {
            Ok(CommandOutput {
                output: "ok".into(),
                exit_code: Some(0),
            })
        }
    }

    fn make_runtime(response: &str) -> AgentRuntime {
        let dir = tempfile::tempdir().unwrap();
        AgentRuntime::new(
            Box::new(MockLlm {
                response: response.to_string(),
            }),
            Box::new(NoopSandbox),
            dir.path(),
        )
    }

    #[test]
    fn add_and_read_history() {
        let mut rt = make_runtime("hi");
        rt.add_user_message("hello".into());
        assert_eq!(rt.history().len(), 1);
        assert_eq!(rt.history()[0].content, "hello");
    }

    #[test]
    fn clear_history_resets_tokens() {
        let mut rt = make_runtime("hi");
        rt.add_user_message("hello world".into());
        rt.clear_history();
        assert!(rt.history().is_empty());
        assert_eq!(rt.tokens_used(), 0);
    }

    #[test]
    fn compact_history_keeps_system_and_tail() {
        let mut rt = make_runtime("hi");
        rt.add_system_message("system".into());
        for i in 0..5 {
            rt.add_user_message(format!("msg {i}"));
        }
        rt.compact_history(1, 2);
        // 1 system + 2 tail
        assert_eq!(rt.history().len(), 3);
        assert_eq!(rt.history()[0].role, Role::System);
    }

    #[tokio::test]
    async fn complete_stream_emits_chunks_then_done() {
        let dir = tempfile::tempdir().unwrap();
        let chunks = vec![
            Ok(ChatChunk {
                delta: "Hello".into(),
                finish_reason: None,
            }),
            Ok(ChatChunk {
                delta: " world".into(),
                finish_reason: Some("stop".into()),
            }),
        ];

        struct ChunkLlm(Vec<anyhow::Result<ChatChunk>>);
        #[async_trait]
        impl LlmClient for ChunkLlm {
            async fn complete(&self, _: Vec<Message>) -> anyhow::Result<String> {
                Ok(String::new())
            }
            async fn complete_stream(
                &self,
                _: Vec<Message>,
            ) -> anyhow::Result<
                Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>,
            > {
                let v: Vec<anyhow::Result<ChatChunk>> = self
                    .0
                    .iter()
                    .map(|r| match r {
                        Ok(c) => Ok(c.clone()),
                        Err(e) => Err(anyhow::anyhow!("{}", e)),
                    })
                    .collect();
                Ok(Box::pin(stream::iter(v)))
            }
            fn provider_name(&self) -> &str {
                "test"
            }
            fn model_name(&self) -> &str {
                "test"
            }
        }

        let rt = AgentRuntime::new(Box::new(ChunkLlm(chunks)), Box::new(NoopSandbox), dir.path());
        let mut s = rt.complete_stream().await.unwrap();

        let first = s.next().await.unwrap().unwrap();
        assert!(matches!(first, Step::AssistantChunk(_)));
        if let Step::AssistantChunk(c) = first {
            assert_eq!(c.delta, "Hello");
        }

        let second = s.next().await.unwrap().unwrap();
        assert!(matches!(second, Step::AssistantDone(_)));
        if let Step::AssistantDone(msg) = second {
            assert_eq!(msg.content, "Hello world");
        }

        assert!(s.next().await.is_none());
    }

    #[tokio::test]
    async fn context_window_full_when_over_budget() {
        let mut rt = make_runtime("hi");
        rt.set_token_budget(1);
        // Add a long message so tokens_used (len/4) exceeds the budget of 1.
        rt.add_user_message("a".repeat(100));

        let mut s = rt.complete_stream().await.unwrap();
        let step = s.next().await.unwrap().unwrap();
        assert!(matches!(step, Step::ContextWindowFull));
    }

    #[tokio::test]
    async fn sandbox_execute_returns_output() {
        let mut rt = make_runtime("hi");
        let out = rt
            .run_command("ws", "echo hi", "alpine", None)
            .await
            .unwrap();
        assert_eq!(out.output, "ok");
        assert_eq!(out.exit_code, Some(0));
    }
}
