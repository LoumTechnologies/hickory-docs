use super::{EntryKind, ProjectDifference, ProjectTree, evaluate_project};
use std::collections::{BTreeMap, BTreeSet};

/// Deterministic textual reconstruction of a successful A-to-B transition.
/// Kept apart from the actual session. Metadata/binary transitions refuse by
/// name until an exact supported transition operation exists for them.
pub fn transition_response(
    before: &ProjectTree,
    after: &ProjectTree,
    explanation: &str,
) -> Result<String, String> {
    let diff = unified_transition(before, after)?;
    let mut prefix = "hick".to_owned();
    let mut number = 0;
    while explanation.contains(&format!("<{prefix}:"))
        || explanation.contains(&format!("</{prefix}:"))
        || diff.contains(&format!("</{prefix}:patch>"))
    {
        number += 1;
        prefix = format!("hick{number}");
    }
    let patch = if diff.is_empty() {
        String::new()
    } else {
        format!("\n<{prefix}:patch id=\"completed\">\n{diff}</{prefix}:patch>\n")
    };
    let response = format!(
        "<{prefix}:doc xmlns:{prefix}=\"http://www.hickorydocs.com/1.0\">\n{explanation}\n\n<{prefix}:project/>\n{patch}</{prefix}:doc>\n"
    );
    let sources = BTreeMap::from([("response.md".into(), response.clone())]);
    let evaluated = evaluate_project("response.md", &sources, before)?;
    if !evaluated.new.compare(after).is_empty() {
        return Err("reconstructed response did not reproduce the completed state".into());
    }
    Ok(response)
}
pub fn unified_transition(before: &ProjectTree, after: &ProjectTree) -> Result<String, String> {
    let paths: BTreeSet<_> = before
        .entries()
        .keys()
        .chain(after.entries().keys())
        .collect();
    let mut diff = String::new();
    for path in paths {
        let old = before.get(path);
        let new = after.get(path);
        if matches!((old,new),(Some(a),Some(b)) if a.same_output(b)) {
            continue;
        }
        for entry in [old, new].into_iter().flatten() {
            if entry.kind != EntryKind::File
                || entry.executable != old.is_some_and(|o| o.executable)
            {
                return Err(format!("unsupported textual transition metadata: {path}"));
            }
        }
        let old_text = old
            .map(|e| std::str::from_utf8(&e.bytes))
            .transpose()
            .map_err(|_| format!("unsupported binary transition: {path}"))?
            .unwrap_or("");
        let new_text = new
            .map(|e| std::str::from_utf8(&e.bytes))
            .transpose()
            .map_err(|_| format!("unsupported binary transition: {path}"))?
            .unwrap_or("");
        let old_lines: Vec<_> = old_text.split_inclusive('\n').collect();
        let new_lines: Vec<_> = new_text.split_inclusive('\n').collect();
        diff.push_str(&format!(
            "--- {}\n+++ {}\n@@ -{},{} +{},{} @@\n",
            if old.is_some() {
                format!("a/{path}")
            } else {
                "/dev/null".into()
            },
            if new.is_some() {
                format!("b/{path}")
            } else {
                "/dev/null".into()
            },
            usize::from(!old_lines.is_empty()),
            old_lines.len(),
            usize::from(!new_lines.is_empty()),
            new_lines.len()
        ));
        for (prefix, lines) in [('-', old_lines), ('+', new_lines)] {
            for line in lines {
                diff.push(prefix);
                diff.push_str(line);
                if !line.ends_with('\n') {
                    diff.push_str("\n\\ No newline at end of file\n");
                }
            }
        }
    }
    Ok(diff)
}

#[derive(Debug, Clone)]
pub enum IncorporationError {
    Construction(String),
    Outputs(Vec<ProjectDifference>),
}

/// A reviewed candidate is pinned to both destination documents and completed
/// code. Validation is pure; writing it is the existing host review's job.
#[derive(Debug, Clone)]
pub struct IncorporationCandidate {
    entry: String,
    baseline_documents: BTreeMap<String, String>,
    documents: BTreeMap<String, String>,
    completed: ProjectTree,
}
impl IncorporationCandidate {
    pub fn verify(
        entry: &str,
        baseline_documents: BTreeMap<String, String>,
        documents: BTreeMap<String, String>,
        completed: ProjectTree,
    ) -> Result<Self, IncorporationError> {
        let evaluated = evaluate_project(entry, &documents, &ProjectTree::default())
            .map_err(IncorporationError::Construction)?;
        if documents
            .keys()
            .any(|path| !evaluated.inventory.documents.contains(path))
        {
            return Err(IncorporationError::Construction(
                "candidate contains documents outside its declared Markdown closure".into(),
            ));
        }
        for path in &evaluated.inventory.read_only {
            if baseline_documents.get(path) != documents.get(path) {
                return Err(IncorporationError::Construction(format!(
                    "Upstream source is read-only: {path}"
                )));
            }
        }
        let generated = evaluated.new;
        let differences = generated.compare(&completed);
        if !differences.is_empty() {
            return Err(IncorporationError::Outputs(differences));
        }
        Ok(Self {
            entry: entry.into(),
            baseline_documents,
            documents,
            completed,
        })
    }
    pub fn entry(&self) -> &str {
        &self.entry
    }
    pub fn documents(&self) -> &BTreeMap<String, String> {
        &self.documents
    }
    pub fn completed(&self) -> &ProjectTree {
        &self.completed
    }
    pub fn recheck_destinations(
        &self,
        documents: &BTreeMap<String, String>,
        code: &ProjectTree,
    ) -> Result<(), String> {
        if &self.baseline_documents != documents {
            return Err(
                "literate destinations changed; generate and review a new candidate".into(),
            );
        }
        if !self.completed.compare(code).is_empty() {
            return Err("completed code changed; generate and review a new candidate".into());
        }
        Ok(())
    }
}

/// Shared capture contract for built-in, Python and ACP harnesses. Capturing
/// native writes establishes actual B, but cannot establish their authorship.
#[derive(Debug, Clone)]
pub struct AgentProjectCapture {
    pub starting: ProjectTree,
    ambiguous_paths: BTreeSet<String>,
    complete: bool,
}
impl AgentProjectCapture {
    pub fn new(starting: ProjectTree) -> Self {
        Self {
            starting,
            ambiguous_paths: BTreeSet::new(),
            complete: true,
        }
    }
    pub fn incomplete(&mut self) {
        self.complete = false;
    }
    pub fn concurrent_write(&mut self, path: &str) {
        self.ambiguous_paths.insert(path.into());
    }
    pub fn resolve_concurrent(&mut self, path: &str) {
        self.ambiguous_paths.remove(path);
    }
    pub fn finalize(&self, completed: &ProjectTree, explanation: &str) -> Result<String, String> {
        if !self.complete {
            return Err(
                "incomplete agent project capture; finalization requires the actual output state"
                    .into(),
            );
        }
        if !self.ambiguous_paths.is_empty() {
            return Err(format!(
                "resolve concurrent writes before attributing the transition: {}",
                self.ambiguous_paths
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        transition_response(&self.starting, completed, explanation)
    }
}
