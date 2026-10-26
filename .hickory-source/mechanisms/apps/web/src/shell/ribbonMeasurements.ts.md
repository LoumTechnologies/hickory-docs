<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/ribbonMeasurements.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-fa4e18871c2b"></a>
## `apps/web/src/shell/ribbonMeasurements.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-fa4e18871c2b-sourceRange"></a>
### `sourceRange`

<storycode:copy id="f-fa4e18871c2b-sourceRange">
import type { Ribbon } from "../lib/ribbons";
import { BlockType, type EditorView } from "@codemirror/view";
import type { RibbonTarget, RibbonLink } from "./Ribbons";
import type { structureOf } from "../editor/wysiwyg";
import { samePath } from "../lib/paths";

/**
 * The char range a ribbon's source claims on screen: the whole enclosing
 * block, not the fragment. A band that covers the copy block says "this
 * block became that text", which is the claim; a band over three characters
 * of it says nothing anyone can read at a glance.
 */
export function sourceRange(
  structure: ReturnType<typeof structureOf>,
  ribbon: Ribbon,
  length: number,
): [number, number] {
  let from = Math.min(ribbon.sourceSpan[0], length);
  let to = Math.min(ribbon.sourceSpan[1], length);
  for (const block of structure.blocks) {
    if (
      (block.name === "copy" ||
        block.name === "cut" ||
        block.name === "file") &&
      ribbon.sourceSpan[0] >= block.from &&
      ribbon.sourceSpan[1] <= block.to
    ) {
      from = Math.min(block.from, length);
      to = Math.min(block.to, length);
      break;
    }
  }
  return [from, to];
}

</storycode:copy>
<a id="f-fa4e18871c2b-bandBetween"></a>
### `bandBetween`

Screen extent of a range, including one scrolled out of sight.
`coordsAtPos` answers for rendered text only — everything outside the
viewport returns null, and a ribbon that cannot be measured is a ribbon
that is not drawn: six relationships showed one. The height map knows where
every line is whether or not it is on screen, and the content box moves
with the scroll, so the two together answer for any position; the caller
clamps the result to the visible pane. The answer is EXACT: the top of the
first line holding `from` to the bottom of the last line holding `to`.

<storycode:copy id="f-fa4e18871c2b-bandBetween">
/**
 * Screen extent of a range, including one scrolled out of sight.
 *
 * `coordsAtPos` answers for rendered text only — everything outside the
 * viewport returns null, and a ribbon that cannot be measured is a ribbon
 * that is not drawn: six relationships showed one. The height map knows where
 * every line is whether or not it is on screen, and the content box moves
 * with the scroll, so the two together answer for any position; the caller
 * clamps the result to the visible pane. The answer is EXACT: the top of the
 * first line holding `from` to the bottom of the last line holding `to`.
 */
export function bandBetween(
  view: EditorView,
  from: number,
  to: number,
): [number, number] | null {
  // Blocks are document-relative; the screen conversion is documentTop —
  // NOT contentDOM's rect, which sits a content-padding away, and by a
  // different amount per editor. The rails convert through documentTop
  // (editor/RightRail.tsx), and a band converted through anything else
  // lands its horns a pane-specific offset away from the numbers.
  const base = view.documentTop;
  const first = textOnly(view.lineBlockAt(Math.max(0, from)));
  const last = textOnly(view.lineBlockAt(Math.max(0, to)));
  const start = { top: base + first.top, bottom: base + first.bottom };
  const end = { top: base + last.top, bottom: base + last.bottom };
  return [Math.min(start.top, end.top), Math.max(start.bottom, end.bottom)];
}

</storycode:copy>
<a id="f-fa4e18871c2b-textOnly"></a>
### `textOnly`

The TEXT extent of a line block. A line carrying block widgets (a copy
chip, a cell panel) reports a composite block spanning the widgets too;
a band measured against that lands its horns offset from the number rows
— braces visibly crossing OUT the very numbers they mean to include. The
rails already align numbers to the text portion (lib/rightRail.ts
textExtent); the bands must measure the same thing, so a brace's edges
sit exactly on line boundaries, between numbers.

<storycode:copy id="f-fa4e18871c2b-textOnly">
/**
 * The TEXT extent of a line block. A line carrying block widgets (a copy
 * chip, a cell panel) reports a composite block spanning the widgets too;
 * a band measured against that lands its horns offset from the number rows
 * — braces visibly crossing OUT the very numbers they mean to include. The
 * rails already align numbers to the text portion (lib/rightRail.ts
 * textExtent); the bands must measure the same thing, so a brace's edges
 * sit exactly on line boundaries, between numbers.
 */
function textOnly(block: { top: number; bottom: number; type: unknown }): {
  top: number;
  bottom: number;
} {
  if (Array.isArray(block.type)) {
    const text = (
      block.type as { type: BlockType; top: number; bottom: number }[]
    ).find((c) => c.type === BlockType.Text);
    if (text) return { top: text.top, bottom: text.bottom };
  }
  return { top: block.top, bottom: block.bottom };
}

</storycode:copy>
<a id="f-fa4e18871c2b-linkTitle"></a>
### `linkTitle`

The tooltip of a context/declared connection: the link's own words.

<storycode:copy id="f-fa4e18871c2b-linkTitle">
/** The tooltip of a context/declared connection: the link's own words. */
export function linkTitle(
  target: Extract<RibbonTarget, { kind: "path" }>,
  links: readonly RibbonLink[],
): string {
  const link = links.find(
    (l) =>
      l.family === target.family &&
      samePath(l.to.path, target.path) &&
      (l.to.lines?.[0] ?? 0) === (target.lines?.[0] ?? 0),
  );
  return link?.title ?? `${target.family}: ${target.path}`;
}
</storycode:copy>

### Assemble `apps/web/src/shell/ribbonMeasurements.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/ribbonMeasurements.ts"><storycode:paste select="#f-fa4e18871c2b-sourceRange"/><storycode:paste select="#f-fa4e18871c2b-bandBetween"/><storycode:paste select="#f-fa4e18871c2b-textOnly"/><storycode:paste select="#f-fa4e18871c2b-linkTitle"/></storycode:file>

</storycode:doc>
