<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/filesWindows.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `4228f9518ac6a42b4bcf75d48357d2ea2a1edeb70eff9da297e5447485054206`.

<book:entry path="apps/web/src/lib/filesWindows.ts">
import { useEffect, useState } from "react";

export type FilesWindows = boolean;
export const FILES_WINDOWS_KEY = "hickory.filesWindows";
const event = "hickory-files-windows";
const defaults = (): FilesWindows => false;

export function loadFilesWindows(storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage): FilesWindows {
  try {
    const stored = JSON.parse(storage?.getItem(FILES_WINDOWS_KEY) ?? "null");
    return stored === true;
  } catch { return defaults(); }
}
export function saveFilesWindows(enabled: FilesWindows) {
  try { localStorage.setItem(FILES_WINDOWS_KEY, JSON.stringify(enabled)); } catch { /* Presentation storage is optional. */ }
  window.dispatchEvent(new CustomEvent(event, { detail: enabled }));
}
export function useFilesWindows() {
  const [enabled, setEnabled] = useState(loadFilesWindows);
  useEffect(() => {
    const local = (change: Event) => setEnabled((change as CustomEvent<FilesWindows>).detail);
    const otherWindow = (change: StorageEvent) => { if (!change.key || change.key === FILES_WINDOWS_KEY) setEnabled(loadFilesWindows()); };
    window.addEventListener(event, local); window.addEventListener("storage", otherWindow);
    return () => { window.removeEventListener(event, local); window.removeEventListener("storage", otherWindow); };
  }, []);
  return enabled;
}
</book:entry>

</book:doc>
