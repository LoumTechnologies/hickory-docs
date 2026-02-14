//! hick-mcp: MCP server binary for AI coding agent integration.
//!
//! Exposes hick lens and exec operations as MCP tools. Run with `--stdio`
//! for integration with Claude Code or other MCP-aware clients.

use rmcp::{ServiceExt, transport::stdio};

use hick_mcp::server::HickMcpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Log to stderr (stdout reserved for MCP protocol)
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting hick-mcp server");

    let server = HickMcpServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
