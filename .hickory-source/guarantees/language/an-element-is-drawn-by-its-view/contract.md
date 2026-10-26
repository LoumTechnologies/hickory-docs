<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Element Is Drawn By Its View

For: Authors embedding code in Markdown and maintainers changing the document grammar.

Given a block the editor renders in place of its source — a cell, a
diagram, a formula, a table, a picture — when the Document view mounts
something in the block's place, then what it mounts is the one view
registered for that block's `kind` in `apps/web/src/elements`, and which
blocks are renderable at all is what those views say they draw. The
editor itself has no branch per element: it asks the registry which view
draws a block, folds the block's lines, and hands the view its slot and
one context object. A cell's Run goes through the element's `run` action
(`POST /api/docs/:id/blocks/:at/:action`) when the render has given the
cell a span to address, and through the older run route until it has.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/language/an-element-is-drawn-by-its-view.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-e188c4134b98"></a>
## `apps/web/src/elements/index.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Trace conversation passages without confusing context with support](../../agent/an-answer-in-the-agent-pane-has-ribbons/contract.md), [An Element Is Drawn By Its View](contract.md).

<a id="f-e188c4134b98-KINDS"></a>
### `KINDS`

Protects docs/guarantees/language/an-element-is-drawn-by-its-view.md:
one view per kind, and the walk asks the views which blocks they draw.

<storycode:copy id="f-e188c4134b98-KINDS">
// Protects docs/guarantees/language/an-element-is-drawn-by-its-view.md:
// one view per kind, and the walk asks the views which blocks they draw.
import { describe, expect, it } from "vitest";

import { parseHickDoc } from "../editor/hickDoc";
import { elementViews, renderableBlocks, slotKindOf } from "./index";
import type { SlotKind } from "./types";

const KINDS: SlotKind[] = [
  "dataset",
  "patch",
  "exec",
  "output",
  "diagram",
  "math",
  "table",
  "picture",
  // The conversation's elements: a session is a document, drawn by these.
  "session-user",
  "session-assistant",
  "session-tool",
  "session-tool-result",
  "session-read",
  "session-wrote",
  "session-context",
  "session-observation",
  "session-action",
  "session-reasoning",
  "session-input",
  "session-meta",
];

</storycode:copy>
<a id="f-e188c4134b98-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-e188c4134b98-expression_statement">
describe("the element views", () => {
  it("has exactly one view per kind, and each view names its kind", () => {
    expect(Object.keys(elementViews).sort()).toEqual([...KINDS].sort());
    for (const kind of KINDS) expect(elementViews[kind].kind).toBe(kind);
  });

  it("claims a file only when it writes a picture", () => {
    const { blocks } = parseHickDoc(
      '<hick:file path="chart.svg">x</hick:file>\n<hick:file path="a.py">y</hick:file>\n',
    );
    expect(slotKindOf(blocks[0])).toBe("picture");
    expect(slotKindOf(blocks[1])).toBeNull();
  });

  it("lists renderable blocks in document order, and nothing else", () => {
    const src =
      '<hick:copy id="c">z</hick:copy>\n<hick:exec container="c">a</hick:exec>\n<hick:math>x^2</hick:math>\n<hick:table>1,2</hick:table>\n<hick:diagram>g</hick:diagram>\n';
    const structure = parseHickDoc(src);
    expect(renderableBlocks(structure).map((b) => b.name)).toEqual([
      "exec",
      "math",
      "table",
      "diagram",
    ]);
  });

  it("draws fenced exec and derived output blocks", () => {
    const structure = parseHickDoc(
      "```shell container=\"reporter\" show=\"output\"\necho hello\n```\n\n```output-for=\"#reporter-1\" hash=\"x\" input-hash=\"y\" exit=\"0\"\nhello\n```\n",
    );
    expect(renderableBlocks(structure).map((block) => block.name)).toEqual(["exec", "output"]);
  });
});
</storycode:copy>

### Assemble `apps/web/src/elements/index.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/elements/index.test.ts"><storycode:paste select="#f-e188c4134b98-KINDS"/><storycode:paste select="#f-e188c4134b98-expression_statement"/></storycode:file>

<a id="output-3774657b8b32"></a>
## `apps/web/src/elements/types.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An Element Is Drawn By Its View](contract.md).

<a id="f-3774657b8b32-SlotKind"></a>
### `SlotKind`

The frontend half of the element registry.
The server declares each element once (`crates/hick-blocks`): its tag,
its attributes, how it renders to a block and which actions it answers.
This is the mirror: one view per block `kind`, keyed by the same string
that comes over the wire, so adding an element is one Rust file and one
folder here — never a new branch in the editor.
A view draws a RENDERED block: the editor has folded the block's source
lines away (see editor/rendered.ts) and mounted an element in their
place; the view fills it. It gets the slot (where and what) and a context
(what the document around it knows), and returns what to draw.

