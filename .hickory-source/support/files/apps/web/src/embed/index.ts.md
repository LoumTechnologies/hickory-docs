<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/index.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `1b7456ee725b2082bc519dc8f74599121f9e83b810df62eb6e76368710e38143`.

<book:entry path="apps/web/src/embed/index.ts">
export { DocumentEmbed } from "./DocumentEmbed";
export type { DocumentEmbedProps, EmbedHost, Capability } from "./DocumentEmbed";
export { createBrowserDebugger } from "../debug/browser/transport";
export { DebuggableDocument } from "./DebuggableDocument";
export { createMemoryStorage, openLocalStorage, StorageConflict, exportWorkspace, decodeWorkspaceExport, importWorkspace, searchWorkspace } from "./storage";
export type { WorkspaceStorage, StoredFile, WorkspaceExport, StorageMutation, SearchMatch } from "./storage";
export { BrowserWorkspace } from "./BrowserWorkspace";

export { openS3Storage, S3Failure } from "./s3Storage";
export type { S3Options, S3Request } from "./s3Storage";
export { openSyncedStorage } from "./syncedStorage";
export type { SyncedStorage, SyncState, SyncConflict, SyncResolution } from "./syncedStorage";

export { PublishedSnapshot, decodeSnapshot } from "./PublishedSnapshot";
export type { ViewerSnapshot } from "./PublishedSnapshot";

export { StaticUiSurface } from "./StaticUiSurface";
export { createEmbedUiSession } from "./uiHostSession";
export { UiActionContext as EmbedUiContext } from "../lib/uiActionBridge";
</book:entry>

</book:doc>
