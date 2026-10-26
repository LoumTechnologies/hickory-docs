<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-exec/src/dag.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e6f480388088"></a>
## `crates/hick-exec/src/dag.rs`

Related contracts: [An Agent Cell Is A DAG Vertex, And Its Edges Are A Barrier](../../../../guarantees/agent/an-agent-cell-is-a-dag-barrier/contract.md), [A Sample Shows Generated Lines Without Storing Them](../../../../guarantees/authoring/a-sample-shows-generated-lines-without-storing-them/contract.md), [A Scaffolder's Output Becomes Bytes The Document Owns, Or The Ingest Is Refused](../../../../guarantees/authoring/ingest-owns-what-a-scaffolder-wrote/contract.md), [A Cell Cannot Hang A Run](../../../../guarantees/execution/a-cell-cannot-hang-a-run/contract.md), [Apply repository exclusions when seeding execution inputs](../../../../guarantees/execution/a-volume-carries-what-the-repository-carries/contract.md), [Freeze Is Declared Per Cell, With The Run-Wide Flag As Its Default](../../../../guarantees/verification/freeze-is-declared-per-cell/contract.md).

<a id="f-e6f480388088-ExecId"></a>
### `ExecId`

Information-flow DAG construction and validation.
Analyzes all `<hick:exec>` elements in a parsed hick document and builds
a directed acyclic graph of dependencies based on information flow.

<storycode:copy id="f-e6f480388088-ExecId">
//! Information-flow DAG construction and validation.
//!
//! Analyzes all `<hick:exec>` elements in a parsed hick document and builds
//! a directed acyclic graph of dependencies based on information flow.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use hick_lang::{HickDocument, HickNode, HickTag};

use crate::volume::{VolumeAccess, VolumeAccessRule, VolumeDeclaration, VolumeKind};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Unique identifier for an exec element (document-order index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExecId(pub usize);

</storycode:copy>
<a id="f-e6f480388088-impl-fmt-Display-for-ExecId"></a>
### `impl-fmt::Display for ExecId`

<storycode:copy id="f-e6f480388088-impl-fmt-Display-for-ExecId">
impl fmt::Display for ExecId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exec[{}]", self.0)
    }
}

</storycode:copy>
<a id="f-e6f480388088-DagEdge"></a>
### `DagEdge`

An edge in the DAG representing a dependency.

<storycode:copy id="f-e6f480388088-DagEdge">
/// An edge in the DAG representing a dependency.
#[derive(Debug, Clone)]
pub struct DagEdge {
    pub from: ExecId,
    pub to: ExecId,
    pub reason: DependencyReason,
}

</storycode:copy>
<a id="f-e6f480388088-DependencyReason"></a>
### `DependencyReason`

<storycode:copy id="f-e6f480388088-DependencyReason">
/// Why one exec depends on another.
#[derive(Debug, Clone)]
pub enum DependencyReason {
    /// Two execs in the same container → sequential by document order.
    ContainerState { container: String },
    /// Exec A writes to a volume, exec B reads from it.
    VolumeFlow { volume: String },
    /// Exec A produces `<hick:copy>`, exec B contains `<hick:paste>`.
    CopyPaste { id: String },
    /// Exec A starts a service, exec B has network access to it.
    NetworkService { container: String, port: String },
    /// Fork depends on the source container's last exec.
    Fork { from: String, to: String },
    /// Attenuate depends on the container's previous exec.
    Attenuate { container: String },
    /// An agent cell is a **barrier**: every cell declared before it precedes
    /// it and every cell declared after it follows it.
    ///
    /// Every other edge in this graph is derived from a *declared* read or
    /// write — a mount, a copy id, a container name. An agent cell declares
    /// none of those, and cannot: its read-set and write-set are only known
    /// after it has run, because they are whatever the model decided to look
    /// at and edit. The sound closure over an unknown read/write set is
    /// therefore "reads everything already produced, writes everything not
    /// yet consumed", which is exactly a barrier in document order.
    ///
    /// See `docs/guarantees/agent/an-agent-cell-is-a-dag-barrier.md`.
    AgentBarrier { agent: String },
}

</storycode:copy>
<a id="f-e6f480388088-impl-fmt-Display-for-DependencyReason"></a>
### `impl-fmt::Display for DependencyReason`

<storycode:copy id="f-e6f480388088-impl-fmt-Display-for-DependencyReason">
impl fmt::Display for DependencyReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyReason::ContainerState { container } => {
                write!(f, "container state ({container})")
            }
            DependencyReason::VolumeFlow { volume } => write!(f, "volume ({volume})"),
            DependencyReason::CopyPaste { id } => write!(f, "copy/paste ({id})"),
            DependencyReason::NetworkService { container, port } => {
                write!(f, "network ({container}:{port})")
            }
            DependencyReason::Fork { from, to } => write!(f, "fork ({from} → {to})"),
            DependencyReason::Attenuate { container } => write!(f, "attenuate ({container})"),
            DependencyReason::AgentBarrier { agent } => write!(f, "agent barrier ({agent})"),
        }
    }
}

</storycode:copy>
<a id="f-e6f480388088-AGENT_CONTAINER_PREFIX"></a>
### `AGENT_CONTAINER_PREFIX`

Agent cells
Prefix of the **reserved synthetic container name** an agent cell is given.
`ExecInfo.container` is a plain `String`, and an agent cell has no
container. Rather than restructure `ExecInfo` — and with it every consumer
keyed on `(container, source_line)`: the recording cache directory, the
executor's transcript map, the live event hook — an agent cell is given a
reserved name no document can declare, because `<hick:container name="…">`
names are written by hand and none of them starts with `_agent_`. The same
trick `<hick:script>` already uses (`_script_N`).
The *cell identity* that reaches `never_run` and `hick:expect` is a
`CellId` with **no** container, which is the honest shape; this name only
keeps the container-keyed plumbing addressable. Note the asymmetry
deliberately: identity is containerless, storage is named.

<storycode:copy id="f-e6f480388088-AGENT_CONTAINER_PREFIX">
// ---------------------------------------------------------------------------
// Agent cells
// ---------------------------------------------------------------------------

