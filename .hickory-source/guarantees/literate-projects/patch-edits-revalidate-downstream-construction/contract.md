<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Keep a patch edit as a draft until the whole construction still works

For: Maintainers reorganizing source while preserving the complete generated program.

Change an early patch so a later patch no longer applies. The edit must remain an invalid draft rather than silently publishing a half-valid project.

## What must happen

Editing authored patch code regenerates the patch against its captured predecessor and reevaluates the complete supplied closure. A downstream-breaking edit remains unchecked and cannot publish. Current source identity is checked before local publication; cross-document candidates use the native publication guard. Executable patch identities remain required for this editing surface. Old-side editing follows its declared permission rather than inheriting New-side authority.

## Limits of the evidence

Passing patch evaluation does not certify explanatory prose or runtime behavior. Actual native visual acceptance is separate from editor-controller tests.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/patch-edits-revalidate-downstream-construction.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the implemented editor/WASM/native-guard continuation into the contract and retire the old pending-wiring exclusion.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Capture external output edits before incorporation

Ordinary tools edit generated files without choosing a conceptual owner. Native
project materialization records its exact successful baseline; changed products
become durable pending captures and block regeneration. Capture never amends the
source that supplied those bytes. Authors explicitly create a new concept patch,
or amend/split the existing source and accept only after complete reconstruction
matches the captured result. Binary changes remain captured without pretending
that a textual patch can express them.

Direct authored patch editing and explicit reverse editing retain their existing
validation below. They are incorporation tools, not a default ownership decision
for an unclassified filesystem edit.

Read the [capture mechanism and focused witnesses](output-capture.md).
<storycode:include file="output-capture.md" />

## Implementation

<a id="output-1fd16b1e6cf1"></a>
## `crates/hick-lang/src/project/edit.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

Related contracts: [Keep a patch edit as a draft until the whole construction still works](contract.md).

<a id="f-1fd16b1e6cf1-reverse_project_edit"></a>
### `reverse_project_edit`

<storycode:copy id="f-1fd16b1e6cf1-reverse_project_edit">
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

</storycode:copy>
<a id="f-1fd16b1e6cf1-regenerate_patch"></a>
### `regenerate_patch`

Regenerate one authored patch against its recorded local input. This changes
only the candidate Markdown; downstream construction is re-evaluated and an
invalid later hunk returns its error before any host writes can occur.

<storycode:copy id="f-1fd16b1e6cf1-regenerate_patch">
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
</storycode:copy>

### Assemble `crates/hick-lang/src/project/edit.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/src/project/edit.rs"><storycode:paste select="#f-1fd16b1e6cf1-reverse_project_edit"/><storycode:paste select="#f-1fd16b1e6cf1-regenerate_patch"/></storycode:file>

## Test the behavior: `patch_regeneration_preserves_order_and_reports_invalid_later_context`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-patch_regeneration_preserves_order_and_reports_invalid_later_context"></a>
### `patch_regeneration_preserves_order_and_reports_invalid_later_context`

<storycode:copy id="f-005c1e9344c7-patch_regeneration_preserves_order_and_reports_invalid_later_context">
// Guarantee: docs/guarantees/literate-projects/patch-edits-revalidate-downstream-construction.md
#[test]
fn patch_regeneration_preserves_order_and_reports_invalid_later_context() {
    let source = "<hick:project/><hick:file id=\"base\" path=\"x\">a\n</hick:file><hick:patch id=\"early\" after=\"base\">\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-a\n+b\n</hick:patch><hick:patch id=\"later\" after=\"early\">\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-b\n+c\n</hick:patch>";
    let sources = BTreeMap::from([("project.md".into(), source.into())]);
    let input = ProjectTree::default();
    let error = regenerate_patch(
        "project.md",
        &sources,
        &input,
        "early",
        &tree(&[("x", b"different\n")]),
    )
    .unwrap_err();
    assert!(error.contains("wrong patch context"));
    assert_eq!(sources["project.md"], source);
    let candidate = regenerate_patch(
        "project.md",
        &sources,
        &input,
        "later",
        &tree(&[("x", b"final\nwithout final newline")]),
    )
    .unwrap();
    let generated = evaluate_project("project.md", &candidate, &input).unwrap();
    assert_eq!(
        generated.new.get("x").unwrap().bytes.as_ref(),
        b"final\nwithout final newline"
    );
}
</storycode:copy>

## Test the behavior: `reverse_inserted_code_regenerates_its_patch_and_old_literal_edits_check_downstream`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-reverse_inserted_code_regenerates_its_patch_and_old_literal_edits_check_downstream"></a>
### `reverse_inserted_code_regenerates_its_patch_and_old_literal_edits_check_downstream`

<storycode:copy id="f-005c1e9344c7-reverse_inserted_code_regenerates_its_patch_and_old_literal_edits_check_downstream">
// Guarantee: docs/guarantees/literate-projects/patch-edits-revalidate-downstream-construction.md
#[test]
fn reverse_inserted_code_regenerates_its_patch_and_old_literal_edits_check_downstream() {
    let source = "<hick:project/><hick:file id=\"base\" path=\"app\">prefix\nold\n</hick:file><hick:scope id=\"s\" after=\"base\"><hick:patch id=\"p\">\n--- a/app\n+++ b/app\n@@ -2 +2 @@\n-old\n+new\n</hick:patch></hick:scope>";
    let sources = BTreeMap::from([("project.md".into(), source.into())]);
    let project = evaluate_project("project.md", &sources, &ProjectTree::default()).unwrap();
    let updated = reverse_project_edit(
        "project.md",
        &sources,
        &ProjectTree::default(),
        &project.new,
        "app",
        (7, 10),
        "é\nextra",
    )
    .unwrap();
    let result = evaluate_project("project.md", &updated, &ProjectTree::default()).unwrap();
    assert_eq!(
        &*result.new.get("app").unwrap().bytes,
        "prefix\né\nextra\n".as_bytes()
    );
    assert!(
        reverse_project_edit(
            "project.md",
            &sources,
            &ProjectTree::default(),
            &project.local_states["p"].0,
            "app",
            (7, 10),
            "changed"
        )
        .unwrap_err()
        .contains("downstream")
    );
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/elements/patch/Editor.tsx](../../../mechanisms/apps/web/src/elements/patch/Editor.tsx.md#output-dc5641c063e3)
- [crates/hick-lang/src/project/edit.rs](contract.md#output-1fd16b1e6cf1)
- [crates/hick-lang/src/project/reconstruct.rs](../finalization-and-incorporation-check-distinct-equalities/contract.md#output-bf6f560e06c6)

### Tests and fixtures

- [apps/web/src/components/ChangesetDocument.test.tsx](../../../mechanisms/apps/web/src/components/ChangesetDocument.test.tsx.md#output-9ae884d72abd)
- [apps/web/src/elements/patch/Editor.test.tsx](../../../mechanisms/apps/web/src/elements/patch/Editor.tsx.md#output-dc4ea70ab787)
- [apps/web/src/lib/projectComparison.test.ts](../../../mechanisms/apps/web/src/lib/projectComparison.ts.md#output-a9f1fc73a39f)
- [crates/hick-lang/tests/project.rs](../../../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
