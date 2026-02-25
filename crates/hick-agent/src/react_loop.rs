//! ReAct loop: orchestrate LLM and tool execution.

use std::collections::HashMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use futures::StreamExt;

use crate::context;
use crate::file_io::{load_hick_sources, write_generated_files, write_hick_sources};
use crate::llm::{Message, Role};
use crate::state::AgentState;
use crate::tools::{ToolCall, ToolResult, parse_tool_calls};

use hick_agent_core::{QueryMatch, SourceEdit};
use hick_lens::edit::EditConfidence;

use crate::agent::Agent;

const SYSTEM_PROMPT: &str = r#"You are an AI coding agent for projects using Hick (hickory docs) - an XML format for defining containers, file outputs, and code generation.

You have access to these tools. Output a JSON object per tool call. Example:
{"tool": "exec", "command": "cargo test"}
{"tool": "query", "selector": "fn main", "file": "src/main.rs"}
{"tool": "compute_edits", "file": "src/main.rs", "selector": "fn greet", "new_content": "fn greet() { println!(\"Hi\"); }"}

When done, either output plain text (your final answer) or {"tool": "done", "message": "your summary"}.

Tools:
- run_pipeline: Run the Hick pipeline on loaded .hick sources. Call after editing hick files. params: optional list of "key=value"
- exec: Run shell command in sandbox. command (required), container, image, timeout (seconds)
- query: Find elements in generated output. selector: fn name, struct Name, #id, .class. file: optional filter
- structure: Get structural summary of a generated file
- provenance: Get provenance for lines start_line..end_line in file
- compute_edits: Compute edits to apply. Returns JSON edits to pass to apply_edits
- apply_edits: Apply edits to hick files. edits_json: JSON array from compute_edits
- read_file: Read file from project
- write_file: Write file (use for generated files only when not using hick pipeline)
- list_dir: List directory contents
- create_backing: When a generated file was edited directly, create a backing .hick. output_path: generated file path, backing_path: where to write the .hick
"#;

/// ReAct loop runner.
pub struct ReactLoop {
    pub agent: Agent,
    pub project_dir: PathBuf,
    output_dir: Option<String>,
    hick_sources: HashMap<String, String>,
    max_turns: usize,
    /// Whether to stream LLM output to stdout.
    pub stream_output: bool,
}

impl ReactLoop {
    pub fn new(agent: Agent, project_dir: PathBuf, output_dir: Option<String>) -> Self {
        Self {
            agent,
            project_dir,
            output_dir,
            hick_sources: HashMap::new(),
            max_turns: 50,
            stream_output: true,
        }
    }

    /// Load hick sources from disk.
    pub fn load_sources(&mut self) -> anyhow::Result<()> {
        self.hick_sources = load_hick_sources(&self.project_dir)?;
        Ok(())
    }

    /// Run the pipeline and cache results.
    pub async fn run_pipeline(
        &mut self,
        params: &[(String, String)],
    ) -> anyhow::Result<HashMap<String, String>> {
        let sources: Vec<(String, String)> = self
            .hick_sources
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        self.agent.run_pipeline(&sources, params).await
    }

