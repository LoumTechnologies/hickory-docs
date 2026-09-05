// The element registry as the server publishes it (`GET /api/elements`) and
// what an element's action came to (`POST /api/docs/:id/blocks/:at/:action`).
// See docs/specs/freeform/the-minimal-core.md.

import type { Doc } from "./types";

/** One element as the server declares it (`GET /api/elements`). */
export interface ElementDescription {
  name: string;
  /** The block kind its view draws. */
  kind: string;
  attributes: { name: string; required: boolean; doc: string }[];
  actions: string[];
}

/** What an element's action came to (`POST /api/docs/:id/blocks/:at/:action`). */
export type BlockActionOutcome =
  | { outcome: "answer"; value: unknown }
  | { outcome: "run"; run_id: string; cells: string[] }
  | { outcome: "edit"; span: [number, number]; doc: Doc };
