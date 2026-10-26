<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/openRepresentation.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-429198bb2941"></a>
## `apps/web/src/lib/openRepresentation.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-429198bb2941-OPEN_REPRESENTATION"></a>
### `OPEN_REPRESENTATION`

<storycode:copy id="f-429198bb2941-OPEN_REPRESENTATION">
import { representations, type Backing } from "../api/representations";
export const OPEN_REPRESENTATION = "hickory.open-representation";
</storycode:copy>
<a id="f-429198bb2941-showRepresentation"></a>
### `showRepresentation`

<storycode:copy id="f-429198bb2941-showRepresentation">
export function showRepresentation(id: string, base?: string, target?: string) {
  window.dispatchEvent(new CustomEvent(OPEN_REPRESENTATION, { detail: { id, base, target } }));
}
</storycode:copy>
<a id="f-429198bb2941-openLiterate"></a>
### `openLiterate`

<storycode:copy id="f-429198bb2941-openLiterate">
export async function openLiterate(backing: Backing, base?: string, target?: string) {
  const view = await representations.create(backing);
  showRepresentation(view.id, base, target);
}
</storycode:copy>

### Assemble `apps/web/src/lib/openRepresentation.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/openRepresentation.ts"><storycode:paste select="#f-429198bb2941-OPEN_REPRESENTATION"/><storycode:paste select="#f-429198bb2941-showRepresentation"/><storycode:paste select="#f-429198bb2941-openLiterate"/></storycode:file>

</storycode:doc>
