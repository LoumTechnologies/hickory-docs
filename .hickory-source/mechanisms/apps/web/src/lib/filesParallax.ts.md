<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/filesParallax.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c489754c4cb0"></a>
## `apps/web/src/lib/filesParallax.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-c489754c4cb0-FilesParallax"></a>
### `FilesParallax`

<storycode:copy id="f-c489754c4cb0-FilesParallax">
import { useEffect, useState } from "react";

export type FilesParallax = boolean;
</storycode:copy>
<a id="f-c489754c4cb0-FILES_PARALLAX_KEY"></a>
### `FILES_PARALLAX_KEY`

<storycode:copy id="f-c489754c4cb0-FILES_PARALLAX_KEY">
export const FILES_PARALLAX_KEY = "hickory.filesParallax";
</storycode:copy>
<a id="f-c489754c4cb0-event"></a>
### `event`

<storycode:copy id="f-c489754c4cb0-event">
const event = "hickory-files-parallax";
</storycode:copy>
<a id="f-c489754c4cb0-defaults"></a>
### `defaults`

<storycode:copy id="f-c489754c4cb0-defaults">
const defaults = (): FilesParallax => false;

</storycode:copy>
<a id="f-c489754c4cb0-loadFilesParallax"></a>
### `loadFilesParallax`

<storycode:copy id="f-c489754c4cb0-loadFilesParallax">
export function loadFilesParallax(storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage): FilesParallax {
  try {
    const stored = JSON.parse(storage?.getItem(FILES_PARALLAX_KEY) ?? "null");
    return stored === true;
  } catch { return defaults(); }
}
</storycode:copy>
<a id="f-c489754c4cb0-saveFilesParallax"></a>
### `saveFilesParallax`

<storycode:copy id="f-c489754c4cb0-saveFilesParallax">
export function saveFilesParallax(enabled: FilesParallax) {
  try { localStorage.setItem(FILES_PARALLAX_KEY, JSON.stringify(enabled)); } catch { /* Presentation storage is optional. */ }
  window.dispatchEvent(new CustomEvent(event, { detail: enabled }));
}
</storycode:copy>
<a id="f-c489754c4cb0-useFilesParallax"></a>
### `useFilesParallax`

<storycode:copy id="f-c489754c4cb0-useFilesParallax">
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
</storycode:copy>

### Assemble `apps/web/src/lib/filesParallax.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/filesParallax.ts"><storycode:paste select="#f-c489754c4cb0-FilesParallax"/><storycode:paste select="#f-c489754c4cb0-FILES_PARALLAX_KEY"/><storycode:paste select="#f-c489754c4cb0-event"/><storycode:paste select="#f-c489754c4cb0-defaults"/><storycode:paste select="#f-c489754c4cb0-loadFilesParallax"/><storycode:paste select="#f-c489754c4cb0-saveFilesParallax"/><storycode:paste select="#f-c489754c4cb0-useFilesParallax"/></storycode:file>

</storycode:doc>
