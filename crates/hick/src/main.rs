//! Hick: Secure container orchestration shell.
//!
//! Usage: `hick <file.hick> [file2.hick ...]`
//!
//! Parses `.hick` files, builds an information-flow DAG, validates it,
//! and executes containers with capability-based security.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info, warn};
use rand::RngCore;

use hick_api::{AutoConfirm, GatewayFilter};
use hick_exec::node::{InsertionPoint, StringNode};
use hick_exec::state::MultiDocumentState;
use hick_lang::{HickNode, HickTag};
use hick_token::{ContainerCapabilities, NetworkRule, TokenAuthority};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "hick", about = "Secure container orchestration shell")]
struct Cli {
    /// One or more `.hick` files to process.
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Age identity file for decrypting secrets.
    #[arg(long, default_value = "~/.config/hick/key.txt")]
    key_file: PathBuf,

    /// Directory containing age-encrypted secret files.
    #[arg(long, default_value = "~/.config/hick/secrets")]
    secrets_dir: PathBuf,

    /// Enable verbose output.
    #[arg(short, long)]
    verbose: bool,
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::from_default_env()
        .filter_level(if cli.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();

    // Generate a random root key for this session's token authority
    let mut root_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut root_key);
    let authority = Arc::new(TokenAuthority::new(&root_key));

    // Set up gateway filter for internal APIs
    let gateway_filter = Arc::new(GatewayFilter::new(authority.clone(), Arc::new(AutoConfirm)));

    // Set up secrets provider
    let key_path = expand_tilde(&cli.key_file);
    let secrets_dir = expand_tilde(&cli.secrets_dir);
    let _secrets_provider = hick_secrets::AgeSecretsProvider::new(&key_path, &secrets_dir);

    // Set up multi-document state
    let state = Arc::new(MultiDocumentState::default());

    // Parse all input files
    let mut documents = Vec::new();
    for path in &cli.files {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let doc = hick_lang::parse(&source)
            .map_err(|e| anyhow::anyhow!("parse error in {}: {e}", path.display()))?;
        info!(
            "Parsed {}: {} top-level nodes",
            path.display(),
            doc.nodes.len()
        );
        documents.push((path.clone(), doc));
    }

    // Build and validate DAG for each document
    for (path, doc) in &documents {
        let dag = hick_exec::dag::build_dag(doc)
            .map_err(|e| anyhow::anyhow!("DAG validation failed in {}: {e}", path.display()))?;
        info!(
            "DAG validated for {}: {} exec nodes",
            path.display(),
            dag.execs.len()
        );
    }

    // Collect all nodes across documents for processing
    let all_nodes: Vec<&HickNode> = documents
        .iter()
        .flat_map(|(_, doc)| doc.nodes.iter())
        .collect();

    // Collect container definitions
    let mut container_defs: HashMap<String, ContainerCapabilities> = HashMap::new();
    for node in &all_nodes {
        if let HickNode::Tag(tag) = node
            && tag.name == "container"
        {
            let name = tag_attr(tag, "name").unwrap_or_default();
            let caps = build_capabilities_from_tag(tag);
            debug!("Container '{name}': {:?}", caps);
            container_defs.insert(name, caps);
        }
    }

    // Mint tokens for each container
    for (name, caps) in &container_defs {
        let token = authority
            .mint(name, caps)
            .map_err(|e| anyhow::anyhow!("failed to mint token for '{name}': {e}"))?;
        gateway_filter.register_token(name.clone(), token).await;
        info!("Minted token for container '{name}'");
    }

    // Process copy/cut blocks first (they may be referenced by paste)
    for node in &all_nodes {
        if let HickNode::Tag(tag) = node {
            match tag.name.as_str() {
                "copy" => {
                    let id = tag_attr(tag, "id").unwrap_or_default();
                    let content = collect_text_children(tag);
                    state.register_copy(id, content);
                }
                "cut" => {
                    let id = tag_attr(tag, "id").unwrap_or_default();
                    let content = collect_text_children(tag);
                    state.register_cut(id, content);
                }
                _ => {}
            }
        }
    }

    // Process file outputs
    for node in &all_nodes {
        if let HickNode::Tag(tag) = node
            && tag.name == "file"
        {
            let path = tag_attr(tag, "path").unwrap_or_default();
            let file_ip = Arc::new(InsertionPoint::new());

            // Add child content to the file insertion point
            for child in &tag.children {
                match child {
                    HickNode::Text(text) => {
                        file_ip.add(Arc::new(StringNode::new(text.clone())));
                    }
                    HickNode::Tag(child_tag) if child_tag.name == "exec" => {
                        let container_name = tag_attr(child_tag, "container").unwrap_or_default();
                        // Placeholder: exec output would come from container runtime
                        let exec_text = format!("[exec in container '{container_name}']");
                        file_ip.add(Arc::new(StringNode::new(exec_text)));
                    }
                    HickNode::Tag(child_tag) if child_tag.name == "paste" => {
                        let selector = tag_attr(child_tag, "select").unwrap_or_default();
                        if let Some(content) = state.resolve_paste(&selector) {
                            file_ip.add(Arc::new(StringNode::new(content)));
                        } else {
                            warn!("Paste selector '{selector}' not found");
                        }
                    }
                    _ => {}
                }
            }

            file_ip.close();
            state.add_file_output(path, file_ip);
        }
    }

    // Converge file outputs
    let files = state.get_files().await;
    for (path, content) in &files {
        info!("Writing output: {path}");
        if let Some(parent) = PathBuf::from(path).parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        debug!("  {} bytes written", content.len());
    }

    if files.is_empty() {
        info!("No file outputs produced");
    }

    info!("Done");
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Get an attribute value from a tag by name.
fn tag_attr(tag: &HickTag, name: &str) -> Option<String> {
    tag.attributes
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.clone())
}

/// Build `ContainerCapabilities` from a `<hick:container>` tag's children.
fn build_capabilities_from_tag(tag: &HickTag) -> ContainerCapabilities {
    let mut caps = ContainerCapabilities::new();

    for child in &tag.children {
        if let HickNode::Tag(child_tag) = child {
            match child_tag.name.as_str() {
                "allow" => {
                    if let Some(network) = tag_attr(child_tag, "network")
                        && let Some((host, port)) = network.rsplit_once(':')
                    {
                        caps.network_rules.push(NetworkRule::allow(host, port));
                    }
                    if let Some(path) = tag_attr(child_tag, "file-read") {
                        caps.file_rules.push(hick_token::FileRule::Read(path));
                    }
                    if let Some(path) = tag_attr(child_tag, "file-write") {
                        caps.file_rules.push(hick_token::FileRule::Write(path));
                    }
                }
                "deny" => {
                    if tag_attr(child_tag, "network").is_some() {
                        caps.network_rules.push(NetworkRule::deny_all());
                    }
                }
                "secret" => {
                    let env_var = tag_attr(child_tag, "name").unwrap_or_default();
                    let secret_name = tag_attr(child_tag, "from").unwrap_or_default();
                    caps.secret_rules.push(hick_token::SecretRule {
                        env_var,
                        secret_name,
                    });
                }
                _ => {}
            }
        }
    }

    caps
}

/// Collect text content from a tag's children into a single string.
fn collect_text_children(tag: &HickTag) -> String {
    let mut result = String::new();
    for child in &tag.children {
        if let HickNode::Text(text) = child {
            result.push_str(text);
        }
    }
    result
}

/// Expand `~` to the user's home directory.
fn expand_tilde(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if s.starts_with("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home).join(&s[2..]);
    }
    path.to_path_buf()
}
