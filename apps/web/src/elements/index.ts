// Every element the editor draws in place of its source, keyed by the
// block `kind` the server uses. See ./types.ts.

import { markdownTableBlocks } from "../editor/markdownTables";
import type { HickBlock, HickDocStructure } from "../editor/hickDoc";
import { diagramView } from "./diagram/view";
import { execView } from "./exec/view";
import { mathView } from "./math/view";
import { outputView } from "./output/view";
import { pictureView } from "./picture/view";
import { sessionViews } from "./session/view";
import { tableView } from "./table/view";
import type { ElementView, SlotKind } from "./types";

export type { ElementView, SlotContext, SlotKind } from "./types";

/** The registry. Order is the order `draws` is asked, so a `file` that is
 * a picture is claimed by the picture view and no other. */
export const elementViews: Record<SlotKind, ElementView> = {
  exec: execView,
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

const views = Object.values(elementViews);

/** Which view draws this structure block, or null when none does. */
export function slotKindOf(block: HickBlock): SlotKind | null {
  return views.find((view) => view.draws(block))?.kind ?? null;
}

/** The blocks some element renders, in document order. */
export function renderableBlocks(structure: HickDocStructure, text?: string): HickBlock[] {
  return [...structure.blocks, ...(text === undefined ? [] : markdownTableBlocks(structure, text))]
    .filter((block) => slotKindOf(block) !== null)
    .sort((a, b) => a.from - b.from || b.to - a.to);
}
