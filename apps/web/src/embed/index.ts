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
