//! Hick pipeline library.
//!
//! Provides the core pipeline for processing `.hick` sources: parsing,
//! DAG validation, container capability extraction, copy/paste resolution,
//! and file output collection — all without touching the filesystem.

pub mod executor;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use log::{debug, info, warn};
use rand::RngCore;

use hick_exec::dag;
use hick_exec::node::{InsertionPoint, StringNode};
use hick_exec::state::MultiDocumentState;
use hick_lang::{HickNode, HickTag};
use hick_token::{ContainerCapabilities, NetworkRule, TokenAuthority};

use crate::executor::ContainerExecutor;

// ---------------------------------------------------------------------------
// Pipeline result
// ---------------------------------------------------------------------------

/// The output of a hick pipeline run.
pub struct PipelineResult {
    /// File path -> rendered content for each `<hick:file>` tag.
    pub files: HashMap<String, String>,
    /// Container name -> parsed capabilities for each `<hick:container>` tag.
    pub containers: HashMap<String, ContainerCapabilities>,
}

// ---------------------------------------------------------------------------
// Pipeline entry point
// ---------------------------------------------------------------------------

/// Run the full hick pipeline on in-memory sources.
///
/// Each entry in `sources` is `(document_name, hick_source)`. The pipeline
/// parses, validates the DAG, collects containers, mints tokens, processes
/// copy/paste blocks, and converges file outputs — all without writing to
/// disk.
pub async fn run_pipeline(sources: &[(&str, &str)]) -> Result<PipelineResult> {
    let mut root_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut root_key);
    let authority = Arc::new(TokenAuthority::new(&root_key));

    run_pipeline_with_authority(sources, &authority).await
}

