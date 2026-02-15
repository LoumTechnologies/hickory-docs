//! hick-mcp: MCP server binary for AI coding agent integration.
//!
//! Exposes hick lens and exec operations as MCP tools. Run with `--stdio`
//! for integration with Claude Code or other MCP-aware clients.
//!
//! Pass `--images-dir` to enable sandboxed container execution via `hick_exec`.

use std::path::PathBuf;

use clap::Parser;
use rmcp::{ServiceExt, transport::stdio};

use hick_mcp::server::HickMcpServer;
use hick_mcp::state::{ExecConfig, ServerState};

#[derive(Parser)]
#[command(
    name = "hick-mcp",
    about = "MCP server for hick lens, exec, and container operations"
)]
struct Args {
    /// Directory containing pre-converted .wasm container images.
    /// Required to enable the hick_exec tool.
    #[arg(long)]
    images_dir: Option<PathBuf>,

    /// Working directory to mount in containers (default: current directory).
    #[arg(long)]
    workdir: Option<PathBuf>,

    /// Default container image name.
    #[arg(long, default_value = "alpine")]
    default_image: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Log to stderr (stdout reserved for MCP protocol)
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let args = Args::parse();

    let state = if let Some(images_dir) = args.images_dir {
        let workdir = args
            .workdir
            .unwrap_or_else(|| std::env::current_dir().expect("cannot determine current dir"));
        tracing::info!(
            "Container execution enabled: images={}, workdir={}, default_image={}",
            images_dir.display(),
            workdir.display(),
            args.default_image,
        );
        ServerState::with_exec_config(ExecConfig {
            images_dir,
            workdir,
            default_image: args.default_image,
        })
    } else {
        tracing::info!(
            "Starting hick-mcp server (container execution disabled, pass --images-dir to enable)"
        );
        ServerState::new()
    };

    let server = HickMcpServer::with_state(state);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