    /// Execute a single tool call that requires agent state.
    pub async fn execute_tool(&mut self, call: ToolCall) -> ToolResult {
        match call {
            ToolCall::RunPipeline { params } => {
                let params: Vec<(String, String)> = params
                    .iter()
                    .filter_map(|p| {
                        let mut s = p.splitn(2, '=');
                        Some((s.next()?.to_string(), s.next().unwrap_or("").to_string()))
                    })
                    .collect();
                match self.run_pipeline(&params).await {
                    Ok(files) => {
                        if let Err(e) = write_generated_files(
                            &self.project_dir,
                            self.output_dir.as_deref(),
                            &files,
                        ) {
                            ToolResult::err("run_pipeline", format!("write failed: {e}"))
                        } else {
                            let summary: Vec<String> = files.keys().cloned().collect();
                            ToolResult::ok(
                                "run_pipeline",
                                format!("Generated {} files: {:?}", files.len(), summary),
                            )
                        }
                    }
                    Err(e) => ToolResult::err("run_pipeline", format!("{e}")),
                }
            }
            ToolCall::Exec {
                command,
                container,
                image,
                timeout,
            } => {
                match self
                    .agent
                    .exec(&command, container.as_deref(), image.as_deref(), timeout)
                    .await
                {
                    Ok(out) => ToolResult::ok("exec", out),
                    Err(e) => ToolResult::err("exec", format!("{e}")),
                }
            }
            ToolCall::Query { selector, file } => {
                match self.agent.query(&selector, file.as_deref()).await {
                    Ok(matches) => {
                        let out = format_query_matches(&matches);
                        ToolResult::ok("query", out)
                    }
                    Err(e) => ToolResult::err("query", format!("{e}")),
                }
            }
            ToolCall::Structure { file } => match self.agent.structure(&file).await {
                Ok(s) => ToolResult::ok("structure", s),
                Err(e) => ToolResult::err("structure", format!("{e}")),
            },
            ToolCall::Provenance {
                file,
                start_line,
                end_line,
            } => match self.agent.provenance(&file, start_line, end_line).await {
                Ok(s) => ToolResult::ok("provenance", s),
                Err(e) => ToolResult::err("provenance", format!("{e}")),
            },
            ToolCall::ComputeEdits {
                file,
                selector,
                new_content,
            } => {
                match self
                    .agent
                    .compute_edits(&file, &selector, &new_content)
                    .await
                {
                    Ok(edits) => {
                        let serializable: Vec<EditOutput> = edits
                            .iter()
                            .map(|e| EditOutput {
                                file: e.file.to_string(),
                                source_start: e.source_start,
                                source_end: e.source_end,
                                new_content: e.new_content.clone(),
                            })
                            .collect();
                        let json = serde_json::to_string(&serializable)
                            .unwrap_or_else(|_| "[]".to_string());
                        ToolResult::ok("compute_edits", json)
                    }
                    Err(e) => ToolResult::err("compute_edits", format!("{e}")),
                }
            }
            ToolCall::ApplyEdits { edits_json } => {
                let edits: Vec<EditInput> = match serde_json::from_str(&edits_json) {
                    Ok(e) => e,
                    Err(e) => return ToolResult::err("apply_edits", format!("invalid JSON: {e}")),
                };
                let source_edits: Vec<SourceEdit> = edits
                    .into_iter()
                    .map(|e| SourceEdit {
                        file: std::sync::Arc::from(e.file),
                        source_start: e.source_start,
                        source_end: e.source_end,
                        new_content: e.new_content,
                        confidence: EditConfidence::Exact,
                    })
                    .collect();
                let updated = Agent::apply_edits(&self.hick_sources, &source_edits);
                self.hick_sources = updated;
                match write_hick_sources(&self.project_dir, &self.hick_sources) {
                    Ok(()) => ToolResult::ok("apply_edits", "Edits applied and written to disk"),
                    Err(e) => ToolResult::err("apply_edits", format!("write failed: {e}")),
                }
            }
            ToolCall::ReadFile { path } => {
                let full = self.project_dir.join(&path);
                match std::fs::read_to_string(&full) {
                    Ok(c) => ToolResult::ok("read_file", c),
                    Err(e) => ToolResult::err("read_file", format!("{e}")),
                }
            }
            ToolCall::WriteFile { path, content } => {
                let full = self.project_dir.join(&path);
                if let Some(p) = full.parent() {
                    let _ = std::fs::create_dir_all(p);
                }
                match std::fs::write(&full, content) {
                    Ok(()) => ToolResult::ok("write_file", "written"),
                    Err(e) => ToolResult::err("write_file", format!("{e}")),
                }
            }
            ToolCall::CreateBacking {
                output_path,
                backing_path,
            } => {
                let full = self.project_dir.join(&output_path);
                let content = match std::fs::read_to_string(&full) {
                    Ok(c) => c,
                    Err(e) => {
                        return ToolResult::err("create_backing", format!("read failed: {e}"));
                    }
                };
                let backing_content =
                    crate::backing::create_backing_file(&output_path, &content, &self.project_dir);
                let backing_full = self.project_dir.join(&backing_path);
                if let Some(p) = backing_full.parent() {
                    let _ = std::fs::create_dir_all(p);
                }
                match std::fs::write(&backing_full, backing_content) {
                    Ok(()) => ToolResult::ok("create_backing", format!("Created {}", backing_path)),
                    Err(e) => ToolResult::err("create_backing", format!("write failed: {e}")),
                }
            }
            ToolCall::ListDir { path } => {
                let full = self.project_dir.join(&path);
                match std::fs::read_dir(&full) {
                    Ok(entries) => {
                        let names: Vec<String> = entries
                            .filter_map(|e| e.ok())
                            .map(|e| e.file_name().to_string_lossy().to_string())
                            .collect();
                        ToolResult::ok("list_dir", names.join("\n"))
                    }
                    Err(e) => ToolResult::err("list_dir", format!("{e}")),
                }
            }
            ToolCall::Done { message } => ToolResult::ok("done", message.unwrap_or_default()),
        }
    }

