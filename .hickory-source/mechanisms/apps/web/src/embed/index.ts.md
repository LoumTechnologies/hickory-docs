<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/index.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1541a1b462e9"></a>
## `apps/web/src/embed/index.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-1541a1b462e9-export_statement"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement">
export { DocumentEmbed } from "./DocumentEmbed";
</storycode:copy>
<a id="f-1541a1b462e9-export_statement-2"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-2">
export type { DocumentEmbedProps, EmbedHost, Capability } from "./DocumentEmbed";
</storycode:copy>
<a id="f-1541a1b462e9-export_statement-3"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-3">
export { createBrowserDebugger } from "../debug/browser/transport";
</storycode:copy>
<a id="f-1541a1b462e9-export_statement-4"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-4">
export { DebuggableDocument } from "./DebuggableDocument";
</storycode:copy>
<a id="f-1541a1b462e9-export_statement-5"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-5">
export { createMemoryStorage, openLocalStorage, StorageConflict, exportWorkspace, decodeWorkspaceExport, importWorkspace, searchWorkspace } from "./storage";
</storycode:copy>
<a id="f-1541a1b462e9-export_statement-6"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-6">
export type { WorkspaceStorage, StoredFile, WorkspaceExport, StorageMutation, SearchMatch } from "./storage";
</storycode:copy>
<a id="f-1541a1b462e9-export_statement-7"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-7">
export { BrowserWorkspace } from "./BrowserWorkspace";

</storycode:copy>
<a id="f-1541a1b462e9-export_statement-8"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-8">
export { openS3Storage, S3Failure } from "./s3Storage";
</storycode:copy>
<a id="f-1541a1b462e9-export_statement-9"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-9">
export type { S3Options, S3Request } from "./s3Storage";
</storycode:copy>
<a id="f-1541a1b462e9-export_statement-10"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-10">
export { openSyncedStorage } from "./syncedStorage";
</storycode:copy>
<a id="f-1541a1b462e9-export_statement-11"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-11">
export type { SyncedStorage, SyncState, SyncConflict, SyncResolution } from "./syncedStorage";

</storycode:copy>
<a id="f-1541a1b462e9-export_statement-12"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-12">
export { PublishedSnapshot, decodeSnapshot } from "./PublishedSnapshot";
</storycode:copy>
<a id="f-1541a1b462e9-export_statement-13"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-13">
export type { ViewerSnapshot } from "./PublishedSnapshot";

</storycode:copy>
<a id="f-1541a1b462e9-export_statement-14"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-14">
export { StaticUiSurface } from "./StaticUiSurface";
</storycode:copy>
<a id="f-1541a1b462e9-export_statement-15"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-15">
export { createEmbedUiSession } from "./uiHostSession";
</storycode:copy>
<a id="f-1541a1b462e9-export_statement-16"></a>
### `export_statement`

<storycode:copy id="f-1541a1b462e9-export_statement-16">
export { UiActionContext as EmbedUiContext } from "../lib/uiActionBridge";
</storycode:copy>

### Assemble `apps/web/src/embed/index.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/index.ts"><storycode:paste select="#f-1541a1b462e9-export_statement"/><storycode:paste select="#f-1541a1b462e9-export_statement-2"/><storycode:paste select="#f-1541a1b462e9-export_statement-3"/><storycode:paste select="#f-1541a1b462e9-export_statement-4"/><storycode:paste select="#f-1541a1b462e9-export_statement-5"/><storycode:paste select="#f-1541a1b462e9-export_statement-6"/><storycode:paste select="#f-1541a1b462e9-export_statement-7"/><storycode:paste select="#f-1541a1b462e9-export_statement-8"/><storycode:paste select="#f-1541a1b462e9-export_statement-9"/><storycode:paste select="#f-1541a1b462e9-export_statement-10"/><storycode:paste select="#f-1541a1b462e9-export_statement-11"/><storycode:paste select="#f-1541a1b462e9-export_statement-12"/><storycode:paste select="#f-1541a1b462e9-export_statement-13"/><storycode:paste select="#f-1541a1b462e9-export_statement-14"/><storycode:paste select="#f-1541a1b462e9-export_statement-15"/><storycode:paste select="#f-1541a1b462e9-export_statement-16"/></storycode:file>

</storycode:doc>