/// Prefix of the **reserved synthetic container name** an agent cell is given.
///
/// `ExecInfo.container` is a plain `String`, and an agent cell has no
/// container. Rather than restructure `ExecInfo` — and with it every consumer
/// keyed on `(container, source_line)`: the recording cache directory, the
/// executor's transcript map, the live event hook — an agent cell is given a
/// reserved name no document can declare, because `<hick:container name="…">`
/// names are written by hand and none of them starts with `_agent_`. The same
/// trick `<hick:script>` already uses (`_script_N`).
///
/// The *cell identity* that reaches `never_run` and `hick:expect` is a
/// `CellId` with **no** container, which is the honest shape; this name only
/// keeps the container-keyed plumbing addressable. Note the asymmetry
/// deliberately: identity is containerless, storage is named.
pub const AGENT_CONTAINER_PREFIX: &str = "_agent_";

</storycode:copy>
<a id="f-e6f480388088-agent_container_name"></a>
### `agent_container_name`

The reserved container name for the agent cell at document index `index`.
Per-cell rather than one shared name, so two agent cells never acquire a
[`DependencyReason::ContainerState`] edge between them — their ordering
comes from [`DependencyReason::AgentBarrier`], which says something
stronger and truer.

<storycode:copy id="f-e6f480388088-agent_container_name">
/// The reserved container name for the agent cell at document index `index`.
///
/// Per-cell rather than one shared name, so two agent cells never acquire a
/// [`DependencyReason::ContainerState`] edge between them — their ordering
/// comes from [`DependencyReason::AgentBarrier`], which says something
/// stronger and truer.
pub fn agent_container_name(index: usize) -> String {
    format!("{AGENT_CONTAINER_PREFIX}{index}")
}

</storycode:copy>
<a id="f-e6f480388088-is_agent_container"></a>
### `is_agent_container`

Whether a container name is the reserved synthetic name of an agent cell.

<storycode:copy id="f-e6f480388088-is_agent_container">
/// Whether a container name is the reserved synthetic name of an agent cell.
pub fn is_agent_container(container: &str) -> bool {
    container.starts_with(AGENT_CONTAINER_PREFIX)
}

</storycode:copy>
<a id="f-e6f480388088-AgentCell"></a>
### `AgentCell`

What a `<hick:agent>` cell declares.
This is the single additive field on [`ExecInfo`]. It is not a
restructuring of `container`/`command`: those keep their shapes (a
reserved synthetic name and the prompt text respectively) so that every
consumer keyed on them keeps working unchanged.

<storycode:copy id="f-e6f480388088-AgentCell">
/// What a `<hick:agent>` cell declares.
///
/// This is the single additive field on [`ExecInfo`]. It is not a
/// restructuring of `container`/`command`: those keep their shapes (a
/// reserved synthetic name and the prompt text respectively) so that every
/// consumer keyed on them keeps working unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCell {
    /// The cell's `id=`, when declared. This is what identifies the cell
    /// across a re-preparation, after the agent's own edits have moved every
    /// source line in the document.
    pub id: Option<String>,
    /// Ordinal among the agent cells of this document, in document order.
    /// The fallback identity when no `id=` is declared.
    pub ordinal: usize,
    /// The prompt text (`<hick:prompt>`, or the cell's own text content).
    pub prompt: String,
    /// `max-turns=`, when declared. Absent means the runner's default.
    pub max_turns: Option<usize>,
    /// `model=`, when declared.
    ///
    /// The recording key for an agent cell is the prompt **and** the model, so
    /// a declared model is what makes the cell replayable by a caller that has
    /// no agent runner configured — `hick weave`, a dry run, or CI with no
    /// API key. Absent, the key can only be computed while a runner is present
    /// to name the model it would have used.
    pub model: Option<String>,
}

</storycode:copy>
<a id="f-e6f480388088-impl-AgentCell"></a>
### `impl-AgentCell`

<storycode:copy id="f-e6f480388088-impl-AgentCell">
impl AgentCell {
    /// Stable identity across a re-preparation: the declared `id=`, else the
    /// cell's ordinal among agent cells.
    pub fn key(&self) -> AgentKey {
        match &self.id {
            Some(id) => AgentKey::Id(id.clone()),
            None => AgentKey::Ordinal(self.ordinal),
        }
    }
}

</storycode:copy>
<a id="f-e6f480388088-AgentKey"></a>
### `AgentKey`

Identity of an agent cell that survives the agent editing its own document.
Source lines are useless here: an agent's first act is usually to insert
text, which moves every line below it — including its own.

<storycode:copy id="f-e6f480388088-AgentKey">
/// Identity of an agent cell that survives the agent editing its own document.
///
/// Source lines are useless here: an agent's first act is usually to insert
/// text, which moves every line below it — including its own.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AgentKey {
    Id(String),
    Ordinal(usize),
}

</storycode:copy>
<a id="f-e6f480388088-impl-fmt-Display-for-AgentKey"></a>
### `impl-fmt::Display for AgentKey`

<storycode:copy id="f-e6f480388088-impl-fmt-Display-for-AgentKey">
impl fmt::Display for AgentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentKey::Id(id) => write!(f, "id=\"{id}\""),
            AgentKey::Ordinal(n) => write!(f, "agent cell #{n} (no id= declared)"),
        }
    }
}

</storycode:copy>
<a id="f-e6f480388088-ExecInfo"></a>
### `ExecInfo`

Information about a discovered exec element.

