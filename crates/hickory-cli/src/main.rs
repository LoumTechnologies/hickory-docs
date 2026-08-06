//! `hickory` — run, verify, weave, and promote executable documents.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context as _, Result};
use clap::{Parser, Subcommand};

use hickory_cli::{
    CheckFailure, DocRun, ExecutorChoice, RunMode, block_model_json, check_failures, expand_docs,
    run_doc, write_outputs,
};

#[derive(Parser)]
#[command(
    name = "hickory",
    about = "Reproducible, verifiable, executable documents"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Execute a document (or every document in a directory) and write its
    /// outputs, including woven markdown.
    Run(RunArgs),
    /// Verification mode: re-execute and fail on any unmet <hick:expect>
    /// expectation or drift between produced outputs and committed files.
    Check(CheckArgs),
    /// Weave without executing: cached transcripts where present, otherwise
    /// blocks are marked never-run.
    Weave(WeaveArgs),
    /// Print the byte-precise lineage (Provenance[]) of a generated output
    /// file: which source spans produced each byte range. Weaves without
    /// executing.
    Lineage(LineageArgs),
    /// Promote a session document into a clean pipeline document.
    Promote(PromoteArgs),
    /// Run the AI agent on a prompt; the session is written as a
    /// hick:session document under the project's sessions/ directory.
    Agent(AgentArgs),
    /// Set up a local git repository for hickory: pre-commit drift gate,
    /// .gitignore entry, and an AGENTS.md section for coding agents.
    /// Idempotent — re-run any time to refresh the managed blocks.
    Init(InitArgs),
}

#[derive(clap::Args)]
struct RunArgs {
    /// A `.hick` document or a directory of documents.
    path: PathBuf,
    /// Parameter overrides, `key=value` (repeatable).
    #[arg(long = "param", value_parser = hick_literate::parse_param)]
    params: Vec<(String, String)>,
    /// Enable `<hick:feature>` flags (comma-separated, repeatable).
    /// Shorthand for `--param features=a,b`; feeds `<hick:when test="...">`.
    #[arg(long = "features", value_delimiter = ',')]
    features: Vec<String>,
    /// Output directory (default: each document's own directory).
    #[arg(long = "out")]
    out: Option<PathBuf>,
    /// Emit the block model as JSON on stdout instead of a summary.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct CheckArgs {
    /// A `.hick` document or a directory of documents.
    path: PathBuf,
    /// Parameter overrides, `key=value` (repeatable).
    #[arg(long = "param", value_parser = hick_literate::parse_param)]
    params: Vec<(String, String)>,
    /// Enable `<hick:feature>` flags (comma-separated, repeatable).
    /// Shorthand for `--param features=a,b`; feeds `<hick:when test="...">`.
    #[arg(long = "features", value_delimiter = ',')]
    features: Vec<String>,
    /// Directory holding the committed outputs (default: each document's own
    /// directory).
    #[arg(long = "out")]
    out: Option<PathBuf>,
    /// Emit the block model as JSON on stdout in addition to failures.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct WeaveArgs {
    /// A `.hick` document.
    path: PathBuf,
    /// Parameter overrides, `key=value` (repeatable).
    #[arg(long = "param", value_parser = hick_literate::parse_param)]
    params: Vec<(String, String)>,
    /// Enable `<hick:feature>` flags (comma-separated, repeatable).
    /// Shorthand for `--param features=a,b`; feeds `<hick:when test="...">`.
    #[arg(long = "features", value_delimiter = ',')]
    features: Vec<String>,
    /// Output directory (default: the document's directory).
    #[arg(long = "out")]
    out: Option<PathBuf>,
    /// Emit the block model as JSON on stdout instead of a summary.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct LineageArgs {
    /// A `.hick` document.
    doc: PathBuf,
    /// The generated output file to trace (its `<hick:file path>` value).
    #[arg(long = "output")]
    output: String,
    /// Parameter overrides, `key=value` (repeatable).
    #[arg(long = "param", value_parser = hick_literate::parse_param)]
    params: Vec<(String, String)>,
    /// Emit the Provenance[] JSON on stdout instead of a summary.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct PromoteArgs {
    /// A `hick:session` document.
    session: PathBuf,
    /// Where to write the promoted pipeline (default: stdout).
    #[arg(long = "out")]
    out: Option<PathBuf>,
}

#[derive(clap::Args)]
struct InitArgs {
    /// Directory inside the git repository to initialize (default: cwd).
    #[arg(default_value = ".")]
    dir: PathBuf,
}

