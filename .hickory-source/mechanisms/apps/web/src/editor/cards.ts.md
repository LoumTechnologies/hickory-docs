<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/cards.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-84385185ee44"></a>
## `apps/web/src/editor/cards.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Generated Picture Shows As A Picture](../../../../../guarantees/authoring/a-generated-picture-shows-as-a-picture/contract.md), [A Table Is A Dataset And A Paragraph](../../../../../guarantees/authoring/a-table-is-a-dataset-and-a-paragraph/contract.md), [An Equation Renders And Shows Its Source](../../../../../guarantees/authoring/an-equation-renders-and-shows-its-source/contract.md), [Markdown tables use the table editor](../../../../../guarantees/authoring/markdown-tables-use-the-table-editor/contract.md), [The Gutters Never Skip A Number](../../../../../guarantees/authoring/the-gutters-never-skip-a-number/contract.md).

<a id="f-84385185ee44-CardKind"></a>
### `CardKind`

What the document's action rail lists.
Every card the document used to draw IN the text — the exec cell's run
strip, the rendered diagram — is now an icon on a rail down the right-hand
edge, and its UI opens beside that icon. The reason is the gutters: a block
widget is a screen row with no document line behind it, so the numbers on
the left and the right both skip, and a person reading monospace loses the
one invariant that column is for. The environment card was moved inline for
exactly this reason (see EnvCardWidget); the rail is the same decision made
for the UI too tall to fit on a line.
The fence card is the other half: a ``` block in prose is a command nobody
wired up, and its icon offers to make it a real cell.
This module is pure — structure in, card list out — so where the rail's
icons come from is testable without a browser.

<storycode:copy id="f-84385185ee44-CardKind">
// What the document's action rail lists.
//
// Every card the document used to draw IN the text — the exec cell's run
// strip, the rendered diagram — is now an icon on a rail down the right-hand
// edge, and its UI opens beside that icon. The reason is the gutters: a block
// widget is a screen row with no document line behind it, so the numbers on
// the left and the right both skip, and a person reading monospace loses the
// one invariant that column is for. The environment card was moved inline for
// exactly this reason (see EnvCardWidget); the rail is the same decision made
// for the UI too tall to fit on a line.
//
// The fence card is the other half: a ``` block in prose is a command nobody
// wired up, and its icon offers to make it a real cell.
//
// This module is pure — structure in, card list out — so where the rail's
// icons come from is testable without a browser.

import { markdownTableBlocks } from "./markdownTables";
import { execBlocksOf, blocksNamed, pictureBlocksOf, proseFences } from "./hickDoc";
import type { HickDocStructure } from "./hickDoc";

export type CardKind = "dataset" | "exec" | "diagram" | "math" | "table" | "picture" | "fence";

</storycode:copy>
<a id="f-84385185ee44-DocCard"></a>
### `DocCard`

<storycode:copy id="f-84385185ee44-DocCard">
export interface DocCard {
  /** Stable within one document version; the rail keys its buttons on it. */
  key: string;
  kind: CardKind;
  /** Ordinal among cards of the same kind, in document order. This is what
   * matches an exec card to the server's Nth rendered exec block. */
  index: number;
  /** Where the icon sits: the start of the card's first line. */
  at: number;
  /** The source the card is about. */
  from: number;
  to: number;
  /** The rail button's accessible name and tooltip. */
  label: string;
  /** This cell draws a picture: it lives inside a `<hick:file>` block that
   * writes one. Such a cell has no source toggle of its own — the picture
   * block owns that verb, and its two states are the picture and this cell's
   * editable text. */
  insidePicture?: boolean;
}

</storycode:copy>
<a id="f-84385185ee44-CardsOptions"></a>
### `CardsOptions`

<storycode:copy id="f-84385185ee44-CardsOptions">
export interface CardsOptions {
  /** Diagrams need a rendering engine; the marketing demo ships without one
   * and must not offer a card it cannot draw. */
  diagrams?: boolean;
  /** The whole document text — fences are found by scanning it. */
  text: string;
}