<storycode:copy id="f-e6f480388088-ExecInfo">
/// Information about a discovered exec element.
#[derive(Debug, Clone)]
pub struct ExecInfo {
    pub id: ExecId,
    pub container: String,
    pub image: Option<String>,
    /// Volume mounts: `(volume_name, mount_path)`.
    pub mounts: Vec<(String, String)>,
    /// Copy IDs produced by this exec.
    pub produces_copy: Vec<String>,
    /// Paste selectors consumed by this exec.
    pub consumes_paste: Vec<String>,
    /// The command text inside the exec tag.
    pub command: String,
    /// Source line number.
    pub source_line: usize,
    /// Child elements to evaluate as stdin content for the command.
    /// Empty when the exec tag has only text children (command text).
    pub stdin_children: Vec<HickNode>,
    /// True when this exec represents a `<hick:script>` block dispatched
    /// via the lightweight shell interpreter instead of a container.
    pub is_script: bool,
    /// Per-cell freeze declaration from `freeze="true"`/`freeze="false"`.
    ///
    /// `None` means the cell inherits the run-wide default (`hick run
    /// --freeze`). `Some(true)` means this cell is checked against its
    /// recorded output and never executed; `Some(false)` means this cell is
    /// always executed and never satisfied from a recording, even under a
    /// run-wide freeze.
    pub freeze: Option<bool>,
    /// Per-cell execution time limit from `timeout="<seconds>"`.
    ///
    /// `None` means the cell inherits the run-wide default
    /// (`HICKORY_CELL_TIMEOUT`, or 120 seconds). `Some(0)` means this cell
    /// declares itself unbounded — allowed, but only explicitly. Any other
    /// value is the limit in whole seconds.
    pub timeout_secs: Option<u64>,
    /// Path to the WASM toolchain directory (only for script blocks).
    pub toolchain: Option<String>,
    /// Set when this vertex is a `<hick:agent>` cell rather than a command.
    ///
    /// `container` then holds the reserved synthetic name from
    /// [`agent_container_name`] and `command` holds the prompt, so consumers
    /// that only know about `(container, command, source_line)` keep working.
    pub agent: Option<AgentCell>,
}

</storycode:copy>
<a id="f-e6f480388088-impl-ExecInfo"></a>
### `impl-ExecInfo`

<storycode:copy id="f-e6f480388088-impl-ExecInfo">
impl ExecInfo {
    /// Whether this vertex is a `<hick:agent>` cell.
    pub fn is_agent(&self) -> bool {
        self.agent.is_some()
    }
}

</storycode:copy>
<a id="f-e6f480388088-FlowDag"></a>
### `FlowDag`

<storycode:copy id="f-e6f480388088-FlowDag">
/// The validated information-flow DAG.
#[derive(Debug)]
pub struct FlowDag {
    pub execs: Vec<ExecInfo>,
    pub edges: Vec<DagEdge>,
    /// Exec IDs with no incoming edges (ready to start).
    pub roots: Vec<ExecId>,
    /// Volume declarations parsed from `<hick:volume>` tags.
    pub volumes: HashMap<String, VolumeDeclaration>,
}

</storycode:copy>
<a id="f-e6f480388088-impl-FlowDag"></a>
### `impl-FlowDag`

<storycode:copy id="f-e6f480388088-impl-FlowDag">
impl FlowDag {
    /// Get successors of a given exec.
    pub fn successors(&self, id: ExecId) -> Vec<ExecId> {
        self.edges
            .iter()
            .filter(|e| e.from == id)
            .map(|e| e.to)
            .collect()
    }

    /// Get predecessors of a given exec.
    pub fn predecessors(&self, id: ExecId) -> Vec<ExecId> {
        self.edges
            .iter()
            .filter(|e| e.to == id)
            .map(|e| e.from)
            .collect()
    }

    /// Return a topological ordering of exec IDs.
    pub fn topological_order(&self) -> Vec<ExecId> {
        let mut in_degree: HashMap<ExecId, usize> = HashMap::new();
        for exec in &self.execs {
            in_degree.entry(exec.id).or_insert(0);
        }
        for edge in &self.edges {
            *in_degree.entry(edge.to).or_insert(0) += 1;
        }

        let mut queue: VecDeque<ExecId> = in_degree
            .iter()
            .filter(|&(_, d)| *d == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::new();
        while let Some(id) = queue.pop_front() {
            order.push(id);
            for succ in self.successors(id) {
                if let Some(d) = in_degree.get_mut(&succ) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(succ);
                    }
                }
            }
        }

        order
    }
}

</storycode:copy>
<a id="f-e6f480388088-DagValidationError"></a>
### `DagValidationError`

<storycode:copy id="f-e6f480388088-DagValidationError">
// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum DagValidationError {
    #[error("cycle detected involving {0}")]
    Cycle(ExecId),

    #[error("broken flow: exec {consumer} reads volume '{volume}' but no exec writes to it")]
    BrokenVolumeFlow { consumer: ExecId, volume: String },

    #[error(
        "broken flow: exec {consumer} uses paste select '{selector}' but no copy with that id exists"
    )]
    BrokenPasteFlow { consumer: ExecId, selector: String },

    #[error(
        "line {line}: this cell's command contains a <{prefix}:paste select=\"{selector}\">, \n\
         which is not substituted into a command and would have run as nothing.\n\
         Move it out of the command block to make it the cell's STDIN:\n\
         \n\
         \x20   <{prefix}:exec …>\n\
         \x20   <{prefix}:copy id=\"…\">your command, reading stdin</{prefix}:copy>\n\
         \x20   <{prefix}:paste select=\"{selector}\" />\n\
         \x20   </{prefix}:exec>"
    )]
    PasteInsideCommand {
        line: usize,
        selector: String,
        prefix: String,
    },

    #[error("exec {exec} references undeclared container '{container}'")]
    UndeclaredContainer { exec: ExecId, container: String },

    #[error("exec {exec} mounts undeclared volume '{volume}'")]
    UndeclaredVolume { exec: ExecId, volume: String },

    #[error("container '{container}' denied {access} access to volume '{volume}'")]
    VolumeAccessDenied {
        container: String,
        volume: String,
        access: String,
    },

    #[error(
        "exec at line {line} has freeze=\"{value}\", which is not a boolean.\n\
         Next steps: write freeze=\"true\" to check this cell against its recorded \
         output instead of running it, or freeze=\"false\" to always run it. Omit the \
         attribute entirely to inherit the run-wide default set by `hick run --freeze`.\n\
         Accepted values are exactly `true` and `false` (case-insensitive); `1`, `yes`, \
         and `on` are not accepted."
    )]
    InvalidFreeze { line: usize, value: String },

    #[error(
        "the cell at line {line} has timeout=\"{value}\", which is not a whole number of \
         seconds.\n\
         Next steps: write timeout=\"300\" (any whole number of seconds) to give this cell \
         five minutes, timeout=\"0\" to let it run unbounded, or omit the attribute to \
         inherit the run-wide default (HICKORY_CELL_TIMEOUT, or 120 seconds).\n\
         Fractions and units are not accepted: timeout=\"1.5\" and timeout=\"2m\" are both \
         invalid — a malformed limit must fail loudly rather than silently run unbounded."
    )]
    InvalidTimeout { line: usize, value: String },

    #[error(
        "the agent cell at line {line} has max-turns=\"{value}\", which is not a positive whole \
         number.\n\
         Next steps: write max-turns=\"20\" (any positive integer), or omit the attribute to use \
         the runner's default.\n\
         max-turns is a graph invariant, not a cost knob: a cell that never settles blocks the \
         document, so exhausting the budget is a failure rather than a partial result."
    )]
    InvalidMaxTurns { line: usize, value: String },

    #[error(
        "the agent cell at line {line} declares no prompt.\n\
         Next steps: give it a <hick:prompt>…</hick:prompt> child describing the task, or put the \
         prompt text directly inside the <hick:agent> element.\n\
         An agent cell with no prompt has nothing to key its recording on, so it could never be \
         frozen or verified."
    )]
    AgentWithoutPrompt { line: usize },
}

