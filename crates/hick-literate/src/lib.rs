//! Hick pipeline library.
//!
//! Provides the core pipeline for processing `.hick` sources: parsing,
//! DAG validation, container capability extraction, copy/paste resolution,
//! and file output collection — all without touching the filesystem.

pub mod agents;
pub mod auth;
pub mod pipeline;
pub mod cache;
pub mod compact;
pub mod config;
pub mod promote;
pub mod equiv;
pub mod executor;
pub mod generate_matrix;
pub mod output_cleanup;
pub mod pool;
pub mod repl;
pub mod store_config;
mod text;
pub mod transcript;
pub mod visual_regression;
pub mod volume_state;
pub mod watch;
mod weave;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use log::{debug, info, warn};
use rand::RngCore;

use hick_exec::dag;
use hick_exec::node::{
    FileContent, InsertionPoint, Node, ProvenanceMap, ProvenanceTransformNode, SourceOrigin,
    SpanNode, StringNode, TransformSegment, TransformSegmentOrigin,
};
use hick_exec::state::{FeatureInfo, MultiDocumentState};
use hick_handlers::{ProcessingContext, ProcessingPhase, TagRegistry, TagResult, TranscriptEntry};

use hick_feature::{FeatureDef, FeatureRegistry, FeatureSet};
use hick_lang::{HickDocument, HickNode, HickTag, dedent};
use hick_token::{ContainerCapabilities, NetworkRule, TokenAuthority};

use crate::executor::{ContainerExecutor, ContainerResourceStats, ExecTranscriptEntry};
use crate::pool::SharedPool;
use hick_condition::Condition;

// ---------------------------------------------------------------------------
// Pipeline result
// ---------------------------------------------------------------------------

/// The output of a hick pipeline run.
pub struct PipelineResult {
    /// File path -> rendered content for each `<hick:file>` tag.
    pub files: HashMap<String, FileContent>,
    /// File path -> character-level provenance map for each `<hick:file>` output.
    /// Maps output byte ranges back to their source `.hick` file positions.
    pub provenance_maps: HashMap<String, ProvenanceMap>,
    /// Container name -> parsed capabilities for each `<hick:container>` tag.
    pub containers: HashMap<String, ContainerCapabilities>,
    /// Volume name -> list of containers that contributed to its final state.
    pub volume_provenance: HashMap<String, Vec<String>>,
    /// Per-container resource stats (boot time, exec time, I/O, pause info).
    /// Empty for dry-run pipelines.
    pub resource_stats: HashMap<String, ContainerResourceStats>,
    /// Number of containers adopted from the pool (vs fresh boot).
    /// Zero when no pool is used.
    pub pool_hits: usize,
    /// Total boot time saved by adopting pre-warmed containers from the pool.
    /// Zero when no pool is used or in dry-run mode.
    pub pool_boot_time_saved: Duration,
}

// ---------------------------------------------------------------------------
// Pipeline setup (shared between dry-run and live)
// ---------------------------------------------------------------------------

/// Intermediate state after parsing, filtering, and collecting definitions.
struct PreparedPipeline<'a> {
    documents: Vec<(&'a str, HickDocument)>,
    state: Arc<MultiDocumentState>,
    container_defs: HashMap<String, ContainerCapabilities>,
    /// Fork registrations: (from, to, additional_caps).
    /// Needed by live pipeline to register forks with the executor.
    fork_registrations: Vec<(String, String, Option<ContainerCapabilities>)>,
}

