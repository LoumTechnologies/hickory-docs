use super::{ProjectTree, evaluate_project, unified_transition};
use std::collections::BTreeMap;

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
        for tag in doc.find_tags("patch") {
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
