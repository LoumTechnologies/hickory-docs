use super::{
    ByteOrigin, EntryKind, ProjectEntry, ProjectTree, apply_project_patch, validate_project_path,
};
use crate::{HickNode, HickTag, parse};
use std::collections::{BTreeMap, BTreeSet};

/// Source inventory is carried by the entry Markdown. Exclusions are explicit
/// paths or directory prefixes, never implicit source extensions or gitignore.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectInventory {
    pub documents: BTreeSet<String>,
    /// Inputs supplying fragments through upstream edges remain immutable.
    #[cfg_attr(feature = "serde", serde(default))]
    pub read_only: BTreeSet<String>,
    pub exclude: BTreeSet<String>,
}
impl Default for ProjectInventory {
    fn default() -> Self {
        // Language-defined non-source metadata; arbitrary source exclusions
        // still have to be declared by the Markdown project.
        Self {
            documents: BTreeSet::new(),
            read_only: BTreeSet::new(),
            exclude: [
                ".git/",
                ".hg/",
                ".hick-cache/",
                ".hick-journal/",
                "sessions/",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        }
    }
}
impl ProjectInventory {
    pub fn contains(&self, path: &str) -> bool {
        !self.documents.contains(path)
            && !self.exclude.iter().any(|excluded| {
                if excluded.ends_with('/') {
                    path == excluded.trim_end_matches('/') || path.starts_with(excluded)
                } else {
                    path == excluded
                }
            })
    }
    pub fn select(&self, tree: &ProjectTree) -> Result<ProjectTree, String> {
        let mut selected = ProjectTree::default();
        for (path, entry) in tree.entries() {
            if self.contains(path) {
                selected.insert(path, entry.as_ref().clone())?;
            }
        }
        Ok(selected)
    }
}

#[derive(Debug, Clone)]
pub struct EvaluatedProject {
    /// Atomic Old endpoint. Empty for maintained source generation.
    pub old: ProjectTree,
    /// Atomic New endpoint. Always the completed scope, never a local patch.
    pub new: ProjectTree,
    pub inventory: ProjectInventory,
    /// Explicit advanced inspection only; default services use old/new above.
    pub local_states: BTreeMap<String, (ProjectTree, ProjectTree)>,
    /// Explicit atomic scopes; their New endpoint includes every nested act.
    pub scopes: BTreeMap<String, (ProjectTree, ProjectTree)>,
    pub recordings: Vec<super::recorded::RecordedCell>,
}
#[derive(Clone)]
struct Operation {
    id: String,
    after: BTreeSet<String>,
    document: String,
    tag: HickTag,
    bom: usize,
    members: BTreeSet<String>,
}

/// Evaluate a Markdown closure from captured input in memory. All source bytes
/// must be supplied; this function cannot read files or spawn a process.
pub fn evaluate_project(
    entry: &str,
    sources: &BTreeMap<String, String>,
    input: &ProjectTree,
) -> Result<EvaluatedProject, String> {
    validate_project_path(entry)?;
    let mut entry = entry.to_owned();
    let mut routes = BTreeSet::new();
    loop {
        if !routes.insert(entry.clone()) {
            return Err(format!("project entry routing cycle at {entry}"));
        }
        let source = sources
            .get(&entry)
            .ok_or_else(|| format!("Markdown closure missing {entry}"))?;
        let doc = parse(source).map_err(|e| e.to_string())?;
        let projects = doc.find_tags("project");
        if projects.len() != 1 {
            return Err(format!(
                "{entry} must declare exactly one project or project entry route"
            ));
        }
        let Some(target) = projects[0].get_attribute("entry") else {
            break;
        };
        if projects[0]
            .attributes
            .iter()
            .any(|(name, _)| name != "entry")
        {
            return Err("a project entry route accepts only entry".into());
        }
        entry = relative_document(&entry, target)?;
    }
    let mut inventory = ProjectInventory::default();
    let mut operations = BTreeMap::new();
    let mut copies = BTreeMap::new();
    let mut visiting = BTreeSet::new();
    visit(
        &entry,
        sources,
        &mut inventory,
        &mut operations,
        &mut copies,
        &mut visiting,
        &entry,
    )?;
    for routed in routes {
        if !inventory.documents.contains(&routed) {
            return Err(format!(
                "entry route {routed} is not part of the declared Markdown closure"
            ));
        }
    }
    let recordings = super::recorded::load(sources, &inventory.documents)?;
    let mut result = input.clone();
    let mut completed: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut writers: BTreeMap<String, String> = BTreeMap::new();
    let mut local_states = BTreeMap::new();
    let mut deltas: Vec<(String, EntryDelta)> = vec![];
    while completed.len() < operations.len() {
        let ready = operations
            .values()
            .find(|op| {
                !completed.contains_key(&op.id)
                    && op.after.iter().all(|id| completed.contains_key(id))
            })
            .cloned();
        let Some(op) = ready else {
            let pending: Vec<_> = operations
                .values()
                .filter(|op| !completed.contains_key(&op.id))
                .map(|op| op.id.as_str())
                .collect();
            return Err(format!(
                "missing predecessor or dependency cycle: {}",
                pending.join(", ")
            ));
        };
        let mut ancestors = op.after.clone();
        for id in &op.after {
            ancestors.extend(completed[id].iter().cloned());
        }
        let mut before = input.clone();
        for (id, delta) in &deltas {
            if ancestors.contains(id) {
                apply_delta(&mut before, delta)?;
            }
        }
        let mut local = before.clone();
        let paths = if matches!(op.tag.name.as_str(), "scope" | "retained-generator") {
            BTreeSet::new()
        } else if op.tag.name == "patch" {
            let (diff, origins) = text_body(&op.tag, &op.document, op.bom, &copies, &recordings)?;
            // A patch is a verbatim body: no dedenting or copy substitution.
            let offset = origins.first().map_or(0, |o| o.source.0);
            let patch = apply_project_patch(&local, &diff, &op.document, offset).map_err(|error|format!("invalid or unordered input for patch {}: {error}; declare its generated-file predecessors with after",op.id))?;
            local = patch.after;
            patch.paths
        } else {
            let path = required(&op.tag, "path")?;
            validate_project_path(path)?;
            if local.get(path).is_some() {
                return Err(format!(
                    "construction {0} would overwrite {path}; use a patch",
                    op.id
                ));
            }
            let (text, mut origins) =
                text_body(&op.tag, &op.document, op.bom, &copies, &recordings)?;
            let kind = match op.tag.get_attribute("kind").unwrap_or("file") {
                "file" => EntryKind::File,
                "symlink" => EntryKind::Symlink,
                other => return Err(format!("unsupported project entry kind: {other}")),
            };
            let bytes = match op.tag.get_attribute("encoding").unwrap_or("utf8") {
                "utf8" => text.into_bytes(),
                "hex" => {
                    origins.clear(); // Encoded bytes have no byte-equal reverse map.
                    decode_hex(&text)?
                }
                other => return Err(format!("unsupported entry encoding: {other}")),
            };
            if kind == EntryKind::Symlink {
                std::str::from_utf8(&bytes).map_err(|_| "symlink target requires UTF-8")?;
            }
            let executable = match op.tag.get_attribute("executable").unwrap_or("false") {
                "false" => false,
                "true" if kind == EntryKind::File => true,
                _ => return Err("executable must be true/false on a regular file".into()),
            };
            local.insert(
                path,
                ProjectEntry {
                    bytes: bytes.into(),
                    kind,
                    executable,
                    origins,
                },
            )?;
            BTreeSet::from([path.to_owned()])
        };
        let delta: EntryDelta = paths
            .iter()
            .map(|path| (path.clone(), local.get(path).cloned()))
            .collect();
        for path in paths {
            if !inventory.contains(&path) {
                return Err(format!(
                    "construction output {path} overlaps the Markdown closure or excluded inventory"
                ));
            }
            if let Some(writer) = writers.get(&path)
                && !ancestors.contains(writer)
            {
                return Err(format!(
                    "unordered transformations of {path}: {writer} and {} (declare after)",
                    op.id
                ));
            }
            for document in &inventory.documents {
                super::tree::validate_path_collision(document, &path)?;
            }
            writers.insert(path, op.id.clone());
        }
        apply_delta(&mut result, &delta)?;
        local_states.insert(op.id.clone(), (before, local));
        deltas.push((op.id.clone(), delta));
        completed.insert(op.id, ancestors);
    }
    let mut scopes = BTreeMap::new();
    for op in operations.values().filter(|op| op.tag.name == "scope") {
        let mut old = input.clone();
        for (id, delta) in &deltas {
            if completed[&op.id].contains(id) && !op.members.contains(id) {
                apply_delta(&mut old, delta)?;
            }
        }
        scopes.insert(op.id.clone(), (old, local_states[&op.id].1.clone()));
    }
    Ok(EvaluatedProject {
        scopes,
        recordings: recordings.into_values().collect(),
        old: input.clone(),
        new: result,
        inventory,
        local_states,
    })
}

fn visit(
    path: &str,
    sources: &BTreeMap<String, String>,
    inventory: &mut ProjectInventory,
    operations: &mut BTreeMap<String, Operation>,
    copies: &mut BTreeMap<String, (HickTag, String, usize)>,
    visiting: &mut BTreeSet<String>,
    project_entry: &str,
) -> Result<(), String> {
    let entry = path == project_entry;
    if visiting.contains(path) {
        return Err(format!("Markdown source cycle at {path}"));
    }
    if inventory.documents.contains(path) {
        if !inventory.read_only.remove(path) {
            return Ok(());
        }
        copies.retain(|_, (_, document, _)| document != path);
    }
    let source = sources
        .get(path)
        .ok_or_else(|| format!("Markdown closure missing {path}"))?;
    let doc = parse(source).map_err(|e| format!("{path}: {e}"))?;
    let bom = usize::from(source.starts_with('\u{feff}')) * 3;
    visiting.insert(path.to_owned());
    for existing in &inventory.documents {
        super::tree::validate_path_collision(existing, path)?;
    }
    inventory.documents.insert(path.to_owned());
    if let Some(woven) = &doc.weave_path {
        let woven = path
            .rsplit_once('/')
            .map_or(woven.clone(), |(dir, _)| format!("{dir}/{woven}"));
        validate_project_path(&woven)?;
        if woven != path {
            inventory.exclude.insert(woven);
        }
    }
    let projects = doc.find_tags("project");
    if entry && projects.len() != 1 {
        return Err("entry Markdown must declare exactly one project inventory".into());
    }
    if !entry && projects.iter().any(|p| p.get_attribute("entry").is_none()) {
        return Err(format!(
            "only entry Markdown declares the project inventory: {path}"
        ));
    }
    for project in projects {
        if let Some(target) = project.get_attribute("entry") {
            if project.attributes.iter().any(|(name, _)| name != "entry")
                || relative_document(path, target)? != project_entry
            {
                return Err(format!(
                    "{path} must route to this evaluated project's entry: {project_entry}"
                ));
            }
            continue;
        }
        for excluded in project
            .get_attribute("exclude")
            .unwrap_or("")
            .split_whitespace()
        {
            validate_project_path(excluded.trim_end_matches('/'))?;
            inventory.exclude.insert(excluded.to_owned());
        }
        for document in project
            .get_attribute("sources")
            .unwrap_or("")
            .split_whitespace()
        {
            let document = relative_document(path, document)?;
            visit(
                &document,
                sources,
                inventory,
                operations,
                copies,
                visiting,
                project_entry,
            )?;
        }
    }
    for node in &doc.nodes {
        let HickNode::Tag(tag) = node else { continue };
        check_attributes(tag)?;
        match tag.name.as_str() {
            "project" | "container" | "volume" | "allow" | "private" => (),
            "agent" => (),
            "upstream" => {
                let name = relative_document(path, required(tag, "file")?)?;
                upstream_fragments(&name, sources, inventory, copies, &mut BTreeSet::new())?;
            }
            "include" => {
                let document = relative_document(path, required(tag, "file")?)?;
                visit(
                    &document,
                    sources,
                    inventory,
                    operations,
                    copies,
                    visiting,
                    project_entry,
                )?;
            }
            "copy" => {
                if tag.get_attribute("id").is_none() && tag.get_attribute("class").is_none() {
                    return Err("Copy requires id or class".into());
                }
                let id = tag
                    .get_attribute("id")
                    .map(str::to_owned)
                    .unwrap_or_else(|| {
                        format!(
                            "{path}:copy:{}",
                            tag.source_span.map_or(tag.source_line, |s| s.start)
                        )
                    });
                let mut copy = tag.clone();
                if doc
                    .nodes
                    .iter()
                    .any(|n| matches!(n,HickNode::Tag(t) if t.name == "private"))
                {
                    copy.attributes
                        .push(("__project-private".into(), "true".into()));
                }
                if copies
                    .insert(id.clone(), (copy, path.into(), bom))
                    .is_some()
                {
                    return Err(format!("duplicate copy id: {id}"));
                }
            }
            "file" | "entry" | "patch" | "scope" | "exec" => {
                collect_operation(tag, path, bom, &BTreeSet::new(), operations)?;
            }
            // Never report an incomplete project as generated successfully.
            other => {
                return Err(format!(
                    "{path}: <{other}> is unsupported in pure project generation; execution or workspace resolution is required"
                ));
            }
        }
    }
    visiting.remove(path);
    Ok(())
}
fn relative_document(parent: &str, child: &str) -> Result<String, String> {
    validate_project_path(child)?;
    let path = parent
        .rsplit_once('/')
        .map_or(child.to_owned(), |(dir, _)| format!("{dir}/{child}"));
    validate_project_path(&path)?;
    if !path.ends_with(".md") {
        return Err(format!("source closure requires Markdown: {path}"));
    }
    Ok(path)
}
fn required<'a>(tag: &'a HickTag, attr: &str) -> Result<&'a str, String> {
    tag.get_attribute(attr)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("{} requires {attr}", tag.name))
}
fn text_body(
    tag: &HickTag,
    document: &str,
    bom: usize,
    copies: &BTreeMap<String, (HickTag, String, usize)>,
    recordings: &super::recorded::Recordings,
) -> Result<(String, Vec<ByteOrigin>), String> {
    body_inner(tag, document, bom, copies, recordings, &mut BTreeSet::new())
}
fn body_inner(
    tag: &HickTag,
    document: &str,
    bom: usize,
    copies: &BTreeMap<String, (HickTag, String, usize)>,
    recordings: &super::recorded::Recordings,
    visiting: &mut BTreeSet<String>,
) -> Result<(String, Vec<ByteOrigin>), String> {
    let mut output = String::new();
    let mut origins = vec![];
    for (i, node) in tag.children.iter().enumerate() {
        match node {
            HickNode::Text(text, span) => {
                let (text, span) = if i == 0 {
                    crate::strip_opening_break(text, span.as_ref())
                } else {
                    (text.as_str(), *span)
                };
                let mut source = span
                    .ok_or("construction body requires a source span")?
                    .start
                    + bom;
                for line in text.split_inclusive('\n') {
                    let removed = if matches!(tag.name.as_str(), "patch" | "entry") {
                        0
                    } else {
                        line.bytes()
                            .take_while(|b| *b == b' ')
                            .count()
                            .min(tag.source_column)
                    };
                    let text = &line[removed..];
                    let start = output.len();
                    output.push_str(text);
                    if !text.is_empty() {
                        origins.push(ByteOrigin {
                            output: (start, output.len()),
                            document: document.into(),
                            source: (source + removed, source + line.len()),
                        });
                    }
                    source += line.len();
                }
            }
            HickNode::Tag(paste) if paste.name == "paste" => {
                check_attributes(paste)?;
                let selector = required(paste, "select")?;
                let selected: Vec<_> = if let Some(id) = selector.strip_prefix('#') {
                    vec![(
                        id.to_owned(),
                        copies
                            .get(id)
                            .ok_or_else(|| format!("Missing copy: {id}"))?,
                    )]
                } else if let Some(class) = selector.strip_prefix('.') {
                    let current_document = document;
                    let mut matches: Vec<_> = copies
                        .iter()
                        .filter(|(_, (tag, source_document, _))| {
                            (source_document == document
                                || tag.get_attribute("__project-private").is_none())
                                && tag
                                    .get_attribute("class")
                                    .unwrap_or("")
                                    .split_whitespace()
                                    .any(|name| name == class)
                        })
                        .map(|(id, value)| (id.clone(), value))
                        .collect();
                    matches.sort_by_key(|(_, (tag, document, _))| {
                        (
                            document != current_document,
                            document.clone(),
                            tag.source_span.map_or(tag.source_line, |s| s.start),
                        )
                    });
                    if matches.is_empty() {
                        return Err(format!("Missing contributed class: {selector}"));
                    }
                    matches
                } else {
                    return Err("Paste requires #id or .class selector".into());
                };
                for (id, (copy, document, bom)) in selected {
                    if !visiting.insert(id.clone()) {
                        return Err(format!("Copy cycle at {id}"));
                    }
                    let (text, mappings) =
                        body_inner(copy, document, *bom, copies, recordings, visiting)?;
                    let offset = output.len();
                    output.push_str(&text);
                    origins.extend(mappings.into_iter().map(|mut origin| {
                        origin.output.0 += offset;
                        origin.output.1 += offset;
                        origin
                    }));
                    visiting.remove(&id);
                }
            }
            HickNode::Tag(cell) if matches!(cell.name.as_str(), "exec" | "agent") => {
                let (text, mappings) = super::recorded::render(cell, document, recordings)?;
                let offset = output.len();
                output.push_str(&text);
                origins.extend(mappings.into_iter().map(|mut origin| {
                    origin.output.0 += offset;
                    origin.output.1 += offset;
                    origin
                }));
            }
            HickNode::Tag(other) => {
                return Err(format!(
                    "{} body requires execution/resolution for {}",
                    tag.name, other.name
                ));
            }
        }
    }
    Ok((output, origins))
}
fn decode_hex(text: &str) -> Result<Vec<u8>, String> {
    let digits: String = text.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    if !digits.len().is_multiple_of(2) || !digits.is_ascii() {
        return Err("invalid hex entry".into());
    }
    (0..digits.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&digits[i..i + 2], 16).map_err(|_| "invalid hex entry".into()))
        .collect()
}