/// Shared prologue: parse sources, resolve includes, set up state, process
/// features, filter conditionals, collect containers and forks, mint tokens.
fn prepare_pipeline<'a>(
    sources: &[(&'a str, &str)],
    authority: &Arc<TokenAuthority>,
    params: &[(String, String)],
) -> Result<PreparedPipeline<'a>> {
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

    // Process features BEFORE conditional filtering (so features work in conditions)
    let all_docs: Vec<_> = documents.iter().map(|(_, d)| d).collect();
    process_features(&all_docs, &state)?;
    drop(all_docs);

    // Filter conditional content (removes nodes with false conditions)
    for (_, doc) in &mut documents {
        filter_conditionals(&mut doc.nodes, &state);
    }

    // Build and validate DAG for each document (after conditional filtering)
    for (name, doc) in &documents {
        let dag_result = hick_exec::dag::build_dag(doc)
            .map_err(|e| anyhow::anyhow!("DAG validation failed in {name}: {e}"))?;
        info!(
            "DAG validated for {name}: {} exec nodes",
            dag_result.execs.len()
        );
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
                let forked_caps = if let Some(extra) = &additional_caps {
                    source_caps.merge_caveats(extra)
                } else {
                    source_caps.clone()
                };
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

    Ok(PreparedPipeline {
        documents,
        state,
        container_defs,
        fork_registrations,
    })
}

/// Create the tag registry with all built-in handlers.
fn create_tag_registry() -> TagRegistry {
    let mut registry = TagRegistry::new();
    hick_handlers::handlers::register_builtins(&mut registry);
    hick_live::handlers::register_live_builtins(&mut registry);
    registry
}

/// Convert executor transcript entries to handler transcript entries.
fn convert_transcripts(
    exec_transcripts: &HashMap<String, Vec<ExecTranscriptEntry>>,
) -> HashMap<String, Vec<TranscriptEntry>> {
    exec_transcripts
        .iter()
        .map(|(k, entries)| {
            let converted: Vec<TranscriptEntry> = entries
                .iter()
                .map(|e| TranscriptEntry {
                    commands: e.commands.clone(),
                    output: e.output.clone(),
                })
                .collect();
            (k.clone(), converted)
        })
        .collect()
}

/// Shared epilogue: register copy/cut/substitute/exclude blocks, process
/// file outputs, weave output, and converge files with exclusion filtering.
async fn process_pipeline_outputs(
    documents: &[(&str, HickDocument)],
    state: &Arc<MultiDocumentState>,
    transcripts: &HashMap<String, Vec<ExecTranscriptEntry>>,
    max_rounds: usize,
) -> (HashMap<String, FileContent>, HashMap<String, ProvenanceMap>) {
    let registry = create_tag_registry();
    let handler_transcripts = convert_transcripts(transcripts);

    // --- Round 0: process the original documents (identical to previous behavior) ---
    process_documents_round(documents, state, &handler_transcripts, &registry);

    // Process weave output if enabled
    weave::process_weave_output(documents, &handler_transcripts, state, &registry);

    // --- Rounds 1..max_rounds: re-evaluate .hick files produced by containers ---
    if max_rounds > 1 {
        let mut prev_hick_paths: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for round in 1..max_rounds {
            let files = state.get_files().await;

            // Find new .hick files in output
            let hick_sources: Vec<(String, String)> = files
                .iter()
                .filter(|(path, _)| path.ends_with(".hick"))
                .filter_map(|(path, content)| {
                    if let FileContent::Text(s) = content {
                        Some((path.clone(), s.clone()))
                    } else {
                        None
                    }
                })
                .collect();

            // Check convergence: no new .hick files
            let current_paths: std::collections::HashSet<String> =
                hick_sources.iter().map(|(p, _)| p.clone()).collect();
            if current_paths.is_subset(&prev_hick_paths) {
                info!("Pipeline converged after {round} rounds (no new .hick files)");
                break;
            }
            prev_hick_paths = current_paths;

            if hick_sources.is_empty() {
                break;
            }

            // Parse new .hick sources and process them
            let mut new_documents = Vec::new();
            for (path, source) in &hick_sources {
                match hick_lang::parse(source) {
                    Ok(doc) => {
                        info!("Round {round}: processing generated .hick file: {path}");
                        new_documents.push((path.as_str(), doc));
                    }
                    Err(e) => {
                        warn!("Round {round}: failed to parse generated .hick file {path}: {e}");
                    }
                }
            }

            if new_documents.is_empty() {
                break;
            }

            // Process the new documents through declaration + content phases
            process_documents_round(&new_documents, state, &handler_transcripts, &registry);
        }
    }

    // Converge file outputs with provenance
    let files_with_prov = state.get_files_with_provenance().await;

    let mut files = HashMap::new();
    let mut provenance_maps = HashMap::new();
    for (path, (content, prov_map)) in files_with_prov {
        files.insert(path.clone(), content);
        provenance_maps.insert(path, prov_map);
    }

    // Apply exclusion patterns
    let files = apply_exclusions(files, &state.get_exclusion_patterns());

    // Remove provenance maps for excluded files
    provenance_maps.retain(|path, _| files.contains_key(path));

    (files, provenance_maps)
}

/// Process a batch of documents through declaration and content phases.
fn process_documents_round(
    documents: &[(&str, HickDocument)],
    state: &Arc<MultiDocumentState>,
    handler_transcripts: &HashMap<String, Vec<TranscriptEntry>>,
    registry: &TagRegistry,
) {
    let all_nodes: Vec<&HickNode> = documents
        .iter()
        .flat_map(|(_, doc)| doc.nodes.iter())
        .collect();

    // Declaration phase: process copy/cut/substitute/exclude via handlers
    let decl_ctx = ProcessingContext {
        state,
        transcripts: handler_transcripts,
        indent: 0,
        registry: Some(registry),
        context: None,
        source_file: None,
    };
    for node in &all_nodes {
        if let HickNode::Tag(tag) = node
            && let Some(handler) = registry.find(&tag.name)
            && handler.phase() == ProcessingPhase::Declaration
        {
            let _ = handler.process(tag, &decl_ctx);
        }
    }

    // Process file outputs
    for (doc_name, doc) in documents {
        let source_file: Arc<str> = Arc::from(*doc_name);
        for node in &doc.nodes {
            if let HickNode::Tag(tag) = node
                && tag.name == "file"
            {
                let raw_path = tag_attr(tag, "path").unwrap_or_default();
                let path = interpolate_path(&raw_path, state);
                let raw_ip = Arc::new(InsertionPoint::new());

                process_file_children(
                    &tag.children,
                    &raw_ip,
                    handler_transcripts,
                    state,
                    tag.source_column,
                    registry,
                    Some(&source_file),
                );

                raw_ip.close();

                let subs_state = state.clone();
                let transform: Arc<dyn Node> = Arc::new(ProvenanceTransformNode::new(
                    raw_ip,
                    move |text| apply_substitutions_segmented_to_transform(text, &subs_state),
                    "substitute",
                ));

                let file_ip = Arc::new(InsertionPoint::new());
                file_ip.add(transform);
                file_ip.close();
                state.add_file_output(path, file_ip);
            }
        }
    }
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
    let prepared = prepare_pipeline(sources, authority, params)?;
    let PreparedPipeline {
        documents,
        state,
        container_defs,
        ..
    } = prepared;

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

    let documents_ref: Vec<(&str, HickDocument)> =
        documents.iter().map(|(n, d)| (*n, d.clone())).collect();
    let (files, provenance_maps) =
        process_pipeline_outputs(&documents_ref, &state, &transcripts, 1).await;

    Ok(PipelineResult {
        files,
        provenance_maps,
        containers: container_defs,
        volume_provenance: HashMap::new(),
        resource_stats: HashMap::new(),
        pool_hits: 0,
        pool_boot_time_saved: Duration::ZERO,
    })
}

// ---------------------------------------------------------------------------
// Live pipeline (real container execution)
// ---------------------------------------------------------------------------

/// Configuration for live pipeline execution.
pub struct PipelineConfig {
    /// Directory containing pre-converted `.wasm` images.
    pub images_dir: PathBuf,
    /// Directory containing `.wasm` command binaries for `<hick:script>` blocks.
    /// Each `.wasm` file represents a command available to the shell interpreter.
    pub toolchain_dir: Option<PathBuf>,
    /// Working directory for resolving relative paths in volume declarations.
    pub working_dir: Option<PathBuf>,
    /// Maximum pipeline rounds. `1` (default) preserves single-pass behavior.
    /// Higher values enable reactive re-evaluation of paste selectors that
    /// reference content from container-generated `.hick` files.
    pub max_rounds: usize,
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
    pool: Option<&SharedPool>,
) -> Result<PipelineResult> {
    let mut root_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut root_key);
    let authority = Arc::new(TokenAuthority::new(&root_key));

    let prepared = prepare_pipeline(sources, &authority, params)?;
    let PreparedPipeline {
        documents,
        state,
        container_defs,
        fork_registrations,
    } = prepared;

    // Build DAGs, run execs in containers (with optional caching)
    let mut executor = if let Some(pool) = pool {
        ContainerExecutor::new_with_pool(
            config.images_dir.clone(),
            container_defs.clone(),
            pool.clone(),
        )
        .await?
    } else {
        ContainerExecutor::new(config.images_dir.clone(), container_defs.clone())?
    };

    // Register fork definitions with the executor
    for (from, to, additional_caps) in &fork_registrations {
        executor.register_fork(to, from, additional_caps.clone());
    }

    // Collect volume declarations and set up volume store
    let mut volume_store = volume_state::VolumeStore::new();
    let mut all_volume_decls: HashMap<String, hick_exec::volume::VolumeDeclaration> =
        HashMap::new();
    let mut volume_provenance: HashMap<String, Vec<String>> = HashMap::new();

    for (name, doc) in &documents {
        let flow_dag = dag::build_dag(doc)
            .map_err(|e| anyhow::anyhow!("DAG validation failed in {name}: {e}"))?;
        for (vol_name, vol_decl) in &flow_dag.volumes {
            all_volume_decls.insert(vol_name.clone(), vol_decl.clone());
        }
    }

    // Seed input volumes from host directories
    let working_dir = config
        .working_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    for (vol_name, vol_decl) in &all_volume_decls {
        match &vol_decl.kind {
            hick_exec::volume::VolumeKind::Input { path }
            | hick_exec::volume::VolumeKind::InputOutput { input: path, .. } => {
                let abs_path = if Path::new(path).is_absolute() {
                    PathBuf::from(path)
                } else {
                    working_dir.join(path)
                };
                if abs_path.is_dir() {
                    info!(
                        "Seeding input volume '{vol_name}' from {}",
                        abs_path.display()
                    );
                    volume_store.seed_from_directory(vol_name, &abs_path)?;
                } else {
                    warn!(
                        "Input volume '{vol_name}' path does not exist: {}",
                        abs_path.display()
                    );
                }
            }
            _ => {}
        }
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
            if exec_info.is_script {
                // Run via hick-shell lightweight interpreter
                let toolchain_dir = exec_info
                    .toolchain
                    .as_deref()
                    .map(PathBuf::from)
                    .or_else(|| config.toolchain_dir.clone())
                    .unwrap_or_else(|| config.images_dir.clone());

                let env_vars: HashMap<String, String> = HashMap::new();
                let preopened_dirs: Vec<(PathBuf, String)> = exec_info
                    .mounts
                    .iter()
                    .map(|(_, mount_path)| {
                        (PathBuf::from(mount_path), mount_path.clone())
                    })
                    .collect();

                let output = hick_shell::run_script(
                    &exec_info.command,
                    &toolchain_dir,
                    &preopened_dirs,
                    &env_vars,
                )
                .map_err(|e| anyhow::anyhow!("script execution failed in '{}': {e}", exec_info.container))?;

                info!(
                    "Script '{}' exited with code {}",
                    exec_info.container, output.exit_code
                );

                executor.inject_transcript_entry(
                    &exec_info.container,
                    ExecTranscriptEntry {
                        commands: vec![exec_info.command.clone()],
                        output: output.stdout,
                    },
                );
            } else {
                // Container-based execution
                executor.ensure_started(&exec_info.container, image).await?;

                // Inject volumes before exec (or just create the mount point
                // for the first writer when the volume is still empty).
                for (vol_name, mount_path) in &exec_info.mounts {
                    if let Some(tar_data) = volume_store.get(vol_name) {
                        info!(
                            "Injecting volume '{vol_name}' into container '{}' at {mount_path}",
                            exec_info.container
                        );
                        executor
                            .inject_volume(&exec_info.container, mount_path, tar_data)
                            .await?;
                    } else {
                        executor
                            .create_mount_point(&exec_info.container, mount_path)
                            .await?;
                    }
                }

                // Evaluate stdin children if present
                let stdin_content = if !exec_info.stdin_children.is_empty() {
                    let stdin_registry = create_tag_registry();
                    let current_transcripts = convert_transcripts(executor.transcripts());
                    let mut parts = Vec::new();
                    for child in &exec_info.stdin_children {
                        match child {
                            hick_lang::HickNode::Text(t, _) => parts.push(t.clone()),
                            hick_lang::HickNode::Tag(child_tag) => {
                                if let Some(handler) = stdin_registry.find(&child_tag.name) {
                                    let child_ctx = ProcessingContext {
                                        state: &state,
                                        transcripts: &current_transcripts,
                                        indent: 0,
                                        registry: Some(&stdin_registry),
                                        context: None,
                                        source_file: None,
                                    };
                                    if let Ok(TagResult::Node(n)) =
                                        handler.process(child_tag, &child_ctx)
                                        && let Some(s) = n.as_string_value()
                                    {
                                        parts.push(s.to_string());
                                    }
                                }
                            }
                        }
                    }
                    let joined = parts.join("");
                    if joined.is_empty() {
                        None
                    } else {
                        Some(joined)
                    }
                } else {
                    None
                };

                if let Some(stdin_data) = &stdin_content {
                    executor
                        .execute_with_stdin(&exec_info.container, &exec_info.command, stdin_data)
                        .await?;
                } else {
                    executor
                        .execute(&exec_info.container, &exec_info.command)
                        .await?;
                }

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

                // Extract volumes after exec (if container has write access)
                for (vol_name, mount_path) in &exec_info.mounts {
                    let has_write = if let Some(decl) = all_volume_decls.get(vol_name) {
                        decl.check_write(&exec_info.container, "**") || decl.access_rules.is_empty()
                    } else {
                        true // no declaration = unrestricted
                    };

                    if has_write {
                        info!(
                            "Extracting volume '{vol_name}' from container '{}' at {mount_path}",
                            exec_info.container
                        );
                        let tar_data = executor
                            .extract_volume(&exec_info.container, mount_path)
                            .await?;
                        volume_store.update(vol_name, tar_data);
                        volume_provenance
                            .entry(vol_name.clone())
                            .or_default()
                            .push(exec_info.container.clone());
                    }
                }
            }
        }
    }

    // Flush output volumes into pipeline result files
    let mut volume_files: HashMap<String, FileContent> = HashMap::new();
    for (vol_name, vol_decl) in &all_volume_decls {
        let output_prefix = match &vol_decl.kind {
            hick_exec::volume::VolumeKind::Output { path } => Some(path.as_str()),
            hick_exec::volume::VolumeKind::InputOutput { output, .. } => Some(output.as_str()),
            _ => None,
        };

        if let Some(prefix) = output_prefix
            && volume_store.contains(vol_name)
        {
            let unpacked = volume_store.unpack_to_files(vol_name)?;
            for (file_path, content) in unpacked {
                let output_path = if prefix.is_empty() || prefix == "." {
                    file_path
                } else {
                    format!(
                        "{}{}",
                        prefix.trim_end_matches('/'),
                        if file_path.starts_with('/') {
                            file_path
                        } else {
                            format!("/{file_path}")
                        }
                    )
                };
                volume_files.insert(output_path, FileContent::Text(content));
            }
        }
    }

    let transcripts = executor.transcripts().clone();
    let pool_hits = executor.pool_hits();
    let pool_boot_time_saved = executor.pool_boot_time_saved();
    executor.shutdown().await;
    let resource_stats = executor.resource_stats().clone();

    let documents_ref: Vec<(&str, HickDocument)> =
        documents.iter().map(|(n, d)| (*n, d.clone())).collect();
    let (mut files, provenance_maps) =
        process_pipeline_outputs(&documents_ref, &state, &transcripts, config.max_rounds).await;

    // Merge volume output files into the result
    files.extend(volume_files);

    Ok(PipelineResult {
        files,
        provenance_maps,
        containers: container_defs,
        volume_provenance,
        resource_stats,
        pool_hits,
        pool_boot_time_saved,
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
// Feature processing
// ---------------------------------------------------------------------------

/// Process feature definitions from all documents.
///
/// This scans for `<hick:feature>` tags, validates dependencies, and enables
/// features based on the `features` parameter. Enabled features are set as
/// variables so they can be used in `when` conditions.
fn process_features(
    documents: &[&hick_lang::HickDocument],
    state: &MultiDocumentState,
) -> Result<()> {
    // Collect feature definitions from all documents
    let mut feature_registry = FeatureRegistry::new();

    for doc in documents {
        scan_and_register_features(&doc.nodes, &feature_registry, state);
    }

    // Re-scan to actually register (we need mutable access)
    for doc in documents {
        collect_feature_defs(&doc.nodes, &mut feature_registry, state);
    }

    // Validate feature registry (checks for circular deps and unknown deps)
    if let Err(e) = feature_registry.validate() {
        let available = feature_registry
            .all_features()
            .map(|(name, def)| format!("  {} - {}", name, def.description))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(anyhow::anyhow!(
            "Feature validation error: {e}\n\nAvailable features:\n{available}"
        ));
    }

    // Process enabled features (from --features param or "features" var)
    if let Some(features_param) = state.resolve_var("features") {
        let requested = FeatureSet::parse(&features_param);
        match requested.validate_and_expand(&feature_registry) {
            Ok(expanded) => {
                // Enable all features (including transitive deps)
                for name in expanded.enabled() {
                    state.enable_feature(name.to_string());
                    // Also set as a variable so conditions like `when="auth"` work
                    state.set_param(name.to_string(), "1".to_string());
                }
            }
            Err(e) => {
                let available = feature_registry
                    .all_features()
                    .map(|(name, def)| format!("  {} - {}", name, def.description))
                    .collect::<Vec<_>>()
                    .join("\n");
                return Err(anyhow::anyhow!(
                    "Feature error: {e}\n\nAvailable features:\n{available}"
                ));
            }
        }
    }

    Ok(())
}

/// First pass: just scan to register features with state (immutable registry).
fn scan_and_register_features(
    nodes: &[HickNode],
    _registry: &FeatureRegistry,
    _state: &MultiDocumentState,
) {
    // This pass just ensures we find all feature tags
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            scan_and_register_features(&tag.children, _registry, _state);
        }
    }
}

