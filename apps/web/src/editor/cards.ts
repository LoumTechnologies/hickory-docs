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

import { execBlocksOf, blocksNamed, proseFences } from "./hickDoc";
import type { HickDocStructure } from "./hickDoc";

export type CardKind = "exec" | "diagram" | "fence";

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
}

export interface CardsOptions {
  /** Diagrams need a rendering engine; the marketing demo ships without one
   * and must not offer a card it cannot draw. */
  diagrams?: boolean;
  /** The whole document text — fences are found by scanning it. */
  text: string;
}

/**
 * Every card in the document, in the order their icons stack down the rail.
 *
 * Exec cells come from the parsed structure so a cell the server has not
 * rendered yet still gets an icon: an exec you just typed must be runnable
 * before a round-trip, and an icon that appears late reads as a bug.
 */
export function cardsOf(structure: HickDocStructure, options: CardsOptions): DocCard[] {
  const cards: DocCard[] = [];

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
    });
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