fn check_attributes(tag: &HickTag) -> Result<(), String> {
    let accepted: &[&str] = match tag.name.as_str() {
        "project" => &["sources", "exclude", "entry"],
        "file" => &["path", "id", "after", "language", "executable"],
        "entry" => &["path", "id", "after", "kind", "encoding", "executable"],
        "patch" | "scope" => &["id", "after"],
        "copy" => &["id", "class"],
        "paste" => &["select"],
        "include" | "upstream" => &["file"],
        _ => return Ok(()),
    };
    for (name, _) in &tag.attributes {
        if !accepted.contains(&name.as_str()) {
            return Err(format!(
                "{} attribute {name} is unsupported in pure project construction",
                tag.name
            ));
        }
    }
    Ok(())
}

type EntryDelta = BTreeMap<String, Option<std::sync::Arc<ProjectEntry>>>;
fn apply_delta(tree: &mut ProjectTree, delta: &EntryDelta) -> Result<(), String> {
    // Remove the act's old names before inserting new ones, including case-only
    // renames. This is virtual composition, never a filesystem transaction.
    for path in delta.keys() {
        if tree.get(path).is_some() {
            tree.remove(path)?;
        }
    }
    for (path, entry) in delta {
        if let Some(entry) = entry {
            tree.insert(path, entry.as_ref().clone())?;
        }
    }
    Ok(())
}

