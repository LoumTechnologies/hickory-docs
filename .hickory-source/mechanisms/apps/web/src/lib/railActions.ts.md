<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/railActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-499155452cf6"></a>
## `apps/web/src/lib/railActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Generated Picture Shows As A Picture](../../../../../guarantees/authoring/a-generated-picture-shows-as-a-picture/contract.md), [A Table Is A Dataset And A Paragraph](../../../../../guarantees/authoring/a-table-is-a-dataset-and-a-paragraph/contract.md), [An Equation Renders And Shows Its Source](../../../../../guarantees/authoring/an-equation-renders-and-shows-its-source/contract.md), [The Gutters Never Skip A Number](../../../../../guarantees/authoring/the-gutters-never-skip-a-number/contract.md), [Keep editor actions attached to the view they affect](../../../../../guarantees/editor-intelligence/actions-belong-to-their-subject/contract.md).

<a id="f-499155452cf6-RailAction"></a>
### `RailAction`

What each card offers on the action rail.
The rail used to carry one icon per card, and that icon did the only thing
a card could do: swap the block between its result and its source. Every
other control — Run, Replay, "source" — lived INSIDE the rendered card, as
a button floating over the thing it acted on.
They are all on the rail now. A rendered card is something to read; the
things you can click belong in the one column that exists to be clicked.
So a card contributes a short COLUMN of icons rather than a single one,
stacked downward from its own line by the same `stackIcons` that keeps two
crowded cards apart — an action never rises above the line it belongs to.
Pure — kind and a couple of flags in, action list out — because "which
icons does a failed cell with a transcript show?" is a question worth
answering in a test rather than by clicking through a browser.

<storycode:copy id="f-499155452cf6-RailAction">
// What each card offers on the action rail.
//
// The rail used to carry one icon per card, and that icon did the only thing
// a card could do: swap the block between its result and its source. Every
// other control — Run, Replay, "source" — lived INSIDE the rendered card, as
// a button floating over the thing it acted on.
//
// They are all on the rail now. A rendered card is something to read; the
// things you can click belong in the one column that exists to be clicked.
// So a card contributes a short COLUMN of icons rather than a single one,
// stacked downward from its own line by the same `stackIcons` that keeps two
// crowded cards apart — an action never rises above the line it belongs to.
//
// Pure — kind and a couple of flags in, action list out — because "which
// icons does a failed cell with a transcript show?" is a question worth
// answering in a test rather than by clicking through a browser.

import type { CardKind } from "../editor/cards";

/** One thing a rail icon does. */
export type RailAction = "run" | "source" | "replay" | "convert";

</storycode:copy>
<a id="f-499155452cf6-ReplayInput"></a>
### `ReplayInput`

What a cell must look like for the replay icon to be worth offering.

<storycode:copy id="f-499155452cf6-ReplayInput">
/** What a cell must look like for the replay icon to be worth offering. */
export interface ReplayInput {
  /** Server-side status of the block, when the server knows it. */
  status?: string;
  /** Whether the cell pins its output with an expect block. */
  hasExpect: boolean;
  /** How many transcript events the last run produced. */
  transcriptLength: number;
  /** Whether the cell is running right now. */
  running: boolean;
}

</storycode:copy>
<a id="f-499155452cf6-hasReplay"></a>
### `hasReplay`

Whether a cell's transcript is hidden behind a replay toggle.
A cell whose output is PINNED by an expect block does not show its
transcript by default — the expect block above it is the output, already on
numbered lines, and repeating it underneath is a second copy of the same
bytes. Replay reveals what actually ran. A cell with no expect block has
nothing standing in for its transcript, so the transcript is simply shown
and there is nothing to toggle.
Shared with CellPanel so the icon and the panel cannot disagree about
whether there is anything to replay.

<storycode:copy id="f-499155452cf6-hasReplay">
/**
 * Whether a cell's transcript is hidden behind a replay toggle.
 *
 * A cell whose output is PINNED by an expect block does not show its
 * transcript by default — the expect block above it is the output, already on
 * numbered lines, and repeating it underneath is a second copy of the same
 * bytes. Replay reveals what actually ran. A cell with no expect block has
 * nothing standing in for its transcript, so the transcript is simply shown
 * and there is nothing to toggle.
 *
 * Shared with CellPanel so the icon and the panel cannot disagree about
 * whether there is anything to replay.
 */
export function hasReplay({
  status,
  hasExpect,
  transcriptLength,
  running,
}: ReplayInput): boolean {
  if (running || transcriptLength === 0 || !hasExpect) return false;
  return status === "ok" || status === "failed";
}

</storycode:copy>
<a id="f-499155452cf6-actionsFor"></a>
### `actionsFor`

The icons a card contributes, top to bottom.
Order is deliberate and stable: the action you reach for most is nearest
the line. Run comes first because a cell exists to be run; source second
because it is the way back; replay last because it is the rare one, and
because an icon that comes and goes must not push the others around.

