import { useEffect, useState } from "react";

export type FilesParallax = boolean;
export const FILES_PARALLAX_KEY = "hickory.filesParallax";
const event = "hickory-files-parallax";
const defaults = (): FilesParallax => false;

export function loadFilesParallax(storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage): FilesParallax {
  try {
    const stored = JSON.parse(storage?.getItem(FILES_PARALLAX_KEY) ?? "null");
    return stored === true;
  } catch { return defaults(); }
}
export function saveFilesParallax(enabled: FilesParallax) {
  try { localStorage.setItem(FILES_PARALLAX_KEY, JSON.stringify(enabled)); } catch { /* Presentation storage is optional. */ }
  window.dispatchEvent(new CustomEvent(event, { detail: enabled }));
}
export function useFilesParallax() {
  const [enabled, setEnabled] = useState(loadFilesParallax);
  useEffect(() => {
    const local = (change: Event) => setEnabled((change as CustomEvent<FilesParallax>).detail);
    const otherWindow = (change: StorageEvent) => { if (!change.key || change.key === FILES_PARALLAX_KEY) setEnabled(loadFilesParallax()); };
    window.addEventListener(event, local); window.addEventListener("storage", otherWindow);
    return () => { window.removeEventListener(event, local); window.removeEventListener("storage", otherWindow); };
  }, []);
  return enabled;
}
