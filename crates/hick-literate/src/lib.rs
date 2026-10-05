//! Hick pipeline library.
//!
//! Provides the core pipeline for processing `.hick` sources: parsing,
//! DAG validation, container capability extraction, copy/paste resolution,
//! and file output collection — all without touching the filesystem.

pub mod agent_cell;
pub mod cache;
pub mod capture;
pub mod cell_timeout;
pub mod compact;
pub mod config;
pub mod csv_table;
pub mod equiv;
pub mod expect;
mod links;
pub mod needs;
pub mod output_cleanup;
pub mod pipeline;
pub mod promote;
pub mod render;
pub mod scene;
pub mod session_elements;
pub mod store_config;
mod text;
pub mod volume_state;
pub mod watch;
mod weave;

/// Image used when neither the exec nor its container declares one.
const DEFAULT_IMAGE: &str = "alpine";

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context as _, Result};
use log::{debug, info, trace, warn};
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

pub use hickory_executor::{
    ContainerResourceStats, ExecOptions, ExecTranscriptEntry, Executor, LocalExecutor,
    TranscriptEvent, Transcripts,
};

use crate::expect::ExpectationOutcome;
use hick_condition::Condition;

// ---------------------------------------------------------------------------
// Cell identity and missing baselines
// ---------------------------------------------------------------------------

/// Identity of one cell in a document.
///
/// An `<hick:exec>` cell names the container it runs in. A cell with **no**
/// container — the agent cell described in
/// `docs/specs/freeform/agent-cells.md` — leaves `container` `None`. That is
/// why this is a struct rather than the `(container, source_line)` tuple it
/// replaces: a key that assumes a container cannot name a cell that has none.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellId {
    /// The container this cell executes in, when it has one.
    pub container: Option<String>,
    /// Line of the cell's opening tag in its source document.
    pub source_line: usize,
}

impl CellId {
    /// A cell that runs inside a container (`<hick:exec container="…">`).
    pub fn exec(container: impl Into<String>, source_line: usize) -> Self {
        Self {
            container: Some(container.into()),
            source_line,
        }
    }

    /// A cell with no container of its own.
    pub fn containerless(source_line: usize) -> Self {
        Self {
            container: None,
            source_line,
        }
    }

    /// The container name, when this cell has one.
    pub fn container(&self) -> Option<&str> {
        self.container.as_deref()
    }
}

impl std::fmt::Display for CellId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.container {
            Some(c) => write!(f, "line {} (container '{c}')", self.source_line),
            None => write!(f, "line {}", self.source_line),
        }
    }
}

/// Why a cell has no baseline to be verified against.
///
/// This is the *unverifiable* half of `check`'s verdict: nothing was ever
/// established for the cell, which is a different fact from "what was
/// established has since changed" (drift). See
/// `docs/guarantees/verification/test-separates-unverifiable-from-drifted.md`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NoBaseline {
    /// Weave or dry-run: nothing is executed, and no recording answered this
    /// cell.
    NotExecuted,
    /// The cell is frozen and no recording exists for it, in a mode that may
    /// not establish one (`hick test`). `frozen_by_cell` distinguishes
    /// `freeze="true"` on the cell itself from a run-wide freeze the cell
    /// merely inherited.
    FrozenWithoutRecording {
        /// First non-empty command line, so a report can name the cell.
        command: String,
        frozen_by_cell: bool,
    },
    /// The cell declares `freeze="true"`, but this run has no cache directory
    /// at all — there is nowhere for a recording to live.
    FrozenWithoutCacheDirectory { command: String },
    /// The cell is a `<hick:agent>` cell, no recording answered it, and this
    /// run has no agent runner configured — no model credentials, so nothing
    /// could have run it.
    ///
    /// This is the ordinary state of CI and of any machine without an API key,
    /// and it is deliberately *unverifiable* rather than fatal: the rest of
    /// the document still runs, and the one cell nothing was established for
    /// is named. `model_declared` records whether the cell says which model it
    /// wants, because a cell that does can be replayed from a recording with
    /// no runner at all, and a cell that does not never can.
    AgentWithoutRunner {
        prompt: String,
        model_declared: bool,
    },
}

/// Cells with no baseline, keyed by cell.
pub type NeverRun = std::collections::BTreeMap<CellId, NoBaseline>;

/// The [`CellId`] of a DAG vertex.
///
/// An agent cell is containerless — that is what `CellId::container` being an
/// `Option` is for — even though `ExecInfo` gives it a reserved synthetic
/// container name so the container-keyed plumbing (recording directory,
/// transcript map) stays addressable.
fn cell_id_of(info: &dag::ExecInfo) -> CellId {
    if info.is_agent() {
        CellId::containerless(info.source_line)
    } else {
        CellId::exec(&info.container, info.source_line)
    }
}

/// The first non-empty line of a command, for naming a cell in a report.
fn first_command_line(command: &str) -> String {
    command
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("?")
        .to_string()
}

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
    /// Per-container resource stats (boot time, exec time).
    /// Empty for dry-run pipelines.
    pub resource_stats: HashMap<String, ContainerResourceStats>,
    /// Container name -> ordered transcript entries (with timed events and
    /// `source_line` provenance pointing at the producing exec tag).
    pub transcripts: Transcripts,
    /// Expectation (`<hick:expect>`) outcomes, in execution order. Recorded
    /// but non-fatal on `run`; `check` turns failures into a non-zero exit.
    pub expectations: Vec<ExpectationOutcome>,
    /// Cells with no baseline, and why: they neither executed nor were
    /// answered from a recording. Populated by dry-run and
    /// weave-without-cache modes always, and by the live pipeline when
    /// [`PipelineConfig::collect_unverifiable`] is set.
    pub never_run: NeverRun,
    /// Cells answered from a recording whose key no longer matches their
    /// inputs — recorded, but *stale*. The weave shows the last recorded
    /// output rather than a marker; a run brings it forward; `hick test`
    /// reports them apart from the unrecorded. Axis 1 of
    /// `docs/specs/freeform/three-axes.md`.
    pub stale: std::collections::BTreeMap<CellId, String>,
    /// The cache key each cell was looked up (or recorded) under.
    pub keys: std::collections::BTreeMap<CellId, String>,
    /// Cells that executed and whose document keeps a recording of them, so
    /// the document's copy is now behind and should be brought forward.
    pub refreshed: Vec<RefreshedRecording>,
    /// Cells answered from a recording the document keeps, rather than from
    /// the cache. The ingest's gate reads this to prove a recording it just
    /// wrote is the one the weave uses.
    pub from_document: std::collections::BTreeSet<CellId>,
    /// What each OUTPUT VOLUME produced on this run, by volume name.
    ///
    /// Separate from [`PipelineResult::files`] for two reasons, and both are
    /// bugs that existed while it was merged in:
    ///
    /// - **A volume a document has ingested must not be flushed over the
    ///   document's own bytes**, or the next run overwrites the edits the
    ///   ingest exists to protect. Those volumes appear here and NOT in
    ///   `files`.
    /// - **`hick ingest` has to know which files came from the volume.**
    ///   Reading them out of `files` by path prefix cannot work: a volume
    ///   declared `output="."` shares its prefix with every other output the
    ///   document produces, so an ingest would swallow files belonging to the
    ///   document's own `hick:file` blocks.
    ///
    /// Keys are the output paths as `files` would name them — prefixed by the
    /// volume's `output=` — because that is where the bytes actually land, and
    /// an ingest must not move them.
    pub volume_outputs: HashMap<String, HashMap<String, FileContent>>,
    /// Canonical paths of every file spliced into the pipeline's documents
    /// by `<hick:include>`/`<hick:upstream>` (the union of the resolved
    /// documents' [`hick_lang::HickDocument::span_files`]). Provenance can
    /// name these as an edit's destination, so a caller that guards against
    /// stale documents needs the full list.
    pub span_files: Vec<String>,
}

impl PipelineResult {
    /// Output paths that carry bytes from a cell with no baseline.
    ///
    /// A weave never executes: a cell whose recording it cannot find is woven
    /// as `[never run]`, and every file that cell fed then *says* the cell
    /// never ran. As a statement about this weave that is honest; as bytes
    /// written over a committed artifact it is destruction — the SVG a real
    /// run produced is replaced by a marker, and the app, which executes,
    /// goes on rendering the chart. Disk and app disagree, silently.
    ///
    /// So the writer needs to know which files those are, and provenance
    /// already knows: a span whose origin is the exec cell named in
    /// [`PipelineResult::never_run`] is exactly a byte the cell contributed.
    ///
    /// **Agent cells are not covered.** `SourceOrigin::Agent` carries the
    /// session and turn rather than a source line, so an agent cell's bytes
    /// cannot be matched back to its [`CellId`]. Agent cells write into
    /// documents rather than into `hick:file` products, so this has no
    /// bearing on the artifact case; it is stated because the omission is
    /// deliberate rather than overlooked.
    pub fn outputs_missing_a_recording(&self) -> std::collections::BTreeSet<String> {
        if self.never_run.is_empty() {
            return std::collections::BTreeSet::new();
        }
        let mut affected = std::collections::BTreeSet::new();
        for (path, map) in &self.provenance_maps {
            let missing = map.spans().iter().any(|span| match &span.origin {
                hick_exec::node::SourceOrigin::Exec {
                    container,
                    tag_line,
                } => self
                    .never_run
                    .contains_key(&CellId::exec(container.as_ref(), *tag_line)),
                hick_exec::node::SourceOrigin::Script { tag_line } => self
                    .never_run
                    .contains_key(&CellId::containerless(*tag_line)),
                _ => false,
            });
            if missing {
                affected.insert(path.clone());
            }
        }
        affected
    }
}

// ---------------------------------------------------------------------------
// Pipeline setup (shared between dry-run and live)
// ---------------------------------------------------------------------------

/// Intermediate state after parsing, filtering, and collecting definitions.
struct PreparedPipeline<'a> {
    documents: Vec<(&'a str, HickDocument)>,
    state: Arc<MultiDocumentState>,
    container_defs: HashMap<String, ContainerCapabilities>,
    /// Image each container declares, so the executor is told which
    /// environment to provide. Capabilities and images are collected from the
    /// same tag but were not carried together.
    container_images: HashMap<String, String>,
    /// What each container's cells expect to find installed
    /// (`<hick:needs bin="…" />`), checked before anything runs.
    container_needs: std::collections::BTreeMap<String, Vec<crate::needs::Need>>,
    /// Fork registrations: (from, to, additional_caps).
    /// Needed by live pipeline to register forks with the executor.
    fork_registrations: Vec<(String, String, Option<ContainerCapabilities>)>,
    /// Volume declarations across every document, by volume name. Collected
    /// here rather than in the live pipeline because their `<hick:allow>`
    /// children are part of a container's capabilities, and capabilities are
    /// minted into tokens here.
    volumes: HashMap<String, hick_exec::volume::VolumeDeclaration>,
}

/// Whether a document keeps its fragments to itself.
///
/// `<hick:private />` is the opt-out from ambient contribution. The default
/// is the other way round — a fragment is offered to the folder — because the
/// case this exists for is many documents feeding one shared file, and making
/// each of them announce itself puts the coupling straight back.
fn declares_private(nodes: &[HickNode]) -> bool {
    nodes
        .iter()
        .any(|node| matches!(node, HickNode::Tag(tag) if tag.name == "private"))
}

/// Append a `hick:upstream` edge for every other `.hick` beside this one.
///
/// Directory-scoped and not recursive: a folder is the unit a person can hold
/// in their head, and walking a whole repository would let a fragment in some
/// unrelated corner change this document's output.
///
/// A sibling that cannot be read or does not PARSE is skipped rather than
/// failing this run: that is the other document's error, reported when it is
/// run, and it must not stop an unrelated document from weaving.
fn attach_ambient_contributors(
    doc: &mut HickDocument,
    name: &str,
    base_dir: &std::path::Path,
) -> std::io::Result<()> {
    // Every file already spliced into this document, by include or by
    // upstream, at any depth. `span_files` is the right record because both
    // branches stamp it; the resolver's `seen` set is a cycle-detection
    // STACK and is popped on the way back out, so by the time it returns it
    // holds nothing but the document itself.
    let already_merged: std::collections::HashSet<std::path::PathBuf> = doc
        .span_files
        .iter()
        .filter_map(|f| std::fs::canonicalize(f).ok())
        .collect();
    // A document named without a directory has `""` as its parent, and
    // `read_dir("")` is an error rather than the current directory.
    let dir = if base_dir.as_os_str().is_empty() {
        std::path::Path::new(".")
    } else {
        base_dir
    };
    let self_path = std::fs::canonicalize(name).ok();
    let mut siblings: Vec<std::path::PathBuf> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let canonical = std::fs::canonicalize(&path).ok();
        if canonical == self_path {
            continue;
        }
        // Already reachable through an `include` or `upstream` the author
        // wrote — possibly several hops away, which is why this asks the
        // resolver's own record rather than re-reading the tags. Splicing it
        // a second time is a duplicate-id error, not a no-op.
        if let Some(canonical) = &canonical
            && already_merged.contains(canonical)
        {
            continue;
        }
        siblings.push(path);
    }
    // Directory read order is not stable across filesystems, and a `.class`
    // paste concatenates in document order — so without this the same folder
    // could weave two different files on two machines.
    siblings.sort();

    for path in siblings {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(sibling) = hick_lang::parse(&source) else {
            continue;
        };
        if declares_private(&sibling.nodes) {
            debug!("{}: private, not contributing to {name}", path.display());
            continue;
        }
        if let Err(e) = hick_lang::attach_contribution(doc, &path, &source, 0) {
            debug!("{}: not contributing to {name}: {e}", path.display());
        }
    }
    Ok(())
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
        doc.weave_path = None;

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

        // Every other document in this folder contributes its fragments,
        // without this one naming any of them. That is the point: a document
        // that owns a shared file should not have to be edited every time
        // somebody has a line to add to it.
        //
        // Synthesised as `hick:upstream` edges rather than as a second
        // mechanism, so an ambient contributor gets exactly what a declared
        // one already gets — fragments selectable, nothing of the contributor
        // rendered, and spans stamped to the CONTRIBUTOR's file so lineage
        // and the reverse edit land there rather than here.
        attach_ambient_contributors(&mut doc, name, base_dir)
            .map_err(|e| anyhow::anyhow!("reading contributors beside {name}: {e}"))?;

        // Derive speaker turns from every `hick:transcript`, AFTER includes so
        // that a transcript spliced in from another file is derived too. The
        // file on disk keeps only the raw block; this is the projection over it
        // (`docs/specs/freeform/ingest.md`).
        hick_transcript::expand(&mut doc);

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

    // Literal copy blocks, registered before anything executes, so a
    // `<hick:paste>` used as a cell's stdin can resolve one. Without this the
    // copy handler registers them during the render pass — which runs AFTER
    // every cell — and a stdin paste silently produced an empty string.
    for (name, doc) in &documents {
        register_literal_copies(&doc.nodes, &state, name, &doc.span_files);
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
    let mut volumes: HashMap<String, hick_exec::volume::VolumeDeclaration> = HashMap::new();
    for (name, doc) in &documents {
        let dag_result = hick_exec::dag::build_dag(doc)
            .map_err(|e| anyhow::anyhow!("DAG validation failed in {name}: {e}"))?;
        info!(
            "DAG validated for {name}: {} exec nodes",
            dag_result.execs.len()
        );
        for (vol_name, vol_decl) in &dag_result.volumes {
            volumes.insert(vol_name.clone(), vol_decl.clone());
        }
    }

    // Collect all nodes across documents (after filtering)
    let all_nodes: Vec<&HickNode> = documents
        .iter()
        .flat_map(|(_, doc)| doc.nodes.iter())
        .collect();

    // Collect container definitions
    let mut container_defs: HashMap<String, ContainerCapabilities> = HashMap::new();
    // ...and the image each container declares. This was collected nowhere
    // for a long time: `<hick:container image="python:3.12">` was parsed for
    // capabilities and its image dropped, so the executor only ever saw an
    // image when one appeared on the `<hick:exec>` tag itself. `LocalExecutor`
    // ignores images, so nothing surfaced it until a backend honoured them.
    let mut container_images: HashMap<String, String> = HashMap::new();
    // What each container's cells expect to find installed. Checked before
    // anything runs — see `needs`.
    let mut container_needs: std::collections::BTreeMap<String, Vec<crate::needs::Need>> =
        std::collections::BTreeMap::new();
    for node in &all_nodes {
        if let HickNode::Tag(tag) = node
            && tag.name == "container"
        {
            let name = tag_attr(tag, "name").unwrap_or_default();
            let caps = build_capabilities_from_tag(tag);
            debug!("Container '{name}': {:?}", caps);
            if let Some(image) = tag_attr(tag, "image") {
                container_images.insert(name.clone(), image);
            }
            let needs = crate::needs::needs_of(tag);
            if !needs.is_empty() {
                // Extended rather than replaced: one container may be
                // declared in several documents of a pipeline, and each
                // declaration's needs are all real.
                container_needs
                    .entry(name.clone())
                    .or_default()
                    .extend(needs);
            }
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

            if let Some(image) = container_images.get(&from).cloned() {
                container_images.entry(to.clone()).or_insert(image);
            }
            fork_registrations.push((from, to, additional_caps));
        }
    }

    // Carry each volume's `<hick:allow>` rules into the capabilities of the
    // container they name. The rules used to exist only inside the volume
    // declaration, where the DAG builder read them to order execs — so a
    // container's token said nothing about the data it was about to be
    // handed. A container named by a rule but never `<hick:container>`-
    // declared still gets an entry: it is a real container, and an empty
    // capability set is the correct (deny-everything-else) description of it.
    for decl in volumes.values() {
        for rule in &decl.access_rules {
            let caps = container_defs.entry(rule.container.clone()).or_default();
            let vol_rule = match &rule.access {
                hick_exec::volume::VolumeAccess::Read(pattern) => {
                    hick_token::VolumeRule::read(&decl.name, pattern)
                }
                hick_exec::volume::VolumeAccess::Write(pattern) => {
                    hick_token::VolumeRule::write(&decl.name, pattern)
                }
            };
            if !caps.volume_rules.contains(&vol_rule) {
                caps.volume_rules.push(vol_rule);
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

    Ok(PreparedPipeline {
        documents,
        state,
        container_defs,
        container_images,
        container_needs,
        fork_registrations,
        volumes,
    })
}

/// Create the tag registry with all built-in handlers.
fn create_tag_registry() -> TagRegistry {
    let mut registry = TagRegistry::new();
    hick_handlers::handlers::register_builtins(&mut registry);
    // NOTE: hick-live's reactive handlers were part of the removed wasm
    // runtime stack and are not registered; `<hick:live>`-family tags are
    // ignored. See docs/developers/vendoring-notes.md.
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
                    source_line: e.source_line,
                })
                .collect();
            (k.clone(), converted)
        })
        .collect()
}

