<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/elements.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-944afcfc60ab"></a>
## `apps/web/src/api/elements.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-944afcfc60ab-ElementDescription"></a>
### `ElementDescription`

The element registry as the server publishes it (`GET /api/elements`) and
what an element's action came to (`POST /api/docs/:id/blocks/:at/:action`).
See docs/specs/freeform/the-minimal-core.md.

<storycode:copy id="f-944afcfc60ab-ElementDescription">
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

</storycode:copy>
<a id="f-944afcfc60ab-BlockActionOutcome"></a>
### `BlockActionOutcome`

What an element's action came to (`POST /api/docs/:id/blocks/:at/:action`).

<storycode:copy id="f-944afcfc60ab-BlockActionOutcome">
/** What an element's action came to (`POST /api/docs/:id/blocks/:at/:action`). */
export type BlockActionOutcome =
  | { outcome: "answer"; value: unknown }
  | { outcome: "run"; run_id: string; cells: string[] }
  | { outcome: "edit"; span: [number, number]; doc: Doc };
</storycode:copy>

### Assemble `apps/web/src/api/elements.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/elements.ts"><storycode:paste select="#f-944afcfc60ab-ElementDescription"/><storycode:paste select="#f-944afcfc60ab-BlockActionOutcome"/></storycode:file>

</storycode:doc>