#[derive(clap::Args)]
struct AgentArgs {
    /// The task prompt for the agent.
    prompt: String,
    /// A `.hick` document to give the agent as context.
    #[arg(long = "doc")]
    doc: Option<PathBuf>,
    /// Project directory (sessions land in `<dir>/sessions/`; default: cwd).
    #[arg(long = "dir")]
    dir: Option<PathBuf>,
    /// Anthropic model id override (default: the current Sonnet-class alias).
    #[arg(long = "model")]
    model: Option<String>,
    /// Maximum LLM turns before giving up.
    #[arg(long = "max-turns", default_value_t = 20)]
    max_turns: usize,
}

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let cli = Cli::parse();

    let runtime = tokio::runtime::Runtime::new().expect("failed to build tokio runtime");
    let outcome = runtime.block_on(async {
        match cli.command {
            Command::Run(args) => cmd_run(args).await,
            Command::Check(args) => cmd_check(args).await,
            Command::Weave(args) => cmd_weave(args).await,
            Command::Lineage(args) => cmd_lineage(args).await,
            Command::Promote(args) => cmd_promote(args),
            Command::Agent(args) => cmd_agent(args).await,
            Command::Init(args) => cmd_init(args),
        }
    });

    match outcome {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn print_run_summary(run: &DocRun, written: &[PathBuf]) {
    let n_expect = run.result.expectations.len();
    let n_failed = run.result.expectations.iter().filter(|o| !o.passed).count();
    eprintln!(
        "{}: {} file(s) written, {} expectation(s) ({} failed)",
        run.doc_path.display(),
        written.len(),
        n_expect,
        n_failed
    );
    for path in written {
        eprintln!("  wrote {}", path.display());
    }
    for outcome in &run.result.expectations {
        if !outcome.passed {
            eprintln!(
                "  expectation FAILED ({} line {}): {}",
                outcome.container, outcome.line, outcome.detail
            );
        }
    }
}

/// Fold `--features a,b` into the param list as the `features` variable —
/// the same variable `hick-literate`'s feature system reads (enabled
/// features then become condition variables for `<hick:when test="...">`).
fn params_with_features(params: &[(String, String)], features: &[String]) -> Vec<(String, String)> {
    let mut params = params.to_vec();
    if !features.is_empty() {
        params.push(("features".to_string(), features.join(",")));
    }
    params
}

async fn cmd_run(args: RunArgs) -> Result<ExitCode> {
    let executor_choice = ExecutorChoice::from_env()?;
    let docs = expand_docs(&args.path)?;
    let params = params_with_features(&args.params, &args.features);
    let mut json_blocks = Vec::new();
    for doc_path in &docs {
        let run = run_doc(doc_path, &params, RunMode::Execute, executor_choice).await?;
        let written = write_outputs(&run, args.out.as_deref())?;
        if args.json {
            json_blocks.push(block_model_json(&run)?);
        } else {
            print_run_summary(&run, &written);
        }
    }
    if args.json {
        emit_json(json_blocks)?;
    }
    Ok(ExitCode::SUCCESS)
}

async fn cmd_check(args: CheckArgs) -> Result<ExitCode> {
    let executor_choice = ExecutorChoice::from_env()?;
    let docs = expand_docs(&args.path)?;
    let params = params_with_features(&args.params, &args.features);
    let mut any_failed = false;
    let mut json_blocks = Vec::new();
    for doc_path in &docs {
        let run = run_doc(doc_path, &params, RunMode::Execute, executor_choice).await?;
        let failures = check_failures(&run, args.out.as_deref())?;
        if args.json {
            json_blocks.push(block_model_json(&run)?);
        }
        if failures.is_empty() {
            eprintln!("ok: {}", doc_path.display());
            continue;
        }
        any_failed = true;
        for failure in &failures {
            match failure {
                CheckFailure::Expectation(o) => {
                    let span = o
                        .span
                        .map(|(s, e)| format!(" (bytes {s}..{e})"))
                        .unwrap_or_default();
                    eprintln!(
                        "FAIL {}:{}{span} [container '{}', match {}]\n  {}\n  expected:\n{}\n  actual:\n{}",
                        o.doc,
                        o.line,
                        o.container,
                        o.mode,
                        o.detail,
                        indent(&o.expected),
                        indent(&o.actual),
                    );
                }
                CheckFailure::Drift {
                    doc,
                    output_path,
                    detail,
                } => {
                    eprintln!(
                        "FAIL {} -> {}: {detail}",
                        doc.display(),
                        output_path.display()
                    );
                }
            }
        }
    }
    if args.json {
        emit_json(json_blocks)?;
    }
    Ok(if any_failed {
        eprintln!("hickory check: documentation drift detected");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

async fn cmd_weave(args: WeaveArgs) -> Result<ExitCode> {
    let docs = expand_docs(&args.path)?;
    let params = params_with_features(&args.params, &args.features);
    let mut json_blocks = Vec::new();
    for doc_path in &docs {
        let run = run_doc(doc_path, &params, RunMode::Weave, ExecutorChoice::Local).await?;
        let written = write_outputs(&run, args.out.as_deref())?;
        if args.json {
            json_blocks.push(block_model_json(&run)?);
        } else {
            print_run_summary(&run, &written);
            if !run.result.never_run.is_empty() {
                eprintln!(
                    "  {} block(s) never run (no cached transcript)",
                    run.result.never_run.len()
                );
            }
        }
    }
    if args.json {
        emit_json(json_blocks)?;
    }
    Ok(ExitCode::SUCCESS)
}

/// `hickory lineage <doc> --output <path> [--json]` — the same Provenance[]
/// the server serves from GET /api/docs/:id/outputs/file, computed locally
/// from a weave (no execution).
async fn cmd_lineage(args: LineageArgs) -> Result<ExitCode> {
    let run = run_doc(
        &args.doc,
        &args.params,
        RunMode::Weave,
        ExecutorChoice::Local,
    )
    .await?;
    let provenance = hickory_cli::output_lineage(&run, &args.output)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&provenance)?);
    } else {
        eprintln!(
            "{} -> {}: {} provenance span(s)",
            args.doc.display(),
            args.output,
            provenance.len()
        );
        for p in &provenance {
            match &p.origin {
                hickory_lineage::Origin::Synthetic => {
                    println!("{:>8}..{:<8} synthetic", p.start, p.end);
                }
                origin => {
                    let (doc_path, s, e) = origin.location().unwrap_or(("?", 0, 0));
                    let kind = match origin {
                        hickory_lineage::Origin::Literal { .. } => "literal",
                        hickory_lineage::Origin::Paste { .. } => "paste",
                        hickory_lineage::Origin::Exec { .. } => "exec",
                        hickory_lineage::Origin::Variable { .. } => "variable",
                        hickory_lineage::Origin::Substitution { .. } => "substitution",
                        hickory_lineage::Origin::Synthetic => unreachable!(),
                    };
                    println!(
                        "{:>8}..{:<8} {kind:<12} {doc_path} bytes {s}..{e}",
                        p.start, p.end
                    );
                }
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_promote(args: PromoteArgs) -> Result<ExitCode> {
    use hick_literate::promote::{PromoteOpts, promote};
    let source = std::fs::read_to_string(&args.session)
        .with_context(|| format!("failed to read {}", args.session.display()))?;
    let project_dir = args
        .session
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let session_name = args
        .session
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| args.session.display().to_string());
    let result = promote(&PromoteOpts {
        session_source: &source,
        project_dir: &project_dir,
        session_name: &session_name,
    })?;
    match &args.out {
        Some(path) => {
            std::fs::write(path, &result.promoted_source)?;
            eprintln!(
                "promoted {} -> {} ({} of {} writes surviving)",
                args.session.display(),
                path.display(),
                result.surviving_writes,
                result.total_writes
            );
        }
        None => print!("{}", result.promoted_source),
    }
    Ok(ExitCode::SUCCESS)
}

async fn cmd_agent(args: AgentArgs) -> Result<ExitCode> {
    use std::io::Write as _;

    use hickory_agent::{AgentConfig, AgentEvent, AnthropicClient, run_agent};

    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        anyhow::bail!("ANTHROPIC_API_KEY is not set (required by `hickory agent`)");
    }

    let project_dir = match &args.dir {
        Some(dir) => dir.clone(),
        None => std::env::current_dir()?,
    };
    let mut llm = AnthropicClient::new();
    if let Some(model) = &args.model {
        llm = llm.with_model(model.clone());
    }
    // Executor selection follows HICKORY_EXECUTOR, same as run/check.
    let executor = ExecutorChoice::from_env()?.build()?;

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

fn cmd_init(args: InitArgs) -> Result<ExitCode> {
    let report = hickory_cli::run_init(&args.dir)?;
    hickory_cli::print_init_report(&report);
    Ok(ExitCode::SUCCESS)
}

fn emit_json(mut docs: Vec<serde_json::Value>) -> Result<()> {
    let value = if docs.len() == 1 {
        docs.pop().unwrap()
    } else {
        serde_json::Value::Array(docs)
    };
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn indent(s: &str) -> String {
    s.lines()
        .map(|l| format!("    | {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}