/// Shared epilogue: register copy/cut/substitute/exclude blocks, process
/// file outputs, weave output, and converge files with exclusion filtering.
/// Refuse the run if any `<hick:paste min=/max=>` gate went unmet.
///
/// A cardinality gate exists precisely to catch a caret that collected
/// nothing. Reporting it as a warning and weaving the empty file anyway made
/// `hick run` and `hick test` both exit 0 on a document that had asked to be
/// told — the same silence `resolve_paste_node` was fixed for, one layer up.
///
/// Checked against a RUN, never against a weave. A fragment a cell writes
/// does not exist until that cell has run, so enforcing this in `hick weave`
/// or `hick lineage` — which execute nothing on purpose — would report every
/// such document as broken. That is the same reason a cell with no
/// transcript weaves as `[never run]` instead of failing.
fn refuse_on_paste_failures(state: &MultiDocumentState) -> Result<()> {
    let failures = state.paste_failures();
    if failures.is_empty() {
        return Ok(());
    }
    anyhow::bail!("{}", failures.join("\n"));
}

async fn process_pipeline_outputs(
    documents: &[(&str, HickDocument)],
    state: &Arc<MultiDocumentState>,
    transcripts: &HashMap<String, Vec<ExecTranscriptEntry>>,
    max_rounds: usize,
) -> (HashMap<String, FileContent>, HashMap<String, ProvenanceMap>) {
    let registry = create_tag_registry();
    let handler_transcripts = convert_transcripts(transcripts);

    // --- Round 0: process the original documents (identical to previous behavior) ---
    state.clear_paste_failures();
    process_documents_round(documents, state, &handler_transcripts, &registry);

    // Process weave output if enabled
    weave::process_weave_output(documents, &handler_transcripts, state, &registry);

    // --- Rounds 1..max_rounds: re-evaluate .hick files produced by containers ---
    if max_rounds > 1 {
        let mut prev_hick_paths: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for round in 1..max_rounds {
            let files = state.get_files().await;

            // Find new .hick files in output — from `<hick:file>` blocks and
            // from output VOLUMES both, since a generator writes into a
            // volume and those bytes are merged into `files` only after this
            // whole function has returned.
            let mut hick_sources: Vec<(String, String)> = files
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
            hick_sources.extend(state.produced_hick_files());
            hick_sources.sort();
            hick_sources.dedup();

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

            // Then the ORIGINAL documents again, so a `<hick:paste>` that was
            // written before the generator ran can now see what it emitted.
            // Without this a cell could write fragments nothing was able to
            // read, which is the whole point of letting it write them.
            //
            // Safe to repeat only because registration is keyed by where a
            // block was declared (`ContentBlock::origin_key`) and a file
            // output is stored by path — so a second pass replaces what the
            // first produced instead of appending to it.
            state.clear_paste_failures();
            process_documents_round(documents, state, &handler_transcripts, &registry);
            weave::process_weave_output(documents, &handler_transcripts, state, &registry);
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
/// Run the declaration-phase handlers over top-level tags — and over the
/// fragments a `hick:upstream` edge brought in, which sit under that node
/// rather than at the top level so the weave can skip them as a unit.
fn declare_nodes(nodes: &[HickNode], registry: &TagRegistry, ctx: &ProcessingContext) {
    for node in nodes {
        let HickNode::Tag(tag) = node else { continue };
        // A pipeline edge holds fragments; a claim wraps what it asserts. Both
        // are containers whose children declare as if they stood at the top.
        if tag.name == "upstream" || tag.name == "claim" {
            declare_nodes(&tag.children, registry, ctx);
            continue;
        }
        // A cell with an id is quotable: what it SHOWS (its transcript,
        // rendered per `show=`) is registered under `#id`/`.class`, so a
        // finding can paste the number a cell printed instead of retyping it
        // — and the paste carries the cell's exec provenance, so the ribbon
        // ends at the computation rather than one hop short of it.
        if tag.name == "exec"
            && let Some(id) = tag_attr(tag, "id")
            && let Some((node, text)) = hick_handlers::handlers::quotable_exec_node(tag, ctx)
        {
            ctx.state
                .register_copy_node(id, tag_attr(tag, "class").as_deref(), node, text);
            continue;
        }
        if let Some(handler) = registry.find(&tag.name)
            && handler.phase() == ProcessingPhase::Declaration
        {
            let _ = handler.process(tag, ctx);
        }
    }
}

fn process_documents_round(
    documents: &[(&str, HickDocument)],
    state: &Arc<MultiDocumentState>,
    handler_transcripts: &HashMap<String, Vec<TranscriptEntry>>,
    registry: &TagRegistry,
) {
    // Declaration phase: process copy/cut/substitute/exclude via handlers.
    // Each document's declarations carry its name as `source_file` so copy
    // blocks get byte-precise Literal provenance.
    for (doc_name, doc) in documents {
        let span_files = span_file_table(doc);
        let decl_ctx = ProcessingContext {
            state,
            transcripts: handler_transcripts,
            indent: 0,
            paste_line_indent: None,
            registry: Some(registry),
            context: None,
            source_file: Some(Arc::from(*doc_name)),
            span_files: &span_files,
        };
        declare_nodes(&doc.nodes, registry, &decl_ctx);
    }

    // Process file outputs
    for (doc_name, doc) in documents {
        let source_file: Arc<str> = Arc::from(*doc_name);
        let span_files = span_file_table(doc);
        for node in &doc.nodes {
            // A `hick:table` with a `path` is a file output too: its CSV is
            // the dataset, and the woven markdown is how the document reads.
            // Without a `path` it is prose that happens to be tabular, and
            // writes nothing.
            if let HickNode::Tag(tag) = node
                && (tag.name == "file"
                    || (tag.name == "table"
                        && tag_attr(tag, "path").is_some_and(|p| !p.is_empty())))
            {
                add_file_output_for(
                    tag,
                    None,
                    handler_transcripts,
                    state,
                    registry,
                    &source_file,
                    &span_files,
                );
            }
        }

        // The files a scaffolder wrote and this document now owns. They sit
        // at `exec > ingested > file` — deliberately not at the top level,
        // because containment says "running this produced these" with no
        // string reference to resolve, and deliberately not `exec > file`,
        // because `file > exec` already means the opposite. See
        // `docs/specs/freeform/owning-what-a-scaffolder-wrote.md`.
        for (run, tag) in ingested_file_blocks(&doc.nodes) {
            add_file_output_for(
                tag,
                Some(&run),
                handler_transcripts,
                state,
                registry,
                &source_file,
                &span_files,
            );
        }
    }
}

/// The volumes whose bytes a document has ingested: every volume mounted by
/// an `<hick:exec>` that carries an `<hick:ingested>` child.
fn ingested_volume_names<'a>(
    docs: impl Iterator<Item = &'a HickDocument>,
) -> std::collections::HashSet<String> {
    fn walk(nodes: &[HickNode], out: &mut std::collections::HashSet<String>) {
        for node in nodes {
            let HickNode::Tag(tag) = node else { continue };
            if tag.name == "exec"
                && tag
                    .child_tags()
                    .any(|c| c.name == "ingested" && c.get_attribute("key").is_none())
            {
                for entry in tag_attr(tag, "mount").unwrap_or_default().split(',') {
                    if let Some((vol, _)) = entry.trim().split_once(':') {
                        out.insert(vol.to_string());
                    }
                }
            }
            walk(&tag.children, out);
        }
    }
    let mut out = std::collections::HashSet::new();
    for doc in docs {
        walk(&doc.nodes, &mut out);
    }
    out
}

/// The files ONE output (or input-output) volume's CURRENT in-memory state
/// would flush into pipeline result files, prefixed under its declared
/// `output=` path.
///
/// Shared between the per-cell incremental flush hook and the post-loop
/// consolidated pass so the unpack-and-prefix logic has exactly one
/// implementation. An input-only or ephemeral volume, or one not yet in the
/// store, flushes nothing — that is the honest answer for "declared but not
/// an output" and "declared but not yet written", not an error.
fn flushable_volume_files(
    vol_decl: &hick_exec::volume::VolumeDeclaration,
    tar: Option<&[u8]>,
    seeded: Option<&[u8]>,
) -> Result<HashMap<String, FileContent>> {
    let output_prefix = match &vol_decl.kind {
        hick_exec::volume::VolumeKind::Output { path } => Some(path.as_str()),
        hick_exec::volume::VolumeKind::InputOutput { output, .. } => Some(output.as_str()),
        _ => None,
    };
    let (Some(prefix), Some(tar)) = (output_prefix, tar) else {
        return Ok(HashMap::new());
    };
    // Bytes, not strings. A scaffolder writes binaries alongside source
    // (`dotnet new` alone leaves an `obj/` full of them), and reading every
    // entry as UTF-8 used to fail the whole run on the first one — an
    // obscure "failed to read tar entry" for something entirely normal. A
    // binary becomes `FileContent::Binary`, which is also what lets `hick
    // ingest` name it rather than mangle it.
    let unpacked = volume_state::read_tar_files(tar)?;
    // What the run changed, not what it read. A file identical to its seeded
    // bytes is an input that came along for the ride — the document itself,
    // when a volume is seeded from `.`, or a placeholder staged for a file a
    // cell fills — and flushing it back would write it over the cell's real
    // product, or over an edit made while the run was going.
    // docs/guarantees/execution/an-output-volume-flushes-only-what-the-run-changed.md
    let unchanged: HashMap<String, Vec<u8>> = match seeded {
        Some(seeded) => volume_state::read_tar_files(seeded)?.into_iter().collect(),
        None => HashMap::new(),
    };
    let mut out = HashMap::new();
    for (file_path, bytes) in unpacked {
        if unchanged
            .get(&file_path)
            .is_some_and(|before| *before == bytes)
        {
            continue;
        }
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
        let content = match String::from_utf8(bytes) {
            Ok(text) => FileContent::Text(text),
            Err(e) => FileContent::Binary(hick_exec::node::BinaryData::Inline(e.into_bytes())),
        };
        out.insert(output_path, content);
    }
    Ok(out)
}

/// Every `<hick:file>` inside a `<hick:ingested>` block, paired with the run
/// fingerprint the block records.
///
/// The fingerprint is `sha256=`, which is the recorded base a re-ingest
/// merges against — so a block without one yields nothing rather than
/// yielding files whose origin could not name where they came from.
fn ingested_file_blocks(nodes: &[HickNode]) -> Vec<(Arc<str>, &hick_lang::HickTag)> {
    fn walk<'a>(nodes: &'a [HickNode], out: &mut Vec<(Arc<str>, &'a hick_lang::HickTag)>) {
        for node in nodes {
            let HickNode::Tag(tag) = node else { continue };
            if tag.name == "ingested" {
                let Some(run) = tag_attr(tag, "sha256").filter(|v| !v.is_empty()) else {
                    warn!(
                        "<hick:ingested> at line {} has no sha256= — the run it                          records cannot be named, so its files are skipped",
                        tag.source_line
                    );
                    continue;
                };
                let run: Arc<str> = Arc::from(run.as_str());
                for child in tag.child_tags() {
                    if child.name == "file" {
                        out.push((run.clone(), child));
                    }
                }
                continue;
            }
            walk(&tag.children, out);
        }
    }
    let mut out = Vec::new();
    walk(nodes, &mut out);
    out
}

/// Register one `<hick:file>`/`<hick:table path>` tag as a pipeline file
/// output. `ingested` carries the run fingerprint when the block's bytes
/// came from a tool outside this document.
fn add_file_output_for(
    tag: &hick_lang::HickTag,
    ingested: Option<&Arc<str>>,
    handler_transcripts: &HashMap<String, Vec<TranscriptEntry>>,
    state: &Arc<MultiDocumentState>,
    registry: &TagRegistry,
    source_file: &Arc<str>,
    span_files: &[Arc<str>],
) {
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
        Some(source_file),
        span_files,
        ingested,
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

// ---------------------------------------------------------------------------
// Pipeline entry point
// ---------------------------------------------------------------------------

/// Seed input volumes from host directories.
///
/// `skip` names volumes an exec has already written to in this pass: an agent
/// cell's re-preparation re-seeds so the agent's edits are visible to later
/// cells, and re-seeding a volume a cell already wrote would silently throw
/// that write away.
fn seed_input_volumes(
    volume_store: &mut volume_state::VolumeStore,
    volumes: &HashMap<String, hick_exec::volume::VolumeDeclaration>,
    working_dir: &Path,
    skip: &[String],
) -> Result<()> {
    for (vol_name, vol_decl) in volumes {
        if skip.iter().any(|v| v == vol_name) {
            continue;
        }
        let path = match &vol_decl.kind {
            hick_exec::volume::VolumeKind::Input { path }
            | hick_exec::volume::VolumeKind::InputOutput { input: path, .. } => path,
            _ => continue,
        };
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
    Ok(())
}

/// Paths inside a volume that are this tool's own bookkeeping rather than
/// anything the document reads.
///
/// `.hick-cache/` has to be excluded or the key eats itself: an input volume
/// declared `input="."` seeds from the project directory, which *contains*
/// the recordings. Each run writes a recording, which changes the volume, which
/// changes the key, which writes a new recording under a new key — a cache
/// that never hits and grows a file per run. `.git/` is excluded for the same
/// reason with a slower fuse: committing anything would invalidate every cell.
fn is_run_artifact(path: &str) -> bool {
    // At ANY depth, not only the first component: a folder mounted as `.`
    // holds its subfolders' caches too, and a recording written under
    // `sub/.hick-cache/` by another document's run moved every key of a
    // cell mounting the parent.
    path.split(['/', '\\'])
        .any(|part| part == ".hick-cache" || part == ".git")
}

/// Digest the contents of every volume mounted into a cell.
///
/// This is the term that makes a cell's *inputs* part of its cache key. A
/// `<hick:file>` the document assembles is seeded into an input volume before
/// the run; a cell that mounts that volume therefore re-executes when the file
/// changes, without the command text having changed at all.
///
/// Mount paths are folded in as well as volume names: the same volume mounted
/// at a different path is a different view of the filesystem, and a command
/// addressing it relatively would behave differently.
///
/// A volume that is not in the store yet contributes its name and nothing
/// else. That is the honest digest of "declared but empty", and it differs
/// from the same volume once seeded, which is what matters.
///
/// `project_dir` is consulted against the SAME `.gitignore` filter
/// `seed_from_directory` already applies when an input volume is first
/// seeded from a host directory
/// (`docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md`).
/// Without it, a prior cell's own build output — `bin/`, `obj/`,
/// `__pycache__` — sitting in a volume this cell also mounts destabilizes
/// the digest for every cell sharing that mount, even though the same
/// `.gitignore` correctly keeps that output out of the host write. Paths are
/// checked as they appear WITHIN the volume (not reconstructed to their true
/// path relative to `project_dir`), which is an approximation — see the
/// caveat on the guarantee above — but it is exact for the unanchored
/// patterns (`bin/`, `obj/`, `__pycache__`) that are the actual, observed
/// failure mode.
/// A recording a document keeps of one of its cells: `<hick:ingested
/// key="…">` inside the `hick:exec`, holding the recorded output verbatim.
/// Axis 1 of docs/specs/freeform/three-axes.md — evidence the document
/// makes about itself, so it travels with the document rather than living
/// in a cache a clone does not have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocRecording {
    pub key: String,
    pub output: String,
}

/// What executed and has a recording in its document, so the document's
/// copy can be brought forward by whoever writes documents (`hick run`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshedRecording {
    pub cell: CellId,
    pub key: String,
    pub output: String,
}

/// Every recording the documents keep, by cell.
pub fn document_recordings<'a>(
    documents: impl Iterator<Item = &'a hick_lang::HickDocument>,
) -> HashMap<CellId, Vec<DocRecording>> {
    let mut out: HashMap<CellId, Vec<DocRecording>> = HashMap::new();
    for doc in documents {
        for exec in doc.all_tags().into_iter().filter(|t| t.name == "exec") {
            let Some(container) = exec.get_attribute("container") else {
                continue;
            };
            for child in exec.child_tags().filter(|c| c.name == "ingested") {
                let Some(key) = child.get_attribute("key") else {
                    continue;
                };
                out.entry(CellId::exec(container.trim(), exec.source_line))
                    .or_default()
                    .push(DocRecording {
                        key: key.trim().to_string(),
                        output: recording_body(&child.text_content()),
                    });
            }
        }
    }
    out
}

/// The recorded output as written: the writer puts one newline after the
/// opening tag so the output starts on its own line, and that newline is
/// not part of the output.
pub fn recording_body(text: &str) -> String {
    text.strip_prefix('\n').unwrap_or(text).to_string()
}

/// Every `hick:file` a cell fills, keyed by the cell: (container, source
/// line) → (the file's document-relative path, how its transcript renders).
///
/// A cell's product is what the weave puts in the file — `show="output"`
/// gives the bytes alone, the default gives the command line first — so the
/// same renderer decides both, and what a later cell reads from a volume is
/// exactly what the file will hold.
pub(crate) fn cell_filled_files<'a>(
    documents: impl Iterator<Item = &'a hick_lang::HickDocument>,
) -> HashMap<(String, usize), (String, hick_handlers::ExecShow)> {
    let mut out = HashMap::new();
    for doc in documents {
        for tag in doc.all_tags().into_iter().filter(|t| t.name == "file") {
            let Some(path) = tag.get_attribute("path") else {
                continue;
            };
            for exec in tag.child_tags().filter(|c| c.name == "exec") {
                let Some(container) = exec.get_attribute("container") else {
                    continue;
                };
                out.insert(
                    (container.trim().to_string(), exec.source_line),
                    (
                        path.trim().trim_start_matches("./").to_string(),
                        hick_handlers::parse_exec_show(exec),
                    ),
                );
            }
        }
    }
    out
}

