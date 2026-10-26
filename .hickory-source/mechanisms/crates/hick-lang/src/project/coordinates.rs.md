<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lang/src/project/coordinates.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-728d4259b884"></a>
## `crates/hick-lang/src/project/coordinates.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

<a id="f-728d4259b884-project_byte_location"></a>
### `project_byte_location`

<storycode:copy id="f-728d4259b884-project_byte_location">
use super::ProjectTree;

/// A local construction position may survive in a different final file. Match
/// exact origin bytes; do not search for equal lines or guess an offset.
pub fn project_byte_location(
    local: &ProjectTree,
    endpoint: &ProjectTree,
    path: &str,
    byte: usize,
) -> Result<Option<(String, usize)>, String> {
    let entry = local
        .get(path)
        .ok_or_else(|| format!("Code absent from local state: {path}"))?;
    if byte > entry.bytes.len() {
        return Err("Byte outside local code".into());
    }
    if let Some(target) = endpoint.get(path)
        && entry.same_output(target)
        && (entry.origins == target.origins || target.origins.is_empty())
    {
        return Ok(Some((path.into(), byte)));
    }
    let mut found = None;
    for origin in &entry.origins {
        if !(origin.output.0 <= byte && byte < origin.output.1) {
            continue;
        }
        let source = origin.source.0 + byte - origin.output.0;
        for (path, target) in endpoint.entries() {
            for other in &target.origins {
                if other.document == origin.document
                    && other.source.0 <= source
                    && source < other.source.1
                {
                    let output = other.output.0 + source - other.source.0;
                    if entry.bytes.get(byte) != target.bytes.get(output) {
                        return Err("Endpoint provenance does not name the displayed byte".into());
                    }
                    if found.is_some() {
                        return Err("Ambiguous endpoint; choose the contributing source or an explicit local state".into());
                    }
                    found = Some((path.clone(), output));
                }
            }
        }
    }
    Ok(found)
}
</storycode:copy>

### Assemble `crates/hick-lang/src/project/coordinates.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/src/project/coordinates.rs"><storycode:paste select="#f-728d4259b884-project_byte_location"/></storycode:file>

</storycode:doc>
