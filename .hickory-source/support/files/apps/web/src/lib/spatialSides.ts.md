<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/spatialSides.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Spatial Views Preserve The Editor](../../../../../../guarantees/shell/spatial-views-preserve-the-editor/contract.md)

Captured SHA-256: `08570a2de5038bafebda34808ab8539d8b6a6e051ed3d97560140ff1c5e8330b`.

<book:entry path="apps/web/src/lib/spatialSides.ts">
import { useEffect, useState } from "react";

export interface SpatialSides { left: boolean; right: boolean }
export const SPATIAL_SIDES_KEY = "hickory.spatialSides";
const event = "hickory-spatial-sides";
const defaults = (): SpatialSides => ({ left: true, right: false });

export function loadSpatialSides(storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage): SpatialSides {
  try {
    const stored = JSON.parse(storage?.getItem(SPATIAL_SIDES_KEY) ?? "null");
    return { left: typeof stored?.left === "boolean" ? stored.left : true, right: typeof stored?.right === "boolean" ? stored.right : false };
  } catch { return defaults(); }
}
export function saveSpatialSides(sides: SpatialSides) {
  try { localStorage.setItem(SPATIAL_SIDES_KEY, JSON.stringify(sides)); } catch { /* Presentation storage is optional. */ }
  window.dispatchEvent(new CustomEvent(event, { detail: sides }));
}
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
</book:entry>

</book:doc>
