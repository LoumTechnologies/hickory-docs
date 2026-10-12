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
