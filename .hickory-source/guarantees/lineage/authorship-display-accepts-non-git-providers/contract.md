<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Authorship display accepts non-Git providers

For: Authors editing generated text and reviewers tracing a result to its source.

Given an attribution provider supplying line metadata, when author colors are
turned on in Lineage settings, then the editor distinguishes credited authors
without replacing syntax colors. Git blame is the default provider. A host can
install a replacement provider without Git. Recorded agent writes supplement
Git attribution using the existing strong session-write fingerprints. Session
user and assistant elements expose their recorded roles. Absent origin evidence
remains unknown; a Git identity alone never classifies text as human or agent.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/authorship-display-accepts-non-git-providers.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-41082384fcfa"></a>
## `apps/web/src/lib/attribution.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Authorship display accepts non-Git providers](contract.md).

<a id="f-41082384fcfa-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-41082384fcfa-expression_statement">
import { afterEach, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { authorColor, attributionProvider, installAttributionProvider } from "./attribution";
import { blameGutter, authorshipField, setBlame, setAuthorshipShown } from "../editor/blameGutter";
afterEach(() => localStorage.clear());
</storycode:copy>
<a id="f-41082384fcfa-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-41082384fcfa-expression_statement-2">
// Guarantee: docs/guarantees/lineage/authorship-display-accepts-non-git-providers.md
it("accepts a non-Git provider and restores Git on removal", async () => {
  const remove = installAttributionProvider({ id: "receipts", label: "Editor receipts", lines: async () => [] });
  expect(attributionProvider().id).toBe("receipts");
  expect(await attributionProvider().lines("note.md")).toEqual([]);
  remove(); expect(attributionProvider().id).toBe("git");
});
</storycode:copy>
<a id="f-41082384fcfa-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-41082384fcfa-expression_statement-3">
it("clears attribution when the attributed text changes", () => {
  let state = EditorState.create({ doc: "agent words", extensions: blameGutter() });
  state = state.update({ effects: [setAuthorshipShown.of(true), setBlame.of([{ line: 1, commit: "abc", author: "Sam", email: "sam@example.com", time: 0, summary: "", uncommitted: false, origin: "agent" }])] }).state;
  expect(state.field(authorshipField).marks.size).toBe(1);
  state = state.update({ changes: { from: 0, to: 5, insert: "human" } }).state;
  expect(state.field(authorshipField).marks.size).toBe(0);
});
</storycode:copy>
<a id="f-41082384fcfa-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-41082384fcfa-expression_statement-4">
it("keeps credited-author colors stable", () => {
  expect(authorColor("sam@example.com")).toBe(authorColor("sam@example.com"));
  expect(authorColor("sam@example.com")).toBeGreaterThanOrEqual(0);
});
</storycode:copy>

### Assemble `apps/web/src/lib/attribution.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/attribution.test.ts"><storycode:paste select="#f-41082384fcfa-expression_statement"/><storycode:paste select="#f-41082384fcfa-expression_statement-2"/><storycode:paste select="#f-41082384fcfa-expression_statement-3"/><storycode:paste select="#f-41082384fcfa-expression_statement-4"/></storycode:file>

## Test the behavior: `conversation_roles_supply_origin_without_git`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_blame.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-39e5027e9d27-conversation_roles_supply_origin_without_git"></a>
### `conversation_roles_supply_origin_without_git`

<storycode:copy id="f-39e5027e9d27-conversation_roles_supply_origin_without_git">
// Guarantee: docs/guarantees/lineage/authorship-display-accepts-non-git-providers.md
#[tokio::test(flavor = "multi_thread")]
async fn conversation_roles_supply_origin_without_git() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("session.md"),"<hick:session>\n<hick:user>\nHuman requirement\n</hick:user>\n<hick:assistant>\nAgent proposal\n</hick:assistant>\n</hick:session>\n").unwrap();
    let session = start(dir).await;
    let (status, body) = get(&session, "/api/blame?path=session.md").await;
    assert_eq!(status, 200);
    let lines = body["lines"].as_array().unwrap();
    assert!(
        lines
            .iter()
            .any(|line| line["line"] == 3 && line["origin"] == "human"),
        "{body}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line["line"] == 6 && line["origin"] == "agent"),
        "{body}"
    );
}

</storycode:copy>

## Test the behavior: `unicode_and_mixed_roles_do_not_fabricate_a_single_origin`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_blame.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-39e5027e9d27-unicode_and_mixed_roles_do_not_fabricate_a_single_origin"></a>
### `unicode_and_mixed_roles_do_not_fabricate_a_single_origin`

<storycode:copy id="f-39e5027e9d27-unicode_and_mixed_roles_do_not_fabricate_a_single_origin">
// Guarantee: docs/guarantees/lineage/authorship-display-accepts-non-git-providers.md
#[tokio::test(flavor = "multi_thread")]
async fn unicode_and_mixed_roles_do_not_fabricate_a_single_origin() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("session.md"),
        "<hick:session><hick:user>☕</hick:user><hick:assistant>🤖</hick:assistant></hick:session>",
    )
    .unwrap();
    let session = start(dir).await;
    let (status, body) = get(&session, "/api/blame?path=session.md").await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["lines"][0]["origin"], "unknown", "{body}");
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/components/LineageSettings.tsx](../../../mechanisms/apps/web/src/components/LineageSettings.tsx.md#output-6f1fcaffbc4e)
- [apps/web/src/editor/blameGutter.ts](../the-blame-column-is-optional-and-honest/contract.md#output-cd7f4a46ce4a)
- [apps/web/src/editor/useBlame.ts](../../../mechanisms/apps/web/src/editor/useBlame.ts.md#output-9b02a0106d13)
- [apps/web/src/lib/attribution.ts](../../../mechanisms/apps/web/src/lib/attribution.ts.md#output-42f83e0db578)
- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)

### Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../mechanisms/apps/web/e2e/browserEmbedding.spec.ts.md#output-31019d3567c7)
- [apps/web/src/editor/blameGutter.controls.test.tsx](../../../mechanisms/apps/web/src/editor/blameGutter.controls.test.tsx.md#output-de1d44118481)
- [apps/web/src/editor/blameGutter.test.ts](../the-blame-column-is-optional-and-honest/contract.md#output-3d8799779e43)
- [apps/web/src/lib/attribution.test.ts](contract.md#output-41082384fcfa)
- [crates/hickory-cli/tests/serve_blame.rs](../../../mechanisms/crates/hickory-cli/tests/serve_blame.rs.md#output-39e5027e9d27)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
