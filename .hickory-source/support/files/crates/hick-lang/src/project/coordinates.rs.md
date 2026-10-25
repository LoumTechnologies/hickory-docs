<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/src/project/coordinates.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `a71e95c29786459497c4e9ce3395236b0a471a3d38b9c029168d95d68778aeea`.

<book:entry path="crates/hick-lang/src/project/coordinates.rs">
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
</book:entry>

</book:doc>
