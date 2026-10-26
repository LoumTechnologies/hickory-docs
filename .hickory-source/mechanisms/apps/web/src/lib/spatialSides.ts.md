<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/spatialSides.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-562ade054815"></a>
## `apps/web/src/lib/spatialSides.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Spatial Views Preserve The Editor](../../../../../guarantees/shell/spatial-views-preserve-the-editor/contract.md).

<a id="f-562ade054815-SpatialSides"></a>
### `SpatialSides`

<storycode:copy id="f-562ade054815-SpatialSides">
import { useEffect, useState } from "react";

export interface SpatialSides { left: boolean; right: boolean }
</storycode:copy>
<a id="f-562ade054815-SPATIAL_SIDES_KEY"></a>
### `SPATIAL_SIDES_KEY`

<storycode:copy id="f-562ade054815-SPATIAL_SIDES_KEY">
export const SPATIAL_SIDES_KEY = "hickory.spatialSides";
</storycode:copy>
<a id="f-562ade054815-event"></a>
### `event`

<storycode:copy id="f-562ade054815-event">
const event = "hickory-spatial-sides";
</storycode:copy>
<a id="f-562ade054815-defaults"></a>
### `defaults`

<storycode:copy id="f-562ade054815-defaults">
const defaults = (): SpatialSides => ({ left: true, right: false });

</storycode:copy>
<a id="f-562ade054815-loadSpatialSides"></a>
### `loadSpatialSides`

<storycode:copy id="f-562ade054815-loadSpatialSides">
export function loadSpatialSides(storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage): SpatialSides {
  try {
    const stored = JSON.parse(storage?.getItem(SPATIAL_SIDES_KEY) ?? "null");
    return { left: typeof stored?.left === "boolean" ? stored.left : true, right: typeof stored?.right === "boolean" ? stored.right : false };
  } catch { return defaults(); }
}
</storycode:copy>
<a id="f-562ade054815-saveSpatialSides"></a>
### `saveSpatialSides`

<storycode:copy id="f-562ade054815-saveSpatialSides">
export function saveSpatialSides(sides: SpatialSides) {
  try { localStorage.setItem(SPATIAL_SIDES_KEY, JSON.stringify(sides)); } catch { /* Presentation storage is optional. */ }
  window.dispatchEvent(new CustomEvent(event, { detail: sides }));
}
</storycode:copy>
<a id="f-562ade054815-useSpatialSides"></a>
### `useSpatialSides`

<storycode:copy id="f-562ade054815-useSpatialSides">
export function useSpatialSides() {
  const [sides, setSides] = useState(loadSpatialSides);
  useEffect(() => {
    const local = (change: Event) => setSides((change as CustomEvent<SpatialSides>).detail);
    const otherWindow = (change: StorageEvent) => { if (!change.key || change.key === SPATIAL_SIDES_KEY) setSides(loadSpatialSides()); };
    window.addEventListener(event, local); window.addEventListener("storage", otherWindow);
    return () => { window.removeEventListener(event, local); window.removeEventListener("storage", otherWindow); };
  }, []);
  return sides;
}
</storycode:copy>

### Assemble `apps/web/src/lib/spatialSides.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/spatialSides.ts"><storycode:paste select="#f-562ade054815-SpatialSides"/><storycode:paste select="#f-562ade054815-SPATIAL_SIDES_KEY"/><storycode:paste select="#f-562ade054815-event"/><storycode:paste select="#f-562ade054815-defaults"/><storycode:paste select="#f-562ade054815-loadSpatialSides"/><storycode:paste select="#f-562ade054815-saveSpatialSides"/><storycode:paste select="#f-562ade054815-useSpatialSides"/></storycode:file>

</storycode:doc>