/// Put a cell's product into every input volume that covers its path, so
/// the next cell that mounts that volume reads what this one just made.
///
/// This is the chain the documents are for — a program writes
/// `openapi.json`, a generator reads it and writes the client — and it has
/// to work inside ONE run. Without this, a cell-filled file reached the
/// volume only through the next run's seeding from disk, so the generator
/// read either nothing or the previous run's spec, and the run reported
/// nothing wrong. The volume's seeded record is left alone: the product is
/// something the run changed, and the flush must treat it as such.
/// The documents' literal `hick:file` products — bodies with no tag children —
/// as the documents hold them now.
///
/// Seeded into the volumes that cover them BEFORE any cell runs, and into
/// the seeded record the key is taken from. A volume seeded from disk holds
/// the previous run's copy of such a file; the document is the truth, and a
/// cell that reads the file must re-execute when the document changed it
/// (`a-recording-is-keyed-by-the-cells-inputs.md`). This used to hold by
/// accident, because the document itself was in the key; the document is
/// out of the key now (it carries its recordings), so the product goes in
/// on its own.
pub(crate) fn literal_products<'a>(
    documents: impl Iterator<Item = &'a hick_lang::HickDocument>,
) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for doc in documents {
        for tag in doc.all_tags().into_iter().filter(|t| t.name == "file") {
            let Some(path) = tag.get_attribute("path") else {
                continue;
            };
            if tag.child_tags().next().is_none() {
                out.push((
                    path.trim().trim_start_matches("./").to_string(),
                    tag.text_content().into_bytes(),
                ));
            }
        }
    }
    out
}

fn inject_products_into_volumes(
    volume_store: &mut volume_state::VolumeStore,
    volumes: &HashMap<String, hick_exec::volume::VolumeDeclaration>,
    products: &[(String, Vec<u8>)],
) -> Result<()> {
    inject_products(volume_store, volumes, products, false)
}

/// `into_seed`: also make the products part of what the volume was SEEDED
/// with, so they are in every cell's key.
fn inject_products(
    volume_store: &mut volume_state::VolumeStore,
    volumes: &HashMap<String, hick_exec::volume::VolumeDeclaration>,
    products: &[(String, Vec<u8>)],
    into_seed: bool,
) -> Result<()> {
    for (vol_name, vol_decl) in volumes {
        let input = match &vol_decl.kind {
            hick_exec::volume::VolumeKind::Input { path }
            | hick_exec::volume::VolumeKind::InputOutput { input: path, .. } => path,
            _ => continue,
        };
        let input = input.trim().trim_start_matches("./").trim_end_matches('/');
        let mut entries: Vec<(String, Vec<u8>)> = match volume_store.get(vol_name) {
            Some(tar) => volume_state::read_tar_files(tar)?,
            None => Vec::new(),
        };
        let mut changed = false;
        for (rel_path, bytes) in products {
            let inside = if input.is_empty() || input == "." {
                Some(rel_path.as_str())
            } else {
                rel_path
                    .strip_prefix(input)
                    .and_then(|r| r.strip_prefix('/'))
            };
            let Some(inside) = inside else {
                continue;
            };
            match entries.iter_mut().find(|(path, _)| path == inside) {
                Some((_, existing)) => *existing = bytes.clone(),
                None => entries.push((inside.to_string(), bytes.clone())),
            }
            changed = true;
        }
        if changed {
            let tar = volume_state::pack_tar_files(&entries)?;
            if into_seed {
                volume_store.seed_tar(vol_name, tar);
            } else {
                volume_store.update(vol_name, tar);
            }
        }
    }
    Ok(())
}

/// The files a pipeline's documents produce whose bytes change from run to
/// run: every weave target, and every `hick:file` a cell fills.
///
/// These are kept OUT of a cell's input digest. A cell that mounts the
/// folder it lives in mounts its own weave, which carries its own transcript;
/// keying the recording on that made the key change on every run, so no
/// recording of such a cell was ever findable again, and the next weave wrote
/// `[never run]` over the output of a run that really happened. That used to
/// be a warning telling the author to mount something narrower. It is now
/// simply not in the key: a document's own unstable products are not inputs
/// to the run that produces them, whatever directory a cell mounts.
///
/// Only the UNSTABLE products. A `hick:file` assembled from literal text is
/// the central move of literate programming — a cell running a script its
/// own document wrote — and its bytes are exactly the input the author means
/// them to be (`a-recording-is-keyed-by-the-cells-inputs.md`).
pub(crate) fn unstable_outputs<'a>(
    documents: impl Iterator<Item = &'a hick_lang::HickDocument>,
) -> HashSet<String> {
    let mut out = HashSet::new();
    for doc in documents {
        for tag in doc.all_tags().into_iter().filter(|t| t.name == "file") {
            if tag.child_tags().any(|c| c.name == "exec")
                && let Some(path) = tag.get_attribute("path")
            {
                out.insert(path.trim().trim_start_matches("./").to_string());
            }
        }
        if let Some(weave) = doc.weave_path.as_deref()
            && weave != hick_lang::WEAVE_NONE
        {
            out.insert(weave.trim().trim_start_matches("./").to_string());
        }
    }
    out
}

/// Whether a path inside a mounted volume is one of the documents' unstable
/// products. Matched by suffix as well as exactly, because a volume seeded
/// from `project/` names `project/x.svg` as `x.svg`.
fn is_unstable_output(path: &str, unstable: &HashSet<String>) -> bool {
    unstable.contains(path)
        || unstable
            .iter()
            .any(|u| u.ends_with(path) && u[..u.len() - path.len()].ends_with('/'))
}

/// What a cell's input digest leaves out and puts back, beyond the volume's
/// own bytes. See `docs/specs/freeform/three-axes.md`, axis 1, and
/// `a-recording-is-keyed-by-the-cells-inputs.md`.
#[derive(Default)]
pub(crate) struct DigestPolicy {
    /// The documents' own unstable products and the documents themselves.
    ///
    /// Deliberately NOT "everything under an output volume's path": with
    /// `output="."` that would remove every input and a changed script would
    /// never re-execute the cell that reads it. What an earlier cell wrote
    /// into a volume is kept out by digesting the volume as seeded instead.
    pub unstable: HashSet<String>,
}

impl DigestPolicy {
    pub(crate) fn from_documents(
        documents: &[(&str, hick_lang::HickDocument)],
        _volumes: &HashMap<String, hick_exec::volume::VolumeDeclaration>,
    ) -> Self {
        let mut unstable = unstable_outputs(documents.iter().map(|(_, d)| d));
        // The documents' own bytes are not inputs to their cells either: a
        // recording kept in the document changes the document, and must not
        // thereby go stale the moment it is kept.
        for (name, _) in documents {
            unstable.insert(normalize_rel(name));
        }
        Self { unstable }
    }
}

/// A document's bytes with every `<prefix:ingested key="…">…</prefix:ingested>`
/// removed, for keying. Textual, on any prefix: the recordings are
/// evidence, and a key that included them would move whenever a document
/// kept one.
pub(crate) fn strip_kept_recordings(bytes: &[u8]) -> Vec<u8> {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return bytes.to_vec();
    };
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    loop {
        // `<x:ingested key=`, whatever the prefix.
        let Some(open) = rest
            .find(":ingested key=")
            .and_then(|i| rest[..i].rfind('<'))
        else {
            out.push_str(rest);
            break;
        };
        let prefix = &rest[open + 1..rest[open..].find(':').map(|i| open + i).unwrap_or(open + 1)];
        let close = format!("</{prefix}:ingested>");
        let Some(end) = rest[open..].find(&close).map(|i| open + i + close.len()) else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..open]);
        rest = &rest[end..];
    }
    out.into_bytes()
}

/// `./a/b/`, `a/b`, `.` → `a/b`, or `` for the project root.
fn normalize_rel(path: &str) -> String {
    let p = path.trim().trim_start_matches("./").trim_end_matches('/');
    if p == "." {
        String::new()
    } else {
        p.to_string()
    }
}

fn mounted_inputs_digest(
    volume_store: &volume_state::VolumeStore,
    mounts: &[(String, String)],
    project_dir: &Path,
    policy: &DigestPolicy,
) -> String {
    let unstable = &policy.unstable;
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    for (vol_name, mount_path) in mounts {
        // The volume AS SEEDED, not as earlier cells left it. What an
        // earlier cell wrote into a shared volume is that cell's output, and
        // it reaches this key through the upstream-key term; hashing it
        // here as well gave a run-time key no weave could recompute, since
        // a weave runs nothing and the writes are never there.
        // An unseeded volume — output-only, or seeded from a directory that
        // did not exist — digests as "declared but empty", never as whatever
        // earlier cells left in it.
        let Some(tar) = volume_store.seeded(vol_name) else {
            entries.push((format!("{vol_name}@{mount_path}/"), Vec::new()));
            continue;
        };
        // Hash the unpacked entries, not the tar: a tar carries mtimes, so
        // hashing it directly would give the same bytes a new key on every
        // run and nothing would ever hit its recording.
        match volume_state::read_tar_files(tar) {
            Ok(files) => {
                let candidates: Vec<String> = files
                    .iter()
                    .map(|(path, _)| path.clone())
                    .filter(|path| !is_run_artifact(path) && !is_unstable_output(path, unstable))
                    .collect();
                // A failure to check (no `git`, no repository) folds into
                // the digest rather than being silently treated as "nothing
                // is ignored" — matching the unreadable-volume branch below,
                // and `gitignored`'s own contract of never staying quiet
                // about "could not check" versus "checked, found nothing".
                let ignored: Option<HashSet<String>> =
                    match volume_state::gitignored(project_dir, &candidates) {
                        Ok(set) => set.map(|v| v.into_iter().collect()),
                        Err(e) => {
                            warn!(
                                "could not check volume '{vol_name}' against \
                                 .gitignore to key the cache: {e}"
                            );
                            entries.push((
                                format!("{vol_name}@{mount_path}/<gitignore check failed>"),
                                e.to_string().into_bytes(),
                            ));
                            None
                        }
                    };
                for (path, body) in files {
                    if is_run_artifact(&path) || is_unstable_output(&path, unstable) {
                        continue;
                    }
                    // At trace level, so a key that differs between a run and
                    // its weave can be explained by the entries rather than
                    // guessed at from the hash.
                    trace!(
                        "key term: {vol_name}@{mount_path}/{path} ({} bytes)",
                        body.len()
                    );
                    if ignored.as_ref().is_some_and(|set| set.contains(&path)) {
                        continue;
                    }
                    // A sibling document is an input like any file — minus
                    // the recordings it keeps, which are evidence about
                    // itself and not input to anything. Without this, keeping
                    // a recording in one document staled every cell in the
                    // folder that mounted it.
                    let body = if path.ends_with(".hick") {
                        strip_kept_recordings(&body)
                    } else {
                        body
                    };
                    entries.push((format!("{vol_name}@{mount_path}/{path}"), body));
                }
            }
            Err(e) => {
                // An unreadable volume must not silently digest as empty —
                // that would let two different states share a key. Fold the
                // error in so the key changes rather than collides.
                warn!("could not read volume '{vol_name}' to key the cache: {e}");
                entries.push((
                    format!("{vol_name}@{mount_path}/<unreadable>"),
                    e.to_string().into_bytes(),
                ));
            }
        }
    }
    cache::inputs_digest(&entries)
}

