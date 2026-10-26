<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Several Cursors Edit At Once

For: Authors editing executable documents and the files those documents produce.

Given any editor in the app — a document, a plain file, a generated file —
when the person Alt+clicks, Alt+drags a column, or presses Ctrl+D (Cmd+D) on
a selection, then a second cursor or selection is added rather than the first
moved, Ctrl+Shift+L selects every occurrence, and typing, deleting and
pasting act at every cursor at once.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/several-cursors-edit-at-once.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-1ef997faf9f3"></a>
## `apps/web/src/editor/multiCursor.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Several Cursors Edit At Once](contract.md).

<a id="f-1ef997faf9f3-expression_statement"></a>
### `expression_statement`

Protects docs/guarantees/authoring/several-cursors-edit-at-once.md

<storycode:copy id="f-1ef997faf9f3-expression_statement">
// Protects docs/guarantees/authoring/several-cursors-edit-at-once.md

import { EditorSelection, EditorState } from "@codemirror/state";
import { describe, expect, it } from "vitest";
import { multipleCursors } from "./multiCursor";

describe("several cursors", () => {
  it("lets a state hold more than one selection", () => {
    const state = EditorState.create({ doc: "one\ntwo\nthree", extensions: multipleCursors() });
    expect(state.facet(EditorState.allowMultipleSelections)).toBe(true);
    const two = state.update({
      selection: EditorSelection.create([EditorSelection.cursor(0), EditorSelection.cursor(4)]),
    }).state;
    expect(two.selection.ranges).toHaveLength(2);
  });

  it("types at every cursor at once", () => {
    const state = EditorState.create({ doc: "one\ntwo", extensions: multipleCursors() });
    const two = state.update({
      selection: EditorSelection.create([EditorSelection.cursor(0), EditorSelection.cursor(4)]),
    }).state;
    const typed = two.update(two.replaceSelection("# ")).state;
    expect(typed.doc.toString()).toBe("# one\n# two");
  });

  it("collapses to one selection without the extension, which is the default this replaces", () => {
    const state = EditorState.create({ doc: "one\ntwo" });
    const two = state.update({
      selection: EditorSelection.create([EditorSelection.cursor(0), EditorSelection.cursor(4)]),
    }).state;
    expect(two.selection.ranges).toHaveLength(1);
  });
});
</storycode:copy>

### Assemble `apps/web/src/editor/multiCursor.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/multiCursor.test.ts"><storycode:paste select="#f-1ef997faf9f3-expression_statement"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/OutputEditorPane.tsx](../../../mechanisms/apps/web/src/components/OutputEditorPane.tsx.md#output-2571b0682164)
- [apps/web/src/components/PlainFilePane.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-2f891a49445f)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/multiCursor.ts](../../../mechanisms/apps/web/src/editor/multiCursor.ts.md#output-08df9c7d9da5)

### Tests and fixtures

- [apps/web/src/editor/multiCursor.test.ts](contract.md#output-1ef997faf9f3)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
