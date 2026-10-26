<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Markdown Paragraphs Unwrap By Default

For: Authors editing executable documents and the files those documents produce.

Given an editable document in the Document view, when it opens, receives source
from the live room, or receives pasted or dropped text, then ordinary Markdown
paragraphs have their soft line breaks replaced with spaces. The editor wraps
the resulting paragraph visually at the ruler's measure.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/paragraphs-unwrap-by-default.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-df418326657a"></a>
## `apps/web/src/editor/unwrapParagraphs.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Markdown Paragraphs Unwrap By Default](contract.md).

<a id="f-df418326657a-reflow"></a>
### `reflow`

<storycode:copy id="f-df418326657a-reflow">
// @vitest-environment jsdom
// Guarantee: docs/guarantees/authoring/paragraphs-unwrap-by-default.md
import { afterEach, describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { history, undo } from "@codemirror/commands";
import { paragraphUnwrapChanges, unwrapParagraphs } from "./unwrapParagraphs";
import { loadUnwrapParagraphs, saveUnwrapParagraphs, UNWRAP_PARAGRAPHS_KEY } from "../lib/unwrapParagraphs";

const reflow = (doc: string) => EditorState.create({ doc }).update({
  changes: paragraphUnwrapChanges(doc),
}).state.doc.toString();

</storycode:copy>
<a id="f-df418326657a-views"></a>
### `views`

<storycode:copy id="f-df418326657a-views">
const views: EditorView[] = [];
</storycode:copy>
<a id="f-df418326657a-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-df418326657a-expression_statement">
afterEach(() => {
  for (const view of views.splice(0)) view.destroy();
  localStorage.removeItem(UNWRAP_PARAGRAPHS_KEY);
});
</storycode:copy>
<a id="f-df418326657a-editor"></a>
### `editor`

<storycode:copy id="f-df418326657a-editor">
function editor(doc: string, readOnly = false, onOpen = true) {
  const view = new EditorView({ state: EditorState.create({
    doc, extensions: [history(), unwrapParagraphs({ onOpen }), EditorState.readOnly.of(readOnly)],
  }) });
  views.push(view);
  return view;
}

</storycode:copy>
<a id="f-df418326657a-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-df418326657a-expression_statement-2">
describe("paragraph unwrapping", () => {
  it("keeps seeded Untitled bytes clean through sync, then reflows a real paste", async () => {
    const original = "An introduction\nwith a soft break.";
    const view = editor(original, false, false);
    await Promise.resolve();
    expect(view.state.doc.toString()).toBe(original);
    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: original } });
    await Promise.resolve();
    expect(view.state.doc.toString()).toBe(original);
    view.dispatch({ changes: { from: view.state.doc.length, insert: "\n\nPasted prose\ncontinues." }, userEvent: "input.paste" });
    await Promise.resolve();
    expect(view.state.doc.toString()).toBe("An introduction with a soft break.\n\nPasted prose continues.");
  });

  it("joins soft breaks without merging paragraphs or changing inline markup", () => {
    const input = "# Heading\n\nA **long** paragraph\nwith a [link](https://example.com)\nand more text.\n\nAnother paragraph\nwith text.\n";
    const output = "# Heading\n\nA **long** paragraph with a [link](https://example.com) and more text.\n\nAnother paragraph with text.\n";
    expect(reflow(input)).toBe(output);
    expect(reflow(output)).toBe(output);
  });

  it("preserves explicit breaks, lists, quotes, tables, code, HTML and YAML", () => {
    const input = [
      "---\nsummary: |\n  first line\n  second line\n---",
      "First  \nsecond\\\nthird",
      "- item\n  continuation\n- next item",
      "> quote\n> continuation",
      "a | b\n--- | ---\nx | y\nz | w",
      "```md\ncode paragraph\ncontinues\n```",
      "    indented code\n    more code",
      "<div>\nHTML content\nmore content\n</div>",
      "Text with `multi\nline code`.",
      "Text [link\ntext](url).",
      "$$\nx = 1\ny = 2\n$$",
    ].join("\n\n");
    expect(reflow(input)).toBe(input);
  });

  it("unwraps prose in containers but leaves payload bytes and tag attributes intact", () => {
    const input = '<hick:doc>\nA paragraph\ncontinues.\n\n<hick:copy id="source">\nOriginal transcript\ncontinues.\n</hick:copy>\n<hick:file path="out.md">\nFile content\ncontinues.\n</hick:file>\n<hick:exec container="shell">\necho first\necho second\n</hick:exec>\n</hick:doc>';
    expect(reflow(input)).toBe(input.replace("A paragraph\ncontinues.", "A paragraph continues."));
    expect(reflow('<hick:doc title="first\nsecond">\nProse\ncontinues.\n</hick:doc>')).toContain('title="first\nsecond"');
  });

  it("is on by default, persists off, and leaves read-only documents alone", async () => {
    expect(loadUnwrapParagraphs()).toBe(true);
    saveUnwrapParagraphs(false);
    expect(loadUnwrapParagraphs()).toBe(false);
    const off = editor("A paragraph\ncontinues.");
    await Promise.resolve();
    expect(off.state.doc.toString()).toBe("A paragraph\ncontinues.");
    saveUnwrapParagraphs(true);
    const readOnly = editor("A paragraph\ncontinues.", true);
    await Promise.resolve();
    expect(readOnly.state.doc.toString()).toBe("A paragraph\ncontinues.");
  });

  it("reflows initial and synced text as an undoable edit without undo loops", async () => {
    const view = editor("A paragraph\ncontinues.");
    await Promise.resolve();
    expect(view.state.doc.toString()).toBe("A paragraph continues.");
    expect(undo(view)).toBe(true);
    await Promise.resolve();
    expect(view.state.doc.toString()).toBe("A paragraph\ncontinues.");
    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "New text\nfrom disk." } });
    await Promise.resolve();
    expect(view.state.doc.toString()).toBe("New text from disk.");
  });

  it("reflows pasted paragraphs, maps the caret, and does not fight Enter", async () => {
    const view = editor("");
    await Promise.resolve();
    view.dispatch({ changes: { from: 0, insert: "Pasted text\ncontinues." }, selection: { anchor: 22 }, userEvent: "input.paste" });
    await Promise.resolve();
    expect(view.state.doc.toString()).toBe("Pasted text continues.");
    expect(view.state.selection.main.head).toBe(view.state.doc.length);
    view.dispatch({ changes: { from: 11, insert: "\n" }, userEvent: "input.type" });
    await Promise.resolve();
    expect(view.state.doc.toString()).toContain("\n");
    saveUnwrapParagraphs(false);
    view.dispatch({ changes: { from: view.state.doc.length, insert: "\nmore pasted text" }, userEvent: "input.paste" });
    await Promise.resolve();
    expect(view.state.doc.toString()).toContain("\nmore pasted text");
  });
});
</storycode:copy>

