<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/elements/index.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9b7b2fa9b3c3"></a>
## `apps/web/src/elements/index.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Markdown tables use the table editor](../../../../../guarantees/authoring/markdown-tables-use-the-table-editor/contract.md), [An Element Is Drawn By Its View](../../../../../guarantees/language/an-element-is-drawn-by-its-view/contract.md), [Patch elements use the shared diff renderer](../../../../../guarantees/literate-projects/patch-view-uses-the-shared-diff-renderer/contract.md).

<a id="f-9b7b2fa9b3c3-export_statement"></a>
### `export_statement`

Every element the editor draws in place of its source, keyed by the
block `kind` the server uses. See ./types.ts.

<storycode:copy id="f-9b7b2fa9b3c3-export_statement">
// Every element the editor draws in place of its source, keyed by the
// block `kind` the server uses. See ./types.ts.

import { markdownTableBlocks } from "../editor/markdownTables";
import type { HickBlock, HickDocStructure } from "../editor/hickDoc";
import { datasetView } from "./dataset/view";
import { diagramView } from "./diagram/view";
import { execView } from "./exec/view";
import { mathView } from "./math/view";
import { outputView } from "./output/view";
import { patchView } from "./patch/view";
import { pictureView } from "./picture/view";
import { sessionViews } from "./session/view";
import { tableView } from "./table/view";
import type { ElementView, SlotKind } from "./types";

export type { ElementView, SlotContext, SlotKind } from "./types";

</storycode:copy>
<a id="f-9b7b2fa9b3c3-elementViews"></a>
### `elementViews`

The registry. Order is the order `draws` is asked, so a `file` that is
a picture is claimed by the picture view and no other.

<storycode:copy id="f-9b7b2fa9b3c3-elementViews">
/** The registry. Order is the order `draws` is asked, so a `file` that is
 * a picture is claimed by the picture view and no other. */
export const elementViews: Record<SlotKind, ElementView> = {
  exec: execView,
  dataset: datasetView,
  patch: patchView,
  output: outputView,
  diagram: diagramView,
  math: mathView,
  table: tableView,
  picture: pictureView,
  ...(Object.fromEntries(sessionViews.map((v) => [v.kind, v])) as Record<
    Extract<SlotKind, `session-${string}`>,
    ElementView
  >),
};

</storycode:copy>
<a id="f-9b7b2fa9b3c3-views"></a>
### `views`

<storycode:copy id="f-9b7b2fa9b3c3-views">
const views = Object.values(elementViews);

</storycode:copy>
<a id="f-9b7b2fa9b3c3-slotKindOf"></a>
### `slotKindOf`

Which view draws this structure block, or null when none does.

<storycode:copy id="f-9b7b2fa9b3c3-slotKindOf">
/** Which view draws this structure block, or null when none does. */
export function slotKindOf(block: HickBlock): SlotKind | null {
  return views.find((view) => view.draws(block))?.kind ?? null;
}

</storycode:copy>
<a id="f-9b7b2fa9b3c3-renderableBlocks"></a>
### `renderableBlocks`

The blocks some element renders, in document order.

<storycode:copy id="f-9b7b2fa9b3c3-renderableBlocks">
/** The blocks some element renders, in document order. */
export function renderableBlocks(structure: HickDocStructure, text?: string): HickBlock[] {
  return [...structure.blocks, ...(text === undefined ? [] : markdownTableBlocks(structure, text))]
    .filter((block) => slotKindOf(block) !== null)
    .sort((a, b) => a.from - b.from || b.to - a.to);
}
</storycode:copy>

### Assemble `apps/web/src/elements/index.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/elements/index.ts"><storycode:paste select="#f-9b7b2fa9b3c3-export_statement"/><storycode:paste select="#f-9b7b2fa9b3c3-elementViews"/><storycode:paste select="#f-9b7b2fa9b3c3-views"/><storycode:paste select="#f-9b7b2fa9b3c3-slotKindOf"/><storycode:paste select="#f-9b7b2fa9b3c3-renderableBlocks"/></storycode:file>

</storycode:doc>
