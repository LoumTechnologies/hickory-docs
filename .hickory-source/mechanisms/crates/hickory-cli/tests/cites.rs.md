<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/cites.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a19643f3dc7c"></a>
## `crates/hickory-cli/tests/cites.rs`

Related contracts: [A Citation That Points At Nothing Fails](../../../../guarantees/lineage/a-citation-that-points-at-nothing-fails/contract.md), [An Edited AI Passage Stops Claiming The Model Wrote It](../../../../guarantees/lineage/an-edited-ai-passage-stops-claiming-the-model-wrote-it/contract.md), [Three Provenances Are Drawn Apart, And Any Subset Can Be Shown](../../../../guarantees/lineage/three-provenances-are-drawn-apart/contract.md).

[Read the test `cites_resolve_across_the_chain_and_report_dangling_ones` beside its requirement](../../../../guarantees/lineage/three-provenances-are-drawn-apart/contract.md#f-a19643f3dc7c-cites_resolve_across_the_chain_and_report_dangling_ones).
<a id="f-a19643f3dc7c-a_cited_claim_weaves_its_citation_as_words"></a>
### `a_cited_claim_weaves_its_citation_as_words`

<storycode:copy id="f-a19643f3dc7c-a_cited_claim_weaves_its_citation_as_words">
#[test]
fn a_cited_claim_weaves_its_citation_as_words() {
    let dir = tempfile::tempdir().unwrap();
    let note = dir.path().join("n.md");
    std::fs::write(
        &note,
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"n.md\">\n<hick:copy id=\"a\">x</hick:copy>\n<hick:claim by=\"nate\" standing=\"report\" cites=\"#a\">Said so.</hick:claim>\n</hick:doc>\n",
    )
    .unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_hick"))
        .args(["weave", note.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let woven = std::fs::read_to_string(dir.path().join("n.md")).unwrap();
    assert!(woven.contains("Said so."), "{woven}");
    assert!(woven.contains("*cites: #a*"), "{woven}");
    assert!(!woven.contains("✓"), "never a checkmark: {woven}");
}

</storycode:copy>
[Read the test `a_dangling_citation_fails_verification` beside its requirement](../../../../guarantees/lineage/a-citation-that-points-at-nothing-fails/contract.md#f-a19643f3dc7c-a_dangling_citation_fails_verification).
<a id="f-a19643f3dc7c-a_citation_that_resolves_is_not_reported"></a>
### `a_citation_that_resolves_is_not_reported`

<storycode:copy id="f-a19643f3dc7c-a_citation_that_resolves_is_not_reported">
#[test]
fn a_citation_that_resolves_is_not_reported() {
    let dir = tempfile::tempdir().unwrap();
    let note = dir.path().join("note.md");
    std::fs::write(
        &note,
        "<hick:copy id=\"finding\" class=\"f\">Pool is 40.</hick:copy>\n\n         <hick:claim by=\"nate\" standing=\"judgment\" cites=\"#finding,.f\">We are fine.</hick:claim>\n",
    )
    .unwrap();
    let source = std::fs::read_to_string(&note).unwrap();
    assert!(dangling_citations(&note, &source).unwrap().is_empty());
    // Two selectors, one fragment: the same place cited twice is listed once.
    let cites = declared_cites(&note, &source).unwrap();
    assert_eq!(cites[0].to.len(), 1, "{:?}", cites[0].to);
    assert!(cites[0].dangling.is_empty());
}

</storycode:copy>
[Read the test `edited_passages` beside its requirement](../../../../guarantees/lineage/an-edited-ai-passage-stops-claiming-the-model-wrote-it/contract.md#f-a19643f3dc7c-edited_passages).

### Assemble `crates/hickory-cli/tests/cites.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/cites.rs"><storycode:paste select="#f-a19643f3dc7c-cites_resolve_across_the_chain_and_report_dangling_ones"/><storycode:paste select="#f-a19643f3dc7c-a_cited_claim_weaves_its_citation_as_words"/><storycode:paste select="#f-a19643f3dc7c-a_dangling_citation_fails_verification"/><storycode:paste select="#f-a19643f3dc7c-a_citation_that_resolves_is_not_reported"/><storycode:paste select="#f-a19643f3dc7c-edited_passages"/></storycode:file>

</storycode:doc>