### Assemble `apps/web/src/editor/unwrapParagraphs.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/unwrapParagraphs.test.ts"><storycode:paste select="#f-df418326657a-reflow"/><storycode:paste select="#f-df418326657a-views"/><storycode:paste select="#f-df418326657a-expression_statement"/><storycode:paste select="#f-df418326657a-editor"/><storycode:paste select="#f-df418326657a-expression_statement-2"/></storycode:file>

<a id="output-e98ba828b293"></a>
## `apps/web/src/editor/unwrapParagraphs.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Markdown Paragraphs Unwrap By Default](contract.md).

<a id="f-e98ba828b293-paragraphUnwrapChanges"></a>
### `paragraphUnwrapChanges`

<storycode:copy id="f-e98ba828b293-paragraphUnwrapChanges">
import { markdownLanguage } from "@codemirror/lang-markdown";
import { isolateHistory } from "@codemirror/commands";
import { Transaction, type ChangeSpec, type Extension } from "@codemirror/state";
import { ViewPlugin, type EditorView, type ViewUpdate } from "@codemirror/view";
import { loadUnwrapParagraphs } from "../lib/unwrapParagraphs";
import { parseHickDoc, verbatimRanges } from "./hickDoc";

/** Join Markdown soft breaks only in ordinary paragraphs. Hickory's own
 * parser supplies the byte-preserving payload and tag boundaries. */