/// Second pass: collect feature definitions into the registry.
fn collect_feature_defs(
    nodes: &[HickNode],
    registry: &mut FeatureRegistry,
    state: &MultiDocumentState,
) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            if tag.name == "feature" {
                let name = tag_attr(tag, "name").unwrap_or_default();
                let description = tag_attr(tag, "description").unwrap_or_default();
                let requires_str = tag_attr(tag, "requires").unwrap_or_default();
                let requires: Vec<String> = requires_str
                    .split_whitespace()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();

                // Register in the feature registry for validation
                let mut def = FeatureDef::new(name.clone(), description.clone());
                for req in &requires {
                    def = def.with_requires(req.clone());
                }
                registry.register(def);

                // Also register in state for later access
                state.register_feature(FeatureInfo {
                    name,
                    description,
                    requires,
                });
            }
            collect_feature_defs(&tag.children, registry, state);
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
                let cond = Condition::parse(&test);
                if cond.evaluate(state) {
                    // Condition true — unwrap children into parent
                    filter_conditionals(&mut tag.children, state);
                    new_nodes.extend(tag.children);
                }
                // Condition false — drop entire node
            }
            HickNode::Tag(mut tag) => {
                if let Some(when_val) = tag_attr(&tag, "when") {
                    let cond = Condition::parse(&when_val);
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

use text::{apply_exclusions, apply_substitutions_segmented, interpolate_path};

/// Bridge from `text::TransformSegment` to `hick_flow::TransformSegment`.
fn apply_substitutions_segmented_to_transform(
    text: &str,
    state: &MultiDocumentState,
) -> Vec<TransformSegment> {
    apply_substitutions_segmented(text, state)
        .into_iter()
        .map(|seg| TransformSegment {
            text: seg.text,
            origin: match seg.origin {
                text::SegmentOrigin::Passthrough => TransformSegmentOrigin::Passthrough,
                text::SegmentOrigin::Substituted { pattern, .. } => {
                    TransformSegmentOrigin::Substituted { pattern }
                }
            },
        })
        .collect()
}

/// Process children of a `<hick:file>` tag, adding rendered content to the
/// insertion point. Delegates content tags (exec, paste, val) to the handler
/// registry.
///
/// The `indent` parameter specifies how many leading spaces to strip from each
/// line of text content (typically the source_column of the parent tag).
fn process_file_children(
    children: &[HickNode],
    file_ip: &Arc<InsertionPoint>,
    transcripts: &HashMap<String, Vec<TranscriptEntry>>,
    state: &Arc<MultiDocumentState>,
    indent: usize,
    registry: &TagRegistry,
    source_file: Option<&Arc<str>>,
) {
    let ctx = ProcessingContext {
        state,
        transcripts,
        indent,
        registry: Some(registry),
        context: None,
        source_file: source_file.cloned(),
    };

    for child in children {
        match child {
            HickNode::Text(text, span) => {
                let dedented = dedent(text, indent);
                if let (Some(span), Some(source_file)) = (span, ctx.source_file.as_ref()) {
                    let origin = SourceOrigin::Literal {
                        file: source_file.clone(),
                        span: *span,
                    };
                    file_ip.add(Arc::new(SpanNode::new(dedented, origin)));
                } else {
                    file_ip.add(Arc::new(StringNode::new(dedented)));
                }
            }
            HickNode::Tag(child_tag) => {
                if let Some(handler) = registry.find(&child_tag.name) {
                    match handler.process(child_tag, &ctx) {
                        Ok(TagResult::Node(n)) => file_ip.add(n),
                        Ok(TagResult::Nodes(ns)) => {
                            for n in ns {
                                file_ip.add(n);
                            }
                        }
                        Ok(TagResult::Declaration) => {}
                        Err(e) => {
                            warn!("Handler error for <{}>: {}", child_tag.name, e);
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Multi-stage pipeline
// ---------------------------------------------------------------------------

/// Result of a multi-stage pipeline run.
#[derive(Debug)]
pub struct MultiStagePipelineResult {
    /// Combined file outputs from all stages (later stages override earlier).
    pub files: HashMap<String, FileContent>,
    /// Provenance for each output file.
    pub provenance: HashMap<String, hick_store::FileProvenance>,
}

/// Run a multi-stage pipeline.
///
/// Detects `.hick` files in the output of each stage and feeds them as
/// sources to the next stage. Repeats up to `max_stages` times.
///
/// Files with `.hick` extension are removed from the final output (they
/// are intermediate).
pub async fn run_pipeline_multi_stage(
    sources: &[(&str, &str)],
    params: &[(String, String)],
    max_stages: usize,
) -> Result<MultiStagePipelineResult> {
    let mut all_files: HashMap<String, FileContent> = HashMap::new();
    let mut provenance: HashMap<String, hick_store::FileProvenance> = HashMap::new();

    // Build owned sources for first stage
    let mut current_sources: Vec<(String, String)> = sources
        .iter()
        .map(|(n, s)| (n.to_string(), s.to_string()))
        .collect();

    for stage in 0..max_stages {
        let stage_num = stage + 1;
        info!("Running pipeline stage {stage_num}");

        let source_refs: Vec<(&str, &str)> = current_sources
            .iter()
            .map(|(n, s)| (n.as_str(), s.as_str()))
            .collect();

        let result = run_pipeline(&source_refs, params).await?;

        // Separate .hick outputs from regular outputs
        let mut hick_files: Vec<(String, String)> = Vec::new();
        let mut regular_files: HashMap<String, FileContent> = HashMap::new();

        for (path, content) in result.files {
            if path.ends_with(".hick") {
                // .hick files must be text to feed as sources to next stage
                let text = match content {
                    FileContent::Text(s) => s,
                    FileContent::Binary(_) => {
                        warn!("Binary .hick file '{}' skipped", path);
                        continue;
                    }
                };
                hick_files.push((path, text));
            } else {
                regular_files.insert(path, content);
            }
        }

        // Add regular files to combined output (later stages override)
        for (path, content) in regular_files {
            let prov = if stage == 0 {
                hick_store::FileProvenance::HickFile {
                    source: current_sources
                        .first()
                        .map(|(n, _)| n.clone())
                        .unwrap_or_default(),
                }
            } else {
                hick_store::FileProvenance::GeneratedHick {
                    source_container: format!("stage-{stage_num}"),
                }
            };
            provenance.insert(path.clone(), prov);
            all_files.insert(path, content);
        }

        // If no .hick files produced, we're done
        if hick_files.is_empty() {
            info!("Stage {stage_num} produced no .hick files; pipeline complete");
            return Ok(MultiStagePipelineResult {
                files: all_files,
                provenance,
            });
        }

        // Feed .hick files as sources to next stage
        info!(
            "Stage {stage_num} produced {} .hick file(s); feeding to stage {}",
            hick_files.len(),
            stage_num + 1
        );
        current_sources = hick_files;
    }

    // Reached max stages and still producing .hick files
    let offending: Vec<String> = current_sources.iter().map(|(n, _)| n.clone()).collect();
    Err(anyhow::anyhow!(
        "multi-stage pipeline reached limit ({max_stages} stages) but still producing .hick files: {}",
        offending.join(", ")
    ))
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
        if let HickNode::Text(text, _) = child {
            result.push_str(text);
        }
    }
    result
}

/// Format a duration for human-readable display.
pub fn fmt_duration(d: Duration) -> String {
    if d.as_secs() >= 60 {
        format!(
            "{}m {:.1}s",
            d.as_secs() / 60,
            (d.as_millis() % 60_000) as f64 / 1000.0
        )
    } else if d.as_millis() >= 1000 {
        format!("{:.2}s", d.as_secs_f64())
    } else {
        format!("{}ms", d.as_millis())
    }
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

// ---------------------------------------------------------------------------
// Feature extraction (for generate-matrix)
// ---------------------------------------------------------------------------

/// Extracted feature definition for external use.
#[derive(Debug, Clone)]
pub struct ExtractedFeatureDef {
    pub name: String,
    pub description: String,
    pub requires: Vec<String>,
    /// Features that conflict with this one (mutually exclusive).
    pub conflicts_with: Vec<String>,
    /// Exclusive group name - only one feature from each group can be enabled.
    pub exclusive_group: Option<String>,
}

/// Extract feature definitions from hick sources without running the pipeline.
///
/// This is useful for tools that need to inspect features without executing
/// containers (e.g., the generate-matrix command).
pub fn extract_feature_definitions(
    sources: &[(&str, &str)],
) -> Result<HashMap<String, ExtractedFeatureDef>> {
    let mut features: HashMap<String, ExtractedFeatureDef> = HashMap::new();

    for (name, source) in sources {
        let mut doc =
            hick_lang::parse(source).map_err(|e| anyhow::anyhow!("parse error in {name}: {e}"))?;

        // Resolve includes
        let base_dir = std::path::Path::new(name)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let mut seen = std::collections::HashSet::new();
        if let Ok(canonical) = std::fs::canonicalize(name) {
            seen.insert(canonical);
        }
        hick_lang::resolve_includes(&mut doc, base_dir, &mut seen)
            .map_err(|e| anyhow::anyhow!("include error in {name}: {e}"))?;

        // Extract features from nodes
        extract_features_from_nodes(&doc.nodes, &mut features);
    }

    Ok(features)
}

/// Recursively extract feature definitions from nodes.
fn extract_features_from_nodes(
    nodes: &[HickNode],
    features: &mut HashMap<String, ExtractedFeatureDef>,
) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            if tag.name == "feature" {
                let name = tag_attr(tag, "name").unwrap_or_default();
                let description = tag_attr(tag, "description").unwrap_or_default();

                // Parse space-separated list of required features
                let requires_str = tag_attr(tag, "requires").unwrap_or_default();
                let requires: Vec<String> = requires_str
                    .split_whitespace()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();

                // Parse space-separated list of conflicting features
                let conflicts_str = tag_attr(tag, "conflicts_with").unwrap_or_default();
                let conflicts_with: Vec<String> = conflicts_str
                    .split_whitespace()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();

                // Parse optional exclusive group
                let exclusive_group = tag_attr(tag, "exclusive_group")
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());

                features.insert(
                    name.clone(),
                    ExtractedFeatureDef {
                        name,
                        description,
                        requires,
                        conflicts_with,
                        exclusive_group,
                    },
                );
            }
            // Recurse into children
            extract_features_from_nodes(&tag.children, features);
        }
    }
}

// ---------------------------------------------------------------------------
// Verify command extraction (for generate-matrix)
// ---------------------------------------------------------------------------

/// Extracted verify command from a `<hick:verify>` element.
#[derive(Debug, Clone)]
pub struct ExtractedVerifyCommand {
    pub command: String,
    pub description: Option<String>,
    pub source_file: String,
    pub source_line: usize,
}

/// Extract verify commands from hick sources without running the pipeline.
pub fn extract_verify_commands(sources: &[(&str, &str)]) -> Result<Vec<ExtractedVerifyCommand>> {
    let mut commands = Vec::new();

    for (name, source) in sources {
        let mut doc =
            hick_lang::parse(source).map_err(|e| anyhow::anyhow!("parse error in {name}: {e}"))?;

        // Resolve includes
        let base_dir = std::path::Path::new(name)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let mut seen = std::collections::HashSet::new();
        if let Ok(canonical) = std::fs::canonicalize(name) {
            seen.insert(canonical);
        }
        hick_lang::resolve_includes(&mut doc, base_dir, &mut seen)
            .map_err(|e| anyhow::anyhow!("include error in {name}: {e}"))?;

        extract_verify_from_nodes(&doc.nodes, name, &mut commands);
    }

    Ok(commands)
}

/// Recursively extract verify commands from nodes.
fn extract_verify_from_nodes(
    nodes: &[HickNode],
    source_file: &str,
    commands: &mut Vec<ExtractedVerifyCommand>,
) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            if tag.name == "verify"
                && let Some(cmd) = tag_attr(tag, "command")
                && !cmd.trim().is_empty()
            {
                let description = tag_attr(tag, "description").filter(|s| !s.trim().is_empty());
                commands.push(ExtractedVerifyCommand {
                    command: cmd,
                    description,
                    source_file: source_file.to_string(),
                    source_line: tag.source_line,
                });
            }
            // Recurse into children
            extract_verify_from_nodes(&tag.children, source_file, commands);
        }
    }
}

// ---------------------------------------------------------------------------
// Pipeline CLI helpers (shared between hick and hick-agent binaries)
// ---------------------------------------------------------------------------

/// Expand a CLI argument into a list of `.hick` files.
///
/// - If `path` is a file, return it directly.
/// - If `path` is a directory with `_hick.yml`, use that config to resolve files.
/// - If `path` is a directory without config, expand `**/*.hick`.
pub fn expand_path_arg(path: &Path) -> Result<(Vec<PathBuf>, Option<PathBuf>)> {
    use config::HickConfig;
    use anyhow::{bail, Context as _};
    if path.is_file() {
        let parent_config = path
            .parent()
            .map(|p| p.join("_hick.yml"))
            .filter(|p| p.is_file());
        return Ok((vec![path.to_path_buf()], parent_config));
    }
    if path.is_dir() {
        let config_path = path.join("_hick.yml");
        if config_path.is_file() {
            let config = HickConfig::load(&config_path)?;
            let files = config.resolve_files(path)?;
            return Ok((files, Some(config_path)));
        }
        let pattern = path.join("**/*.hick");
        let pattern_str = pattern.to_string_lossy();
        let mut matches: Vec<PathBuf> = glob::glob(&pattern_str)
            .with_context(|| format!("invalid glob pattern: {}", pattern_str))?
            .filter_map(|r| r.ok())
            .collect();
        matches.sort();
        if matches.is_empty() {
            bail!(
                "No .hick files found in '{}'\n\n\
                 To fix this, either:\n  \
                 1. Create a _hick.yml config file listing your .hick files\n  \
                 2. Add .hick files to the directory\n  \
                 3. Specify files directly: hick run file1.hick file2.hick",
                path.display()
            );
        }
        return Ok((matches, None));
    }
    bail!(
        "Path '{}' does not exist\n\n\
         Check that the path is correct and try again.\n\
         Use 'hick --help' to see usage information.",
        path.display()
    );
}

/// Parse a `key=value` string into a `(String, String)` pair (for clap `value_parser`).
pub fn parse_param(s: &str) -> std::result::Result<(String, String), String> {
    let (key, value) = s
        .split_once('=')
        .ok_or_else(|| format!("invalid param format '{s}', expected key=value"))?;
    Ok((key.to_string(), value.to_string()))
}

/// Configuration for `run_pipeline_cmd`.
pub struct PipelineRunOpts {
    pub files: Vec<PathBuf>,
    pub config_path: Option<PathBuf>,
    pub key_file: Option<PathBuf>,
    pub secrets_dir: Option<PathBuf>,
    pub images_dir: Option<PathBuf>,
    pub params: Vec<(String, String)>,
    pub features: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub dry_run: bool,
    pub cache: bool,
    pub freeze: bool,
    pub clear_cache: bool,
    pub verbose: bool,
}

/// Execute the pipeline run command.
///
/// This is the core logic shared between the `hick run` subcommand in
/// both the `hick` binary and the unified `hick` binary produced by the
/// `hick-agent` package.
pub async fn run_pipeline_cmd(opts: PipelineRunOpts) -> Result<()> {
    use std::time::Instant;
    use anyhow::{bail, Context as _};
    use log::{debug, info};
    use config::{HickConfig, find_config};
    use cache::CacheConfig;

    let pipeline_start = Instant::now();

    let (expanded_files, dir_config) = if !opts.files.is_empty() {
        let mut all_files = Vec::new();
        let mut found_config = None;
        for path in &opts.files {
            let (files, cfg) = expand_path_arg(path)?;
            all_files.extend(files);
            if cfg.is_some() {
                found_config = cfg;
            }
        }
        (all_files, found_config)
    } else {
        (Vec::new(), None)
    };

    let config_path = opts
        .config_path
        .or(dir_config)
        .or_else(|| find_config(&std::env::current_dir().unwrap_or_default()));

    let config = if let Some(ref path) = config_path {
        info!("Using config: {}", path.display());
        HickConfig::load(path)?
    } else {
        HickConfig::default()
    };

    let config_dir = config_path
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or(Path::new("."));

    let files = if !expanded_files.is_empty() {
        expanded_files
    } else {
        config.resolve_files(config_dir)?
    };

    if files.is_empty() {
        bail!(
            "No .hick files specified\n\n\
             Usage:\n  \
             hick run <file.hick>         Run a single file\n  \
             hick run <directory>         Run all .hick files in directory\n  \
             hick run                     Use _hick.yml config if present\n\n\
             Create a _hick.yml file to define default files and variables."
        );
    }

    let mut params: Vec<(String, String)> = config
        .vars
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    for (k, v) in &opts.params {
        params.retain(|(pk, _)| pk != k);
        params.push((k.clone(), v.clone()));
    }
    if let Some(ref features) = opts.features {
        params.retain(|(pk, _)| pk != "features");
        params.push(("features".to_string(), features.clone()));
    }

    let key_path = opts
        .key_file
        .or_else(|| config.secrets.key_file.as_ref().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("~/.config/hick/key.txt"));
    let key_path = expand_tilde(&key_path);

    let secrets_dir = opts
        .secrets_dir
        .or_else(|| config.secrets.secrets_dir.as_ref().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("~/.config/hick/secrets"));
    let secrets_dir = expand_tilde(&secrets_dir);

    let output_dir = opts
        .output_dir
        .or_else(|| config.output_dir.as_ref().map(|d| config_dir.join(d)))
        .unwrap_or_else(|| PathBuf::from("."));

    let _secrets_provider = hick_secrets::AgeSecretsProvider::new(&key_path, &secrets_dir);

    // Check if it's a session file (single-file replay)
    if files.len() == 1 {
        let path = &files[0];
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if hick_lang::is_session_source(&source) {
            let images_dir = opts
                .images_dir
                .or_else(|| {
                    config
                        .defaults
                        .images_dir
                        .as_ref()
                        .map(|d| expand_tilde(Path::new(d)))
                })
                .unwrap_or_else(|| PathBuf::from("."));
            return crate::pipeline_session_replay(path, &source, images_dir, opts.verbose).await;
        }
    }

    let mut file_contents = Vec::new();
    for path in &files {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        file_contents.push((path.display().to_string(), source));
    }

    let sources: Vec<(&str, &str)> = file_contents
        .iter()
        .map(|(name, content)| (name.as_str(), content.as_str()))
        .collect();

    let cache_enabled = opts.cache || opts.freeze;
    let cc = if cache_enabled {
        let cc = CacheConfig::new(config_dir, true, opts.freeze);
        if opts.clear_cache {
            cache::cache_clear(&cc)?;
            info!("Cache cleared");
        }
        Some(cc)
    } else if opts.clear_cache {
        let cc = CacheConfig::new(config_dir, false, false);
        cache::cache_clear(&cc)?;
        info!("Cache cleared");
        None
    } else {
        None
    };

    let result = if opts.dry_run {
        run_pipeline(&sources, &params).await?
    } else {
        let images_dir = opts
            .images_dir
            .or_else(|| {
                config
                    .defaults
                    .images_dir
                    .as_ref()
                    .map(|d| expand_tilde(Path::new(d)))
            })
            .unwrap_or_else(|| PathBuf::from("."));

        let pipeline_config = PipelineConfig {
            images_dir,
            toolchain_dir: None,
            working_dir: None,
            max_rounds: 1,
        };
        run_pipeline_live(&sources, &pipeline_config, &params, cc.as_ref(), None).await?
    };

    let mut expected_paths: std::collections::HashSet<PathBuf> =
        std::collections::HashSet::new();
    let mut files_written = 0usize;
    let mut files_unchanged = 0usize;

    for (path, content) in &result.files {
        let full_path = if output_dir == Path::new(".") {
            PathBuf::from(path)
        } else {
            expected_paths.insert(PathBuf::from(path));
            output_dir.join(path)
        };

        let needs_write = match content {
            hick_exec::node::FileContent::Text(s) => {
                if full_path.is_file() {
                    match std::fs::read_to_string(&full_path) {
                        Ok(existing) => existing != *s,
                        Err(_) => true,
                    }
                } else {
                    true
                }
            }
            hick_exec::node::FileContent::Binary(data) => {
                if full_path.is_file() {
                    match std::fs::read(&full_path) {
                        Ok(existing) => existing != data.to_bytes().unwrap_or_default(),
                        Err(_) => true,
                    }
                } else {
                    true
                }
            }
        };

        if needs_write {
            info!("Writing output: {}", full_path.display());
            if let Some(parent) = full_path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)?;
            }
            match content {
                hick_exec::node::FileContent::Text(s) => std::fs::write(&full_path, s)?,
                hick_exec::node::FileContent::Binary(data) => {
                    std::fs::write(&full_path, data.to_bytes()?)?;
                }
            }
            debug!("  {} bytes written", content.len());
            files_written += 1;
        } else {
            debug!("Unchanged: {}", full_path.display());
            files_unchanged += 1;
        }
    }

    if output_dir != Path::new(".") {
        output_cleanup::clean_stale_outputs(&output_dir, &expected_paths)?;
    }

    if result.files.is_empty() {
        info!("No file outputs produced");
    }

    let wall_time = pipeline_start.elapsed();
    let num_containers = result.resource_stats.len();
    let num_files = result.files.len();
    let total_output_bytes: usize = result.files.values().map(|c| c.len()).sum();

    let total_boot: Duration = result
        .resource_stats
        .values()
        .map(|s| s.boot_duration)
        .sum();
    let total_exec: Duration = result
        .resource_stats
        .values()
        .map(|s| s.total_exec_duration)
        .sum();
    let total_commands: usize = result
        .resource_stats
        .values()
        .map(|s| s.command_durations.len())
        .sum();

    let pool_info = if result.pool_hits > 0 {
        format!(" ({} from pool)", result.pool_hits)
    } else {
        String::new()
    };

    let pipeline_boot = total_boot.saturating_sub(result.pool_boot_time_saved);

    let file_stats = if files_unchanged > 0 {
        format!(
            "{} file{} ({} written, {} unchanged)",
            num_files,
            if num_files == 1 { "" } else { "s" },
            files_written,
            files_unchanged
        )
    } else {
        format!(
            "{} file{} written",
            files_written,
            if files_written == 1 { "" } else { "s" }
        )
    };

    eprintln!(
        "Done: {} ({} bytes), {} container{}{}, {} command{} — {} total (boot {}, exec {})",
        file_stats,
        total_output_bytes,
        num_containers,
        if num_containers == 1 { "" } else { "s" },
        pool_info,
        total_commands,
        if total_commands == 1 { "" } else { "s" },
        fmt_duration(wall_time),
        fmt_duration(pipeline_boot),
        fmt_duration(total_exec),
    );

    Ok(())
}