    /// Execute a batch of tool calls, running filesystem-only ones in parallel.
    async fn execute_tools(&mut self, tool_calls: Vec<ToolCall>) -> Vec<ToolResult> {
        let mut fs_tasks = Vec::new();
        let mut sequential_calls = Vec::new();
        let mut indexed_results: Vec<(usize, ToolResult)> = Vec::new();

        for (idx, call) in tool_calls.into_iter().enumerate() {
            if call.is_filesystem_only() {
                fs_tasks.push((idx, call));
            } else {
                sequential_calls.push((idx, call));
            }
        }

        // Run filesystem-only tools in parallel
        if !fs_tasks.is_empty() {
            let mut set = tokio::task::JoinSet::new();
            for (idx, call) in fs_tasks {
                let dir = self.project_dir.clone();
                set.spawn(async move { (idx, execute_fs_tool(call, &dir).await) });
            }
            while let Some(result) = set.join_next().await {
                match result {
                    Ok(pair) => indexed_results.push(pair),
                    Err(e) => {
                        indexed_results.push((usize::MAX, ToolResult::err("join", format!("{e}"))))
                    }
                }
            }
        }

        // Run remaining tools sequentially (they need agent state)
        for (idx, call) in sequential_calls {
            let result = self.execute_tool(call).await;
            indexed_results.push((idx, result));
        }

        // Sort by original index to preserve tool call ordering
        indexed_results.sort_by_key(|(idx, _)| *idx);
        indexed_results.into_iter().map(|(_, r)| r).collect()
    }

