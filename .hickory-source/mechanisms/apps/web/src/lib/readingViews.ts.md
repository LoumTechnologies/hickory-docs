<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/readingViews.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-89af48b2dad1"></a>
## `apps/web/src/lib/readingViews.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-89af48b2dad1-ReadingView"></a>
### `ReadingView`

<storycode:copy id="f-89af48b2dad1-ReadingView">
import { useSyncExternalStore } from "react";

export interface ReadingView {
  title: string;
  path: string;
  before: string;
  after: string;
  status: string;
  editable?: boolean;
  codeUri?: string;
  debugOwner?: string;
  location?: {line:number;character:number};
  dispose?: () => void;
  project?: import("../editor/hickLang").ProjectEntry[];
  decide?: (accepted: boolean, newText?: string) => Promise<void | import("./uiActions").UiOutcome>;
}
</storycode:copy>
<a id="f-89af48b2dad1-OPEN_READING"></a>
### `OPEN_READING`

<storycode:copy id="f-89af48b2dad1-OPEN_READING">
export const OPEN_READING = "hickory.open-reading";
</storycode:copy>
<a id="f-89af48b2dad1-readings"></a>
### `readings`

<storycode:copy id="f-89af48b2dad1-readings">
const readings = new Map<string, ReadingView>();
</storycode:copy>
<a id="f-89af48b2dad1-listeners"></a>
### `listeners`

<storycode:copy id="f-89af48b2dad1-listeners">
const listeners = new Set<() => void>();
</storycode:copy>
<a id="f-89af48b2dad1-version"></a>
### `version`

<storycode:copy id="f-89af48b2dad1-version">
let version = 0;
</storycode:copy>
<a id="f-89af48b2dad1-changed"></a>
### `changed`

<storycode:copy id="f-89af48b2dad1-changed">
function changed() { version++; listeners.forEach(listener => listener()); }
</storycode:copy>
<a id="f-89af48b2dad1-readingStatus"></a>
### `readingStatus`

<storycode:copy id="f-89af48b2dad1-readingStatus">
export const readingStatus = (id: string) => readings.get(id)?.status;
</storycode:copy>
<a id="f-89af48b2dad1-useReadingsVersion"></a>
### `useReadingsVersion`

<storycode:copy id="f-89af48b2dad1-useReadingsVersion">
export function useReadingsVersion() {
  return useSyncExternalStore(listener => { listeners.add(listener); return () => listeners.delete(listener); }, () => version);
}
</storycode:copy>
<a id="f-89af48b2dad1-publishReading"></a>
### `publishReading`

<storycode:copy id="f-89af48b2dad1-publishReading">
export function publishReading(id: string, view: ReadingView) { const previous = readings.get(id); readings.set(id, view); previous?.dispose?.(); changed(); }
</storycode:copy>
<a id="f-89af48b2dad1-forgetReading"></a>
### `forgetReading`

<storycode:copy id="f-89af48b2dad1-forgetReading">
export function forgetReading(id: string) { readings.get(id)?.dispose?.(); readings.delete(id); changed(); }
</storycode:copy>
<a id="f-89af48b2dad1-openReading"></a>
### `openReading`

<storycode:copy id="f-89af48b2dad1-openReading">
export function openReading(id: string, title: string, background = false) {
  window.dispatchEvent(new CustomEvent(OPEN_READING, { detail: { id, title, ...(background ? { background: true } : {}) } }));
}
</storycode:copy>
<a id="f-89af48b2dad1-openCommit"></a>
### `openCommit`

<storycode:copy id="f-89af48b2dad1-openCommit">
export function openCommit(sha: string) { openReading(`commit:${sha}`, "Commit"); }
</storycode:copy>
<a id="f-89af48b2dad1-useReading"></a>
### `useReading`

<storycode:copy id="f-89af48b2dad1-useReading">
export function useReading(id: string) {
  return useSyncExternalStore(listener => { listeners.add(listener); return () => listeners.delete(listener); }, () => readings.get(id));
}
</storycode:copy>

### Assemble `apps/web/src/lib/readingViews.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/readingViews.ts"><storycode:paste select="#f-89af48b2dad1-ReadingView"/><storycode:paste select="#f-89af48b2dad1-OPEN_READING"/><storycode:paste select="#f-89af48b2dad1-readings"/><storycode:paste select="#f-89af48b2dad1-listeners"/><storycode:paste select="#f-89af48b2dad1-version"/><storycode:paste select="#f-89af48b2dad1-changed"/><storycode:paste select="#f-89af48b2dad1-readingStatus"/><storycode:paste select="#f-89af48b2dad1-useReadingsVersion"/><storycode:paste select="#f-89af48b2dad1-publishReading"/><storycode:paste select="#f-89af48b2dad1-forgetReading"/><storycode:paste select="#f-89af48b2dad1-openReading"/><storycode:paste select="#f-89af48b2dad1-openCommit"/><storycode:paste select="#f-89af48b2dad1-useReading"/></storycode:file>

</storycode:doc>
