<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/agent_command.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9a29232884e6"></a>
## `crates/hickory-cli/src/agent_command.rs`

<a id="f-9a29232884e6-AgentArgs"></a>
### `AgentArgs`

Agent CLI selection, kept apart from document commands.

<storycode:copy id="f-9a29232884e6-AgentArgs">
//! Agent CLI selection, kept apart from document commands.
use anyhow::Result;
use std::path::PathBuf;
use std::process::ExitCode;
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
    /// Absolute path to the installed Python agent runtime.
    #[arg(long)]
    python: PathBuf,
    /// Maximum LLM turns before giving up.
    #[arg(long = "max-turns", default_value_t = 20)]
    max_turns: usize,
}

</storycode:copy>
<a id="f-9a29232884e6-run"></a>
### `run`

<storycode:copy id="f-9a29232884e6-run">
pub(crate) async fn run(args: AgentArgs) -> Result<ExitCode> {
    use hickory_agent::{KeyStore, client_for_with_store, resolve_selector_with_store};
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
    use hickory_cli::python_agent::{Config, TerminalReview};
    let session = hickory_cli::python_agent::run(
        Config {
            python: args.python,
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
    Ok(ExitCode::SUCCESS)
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/agent_command.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/agent_command.rs"><storycode:paste select="#f-9a29232884e6-AgentArgs"/><storycode:paste select="#f-9a29232884e6-run"/></storycode:file>

</storycode:doc>
