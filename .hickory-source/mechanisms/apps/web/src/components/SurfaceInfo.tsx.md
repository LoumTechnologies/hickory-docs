<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/SurfaceInfo.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6c34dd749b33"></a>
## `apps/web/src/components/SurfaceInfo.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](../../../../../guarantees/authoring/the-editor-keeps-its-space/contract.md), [A view declares where edits land](../../../../../guarantees/lenses/a-view-declares-where-edits-land/contract.md).

<a id="f-6c34dd749b33-SurfaceInfo"></a>
### `SurfaceInfo`

Retain identity and write-destination metadata for agent inspection without rendering an information button explaining ordinary editor conventions. Actual permission and revision checks remain with save operations.

<storycode:copy id="f-6c34dd749b33-SurfaceInfo">
import { useUiReading } from "../lib/uiReading";

export function SurfaceInfo({ name, editable, destination }: {
  name: string; editable: boolean; destination: string;
}) {
  useUiReading("surface identity", { name, editable, destination });
  return null;
}
</storycode:copy>

### Assemble `apps/web/src/components/SurfaceInfo.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/SurfaceInfo.tsx"><storycode:paste select="#f-6c34dd749b33-SurfaceInfo"/></storycode:file>

</storycode:doc>
