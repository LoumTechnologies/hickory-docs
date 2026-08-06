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

impl fmt::Display for ExecId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exec[{}]", self.0)
    }
}

/// An edge in the DAG representing a dependency.
#[derive(Debug, Clone)]
pub struct DagEdge {
    pub from: ExecId,
    pub to: ExecId,
    pub reason: DependencyReason,
}

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
}

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
        }
    }
}

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
    /// Path to the WASM toolchain directory (only for script blocks).
    pub toolchain: Option<String>,
}

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
}

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

    // Pass 1: Collect containers, volumes, execs, forks, attenuates
    let mut exec_index = 0usize;
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
                    let info = extract_exec_info(tag, exec_index);
                    execs.push(info);
                    exec_index += 1;
                }
                "script" => {
                    let info = extract_script_info(tag, exec_index);
                    execs.push(info);
                    exec_index += 1;
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
                    collect_nested_execs(tag, &mut execs, &mut exec_index);
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

// ---------------------------------------------------------------------------
// Extraction helpers
// ---------------------------------------------------------------------------

fn extract_exec_info(tag: &HickTag, index: usize) -> ExecInfo {
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

    let command = command_text(tag);

    // Collect non-command child tags (paste, val, etc.) as stdin sources.
    // Command text is extracted separately via command_text() above.
    // `<hick:expect>` children are verification metadata, never stdin.
    let stdin_children: Vec<HickNode> = tag
        .children
        .iter()
        .filter(|child| {
            matches!(child, HickNode::Tag(t)
                if t.name != "copy" && t.name != "cut" && t.name != "expect")
        })
        .cloned()
        .collect();

    ExecInfo {
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
    }
}

/// Extract script info from a `<hick:script>` tag.
///
/// Script blocks get a synthetic container name (`_script_N`) so they
/// participate in the DAG without creating container-state edges between
/// unrelated scripts. Each script is independent unless connected by
/// explicit volume or copy/paste dependencies.
fn extract_script_info(tag: &HickTag, index: usize) -> ExecInfo {
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

    ExecInfo {
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
    }
}

/// The command text of an exec/script tag: all text content EXCLUDING any
/// `<hick:expect>` subtree. Expectations sit inside the exec tag for locality
/// but are verification metadata, not part of the command.
fn command_text(tag: &HickTag) -> String {
    fn collect(nodes: &[HickNode], out: &mut String) {
        for node in nodes {
            match node {
                HickNode::Text(t, _) => out.push_str(t),
                HickNode::Tag(t) if t.name == "expect" => {}
                HickNode::Tag(t) => collect(&t.children, out),
            }
        }
    }
    let mut out = String::new();
    collect(&tag.children, &mut out);
    out
}

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
                _ => {
                    scan_copy_paste(&tag.children, copies, pastes);
                }
            }
        }
    }
}

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

fn collect_nested_execs(tag: &HickTag, execs: &mut Vec<ExecInfo>, index: &mut usize) {
    for child in &tag.children {
        if let HickNode::Tag(child_tag) = child {
            if child_tag.name == "exec" {
                let info = extract_exec_info(child_tag, *index);
                execs.push(info);
                *index += 1;
            } else if child_tag.name == "script" {
                let info = extract_script_info(child_tag, *index);
                execs.push(info);
                *index += 1;
            } else {
                collect_nested_execs(child_tag, execs, index);
            }
        }
    }
}

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_and_build(src: &str) -> Result<FlowDag, DagValidationError> {
        let doc = hick_lang::parse(src).expect("parse failed");
        build_dag(&doc)
    }

    #[test]
    fn single_exec_no_deps() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
echo hello
</hick:exec>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        assert_eq!(dag.execs.len(), 1);
        assert_eq!(dag.edges.len(), 0);
        assert_eq!(dag.roots.len(), 1);
    }

    #[test]
    fn sequential_execs_same_container() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
apk add curl
</hick:exec>
<hick:exec container="demo">
curl --version
</hick:exec>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        assert_eq!(dag.execs.len(), 2);
        assert_eq!(dag.edges.len(), 1);
        assert_eq!(dag.edges[0].from, ExecId(0));
        assert_eq!(dag.edges[0].to, ExecId(1));
        assert!(matches!(
            dag.edges[0].reason,
            DependencyReason::ContainerState { .. }
        ));
    }

    #[test]
    fn parallel_execs_different_containers() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="a" image="alpine" />
<hick:container name="b" image="alpine" />
<hick:exec container="a">
echo a
</hick:exec>
<hick:exec container="b">
echo b
</hick:exec>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        assert_eq!(dag.execs.len(), 2);
        assert_eq!(dag.edges.len(), 0);
        assert_eq!(dag.roots.len(), 2);
    }

    #[test]
    fn volume_dependency() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="writer" image="alpine" />
<hick:container name="reader" image="alpine" />
<hick:volume name="shared" />
<hick:exec container="writer" mount="shared:/output">
echo data > /output/file.txt
</hick:exec>
<hick:exec container="reader" mount="shared:/input">
cat /input/file.txt
</hick:exec>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        assert_eq!(dag.execs.len(), 2);
        assert!(!dag.edges.is_empty());
        let volume_edge = dag
            .edges
            .iter()
            .find(|e| matches!(&e.reason, DependencyReason::VolumeFlow { .. }));
        assert!(volume_edge.is_some());
    }

    #[test]
    fn fork_dependency() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="base" image="ubuntu:22.04" />
