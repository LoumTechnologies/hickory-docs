<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/python_agent_main.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-23dd3575a49b"></a>
## `crates/hickory-cli/src/python_agent_main.rs`

<a id="f-23dd3575a49b-Args"></a>
### `Args`

Standalone front door to the supervised Python agent; no UI required.

<storycode:copy id="f-23dd3575a49b-Args">
//! Standalone front door to the supervised Python agent; no UI required.
use anyhow::Result;
use clap::Parser;
use hickory_agent::{KeyStore, client_for_with_store, resolve_selector_with_store};
use hickory_cli::python_agent::{Config, TerminalReview, run};
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Run the confined Python Hickory agent")]
struct Args {
    prompt: String,
    /// Absolute path to Python in the locked, non-editable agent venv.
    #[arg(long)]
    python: PathBuf,
    #[arg(long, default_value = ".")]
    dir: PathBuf,
    #[arg(long)]
    doc: Option<PathBuf>,
    #[arg(long)]
    provider: Option<String>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long, default_value_t = 20)]
    max_turns: usize,
}

</storycode:copy>
<a id="f-23dd3575a49b-main"></a>
### `main`

<storycode:copy id="f-23dd3575a49b-main">
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let store = KeyStore::desktop();
    let provider = resolve_selector_with_store(args.provider.as_deref(), &store)?;
    let client = client_for_with_store(&provider, args.model.as_deref(), &store)?;
    let session = run(
        Config {
            python: args.python,
            root: args.dir,
            doc: args.doc,
            prompt: args.prompt,
            max_turns: args.max_turns,
        },
        &client,
        &mut TerminalReview,
    )
    .await?;
    println!("{}", session.display());
    Ok(())
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/python_agent_main.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/python_agent_main.rs"><storycode:paste select="#f-23dd3575a49b-Args"/><storycode:paste select="#f-23dd3575a49b-main"/></storycode:file>

</storycode:doc>