</storycode:copy>
<a id="f-84385185ee44-cardsOf"></a>
### `cardsOf`

Every card in the document, in the order their icons stack down the rail.
Exec cells come from the parsed structure so a cell the server has not
rendered yet still gets an icon: an exec you just typed must be runnable
before a round-trip, and an icon that appears late reads as a bug.

<storycode:copy id="f-84385185ee44-cardsOf">
/**
 * Every card in the document, in the order their icons stack down the rail.
 *
 * Exec cells come from the parsed structure so a cell the server has not
 * rendered yet still gets an icon: an exec you just typed must be runnable
 * before a round-trip, and an icon that appears late reads as a bug.
 */
export function cardsOf(structure: HickDocStructure, options: CardsOptions): DocCard[] {
  const cards: DocCard[] = [];

  const pictures = pictureBlocksOf(structure);
  execBlocksOf(structure).forEach((block, index) => {
    const container = block.attrs.container ?? block.attrs.image ?? "";
    cards.push({
      key: `exec-${index}`,
      kind: "exec",
      index,
      at: block.from,
      from: block.from,
      to: block.to,
      label: container ? `Cell ${index + 1} — ${container}` : `Cell ${index + 1}`,
      insidePicture: pictures.some((p) => block.from > p.from && block.from < p.to),
    });
  });

  blocksNamed(structure, "dataset").forEach((block, index) => {
    cards.push({ key: `dataset-${index}`, kind: "dataset", index, at: block.from, from: block.from, to: block.to, label: `Dataset — ${block.attrs.src ?? ""}` });
  });

  if (options.diagrams !== false) {
    blocksNamed(structure, "diagram").forEach((block, index) => {
      cards.push({
        key: `diagram-${index}`,
        kind: "diagram",
        index,
        at: block.from,
        from: block.from,
        to: block.to,
        label: `Diagram ${index + 1} — ${block.attrs.renderer ?? "mermaid"}`,
      });
    });
  }

  // A `<hick:math>` block is a paragraph-sized equation, so it gets a rail
  // icon rather than the caret-reveal that inline maths uses: a reader who
  // wants the LaTeX of a displayed equation should not have to put the cursor
  // inside it to see it.
  blocksNamed(structure, "math").forEach((block, index) => {
    cards.push({
      key: `math-${index}`,
      kind: "math",
      index,
      at: block.from,
      from: block.from,
      to: block.to,
      label: `Equation ${index + 1}`,
    });
  });

  // A `<hick:table>` is a dataset that is also prose; the card is how its
  // grid is reached, and how the CSV underneath is got back to.
  [...blocksNamed(structure, "table"), ...markdownTableBlocks(structure, options.text)].sort((a, b) => a.from - b.from).forEach((block, index) => {
    const path = block.attrs.path;
    cards.push({
      key: `table-${index}`,
      kind: "table",
      index,
      at: block.from,
      from: block.from,
      to: block.to,
      label: path ? `Table ${index + 1} — ${path}` : `Table ${index + 1}`,
    });
  });

  // A `<hick:file>` that writes a chart is machinery wrapped around a
  // picture. Its card is the way to see which one it is without reading the
  // plotting code, and the way back to the plotting code.
  pictureBlocksOf(structure).forEach((block, index) => {
    cards.push({
      key: `picture-${index}`,
      kind: "picture",
      index,
      at: block.from,
      from: block.from,
      to: block.to,
      label: `Picture ${index + 1} — ${block.attrs.path}`,
    });
  });

  proseFences(structure, options.text).forEach((fence, index) => {
    cards.push({
      key: `fence-${index}`,
      kind: "fence",
      index,
      at: fence.from,
      from: fence.from,
      to: fence.to,
      label: fence.info
        ? `Code fence (${fence.info}) — make it a cell`
        : "Code fence — make it a cell",
    });
  });

  // Document order, and stable when two cards start at the same offset (a
  // fence cannot, but an empty exec and a diagram could after an edit).
  cards.sort((a, b) => a.at - b.at || a.kind.localeCompare(b.kind));
  return cards;
}
</storycode:copy>