<storycode:copy id="f-3774657b8b32-SlotKind">
// The frontend half of the element registry.
//
// The server declares each element once (`crates/hick-blocks`): its tag,
// its attributes, how it renders to a block and which actions it answers.
// This is the mirror: one view per block `kind`, keyed by the same string
// that comes over the wire, so adding an element is one Rust file and one
// folder here — never a new branch in the editor.
//
// A view draws a RENDERED block: the editor has folded the block's source
// lines away (see editor/rendered.ts) and mounted an element in their
// place; the view fills it. It gets the slot (where and what) and a context
// (what the document around it knows), and returns what to draw.

import type { ReactNode } from "react";
import type { EditorView } from "@codemirror/view";

import type { Block, DiagramBlock, ExecBlock, ExecutorInfo, SessionLink } from "../api/types";
import type { HickBlock } from "../editor/hickDoc";
import type { RenderedSlot } from "../editor/rendered";
import type { TableLayout } from "../components/TablePanel";

/** The kinds of block the editor renders in place of their source. */
export type SlotKind =
  | "dataset"
  | "patch"
  | "exec"
  | "output"
  | "diagram"
  | "math"
  | "table"
  | "picture"
  | "session-user"
  | "session-assistant"
  | "session-tool"
  | "session-tool-result"
  | "session-read"
  | "session-wrote"
  | "session-context"
  | "session-observation"
  | "session-action"
  | "session-reasoning"
  | "session-input"
  | "session-meta";

</storycode:copy>
<a id="f-3774657b8b32-SlotContext"></a>
### `SlotContext`

What the document around a rendered block knows, handed to every view.

<storycode:copy id="f-3774657b8b32-SlotContext">
/** What the document around a rendered block knows, handed to every view. */
export interface SlotContext {
  docId?: string;
  /** Frozen final-artifact closure and A. Never borrowed from the working tree. */
  capturedProject?: { activePatch?: string | null; draftPending?: (id: string, pending: boolean) => void; editPatch?: (sources: Record<string, string>) => void; filesystemOnly?: boolean; tools?: boolean; entry: string; document?: string; sources: Record<string, string>; input: import("../editor/hickLang").ProjectEntry[] };
  /** The live editor, when mounted. */
  view: EditorView | null;
  /** This document's path (for assets and per-table state), or null. */
  path: string | null;
  /** The server's rendered exec blocks — statuses and transcripts. */
  execBlocks: ExecBlock[];
  /** The server's rendered diagram blocks, pastes resolved. */
  diagramBlocks: DiagramBlock[];
  /** A session's blocks from the server, for the conversation's cards. */
  sessionBlocks?: Block[];
  /** The provenance each answer rests on, keyed by the answer's block
   * start: what the turn read, wrote and pointed at, as chips under it. */
  sessionLinksAt?: ReadonlyMap<number, readonly SessionLink[]>;
  runningCells: Set<string>;
  /** Block starts whose transcript is revealed (the rail's Replay). */
  replaying: readonly number[];
  executor?: ExecutorInfo;
  tableLayouts?: Record<string, TableLayout>;
  onTableLayout?: (key: string, layout: TableLayout) => void;
  /** Replace the slot's block content in the document; `origin` names the
   * user event for the undo history. */
  replaceBlockContent: (slot: RenderedSlot, text: string, origin?: string) => boolean | void;
}

</storycode:copy>
<a id="f-3774657b8b32-ElementView"></a>
### `ElementView`

<storycode:copy id="f-3774657b8b32-ElementView">
export interface ElementView {
  kind: SlotKind;
  /** Whether this view draws the given structure block. */
  draws: (block: HickBlock) => boolean;
  /** Draw the rendered block. */
  render: (slot: RenderedSlot, cx: SlotContext) => ReactNode;
}
</storycode:copy>

### Assemble `apps/web/src/elements/types.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/elements/types.ts"><storycode:paste select="#f-3774657b8b32-SlotKind"/><storycode:paste select="#f-3774657b8b32-SlotContext"/><storycode:paste select="#f-3774657b8b32-ElementView"/></storycode:file>

<a id="output-edaa67b6460c"></a>
## `apps/web/src/lib/blockMatch.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An Element Is Drawn By Its View](contract.md).

<a id="f-edaa67b6460c-overlap"></a>
### `overlap`