<storycode:copy id="f-499155452cf6-actionsFor">
/**
 * The icons a card contributes, top to bottom.
 *
 * Order is deliberate and stable: the action you reach for most is nearest
 * the line. Run comes first because a cell exists to be run; source second
 * because it is the way back; replay last because it is the rare one, and
 * because an icon that comes and goes must not push the others around.
 */
export function actionsFor(
  kind: CardKind,
  options: { replay?: boolean; source?: boolean } = {},
): RailAction[] {
  switch (kind) {
    case "exec": {
      // A cell that draws a picture has no source toggle of its own: the
      // picture block above it owns that verb, and offering it twice is how
      // a two-state thing grows a third state.
      const own: RailAction[] = options.source === false ? ["run"] : ["run", "source"];
      return options.replay ? [...own, "replay"] : own;
    }
    case "dataset":
    case "diagram":
    // An equation is a picture too: the only verb it has is "show me what I
    // actually typed".
    case "math":
    // A table's grid IS the edited thing, so its one verb is the way back to
    // the CSV — which is what a reviewer reads in the diff.
    case "table":
    // A generated picture has the same one verb: show me the code that drew
    // this. Running it belongs to the cell INSIDE the file block, which keeps
    // its own Run icon — the picture card never grows a second one.
    case "picture":
      return ["source"];
    case "fence":
      return ["convert"];
  }
}
</storycode:copy>

### Assemble `apps/web/src/lib/railActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/railActions.ts"><storycode:paste select="#f-499155452cf6-RailAction"/><storycode:paste select="#f-499155452cf6-ReplayInput"/><storycode:paste select="#f-499155452cf6-hasReplay"/><storycode:paste select="#f-499155452cf6-actionsFor"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-49410ea38e0d"></a>
## `apps/web/src/lib/railActions.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Generated Picture Shows As A Picture](../../../../../guarantees/authoring/a-generated-picture-shows-as-a-picture/contract.md), [Keep editor actions attached to the view they affect](../../../../../guarantees/editor-intelligence/actions-belong-to-their-subject/contract.md).

<a id="f-49410ea38e0d-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-49410ea38e0d-expression_statement">
import { describe, expect, it } from "vitest";

import { actionsFor, hasReplay } from "./railActions";

describe("which icons a card puts on the rail", () => {
  it("gives a cell its run and source icons, in that order", () => {
    // Order is the contract: the icon nearest the line is the one a cell
    // exists for, and a reader learns the column's shape once.
    expect(actionsFor("exec")).toEqual(["run", "source"]);
  });

  it("adds replay last, so appearing does not move the other two", () => {
    expect(actionsFor("exec", { replay: true })).toEqual(["run", "source", "replay"]);
  });

  it("gives a diagram only the way back to its source", () => {
    // A diagram has no verb of its own — nothing to run, nothing to replay.
    expect(actionsFor("diagram")).toEqual(["source"]);
  });

  it("takes the source icon off the cell that draws a picture", () => {
    // Two states, not three: the picture owns the toggle, the cell keeps Run.
    expect(actionsFor("exec", { source: false })).toEqual(["run"]);
    expect(actionsFor("exec", { source: false, replay: true })).toEqual(["run", "replay"]);
  });

  it("gives a picture only the way back to the code that drew it", () => {
    // Running belongs to the cell inside the file block, which has its own
    // Run icon; a second one here would run the same cell twice over.
    expect(actionsFor("picture")).toEqual(["source"]);
  });

  it("gives a prose fence only the converter", () => {
    expect(actionsFor("fence")).toEqual(["convert"]);
  });
});

</storycode:copy>
<a id="f-49410ea38e0d-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-49410ea38e0d-expression_statement-2">
describe("whether a cell has anything to replay", () => {
  const cell = {
    status: "ok",
    hasExpect: true,
    transcriptLength: 3,
    running: false,
  };

  it("offers replay when an expect block is standing in for the output", () => {
    expect(hasReplay(cell)).toBe(true);
  });

  it("offers nothing while the cell is running — the transcript is already live", () => {
    expect(hasReplay({ ...cell, running: true })).toBe(false);
  });

  it("offers nothing when the transcript is already the visible output", () => {
    // No expect block means nothing stands in for the transcript, so it is
    // simply shown and there is no second state to toggle to.
    expect(hasReplay({ ...cell, hasExpect: false })).toBe(false);
  });

  it("offers nothing when there is no transcript to reveal", () => {
    expect(hasReplay({ ...cell, transcriptLength: 0 })).toBe(false);
  });

  it("offers nothing for a cell the server has not run", () => {
    expect(hasReplay({ ...cell, status: undefined })).toBe(false);
  });

  it("covers a failed cell too — the diff is a summary, not what ran", () => {
    expect(hasReplay({ ...cell, status: "failed" })).toBe(true);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/railActions.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/railActions.test.ts"><storycode:paste select="#f-49410ea38e0d-expression_statement"/><storycode:paste select="#f-49410ea38e0d-expression_statement-2"/></storycode:file>


</storycode:doc>