</storycode:copy>
<a id="f-e6f480388088-build_dag"></a>
### `build_dag`

DAG Construction
Build the information-flow DAG from a parsed hick document.

<storycode:copy id="f-e6f480388088-build_dag">
// ---------------------------------------------------------------------------
// DAG Construction
// ---------------------------------------------------------------------------

/// Build the information-flow DAG from a parsed hick document.
pub fn build_dag(doc: &HickDocument) -> Result<FlowDag, DagValidationError> {
    let mut execs = Vec::new();
    let mut containers: HashSet<String> = HashSet::new();
    let mut volume_decls: HashMap<String, VolumeDeclaration> = HashMap::new();
    let mut forks: Vec<(String, String, usize)> = Vec::new(); // (from, to, doc_order)
    let mut attenuates: Vec<(String, usize)> = Vec::new(); // (container, doc_order)

    // Pass 1: Collect containers, volumes, execs, agents, forks, attenuates
    let mut exec_index = 0usize;
    let mut agent_ordinal = 0usize;
    for node in &doc.nodes {
        if let HickNode::Tag(tag) = node {
            match tag.name.as_str() {
                "container" => {
                    if let Some(name) = tag.get_attribute("name") {
                        containers.insert(name.to_string());
                    }
                }
                "volume" => {
                    if let Some(decl) = parse_volume_declaration(tag) {
                        volume_decls.insert(decl.name.clone(), decl);
                    }
                }
                "exec" => {
                    execs.push(extract_exec_info(tag, exec_index)?);
                    exec_index += 1;
                }
                "script" => {
                    execs.push(extract_script_info(tag, exec_index)?);
                    exec_index += 1;
                }
                "agent" => {
                    execs.push(extract_agent_info(tag, exec_index, agent_ordinal)?);
                    exec_index += 1;
                    agent_ordinal += 1;
                }
                "fork" => {
                    if let (Some(from), Some(to)) =
                        (tag.get_attribute("from"), tag.get_attribute("to"))
                    {
                        forks.push((from.to_string(), to.to_string(), exec_index));
                        // Fork creates a new container
                        containers.insert(to.to_string());
                    }
                }
                "attenuate" => {
                    if let Some(container) = tag.get_attribute("container") {
                        attenuates.push((container.to_string(), exec_index));
                    }
                }
                "file" => {
                    // Files can contain nested execs
                    collect_nested_execs(tag, &mut execs, &mut exec_index)?;
                }
                _ => {}
            }
        }
    }

    // Also include containers referenced by execs that have an image attribute
    // (implicit container creation)
    for exec in &execs {
        if exec.image.is_some() {
            containers.insert(exec.container.clone());
        }
    }

    // Pass 2: Build edges
    let mut edges = Vec::new();

    // Container state: sequential dependency for execs in the same container
    let mut last_exec_per_container: HashMap<String, ExecId> = HashMap::new();
    for exec in &execs {
        if let Some(&prev) = last_exec_per_container.get(&exec.container) {
            edges.push(DagEdge {
                from: prev,
                to: exec.id,
                reason: DependencyReason::ContainerState {
                    container: exec.container.clone(),
                },
            });
        }
        last_exec_per_container.insert(exec.container.clone(), exec.id);
    }

    // Classify each exec's role per volume using explicit access rules when
    // available, otherwise fall back to the first-mounter heuristic.
    let mut volume_writers: HashMap<String, Vec<ExecId>> = HashMap::new();
    let mut volume_readers: HashMap<String, Vec<ExecId>> = HashMap::new();
    let mut volume_first_seen: HashSet<String> = HashSet::new();

    for exec in &execs {
        for (vol_name, _mount_path) in &exec.mounts {
            if let Some(decl) = volume_decls.get(vol_name)
                && !decl.access_rules.is_empty()
            {
                // Use explicit access rules
                let has_write = decl.access_rules.iter().any(|r| {
                    r.container == exec.container && matches!(r.access, VolumeAccess::Write(_))
                });
                let has_read = decl.access_rules.iter().any(|r| {
                    r.container == exec.container && matches!(r.access, VolumeAccess::Read(_))
                });

                if has_write {
                    volume_writers
                        .entry(vol_name.clone())
                        .or_default()
                        .push(exec.id);
                }
                if has_read && !has_write {
                    volume_readers
                        .entry(vol_name.clone())
                        .or_default()
                        .push(exec.id);
                }
                continue;
            }

            // Fallback heuristic: first mounter is writer, rest are readers
            if volume_first_seen.contains(vol_name) {
                volume_readers
                    .entry(vol_name.clone())
                    .or_default()
                    .push(exec.id);
            } else {
                volume_first_seen.insert(vol_name.clone());
                volume_writers
                    .entry(vol_name.clone())
                    .or_default()
                    .push(exec.id);
            }
        }
    }

    // Build edges: writers → readers, sequential ordering among writers
    for (vol_name, readers) in &volume_readers {
        if let Some(writers) = volume_writers.get(vol_name) {
            for &writer in writers {
                for &reader in readers {
                    edges.push(DagEdge {
                        from: writer,
                        to: reader,
                        reason: DependencyReason::VolumeFlow {
                            volume: vol_name.clone(),
                        },
                    });
                }
            }
        }
    }

    // Sequential ordering for multiple writers on the same volume
    for (_vol_name, writers) in &volume_writers {
        for window in writers.windows(2) {
            edges.push(DagEdge {
                from: window[0],
                to: window[1],
                reason: DependencyReason::VolumeFlow {
                    volume: _vol_name.clone(),
                },
            });
        }
    }

    // Copy/paste dependencies
    let mut copy_producers: HashMap<String, ExecId> = HashMap::new();
    for exec in &execs {
        for copy_id in &exec.produces_copy {
            copy_producers.insert(copy_id.clone(), exec.id);
        }
    }

    for exec in &execs {
        for selector in &exec.consumes_paste {
            // Extract id from selector (e.g., "#version-info" -> "version-info")
            let id = selector.trim_start_matches('#');
            if let Some(&producer) = copy_producers.get(id)
                && producer != exec.id
            {
                edges.push(DagEdge {
                    from: producer,
                    to: exec.id,
                    reason: DependencyReason::CopyPaste { id: id.to_string() },
                });
            }
        }
    }

    // Fork dependencies: fork depends on the source container's last exec
    for (from_container, _to_container, _order) in &forks {
        if let Some(&last_exec) = last_exec_per_container.get(from_container) {
            // Find execs in the target container
            let to_container = &_to_container;
            for exec in &execs {
                if &exec.container == *to_container {
                    edges.push(DagEdge {
                        from: last_exec,
                        to: exec.id,
                        reason: DependencyReason::Fork {
                            from: from_container.clone(),
                            to: to_container.to_string(),
                        },
                    });
                    break; // Only the first exec in the forked container
                }
            }
        }
    }

    // Attenuate dependencies: attenuate is a barrier between execs in the same container
    // Already handled by container state sequential ordering

    // Agent barriers. An agent cell's read-set and write-set are known only
    // AFTER it runs, so no declared mount, copy id, or container name can tell
    // us what it depends on. The only sound edge set over an unknown
    // read/write set is the conservative closure: everything declared before
    // the cell precedes it, everything declared after it follows it.
    //
    // Cheap to state, and it buys the property the placement spike found
    // decisive — an `<hick:exec>` written after an agent cell really does
    // observe that agent's edits in the same pass, because the barrier is what
    // guarantees it has not run yet. It stays acyclic for free: every edge
    // points from a lower document index to a higher one.
    for agent in execs.iter().filter(|e| e.is_agent()) {
        let name = agent
            .agent
            .as_ref()
            .map(|a| a.key().to_string())
            .unwrap_or_default();
        for other in &execs {
            if other.id == agent.id {
                continue;
            }
            let (from, to) = if other.id.0 < agent.id.0 {
                (other.id, agent.id)
            } else {
                (agent.id, other.id)
            };
            edges.push(DagEdge {
                from,
                to,
                reason: DependencyReason::AgentBarrier {
                    agent: name.clone(),
                },
            });
        }
    }

    // Validate
    let dag = FlowDag {
        execs,
        edges,
        roots: Vec::new(),
        volumes: volume_decls,
    };

    validate_dag(&dag)?;

    // Compute roots
    let has_incoming: HashSet<ExecId> = dag.edges.iter().map(|e| e.to).collect();
    let roots: Vec<ExecId> = dag
        .execs
        .iter()
        .map(|e| e.id)
        .filter(|id| !has_incoming.contains(id))
        .collect();

    Ok(FlowDag {
        execs: dag.execs,
        edges: dag.edges,
        roots,
        volumes: dag.volumes,
    })
}

