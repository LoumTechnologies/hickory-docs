//! hick-plan-runner: Sidecar binary for executing hick XML plans.
//!
//! Parses a plan XML, sets up the tag registry with hick-live handlers,
//! builds a reactive Node DAG, and executes it — collecting results and
//! API call records.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use hick_agent_sdk::{XmlDiagnosticLogger, NullSessionLog};
use clap::Parser;
use futures::StreamExt;
use hick_flow::node::Node;
use hick_handlers::{ProcessingContext, TagRegistry, TagResult};
use hick_lang::HickNode;
use hick_sink::{MockSinkBackend, SinkBackend};
use hick_token::ContainerCapabilities;

/// Execute hick XML plans with mock or live API backends.
#[derive(Parser)]
#[command(name = "hick-plan-runner")]
struct Cli {
    /// Plan XML file (or - for stdin).
    #[arg(long)]
    plan: String,

    /// Data file(s) directory.
    #[arg(long)]
    data: Option<PathBuf>,

    /// Token caveats JSON file.
    #[arg(long)]
    token: Option<PathBuf>,

    /// API execution mode.
    #[arg(long, default_value = "mock")]
    mode: String,

    /// Python policy script.
    #[arg(long)]
    policy: Option<PathBuf>,

    /// Policy function name.
    #[arg(long, default_value = "evaluate")]
    policy_fn: String,

    /// Output file for API call records.
    #[arg(long, default_value = "api_calls.json")]
    api_log: PathBuf,

    /// Output file for result data (default: stdout as CSV).
    #[arg(long)]
    result: Option<PathBuf>,

    /// Log file for missing operator requests.
    #[arg(long)]
    missing_ops: Option<PathBuf>,

    /// Sink definitions JSON file.
    #[arg(long)]
    sink_defs: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    log::set_boxed_logger(Box::new(XmlDiagnosticLogger::new(Arc::new(NullSessionLog)))).ok();
    log::set_max_level(log::LevelFilter::Info);
    let cli = Cli::parse();

    // 1. Read plan XML
    let xml = if cli.plan == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        std::fs::read_to_string(&cli.plan)
            .with_context(|| format!("failed to read plan file: {}", cli.plan))?
    };

    // 2. Parse plan
    let doc = hick_lang::parse(&xml).map_err(|e| anyhow::anyhow!("parse error: {e}"))?;

    // 3. Load token caveats
    let capabilities = if let Some(token_path) = &cli.token {
        let token_json = std::fs::read_to_string(token_path)
            .with_context(|| format!("failed to read token file: {}", token_path.display()))?;
        let caveats: Vec<String> = serde_json::from_str(&token_json)?;
        ContainerCapabilities::from_caveats(&caveats)
    } else {
        // Default: permissive capabilities for testing
        ContainerCapabilities::new().with_expires("2099-01-01T00:00:00Z")
    };

    // 4. Set up tag registry
    let mut registry = TagRegistry::new();
    hick_live::handlers::register_live_builtins(&mut registry);

    // 5. Set up sink backend
    let mock = Arc::new(MockSinkBackend::new());
    let backend: Arc<dyn SinkBackend> = mock.clone();

    // 6. Build processing context
    let state = Arc::new(hick_exec::state::MultiDocumentState::default());
    let transcripts = HashMap::new();
    let mut flow_context = hick_flow::Context::default();
    flow_context.set_extension::<Arc<dyn SinkBackend>>(backend);
    flow_context.set_extension::<ContainerCapabilities>(capabilities);

    let proc_ctx = ProcessingContext {
        state: &state,
        transcripts: &transcripts,
        indent: 0,
        registry: Some(&registry),
        context: Some(&flow_context),
        source_file: None,
    };

    // 7. Process tags → build Node DAG
    let mut missing_ops = Vec::new();
    let mut root_node: Option<Arc<dyn Node>> = None;

    for node in &doc.nodes {
        if let HickNode::Tag(tag) = node {
            if let Some(handler) = registry.find(&tag.name) {
                match handler.process(tag, &proc_ctx) {
                    Ok(TagResult::Node(n)) => {
                        root_node = Some(n);
                    }
                    Ok(_) => {
                        log::debug!("Tag '{}' produced non-node result", tag.name);
                    }
                    Err(e) => {
                        log::warn!("Handler error for '{}': {}", tag.name, e);
                    }
                }
            } else {
                log::info!("Unknown operator: {}", tag.name);
                missing_ops.push(serde_json::json!({
                    "tag_name": tag.name,
                    "attributes": tag.attributes.iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>(),
                }));
            }
        }
    }

    // 8. Write missing operator requests
    if let Some(missing_path) = &cli.missing_ops {
        if !missing_ops.is_empty() {
            let json = serde_json::to_string_pretty(&missing_ops)?;
            std::fs::write(missing_path, json)?;
        }
    }

    // 9. Execute the DAG
    if let Some(node) = root_node {
        let stream = node.get_stream(flow_context.clone());
        let events: Vec<_> = stream.collect().await;

        // Write result summary
        if let Some(result_path) = &cli.result {
            let summary = serde_json::json!({
                "event_count": events.len(),
                "status": "ok",
            });
            std::fs::write(result_path, serde_json::to_string_pretty(&summary)?)?;
        } else {
            println!("Executed plan: {} event batches", events.len());
        }
    } else {
        bail!("No root node produced from plan XML");
    }

    // 10. Write API call records
    let calls = mock.calls();
    let calls_json = serde_json::to_string_pretty(&calls)?;
    std::fs::write(&cli.api_log, calls_json)?;

    if !calls.is_empty() {
        log::info!("Recorded {} API calls", calls.len());
    }

    Ok(())
}
