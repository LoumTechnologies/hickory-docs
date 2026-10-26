<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Historical evidence stays historical

For: Authors using an agent to change documents while retaining review control.

Given a new completed read receipt, when its session is inspected later, then
the audit retains the exact returned source passage, its original range and
whole-source SHA-256. The passage does not become today's file contents. A
separate current-source action opens today's file; current source is labeled
unchanged, changed or unavailable by a confined mechanical digest comparison.
Guarantee dependency snapshots use the same independent current check.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/historical-evidence-stays-historical.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-db14835b3e44"></a>
## `crates/hickory-cli/src/serve/audit_evidence.rs`

Related contracts: [Historical evidence stays historical](contract.md).

<a id="f-db14835b3e44-fingerprint"></a>
### `fingerprint`

Current mechanical checks kept apart from historical or semantic claims.

<storycode:copy id="f-db14835b3e44-fingerprint">
//! Current mechanical checks kept apart from historical or semantic claims.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

fn fingerprint(root: &Path, path: &str) -> Option<String> {
    let root = root.canonicalize().ok()?;
    let path = root.join(path).canonicalize().ok()?;
    if !path.starts_with(&root) {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    Some(format!("{:x}", Sha256::digest(bytes)))
}

</storycode:copy>
<a id="f-db14835b3e44-enrich"></a>
### `enrich`

<storycode:copy id="f-db14835b3e44-enrich">
pub fn enrich(root: &Path, audit: &mut Vec<Value>, links: &Value) {
    for record in audit.iter_mut() {
        if record["record_kind"] == "read-snapshot" {
            let current = record["path"].as_str().and_then(|p| fingerprint(root, p));
            record["current_state"] = json!(match current {
                Some(ref hash) if record["sha256"] == *hash => "unchanged",
                Some(_) => "changed",
                None => "unavailable",
            });
            record["semantic_support"] =
                json!("Not assessed; availability of bytes is not support for a claim");
        }
        for key in ["dependency_snapshots", "source_snapshots"] {
            if let Some(dependencies) = record[key].as_array_mut() {
                for dependency in dependencies {
                    let current = dependency["path"]
                        .as_str()
                        .and_then(|p| fingerprint(root, p));
                    dependency["current_state"] = json!(match current {
                        Some(ref hash) if dependency["sha256"] == *hash => "unchanged",
                        Some(_) if dependency["sha256"].is_string() => "changed",
                        _ => "unavailable",
                    });
                }
            }
        }
        if record["record_kind"] == "verification" {
            record["current_state"] = json!(
                record["source_snapshots"]
                    .as_array()
                    .map(|sources| {
                        if sources
                            .iter()
                            .any(|source| source["current_state"] == "changed")
                        {
                            "changed"
                        } else if !sources.is_empty()
                            && sources
                                .iter()
                                .all(|source| source["current_state"] == "unchanged")
                        {
                            "unchanged"
                        } else {
                            "unavailable"
                        }
                    })
                    .unwrap_or("unavailable")
            );
        }
    }
    for link in links.as_array().into_iter().flatten().filter(|link| {
        link["family"] == "declared"
            && !link["title"]
                .as_str()
                .unwrap_or("")
                .starts_with("ACP-reported")
    }) {
        let path = link["to"]["path"].as_str().unwrap_or("");
        let text = root
            .join(path)
            .canonicalize()
            .ok()
            .filter(|p| p.starts_with(root))
            .and_then(|p| std::fs::read_to_string(p).ok());
        let lines = link["to"]["lines"].as_array();
        let state = match text {
            None => "unavailable",
            Some(ref text)
                if lines.is_some_and(|lines| {
                    let first = lines.first().and_then(Value::as_u64).unwrap_or(0);
                    let last = lines.get(1).and_then(Value::as_u64).unwrap_or(0);
                    first == 0 || last < first || last > text.lines().count() as u64
                }) =>
            {
                "range-out-of-bounds"
            }
            Some(_) => "target-available",
        };
        audit.push(json!({"record_kind":"citation-check","path":path,"state":state,"session_line":link["lines"][0],"actor":"harness","scope":"Current path/range availability only. Relevance, entailment, freshness of the claim and completeness are not assessed."}));
    }
}

</storycode:copy>
<a id="f-db14835b3e44-attach_passages"></a>
### `attach_passages`

Attach the closest recorded returned passage, never today's replacement bytes.

<storycode:copy id="f-db14835b3e44-attach_passages">
/// Attach the closest recorded returned passage, never today's replacement bytes.
pub fn attach_passages(root: &Path, source: &str, audit: &[Value], links: &mut Value) {
    let turns: Vec<_> = hick_lang::parse(source)
        .ok()
        .into_iter()
        .flat_map(|doc| {
            doc.tags()
                .find(|tag| tag.name == "session")
                .map(|root| {
                    root.child_tags()
                        .filter(|tag| tag.name == "user")
                        .map(|tag| {
                            (
                                tag.source_line as u64,
                                tag.get_attribute("turn").map(str::to_string),
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect();
    for link in links.as_array_mut().into_iter().flatten() {
        if link["family"] != "context" && link["family"] != "declared" {
            continue;
        }
        let path = root.join(link["to"]["path"].as_str().unwrap_or(""));
        let line = link["lines"][0].as_u64().unwrap_or(u64::MAX);
        let turn = turns
            .iter()
            .rev()
            .find(|(start, _)| *start <= line)
            .and_then(|(_, turn)| turn.as_deref());
        if let Some(snapshot) = audit.iter().rev().find(|record| {
            record["record_kind"] == "read-snapshot"
                && record["turn"].as_str() == turn
                && root.join(record["path"].as_str().unwrap_or("")) == path
                && record["session_line"].as_u64().unwrap_or(0) <= line.saturating_add(1)
        }) {
            link["historical"] = snapshot.clone();
            let start = link["span"][0].as_u64().unwrap_or(0) as usize;
            let end = link["span"][1].as_u64().unwrap_or(0) as usize;
            if link["family"] == "declared"
                && let Some(claim) = source.get(start..end)
            {
                link["historical"]["claim"] = json!(claim);
                let mut comparisons = Vec::new();
                for (open, close) in [('“', '”'), ('"', '"')] {
                    let mut rest = claim;
                    while let Some((_, after)) = rest.split_once(open) {
                        let Some((quote, after)) = after.split_once(close) else {
                            break;
                        };
                        if !quote.is_empty() {
                            let present = {
                                let content = snapshot["content"].as_str().unwrap_or("");
                                let first = snapshot["first_line"].as_u64().unwrap_or(1);
                                let last = snapshot["last_line"].as_u64().unwrap_or(0);
                                match link["to"]["lines"].as_array() {
                                    Some(lines) => {
                                        let from = lines[0].as_u64().unwrap_or(0);
                                        let to =
                                            lines.get(1).and_then(Value::as_u64).unwrap_or(from);
                                        from >= first
                                            && to <= last
                                            && from <= to
                                            && content
                                                .split_inclusive('\n')
                                                .skip((from - first) as usize)
                                                .take((to - from + 1) as usize)
                                                .collect::<String>()
                                                .contains(quote)
                                    }
                                    None => content.contains(quote),
                                }
                            };
                            comparisons.push(
                                json!({"literal":quote,"present_in_recorded_passage":present}),
                            );
                        }
                        rest = after;
                    }
                }
                link["historical"]["quoted_literals"] = json!(comparisons);
            }
        }
    }
}

</storycode:copy>
<a id="f-db14835b3e44-tests"></a>
### `tests`

<storycode:copy id="f-db14835b3e44-tests">
#[cfg(test)]
mod tests {
    use super::*;
    // Guarantees: docs/guarantees/agent/historical-evidence-stays-historical.md
    #[test]
    fn quote_comparison_uses_the_recorded_cited_range_and_excludes_abandoned_reads() {
        let dir = tempfile::tempdir().unwrap();
        let source = "<hick:session>\n<hick:user turn=\"abandoned\">one</hick:user>\n<hick:user turn=\"current\">two</hick:user>\n<hick:assistant>Legal said “approved” (source.txt:1)</hick:assistant>\n</hick:session>";
        let start = source.find("Legal said").unwrap();
        let end = source.find("</hick:assistant>").unwrap();
        let mut links = json!([{"family":"declared","span":[start,end],"to":{"path":"source.txt","lines":[1,1]},"lines":[4,4]}]);
        let audit = vec![
            json!({"record_kind":"read-snapshot","path":"source.txt","content":"not approved\napproved\n","first_line":1,"last_line":2,"turn":"current","session_line":3}),
        ];
        attach_passages(dir.path(), source, &audit, &mut links);
        // Literal occurrence is deliberately weaker than semantic support: negation remains a human challenge.
        assert_eq!(
            links[0]["historical"]["quoted_literals"][0]["present_in_recorded_passage"],
            true
        );
        let mut misleading = audit.clone();
        misleading[0]["content"] = json!("pending\napproved\n");
        attach_passages(dir.path(), source, &misleading, &mut links);
        assert_eq!(
            links[0]["historical"]["quoted_literals"][0]["present_in_recorded_passage"],
            false
        );
        links[0].as_object_mut().unwrap().remove("historical");
        misleading[0]["turn"] = json!("abandoned");
        attach_passages(dir.path(), source, &misleading, &mut links);
        assert!(links[0]["historical"].is_null());
    }

    // Guarantee: docs/guarantees/agent/historical-evidence-stays-historical.md
    #[test]
    fn changed_missing_and_outside_inputs_never_look_current() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("source.txt"), "original\n").unwrap();
        let hash = fingerprint(dir.path(), "source.txt").unwrap();
        let mut audit = vec![
            json!({"record_kind":"read-snapshot","path":"source.txt","sha256":hash,"content":"original\n"}),
            json!({"record_kind":"audit","dependency_snapshots":[{"path":"source.txt","sha256":hash}]}),
        ];
        std::fs::write(dir.path().join("source.txt"), "different\n").unwrap();
        enrich(
            dir.path(),
            &mut audit,
            &json!([{ "family":"declared", "to":{"path":"missing.md","lines":[1,1]},"lines":[4,4]}]),
        );
        assert_eq!(audit[0]["current_state"], "changed");
        assert_eq!(audit[0]["content"], "original\n");
        assert_eq!(
            audit[1]["dependency_snapshots"][0]["current_state"],
            "changed"
        );
        assert_eq!(audit[2]["state"], "unavailable");
        assert!(fingerprint(dir.path(), "../outside.txt").is_none());
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/audit_evidence.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/audit_evidence.rs"><storycode:paste select="#f-db14835b3e44-fingerprint"/><storycode:paste select="#f-db14835b3e44-enrich"/><storycode:paste select="#f-db14835b3e44-attach_passages"/><storycode:paste select="#f-db14835b3e44-tests"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/AssumptionAudit.tsx](../../../mechanisms/apps/web/src/components/AssumptionAudit.tsx.md#output-7c5cafd8ed10)
- [crates/hickory-agent/src/session.rs](../../../mechanisms/crates/hickory-agent/src/session.rs.md#output-9c7ae0f28c14)
- [crates/hickory-cli/src/serve/audit_evidence.rs](contract.md#output-db14835b3e44)

### Tests and fixtures

- [apps/web/src/components/AssumptionAudit.test.tsx](../assumptions-remain-auditable-after-the-work/contract.md#output-f63b79e0becf)
- [apps/web/src/elements/session/evidence.test.tsx](../../../mechanisms/apps/web/src/elements/session/evidence.test.tsx.md#output-4f5d36dc9d9d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