### Assemble `apps/web/src/editor/cards.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/cards.ts"><storycode:paste select="#f-84385185ee44-CardKind"/><storycode:paste select="#f-84385185ee44-DocCard"/><storycode:paste select="#f-84385185ee44-CardsOptions"/><storycode:paste select="#f-84385185ee44-cardsOf"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-66013e5cdf76"></a>
## `apps/web/src/editor/cards.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Fence Becomes A Cell That Runs](../../../../../guarantees/authoring/a-fence-becomes-a-cell-that-runs/contract.md), [A Generated Picture Shows As A Picture](../../../../../guarantees/authoring/a-generated-picture-shows-as-a-picture/contract.md), [The Gutters Never Skip A Number](../../../../../guarantees/authoring/the-gutters-never-skip-a-number/contract.md).

<a id="f-66013e5cdf76-cards"></a>
### `cards`

<storycode:copy id="f-66013e5cdf76-cards">
import { describe, expect, it } from "vitest";

// Protects docs/guarantees/authoring/the-gutters-never-skip-a-number.md
// and docs/guarantees/authoring/a-fence-becomes-a-cell-that-runs.md
import { cardsOf } from "./cards";
import { containerNamesOf, parseHickDoc, proseCodeFences, proseFences } from "./hickDoc";

const cards = (text: string) => cardsOf(parseHickDoc(text), { text });

</storycode:copy>
<a id="f-66013e5cdf76-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-66013e5cdf76-expression_statement">
describe("what the rail lists", () => {
  it("gives every exec cell an icon, in document order", () => {
    const text =
      '<hick:exec container="a">\nls\n</hick:exec>\n\n<hick:exec container="b">\npwd\n</hick:exec>\n';
    const list = cards(text).filter((c) => c.kind === "exec");
    expect(list.map((c) => c.index)).toEqual([0, 1]);
    expect(list[0].label).toContain("a");
    expect(list[1].label).toContain("b");
    expect(list[0].at).toBeLessThan(list[1].at);
  });

  it("gives a cell an icon before the server has ever heard of it", () => {
    // A cell you just typed must be runnable without a round-trip; an icon
    // that appeared late would read as a bug.
    expect(cards("<hick:exec>\nls\n</hick:exec>\n")).toHaveLength(1);
  });

  it("lists diagrams, and can be told not to", () => {
    const text = '<hick:diagram renderer="mermaid">\ngraph TD\n</hick:diagram>\n';
    expect(cards(text).map((c) => c.kind)).toEqual(["diagram"]);
    expect(cardsOf(parseHickDoc(text), { text, diagrams: false })).toEqual([]);
  });

  it("gives a file block that writes a picture its own icon", () => {
    // The block's source is the plotting code; what it MEANS is a chart, so
    // the rail offers the way to see which chart it is.
    const text =
      '<hick:file path="chart.svg">\n<hick:exec container="r">\nplot\n</hick:exec>\n</hick:file>\n';
    const list = cards(text);
    expect(list.map((c) => c.kind).sort()).toEqual(["exec", "picture"]);
    expect(list.find((c) => c.kind === "picture")?.label).toContain("chart.svg");
    // And the cell inside it is marked, so the rail can leave its source
    // toggle off — the picture block owns that verb.
    expect(list.find((c) => c.kind === "exec")?.insidePicture).toBe(true);
  });

  it("leaves a cell outside a picture with its own source toggle", () => {
    const list = cards('<hick:exec container="a">\nls\n</hick:exec>\n');
    expect(list[0].insidePicture).toBeFalsy();
  });

  it("leaves a file block that writes text alone", () => {
    // A README is read as its source. Only a file a browser can DRAW has a
    // second thing to show.
    const text = '<hick:file path="README.md">\nhello\n</hick:file>\n';
    expect(cards(text).some((c) => c.kind === "picture")).toBe(false);
  });

  it("sorts every kind into one document-order rail", () => {
    const text =
      "# Doc\n\n```bash\nls\n```\n\n" +
      '<hick:exec container="a">\npwd\n</hick:exec>\n';
    expect(cards(text).map((c) => c.kind)).toEqual(["fence", "exec"]);
  });
});

