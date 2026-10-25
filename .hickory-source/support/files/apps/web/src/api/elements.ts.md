<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/api/elements.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `b84eba37d32aedcf9219466f9ad0dc4f741c0299c9d514662df37eac8b1c2916`.

<book:entry path="apps/web/src/api/elements.ts">
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
</book:entry>

</book:doc>