/// Run the pipeline with a caller-supplied [`TokenAuthority`].
///
/// Useful for tests that need to verify tokens against the same authority.
pub async fn run_pipeline_with_authority(
    sources: &[(&str, &str)],
    authority: &Arc<TokenAuthority>,
) -> Result<PipelineResult> {
    // Parse all sources
    let mut documents = Vec::new();
    for (name, source) in sources {
        let doc =
            hick_lang::parse(source).map_err(|e| anyhow::anyhow!("parse error in {name}: {e}"))?;
        info!("Parsed {name}: {} top-level nodes", doc.nodes.len());
        documents.push((*name, doc));
    }

    // Build and validate DAG for each document
    for (name, doc) in &documents {
        let dag = hick_exec::dag::build_dag(doc)
            .map_err(|e| anyhow::anyhow!("DAG validation failed in {name}: {e}"))?;
        info!("DAG validated for {name}: {} exec nodes", dag.execs.len());
    }

    // Collect all nodes across documents
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
        let _token = authority
            .mint(name, caps)
            .map_err(|e| anyhow::anyhow!("failed to mint token for '{name}': {e}"))?;
        info!("Minted token for container '{name}'");
    }

    // Set up multi-document state
    let state = Arc::new(MultiDocumentState::default());

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

            for child in &tag.children {
                match child {
                    HickNode::Text(text) => {
                        file_ip.add(Arc::new(StringNode::new(text.clone())));
                    }
                    HickNode::Tag(child_tag) if child_tag.name == "exec" => {
                        let container_name = tag_attr(child_tag, "container").unwrap_or_default();
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

    Ok(PipelineResult {
        files,
        containers: container_defs,
    })
}

// ---------------------------------------------------------------------------
// Live pipeline (real container execution)
// ---------------------------------------------------------------------------

/// Configuration for live pipeline execution.
pub struct PipelineConfig {
    /// Directory containing pre-converted `.wasm` images.
    pub images_dir: PathBuf,
}

/// Run the pipeline with real container execution.
///
/// Unlike [`run_pipeline`] which produces placeholder strings for exec tags,
/// this function starts WASM containers, runs commands, and captures output.
pub async fn run_pipeline_live(
    sources: &[(&str, &str)],
    config: &PipelineConfig,
) -> Result<PipelineResult> {
    let mut root_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut root_key);
    let authority = Arc::new(TokenAuthority::new(&root_key));

    // Parse all sources
    let mut documents = Vec::new();
    for (name, source) in sources {
        let doc =
            hick_lang::parse(source).map_err(|e| anyhow::anyhow!("parse error in {name}: {e}"))?;
        info!("Parsed {name}: {} top-level nodes", doc.nodes.len());
        documents.push((*name, doc));
    }

    // Collect all nodes across documents
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
        let _token = authority
            .mint(name, caps)
            .map_err(|e| anyhow::anyhow!("failed to mint token for '{name}': {e}"))?;
        info!("Minted token for container '{name}'");
    }

    // Build DAGs, run execs, collect outputs
    // Key: (container_name, source_line) -> captured output
    let mut exec_outputs: HashMap<(String, usize), String> = HashMap::new();
    let mut executor = ContainerExecutor::new(config.images_dir.clone(), container_defs.clone())?;

    for (name, doc) in &documents {
        let flow_dag = dag::build_dag(doc)
            .map_err(|e| anyhow::anyhow!("DAG validation failed in {name}: {e}"))?;
        info!(
            "DAG validated for {name}: {} exec nodes, running in topological order",
            flow_dag.execs.len()
        );

        for exec_id in flow_dag.topological_order() {
            let exec_info = flow_dag
                .execs
                .iter()
                .find(|e| e.id == exec_id)
                .unwrap();

            let image = exec_info.image.as_deref().unwrap_or("alpine");
            executor
                .ensure_started(&exec_info.container, image)
                .await?;

            let output = executor
                .execute(&exec_info.container, &exec_info.command)
                .await?;

            exec_outputs.insert(
                (exec_info.container.clone(), exec_info.source_line),
                output,
            );
        }
    }

    executor.shutdown().await;

    // Set up multi-document state
    let state = Arc::new(MultiDocumentState::default());

    // Process copy/cut blocks
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

    // Process file outputs — use real exec output instead of placeholders
    for node in &all_nodes {
        if let HickNode::Tag(tag) = node
            && tag.name == "file"
        {
            let path = tag_attr(tag, "path").unwrap_or_default();
            let file_ip = Arc::new(InsertionPoint::new());

            for child in &tag.children {
                match child {
                    HickNode::Text(text) => {
                        file_ip.add(Arc::new(StringNode::new(text.clone())));
                    }
                    HickNode::Tag(child_tag) if child_tag.name == "exec" => {
                        let container_name =
                            tag_attr(child_tag, "container").unwrap_or_default();
                        let key = (container_name.clone(), child_tag.source_line);
                        if let Some(output) = exec_outputs.get(&key) {
                            file_ip.add(Arc::new(StringNode::new(output.clone())));
                        } else {
                            warn!(
                                "No exec output for container '{}' at line {}",
                                container_name, child_tag.source_line
                            );
                            let placeholder =
                                format!("[exec in container '{container_name}']");
                            file_ip.add(Arc::new(StringNode::new(placeholder)));
                        }
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

    Ok(PipelineResult {
        files,
        containers: container_defs,
    })
}

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

/// Get an attribute value from a tag by name.
pub fn tag_attr(tag: &HickTag, name: &str) -> Option<String> {
    tag.attributes
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.clone())
}

/// Build `ContainerCapabilities` from a `<hick:container>` tag's children.
pub fn build_capabilities_from_tag(tag: &HickTag) -> ContainerCapabilities {
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
pub fn collect_text_children(tag: &HickTag) -> String {
    let mut result = String::new();
    for child in &tag.children {
        if let HickNode::Text(text) = child {
            result.push_str(text);
        }
    }
    result
}

/// Expand `~` to the user's home directory.
pub fn expand_tilde(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if s.starts_with("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home).join(&s[2..]);
    }
    path.to_path_buf()
}
