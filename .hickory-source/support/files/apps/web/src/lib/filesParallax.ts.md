<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/filesParallax.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `0193be0112f00602a7043c5699f57f7f86fb3fa37cc4bffaed168fd4c5942545`.

<book:entry path="apps/web/src/lib/filesParallax.ts">
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
</book:entry>

</book:doc>