</storycode:copy>
<a id="f-e6f480388088-parse_freeze"></a>
### `parse_freeze`

Extraction helpers
Parse a `freeze="…"` attribute into a per-cell freeze declaration.
Absent means "inherit the run-wide default"; anything other than an exact
`true`/`false` is rejected rather than silently treated as false, because a
typo in a verification switch that quietly disables verification is the
worst possible failure mode for this attribute.

<storycode:copy id="f-e6f480388088-parse_freeze">
// ---------------------------------------------------------------------------
// Extraction helpers
// ---------------------------------------------------------------------------

/// Parse a `freeze="…"` attribute into a per-cell freeze declaration.
///
/// Absent means "inherit the run-wide default"; anything other than an exact
/// `true`/`false` is rejected rather than silently treated as false, because a
/// typo in a verification switch that quietly disables verification is the
/// worst possible failure mode for this attribute.
fn parse_freeze(tag: &HickTag) -> Result<Option<bool>, DagValidationError> {
    match tag.get_attribute("freeze") {
        None => Ok(None),
        Some(raw) => match raw.trim().to_ascii_lowercase().as_str() {
            "true" => Ok(Some(true)),
            "false" => Ok(Some(false)),
            _ => Err(DagValidationError::InvalidFreeze {
                line: tag.source_line,
                value: raw.to_string(),
            }),
        },
    }
}

</storycode:copy>
<a id="f-e6f480388088-parse_timeout"></a>
### `parse_timeout`

Parse a `timeout="…"` attribute into a per-cell limit in whole seconds.
Absent means "inherit the run-wide default"; `0` means "this cell runs
unbounded" — allowed, but only as an explicit declaration. Anything that
is not a whole number of seconds is rejected rather than silently
ignored, because a typo in a time limit that quietly removes the limit
is precisely the hang this attribute exists to prevent.

<storycode:copy id="f-e6f480388088-parse_timeout">
/// Parse a `timeout="…"` attribute into a per-cell limit in whole seconds.
///
/// Absent means "inherit the run-wide default"; `0` means "this cell runs
/// unbounded" — allowed, but only as an explicit declaration. Anything that
/// is not a whole number of seconds is rejected rather than silently
/// ignored, because a typo in a time limit that quietly removes the limit
/// is precisely the hang this attribute exists to prevent.
fn parse_timeout(tag: &HickTag) -> Result<Option<u64>, DagValidationError> {
    match tag.get_attribute("timeout") {
        None => Ok(None),
        Some(raw) => match raw.trim().parse::<u64>() {
            Ok(n) => Ok(Some(n)),
            Err(_) => Err(DagValidationError::InvalidTimeout {
                line: tag.source_line,
                value: raw.to_string(),
            }),
        },
    }
}