/// The cache keys of a cell's predecessors, in the DAG's own order.
///
/// A predecessor with no key yet is one that was served without a key being
/// computed — an agent cell, or a cell reached on a re-prepared graph. It
/// contributes a placeholder rather than being skipped, so that "this
/// predecessor existed and we could not key it" never digests identically to
/// "this predecessor was absent".
fn upstream_keys(
    flow_dag: &hick_exec::dag::FlowDag,
    exec_id: hick_exec::dag::ExecId,
    keys_by_exec: &HashMap<hick_exec::dag::ExecId, String>,
) -> Vec<String> {
    let mut predecessors = flow_dag.predecessors(exec_id);
    predecessors.sort_by_key(|id| id.0);
    predecessors
        .into_iter()
        .map(|id| {
            keys_by_exec
                .get(&id)
                .cloned()
                .unwrap_or_else(|| format!("unkeyed:{}", id.0))
        })
        .collect()
}

/// Collect every document's `<hick:expect>` declarations, keyed by cell.
/// Collect every cell's `<hick:capture>` declarations, with the document
/// each came from — the same shape as the expectations, for the same reason:
/// a message about a bad capture must name the file it is in.
fn collect_capture_specs(
    documents: &[(&str, HickDocument)],
) -> Result<HashMap<CellId, (String, Vec<hick_dap::CaptureSpec>)>> {
    let mut out = HashMap::new();
    for (name, doc) in documents {
        let cells = capture::collect(&doc.nodes)
            .map_err(|e| anyhow::anyhow!("invalid <hick:capture> in {name}: {e}"))?;
        for cell in cells {
            out.insert(cell.cell.clone(), (name.to_string(), cell.specs));
        }
    }
    Ok(out)
}

fn collect_expect_specs(
    documents: &[(&str, HickDocument)],
) -> Result<HashMap<CellId, (String, expect::ExpectSpec)>> {
    let mut out = HashMap::new();
    for (name, doc) in documents {
        let specs = expect::collect_expectations(&doc.nodes)
            .map_err(|e| anyhow::anyhow!("invalid <hick:expect> in {name}: {e}"))?;
        for spec in specs {
            out.insert(spec.cell.clone(), (name.to_string(), spec));
        }
    }
    Ok(out)
}

/// Collect `<hick:container>` capabilities and images into the run's maps.
///
/// Used once during preparation and again after an agent cell edits a
/// document, so a container the agent declared is a container the rest of the
/// pass can use.
fn collect_container_defs(
    nodes: &[HickNode],
    container_defs: &mut HashMap<String, ContainerCapabilities>,
    container_images: &mut HashMap<String, String>,
) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            if tag.name == "container" {
                let name = tag_attr(tag, "name").unwrap_or_default();
                if let Some(image) = tag_attr(tag, "image") {
                    container_images.insert(name.clone(), image);
                }
                container_defs.insert(name, build_capabilities_from_tag(tag));
            }
            collect_container_defs(&tag.children, container_defs, container_images);
        }
    }
}

/// Re-read and re-prepare one document after an agent cell edited it on disk.
///
/// Mirrors the per-document half of [`prepare_pipeline`]: parse, resolve
/// includes, register vars, filter conditionals. Deliberately NOT re-run:
/// feature processing and token minting, which are run-wide and already
/// happened — an agent that needs a new capability declares it and the run
/// says so, rather than silently minting one mid-pass.
fn reprepare_document(name: &str, state: &MultiDocumentState) -> Result<HickDocument> {
    let source = std::fs::read_to_string(name).with_context(|| {
        format!(
            "failed to re-read {name} after an agent cell edited it.\n  \
             Next steps: check the file still exists and is readable — an agent's only write \
             channel is edit_doc/edit_output on this path, so a missing file here means \
             something outside the run deleted or moved it."
        )
    })?;
    let mut doc = hick_lang::parse_from_path(&source, std::path::Path::new(name)).map_err(|e| {
        anyhow::anyhow!(
            "parse error in {name} after an agent cell edited it: {e}\n  \
             Next steps: inspect the document — the agent wrote hick markup that no longer \
             parses, and the edit is on disk, so `git diff {name}` shows exactly what it did."
        )
    })?;
    let base_dir = std::path::Path::new(name)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    let mut seen = std::collections::HashSet::new();
    if let Ok(canonical) = std::fs::canonicalize(name) {
        seen.insert(canonical);
    }
    hick_lang::resolve_includes(&mut doc, base_dir, &mut seen)
        .map_err(|e| anyhow::anyhow!("include error in {name}: {e}"))?;
    scan_and_register_vars(&doc.nodes, state);
    filter_conditionals(&mut doc.nodes, state);
    Ok(doc)
}

/// Run the full hick pipeline on in-memory sources.
///
/// Each entry in `sources` is `(document_name, hick_source)`. The pipeline
/// parses, validates the DAG, collects containers, mints tokens, processes
/// copy/paste blocks, and converges file outputs — all without writing to
/// disk.
///
/// `params` supplies CLI `--param key=value` pairs that override document
/// variables.
/// Where a replayed session's action scripts are written, inside the
/// container workdir rather than at an absolute path that only one platform
/// has.
const REPLAY_DIR: &str = ".hick-replay";

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
    let mut never_run: NeverRun = NeverRun::new();
    for (_, doc) in &documents {
        let flow_dag = dag::build_dag(doc).unwrap();
        for exec_id in flow_dag.topological_order() {
            let info = flow_dag.execs.iter().find(|e| e.id == exec_id).unwrap();
            let commands: Vec<String> = vec![info.command.trim().to_string()];
            never_run.insert(cell_id_of(info), NoBaseline::NotExecuted);
            transcripts
                .entry(info.container.clone())
                .or_default()
                .push(ExecTranscriptEntry {
                    commands,
                    output: String::new(),
                    events: Vec::new(),
                    source_line: Some(info.source_line),
                });
        }
    }

    let span_files = union_span_files(documents.iter().map(|(_, d)| d));
    let documents_ref: Vec<(&str, HickDocument)> =
        documents.iter().map(|(n, d)| (*n, d.clone())).collect();
    let (files, provenance_maps) =
        process_pipeline_outputs(&documents_ref, &state, &transcripts, 1).await;

    Ok(PipelineResult {
        files,
        // A dry run executes nothing, so no volume produced anything.
        volume_outputs: HashMap::new(),
        provenance_maps,
        containers: container_defs,
        volume_provenance: HashMap::new(),
        resource_stats: HashMap::new(),
        transcripts,
        expectations: Vec::new(),
        never_run,
        stale: std::collections::BTreeMap::new(),
        keys: std::collections::BTreeMap::new(),
        refreshed: Vec::new(),
        from_document: std::collections::BTreeSet::new(),
        span_files,
    })
}

// ---------------------------------------------------------------------------
// Live pipeline (real container execution)
// ---------------------------------------------------------------------------

/// Callback fired after each exec block completes (success, cache hit, or
/// failure): `(container, exec source line, transcript entry)`. The block id
/// in the rendered block model is `"{container}:{line}"`, so callers can key
/// streamed events exactly like `render` does.
pub type ExecEventHook = Arc<dyn Fn(&str, usize, &ExecTranscriptEntry) + Send + Sync>;

/// Callback fired after EACH cell's own output-volume writes are known —
/// not only once, for all volumes, after the whole run succeeds:
/// `(volume name, that volume's current flushable files)`.
///
/// The post-loop consolidated pass (`PipelineResult::files`) is still the
/// authoritative account of a fully successful run, and still runs
/// regardless of whether this hook is set. This hook exists so a caller
/// (`hick run`/`hick up`) can write a cell's real, already-succeeded output
/// to host disk as it becomes available, instead of only after every LATER
/// cell in the same run also succeeds — see
/// `docs/guarantees/execution/an-earlier-cells-output-survives-a-later-cells-failure.md`.
/// Fired once per output volume a cell's mounts touch, with that volume's
/// CURRENT state (which may include earlier cells' contributions too, for a
/// volume several cells share) — never fired for a volume this document has
/// already ingested, matching the consolidated pass's own exclusion.
pub type VolumeFlushHook = Arc<dyn Fn(&str, &HashMap<String, FileContent>) + Send + Sync>;

/// Configuration for live pipeline execution.
#[derive(Default)]
pub struct PipelineConfig {
    /// Working directory for resolving relative paths in volume declarations.
    pub working_dir: Option<PathBuf>,
    /// Maximum pipeline rounds. `1` (default) preserves single-pass behavior.
    /// Higher values enable reactive re-evaluation of paste selectors that
    /// reference content from container-generated `.hick` files.
    pub max_rounds: usize,
    /// Optional per-exec live event hook (server run streaming).
    pub on_exec: Option<ExecEventHook>,
    /// Optional per-volume incremental flush hook — see [`VolumeFlushHook`].
    pub on_volume_flush: Option<VolumeFlushHook>,
    /// Restrict execution to this subgraph of the document's own DAG, when
    /// set. A cell whose [`hick_exec::dag::ExecId`] is not in the set is
    /// treated as absent — not run, not required to succeed, and not a
    /// dependency failure for anything inside the set. `None` (the default)
    /// is the ordinary, unchanged whole-document run. See
    /// `crate::run_pipeline_live`'s exec loop, and `hickory_cli::run_doc_subset`
    /// for the one caller that sets this (`hick ingest --from`).
    pub subset: Option<std::collections::HashSet<hick_exec::dag::ExecId>>,
    /// Collect cells with no baseline into [`PipelineResult::never_run`]
    /// instead of aborting the run (default: abort).
    ///
    /// `run` aborts: it asks "what is the answer now", and a cell that cannot
    /// answer is a hard stop the user should see immediately. `check` sets
    /// this, because it asks "does this document still verify" — a question
    /// whose honest answer is *unverifiable*, reported for every affected
    /// cell in one pass with its own exit code, not a single abort at the
    /// first one.
    pub collect_unverifiable: bool,
    /// How to run a `<hick:agent>` cell, when this caller can run one at all.
    ///
    /// `None` is the normal state of CI and of any machine with no model
    /// credentials: agent cells are then reported as unverifiable (or served
    /// from a recording, when the cell declares its `model=`) instead of
    /// failing the run. See [`crate::agent_cell`].
    pub agent_runner: Option<Arc<dyn agent_cell::AgentRunner>>,
    /// Hard ceiling on how many times one document may be re-prepared because
    /// an agent cell edited its source.
    ///
    /// `0` (the default) means "the number of agent cells this document
    /// declared when it was first parsed", which is the bound the fixed point
    /// implies: every re-preparation is caused by a distinct agent cell
    /// running, and a cell runs at most once per pass. A positive value
    /// overrides it. Exceeding the bound is a failure, not a truncation —
    /// the same class of invariant as `max_turns`. See
    /// `docs/guarantees/agent/re-preparation-terminates.md`.
    pub max_agent_reprepares: usize,
    /// Run-wide default time limit for a cell that declares no `timeout=`
    /// of its own. `Default` is the built-in 120 seconds; callers that
    /// honour `HICKORY_CELL_TIMEOUT` resolve it with
    /// [`cell_timeout::CellTimeoutDefault::from_env`]. See
    /// `docs/guarantees/execution/a-cell-cannot-hang-a-run.md`.
    pub cell_timeout: cell_timeout::CellTimeoutDefault,
}