fn collect_operation(
    tag: &HickTag,
    document: &str,
    bom: usize,
    inherited: &BTreeSet<String>,
    operations: &mut BTreeMap<String, Operation>,
) -> Result<BTreeSet<String>, String> {
    check_attributes(tag)?;
    let mut owned;
    let tag = if tag.name == "exec" {
        owned = tag.clone();
        owned.name = "retained-generator".into();
        if let Some(ingested) = tag
            .child_tags()
            .find(|t| t.name == "ingested" && t.get_attribute("key").is_none())
        {
            let fingerprint = ingested
                .get_attribute("sha256")
                .ok_or("Owned ingested files require their run fingerprint")?;
            if fingerprint.len() != 64 || !fingerprint.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err("Owned ingest fingerprint must be SHA-256 hex".into());
            }
            owned.children = ingested.children.clone();
        } else if tag
            .child_tags()
            .any(|t| t.name == "ingested" && t.get_attribute("key").is_some())
        {
            owned.children.clear();
        } else {
            return Err(format!(
                "Standalone cell at line {} requires execution and explicit retained outputs",
                tag.source_line
            ));
        }

        &owned
    } else {
        tag
    };
    let id = tag
        .get_attribute("id")
        .map(str::to_owned)
        .unwrap_or_else(|| {
            format!(
                "{document}:{}",
                tag.source_span.map_or(tag.source_line, |s| s.start)
            )
        });
    if tag.name == "scope" && tag.get_attribute("id").is_none() {
        return Err("An explicit atomic scope requires id".into());
    }
    let mut after = inherited.clone();
    after.extend(
        tag.get_attribute("after")
            .unwrap_or("")
            .split_whitespace()
            .map(|s| s.trim_start_matches('#').to_owned()),
    );
    let mut members = BTreeSet::new();
    if matches!(tag.name.as_str(), "scope" | "retained-generator") {
        for node in &tag.children {
            let HickNode::Tag(child) = node else {
                continue;
            };
            if !matches!(child.name.as_str(), "scope" | "patch" | "file" | "entry") {
                return Err(format!(
                    "Atomic scope {id} cannot contain {}; declare contributors in the closure and reference their constructions with after",
                    child.name
                ));
            }
            members.extend(collect_operation(child, document, bom, &after, operations)?);
        }
        after.extend(members.iter().cloned());
    }
    let mut identities = members.clone();
    identities.insert(id.clone());
    if operations
        .insert(
            id.clone(),
            Operation {
                id: id.clone(),
                after,
                document: document.into(),
                tag: tag.clone(),
                bom,
                members,
            },
        )
        .is_some()
    {
        return Err(format!("Duplicate construction id: {id}"));
    }
    Ok(identities)
}

