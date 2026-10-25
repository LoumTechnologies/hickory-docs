<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/src/project/edit.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Patch regeneration revalidates downstream construction](../../../../../contract.md)

Captured SHA-256: `4a4e4414c63596b9bf38e8bc03c59de9923b413db594f88172c1cac43980f2af`.

<book:entry path="crates/hick-lang/src/project/edit.rs">
use super::{ProjectTree, evaluate_project, unified_transition};
use std::collections::BTreeMap;

/// Reverse a byte edit through a unique literal/inserted origin. Existing hunks
/// are then checked against the changed construction; none are silently rebased.
pub fn reverse_project_edit(
    entry: &str,
    sources: &BTreeMap<String, String>,
    input: &ProjectTree,
    endpoint: &ProjectTree,
    path: &str,
    range: (usize, usize),
    replacement: &str,
) -> Result<BTreeMap<String, String>, String> {
    let file = endpoint
        .get(path)
        .ok_or_else(|| format!("code absent from selected project: {path}"))?;
    let text = std::str::from_utf8(&file.bytes)
        .map_err(|_| format!("binary code has no text edit destination: {path}"))?;
    if range.0 > range.1 || !text.is_char_boundary(range.0) || !text.is_char_boundary(range.1) {
        return Err("edit is not a valid UTF-8 byte range".into());
    }
    let origins: Vec<_> = file
        .origins
        .iter()
        .filter(|o| o.output.0 <= range.0 && range.1 <= o.output.1)
        .collect();
    if origins.len() != 1 {
        return Err(format!(
            "synthetic, ambiguous or discontinuous edit in {path}; contributing source destinations: {}",
            file.origins
                .iter()
                .map(|o| o.document.as_str())
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let origin = origins[0];
    let evaluated = evaluate_project(entry, sources, input)?;
    if evaluated.inventory.read_only.contains(&origin.document) {
        return Err(format!(
            "Upstream source {} is read-only; add a local generated-file patch with explicit predecessors instead",
            origin.document
        ));
    }
    let destination = sources
        .get(&origin.document)
        .ok_or_else(|| format!("missing contributing source {}", origin.document))?;
    let start = origin.source.0 + range.0 - origin.output.0;
    let end = start + range.1 - range.0;
    if destination.get(start..end) != Some(&text[range.0..range.1]) {
        return Err(format!("source destination changed: {}", origin.document));
    }
    let doc = crate::parse(destination).map_err(|e| e.to_string())?;
    let bom = usize::from(destination.starts_with('\u{feff}')) * 3;
    if doc.all_tags().iter().any(|tag| {
        tag.name == "ingested"
            && tag.get_attribute("key").is_some()
            && tag.source_span.is_some_and(|s| s.end + bom <= start)
            && tag.close_span.is_some_and(|s| end <= s.start + bom)
    }) {
        return Err(format!(
            "Generator output in {} is synthetic; edit its cell command and re-record instead",
            origin.document
        ));
    }
    // A patch line's prefix is grammar, not code. Multi-line edits regenerate
    // that act from its local tree rather than injecting unprefixed payload.
    if let Some(tag) = doc
        .all_tags()
        .into_iter()
        .filter(|tag| tag.name == "patch")
        .into_iter()
        .find(|t| {
            t.source_span.is_some_and(|s| s.end + bom <= start)
                && t.close_span.is_some_and(|s| end <= s.start + bom)
        })
    {
        let id = tag.get_attribute("id").ok_or_else(|| {
            format!(
                "patch in {} needs an explicit id before reverse editing",
                origin.document
            )
        })?;
        let evaluated = evaluate_project(entry, sources, input)?;
        let mut target = evaluated
            .local_states
            .get(id)
            .ok_or("patch has no local state")?
            .1
            .clone();
        let candidates: Vec<_> = target
            .entries()
            .iter()
            .flat_map(|(path, e)| {
                e.origins
                    .iter()
                    .filter(|o| {
                        o.document == origin.document && o.source.0 <= start && end <= o.source.1
                    })
                    .map(move |o| (path.clone(), o.output.0 + start - o.source.0))
            })
            .collect();
        if candidates.len() != 1 {
            return Err(format!(
                "ambiguous patch destination in {}",
                origin.document
            ));
        }
        let (target_path, offset) = &candidates[0];
        let mut changed = target.get(target_path).unwrap().as_ref().clone();
        let mut bytes = changed.bytes.to_vec();
        bytes.splice(
            *offset..*offset + end - start,
            replacement.as_bytes().iter().copied(),
        );
        changed.bytes = bytes.into();
        changed.origins.clear();
        target.insert(target_path, changed)?;
        return regenerate_patch(entry, sources, input, id, &target);
    }
    let mut candidate = sources.clone();
    let mut changed = destination.clone();
    changed.replace_range(start..end, replacement);
    candidate.insert(origin.document.clone(), changed);
    evaluate_project(entry, &candidate, input).map_err(|e| {
        format!(
            "edit to {} invalidates downstream construction: {e}",
            origin.document
        )
    })?;
    Ok(candidate)
}

/// Regenerate one authored patch against its recorded local input. This changes
/// only the candidate Markdown; downstream construction is re-evaluated and an
/// invalid later hunk returns its error before any host writes can occur.
pub fn regenerate_patch(
    entry: &str,
    sources: &BTreeMap<String, String>,
    input: &ProjectTree,
    id: &str,
    local_new: &ProjectTree,
) -> Result<BTreeMap<String, String>, String> {
    let evaluated = evaluate_project(entry, sources, input)?;
    let (before, _) = evaluated
        .local_states
        .get(id)
        .ok_or_else(|| format!("unknown patch construction: {id}"))?;
    let mut diff = unified_transition(before, local_new)?;
    if diff.is_empty() {
        // Retain the declared dependency node when this act becomes a no-op.
        // Exact zero-count hunks are still valid unified diffs.
        let path = before
            .entries()
            .keys()
            .next()
            .ok_or("cannot express an empty patch over an empty input")?;
        diff = format!("--- a/{path}\n+++ b/{path}\n@@ -0,0 +0,0 @@\n");
    }
    let mut candidate = sources.clone();
    let mut destination = None;
    for document in &evaluated.inventory.documents {
        let source = &sources[document];
        let doc = crate::parse(source).map_err(|e| e.to_string())?;
        for tag in doc.all_tags().into_iter().filter(|tag| tag.name == "patch") {
            if tag.get_attribute("id") != Some(id) {
                continue;
            }
            let open = tag.source_span.ok_or("patch opening span missing")?;
            let close = tag.close_span.ok_or("patch closing span missing")?;
            if diff.contains(&format!("</{}:patch>", doc.prefix)) {
                return Err(format!(
                    "patch payload contains its closing delimiter; rebind the namespace in {document}"
                ));
            }
            if destination.is_some() {
                return Err(format!("ambiguous patch identity: {id}"));
            }
            let bom = usize::from(source.starts_with('\u{feff}')) * 3;
            let mut replacement = source.clone();
            replacement.replace_range(open.end + bom..close.start + bom, &format!("\n{diff}"));
            destination = Some((document.clone(), replacement));
        }
    }
    let (path, source) = destination.ok_or_else(|| {
        format!("{id} is not an authored patch; edit its contributing literal construction")
    })?;
    candidate.insert(path, source);
    let updated = evaluate_project(entry, &candidate, input)
        .map_err(|e| format!("regenerated patch invalidates a construction: {e}"))?;
    let actual = &updated.local_states[id].1;
    if !actual.compare(local_new).is_empty() {
        return Err("regenerated patch did not preserve the requested local New state".into());
    }
    Ok(candidate)
}
</book:entry>

</book:doc>