/// Run the pipeline with real command execution through an [`Executor`].
///
/// Unlike [`run_pipeline`] which produces placeholder strings for exec tags,
/// this function runs commands via the supplied executor and captures output
/// (including timed transcript events).
///
/// When `cache_config` is `Some` and its mode is not
/// [`cache::CacheMode::Off`], execution results are recorded and reused on
/// subsequent runs if the cache key matches.
///
/// Freeze is decided per cell: `CacheConfig::mode` is the run-wide default and
/// a `freeze=` attribute on the cell overrides it in either direction
/// ([`cache::CacheConfig::mode_for`]). A frozen cell is served from its
/// recording rather than executed; a cell with `freeze="false"` is always
/// executed, even under a run-wide freeze.
///
/// **A frozen cell with no recording is not an error here.** It has no
/// baseline, and this pipeline is the one that establishes baselines: the cell
/// executes once and is recorded. Only a verifier — `PipelineConfig::
/// collect_unverifiable`, set by `hick test` — stops instead, reporting the
/// cell in `PipelineResult::never_run`, so verification can never manufacture
/// the baseline it then compares against.
pub async fn run_pipeline_live(
    sources: &[(&str, &str)],
    config: &PipelineConfig,
    params: &[(String, String)],
    cache_config: Option<&cache::CacheConfig>,
    executor: Arc<dyn Executor>,
) -> Result<PipelineResult> {
    let mut root_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut root_key);
    let authority = Arc::new(TokenAuthority::new(&root_key));

    let prepared = prepare_pipeline(sources, &authority, params)?;
    let digest_policy = DigestPolicy::from_documents(&prepared.documents, &prepared.volumes);
    let filled_files = cell_filled_files(prepared.documents.iter().map(|(_, d)| d));
    let doc_recordings = document_recordings(prepared.documents.iter().map(|(_, d)| d));
    let mut cell_keys: std::collections::BTreeMap<CellId, String> =
        std::collections::BTreeMap::new();
    let mut refreshed: Vec<RefreshedRecording> = Vec::new();
    // Products of cells that filled a `hick:file`, waiting to be put into
    // the volumes the next cell mounts. See `inject_products_into_volumes`.
    let mut pending_products: Vec<(String, Vec<u8>)> = Vec::new();
    let PreparedPipeline {
        mut documents,
        state,
        mut container_defs,
        mut container_images,
        container_needs,
        fork_registrations,
        volumes: mut all_volume_decls,
    } = prepared;

    // Computed once, up front, and reused both by the per-cell incremental
    // flush hook (below) and the post-loop consolidated pass: a volume this
    // document has ingested is excluded from BOTH, for the same reason —
    // the document owns those bytes, and a fresh flush over the top of them
    // (partial or complete) would silently overwrite the scaffolder's
    // originals back over the author's edits.
    let ingested_volumes = ingested_volume_names(documents.iter().map(|(_, d)| d));

    // Register fork definitions with the executor
    for (from, to, additional_caps) in &fork_registrations {
        executor
            .register_fork(to, from, additional_caps.clone())
            .await?;
    }

    // Hand every container's declared capabilities to the executor BEFORE
    // anything starts. A sandboxed backend confines a container at start
    // time, so a declaration that arrives with the first exec is a
    // declaration that arrives too late.
    for (name, caps) in &container_defs {
        executor.declare_capabilities(name, caps.clone()).await?;
    }

    // Then check that what the document says it needs is actually here,
    // BEFORE the first cell runs. Half a pipeline's side effects followed by
    // `duckdb: not found` is the worst ordering available: the document has
    // already changed things and still cannot finish.
    {
        let missing =
            crate::needs::preflight(executor.as_ref(), &container_needs, &container_images).await?;
        let named = documents
            .first()
            .map(|(path, _)| path.to_string())
            .unwrap_or_else(|| "this document".to_string());
        crate::needs::require(&missing, &named)?;
    }

    // Collect <hick:expect> expectations per cell. Keyed by `CellId` rather
    // than by `(container, line)` so an agent cell — which has no container —
    // can carry one too.
    let mut expect_specs = collect_expect_specs(&documents)?;
    // Captures are read now, before anything runs, so a malformed `at=` is a
    // parse-time complaint rather than something discovered after the cell it
    // belongs to has already had its side effects.
    let mut capture_specs = collect_capture_specs(&documents)?;
    // The source of each document, for the capture runner: it weaves the
    // document itself, into a scratch directory it then debugs.
    let doc_sources: HashMap<String, String> = sources
        .iter()
        .map(|(name, source)| ((*name).to_string(), (*source).to_string()))
        .collect();
    let mut expectations: Vec<ExpectationOutcome> = Vec::new();
    // Per-container source lines, in the order entries were appended.
    let mut exec_lines: HashMap<String, Vec<usize>> = HashMap::new();
    // Cells that could not be verified because nothing was ever recorded for
    // them. Only ever non-empty when `config.collect_unverifiable` is set;
    // otherwise the same conditions abort the run.
    let mut never_run: NeverRun = NeverRun::new();

    // Volume declarations were collected during preparation (their access
    // rules are part of container capabilities); set up the store over them.
    let mut volume_store = volume_state::VolumeStore::new();
    let mut volume_provenance: HashMap<String, Vec<String>> = HashMap::new();

    // Seed input volumes from host directories
    let working_dir = config
        .working_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    seed_input_volumes(&mut volume_store, &all_volume_decls, &working_dir, &[])?;
    inject_products(
        &mut volume_store,
        &all_volume_decls,
        &literal_products(documents.iter().map(|(_, d)| d)),
        true,
    )?;

    for doc_index in 0..documents.len() {
        let name = documents[doc_index].0;
        let mut flow_dag = dag::build_dag(&documents[doc_index].1)
            .map_err(|e| anyhow::anyhow!("DAG validation failed in {name}: {e}"))?;
        info!(
            "DAG validated for {name}: {} exec nodes, running in topological order",
            flow_dag.execs.len()
        );

        // The re-preparation bound. An agent cell may edit the document it
        // lives in, which changes the graph the rest of this pass runs
        // against; the pipeline therefore re-prepares the document and resumes
        // after that cell's barrier. Left unbounded, a document whose agent
        // writes another agent cell re-prepares forever.
        //
        // Fixed point: a pass in which no agent cell edited the source.
        // Bound: the number of agent cells the document declared when it was
        // first parsed — every re-preparation is caused by a distinct agent
        // cell running, and a cell runs at most once per pass. Exceeding it is
        // a failure, not a truncation, exactly as an exhausted `max_turns` is.
        // See docs/guarantees/agent/re-preparation-terminates.md.
        let reprepare_budget = if config.max_agent_reprepares > 0 {
            config.max_agent_reprepares
        } else {
            flow_dag.execs.iter().filter(|e| e.is_agent()).count()
        };
        let mut reprepares = 0usize;

        // Each cell's cache key, so a downstream cell can chain its
        // predecessors' keys into its own. Filled in topological order, which
        // is why a predecessor's key is always present before it is needed.
        let mut keys_by_exec: HashMap<hick_exec::dag::ExecId, String> = HashMap::new();

        let mut order = flow_dag.topological_order();
        let mut cursor = 0usize;
        while cursor < order.len() {
            let exec_id = order[cursor];
            cursor += 1;

            // A cell outside the requested subgraph is treated as absent:
            // not run, not required to succeed, and — because it is simply
            // never visited — never looked up by a downstream cell's
            // `upstream_keys`. Safe because the caller (`hickory_cli::run_doc_subset`)
            // is required to pass the FULL transitive closure of predecessors,
            // never a partial one: every cell that survives this check has
            // all of its own predecessors surviving it too.
            if let Some(subset) = &config.subset
                && !subset.contains(&exec_id)
            {
                continue;
            }

            // Cloned because an agent cell may replace `flow_dag` underneath
            // us when it re-prepares the document.
            let exec_info = flow_dag
                .execs
                .iter()
                .find(|e| e.id == exec_id)
                .unwrap()
                .clone();
            let exec_info = &exec_info;

            if let Some(agent) = exec_info.agent.clone() {
                let cell = CellId::containerless(exec_info.source_line);
                let container = exec_info.container.clone();

                // Freeze is decided exactly as it is for an exec cell — the
                // point of exec placement is that an agent cell is covered by
                // the same machinery, not by a parallel one.
                let cell_mode = cache::cell_mode(cache_config, exec_info.freeze);

                if exec_info.freeze == Some(true) && cache_config.is_none() {
                    // No recording directory: the declaration cannot be
                    // honoured here (see the exec branch). The cell falls
                    // through to the runner, which an embedded caller without
                    // credentials does not have — and then it is reported as
                    // an agent cell with no runner, which it is.
                    warn!(
                        "the agent cell at line {} declares freeze=\"true\", but this run has \
                         no recording directory, so it was not replayed and will not be \
                         recorded. Run it with `hick run` to establish its baseline.",
                        exec_info.source_line,
                    );
                }

                // The model the recording is keyed on: what the cell declares,
                // else what the configured runner says it would use. With
                // neither there is no key to look anything up with — which is
                // why a cell meant to be replayable in CI should declare
                // `model=`.
                let model = agent.model.clone().or_else(|| {
                    config
                        .agent_runner
                        .as_ref()
                        .map(|r| r.model_name().to_string())
                });

                let mut served = false;
                if let (Some(cc), Some(model)) = (cache_config, model.as_deref())
                    && cell_mode.consults()
                {
                    let key = cache::agent_cache_key(model, &agent.prompt);
                    if let Some(cached) = cache::cache_lookup(cc, &container, &key)? {
                        info!("Cache hit for agent cell at line {}", exec_info.source_line);
                        if let Some((doc_name, spec)) = expect_specs.get(&cell) {
                            expectations.push(expect::evaluate(spec, doc_name, &cached.output));
                        }
                        executor.inject_transcript_entry(
                            &container,
                            ExecTranscriptEntry {
                                commands: cached.commands,
                                output: cached.output,
                                events: Vec::new(),
                                source_line: Some(exec_info.source_line),
                            },
                        );
                        exec_lines
                            .entry(container.clone())
                            .or_default()
                            .push(exec_info.source_line);
                        served = true;
                    } else if cell_mode == cache::CacheMode::Require && config.collect_unverifiable
                    {
                        // A verifier stops here; `run` falls through to the
                        // runner and records what it answers, which is how a
                        // cell frozen from the start gets its baseline.
                        never_run.insert(
                            cell,
                            if cc.cache_dir.is_dir() {
                                NoBaseline::FrozenWithoutRecording {
                                    command: first_command_line(&agent.prompt),
                                    frozen_by_cell: exec_info.freeze == Some(true),
                                }
                            } else {
                                NoBaseline::FrozenWithoutCacheDirectory {
                                    command: first_command_line(&agent.prompt),
                                }
                            },
                        );
                        continue;
                    }
                }
                if served {
                    continue;
                }

                let Some(runner) = config.agent_runner.clone() else {
                    if config.collect_unverifiable {
                        never_run.insert(
                            cell,
                            NoBaseline::AgentWithoutRunner {
                                prompt: first_command_line(&agent.prompt),
                                model_declared: agent.model.is_some(),
                            },
                        );
                        continue;
                    }
                    anyhow::bail!(
                        "the agent cell at line {} needs a model to run, and this run has no \
                         agent runner configured (prompt: {}).\n\
                         Next steps: export the provider's API key (`ANTHROPIC_API_KEY`, or the \
                         key for whichever provider you use) and re-run; or record the cell once \
                         with `hick run --cache` on a machine that has one, declare \
                         model=\"…\" on the cell so the recording can be found, and commit the \
                         recording.\n\
                         Common cause: CI and the server's live preview deliberately run with no \
                         credentials — an agent cell there is reported by `hick test` as \
                         unverifiable rather than executed.",
                        exec_info.source_line,
                        first_command_line(&agent.prompt),
                    );
                };

                let outcome = runner
                    .run(agent_cell::AgentRequest {
                        doc_path: PathBuf::from(name),
                        project_dir: working_dir.clone(),
                        prompt: agent.prompt.clone(),
                        max_turns: agent.max_turns,
                        model: agent.model.clone(),
                        source_line: exec_info.source_line,
                    })
                    .await
                    .with_context(|| {
                        format!(
                            "the agent cell at line {} in {name} did not settle",
                            exec_info.source_line
                        )
                    })?;

                executor.inject_transcript_entry(
                    &container,
                    ExecTranscriptEntry {
                        commands: vec![agent.prompt.trim().to_string()],
                        output: outcome.summary.clone(),
                        events: Vec::new(),
                        source_line: Some(exec_info.source_line),
                    },
                );
                exec_lines
                    .entry(container.clone())
                    .or_default()
                    .push(exec_info.source_line);
                if let Some((doc_name, spec)) = expect_specs.get(&cell) {
                    expectations.push(expect::evaluate(spec, doc_name, &outcome.summary));
                }
                if let Some(cc) = cache_config
                    && !config.collect_unverifiable
                    && (cc.mode.records() || cell_mode.records())
                {
                    let key = cache::agent_cache_key(&outcome.model, &agent.prompt);
                    cache::cache_store(
                        cc,
                        &container,
                        &key,
                        &cache::ExecCacheEntry {
                            commands: vec![agent.prompt.trim().to_string()],
                            output: outcome.summary.clone(),
                            output_hash: cache::sha256_hex(&outcome.summary),
                        },
                    )?;
                }

                if !outcome.edited_source {
                    continue;
                }

                // The agent changed the document. Everything after this cell
                // is now a different graph, so re-prepare and resume after
                // this cell's barrier — which is exactly the set of cells that
                // have not run yet, because the barrier is what guarantees it.
                reprepares += 1;
                if reprepares > reprepare_budget {
                    anyhow::bail!(
                        "{name}: agent cells re-prepared the document {reprepares} times, past \
                         the bound of {reprepare_budget}.\n\
                         The bound is the number of agent cells the document declared when it \
                         was first parsed, because each one runs at most once per pass — so \
                         exceeding it means an agent authored another agent cell, and the \
                         document has no fixed point.\n\
                         Next steps: have the agent write `hick:file` and `hick:exec` cells \
                         rather than another `hick:agent` cell, or raise the bound deliberately \
                         if the extra round is intended.\n\
                         This is the same class of invariant as max-turns: a run that cannot \
                         reach a fixed point fails rather than reporting a partial result."
                    );
                }
                info!(
                    "{name}: agent cell {} edited the source; re-preparing (round {reprepares} \
                     of {reprepare_budget})",
                    agent.key()
                );

                let reprepared = reprepare_document(name, &state)?;
                collect_container_defs(
                    &reprepared.nodes,
                    &mut container_defs,
                    &mut container_images,
                );
                documents[doc_index].1 = reprepared;
                flow_dag = dag::build_dag(&documents[doc_index].1)
                    .map_err(|e| anyhow::anyhow!("DAG validation failed in {name}: {e}"))?;
                for (vol_name, vol_decl) in &flow_dag.volumes {
                    all_volume_decls
                        .entry(vol_name.clone())
                        .or_insert_with(|| vol_decl.clone());
                }
                // Pick up files the agent wrote, without discarding what
                // earlier cells in this same pass already wrote into a volume.
                let written: Vec<String> = volume_provenance.keys().cloned().collect();
                seed_input_volumes(&mut volume_store, &all_volume_decls, &working_dir, &written)?;
                expect_specs = collect_expect_specs(&documents)?;
                capture_specs = collect_capture_specs(&documents)?;

                order = flow_dag.topological_order();
                let key = agent.key();
                let resumed = flow_dag
                    .execs
                    .iter()
                    .find(|e| e.agent.as_ref().is_some_and(|a| a.key() == key))
                    .and_then(|e| order.iter().position(|id| *id == e.id));
                cursor = match resumed {
                    Some(pos) => pos + 1,
                    None => anyhow::bail!(
                        "{name}: the agent cell {key} edited the document and is no longer in \
                         it, so the run cannot say where to resume.\n\
                         Next steps: an agent cell must not delete or rename itself; give it an \
                         id= so it stays identifiable across its own edits, and have it edit the \
                         document around itself rather than over itself."
                    ),
                };
                continue;
            }

            // Precedence: an `image=` on the exec overrides the container's
            // declaration, which overrides the historical default.
            let image = exec_info
                .image
                .as_deref()
                .or_else(|| {
                    container_images
                        .get(&exec_info.container)
                        .map(String::as_str)
                })
                .unwrap_or(DEFAULT_IMAGE);

            // Freeze is a per-cell property with a run-wide default. A frozen
            // cell asks "does the recorded answer still hold", so it is served
            // from its recording rather than executed; the `freeze=` attribute
            // wins over the run-wide flag in both directions, so
            // `freeze="false"` keeps a cell live even under `hick run
            // --freeze`.
            let cell_mode = cache::cell_mode(cache_config, exec_info.freeze);

            if exec_info.freeze == Some(true) && cache_config.is_none() {
                // Embedded run paths — the server's live preview, watch mode,
                // the agent's own tool calls — have no recording directory at
                // all, so they can neither replay this cell nor record it.
                // The declaration cannot be honoured here; executing is the
                // honest fallback, and saying so beats failing a preview over
                // a cell that would run fine under `hick run`.
                warn!(
                    "the exec in container '{}' at line {} declares freeze=\"true\", but this \
                     run has no recording directory, so the cell executed and was not \
                     recorded. Run it with `hick run` (which records a frozen cell the \
                     first time it runs) to establish its baseline.",
                    exec_info.container, exec_info.source_line,
                );
            }

            // The cell's cache key, computed ONCE here — before the cell runs
            // — and reused for both the lookup below and the store further
            // down. Two things make the placement load-bearing: the key
            // describes the cell's *inputs*, which stop being observable the
            // moment the cell writes to a volume it also reads; and a lookup
            // and a store that computed their keys separately could disagree,
            // which would record every cell under a key nothing ever looks up.
            let key_terms_note;
            let exec_key = {
                let caps_canonical = cache::canonical_caps(&container_defs, &exec_info.container);
                let secret_names = cache::secret_names_for(&container_defs, &exec_info.container);
                let secret_refs: Vec<&str> = secret_names.iter().map(|s| s.as_str()).collect();
                let input_digest = mounted_inputs_digest(
                    &volume_store,
                    &exec_info.mounts,
                    &working_dir,
                    &digest_policy,
                );
                let upstream = upstream_keys(&flow_dag, exec_id, &keys_by_exec);
                let upstream_refs: Vec<&str> = upstream.iter().map(String::as_str).collect();
                key_terms_note = format!(
                    "image={image} caps={caps_canonical} secrets={secret_refs:?} input={input_digest} \
                     upstream={upstream_refs:?} command={:?}",
                    exec_info.command
                );
                cache::exec_cache_key(
                    image,
                    &caps_canonical,
                    &exec_info.command,
                    &secret_refs,
                    &input_digest,
                    &upstream_refs,
                )
            };
            keys_by_exec.insert(exec_id, exec_key.clone());
            cell_keys.insert(
                CellId::exec(&exec_info.container, exec_info.source_line),
                exec_key.clone(),
            );
            trace!(
                "key terms for {}: {}",
                CellId::exec(&exec_info.container, exec_info.source_line),
                key_terms_note
            );

            // Check cache before executing
            if let Some(cc) = cache_config
                && cell_mode.consults()
            {
                let key = &exec_key;

                let from_document = doc_recordings
                    .get(&CellId::exec(&exec_info.container, exec_info.source_line))
                    .and_then(|recs| recs.iter().find(|r| &r.key == key))
                    .map(|rec| cache::ExecCacheEntry {
                        commands: vec![exec_info.command.trim().to_string()],
                        output: rec.output.clone(),
                        output_hash: cache::sha256_hex(&rec.output),
                    });
                if let Some(cached) = match from_document {
                    Some(rec) => Some(rec),
                    None => cache::cache_lookup(cc, &exec_info.container, key)?,
                } {
                    info!(
                        "Cache hit for exec in '{}': {}…",
                        exec_info.container,
                        &key[..12]
                    );
                    if let Some((doc_name, spec)) =
                        expect_specs.get(&CellId::exec(&exec_info.container, exec_info.source_line))
                    {
                        expectations.push(expect::evaluate(spec, doc_name, &cached.output));
                    }
                    if let Some((rel_path, show)) =
                        filled_files.get(&(exec_info.container.clone(), exec_info.source_line))
                    {
                        let rendered = hick_handlers::render_transcript(
                            &[TranscriptEntry {
                                commands: cached.commands.clone(),
                                output: cached.output.clone(),
                                source_line: Some(exec_info.source_line),
                            }],
                            *show,
                        );
                        pending_products.push((rel_path.clone(), rendered.into_bytes()));
                    }
                    executor.inject_transcript_entry(
                        &exec_info.container,
                        ExecTranscriptEntry {
                            commands: cached.commands,
                            output: cached.output,
                            events: Vec::new(),
                            source_line: Some(exec_info.source_line),
                        },
                    );
                    exec_lines
                        .entry(exec_info.container.clone())
                        .or_default()
                        .push(exec_info.source_line);
                    if let Some(hook) = &config.on_exec
                        && let Some(entry) = executor
                            .transcripts()
                            .get(&exec_info.container)
                            .and_then(|v| v.last())
                    {
                        hook(&exec_info.container, exec_info.source_line, entry);
                    }
                    continue;
                } else if cell_mode == cache::CacheMode::Require && config.collect_unverifiable {
                    // Frozen, and nothing was ever recorded for it: no
                    // baseline exists. Not drift — drift needs a baseline to
                    // have drifted FROM. Only a verifier stops here: `run`
                    // falls through, executes the cell once, and records it,
                    // which is exactly the baseline a verifier must never
                    // create for itself.
                    never_run.insert(
                        CellId::exec(&exec_info.container, exec_info.source_line),
                        if cc.cache_dir.is_dir() {
                            NoBaseline::FrozenWithoutRecording {
                                command: first_command_line(&exec_info.command),
                                frozen_by_cell: exec_info.freeze == Some(true),
                            }
                        } else {
                            NoBaseline::FrozenWithoutCacheDirectory {
                                command: first_command_line(&exec_info.command),
                            }
                        },
                    );
                    continue;
                }
            }

            // Cache miss or caching disabled — execute for real
            if exec_info.is_script {
                // <hick:script> ran through hick-shell's in-process wasm
                // command interpreter, which was removed with the wasm
                // container runtime. See docs/developers/vendoring-notes.md.
                anyhow::bail!(
                    "<hick:script> blocks are not supported: the in-process shell \
                     interpreter was part of the removed wasm runtime. \
                     Use <hick:exec container=\"...\"> instead (line {}).",
                    exec_info.source_line
                );
            } else {
                // Container-based execution
                executor.ensure_started(&exec_info.container, image).await?;

                // What earlier cells produced into `hick:file`s goes into the
                // volumes first, so this cell reads this run's products.
                if !pending_products.is_empty() {
                    debug!(
                        "injecting {} product(s) into volumes before the cell at line {}",
                        pending_products.len(),
                        exec_info.source_line
                    );
                    inject_products_into_volumes(
                        &mut volume_store,
                        &all_volume_decls,
                        &pending_products,
                    )?;
                    pending_products.clear();
                }

                // Inject volumes before exec (or just create the mount point
                // for the first writer when the volume is still empty).
                //
                // A volume's `<hick:allow>` rules decide what crosses this
                // boundary. A container the rules do not name gets nothing —
                // and is told so, rather than running against an empty
                // directory and reporting a confusing missing-file error. A
                // container named for part of the volume gets that part.
                for (vol_name, mount_path) in &exec_info.mounts {
                    let read_scope = all_volume_decls
                        .get(vol_name)
                        .map(|decl| decl.read_scope(&exec_info.container))
                        .unwrap_or(hick_exec::volume::AccessScope::All);

                    if read_scope.is_none() {
                        anyhow::bail!(
                            "container '{}' mounts volume '{vol_name}' at {mount_path} but the \
                             volume grants it no access (line {}).\n\
                             Next steps: add `<hick:allow container=\"{}\" read=\"**\" />` (or a \
                             narrower glob, or `write=` for write access) inside \
                             `<hick:volume name=\"{vol_name}\">`, or drop the mount from this \
                             exec.\n\
                             Note: a volume with no `<hick:allow>` children at all is \
                             unrestricted — access rules apply to every container once any \
                             container is named.",
                            exec_info.container,
                            exec_info.source_line,
                            exec_info.container,
                        );
                    }

                    if let Some(tar_data) = volume_store.get(vol_name) {
                        info!(
                            "Injecting volume '{vol_name}' into container '{}' at {mount_path}",
                            exec_info.container
                        );
                        let scoped;
                        let tar_data = match &read_scope {
                            hick_exec::volume::AccessScope::All => tar_data,
                            scope => {
                                scoped = volume_state::filter_tar(tar_data, |p| scope.permits(p))?;
                                &scoped
                            }
                        };
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
                    let current_transcripts = convert_transcripts(&assign_source_lines(
                        executor.transcripts(),
                        &exec_lines,
                    ));
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
                                        paste_line_indent: None,
                                        registry: Some(&stdin_registry),
                                        context: None,
                                        source_file: None,
                                        span_files: &[],
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

                // Every executed cell runs under a wall-clock limit: the
                // cell's own timeout= attribute, else the run-wide default.
                // docs/guarantees/execution/a-cell-cannot-hang-a-run.md
                let exec_options = ExecOptions {
                    timeout: config.cell_timeout.for_cell(exec_info.timeout_secs),
                };
                let exec_result = executor
                    .execute_with_options(
                        &exec_info.container,
                        &exec_info.command,
                        stdin_content.as_deref(),
                        exec_options,
                    )
                    .await;
                // Fire the live hook even on failure: the transcript entry
                // (with its exit event) is recorded before the error returns.
                if let Some(hook) = &config.on_exec
                    && let Some(entry) = executor
                        .transcripts()
                        .get(&exec_info.container)
                        .and_then(|v| v.last())
                {
                    hook(&exec_info.container, exec_info.source_line, entry);
                }
                let mut output = exec_result?;

                // Captures run *after* the cell, in a scratch clone of the
                // document under a debug adapter. Two runs rather than one on
                // purpose: the authoritative run is the ordinary sandboxed
                // one, and the debugger — which launches the program itself,
                // outside the container — is never allowed to be the thing
                // whose side effects the document keeps. What it contributes
                // is values from inside the functions, which no amount of
                // stdout reaches.
                if let Some((doc_name, specs)) =
                    capture_specs.get(&CellId::exec(&exec_info.container, exec_info.source_line))
                    && let Some(source) = doc_sources.get(doc_name)
                {
                    let table =
                        capture::run_cell(Path::new(doc_name), source, &working_dir, specs).await;
                    if !table.is_empty() {
                        // Appended to the cell's recorded output, which is
                        // what makes `<hick:expect>` able to pin a value from
                        // inside a function.
                        output.push('\n');
                        output.push_str(&table);
                        executor.inject_transcript_entry(
                            &exec_info.container,
                            hickory_executor::ExecTranscriptEntry {
                                commands: Vec::new(),
                                output: table,
                                events: Vec::new(),
                                source_line: Some(exec_info.source_line),
                            },
                        );
                    }
                }
                exec_lines
                    .entry(exec_info.container.clone())
                    .or_default()
                    .push(exec_info.source_line);

                // A cell that fills a file has a product: what the weave will
                // put in that file, rendered the same way.
                if let Some((rel_path, show)) =
                    filled_files.get(&(exec_info.container.clone(), exec_info.source_line))
                    && let Some(entry) = executor
                        .transcripts()
                        .get(&exec_info.container)
                        .and_then(|v| v.last())
                {
                    let rendered = hick_handlers::render_transcript(
                        &[TranscriptEntry {
                            commands: entry.commands.clone(),
                            output: entry.output.clone(),
                            source_line: Some(exec_info.source_line),
                        }],
                        *show,
                    );
                    debug!(
                        "cell at line {} filled {rel_path} ({} bytes), pending for the next cell",
                        exec_info.source_line,
                        rendered.len()
                    );
                    pending_products.push((rel_path.clone(), rendered.into_bytes()));
                }

                // Evaluate the block's <hick:expect> expectation, if any.
                // Failures are recorded, never fatal here — `check` decides.
                if let Some((doc_name, spec)) =
                    expect_specs.get(&CellId::exec(&exec_info.container, exec_info.source_line))
                {
                    let outcome = expect::evaluate(spec, doc_name, &output);
                    if !outcome.passed {
                        warn!(
                            "expectation failed in '{}' at {}:{}: {}",
                            exec_info.container, doc_name, exec_info.source_line, outcome.detail
                        );
                    }
                    expectations.push(outcome);
                }

                // Store the result after successful execution: because the
                // run asked for recordings, or because this cell did. The
                // second half is what lets `freeze="true"` work from the
                // start — the first `hick run` executes the cell once and
                // records it, with no document edit in between. A cell that
                // opted out of freeze still refreshes its recording when the
                // run is caching: opting out means "run me", not "keep me out
                // of the record".
                //
                // A verifier writes nothing at all, whatever the modes say:
                // `collect_unverifiable` is set only by `hick test`, and a
                // check that can write its own baseline is not a check.
                if let Some(cc) = cache_config
                    && !config.collect_unverifiable
                    && (cc.mode.records() || cell_mode.records())
                {
                    // The key computed before the cell ran. Recomputing it here
                    // would key the recording on the volume contents this cell
                    // just produced rather than the ones it read.
                    let key = &exec_key;

                    // Get the last transcript entry that was just added
                    if let Some(entries) = executor.transcripts().get(&exec_info.container)
                        && let Some(last) = entries.last()
                    {
                        let cache_entry = cache::ExecCacheEntry {
                            commands: last.commands.clone(),
                            output: last.output.clone(),
                            output_hash: cache::sha256_hex(&last.output),
                        };
                        cache::cache_store(cc, &exec_info.container, key, &cache_entry)?;
                        info!("Cached exec in '{}': {}…", exec_info.container, &key[..12]);
                        // The document keeps a recording of this cell and it
                        // is now behind: hand the new one to whoever writes
                        // documents.
                        let cell = CellId::exec(&exec_info.container, exec_info.source_line);
                        if doc_recordings.contains_key(&cell) {
                            refreshed.push(RefreshedRecording {
                                cell,
                                key: key.clone(),
                                output: last.output.clone(),
                            });
                        }
                    }
                }

                // Extract volumes after exec, as far as the container's write
                // grant reaches. A reader's changes never leave its own
                // container; a partial writer's changes are merged in only
                // for the paths it was granted, so `write="Controllers/**"`
                // means what it says instead of being either ignored (writes
                // dropped entirely) or over-honoured (whole volume replaced).
                for (vol_name, mount_path) in &exec_info.mounts {
                    let write_scope = all_volume_decls
                        .get(vol_name)
                        .map(|decl| decl.write_scope(&exec_info.container))
                        .unwrap_or(hick_exec::volume::AccessScope::All);

                    if write_scope.is_none() {
                        debug!(
                            "Not extracting volume '{vol_name}' from '{}': read-only access",
                            exec_info.container
                        );
                        continue;
                    }

                    info!(
                        "Extracting volume '{vol_name}' from container '{}' at {mount_path}",
                        exec_info.container
                    );
                    let tar_data = executor
                        .extract_volume(&exec_info.container, mount_path)
                        .await?;
                    let tar_data = match &write_scope {
                        hick_exec::volume::AccessScope::All => tar_data,
                        scope => volume_state::merge_permitted_writes(
                            volume_store.get(vol_name),
                            &tar_data,
                            |p| scope.permits(p),
                        )?,
                    };
                    volume_store.update(vol_name, tar_data);
                    volume_provenance
                        .entry(vol_name.clone())
                        .or_default()
                        .push(exec_info.container.clone());

                    // Best-effort, incremental — fired as soon as THIS
                    // cell's contribution to the volume is known, not only
                    // once for every volume after the whole run succeeds.
                    // Never fired for an already-ingested volume: the
                    // document owns those bytes, matching the consolidated
                    // pass's own exclusion below.
                    if let Some(hook) = &config.on_volume_flush
                        && !ingested_volumes.contains(vol_name)
                        && let Some(vol_decl) = all_volume_decls.get(vol_name)
                    {
                        let files = flushable_volume_files(
                            vol_decl,
                            volume_store.get(vol_name),
                            volume_store.seeded(vol_name),
                        )?;
                        if !files.is_empty() {
                            hook(vol_name, &files);
                        }
                    }
                }
            }
        }
    }

    // Flush output volumes into pipeline result files — except the ones a
    // document has already ingested.
    //
    // Once `<hick:ingested>` holds a run's bytes, the DOCUMENT owns them: it
    // is where your four lines live and where a reverse edit lands. Flushing
    // the fresh run over the top would overwrite those four lines with the
    // scaffolder's originals on every `hick run`, silently. Comparing the two
    // is a three-way merge, and it is deliberately a later step
    // (`docs/specs/freeform/owning-what-a-scaffolder-wrote.md`, sequence 3).
    // `ingested_volumes` was computed once, up front, before the exec loop —
    // see its binding near the top of this function.
    let mut volume_files: HashMap<String, FileContent> = HashMap::new();
    let mut volume_outputs: HashMap<String, HashMap<String, FileContent>> = HashMap::new();
    for (vol_name, vol_decl) in &all_volume_decls {
        let ingested = ingested_volumes.contains(vol_name);
        if ingested {
            info!(
                "Volume '{vol_name}' is ingested into a document, which now owns its bytes; keeping the fresh run aside rather than over them"
            );
        }
        let files = flushable_volume_files(
            vol_decl,
            volume_store.get(vol_name),
            volume_store.seeded(vol_name),
        )?;
        for (output_path, content) in files {
            // Every output volume is recorded here under its own name; only
            // a NOT-yet-ingested one is also flushed into `files`.
            volume_outputs
                .entry(vol_name.clone())
                .or_default()
                .insert(output_path.clone(), content.clone());
            if !ingested {
                volume_files.insert(output_path, content);
            }
        }
    }

    // Before the weave, not after: a `<hick:sample>` shows bytes from the run
    // that is happening, and these are not on disk until this function
    // returns.
    for (path, content) in &volume_files {
        if let Some(text) = content.as_text() {
            state.register_produced_file(path, text);
        }
    }

    let transcripts = assign_source_lines(executor.transcripts(), &exec_lines);
    let resource_stats = executor.resource_stats();
    executor.shutdown().await?;

    let span_files = union_span_files(documents.iter().map(|(_, d)| d));
    let documents_ref: Vec<(&str, HickDocument)> =
        documents.iter().map(|(n, d)| (*n, d.clone())).collect();
    let (mut files, provenance_maps) =
        process_pipeline_outputs(&documents_ref, &state, &transcripts, config.max_rounds).await;
    refuse_on_paste_failures(&state)?;

    // Merge volume output files into the result
    files.extend(volume_files);

    Ok(PipelineResult {
        files,
        volume_outputs,
        provenance_maps,
        containers: container_defs,
        volume_provenance,
        resource_stats,
        transcripts,
        expectations,
        never_run,
        stale: std::collections::BTreeMap::new(),
        keys: cell_keys,
        refreshed,
        from_document: std::collections::BTreeSet::new(),
        span_files,
    })
}

/// Assign per-exec source lines to executor transcript entries, zipping each
/// container's entries with the lines recorded in execution order.
fn assign_source_lines(
    mut transcripts: Transcripts,
    exec_lines: &HashMap<String, Vec<usize>>,
) -> Transcripts {
    for (container, entries) in transcripts.iter_mut() {
        if let Some(lines) = exec_lines.get(container) {
            for (entry, line) in entries.iter_mut().zip(lines.iter()) {
                entry.source_line = Some(*line);
            }
        }
    }
    transcripts
}

/// Run the pipeline in weave mode: NEVER executes commands. Cached transcript
/// entries are used where present; execs without a cached result are woven
/// with a `[never run]` marker and reported in `PipelineResult::never_run`.
pub async fn run_pipeline_weave(
    sources: &[(&str, &str)],
    params: &[(String, String)],
    cache_config: Option<&cache::CacheConfig>,
) -> Result<PipelineResult> {
    let mut root_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut root_key);
    let authority = Arc::new(TokenAuthority::new(&root_key));

    let prepared = prepare_pipeline(sources, &authority, params)?;
    let digest_policy = DigestPolicy::from_documents(&prepared.documents, &prepared.volumes);
    let PreparedPipeline {
        documents,
        state,
        container_defs,
        container_images,
        volumes: all_volume_decls,
        ..
    } = prepared;

    // Weave never executes, so it never seeds a volume for a cell to read.
    // It has to seed them anyway: a cache key covers the cell's inputs, and a
    // weave that skipped this term would compute a different key from the run
    // that wrote the recording and would therefore find nothing — every cell
    // reported never-run immediately after a successful `hick run`.
    let mut volume_store = volume_state::VolumeStore::new();
    if let Some(cc) = cache_config {
        seed_input_volumes(&mut volume_store, &all_volume_decls, &cc.project_dir, &[])?;
        inject_products(
            &mut volume_store,
            &all_volume_decls,
            &literal_products(documents.iter().map(|(_, d)| d)),
            true,
        )?;
    }

    let mut transcripts: Transcripts = HashMap::new();
    let mut never_run: NeverRun = NeverRun::new();
    let mut stale: std::collections::BTreeMap<CellId, String> = std::collections::BTreeMap::new();
    let mut cell_keys: std::collections::BTreeMap<CellId, String> =
        std::collections::BTreeMap::new();
    let mut from_document: std::collections::BTreeSet<CellId> = std::collections::BTreeSet::new();
    let doc_recordings = document_recordings(documents.iter().map(|(_, d)| d));

    for (name, doc) in &documents {
        let flow_dag = dag::build_dag(doc)
            .map_err(|e| anyhow::anyhow!("DAG validation failed in {name}: {e}"))?;
        let mut keys_by_exec: HashMap<hick_exec::dag::ExecId, String> = HashMap::new();
        for exec_id in flow_dag.topological_order() {
            let info = flow_dag.execs.iter().find(|e| e.id == exec_id).unwrap();
            let commands: Vec<String> = vec![info.command.trim().to_string()];

            let cached = match (cache_config, info.agent.as_ref()) {
                // An agent cell replays only from its declared `model=`: the
                // recording is keyed by prompt AND model, and weave has no
                // runner to ask which model would have run. A cell that wants
                // to be woven offline says which model wrote it.
                (Some(cc), Some(agent)) => match agent.model.as_deref() {
                    Some(model) => cache::cache_lookup(
                        cc,
                        &info.container,
                        &cache::agent_cache_key(model, &agent.prompt),
                    )?,
                    None => None,
                },
                (Some(cc), None) => {
                    let image = info
                        .image
                        .as_deref()
                        .or_else(|| container_images.get(&info.container).map(String::as_str))
                        .unwrap_or(DEFAULT_IMAGE);
                    let caps_canonical = cache::canonical_caps(&container_defs, &info.container);
                    let secret_names = cache::secret_names_for(&container_defs, &info.container);
                    let secret_refs: Vec<&str> = secret_names.iter().map(|s| s.as_str()).collect();
                    let input_digest = mounted_inputs_digest(
                        &volume_store,
                        &info.mounts,
                        &cc.project_dir,
                        &digest_policy,
                    );
                    let upstream = upstream_keys(&flow_dag, exec_id, &keys_by_exec);
                    let upstream_refs: Vec<&str> = upstream.iter().map(String::as_str).collect();
                    let key = cache::exec_cache_key(
                        image,
                        &caps_canonical,
                        &info.command,
                        &secret_refs,
                        &input_digest,
                        &upstream_refs,
                    );
                    keys_by_exec.insert(exec_id, key.clone());
                    cell_keys.insert(cell_id_of(info), key.clone());
                    trace!(
                        "key terms for {}: image={image} caps={caps_canonical} secrets={secret_refs:?} \
                         input={input_digest} upstream={upstream_refs:?} command={:?}",
                        cell_id_of(info),
                        info.command
                    );
                    // The document's own recording first, the cache second:
                    // the document is where evidence it chose to keep lives.
                    if let Some(recs) = doc_recordings.get(&cell_id_of(info))
                        && !recs.iter().any(|r| r.key == key)
                    {
                        debug!(
                            "cell {} keeps {} recording(s) in its document, none under its key {}… (kept: {})",
                            cell_id_of(info),
                            recs.len(),
                            &key[..12],
                            recs.iter()
                                .map(|r| r.key.chars().take(12).collect::<String>())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                    match doc_recordings
                        .get(&cell_id_of(info))
                        .and_then(|recs| recs.iter().find(|r| r.key == key))
                    {
                        Some(rec) => {
                            from_document.insert(cell_id_of(info));
                            Some(cache::ExecCacheEntry {
                                commands: commands.clone(),
                                output: rec.output.clone(),
                                output_hash: cache::sha256_hex(&rec.output),
                            })
                        }
                        None => cache::cache_lookup(cc, &info.container, &key)?,
                    }
                }
                (None, _) => None,
            };

            let entry = match cached {
                Some(cached) => ExecTranscriptEntry {
                    commands: cached.commands,
                    output: cached.output,
                    events: Vec::new(),
                    source_line: Some(info.source_line),
                },
                None => {
                    // Stale, or unrecorded: the two must not look alike. A
                    // cell that was recorded and whose inputs moved shows its
                    // last recording, marked; only a cell that was never
                    // recorded gets the marker.
                    let stale_recording = match (cache_config, info.agent.as_ref()) {
                        (Some(cc), None) => match doc_recordings
                            .get(&cell_id_of(info))
                            .and_then(|recs| recs.last())
                        {
                            // The document's recording under another key: the
                            // inputs moved since it was kept.
                            Some(rec) => Some((
                                rec.key.clone(),
                                cache::ExecCacheEntry {
                                    commands: commands.clone(),
                                    output: rec.output.clone(),
                                    output_hash: cache::sha256_hex(&rec.output),
                                },
                            )),
                            None => cache::stale_lookup(cc, &info.container, &commands)?,
                        },
                        _ => None,
                    };
                    match stale_recording {
                        Some((key, recorded)) => {
                            stale.insert(cell_id_of(info), key);
                            ExecTranscriptEntry {
                                commands: recorded.commands,
                                output: recorded.output,
                                events: Vec::new(),
                                source_line: Some(info.source_line),
                            }
                        }
                        None => {
                            never_run.insert(cell_id_of(info), NoBaseline::NotExecuted);
                            ExecTranscriptEntry {
                                commands,
                                output: "[never run]".to_string(),
                                events: Vec::new(),
                                source_line: Some(info.source_line),
                            }
                        }
                    }
                }
            };
            transcripts
                .entry(info.container.clone())
                .or_default()
                .push(entry);
        }
    }

    let span_files = union_span_files(documents.iter().map(|(_, d)| d));
    let documents_ref: Vec<(&str, HickDocument)> =
        documents.iter().map(|(n, d)| (*n, d.clone())).collect();
    let (files, provenance_maps) =
        process_pipeline_outputs(&documents_ref, &state, &transcripts, 1).await;

    Ok(PipelineResult {
        files,
        // Weave-only: nothing executed, so no volume produced anything.
        volume_outputs: HashMap::new(),
        provenance_maps,
        containers: container_defs,
        volume_provenance: HashMap::new(),
        resource_stats: HashMap::new(),
        transcripts,
        expectations: Vec::new(),
        never_run,
        stale,
        keys: cell_keys,
        refreshed: Vec::new(),
        from_document,
        span_files,
    })
}

// ---------------------------------------------------------------------------
// Variable scanning
// ---------------------------------------------------------------------------

/// Recursively scan nodes for `<hick:var>` declarations and register them.
/// This runs before conditional filtering so that vars inside `<hick:when>`
/// blocks are available for condition evaluation.
/// Pre-register every `<hick:copy>` whose body is plain text.
///
/// Only plain-text bodies: a copy whose content comes from a cell cannot be
/// known before that cell runs, and guessing would be worse than the empty
/// string this replaces. Those still resolve the way they always did, in the
/// render pass.
///
/// This recurses, and the copy HANDLER does not — `declare_nodes` walks top
/// level only. So for a copy nested inside anything (a cell's script, a
/// `hick:file`, a `hick:when`) this pass is the ONLY one that ever sees it,
/// and while it registered `id` alone such a copy resolved by `#id` and was
/// invisible to `.class`. Registering both here is what removes that
/// asymmetry; `origin_key` is what stops the two passes counting one block
/// twice.
fn register_literal_copies(
    nodes: &[HickNode],
    state: &MultiDocumentState,
    doc_name: &str,
    span_files: &[String],
) {
    for node in nodes {
        let HickNode::Tag(tag) = node else { continue };
        if tag.name == "copy" && tag.children.iter().all(|c| matches!(c, HickNode::Text(..))) {
            let id = tag.get_attribute("id").unwrap_or("");
            let class = tag.get_attribute("class");
            // The same file the copy HANDLER will resolve, so the two passes
            // agree on this block's identity and the second replaces the
            // first instead of adding a duplicate.
            let file = tag
                .source_span
                .and_then(|s| s.file_id)
                .and_then(|id| span_files.get(usize::from(id)))
                .map(|f| f.as_str())
                .unwrap_or(doc_name);
            state.pre_register_copy_text(
                id,
                class,
                &hick_lang::tag_text(tag),
                hick_handlers::origin_key_in(tag, file),
            );
        }
        register_literal_copies(&tag.children, state, doc_name, span_files);
    }
}

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
pub(crate) fn apply_substitutions_segmented_to_transform(
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

/// Union of several documents' span-file tables, in first-seen order.
fn union_span_files<'a>(docs: impl Iterator<Item = &'a HickDocument>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for doc in docs {
        for f in &doc.span_files {
            if !out.contains(f) {
                out.push(f.clone());
            }
        }
    }
    out
}

pub(crate) fn span_file_table(doc: &hick_lang::HickDocument) -> Vec<Arc<str>> {
    doc.span_files
        .iter()
        .map(|s| Arc::from(s.as_str()))
        .collect()
}

pub(crate) use hick_lang::strip_opening_break;

/// Drop a BOM only from the woven display, keeping source bytes intact.
pub(crate) fn strip_leading_bom<'a>(
    text: &'a str,
    span: Option<&hick_lang::SourceSpan>,
) -> (&'a str, Option<hick_lang::SourceSpan>) {
    const BOM: &str = "\u{feff}";
    let Some(rest) = text.strip_prefix(BOM) else {
        return (text, span.copied());
    };
    let moved = span.map(|s| hick_lang::SourceSpan {
        start: s.start + BOM.len(),
        end: s.end,
        start_line: s.start_line,
        start_col: s.start_col + 1,
        file_id: s.file_id,
    });
    (rest, moved)
}

// The `span_files` threading (include splicing) pushed these over the
// clippy arg limit; a param-struct refactor belongs to that change, not here.
#[allow(clippy::too_many_arguments)]
fn process_file_children(
    children: &[HickNode],
    file_ip: &Arc<InsertionPoint>,
    transcripts: &HashMap<String, Vec<TranscriptEntry>>,
    state: &Arc<MultiDocumentState>,
    indent: usize,
    registry: &TagRegistry,
    source_file: Option<&Arc<str>>,
    span_files: &[Arc<str>],
    ingested: Option<&Arc<str>>,
) {
    let ctx = ProcessingContext {
        state,
        transcripts,
        indent,
        paste_line_indent: None,
        registry: Some(registry),
        context: None,
        source_file: source_file.cloned(),
        span_files,
    };

    // An aligned paste owns its tag line and its final newline.
    let mut pending_aligned_paste_break = false;
    for (position, child) in children.iter().enumerate() {
        match child {
            HickNode::Text(text, span) => {
                // The opening tag's line break is not file content.
                let (text, span) = match (position, pending_aligned_paste_break) {
                    (0, _) | (_, true) => strip_opening_break(text, span.as_ref()),
                    _ => (text.as_str(), span.as_ref().copied()),
                };
                pending_aligned_paste_break = false;
                let dedented = dedent(text, indent);
                // Included spans point to their included file.
                if let Some((span, file)) = span
                    .as_ref()
                    .and_then(|s| ctx.file_of_span(s).map(|f| (s, f)))
                {
                    // Ingested bytes are editable but not authored here.
                    let origin = match ingested {
                        Some(run) => SourceOrigin::Ingested {
                            file,
                            span: *span,
                            run: run.clone(),
                        },
                        None => SourceOrigin::Literal { file, span: *span },
                    };
                    file_ip.add(Arc::new(SpanNode::new(dedented, origin)));
                } else {
                    file_ip.add(Arc::new(StringNode::new(dedented)));
                }
            }
            HickNode::Tag(child_tag) => {
                if let Some(handler) = registry.find(&child_tag.name) {
                    let paste_line_indent = (child_tag.name == "paste")
                        .then(|| paste_indent_before(children, position, indent))
                        .flatten();
                    let aligned_paste = paste_line_indent.is_some();
                    let child_ctx = ProcessingContext {
                        state,
                        transcripts,
                        indent,
                        paste_line_indent,
                        registry: Some(registry),
                        context: None,
                        source_file: source_file.cloned(),
                        span_files,
                    };
                    match handler.process(child_tag, &child_ctx) {
                        Ok(TagResult::Node(n)) => {
                            pending_aligned_paste_break = aligned_paste;
                            file_ip.add(n);
                        }
                        Ok(TagResult::Nodes(ns)) => {
                            pending_aligned_paste_break = aligned_paste;
                            for n in ns {
                                file_ip.add(n);
                            }
                        }
                        Ok(TagResult::Declaration) => {}
                        Err(e) => {
                            // Paste cardinality errors are reported by the pipeline.
                            if child_tag.name == "paste" {
                                debug!("Handler error for <paste>: {e}");
                            } else {
                                warn!("Handler error for <{}>: {}", child_tag.name, e);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The generated-file prefix before a tag on an otherwise-empty line.
pub(crate) fn paste_indent_before(
    children: &[HickNode],
    position: usize,
    indent: usize,
) -> Option<String> {
    let HickNode::Text(before, _) = children.get(position.checked_sub(1)?)? else {
        return None;
    };
    let before = dedent(before, indent);
    let line = before
        .rsplit_once('\n')
        .map_or(before.as_str(), |(_, line)| line);
    (!line.is_empty() && line.chars().all(char::is_whitespace)).then(|| line.to_string())
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

/// Expand a CLI argument into a list of Markdown documents.
///
/// - If `path` is a file, return it directly.
/// - If `path` is a directory with `_hick.yml`, use that config to resolve files.
/// - If `path` is a directory without config, expand `**/*.hick`.
pub fn expand_path_arg(path: &Path) -> Result<(Vec<PathBuf>, Option<PathBuf>)> {
    use anyhow::{Context as _, bail};
    use config::HickConfig;
    if path.is_file() {
        if path.extension().and_then(|e| e.to_str()) == Some("hick") {
            bail!(
                "{} is a legacy .hick document. Rename it to .md after removing its generated sibling; Hickory no longer writes a generated Markdown sibling.",
                path.display()
            );
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            bail!("{} is not a Markdown document (.md)", path.display());
        }
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
            if let Some(legacy) = files
                .iter()
                .find(|file| file.extension().and_then(|e| e.to_str()) == Some("hick"))
            {
                bail!(
                    "{} is a legacy .hick document. Rename it to .md after removing its generated sibling; Hickory no longer writes a generated Markdown sibling.",
                    legacy.display()
                );
            }
            return Ok((files, Some(config_path)));
        }
        let pattern = path.join("**/*.md");
        let pattern_str = pattern.to_string_lossy();
        let mut matches: Vec<PathBuf> = glob::glob(&pattern_str)
            .with_context(|| format!("invalid glob pattern: {}", pattern_str))?
            .filter_map(|r| r.ok())
            .collect();
        matches.sort();
        if matches.is_empty() {
            bail!(
                "No .md documents found in '{}'\n\n\
                 To fix this, either:\n  \
                 1. Create a _hick.yml config file listing your .md documents\n  \
                 2. Add .md documents to the directory\n  \
                 3. Specify files directly: hick run file1.md file2.md",
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

/// The run-wide [`cache::CacheMode`] the two CLI flags select. `--freeze`
/// wins when both are given: it is the stronger statement about a miss.
pub fn cache_mode(cache: bool, freeze: bool) -> cache::CacheMode {
    match (freeze, cache) {
        (true, _) => cache::CacheMode::Require,
        (false, true) => cache::CacheMode::Reuse,
        (false, false) => cache::CacheMode::Off,
    }
}

/// Configuration for `run_pipeline_cmd`.
pub struct PipelineRunOpts {
    pub files: Vec<PathBuf>,
    pub config_path: Option<PathBuf>,
    pub key_file: Option<PathBuf>,
    pub secrets_dir: Option<PathBuf>,
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
    use anyhow::{Context as _, bail};
    use cache::CacheConfig;
    use config::{HickConfig, find_config};
    use log::{debug, info};
    use std::time::Instant;

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
            return crate::pipeline_session_replay(path, &source, opts.verbose).await;
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

    // A cache config always exists on this path, even when neither --cache nor
    // --freeze was passed: `CacheMode::Off` means nothing is recorded or
    // reused run-wide, but a cell that declares freeze="true" still needs the
    // cache directory to find — or, on its first run, to write — its
    // recording. Making this Option::None again would silently turn per-cell
    // freeze into a no-op.
    let cc = CacheConfig::new(config_dir, cache_mode(opts.cache, opts.freeze));
    if opts.clear_cache {
        cache::cache_clear(&cc)?;
        info!("Cache cleared");
    }
    let cc = Some(cc);

    let result = if opts.dry_run {
        run_pipeline(&sources, &params).await?
    } else {
        let pipeline_config = PipelineConfig {
            working_dir: None,
            max_rounds: 1,
            on_exec: None,
            ..Default::default()
        };
        let executor: Arc<dyn Executor> = Arc::new(LocalExecutor::new()?);
        run_pipeline_live(&sources, &pipeline_config, &params, cc.as_ref(), executor).await?
    };

    let mut expected_paths: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
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

    let pipeline_boot = total_boot;

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
        "Done: {} ({} bytes), {} container{}, {} command{} — {} total (boot {}, exec {})",
        file_stats,
        total_output_bytes,
        num_containers,
        if num_containers == 1 { "" } else { "s" },
        total_commands,
        if total_commands == 1 { "" } else { "s" },
        fmt_duration(wall_time),
        fmt_duration(pipeline_boot),
        fmt_duration(total_exec),
    );

    Ok(())
}

/// Replay a session .hick file without calling the LLM.
///
/// Runs through the [`Executor`] boundary (a fresh [`LocalExecutor`]), so
/// replayed commands execute on the host in a temp workdir — the wasm-era
/// `/workspace` preopen of the invoking directory no longer exists.
async fn pipeline_session_replay(file_path: &Path, source: &str, verbose: bool) -> Result<()> {
    use anyhow::Context as _;

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

    let executor: Arc<dyn Executor> = Arc::new(LocalExecutor::new()?);

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
                    // Written, not composed. This used to be
                    // `printf '%s' '<base64>' | base64 -d > /tmp/… && sh …`:
                    // POSIX text handed to whatever shell the executor
                    // resolves to, writing into a `/tmp` that does not exist
                    // on Windows. Every replayed action failed there, and the
                    // failure was printed and stepped over, so a replay that
                    // executed nothing looked like one whose actions were
                    // simply quiet.
                    let platform = executor.script_platform();
                    let (script_path, cmd) = match action.lang.to_lowercase().as_str() {
                        "python" | "python3" | "py" => {
                            let path =
                                platform.join_path(REPLAY_DIR, &format!("action-{action_idx}.py"));
                            let cmd = platform.python_script_command(&path);
                            (path, cmd)
                        }
                        "node" | "js" | "javascript" => {
                            let path =
                                platform.join_path(REPLAY_DIR, &format!("action-{action_idx}.js"));
                            let cmd = format!("node {path}");
                            (path, cmd)
                        }
                        _ => {
                            let path = platform.join_path(
                                REPLAY_DIR,
                                &format!(
                                    "action-{action_idx}.{}",
                                    platform.shell_script_extension()
                                ),
                            );
                            let cmd = platform.shell_script_command(&path);
                            (path, cmd)
                        }
                    };
                    let is_shell = !matches!(
                        action.lang.to_lowercase().as_str(),
                        "python" | "python3" | "py" | "node" | "js" | "javascript"
                    );
                    let preamble = if is_shell {
                        platform.shell_script_preamble()
                    } else {
                        ""
                    };
                    // A replay that cannot write its own script cannot replay,
                    // so this one does NOT step over the failure.
                    executor
                        .write_file(
                            "replay",
                            &script_path,
                            &format!("{preamble}{}", action.code),
                        )
                        .await?;
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
        executor.shutdown().await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Protects docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md
    // (the cache-digest half — `mounted_inputs_digest` must apply the same
    // `.gitignore` filter `seed_from_directory` already applies at seed time).
    #[test]
    fn a_sibling_documents_kept_recordings_are_not_in_the_key() {
        let with = b"<h:doc>\n<h:exec container=\"c\">cmd<h:ingested key=\"k\" sha256=\"s\" at=\"d\">\nout\n</h:ingested></h:exec>\n</h:doc>\n";
        let without = b"<h:doc>\n<h:exec container=\"c\">cmd</h:exec>\n</h:doc>\n";
        assert_eq!(strip_kept_recordings(with), without.to_vec());
        assert_eq!(strip_kept_recordings(without), without.to_vec());
    }

    /// Protects docs/guarantees/execution/an-output-volume-flushes-only-what-the-run-changed.md
    #[test]
    fn a_flush_writes_only_what_the_run_changed() {
        let decl = hick_exec::volume::VolumeDeclaration {
            name: "work".to_string(),
            kind: hick_exec::volume::VolumeKind::InputOutput {
                input: ".".to_string(),
                output: ".".to_string(),
            },
            access_rules: Vec::new(),
        };
        let seeded = volume_state::pack_tar_files(&[
            ("doc.hick".to_string(), b"<doc/>".to_vec()),
            ("out/a.txt".to_string(), b"[never run]".to_vec()),
        ])
        .unwrap();
        let after = volume_state::pack_tar_files(&[
            ("doc.hick".to_string(), b"<doc/>".to_vec()),
            ("out/a.txt".to_string(), b"[never run]".to_vec()),
            ("out/b.txt".to_string(), b"made by a cell".to_vec()),
        ])
        .unwrap();
        let files = flushable_volume_files(&decl, Some(&after), Some(&seeded)).unwrap();
        // The document and the staged placeholder came along for the ride;
        // only the cell's own product is an output.
        assert_eq!(files.len(), 1, "{files:?}");
        assert!(files.contains_key("out/b.txt"));
        // With no record of the seed, everything flushes, as before.
        let files = flushable_volume_files(&decl, Some(&after), None).unwrap();
        assert_eq!(files.len(), 3);
    }

    /// Protects docs/guarantees/verification/a-recording-is-keyed-by-the-cells-inputs.md
    #[test]
    fn a_documents_own_unstable_products_are_not_in_the_key() {
        // A cell mounting `.` mounts its own weave and the file it fills.
        // Those bytes change on every run, so a key that counted them was
        // never findable again; the recorded output of a real run then read
        // `[never run]` on the next weave. They are not inputs.
        let dir = tempfile::tempdir().unwrap();
        let mut store = volume_state::VolumeStore::new();
        let before = vec![
            ("analysis.py".to_string(), b"print(1)".to_vec()),
            ("report.md".to_string(), b"old weave".to_vec()),
            ("chart.svg".to_string(), b"<svg>old</svg>".to_vec()),
        ];
        let after = vec![
            ("analysis.py".to_string(), b"print(1)".to_vec()),
            ("report.md".to_string(), b"new weave".to_vec()),
            ("chart.svg".to_string(), b"<svg>new</svg>".to_vec()),
        ];
        let mounts = vec![("src".to_string(), "/work".to_string())];
        let unstable: HashSet<String> = ["report.md".to_string(), "chart.svg".to_string()]
            .into_iter()
            .collect();
        store.seed_tar("src", volume_state::pack_tar_files(&before).unwrap());
        let policy = DigestPolicy { unstable };
        let key_before = mounted_inputs_digest(&store, &mounts, dir.path(), &policy);
        store.seed_tar("src", volume_state::pack_tar_files(&after).unwrap());
        let key_after = mounted_inputs_digest(&store, &mounts, dir.path(), &policy);
        assert_eq!(
            key_before, key_after,
            "the run's own products moved the key"
        );

        // The script the document assembles IS an input: change it and the
        // key changes, which is the guarantee's whole point.
        let edited = vec![
            ("analysis.py".to_string(), b"print(2)".to_vec()),
            ("report.md".to_string(), b"new weave".to_vec()),
            ("chart.svg".to_string(), b"<svg>new</svg>".to_vec()),
        ];
        store.seed_tar("src", volume_state::pack_tar_files(&edited).unwrap());
        assert_ne!(
            key_after,
            mounted_inputs_digest(&store, &mounts, dir.path(), &policy)
        );
        // And a product under a subdirectory is matched by its tail.
        assert!(is_unstable_output(
            "chart.svg",
            &["project/chart.svg".to_string()].into_iter().collect()
        ));
        assert!(!is_unstable_output(
            "art.svg",
            &["project/chart.svg".to_string()].into_iter().collect()
        ));
    }

    #[test]
    fn mounted_inputs_digest_excludes_gitignored_paths() {
        let dir = tempfile::tempdir().expect("tempdir");
        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .arg("-C")
                .arg(dir.path())
                .args(args)
                .output()
                .expect("run git");
        };
        git(&["init", "-q"]);
        git(&["config", "user.email", "t@example.com"]);
        git(&["config", "user.name", "T"]);
        std::fs::write(dir.path().join(".gitignore"), "build/\n").expect("write .gitignore");

        let mounts = vec![("v".to_string(), "proj".to_string())];
        let digest_of = |stable: &[u8], junk: &[u8]| {
            let mut store = volume_state::VolumeStore::new();
            let tar = volume_state::pack_tar_files(&[
                ("stable.txt".to_string(), stable.to_vec()),
                ("build/junk.txt".to_string(), junk.to_vec()),
            ])
            .unwrap();
            store.seed_tar("v", tar);
            mounted_inputs_digest(&store, &mounts, dir.path(), &DigestPolicy::default())
        };

        assert_eq!(
            digest_of(b"stable", b"one"),
            digest_of(b"stable", b"two"),
            "a gitignored path's content changing must not move the digest — \
             this is the exact instability that made a build cell's own \
             bin/obj destabilize every cell sharing its volume"
        );
        assert_ne!(
            digest_of(b"stable", b"one"),
            digest_of(b"changed", b"one"),
            "a real, non-ignored input changing must still move the digest — \
             otherwise this test would pass by digesting nothing at all"
        );
    }

    // Sibling: outside a repository, `gitignored` returns `None` and nothing
    // is filtered — matching `seed_from_directory`'s own "no repository, keep
    // everything" rule (`a-volume-carries-what-the-repository-carries.md`) —
    // so the digest still moves. Not a regression: `is_run_artifact`'s fixed
    // `.hick-cache`/`.git` exclusions apply regardless of a repository.
    #[test]
    fn mounted_inputs_digest_outside_a_repository_falls_back_to_the_fixed_exclusions() {
        let dir = tempfile::tempdir().expect("tempdir"); // deliberately NOT a git repo
        let mounts = vec![("v".to_string(), "proj".to_string())];
        let digest_of = |junk: &[u8]| {
            let mut store = volume_state::VolumeStore::new();
            let tar =
                volume_state::pack_tar_files(&[("build/junk.txt".to_string(), junk.to_vec())])
                    .unwrap();
            store.seed_tar("v", tar);
            mounted_inputs_digest(&store, &mounts, dir.path(), &DigestPolicy::default())
        };
        assert_ne!(
            digest_of(b"one"),
            digest_of(b"two"),
            "without a repository nothing is filtered beyond .hick-cache/.git"
        );
    }

    // Protects docs/guarantees/verification/a-woven-fenced-block-has-no-leading-bom.md
    #[test]
    fn strip_leading_bom_removes_the_bom_and_advances_the_span() {
        let span = hick_lang::SourceSpan {
            start: 50,
            end: 90,
            start_line: 3,
            start_col: 0,
            file_id: None,
        };
        let (text, moved) = strip_leading_bom("\u{feff}// real content", Some(&span));
        assert_eq!(text, "// real content");
        let moved = moved.expect("a span in must be a span out");
        // The BOM is 3 UTF-8 bytes (U+FEFF), never 1.
        assert_eq!(moved.start, 53);
        assert_eq!(moved.end, 90);
        assert_eq!(moved.start_line, 3, "no newline was crossed");
        assert_eq!(moved.start_col, 1);
    }

    #[test]
    fn strip_leading_bom_leaves_ordinary_text_untouched() {
        let (text, span) = strip_leading_bom("// real content", None);
        assert_eq!(text, "// real content");
        assert!(span.is_none());
    }

    #[test]
    fn strip_leading_bom_only_strips_a_leading_bom_never_one_mid_file() {
        // A BOM is a file-start marker. One sitting mid-content (however it
        // got there) is real content this function has no business touching.
        let (text, _) = strip_leading_bom("line one\n\u{feff}line two", None);
        assert_eq!(text, "line one\n\u{feff}line two");
    }

    // Protects the "key does not assume a container" clause of
    // docs/guarantees/verification/test-separates-unverifiable-from-drifted.md.
    #[test]
    fn a_cell_with_no_container_can_be_named_and_keyed() {
        let agentish = CellId::containerless(12);
        assert_eq!(agentish.container(), None);
        assert_eq!(agentish.to_string(), "line 12");

        let exec = CellId::exec("build", 12);
        assert_eq!(exec.container(), Some("build"));
        assert_eq!(exec.to_string(), "line 12 (container 'build')");

        // Same line, different cells: the key distinguishes them.
        let mut never_run = NeverRun::new();
        never_run.insert(agentish.clone(), NoBaseline::NotExecuted);
        never_run.insert(exec.clone(), NoBaseline::NotExecuted);
        assert_eq!(never_run.len(), 2);
        assert!(never_run.contains_key(&agentish));
        assert!(never_run.contains_key(&exec));
    }

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
