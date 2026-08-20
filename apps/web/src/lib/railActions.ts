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
  options: { replay?: boolean } = {},
): RailAction[] {
  switch (kind) {
    case "exec":
      return options.replay ? ["run", "source", "replay"] : ["run", "source"];
    case "diagram":
    // An equation is a picture too: the only verb it has is "show me what I
    // actually typed".
    case "math":
      return ["source"];
    case "fence":
      return ["convert"];
  }
}