export function paragraphUnwrapChanges(source: string): ChangeSpec[] {
  const structure = parseHickDoc(source);
  const excluded = [
    ...verbatimRanges(structure.blocks),
    ...structure.tags.map((tag) => [tag.from, tag.to] as [number, number]),
  ];
  // YAML is data, including folded/literal scalars that look like prose.
  const frontmatter = /^(?:\uFEFF)?---[ \t]*\n/.exec(source);
  if (frontmatter) {
    const close = /^(?:---|\.\.\.)[ \t]*(?:\n|$)/m.exec(source.slice(frontmatter[0].length));
    excluded.push([0, close ? frontmatter[0].length + close.index + close[0].length : source.length]);
  }
  // Blank masked regions keep offsets stable and make container tag lines
  // paragraph boundaries, allowing prose inside <hick:doc> to be parsed.
  const chars = source.split("");
  for (const [from, to] of excluded) {
    for (let i = from; i < to; i++) if (chars[i] !== "\n") chars[i] = " ";
  }
  const changes: ChangeSpec[] = [];
  markdownLanguage.parser.parse(chars.join("")).iterate({
    enter(node) {
      // Lists and quotes have structural line prefixes; leave those intact.
      if (node.name !== "Paragraph" || node.node.parent?.name !== "Document") return;
      if (excluded.some(([from, to]) => from < node.to && to > node.from)) return false;
      const paragraph = source.slice(node.from, node.to);
      // Multi-line inline code, links and HTML can depend on their bytes.
      let sensitive = false;
      node.node.toTree().iterate({
        enter(child) {
          if (["InlineCode", "Link", "Image", "HTMLTag"].includes(child.name) &&
              paragraph.slice(child.from, child.to).includes("\n")) sensitive = true;
        },
      });
      if (sensitive || paragraph.includes("$$")) return false;
      for (const match of paragraph.matchAll(/\n[ \t]*/g)) {
        const before = paragraph.slice(0, match.index);
        // Two trailing spaces or an odd run of backslashes is an explicit
        // Markdown hard break, and must keep its newline.
        const slashes = /\\+$/.exec(before)?.[0].length ?? 0;
        if (/ {2,}$/.test(before) || slashes % 2 === 1) continue;
        changes.push({ from: node.from + match.index, to: node.from + match.index + match[0].length, insert: " " });
      }
      return false;
    },
  });
  return changes;
}

</storycode:copy>
<a id="f-e98ba828b293-unwrapParagraphs"></a>
### `unwrapParagraphs`

Open/sync and paste reflow prose; typing Enter and Undo remain literal.
Dispatch after the update so CRDT writes and selection mapping use the
normal editor path, and the formatting is a separate undoable act.

<storycode:copy id="f-e98ba828b293-unwrapParagraphs">
/** Open/sync and paste reflow prose; typing Enter and Undo remain literal.
 * Dispatch after the update so CRDT writes and selection mapping use the
 * normal editor path, and the formatting is a separate undoable act. */
export function unwrapParagraphs({ onOpen = true }: { onOpen?: boolean } = {}): Extension {
  return ViewPlugin.fromClass(class {
    private pending = false;
    private destroyed = false;
    private activated = onOpen;
    private shouldRun = onOpen;
    constructor(private view: EditorView) { if (onOpen) this.schedule(); }
    update(update: ViewUpdate) {
      if (!update.docChanged) return;
      // A seeded Untitled introduction is clean until the person edits it.
      // Its initial CRDT sync must not count as that first edit either.
      if (!this.activated) {
        if (!update.transactions.some((tr) => tr.annotation(Transaction.userEvent) !== undefined)) return;
        this.activated = true;
      }
      this.shouldRun = update.transactions.every((tr) =>
        tr.annotation(Transaction.userEvent) === undefined ||
        tr.isUserEvent("input.paste") || tr.isUserEvent("input.drop"),
      );
      if (this.shouldRun) this.schedule();
    }
    private schedule() {
      if (this.pending) return;
      this.pending = true;
      queueMicrotask(() => {
        this.pending = false;
        if (this.destroyed || !this.shouldRun || this.view.state.readOnly || !loadUnwrapParagraphs()) return;
        const changes = paragraphUnwrapChanges(this.view.state.doc.toString());
        if (changes.length) this.view.dispatch({
          changes,
          annotations: [Transaction.userEvent.of("input.unwrap"), isolateHistory.of("full")],
        });
      });
    }
    destroy() { this.destroyed = true; }
  });
}
</storycode:copy>

### Assemble `apps/web/src/editor/unwrapParagraphs.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/unwrapParagraphs.ts"><storycode:paste select="#f-e98ba828b293-paragraphUnwrapChanges"/><storycode:paste select="#f-e98ba828b293-unwrapParagraphs"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/unwrapParagraphs.ts](contract.md#output-e98ba828b293)
- [apps/web/src/lib/unwrapParagraphs.ts](../../../mechanisms/apps/web/src/lib/unwrapParagraphs.ts.md#output-287fdbcc749d)
- [apps/web/src/views/SettingsView.tsx](../../../mechanisms/apps/web/src/views/SettingsView.tsx.md#output-38894484eb8f)

### Tests and fixtures

- [apps/web/src/editor/DocumentEditor.test.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.test.tsx.md#output-71bbcfe029bb)
- [apps/web/src/editor/unwrapParagraphs.test.ts](contract.md#output-df418326657a)
- [apps/web/src/views/SettingsView.test.tsx](../../../mechanisms/apps/web/src/views/SettingsView.tsx.md#output-5bfac230ae5d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