Matching what the editor has (a block's source span as currently typed)
to what the server rendered (a block with a span from the last render).
Spans drift while a person edits above a cell, so the match is best
overlap first and ordinal as the fallback.

<storycode:copy id="f-edaa67b6460c-overlap">
// Matching what the editor has (a block's source span as currently typed)
// to what the server rendered (a block with a span from the last render).
// Spans drift while a person edits above a cell, so the match is best
// overlap first and ordinal as the fallback.

import type { ExecBlock } from "../api/types";
import { execBlocksOf } from "../editor/hickDoc";
import type { HickDocStructure } from "../editor/hickDoc";

function overlap(a: [number, number], b: [number, number]): number {
  return Math.max(0, Math.min(a[1], b[1]) - Math.max(a[0], b[0]));
}

</storycode:copy>
<a id="f-edaa67b6460c-matchBlock"></a>
### `matchBlock`

The span-overlap-else-ordinal match, for any server block list.

<storycode:copy id="f-edaa67b6460c-matchBlock">
/** The span-overlap-else-ordinal match, for any server block list. */
export function matchBlock<T extends { span: [number, number] }>(
  slot: { span: [number, number]; index: number },
  blocks: readonly T[],
): T | undefined {
  let best: T | undefined;
  let bestOverlap = 0;
  for (const b of blocks) {
    const o = overlap(slot.span, b.span);
    if (o > bestOverlap) {
      bestOverlap = o;
      best = b;
    }
  }
  return best ?? blocks[slot.index];
}

</storycode:copy>
<a id="f-edaa67b6460c-matchExecBlock"></a>
### `matchExecBlock`

Match a widget slot (source span of the exec block as currently typed) to
the server-rendered exec block: best span overlap first, ordinal fallback
(spans drift while the user edits above a cell).

<storycode:copy id="f-edaa67b6460c-matchExecBlock">
/**
 * Match a widget slot (source span of the exec block as currently typed) to
 * the server-rendered exec block: best span overlap first, ordinal fallback
 * (spans drift while the user edits above a cell).
 */
export function matchExecBlock(
  slot: { span: [number, number]; index: number },
  blocks: ExecBlock[],
): ExecBlock | undefined {
  return matchBlock(slot, blocks);
}

</storycode:copy>
<a id="f-edaa67b6460c-assertionStates"></a>
### `assertionStates`

Live state of a diagram's assertions: each `asserts` id resolved to the
exec cell carrying that id in the SOURCE (the server's exec ids are
`container:line`, not the tag's own id), then that cell's last run status.
Unknown when the id names no cell, the cell has never run or is running,
or its result is stale — the panel says "checked by", never "passing",
about anything it does not know.

<storycode:copy id="f-edaa67b6460c-assertionStates">
/**
 * Live state of a diagram's assertions: each `asserts` id resolved to the
 * exec cell carrying that id in the SOURCE (the server's exec ids are
 * `container:line`, not the tag's own id), then that cell's last run status.
 * Unknown when the id names no cell, the cell has never run or is running,
 * or its result is stale — the panel says "checked by", never "passing",
 * about anything it does not know.
 */
export function assertionStates(
  structure: HickDocStructure,
  execBlocks: ExecBlock[],
  asserts: readonly string[],
  runningCells: Set<string>,
): { id: string; state: "passing" | "failing" | "unknown" }[] {
  const execs = execBlocksOf(structure);
  return asserts.map((id) => {
    const index = execs.findIndex((exec) => exec.attrs.id === id);
    const source = index >= 0 ? execs[index] : undefined;
    const block = source
      ? matchExecBlock({ span: [source.from, source.to], index }, execBlocks)
      : undefined;
    const state =
      !block || runningCells.has(block.id)
        ? ("unknown" as const)
        : block.status === "ok"
          ? ("passing" as const)
          : block.status === "failed"
            ? ("failing" as const)
            : ("unknown" as const);
    return { id, state };
  });
}
</storycode:copy>

### Assemble `apps/web/src/lib/blockMatch.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/blockMatch.ts"><storycode:paste select="#f-edaa67b6460c-overlap"/><storycode:paste select="#f-edaa67b6460c-matchBlock"/><storycode:paste select="#f-edaa67b6460c-matchExecBlock"/><storycode:paste select="#f-edaa67b6460c-assertionStates"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/api/client.ts](../../../mechanisms/apps/web/src/api/client.ts.md#output-1e909407d4cd)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/rendered.ts](../../../mechanisms/apps/web/src/editor/rendered.ts.md#output-4f5444d21411)
- [apps/web/src/elements/index.ts](../../../mechanisms/apps/web/src/elements/index.ts.md#output-9b7b2fa9b3c3)
- [apps/web/src/elements/types.ts](contract.md#output-3774657b8b32)
- [apps/web/src/lib/blockMatch.ts](contract.md#output-edaa67b6460c)
- [apps/web/src/views/documentSession.tsx](../../../mechanisms/apps/web/src/views/documentSession.tsx.md#output-bfb59577f48b)

### Tests and fixtures

- [apps/web/src/elements/index.test.ts](contract.md#output-e188c4134b98)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
