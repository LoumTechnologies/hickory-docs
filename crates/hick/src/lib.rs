//! Hick pipeline library.
//!
//! Provides the core pipeline for processing `.hick` sources: parsing,
//! DAG validation, container capability extraction, copy/paste resolution,
//! and file output collection — all without touching the filesystem.

pub mod cache;
pub mod condition;
pub mod config;
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

use crate::executor::{ContainerExecutor, ExecTranscriptEntry};

// ---------------------------------------------------------------------------
// Exec output control
// ---------------------------------------------------------------------------

/// Controls which parts of an exec transcript are rendered in file output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecShow {
    /// Show commands and output (default).
    All,
    /// Show commands only.
    Command,
    /// Show output only.
    Output,
    /// Show nothing (side-effect only).
    None,
}

fn parse_exec_show(tag: &HickTag) -> ExecShow {
    match tag_attr(tag, "show").as_deref() {
        Some("command") => ExecShow::Command,
        Some("output") => ExecShow::Output,
        Some("none") => ExecShow::None,
        _ => ExecShow::All,
    }
}

fn render_transcript(entries: &[ExecTranscriptEntry], show: ExecShow) -> String {
    let mut out = String::new();
    for entry in entries {
        if matches!(show, ExecShow::All | ExecShow::Command) {
            for cmd in &entry.commands {
                out.push_str(&format!("$ {cmd}\n"));
            }
        }
        if matches!(show, ExecShow::All | ExecShow::Output) && !entry.output.is_empty() {
            out.push_str(&entry.output);
            out.push('\n');
        }
    }
    out
}

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
///
/// `params` supplies CLI `--param key=value` pairs that override document
/// variables.
pub async fn run_pipeline(
    sources: &[(&str, &str)],
    params: &[(String, String)],
) -> Result<PipelineResult> {
    let mut root_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut root_key);
    let authority = Arc::new(TokenAuthority::new(&root_key));

    run_pipeline_with_authority(sources, &authority, params).await
}