    /// Run one full ReAct turn: get LLM response, execute tools, return new state.
    pub async fn step(
        &mut self,
        state: AgentState,
        messages: &mut Vec<Message>,
    ) -> anyhow::Result<AgentState> {
        match state {
            AgentState::Thinking { .. } => {
                let response = if self.stream_output {
                    let mut stream = self.agent.llm_stream(messages.clone()).await?;
                    let mut full = String::new();
                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk?;
                        if !chunk.delta.is_empty() {
                            print!("{}", chunk.delta);
                            std::io::stdout().flush().ok();
                            full.push_str(&chunk.delta);
                        }
                    }
                    println!();
                    full
                } else {
                    self.agent.llm_complete(messages.clone()).await?
                };

                let tool_calls = parse_tool_calls(&response);

                if tool_calls.is_empty() {
                    return Ok(AgentState::to_done(response.trim().to_string()));
                }

                let has_done = tool_calls
                    .iter()
                    .any(|c| matches!(c, ToolCall::Done { .. }));
                if has_done {
                    let msg = tool_calls
                        .iter()
                        .find_map(|c| {
                            if let ToolCall::Done { message } = c {
                                message.clone()
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| response.trim().to_string());
                    return Ok(AgentState::to_done(msg));
                }

                let thought = if response.contains('{') {
                    Some(response.split('{').next().unwrap_or("").trim().to_string())
                } else {
                    None
                };

                Ok(AgentState::to_acting(tool_calls, thought))
            }
            AgentState::Acting { tool_calls, .. } => {
                // Check for done before executing
                for call in &tool_calls {
                    if let ToolCall::Done { message } = call {
                        return Ok(AgentState::to_done(
                            message.clone().unwrap_or_default(),
                        ));
                    }
                }

                let results = self.execute_tools(tool_calls).await;

                // Check if any tool result is "done"
                for r in &results {
                    if r.tool == "done" {
                        return Ok(AgentState::to_done(r.output.clone()));
                    }
                }

                let results_text = results
                    .iter()
                    .map(|r| {
                        format!(
                            "[{}] {}: {}",
                            if r.success { "ok" } else { "err" },
                            r.tool,
                            r.output
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n");

                messages.push(Message {
                    role: Role::Assistant,
                    content: format!("Tool results:\n{results_text}"),
                });
                messages.push(Message {
                    role: Role::User,
                    content: "Continue. Use more tools if needed, or provide your final answer."
                        .to_string(),
                });

                Ok(AgentState::to_observing(results))
            }
            _ => Ok(state),
        }
    }

    /// Run the full loop until done or max turns.
    pub async fn run(&mut self, user_message: &str) -> anyhow::Result<String> {
        self.load_sources()?;

        // Load standards-based agent context (AGENTS.md, .agents/, CLAUDE.md)
        let agent_context = context::load_agent_context(&self.project_dir);
        let system = if agent_context.is_empty() {
            SYSTEM_PROMPT.to_string()
        } else {
            format!("{SYSTEM_PROMPT}\n\n---\n\n{agent_context}")
        };

        let mut messages = vec![
            Message {
                role: Role::System,
                content: system,
            },
            Message {
                role: Role::User,
                content: format!(
                    "Project: {}\n\nHick sources loaded: {} files.\n\nUser request: {}",
                    self.project_dir.display(),
                    self.hick_sources.len(),
                    user_message
                ),
            },
        ];

        let mut state = AgentState::Thinking { messages_sent: 2 };
        let mut turns = 0;

        while !state.is_terminal() && turns < self.max_turns {
            state = self.step(state, &mut messages).await?;
            turns += 1;

            if let AgentState::Observing { .. } = state {
                state = AgentState::Thinking {
                    messages_sent: messages.len(),
                };
            }
        }

        match state {
            AgentState::Done { message } => Ok(message),
            AgentState::Error { message } => Err(anyhow::anyhow!("Agent error: {message}")),
            _ => Err(anyhow::anyhow!("Max turns ({}) exceeded", self.max_turns)),
        }
    }
}

/// Execute a filesystem-only tool without needing agent state.
/// These can safely run in parallel via tokio tasks.
async fn execute_fs_tool(call: ToolCall, project_dir: &Path) -> ToolResult {
    match call {
        ToolCall::ReadFile { path } => {
            let full = project_dir.join(&path);
            match tokio::fs::read_to_string(&full).await {
                Ok(c) => ToolResult::ok("read_file", c),
                Err(e) => ToolResult::err("read_file", format!("{e}")),
            }
        }
        ToolCall::ListDir { path } => {
            let full = project_dir.join(&path);
            match tokio::fs::read_dir(&full).await {
                Ok(mut entries) => {
                    let mut names = Vec::new();
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        names.push(entry.file_name().to_string_lossy().to_string());
                    }
                    ToolResult::ok("list_dir", names.join("\n"))
                }
                Err(e) => ToolResult::err("list_dir", format!("{e}")),
            }
        }
        _ => ToolResult::err("internal", "not a filesystem tool"),
    }
}

fn format_query_matches(matches: &[QueryMatch]) -> String {
    if matches.is_empty() {
        return "No matches".to_string();
    }
    matches
        .iter()
        .map(|m| {
            format!(
                "{} '{}' in {} [L{}-L{}]\n{}\nOrigins: {:?}",
                m.element_kind,
                m.element_name,
                m.file,
                m.start_line,
                m.end_line,
                m.output_text,
                m.origins
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[derive(serde::Serialize)]
struct EditOutput {
    file: String,
    source_start: usize,
    source_end: usize,
    new_content: String,
}

#[derive(serde::Deserialize)]
struct EditInput {
    file: String,
    #[serde(alias = "start")]
    source_start: usize,
    #[serde(alias = "end")]
    source_end: usize,
    #[serde(alias = "content")]
    new_content: String,
}