/// Replay a session .hick file without calling the LLM.
async fn pipeline_session_replay(
    file_path: &Path,
    source: &str,
    images_dir: PathBuf,
    verbose: bool,
) -> Result<()> {
    use anyhow::Context as _;
    use base64::Engine as _;
    use hick_token::ContainerCapabilities;
    use std::collections::HashMap;

    let session = hick_lang::parse_session(source)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("failed to parse session file {}", file_path.display()))?;

    eprintln!(
        "Replaying {} from {}",
        if session.nodes.is_empty() {
            "empty session".to_string()
        } else {
            format!("{} session nodes", session.nodes.len())
        },
        file_path.display()
    );

    let mut caps: HashMap<String, ContainerCapabilities> = HashMap::new();
    caps.insert(
        "replay".to_string(),
        ContainerCapabilities::new().deny_all_network(),
    );

    let mut executor = executor::ContainerExecutor::new(images_dir, caps)?;
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    executor.set_preopened_dirs(vec![(cwd, "/workspace".to_string())]);

    let mut started = false;
    let mut action_idx = 0usize;
    let mut command_idx = 0usize;

    for node in &session.nodes {
        match node {
            hick_lang::SessionNode::User { text } => {
                if verbose {
                    eprintln!("[user] {}", text.trim());
                }
            }
            hick_lang::SessionNode::Assistant { text, actions } => {
                if verbose && !text.trim().is_empty() {
                    eprintln!("[assistant] {}", text.trim());
                }
                for action in actions {
                    action_idx += 1;
                    if !started {
                        executor.ensure_started("replay", "alpine").await?;
                        started = true;
                    }
                    let interpreter = match action.lang.to_lowercase().as_str() {
                        "sh" | "shell" | "bash" => "sh",
                        "python" | "python3" | "py" => "python3",
                        "node" | "js" | "javascript" => "node",
                        _ => "sh",
                    };
                    let encoded = base64::engine::general_purpose::STANDARD
                        .encode(action.code.as_bytes());
                    let script_path = format!("/tmp/__hick_action_{action_idx}");
                    let cmd = format!(
                        "printf '%s' '{encoded}' | base64 -d > {script_path} && \
                         {interpreter} {script_path}"
                    );
                    println!("[action-{action_idx}] lang={}", action.lang);
                    if verbose {
                        for line in action.code.trim().lines() {
                            println!("  {line}");
                        }
                    }
                    match executor.execute("replay", &cmd).await {
                        Ok(output) => {
                            if !output.is_empty() {
                                println!("{output}");
                            }
                        }
                        Err(e) => eprintln!("[action-{action_idx}] error: {e}"),
                    }
                }
            }
            hick_lang::SessionNode::Observation {
                source,
                exit_code,
                text,
            } => {
                if verbose {
                    let src = source.as_deref().unwrap_or("-");
                    let exit = exit_code.map_or_else(|| "-".to_string(), |c| c.to_string());
                    eprintln!("[observation source={src} exit={exit}] {}", text.trim());
                }
            }
            hick_lang::SessionNode::Command { text } => {
                command_idx += 1;
                if !started {
                    executor.ensure_started("replay", "alpine").await?;
                    started = true;
                }
                println!("[command-{command_idx}] {}", text.trim());
                match executor.execute("replay", text.trim()).await {
                    Ok(output) => {
                        if !output.is_empty() {
                            println!("{output}");
                        }
                    }
                    Err(e) => eprintln!("[command-{command_idx}] error: {e}"),
                }
            }
        }
    }

    if started {
        executor.shutdown().await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_single_verify_command() {
        let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:verify command="cargo check" />
</hick:doc>"#;
        let sources = vec![("test.hick", source)];
        let cmds = extract_verify_commands(&sources).unwrap();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].command, "cargo check");
        assert!(cmds[0].description.is_none());
        assert_eq!(cmds[0].source_file, "test.hick");
    }

    #[test]
    fn extract_verify_with_condition_and_description() {
        let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:verify command="[backend] cargo test" description="Run backend tests" />
</hick:doc>"#;
        let sources = vec![("test.hick", source)];
        let cmds = extract_verify_commands(&sources).unwrap();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].command, "[backend] cargo test");
        assert_eq!(cmds[0].description.as_deref(), Some("Run backend tests"));
    }

    #[test]
    fn extract_multiple_verify_commands() {
        let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:verify command="cargo check" description="Compile check" />
<hick:verify command="[spa] npm test" description="SPA tests" />
<hick:verify command="[spa+backend] npm run e2e" description="E2E tests" />
</hick:doc>"#;
        let sources = vec![("test.hick", source)];
        let cmds = extract_verify_commands(&sources).unwrap();
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0].command, "cargo check");
        assert_eq!(cmds[1].command, "[spa] npm test");
        assert_eq!(cmds[2].command, "[spa+backend] npm run e2e");
    }

    #[test]
    fn extract_verify_from_multiple_sources() {
        let source1 = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:verify command="cargo check" />
</hick:doc>"#;
        let source2 = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:verify command="npm test" />
</hick:doc>"#;
        let sources = vec![("a.hick", source1), ("b.hick", source2)];
        let cmds = extract_verify_commands(&sources).unwrap();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].source_file, "a.hick");
        assert_eq!(cmds[1].source_file, "b.hick");
    }

    #[test]
    fn extract_verify_skips_empty_command() {
        let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:verify command="" />
<hick:verify command="  " />
<hick:verify command="cargo check" />
</hick:doc>"#;
        let sources = vec![("test.hick", source)];
        let cmds = extract_verify_commands(&sources).unwrap();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].command, "cargo check");
    }

    #[test]
    fn extract_verify_skips_missing_command_attr() {
        let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:verify description="No command attribute" />
<hick:verify command="cargo check" />
</hick:doc>"#;
        let sources = vec![("test.hick", source)];
        let cmds = extract_verify_commands(&sources).unwrap();
        assert_eq!(cmds.len(), 1);
    }

    #[test]
    fn extract_verify_nested_inside_other_elements() {
        let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:when test="backend">
<hick:verify command="cargo test" description="Nested verify" />
</hick:when>
</hick:doc>"#;
        let sources = vec![("test.hick", source)];
        let cmds = extract_verify_commands(&sources).unwrap();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].command, "cargo test");
        assert_eq!(cmds[0].description.as_deref(), Some("Nested verify"));
    }

    #[test]
    fn extract_verify_preserves_source_line() {
        let source = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">

<hick:feature name="backend" description="Backend" />

<hick:verify command="cargo check" />
</hick:doc>"#;
        let sources = vec![("test.hick", source)];
        let cmds = extract_verify_commands(&sources).unwrap();
        assert_eq!(cmds.len(), 1);
        // The verify tag is on line 5
        assert_eq!(cmds[0].source_line, 5);
    }
}