</storycode:copy>
<a id="f-e6f480388088-extract_exec_info"></a>
### `extract_exec_info`

<storycode:copy id="f-e6f480388088-extract_exec_info">
fn extract_exec_info(tag: &HickTag, index: usize) -> Result<ExecInfo, DagValidationError> {
    let container = tag
        .get_attribute("container")
        .unwrap_or("default")
        .to_string();
    let image = tag.get_attribute("image").map(|s| s.to_string());

    // Parse mount attribute: "volume-name:/mount/path"
    let mounts = tag
        .get_attribute("mount")
        .map(|m| {
            m.split(',')
                .filter_map(|entry| {
                    let parts: Vec<&str> = entry.trim().splitn(2, ':').collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    // Find copy/paste tags inside this exec
    let mut produces_copy = Vec::new();
    let mut consumes_paste = Vec::new();
    scan_copy_paste(&tag.children, &mut produces_copy, &mut consumes_paste);

    // A paste nested inside the command block contributes nothing:
    // `command_text` collects text, and a self-closing tag has none. The cell
    // then runs with a blank where the author expected content — no error, no
    // warning, a wrong answer. Refusing here is the only honest option, and
    // the message names the placement that does work.
    if let Some(selector) = paste_inside_command(tag) {
        return Err(DagValidationError::PasteInsideCommand {
            line: tag.source_line,
            selector,
            prefix: "hick".to_string(),
        });
    }

    let command = notebook_command(tag, command_text(tag));

    // Collect non-command child tags (paste, val, etc.) as stdin sources.
    // Command text is extracted separately via command_text() above.
    // `<hick:expect>` and `<hick:capture>` children are verification
    // metadata, never stdin.
    let stdin_children: Vec<HickNode> = tag
        .children
        .iter()
        .filter(|child| {
            matches!(child, HickNode::Tag(t)
                if t.name != "copy"
                    && t.name != "cut"
                    && t.name != "expect"
                    && t.name != "capture"
                    // A window on what this cell produced, rendered by the
                    // weave. Never stdin — feeding a cell its own output back
                    // in is the opposite of what it is for.
                    && t.name != "sample"
                    // Output, not input: an ingested scaffold is never stdin.
                    && t.name != "ingested")
        })
        .cloned()
        .collect();

    Ok(ExecInfo {
        id: ExecId(index),
        container,
        image,
        mounts,
        produces_copy,
        consumes_paste,
        command,
        source_line: tag.source_line,
        stdin_children,
        is_script: false,
        toolchain: None,
        freeze: parse_freeze(tag)?,
        timeout_secs: parse_timeout(tag)?,
        agent: None,
    })
}

</storycode:copy>
<a id="f-e6f480388088-extract_script_info"></a>
### `extract_script_info`

Extract script info from a `<hick:script>` tag.
Script blocks get a synthetic container name (`_script_N`) so they
participate in the DAG without creating container-state edges between
unrelated scripts. Each script is independent unless connected by
explicit volume or copy/paste dependencies.

<storycode:copy id="f-e6f480388088-extract_script_info">
/// Extract script info from a `<hick:script>` tag.
///
/// Script blocks get a synthetic container name (`_script_N`) so they
/// participate in the DAG without creating container-state edges between
/// unrelated scripts. Each script is independent unless connected by
/// explicit volume or copy/paste dependencies.
fn extract_script_info(tag: &HickTag, index: usize) -> Result<ExecInfo, DagValidationError> {
    let container = format!("_script_{index}");
    let toolchain = tag.get_attribute("toolchain").map(|s| s.to_string());

    let mounts = tag
        .get_attribute("mount")
        .map(|m| {
            m.split(',')
                .filter_map(|entry| {
                    let parts: Vec<&str> = entry.trim().splitn(2, ':').collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let mut produces_copy = Vec::new();
    let mut consumes_paste = Vec::new();
    scan_copy_paste(&tag.children, &mut produces_copy, &mut consumes_paste);

    let command = command_text(tag);

    Ok(ExecInfo {
        id: ExecId(index),
        container,
        image: None,
        mounts,
        produces_copy,
        consumes_paste,
        command,
        source_line: tag.source_line,
        stdin_children: Vec::new(),
        is_script: true,
        toolchain,
        freeze: parse_freeze(tag)?,
        timeout_secs: parse_timeout(tag)?,
        agent: None,
    })
}

</storycode:copy>
<a id="f-e6f480388088-extract_agent_info"></a>
### `extract_agent_info`

Extract an agent cell from a `<hick:agent>` tag.
The cell gets the reserved synthetic container name from
[`agent_container_name`] and its prompt as the `command`, so every consumer
keyed on `(container, command, source_line)` keeps working without knowing
what an agent cell is. What it *is* lives in [`ExecInfo::agent`].

<storycode:copy id="f-e6f480388088-extract_agent_info">
/// Extract an agent cell from a `<hick:agent>` tag.
///
/// The cell gets the reserved synthetic container name from
/// [`agent_container_name`] and its prompt as the `command`, so every consumer
/// keyed on `(container, command, source_line)` keeps working without knowing
/// what an agent cell is. What it *is* lives in [`ExecInfo::agent`].
fn extract_agent_info(
    tag: &HickTag,
    index: usize,
    ordinal: usize,
) -> Result<ExecInfo, DagValidationError> {
    // A `<hick:prompt>` child is the declared form; bare text inside the cell
    // is accepted so a one-line agent cell need not nest a tag.
    let prompt = match tag.child_tags().find(|t| t.name == "prompt") {
        Some(prompt_tag) => prompt_tag.text_content(),
        None => command_text(tag),
    };
    if prompt.trim().is_empty() {
        return Err(DagValidationError::AgentWithoutPrompt {
            line: tag.source_line,
        });
    }

    let max_turns = match tag.get_attribute("max-turns") {
        None => None,
        Some(raw) => match raw.trim().parse::<usize>() {
            Ok(n) if n > 0 => Some(n),
            _ => {
                return Err(DagValidationError::InvalidMaxTurns {
                    line: tag.source_line,
                    value: raw.to_string(),
                });
            }
        },
    };

    Ok(ExecInfo {
        id: ExecId(index),
        container: agent_container_name(index),
        image: None,
        mounts: Vec::new(),
        produces_copy: Vec::new(),
        consumes_paste: Vec::new(),
        command: prompt.clone(),
        source_line: tag.source_line,
        stdin_children: Vec::new(),
        is_script: false,
        toolchain: None,
        freeze: parse_freeze(tag)?,
        // An agent cell is bounded by `max-turns`, not by wall-clock: it
        // never reaches the executor's spawn path, so a `timeout=` here
        // would be a knob that does nothing.
        timeout_secs: None,
        agent: Some(AgentCell {
            id: tag.get_attribute("id").map(|s| s.to_string()),
            ordinal,
            prompt,
            max_turns,
            model: tag.get_attribute("model").map(|s| s.to_string()),
        }),
    })
}

</storycode:copy>
<a id="f-e6f480388088-paste_inside_command"></a>
### `paste_inside_command`

The command text of an exec/script tag: all text content EXCLUDING any
`<hick:expect>` or `<hick:capture>` subtree. Both sit inside the exec tag
for locality but are verification metadata, not part of the command.
The selector of the first `paste` nested inside this exec's command
block, if there is one.
Only inside a `copy`/`cut` child — a `paste` that is a direct child of the
exec is the stdin form, which is supported and is what the error suggests.

<storycode:copy id="f-e6f480388088-paste_inside_command">
/// The command text of an exec/script tag: all text content EXCLUDING any
/// `<hick:expect>` or `<hick:capture>` subtree. Both sit inside the exec tag
/// for locality but are verification metadata, not part of the command.
/// The selector of the first `paste` nested inside this exec's command
/// block, if there is one.
///
/// Only inside a `copy`/`cut` child — a `paste` that is a direct child of the
/// exec is the stdin form, which is supported and is what the error suggests.
fn paste_inside_command(tag: &HickTag) -> Option<String> {
    fn find(nodes: &[HickNode]) -> Option<String> {
        for node in nodes {
            if let HickNode::Tag(t) = node {
                if t.name == "paste" {
                    return Some(t.get_attribute("select").unwrap_or("…").to_string());
                }
                if let Some(found) = find(&t.children) {
                    return Some(found);
                }
            }
        }
        None
    }
    tag.children
        .iter()
        .filter_map(|c| match c {
            HickNode::Tag(t) if t.name == "copy" || t.name == "cut" => Some(t),
            _ => None,
        })
        .find_map(|t| find(&t.children))
}

</storycode:copy>
<a id="f-e6f480388088-notebook_command"></a>
### `notebook_command`

Python source stays data while crossing the executor's command boundary.
Hex avoids shell interpolation and works with both POSIX and Windows shells.

<storycode:copy id="f-e6f480388088-notebook_command">
/// Python source stays data while crossing the executor's command boundary.
/// Hex avoids shell interpolation and works with both POSIX and Windows shells.
fn notebook_command(tag: &HickTag, source: String) -> String {
    if !matches!(
        tag.get_attribute("language"),
        Some("python" | "python3" | "py")
    ) {
        return source;
    }
    hick_lang::notebook::python_command(&source)
}

</storycode:copy>
<a id="f-e6f480388088-command_text"></a>
### `command_text`

<storycode:copy id="f-e6f480388088-command_text">
fn command_text(tag: &HickTag) -> String {
    fn collect(nodes: &[HickNode], out: &mut String) {
        for node in nodes {
            match node {
                HickNode::Text(t, _) => out.push_str(t),
                // `ingested` is what the cell PRODUCED, not what it runs.
                // It is the reason the command has to be wrapped in a
                // `<hick:copy>` child: forty file bodies sitting as siblings
                // of ambient command text would otherwise be shipped to the
                // shell. See
                // `docs/specs/freeform/owning-what-a-scaffolder-wrote.md`.
                HickNode::Tag(t)
                    if t.name == "expect"
                        || t.name == "capture"
                        || t.name == "ingested"
                        || t.name == "sample" => {}
                HickNode::Tag(t) => collect(&t.children, out),
            }
        }
    }
    let mut out = String::new();
    collect(&tag.children, &mut out);
    out
}

</storycode:copy>
<a id="f-e6f480388088-scan_copy_paste"></a>
### `scan_copy_paste`

<storycode:copy id="f-e6f480388088-scan_copy_paste">
fn scan_copy_paste(nodes: &[HickNode], copies: &mut Vec<String>, pastes: &mut Vec<String>) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            match tag.name.as_str() {
                "copy" => {
                    if let Some(id) = tag.get_attribute("id") {
                        copies.push(id.to_string());
                    }
                }
                "paste" => {
                    if let Some(select) = tag.get_attribute("select") {
                        pastes.push(select.to_string());
                    }
                }
                // An ingested scaffold's file bodies are ordinary document
                // bytes, not fragments this cell produces or consumes: a
                // `<hick:paste>`-looking string inside forty files of
                // somebody else's code must not become a pipeline edge.
                "ingested" => {}
                _ => {
                    scan_copy_paste(&tag.children, copies, pastes);
                }
            }
        }
    }
}

</storycode:copy>
<a id="f-e6f480388088-parse_volume_declaration"></a>
### `parse_volume_declaration`

Parse a `<hick:volume>` tag into a `VolumeDeclaration`.

<storycode:copy id="f-e6f480388088-parse_volume_declaration">
/// Parse a `<hick:volume>` tag into a `VolumeDeclaration`.
fn parse_volume_declaration(tag: &HickTag) -> Option<VolumeDeclaration> {
    let name = tag.get_attribute("name")?.to_string();

    let input = tag.get_attribute("input").map(|s| s.to_string());
    let output = tag.get_attribute("output").map(|s| s.to_string());

    let kind = match (input, output) {
        (Some(inp), Some(out)) => VolumeKind::InputOutput {
            input: inp,
            output: out,
        },
        (Some(path), None) => VolumeKind::Input { path },
        (None, Some(path)) => VolumeKind::Output { path },
        (None, None) => VolumeKind::Ephemeral,
    };

    // Parse <hick:allow> children for access rules
    let mut access_rules = Vec::new();
    for child in &tag.children {
        if let HickNode::Tag(child_tag) = child
            && child_tag.name == "allow"
            && let Some(container) = child_tag.get_attribute("container")
        {
            let container = container.to_string();
            if let Some(pattern) = child_tag.get_attribute("read") {
                access_rules.push(VolumeAccessRule {
                    container: container.clone(),
                    access: VolumeAccess::Read(pattern.to_string()),
                });
            }
            if let Some(pattern) = child_tag.get_attribute("write") {
                access_rules.push(VolumeAccessRule {
                    container: container.clone(),
                    access: VolumeAccess::Write(pattern.to_string()),
                });
            }
        }
    }

    Some(VolumeDeclaration {
        name,
        kind,
        access_rules,
    })
}

</storycode:copy>
<a id="f-e6f480388088-collect_nested_execs"></a>
### `collect_nested_execs`

<storycode:copy id="f-e6f480388088-collect_nested_execs">
fn collect_nested_execs(
    tag: &HickTag,
    execs: &mut Vec<ExecInfo>,
    index: &mut usize,
) -> Result<(), DagValidationError> {
    for child in &tag.children {
        if let HickNode::Tag(child_tag) = child {
            if child_tag.name == "exec" {
                execs.push(extract_exec_info(child_tag, *index)?);
                *index += 1;
            } else if child_tag.name == "script" {
                execs.push(extract_script_info(child_tag, *index)?);
                *index += 1;
            } else if child_tag.name == "agent" {
                // A nested agent cell keeps counting ordinals from the ones
                // already collected, so `AgentKey::Ordinal` stays document
                // order regardless of nesting.
                let ordinal = execs.iter().filter(|e| e.is_agent()).count();
                execs.push(extract_agent_info(child_tag, *index, ordinal)?);
                *index += 1;
            } else {
                collect_nested_execs(child_tag, execs, index)?;
            }
        }
    }
    Ok(())
}

</storycode:copy>
<a id="f-e6f480388088-validate_dag"></a>
### `validate_dag`

<storycode:copy id="f-e6f480388088-validate_dag">
// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn validate_dag(dag: &FlowDag) -> Result<(), DagValidationError> {
    // Check for cycles using Kahn's algorithm
    let mut in_degree: HashMap<ExecId, usize> = HashMap::new();
    let mut adj: HashMap<ExecId, Vec<ExecId>> = HashMap::new();

    for exec in &dag.execs {
        in_degree.entry(exec.id).or_insert(0);
        adj.entry(exec.id).or_default();
    }
    for edge in &dag.edges {
        *in_degree.entry(edge.to).or_insert(0) += 1;
        adj.entry(edge.from).or_default().push(edge.to);
    }

    let mut queue: VecDeque<ExecId> = in_degree
        .iter()
        .filter(|&(_, d)| *d == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut visited = 0usize;
    while let Some(id) = queue.pop_front() {
        visited += 1;
        if let Some(neighbors) = adj.get(&id) {
            for &neighbor in neighbors {
                if let Some(d) = in_degree.get_mut(&neighbor) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }

    if visited != dag.execs.len() {
        // Find a node that's part of a cycle
        let cycle_node = in_degree
            .iter()
            .find(|&(_, d)| *d > 0)
            .map(|(&id, _)| id)
            .unwrap_or(ExecId(0));
        return Err(DagValidationError::Cycle(cycle_node));
    }

    Ok(())
}

</storycode:copy>
<a id="f-e6f480388088-tests"></a>
### `tests`

<storycode:copy id="f-e6f480388088-tests">
// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "dag/tests/tests.rs"]
mod tests;
</storycode:copy>

### Assemble `crates/hick-exec/src/dag.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-exec/src/dag.rs"><storycode:paste select="#f-e6f480388088-ExecId"/><storycode:paste select="#f-e6f480388088-impl-fmt-Display-for-ExecId"/><storycode:paste select="#f-e6f480388088-DagEdge"/><storycode:paste select="#f-e6f480388088-DependencyReason"/><storycode:paste select="#f-e6f480388088-impl-fmt-Display-for-DependencyReason"/><storycode:paste select="#f-e6f480388088-AGENT_CONTAINER_PREFIX"/><storycode:paste select="#f-e6f480388088-agent_container_name"/><storycode:paste select="#f-e6f480388088-is_agent_container"/><storycode:paste select="#f-e6f480388088-AgentCell"/><storycode:paste select="#f-e6f480388088-impl-AgentCell"/><storycode:paste select="#f-e6f480388088-AgentKey"/><storycode:paste select="#f-e6f480388088-impl-fmt-Display-for-AgentKey"/><storycode:paste select="#f-e6f480388088-ExecInfo"/><storycode:paste select="#f-e6f480388088-impl-ExecInfo"/><storycode:paste select="#f-e6f480388088-FlowDag"/><storycode:paste select="#f-e6f480388088-impl-FlowDag"/><storycode:paste select="#f-e6f480388088-DagValidationError"/><storycode:paste select="#f-e6f480388088-build_dag"/><storycode:paste select="#f-e6f480388088-parse_freeze"/><storycode:paste select="#f-e6f480388088-parse_timeout"/><storycode:paste select="#f-e6f480388088-extract_exec_info"/><storycode:paste select="#f-e6f480388088-extract_script_info"/><storycode:paste select="#f-e6f480388088-extract_agent_info"/><storycode:paste select="#f-e6f480388088-paste_inside_command"/><storycode:paste select="#f-e6f480388088-notebook_command"/><storycode:paste select="#f-e6f480388088-command_text"/><storycode:paste select="#f-e6f480388088-scan_copy_paste"/><storycode:paste select="#f-e6f480388088-parse_volume_declaration"/><storycode:paste select="#f-e6f480388088-collect_nested_execs"/><storycode:paste select="#f-e6f480388088-validate_dag"/><storycode:paste select="#f-e6f480388088-tests"/></storycode:file>

</storycode:doc>
