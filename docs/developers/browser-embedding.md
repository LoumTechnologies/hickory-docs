# Embed a document

Import `DocumentEmbed` from `apps/web/src/embed` in the shared source tree. The
host owns source and storage; each change names the base revision it edited.
Update source/revision after accepting a change. Save explicitly in your host.
The component does not create a CRDT room or contact an engine.

```tsx
const [source, setSource] = useState("# Meeting notes\n");
const [revision, setRevision] = useState(0);

<DocumentEmbed
  path="notes/meeting.md"
  source={source}
  revision={String(revision)}
  host={{ resolveAsset: (path, signal) => myAssets.resolve(path, signal) }}
  onChange={(change) => {
    setSource(change.source);
    setRevision((previous) => previous + 1);
  }}
/>
```

`readOnly` prevents editor input. `onSelection` returns UTF-16 editor positions.
`onViewReady` receives the live CodeMirror view and null at teardown; unmounting
explicitly disposes it. Importing the entry imports scoped styles. Parser assets
are Vite imports and work below a deployment prefix. Parser load failures render
an error; no source is lost. Host-driven changes do not emit a change callback.
Do not attach a room-controlled writer to this host-controlled component.

Use `DebuggableDocument` with the same props for browser-local Debug. It uses the
product's debug client, events, gutter and controls. The first Debug loads a
self-contained interpreter/compiler worker; later runs retain the runner bytes.
A restrictive host CSP needs `worker-src blob: data:` and WASM compilation
permission (`script-src 'wasm-unsafe-eval'`). CodeMirror injects style rules;
the tested style policy allows inline styles. The data fallback
handles WebKit's offline blob-worker loading failure. There is no eval permission
required in the page. Native worker termination makes Stop immediate.

Supported execution is ES5 statements/functions, closures, objects and arrays,
and TypeScript type annotations over that subset. Use `var`; imports, classes,
async, arrows, block-scoped declarations and dynamic code are refused. Call your
entry explicitly in the file, for example `console.log(main(4));`. The component
executes the first literal .js/.ts file; an injected `DebugClient` can select a
program path explicitly. Files are not implicitly joined into a module system. There is no
Node/module harness or implicit shell. Console output is live and bounded to
64 KiB; execution is bounded to two million interpreter steps. Watches support
names, literal property access and primitive arithmetic/comparisons, without
getters/calls/assignments. Debug runs do not create reusable cell transcripts.
Editing a live source labels its previous revision; restart adopts the new one.

`createMemoryStorage` and `openLocalStorage(name)` implement read/list/snapshot,
revision-checked writes/deletes, atomic batches, and close. A null expected revision means
create-if-absent. Never retry a conflict as a blind overwrite. `exportWorkspace`
returns versioned base64 files, including assets/evidence; `decodeWorkspaceExport`
validates transfers without normalizing bytes. `BrowserWorkspace` provides an
optional explicit-save UI, .md import, saved-note text search, and workspace
import/export. Workspace import creates absent paths in one transaction; any
conflict rejects the entire import. Reload explicitly to replace an editor draft. The host owns
storage disposal separately from editor disposal. Local storage is origin-scoped;
incognito/private windows and quota policies affect its durability.

For a cross-origin iframe, serve `iframe.html` and set its `parentOrigin` query
parameter to the exact host origin. Use `sandbox="allow-scripts allow-same-origin"`
with the iframe on a separate origin. The host listens for messages only from
that iframe's window and origin. Both sides use a concrete target origin.

Messages have `channel: "hickory-embed"`, `version: 1`, an optional request `id`,
and either `op` (host request) or `event` (frame response):

| Request | Data | Response |
| --- | --- | --- |
| `load` | `path` ending .md, `source`, `revision`, optional `readOnly`, `assets` path→URL map | `loaded` |
| `save` | none | `save` with current path/source/revision; host performs persistence |
| `dispose` | none | `disposed`; editor/debug worker unmounted |

The frame initially sends `ready` with capabilities and sends `change`,
`selection` and `error` events as they occur. It validates both the sender window
and origin. Opaque sandbox origins are unsupported; storage is host-supplied.
The host must accept changes and send the next revision through `load`.

Run `just test-browser-core` for byte/runtime/editor checks and
`just test-browser-embedding` for Chromium, Firefox and WebKit static acceptance.
Run `just preview-browser-embedding`, then open `/` for the homepage or
`/embed.html` for the external-host fixture (`?workspace` exercises IndexedDB).
No deployment is performed by these recipes.

Full document weave/results, transforms, reverse edits, authenticated remote
engines and hybrid execution remain in the rollout plan. The current editor
preserves their source, but does not claim to execute their semantics.


## S3 and local-first synchronization

`openS3Storage({ prefix, request })` accepts a host callback that supplies signed
HTTPS object requests. The callback receives method, key, headers, bytes and an
abort signal. Preserve and sign the conditional headers, and renew credentials
when needed; the adapter never stores keys. It does not list the bucket.

Opening performs actual disposable-object probes: create-if-absent, successful
If-Match replacement, stale If-Match rejection, changed/visible ETag and readback.
The bucket needs browser CORS for GET/PUT, conditional headers, and exposed ETag.
The host must use HTTPS and configure its allowed origins. A presigning callback
needs to sign each current request, including its conditions; provisioning that
callback can require a signing service. No real provider has been verified in
this change; advertised support must be tested against that provider.

Files/assets and manifests are immutable SHA-256 objects; one `head.json` points
to the published manifest. Batch upload publishes only by conditional head
replacement. ETags are opaque publication tokens. A failed/head-conflicting
batch keeps the prior publication; unused uploaded objects remain. Rename and
delete are ordinary manifest mutations, not git history. No cleanup policy ships.
See [S3 conditional writes](https://docs.aws.amazon.com/AmazonS3/latest/userguide/conditional-writes.html)
and [S3 CORS](https://docs.aws.amazon.com/AmazonS3/latest/userguide/cors.html).

For offline editing, wrap it with `openSyncedStorage(local, remote, name)`.
First initialization requires the remote snapshot; it hydrates absent local
files while retaining existing local drafts. Existing local files are queued
against that snapshot, so the host should initialize a dedicated local workspace.
Subsequent opens can read its initialized journal offline. Local files and their
outbox commit together. The journal keeps original base bytes and remote revision
tokens. This adds storage requirements beyond the note bytes themselves.

`BrowserWorkspace` displays local save and remote sync separately. Sync is
explicit; no automatic retry, polling or background pull occurs. Upload takes
one frozen snapshot and leaves later local edits queued. Conflicts retain both
versions and expose their base/local/remote bytes through `reviewConflicts()`.
The UI offers an explicit saved-version choice. `resolveConflict` checks the
reviewed local and remote revisions before queuing that choice; Sync publishes
it against the chosen remote base. Unsaved editor drafts are preserved separately.
A host can also submit manually merged bytes through that revision-checked API.
There is no automatic portable merge yet.

Sync journal metadata is reserved under `.hick-sync/`; ordinary document/asset
exports omit it. Closing the wrapper leaves its independently owned local and
remote adapters for the host to close. Local quota and memory-only limits still
apply. A connection/acknowledgment failure retains the outbox; do not reinterpret
it as permission to overwrite a changed head.

For a deliberately live example, `DebuggableDocument` also accepts
`startPausedAt`, a zero-based document line. It registers that initial
breakpoint and starts once after both editor and worker transport are ready.
Remount to start a fresh example; ordinary edits do not restart it. A draft
changed before loading finishes cancels the automatic start.
