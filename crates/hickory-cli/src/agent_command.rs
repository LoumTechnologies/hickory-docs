//! Agent CLI selection, kept apart from document commands.
use anyhow::{Context, Result};
use hickory_cli::ExecutorChoice;
use std::path::PathBuf;
use std::process::ExitCode;
#[derive(Clone, clap::ValueEnum)]
enum Runtime {
    Rust,
    Python,
}

#[derive(clap::Args)]
pub(crate) struct AgentArgs {
    /// The task prompt for the agent.
    prompt: String,
    /// A `.md` document to give the agent as context.
    #[arg(long = "doc")]
    doc: Option<PathBuf>,
    /// Project directory (sessions land in `<dir>/sessions/`; default: cwd).
    #[arg(long = "dir")]
    dir: Option<PathBuf>,
    /// Model id override (default: the provider's default model).
    #[arg(long = "model")]
    model: Option<String>,
    /// LLM provider: anthropic, openai, deepseek, grok, openrouter, or gab. Defaults to
    /// HICKORY_LLM_PROVIDER, else to whichever provider's API key is set
    /// (anthropic when several are).
    #[arg(long = "provider")]
    provider: Option<String>,
    /// Agent orchestration runtime; Python is experimental and confined.
    #[arg(long, value_enum, default_value = "rust")]
    runtime: Runtime,
    /// Absolute venv Python path; required with --runtime python.
    #[arg(long, required_if_eq("runtime", "python"))]
    python: Option<PathBuf>,
    /// Maximum LLM turns before giving up.
    #[arg(long = "max-turns", default_value_t = 20)]
    max_turns: usize,
}

pub(crate) async fn run(args: AgentArgs) -> Result<ExitCode> {
    anyhow::ensure!(
        !matches!(args.runtime, Runtime::Rust) || args.python.is_none(),
        "--python requires --runtime python; refusing to run a different runtime"
    );
    use std::io::Write as _;

    use hickory_agent::{
        AgentConfig, AgentEvent, KeyStore, client_for_with_store, resolve_selector_with_store,
        run_agent,
    };

    let project_dir = match &args.dir {
        Some(dir) => dir.clone(),
        None => std::env::current_dir()?,
    };
    // With no --provider, the environment decides: HICKORY_LLM_PROVIDER,
    // else the provider whose key is present. `client_for` then reports an
    // unknown provider or a missing key by name, before anything is executed.
    // A key entered in the desktop app's Settings counts here too: the same
    // file is read, before the environment, so one machine has one answer to
    // "which key".
    let store = KeyStore::desktop();
    let provider = resolve_selector_with_store(args.provider.as_deref(), &store)?;
    let llm = client_for_with_store(&provider, args.model.as_deref(), &store)?;
    if matches!(args.runtime, Runtime::Python) {
        use hickory_cli::python_agent::{Config, TerminalReview};
        let session = hickory_cli::python_agent::run(
            Config {
                python: args.python.context("Python runtime needs --python")?,
                root: project_dir,
                doc: args.doc,
                prompt: args.prompt,
                max_turns: args.max_turns,
            },
            &llm,
            &mut TerminalReview,
        )
        .await?;
        println!("{}", session.display());
        return Ok(ExitCode::SUCCESS);
    }
    // Executor selection follows HICKORY_EXECUTOR, same as run/test — and so
    // does the project it names its scratch directory after, which is the
    // agent's project rather than whatever directory the person typed from.
    let executor = ExecutorChoice::from_env()?
        .build_for(Some(&project_dir))
        .await?;

    let mut config = AgentConfig::new(args.prompt, &project_dir);
    // `--doc` is the session's primary document: it enables the edit tool
    // set (read_doc/read_output/edit_output/edit_doc/verify) instead of
    // inlining the source as context.
    config.doc_path = args.doc.clone();
    config.max_turns = args.max_turns;

    let mut on_event = |event: AgentEvent| {
        let mut stdout = std::io::stdout();
        match &event {
            AgentEvent::SessionStarted {
                model,
                session_path,
            } => eprintln!("agent: model {model}, session {session_path}"),
            AgentEvent::Token { data } => {
                let _ = write!(stdout, "{data}");
                let _ = stdout.flush();
            }
            // Reasoning goes to stderr, dimmed by position rather than
            // colour: it is the model thinking, not the answer, and a pipe
            // reading stdout must not receive it as one.
            AgentEvent::Reasoning { data } => {
                eprint!("{data}");
            }
            AgentEvent::ResponseComplete { .. } => {
                let _ = writeln!(stdout);
            }
            AgentEvent::ScriptStarted { lang, .. } => eprintln!("agent: running {lang} script"),
            AgentEvent::ScriptFinished { result } => {
                if !result.stdout.is_empty() {
                    let _ = write!(stdout, "{}", result.stdout);
                }
                if !result.stderr.is_empty() {
                    eprint!("{}", result.stderr);
                }
                eprintln!(
                    "agent: script exited with {}",
                    result
                        .exit_code
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "unknown".into())
                );
            }
            AgentEvent::ToolStarted { name, .. } => eprintln!("agent: running tool {name}"),
            AgentEvent::ToolFinished { name, ok, text } => {
                let _ = writeln!(stdout, "{text}");
                eprintln!(
                    "agent: tool {name} {}",
                    if *ok { "ok" } else { "refused/failed" }
                );
            }
            AgentEvent::Reprompt { reason, .. } => {
                eprintln!("agent: re-prompting after malformed response ({reason})");
            }
            AgentEvent::Error { message } => eprintln!("agent: error: {message}"),
            AgentEvent::TurnUsage {
                usage,
                cost_usd,
                total_cost_usd,
                ..
            } => {
                let cost =
                    |c: &Option<f64>| c.map(|v| format!("${v:.4}")).unwrap_or_else(|| "?".into());
                eprintln!(
                    "agent: turn usage in={} cache_write={} cache_read={} out={} ({}, session total {})",
                    usage.input_tokens,
                    usage.cache_creation_input_tokens,
                    usage.cache_read_input_tokens,
                    usage.output_tokens,
                    cost(cost_usd),
                    cost(total_cost_usd),
                );
            }
            AgentEvent::UserMessage { .. } | AgentEvent::Thinking | AgentEvent::Done { .. } => {}
        }
    };

    let outcome = run_agent(&llm, executor.clone(), &config, &mut on_event).await?;
    executor.shutdown().await?;

    eprintln!(
        "agent: finished in {} turn(s); session written to {}{}",
        outcome.turns,
        outcome.session_path.display(),
        outcome
            .total_cost_usd
            .map(|c| format!("; spend ${c:.4}"))
            .unwrap_or_default()
    );
    println!("{}", outcome.session_path.display());
    Ok(ExitCode::SUCCESS)
}