/// Read only fragment closure: no file or patch acts from upstream are imported.
fn upstream_fragments(
    path: &str,
    sources: &BTreeMap<String, String>,
    inventory: &mut ProjectInventory,
    copies: &mut BTreeMap<String, (HickTag, String, usize)>,
    visiting: &mut BTreeSet<String>,
) -> Result<(), String> {
    if !visiting.insert(path.into()) {
        return Err(format!("Upstream fragment cycle at {path}"));
    }
    if inventory.documents.contains(path) {
        visiting.remove(path);
        return Ok(());
    }
    let source = sources
        .get(path)
        .ok_or_else(|| format!("Markdown closure missing {path}"))?;
    let doc = parse(source).map_err(|e| e.to_string())?;
    for existing in &inventory.documents {
        super::tree::validate_path_collision(existing, path)?;
    }
    inventory.documents.insert(path.into());
    inventory.read_only.insert(path.into());
    if let Some(woven) = &doc.weave_path {
        let woven = relative_document(path, woven)?;
        if woven != path {
            inventory.exclude.insert(woven);
        }
    }
    let bom = usize::from(source.starts_with('\u{feff}')) * 3;
    let private = doc
        .nodes
        .iter()
        .any(|n| matches!(n,HickNode::Tag(t) if t.name=="private"));
    for tag in doc.all_tags() {
        if tag.name == "copy" {
            check_attributes(tag)?;
            let id = tag
                .get_attribute("id")
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    format!(
                        "{path}:copy:{}",
                        tag.source_span.map_or(tag.source_line, |s| s.start)
                    )
                });
            let mut copy = tag.clone();
            if private {
                copy.attributes
                    .push(("__project-private".into(), "true".into()));
            }
            if copies
                .insert(id.clone(), (copy, path.into(), bom))
                .is_some()
            {
                return Err(format!("Duplicate upstream copy: {id}"));
            }
        }
    }
    for tag in doc.tags() {
        let names: Vec<_> = match tag.name.as_str() {
            "upstream" | "include" => vec![required(tag, "file")?],
            "project" => tag
                .get_attribute("sources")
                .unwrap_or("")
                .split_whitespace()
                .collect(),
            _ => vec![],
        };
        for name in names {
            upstream_fragments(
                &relative_document(path, name)?,
                sources,
                inventory,
                copies,
                visiting,
            )?;
        }
    }
    visiting.remove(path);
    Ok(())
}
