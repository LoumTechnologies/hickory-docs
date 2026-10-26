<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Ingested Block Is Marked In The Document View Too

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a `<hick:ingested>` block's content sitting in the Document view —
plain, editable text, since the owning exec cell's card deliberately stops
rendering right before it — when the view draws it, then it carries the
same `cm-prov-ingested` dashed-underline treatment the Output pane's
per-character lineage already gives ingested bytes there, not zero visual
marking.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/an-ingested-block-is-marked-in-the-document-view-too.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-ec280d7bc551"></a>
## `apps/web/src/editor/wysiwyg.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An Ingested Block Is Marked In The Document View Too](contract.md).

<a id="f-ec280d7bc551-view"></a>
### `view`

@vitest-environment jsdom
The exec card that owns a `<hick:ingested>` block stops rendering exactly
where the ingested content begins (rendered.ts), so those bytes stay
editable as plain text. Before this, that plain text carried no visual
signal that it arrived from a real run — the same `cm-prov-ingested`
treatment the Output pane already gives ingested bytes must also reach
the Document view.

<storycode:copy id="f-ec280d7bc551-view">
// @vitest-environment jsdom
// Guarantee: docs/guarantees/editor-intelligence/an-ingested-block-is-marked-in-the-document-view-too.md
//
// The exec card that owns a `<hick:ingested>` block stops rendering exactly
// where the ingested content begins (rendered.ts), so those bytes stay
// editable as plain text. Before this, that plain text carried no visual
// signal that it arrived from a real run — the same `cm-prov-ingested`
// treatment the Output pane already gives ingested bytes must also reach
// the Document view.
import { afterEach, describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { EnvRegistry, wysiwyg } from "./wysiwyg";

let view: EditorView | undefined;

</storycode:copy>
<a id="f-ec280d7bc551-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ec280d7bc551-expression_statement">
afterEach(() => {
  view?.destroy();
  view = undefined;
});

</storycode:copy>
<a id="f-ec280d7bc551-open"></a>
### `open`

<storycode:copy id="f-ec280d7bc551-open">
function open(doc: string) {
  view = new EditorView({
    state: EditorState.create({ doc, extensions: wysiwyg(new EnvRegistry()) }),
    parent: document.body,
  });
  return view;
}

</storycode:copy>
<a id="f-ec280d7bc551-DOC"></a>
### `DOC`

<storycode:copy id="f-ec280d7bc551-DOC">
const DOC = `<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:exec container="c" mount="v:out">
scaffold
<hick:ingested from="#scaffold" sha256="abc123" at="2026-09-01" files="1" skipped="0">
<hick:file path="app/Program.cs">real content here</hick:file>
</hick:ingested>
</hick:exec>
</hick:doc>
`;

</storycode:copy>
<a id="f-ec280d7bc551-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-ec280d7bc551-expression_statement-2">
describe("an ingested block in the Document view", () => {
  it("carries the same cm-prov-ingested class the Output pane uses", () => {
    const v = open(DOC);
    const marked = v.dom.querySelectorAll(".cm-prov-ingested");
    expect(marked.length).toBeGreaterThan(0);
    const text = Array.from(marked)
      .map((el) => el.textContent)
      .join("");
    expect(text).toContain("real content here");
  });

  it("does not mark ordinary, non-ingested text", () => {
    const v = open('<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\nordinary prose\n</hick:doc>\n');
    expect(v.dom.querySelectorAll(".cm-prov-ingested").length).toBe(0);
  });
});
</storycode:copy>

### Assemble `apps/web/src/editor/wysiwyg.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/wysiwyg.test.ts"><storycode:paste select="#f-ec280d7bc551-view"/><storycode:paste select="#f-ec280d7bc551-expression_statement"/><storycode:paste select="#f-ec280d7bc551-open"/><storycode:paste select="#f-ec280d7bc551-DOC"/><storycode:paste select="#f-ec280d7bc551-expression_statement-2"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/OutputEditorPane.tsx](../../../mechanisms/apps/web/src/components/OutputEditorPane.tsx.md#output-2571b0682164)
- [apps/web/src/editor/rendered.ts](../../../mechanisms/apps/web/src/editor/rendered.ts.md#output-4f5444d21411)
- [apps/web/src/editor/wysiwyg.ts](../../../shared/files/apps/web/src/editor/wysiwyg.ts.md)

### Tests and fixtures

- [apps/web/src/editor/rendered.test.ts](../../../mechanisms/apps/web/src/editor/rendered.ts.md#output-1e1e27143ad4)
- [apps/web/src/editor/wysiwyg.test.ts](contract.md#output-ec280d7bc551)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
