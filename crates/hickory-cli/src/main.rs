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
    /// Rewrite stale `<hick:transform>` passages from their current inputs.
    /// The ONLY command that calls a model.
    Refresh(RefreshArgs),
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
struct RefreshArgs {
    /// A `.hick` document (or directory of them).
    path: PathBuf,
    /// Rewrite every transform, not only the stale ones.
    #[arg(long)]
    all: bool,
    /// Report what would be rewritten without calling a model.
    #[arg(long = "dry-run")]
    dry_run: bool,
    /// Model id override (interpreted by the selected provider).
    #[arg(long = "model")]
    model: Option<String>,
    /// LLM provider: anthropic (default), openai, deepseek, or grok.
    #[arg(long = "provider", default_value = "anthropic")]
    provider: String,
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
    /// Model id override (default: the provider's default model).
    #[arg(long = "model")]
    model: Option<String>,
    /// LLM provider: anthropic (default), openai, deepseek, or grok.
    #[arg(long = "provider", default_value = "anthropic")]
    provider: String,
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
            Command::Refresh(args) => cmd_refresh(args).await,
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
        let mut failures = check_failures(&run, args.out.as_deref())?;
        // Transform passages are checked from the SOURCE, not from a re-run:
        // no model is called, so this stays free and deterministic in CI.
        failures.extend(hickory_cli::stale_transforms(&run.doc_path, &run.source)?);
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
                CheckFailure::StaleTransform {
                    doc,
                    line,
                    select,
                    instruct,
                } => {
                    eprintln!(
                        "STALE {}:{line}: the passage written from '{select}' no longer \
                         matches its input\n  instruction: {instruct}\n  \
                         fix: hickory refresh {}",
                        doc.display(),
                        doc.display(),
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

    use hickory_agent::{AgentConfig, AgentEvent, client_for, run_agent};

    let project_dir = match &args.dir {
        Some(dir) => dir.clone(),
        None => std::env::current_dir()?,
    };
    // `client_for` reports an unknown provider or a missing key by name,
    // before anything is executed.
    let llm = client_for(&args.provider, args.model.as_deref(), None)?;
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

/// `hickory refresh` — rewrite stale transform passages from their inputs.
///
/// This is the only command in the tool that calls a model, and that is a
/// deliberate boundary: `run`, `check`, and `weave` stay free, offline, and
/// deterministic, so a document containing LLM-written prose is still safe to
/// verify in CI.
///
/// The model is shown the previous passage along with the new input. That is
/// what keeps refreshes stable — and what makes a passage you edited by hand
/// survive: your wording is the starting point, not something to be
/// regenerated over.
async fn cmd_refresh(args: RefreshArgs) -> Result<ExitCode> {
    use hickory_agent::{LlmClient, Message, Role, client_for};

    let docs = expand_docs(&args.path)?;
    let mut rewrote = 0usize;
    let mut stale_total = 0usize;

    for doc_path in &docs {
        let source = std::fs::read_to_string(doc_path)?;
        let parsed = hick_lang::parse(&source)
            .map_err(|e| anyhow::anyhow!("parse error in {}: {e}", doc_path.display()))?;

        // Collect the work first: each transform's input, instruction, current
        // passage, and the byte span of its body.
        let mut jobs: Vec<(usize, usize, String, String, String, String)> = Vec::new();
        for tag in parsed.find_tags("transform") {
            let select = tag.get_attribute("select").unwrap_or_default().to_string();
            let instruct = tag
                .get_attribute("instruct")
                .unwrap_or_default()
                .to_string();
            let recorded = tag.get_attribute("from").unwrap_or_default().to_string();
            let input = hickory_cli::transform_input(&parsed, &select);
            let fingerprint = hick_lang::transform_fingerprint(&input, &instruct);
            if fingerprint == recorded && !args.all {
                continue;
            }
            stale_total += 1;
            let Some((body_from, body_to)) = body_span(&source, tag) else {
                eprintln!(
                    "skip {}:{}: cannot locate the passage body",
                    doc_path.display(),
                    tag.source_line
                );
                continue;
            };
            let previous = source[body_from..body_to].to_string();
            jobs.push((body_from, body_to, input, instruct, previous, fingerprint));
        }

        if jobs.is_empty() {
            continue;
        }
        if args.dry_run {
            for (_, _, _, instruct, _, _) in &jobs {
                println!("would refresh {}: {instruct}", doc_path.display());
            }
            continue;
        }
        let llm = client_for(&args.provider, args.model.as_deref(), None)?;

        // Apply back-to-front so earlier spans stay valid.
        jobs.sort_by_key(|j| std::cmp::Reverse(j.0));
        let mut updated = source.clone();
        for (from, to, input, instruct, previous, fingerprint) in jobs {
            let prompt = format!(
                "Rewrite the passage below so it is accurate for the current input.\n\n\
                 Instruction: {instruct}\n\n\
                 Current input:\n{input}\n\n\
                 Previous passage (keep its voice, structure, and any wording that is \
                 still correct — change only what the new input requires):\n{previous}\n\n\
                 Reply with the passage only. No preamble, no code fences."
            );
            let passage = llm
                .complete(vec![
                    Message::new(
                        Role::System,
                        "You rewrite short documentation passages. You preserve the author's \
                         voice and change as little as possible.",
                    ),
                    Message::new(Role::User, prompt),
                ])
                .await?;
            let passage = format!("\n{}\n", passage.trim());
            updated.replace_range(from..to, &passage);
            // Re-stamp the fingerprint this passage now attests to.
            updated = restamp_from(&updated, from, &fingerprint);
            rewrote += 1;
        }
        std::fs::write(doc_path, &updated)?;
        println!("refreshed {}", doc_path.display());
    }

    if args.dry_run {
        println!("{stale_total} transform(s) stale");
    } else {
        println!("{rewrote} passage(s) rewritten");
    }
    Ok(ExitCode::SUCCESS)
}

/// Byte span of a tag's body: between the `>` of the opening tag and the `<`
/// of its closing tag.
fn body_span(source: &str, tag: &hick_lang::HickTag) -> Option<(usize, usize)> {
    let open = tag.source_span?;
    let start = open.end;
    let close = source[start..].find("</")? + start;
    Some((start, close))
}

/// Rewrite the `from="..."` attribute of the transform whose body starts at
/// `body_start`, inserting one if the document does not carry it yet.
fn restamp_from(source: &str, body_start: usize, fingerprint: &str) -> String {
    let head = &source[..body_start];
    let Some(open) = head.rfind("<hick:transform") else {
        return source.to_string();
    };
    let tag_text = &source[open..body_start];
    let replaced = match tag_text.find("from=\"") {
        Some(i) => {
            let value_start = open + i + "from=\"".len();
            let value_end = value_start + source[value_start..].find('"').unwrap_or(0);
            let mut out = source.to_string();
            out.replace_range(value_start..value_end, fingerprint);
            return out;
        }
        None => tag_text.replacen(
            "<hick:transform",
            &format!("<hick:transform from=\"{fingerprint}\""),
            1,
        ),
    };
    let mut out = source.to_string();
    out.replace_range(open..body_start, &replaced);
    out
}