/// Run the pipeline with a caller-supplied [`TokenAuthority`].
///
/// Useful for tests that need to verify tokens against the same authority.
pub async fn run_pipeline_with_authority(
    sources: &[(&str, &str)],
    authority: &Arc<TokenAuthority>,
    params: &[(String, String)],
) -> Result<PipelineResult> {
    // Parse all sources and resolve includes
    let mut documents = Vec::new();
    for (name, source) in sources {
        let mut doc =
            hick_lang::parse(source).map_err(|e| anyhow::anyhow!("parse error in {name}: {e}"))?;

        // Resolve includes relative to the source file's directory
        let base_dir = std::path::Path::new(name)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let mut seen = std::collections::HashSet::new();
        if let Ok(canonical) = std::fs::canonicalize(name) {
            seen.insert(canonical);
        }
        hick_lang::resolve_includes(&mut doc, base_dir, &mut seen)
            .map_err(|e| anyhow::anyhow!("include error in {name}: {e}"))?;

        info!("Parsed {name}: {} top-level nodes", doc.nodes.len());
        documents.push((*name, doc));
    }

    // Set up multi-document state and load variables BEFORE conditional filtering
    let state = Arc::new(MultiDocumentState::default());

    // Load CLI params first (highest priority)
    for (key, value) in params {
        state.set_param(key.clone(), value.clone());
    }

    // Scan all nodes for var declarations (including inside when blocks)
    for (_, doc) in &documents {
        scan_and_register_vars(&doc.nodes, &state);
    }

    // Filter conditional content (removes nodes with false conditions)
    for (_, doc) in &mut documents {
        filter_conditionals(&mut doc.nodes, &state);
    }

    // Build and validate DAG for each document (after conditional filtering)
    for (name, doc) in &documents {
        let dag = hick_exec::dag::build_dag(doc)
            .map_err(|e| anyhow::anyhow!("DAG validation failed in {name}: {e}"))?;
        info!("DAG validated for {name}: {} exec nodes", dag.execs.len());
    }

    // Collect all nodes across documents (after filtering)
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

    // Collect fork definitions: inherit source capabilities and attenuate
    for node in &all_nodes {
        if let HickNode::Tag(tag) = node
            && tag.name == "fork"
        {
            let from = tag_attr(tag, "from").unwrap_or_default();
            let to = tag_attr(tag, "to").unwrap_or_default();
            let additional_caps = build_fork_capabilities(tag);

            if let Some(source_caps) = container_defs.get(&from) {
                let mut forked_caps = source_caps.clone();
                if let Some(extra) = &additional_caps {
                    forked_caps
                        .network_rules
                        .extend(extra.network_rules.clone());
                    forked_caps.file_rules.extend(extra.file_rules.clone());
                }
                debug!("Fork '{from}' -> '{to}': {:?}", forked_caps);
                container_defs.insert(to, forked_caps);
            } else {
                // Source not explicitly declared; fork gets only its own caps
                let caps = additional_caps.unwrap_or_default();
                debug!("Fork '{from}' -> '{to}' (no source caps): {:?}", caps);
                container_defs.insert(to, caps);
            }
        }
    }

    // Mint tokens for each container
    for (name, caps) in &container_defs {
        let _token = authority
            .mint(name, caps)
            .map_err(|e| anyhow::anyhow!("failed to mint token for '{name}': {e}"))?;
        info!("Minted token for container '{name}'");
    }

    // Build dry-run transcripts: collect commands per container in DAG order
    let mut transcripts: HashMap<String, Vec<ExecTranscriptEntry>> = HashMap::new();
    for (_, doc) in &documents {
        let flow_dag = dag::build_dag(doc).unwrap();
        for exec_id in flow_dag.topological_order() {
            let info = flow_dag.execs.iter().find(|e| e.id == exec_id).unwrap();
            let commands: Vec<String> = info
                .command
                .trim()
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            transcripts
                .entry(info.container.clone())
                .or_default()
                .push(ExecTranscriptEntry {
                    commands,
                    output: String::new(),
                });
        }
    }

    // Process copy/cut blocks (they may be referenced by paste)
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

    // Process file outputs — use dry-run transcripts for exec tags
    for node in &all_nodes {
        if let HickNode::Tag(tag) = node
            && tag.name == "file"
        {
            let path = tag_attr(tag, "path").unwrap_or_default();
            let file_ip = Arc::new(InsertionPoint::new());

            process_file_children(&tag.children, &file_ip, &transcripts, &state);

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
///
/// When `cache_config` is `Some`, execution results are cached and reused
/// on subsequent runs if the cache key matches. In freeze mode, missing
/// cache entries produce an error instead of re-executing.
pub async fn run_pipeline_live(
    sources: &[(&str, &str)],
    config: &PipelineConfig,
    params: &[(String, String)],
    cache_config: Option<&cache::CacheConfig>,
) -> Result<PipelineResult> {
    let mut root_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut root_key);
    let authority = Arc::new(TokenAuthority::new(&root_key));

    // Parse all sources and resolve includes
    let mut documents = Vec::new();
    for (name, source) in sources {
        let mut doc =
            hick_lang::parse(source).map_err(|e| anyhow::anyhow!("parse error in {name}: {e}"))?;

        // Resolve includes relative to the source file's directory
        let base_dir = std::path::Path::new(name)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let mut seen = std::collections::HashSet::new();
        if let Ok(canonical) = std::fs::canonicalize(name) {
            seen.insert(canonical);
        }
        hick_lang::resolve_includes(&mut doc, base_dir, &mut seen)
            .map_err(|e| anyhow::anyhow!("include error in {name}: {e}"))?;

        info!("Parsed {name}: {} top-level nodes", doc.nodes.len());
        documents.push((*name, doc));
    }

    // Set up multi-document state and load variables BEFORE conditional filtering
    let state = Arc::new(MultiDocumentState::default());

    // Load CLI params first (highest priority)
    for (key, value) in params {
        state.set_param(key.clone(), value.clone());
    }

    // Scan all nodes for var declarations (including inside when blocks)
    for (_, doc) in &documents {
        scan_and_register_vars(&doc.nodes, &state);
    }

    // Filter conditional content (removes nodes with false conditions)
    for (_, doc) in &mut documents {
        filter_conditionals(&mut doc.nodes, &state);
    }

    // Collect all nodes across documents (after filtering)
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

    // Collect fork definitions: inherit source capabilities and attenuate
    let mut fork_registrations: Vec<(String, String, Option<ContainerCapabilities>)> = Vec::new();
    for node in &all_nodes {
        if let HickNode::Tag(tag) = node
            && tag.name == "fork"
        {
            let from = tag_attr(tag, "from").unwrap_or_default();
            let to = tag_attr(tag, "to").unwrap_or_default();
            let additional_caps = build_fork_capabilities(tag);

            if let Some(source_caps) = container_defs.get(&from) {
                let mut forked_caps = source_caps.clone();
                if let Some(extra) = &additional_caps {
                    forked_caps
                        .network_rules
                        .extend(extra.network_rules.clone());
                    forked_caps.file_rules.extend(extra.file_rules.clone());
                }
                debug!("Fork '{from}' -> '{to}': {:?}", forked_caps);
                container_defs.insert(to.clone(), forked_caps);
            } else {
                let caps = additional_caps.clone().unwrap_or_default();
                debug!("Fork '{from}' -> '{to}' (no source caps): {:?}", caps);
                container_defs.insert(to.clone(), caps);
            }

            fork_registrations.push((from, to, additional_caps));
        }
    }

    // Mint tokens for each container
    for (name, caps) in &container_defs {
        let _token = authority
            .mint(name, caps)
            .map_err(|e| anyhow::anyhow!("failed to mint token for '{name}': {e}"))?;
        info!("Minted token for container '{name}'");
    }

    // Build DAGs, run execs in containers (with optional caching)
    let mut executor = ContainerExecutor::new(config.images_dir.clone(), container_defs.clone())?;

    // Register fork definitions with the executor
    for (from, to, additional_caps) in &fork_registrations {
        executor.register_fork(to, from, additional_caps.clone());
    }

    for (name, doc) in &documents {
        let flow_dag = dag::build_dag(doc)
            .map_err(|e| anyhow::anyhow!("DAG validation failed in {name}: {e}"))?;
        info!(
            "DAG validated for {name}: {} exec nodes, running in topological order",
            flow_dag.execs.len()
        );

        for exec_id in flow_dag.topological_order() {
            let exec_info = flow_dag.execs.iter().find(|e| e.id == exec_id).unwrap();

            let image = exec_info.image.as_deref().unwrap_or("alpine");

            // Check cache before executing
            if let Some(cc) = cache_config {
                let caps_canonical = cache::canonical_caps(&container_defs, &exec_info.container);
                let secret_names = cache::secret_names_for(&container_defs, &exec_info.container);
                let secret_refs: Vec<&str> = secret_names.iter().map(|s| s.as_str()).collect();
                let key =
                    cache::exec_cache_key(image, &caps_canonical, &exec_info.command, &secret_refs);

                if let Some(cached) = cache::cache_lookup(cc, &exec_info.container, &key)? {
                    info!(
                        "Cache hit for exec in '{}': {}…",
                        exec_info.container,
                        &key[..12]
                    );
                    executor.inject_transcript_entry(
                        &exec_info.container,
                        ExecTranscriptEntry {
                            commands: cached.commands,
                            output: cached.output,
                        },
                    );
                    continue;
                } else if cc.freeze {
                    anyhow::bail!(
                        "Freeze mode: no cached result for exec in '{}' (command: {})",
                        exec_info.container,
                        exec_info.command.lines().next().unwrap_or("?"),
                    );
                }
            }

            // Cache miss or caching disabled — execute for real
            executor.ensure_started(&exec_info.container, image).await?;

            executor
                .execute(&exec_info.container, &exec_info.command)
                .await?;

            // Store result in cache after successful execution
            if let Some(cc) = cache_config {
                let caps_canonical = cache::canonical_caps(&container_defs, &exec_info.container);
                let secret_names = cache::secret_names_for(&container_defs, &exec_info.container);
                let secret_refs: Vec<&str> = secret_names.iter().map(|s| s.as_str()).collect();
                let key =
                    cache::exec_cache_key(image, &caps_canonical, &exec_info.command, &secret_refs);

                // Get the last transcript entry that was just added
                if let Some(entries) = executor.transcripts().get(&exec_info.container)
                    && let Some(last) = entries.last()
                {
                    let cache_entry = cache::ExecCacheEntry {
                        commands: last.commands.clone(),
                        output: last.output.clone(),
                        output_hash: cache::sha256_hex(&last.output),
                    };
                    cache::cache_store(cc, &exec_info.container, &key, &cache_entry)?;
                    info!("Cached exec in '{}': {}…", exec_info.container, &key[..12]);
                }
            }
        }
    }

    let transcripts = executor.transcripts().clone();
    executor.shutdown().await;

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

    // Process file outputs — use container transcripts for exec tags
    for node in &all_nodes {
        if let HickNode::Tag(tag) = node
            && tag.name == "file"
        {
            let path = tag_attr(tag, "path").unwrap_or_default();
            let file_ip = Arc::new(InsertionPoint::new());

            process_file_children(&tag.children, &file_ip, &transcripts, &state);

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
// Variable scanning
// ---------------------------------------------------------------------------

/// Recursively scan nodes for `<hick:var>` declarations and register them.
/// This runs before conditional filtering so that vars inside `<hick:when>`
/// blocks are available for condition evaluation.
fn scan_and_register_vars(nodes: &[HickNode], state: &MultiDocumentState) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            if tag.name == "var" {
                let name = tag_attr(tag, "name").unwrap_or_default();
                let value = collect_text_children(tag);
                state.register_var(name, value);
            }
            scan_and_register_vars(&tag.children, state);
        }
    }
}

// ---------------------------------------------------------------------------
// Conditional filtering
// ---------------------------------------------------------------------------

/// Remove nodes whose `when` condition evaluates to false.
/// `<hick:when test="...">` tags are unwrapped (children promoted) when true,
/// or removed entirely when false.
fn filter_conditionals(nodes: &mut Vec<HickNode>, state: &MultiDocumentState) {
    let mut new_nodes = Vec::new();
    for node in std::mem::take(nodes) {
        match node {
            HickNode::Tag(mut tag) if tag.name == "when" => {
                let test = tag_attr(&tag, "test").unwrap_or_default();
                let cond = condition::Condition::parse(&test);
                if cond.evaluate(state) {
                    // Condition true — unwrap children into parent
                    filter_conditionals(&mut tag.children, state);
                    new_nodes.extend(tag.children);
                }
                // Condition false — drop entire node
            }
            HickNode::Tag(mut tag) => {
                if let Some(when_val) = tag_attr(&tag, "when") {
                    let cond = condition::Condition::parse(&when_val);
                    if !cond.evaluate(state) {
                        continue; // Skip this node
                    }
                }
                filter_conditionals(&mut tag.children, state);
                new_nodes.push(HickNode::Tag(tag));
            }
            text => {
                new_nodes.push(text);
            }
        }
    }
    *nodes = new_nodes;
}

// ---------------------------------------------------------------------------
// File output helpers
// ---------------------------------------------------------------------------

/// Process children of a `<hick:file>` tag, adding rendered content to the
/// insertion point. Handles text nodes, exec transcript rendering (with
/// `show` attribute), paste resolution, and val interpolation.
fn process_file_children(
    children: &[HickNode],
    file_ip: &Arc<InsertionPoint>,
    transcripts: &HashMap<String, Vec<ExecTranscriptEntry>>,
    state: &MultiDocumentState,
) {
    for child in children {
        match child {
            HickNode::Text(text) => {
                file_ip.add(Arc::new(StringNode::new(text.clone())));
            }
            HickNode::Tag(child_tag) if child_tag.name == "exec" => {
                let container_name = tag_attr(child_tag, "container").unwrap_or_default();
                let show = parse_exec_show(child_tag);

                if show == ExecShow::None {
                    // Side-effect only, render nothing
                } else if let Some(entries) = transcripts.get(&container_name) {
                    let rendered = render_transcript(entries, show);
                    file_ip.add(Arc::new(StringNode::new(rendered)));
                } else {
                    file_ip.add(Arc::new(StringNode::new(format!(
                        "[no transcript for container '{container_name}']"
                    ))));
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
            HickNode::Tag(child_tag) if child_tag.name == "val" => {
                let var_name = tag_attr(child_tag, "name").unwrap_or_default();
                if let Some(value) = state.resolve_var(&var_name) {
                    file_ip.add(Arc::new(StringNode::new(value)));
                } else {
                    warn!("Variable '{var_name}' not found");
                }
            }
            _ => {}
        }
    }
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

/// Build additional capabilities from a `<hick:fork>` tag's children.
///
/// Returns `None` if the fork tag has no `<deny>` or `<allow>` children,
/// meaning no additional restrictions are applied to the fork.
pub fn build_fork_capabilities(tag: &HickTag) -> Option<ContainerCapabilities> {
    let mut has_rules = false;
    let mut caps = ContainerCapabilities::new();

    for child in &tag.children {
        if let HickNode::Tag(child_tag) = child {
            match child_tag.name.as_str() {
                "allow" => {
                    has_rules = true;
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
                    has_rules = true;
                    if tag_attr(child_tag, "network").is_some() {
                        caps.network_rules.push(NetworkRule::deny_all());
                    }
                }
                _ => {}
            }
        }
    }

    if has_rules { Some(caps) } else { None }
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
