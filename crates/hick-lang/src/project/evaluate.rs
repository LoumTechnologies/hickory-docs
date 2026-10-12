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
    pub exclude: BTreeSet<String>,
}
impl Default for ProjectInventory {
    fn default() -> Self {
        // Language-defined non-source metadata; arbitrary source exclusions
        // still have to be declared by the Markdown project.
        Self {
            documents: BTreeSet::new(),
            exclude: [".git/", ".hg/", ".hick-cache/", ".hick-journal/"]
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
}
#[derive(Clone)]
struct Operation {
    id: String,
    after: BTreeSet<String>,
    document: String,
    tag: HickTag,
    bom: usize,
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
        let paths = if op.tag.name == "patch" {
            let (diff, origins) = text_body(&op.tag, &op.document, op.bom, &copies)?;
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
            let (text, mut origins) = text_body(&op.tag, &op.document, op.bom, &copies)?;
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
    Ok(EvaluatedProject {
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
        return Ok(());
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
            "project" => (),
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
                let id = required(tag, "id")?.to_owned();
                if copies
                    .insert(id.clone(), (tag.clone(), path.into(), bom))
                    .is_some()
                {
                    return Err(format!("duplicate copy id: {id}"));
                }
            }
            "file" | "entry" | "patch" => {
                let id = tag
                    .get_attribute("id")
                    .map(str::to_owned)
                    .unwrap_or_else(|| {
                        format!(
                            "{path}:{}",
                            tag.source_span.map_or(tag.source_line, |s| s.start)
                        )
                    });
                let after = tag
                    .get_attribute("after")
                    .unwrap_or("")
                    .split_whitespace()
                    .map(|s| s.trim_start_matches('#').to_owned())
                    .collect();
                let op = Operation {
                    id: id.clone(),
                    after,
                    document: path.into(),
                    tag: tag.clone(),
                    bom,
                };
                if operations.insert(id.clone(), op).is_some() {
                    return Err(format!("duplicate construction id: {id}"));
                }
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
) -> Result<(String, Vec<ByteOrigin>), String> {
    body_inner(tag, document, bom, copies, &mut BTreeSet::new())
}
fn body_inner(
    tag: &HickTag,
    document: &str,
    bom: usize,
    copies: &BTreeMap<String, (HickTag, String, usize)>,
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
                let id = required(paste, "select")?
                    .strip_prefix('#')
                    .ok_or("paste requires #copy selector")?;
                if !visiting.insert(id.to_owned()) {
                    return Err(format!("copy cycle at {id}"));
                }
                let (copy, document, bom) = copies
                    .get(id)
                    .ok_or_else(|| format!("missing copy: {id}"))?;
                let (text, mappings) = body_inner(copy, document, *bom, copies, visiting)?;
                let offset = output.len();
                output.push_str(&text);
                origins.extend(mappings.into_iter().map(|mut origin| {
                    origin.output.0 += offset;
                    origin.output.1 += offset;
                    origin
                }));
                visiting.remove(id);
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
        "patch" => &["id", "after"],
        "copy" => &["id"],
        "paste" => &["select"],
        "include" => &["file"],
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