</storycode:copy>
<a id="f-66013e5cdf76-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-66013e5cdf76-expression_statement-2">
describe("which fences are offered", () => {
  const fencesIn = (text: string) => proseFences(parseHickDoc(text), text);

  it("finds a closed fence in prose, with its language and body", () => {
    const text = "before\n\n```python\nprint(1)\n```\n\nafter\n";
    const [fence] = fencesIn(text);
    expect(fence.info).toBe("python");
    expect(fence.body).toBe("print(1)");
    expect(text.slice(fence.from, fence.to)).toBe("```python\nprint(1)\n```");
  });

  it("ignores a fence inside a generated file — that is the file's content", () => {
    const text = '<hick:file path="README.md">\n```bash\nls\n```\n</hick:file>\n';
    expect(fencesIn(text)).toEqual([]);
    expect(cards(text).some((c) => c.kind === "fence")).toBe(false);
  });

  it("ignores a fence inside an exec — that is part of a command", () => {
    const text = '<hick:exec container="a">\ncat <<EOF\n```\nEOF\n</hick:exec>\n';
    expect(fencesIn(text)).toEqual([]);
  });

  it("ignores an unterminated fence rather than guessing where it ends", () => {
    expect(fencesIn("text\n\n```python\nprint(1)\n")).toEqual([]);
  });

  it("still displays an unterminated prose fence as code while it is being typed", () => {
    const text = "text\n\n```python\nprint(1)\n";
    expect(proseCodeFences(parseHickDoc(text), text)).toEqual([
      {
        from: 6,
        to: text.length,
        closed: false,
        info: "python",
        body: "print(1)\n",
      },
    ]);
  });

  it("keeps tilde fences and backtick fences apart", () => {
    const text = "~~~js\nlet a\n~~~\n";
    const [fence] = fencesIn(text);
    expect(fence.info).toBe("js");
    expect(fence.body).toBe("let a");
  });

  it("handles an empty fence without producing a negative range", () => {
    const text = "```\n```\n";
    const [fence] = fencesIn(text);
    expect(fence.body).toBe("");
    expect(fence.to).toBeGreaterThan(fence.from);
  });
});

</storycode:copy>
<a id="f-66013e5cdf76-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-66013e5cdf76-expression_statement-3">
describe("the containers a conversion can choose from", () => {
  const names = (text: string) => containerNamesOf(parseHickDoc(text));

  it("includes self-closing declarations", () => {
    expect(names('<hick:container name="py" image="python:3.12" />\n')).toEqual(["py"]);
  });

  it("includes a container that only ever exists implicitly", () => {
    // `image=` on the first exec creates the container, so a document with no
    // <hick:container> at all still has one — and saying "this document
    // declares no container" when its name is right there is just wrong.
    expect(names('<hick:exec container="py" image="python:3.12">\nls\n</hick:exec>\n')).toEqual([
      "py",
    ]);
  });

  it("includes a fork's target", () => {
    const text =
      '<hick:container name="base" image="ubuntu" />\n<hick:fork from="base" to="analyzer" />\n';
    expect(names(text)).toEqual(["base", "analyzer"]);
  });

  it("lists each name once, in document order", () => {
    const text =
      '<hick:exec container="b">\nx\n</hick:exec>\n<hick:exec container="a">\ny\n</hick:exec>\n' +
      '<hick:exec container="b">\nz\n</hick:exec>\n';
    expect(names(text)).toEqual(["b", "a"]);
  });

  it("is empty for a document with no containers at all", () => {
    expect(names("# Just prose\n")).toEqual([]);
  });
});
</storycode:copy>

### Assemble `apps/web/src/editor/cards.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/cards.test.ts"><storycode:paste select="#f-66013e5cdf76-cards"/><storycode:paste select="#f-66013e5cdf76-expression_statement"/><storycode:paste select="#f-66013e5cdf76-expression_statement-2"/><storycode:paste select="#f-66013e5cdf76-expression_statement-3"/></storycode:file>


</storycode:doc>
