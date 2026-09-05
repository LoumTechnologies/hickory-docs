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
