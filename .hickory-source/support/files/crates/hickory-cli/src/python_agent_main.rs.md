<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/python_agent_main.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `aada6cb1f47cd51e26c58a4e7caea222035cdf4a18efcc7c44b19d601891b311`.

<book:entry path="crates/hickory-cli/src/python_agent_main.rs">
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
</book:entry>

</book:doc>
