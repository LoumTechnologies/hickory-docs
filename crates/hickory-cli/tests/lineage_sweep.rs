//! Exhaustive lineage sweep: for EVERY char boundary of a woven output, a
//! replacement and an insertion either map to source edits that — applied
//! and re-woven — reproduce the edited output byte-for-byte, or are
//! rejected for a provable reason (synthetic range / duplicated paste).
//!
//! Guards docs/guarantees/verification (lineage half of api.md v0.2): the
//! Ok ⇒ byte-exact-round-trip invariant has no position-dependent holes.

use std::collections::HashMap;
use std::path::Path;

use hickory_cli::{DocRun, ExecutorChoice, RunMode, output_lineage, run_doc};
use hickory_lineage::{
    LineageError, Origin, OutputEdit, Provenance, apply_source_edits, map_edits,
};

/// Copy blocks (one with multi-byte UTF-8), literal text, an id paste, a
/// class paste with separator (synthetic bytes), and the SAME copy pasted
/// twice — the case where exact reproduction is impossible and rejection is
/// the only honest answer.
const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:copy id="a" class="frag">alpha café ☕
</hick:copy>
<hick:copy id="b" class="frag">beta-β
</hick:copy>
<hick:file path="woven.txt">HEAD
<hick:paste select="#b" />MID
<hick:paste select=".frag" separator=" | " />TAIL
</hick:file>
</hick:doc>
"##;

async fn weave(dir: &Path, source: &str) -> DocRun {
    let doc_path = dir.join("sweep.hick");
    std::fs::write(&doc_path, source).unwrap();
    run_doc(&doc_path, &[], RunMode::Weave, ExecutorChoice::Local)
        .await
        .expect("weave must succeed")
}

fn output_text(run: &DocRun) -> String {
    match run.result.files.get("woven.txt").expect("woven.txt exists") {
        hick_exec::node::FileContent::Text(s) => s.clone(),
        other => panic!("expected text output, got {other:?}"),
    }
}

/// Does `start..end` (or the insertion point for empty ranges) touch a
/// synthetic entry or a provenance gap?
fn touches_synthetic_or_gap(prov: &[Provenance], start: usize, end: usize) -> bool {
    let overlaps = |p: &Provenance| {
        if start == end {
            // Insertion: only "strictly inside" forces the entry's kind;
            // boundary insertions may attach to an editable neighbor.
            start > p.start && start < p.end
        } else {
            start < p.end && end > p.start
        }
    };
    let in_editable_or_boundary = |pos: usize| {
        prov.iter()
            .any(|p| p.origin.source().is_some() && pos >= p.start && pos <= p.end)
    };
    if start == end {
        return prov
            .iter()
            .any(|p| matches!(p.origin, Origin::Synthetic) && overlaps(p))
            || !in_editable_or_boundary(start);
    }
    // Any overlapped synthetic entry, or any byte not covered by an
    // editable entry, justifies rejection.
    if prov
        .iter()
        .any(|p| matches!(p.origin, Origin::Synthetic) && overlaps(p))
    {
        return true;
    }
    (start..end).any(|b| {
        !prov
            .iter()
            .any(|p| p.origin.source().is_some() && b >= p.start && b < p.end)
    })
}

/// Is any editable provenance entry covering `start..end` sourced from a
/// span that also feeds a DIFFERENT output range (same copy pasted twice)?
fn touches_duplicated_source(prov: &[Provenance], start: usize, end: usize) -> bool {
    prov.iter().enumerate().any(|(i, p)| {
        let covered = if start == end {
            start >= p.start && start <= p.end
        } else {
            start < p.end && end > p.start
        };
        if !covered {
            return false;
        }
        let Some((doc, s, e)) = p.origin.source() else {
            return false;
        };
        prov.iter().enumerate().any(|(j, q)| {
            i != j
                && q.origin
                    .source()
                    .is_some_and(|(d2, s2, e2)| d2 == doc && s < e2 && e > s2)
        })
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn every_position_edit_round_trips_or_is_provably_rejected() {
    let base = tempfile::tempdir().unwrap();
    let run = weave(base.path(), DOC).await;
    let content = output_text(&run);
    let prov = output_lineage(&run, "woven.txt").unwrap();

    // Sanity: the fixture must exercise all three regimes.
    assert!(prov.iter().any(|p| matches!(p.origin, Origin::Synthetic)));
    assert!(touches_duplicated_source(&prov, 0, content.len()));

    let boundaries: Vec<usize> = content
        .char_indices()
        .map(|(i, _)| i)
        .chain([content.len()])
        .collect();

    let mut ok_positions = 0usize;
    let mut rejected = 0usize;
    for (bi, &pos) in boundaries.iter().enumerate() {
        let mut cases: Vec<OutputEdit> = vec![OutputEdit {
            start: pos,
            end: pos,
            text: "+".to_string(),
        }];
        if bi + 1 < boundaries.len() {
            // Replace exactly one char (multi-byte-safe) with a multi-byte
            // replacement, stressing both sides of the offset arithmetic.
            cases.push(OutputEdit {
                start: pos,
                end: boundaries[bi + 1],
                text: "Ω".to_string(),
            });
        }
        for edit in cases {
            let (start, end) = (edit.start, edit.end);
            match map_edits(&content, std::slice::from_ref(&edit), &prov) {
                Ok(source_edits) => {
                    let sources: HashMap<String, String> = run
                        .result
                        .provenance_maps
                        .keys()
                        .map(|_| ())
                        .next()
                        .map(|_| HashMap::new())
                        .unwrap_or_default();
                    let _ = sources; // sources built explicitly below
                    let doc_key = source_edits
                        .first()
                        .map(|e| e.doc_path.clone())
                        .expect("Ok with no edits");
                    let mut srcs = HashMap::new();
                    srcs.insert(doc_key.clone(), run.source.clone());
                    let updated = apply_source_edits(&srcs, &source_edits)
                        .unwrap_or_else(|e| panic!("apply failed at {start}..{end}: {e}"));
                    let new_source = updated.get(&doc_key).unwrap();

                    let dir = tempfile::tempdir().unwrap();
                    let rerun = weave(dir.path(), new_source).await;
                    let new_output = output_text(&rerun);

                    let mut expected = content.clone();
                    expected.replace_range(start..end, &edit.text);
                    assert_eq!(
                        new_output, expected,
                        "position {start}..{end}: mapped Ok but re-weave does not \
                         reproduce the edited output.\nsource edits: {source_edits:?}"
                    );
                    ok_positions += 1;
                }
                Err(LineageError::SyntheticOverlap { .. }) => {
                    assert!(
                        touches_synthetic_or_gap(&prov, start, end),
                        "position {start}..{end}: rejected as synthetic but no \
                         synthetic range or gap covers it.\nprovenance: {prov:?}"
                    );
                    rejected += 1;
                }
                Err(LineageError::Conflict(msg)) => {
                    assert!(
                        touches_duplicated_source(&prov, start, end),
                        "position {start}..{end}: rejected as conflict ({msg}) but \
                         its source span feeds no other output range"
                    );
                    rejected += 1;
                }
                Err(LineageError::InvalidEdit(msg)) => {
                    panic!("position {start}..{end}: unexpectedly invalid: {msg}")
                }
            }
        }
    }
    // The sweep must have exercised both outcomes to mean anything.
    assert!(ok_positions > 0, "no position mapped Ok");
    assert!(rejected > 0, "no position was rejected");
}
