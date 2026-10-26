<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/filesWindows.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4aa98f1666f4"></a>
## `apps/web/src/lib/filesWindows.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-4aa98f1666f4-FilesWindows"></a>
### `FilesWindows`

<storycode:copy id="f-4aa98f1666f4-FilesWindows">
import { useEffect, useState } from "react";

export type FilesWindows = boolean;
</storycode:copy>
<a id="f-4aa98f1666f4-FILES_WINDOWS_KEY"></a>
### `FILES_WINDOWS_KEY`

<storycode:copy id="f-4aa98f1666f4-FILES_WINDOWS_KEY">
export const FILES_WINDOWS_KEY = "hickory.filesWindows";
</storycode:copy>
<a id="f-4aa98f1666f4-event"></a>
### `event`

<storycode:copy id="f-4aa98f1666f4-event">
const event = "hickory-files-windows";
</storycode:copy>
<a id="f-4aa98f1666f4-defaults"></a>
### `defaults`

<storycode:copy id="f-4aa98f1666f4-defaults">
const defaults = (): FilesWindows => false;

</storycode:copy>
<a id="f-4aa98f1666f4-loadFilesWindows"></a>
### `loadFilesWindows`

<storycode:copy id="f-4aa98f1666f4-loadFilesWindows">
export function loadFilesWindows(storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage): FilesWindows {
  try {
    const stored = JSON.parse(storage?.getItem(FILES_WINDOWS_KEY) ?? "null");
    return stored === true;
  } catch { return defaults(); }
}
</storycode:copy>
<a id="f-4aa98f1666f4-saveFilesWindows"></a>
### `saveFilesWindows`

<storycode:copy id="f-4aa98f1666f4-saveFilesWindows">
export function saveFilesWindows(enabled: FilesWindows) {
  try { localStorage.setItem(FILES_WINDOWS_KEY, JSON.stringify(enabled)); } catch { /* Presentation storage is optional. */ }
  window.dispatchEvent(new CustomEvent(event, { detail: enabled }));
}
</storycode:copy>
<a id="f-4aa98f1666f4-useFilesWindows"></a>
### `useFilesWindows`

<storycode:copy id="f-4aa98f1666f4-useFilesWindows">
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
</storycode:copy>

### Assemble `apps/web/src/lib/filesWindows.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/filesWindows.ts"><storycode:paste select="#f-4aa98f1666f4-FilesWindows"/><storycode:paste select="#f-4aa98f1666f4-FILES_WINDOWS_KEY"/><storycode:paste select="#f-4aa98f1666f4-event"/><storycode:paste select="#f-4aa98f1666f4-defaults"/><storycode:paste select="#f-4aa98f1666f4-loadFilesWindows"/><storycode:paste select="#f-4aa98f1666f4-saveFilesWindows"/><storycode:paste select="#f-4aa98f1666f4-useFilesWindows"/></storycode:file>

</storycode:doc>
