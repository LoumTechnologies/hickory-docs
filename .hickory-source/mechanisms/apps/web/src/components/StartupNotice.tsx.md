<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/StartupNotice.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9a5486901220"></a>
## `apps/web/src/components/StartupNotice.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Editor Does Not Wait For Workspace Preparation](../../../../../guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md).

<a id="f-9a5486901220-StartupNotice"></a>
### `StartupNotice`

<storycode:copy id="f-9a5486901220-StartupNotice">
import { useSyncExternalStore } from "react";
import { startupNotice, subscribeStartup } from "../api/workspaceStartup";

export function StartupNotice() {
  const notice = useSyncExternalStore(subscribeStartup, startupNotice);
  return notice ? <div className="menu-notice" role="status">{notice}</div> : null;
}
</storycode:copy>

### Assemble `apps/web/src/components/StartupNotice.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/StartupNotice.tsx"><storycode:paste select="#f-9a5486901220-StartupNotice"/></storycode:file>

</storycode:doc>
