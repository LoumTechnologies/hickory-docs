<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/project_index.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Candidate revision checks never borrow working bytes](../../../../../guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes/contract.md)

Captured SHA-256: `ae6e51551b0497e2fae61e634c28de6377b47326787317f6d1eba5ec2b09e007`.

<book:entry path="crates/hickory-cli/src/project_index.rs">
//! Candidate checks discover declarations in the same frozen index as outputs.
use anyhow::{Context, Result, bail};
use hick_lang::{EntryKind, ProjectTree, evaluate_project, parse};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub fn verify(root: &Path, standalone: Option<&Path>) -> Result<()> {
    let snapshot = crate::project_revision::git(root, ":index")?;
    let covered = verify_with_coverage(&snapshot)?;
    if let Some(destination) = standalone {
        use std::io::Write;
        let mut list = std::fs::File::create(destination)?;
        for (path, entry) in snapshot.entries() {
            if path.ends_with(".md") && entry.kind == EntryKind::File && !covered.contains(path) {
                list.write_all(path.as_bytes())?;
                list.write_all(&[0])?;
            }
        }
    }
    Ok(())
}

pub fn verify_snapshot(snapshot: &ProjectTree) -> Result<()> {
    verify_with_coverage(snapshot).map(|_| ())
}
fn verify_with_coverage(snapshot: &ProjectTree) -> Result<BTreeSet<String>> {
    let mut covered = BTreeSet::new();
    for (path, document) in snapshot.entries() {
        if !path.ends_with(".md") || document.kind != EntryKind::File {
            continue;
        }
        let source = std::str::from_utf8(&document.bytes)
            .with_context(|| format!("candidate Markdown is not UTF-8: {path}"))?;
        // Structure discovery is tolerant of ordinary Markdown, but a project
        // declaration is always strictly parsed when its closure is evaluated.
        let structure = hick_lang::structure(source);
        if !structure.tags.iter().any(|t| {
            t.name == "project" && !t.closing && !t.attrs.iter().any(|(n, _)| n == "entry")
        }) {
            continue;
        }
        let doc = parse(source).with_context(|| format!("candidate project {path}"))?;
        if doc.find_tags("project").is_empty() {
            continue;
        }
        let (prefix, entry) = path
            .rsplit_once('/')
            .map_or((String::new(), path.as_str()), |(parent, name)| {
                (format!("{parent}/"), name)
            });
        let mut scope = ProjectTree::default();
        for (name, value) in snapshot.entries() {
            if let Some(relative) = name.strip_prefix(&prefix) {
                scope
                    .insert(relative, value.as_ref().clone())
                    .map_err(anyhow::Error::msg)?;
            }
        }
        let sources: BTreeMap<String, String> = hick_lang::load_project_sources(entry, &|name| {
            let file = scope
                .get(name)
                .ok_or_else(|| format!("candidate project lacks {name}"))?;
            if file.kind != EntryKind::File {
                return Err(format!("candidate Markdown is not a regular file: {name}"));
            }
            std::str::from_utf8(&file.bytes)
                .map(str::to_owned)
                .map_err(|e| e.to_string())
        })
        .map_err(anyhow::Error::msg)?;
        for (name, text) in &sources {
            let parsed = parse(text)?;
            let errors = hick_blocks::attribute_errors(text, &parsed);
            if !errors.is_empty() {
                bail!("staged source {prefix}{name}: {errors:?}");
            }
            covered.insert(format!("{prefix}{name}"));
            if let Some(woven) = parsed.weave_path {
                covered.insert(format!("{prefix}{woven}"));
            }
        }
        let project = evaluate_project(entry, &sources, &ProjectTree::default())
            .map_err(anyhow::Error::msg)?;
        let expected = project
            .inventory
            .select(&scope)
            .map_err(anyhow::Error::msg)?;
        let differences = project.new.compare(&expected);
        if !differences.is_empty() {
            bail!(
                "candidate project {path} differs: {}",
                differences
                    .iter()
                    .map(|d| format!("{:?}: {}", d.kind, d.path))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        println!(
            "Verified staged project {path} ({} entries).",
            project.new.entries().len()
        );
    }
    Ok(covered)
}
</book:entry>

</book:doc>