<hick:exec container="base">
apt-get update
</hick:exec>
<hick:fork from="base" to="analyzer" />
<hick:exec container="analyzer">
./run-analysis
</hick:exec>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        assert_eq!(dag.execs.len(), 2);
        let fork_edge = dag
            .edges
            .iter()
            .find(|e| matches!(&e.reason, DependencyReason::Fork { .. }));
        assert!(fork_edge.is_some());
    }

    #[test]
    fn topological_order() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
step 1
</hick:exec>
<hick:exec container="demo">
step 2
</hick:exec>
<hick:exec container="demo">
step 3
</hick:exec>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        let order = dag.topological_order();
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], ExecId(0));
        assert_eq!(order[1], ExecId(1));
        assert_eq!(order[2], ExecId(2));
    }

    #[test]
    fn nested_exec_in_file() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="out.md">
<hick:exec container="demo" image="alpine">
cat /etc/alpine-release
</hick:exec>
</hick:file>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        assert_eq!(dag.execs.len(), 1);
        assert_eq!(dag.execs[0].container, "demo");
    }

    #[test]
    fn volume_with_explicit_access_rules() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="scaffolder" image="alpine" />
<hick:container name="linter" image="alpine" />
<hick:volume name="project" output="src/MyApi/">
  <hick:allow container="scaffolder" write="**" />
  <hick:allow container="linter" read="**" />
</hick:volume>
<hick:exec container="scaffolder" mount="project:/out">echo scaffold</hick:exec>
<hick:exec container="linter" mount="project:/in">echo lint</hick:exec>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        assert_eq!(dag.execs.len(), 2);

        // scaffolder is writer, linter is reader → writer→reader edge
        let volume_edge = dag
            .edges
            .iter()
            .find(|e| matches!(&e.reason, DependencyReason::VolumeFlow { .. }));
        assert!(
            volume_edge.is_some(),
            "expected VolumeFlow edge from writer to reader"
        );
        let ve = volume_edge.unwrap();
        assert_eq!(ve.from, ExecId(0)); // scaffolder
        assert_eq!(ve.to, ExecId(1)); // linter

        // Volume declaration is stored
        assert!(dag.volumes.contains_key("project"));
        let vol = &dag.volumes["project"];
        assert_eq!(vol.access_rules.len(), 2);
        assert!(matches!(vol.kind, VolumeKind::Output { .. }));
    }

    #[test]
    fn volume_backward_compat_no_access_rules() {
        // Volume with no <allow> children uses heuristic: first mounter = writer
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="a" image="alpine" />
<hick:container name="b" image="alpine" />
<hick:volume name="scratch" />
<hick:exec container="a" mount="scratch:/data">echo write</hick:exec>
<hick:exec container="b" mount="scratch:/data">echo read</hick:exec>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        let volume_edge = dag
            .edges
            .iter()
            .find(|e| matches!(&e.reason, DependencyReason::VolumeFlow { .. }));
        assert!(
            volume_edge.is_some(),
            "expected VolumeFlow edge via heuristic"
        );
    }

    #[test]
    fn volume_declaration_parsing() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:volume name="input-vol" input="." />
<hick:volume name="output-vol" output="dist/" />
<hick:volume name="both-vol" input="." output="." />
<hick:volume name="ephemeral" />
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();
        assert_eq!(dag.volumes.len(), 4);

        assert!(
            matches!(dag.volumes["input-vol"].kind, VolumeKind::Input { ref path } if path == ".")
        );
        assert!(
            matches!(dag.volumes["output-vol"].kind, VolumeKind::Output { ref path } if path == "dist/")
        );
        assert!(matches!(
            dag.volumes["both-vol"].kind,
            VolumeKind::InputOutput { .. }
        ));
        assert!(matches!(
            dag.volumes["ephemeral"].kind,
            VolumeKind::Ephemeral
        ));
    }

    #[test]
    fn volume_multiple_writers_sequential() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="step1" image="alpine" />
<hick:container name="step2" image="alpine" />
<hick:container name="consumer" image="alpine" />
<hick:volume name="shared">
  <hick:allow container="step1" write="**" />
  <hick:allow container="step2" write="**" />
  <hick:allow container="consumer" read="**" />
</hick:volume>
<hick:exec container="step1" mount="shared:/out">echo step1</hick:exec>
<hick:exec container="step2" mount="shared:/out">echo step2</hick:exec>
<hick:exec container="consumer" mount="shared:/in">echo consume</hick:exec>
</hick:doc>"#;
        let dag = parse_and_build(src).unwrap();

        // Both writers should have edges to consumer
        let writer_to_consumer: Vec<_> = dag
            .edges
            .iter()
            .filter(|e| {
                e.to == ExecId(2) && matches!(&e.reason, DependencyReason::VolumeFlow { .. })
            })
            .collect();
        assert_eq!(
            writer_to_consumer.len(),
            2,
            "both writers should have edge to consumer"
        );

        // Writers should be sequentially ordered
        let writer_seq: Vec<_> = dag
            .edges
            .iter()
            .filter(|e| {
                e.from == ExecId(0)
                    && e.to == ExecId(1)
                    && matches!(&e.reason, DependencyReason::VolumeFlow { .. })
            })
            .collect();
        assert_eq!(
            writer_seq.len(),
            1,
            "writers should be sequentially ordered"
        );
    }
}
