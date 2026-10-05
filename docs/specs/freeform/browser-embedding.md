# Browser embedding: a portable document core with optional host services

*Status: incremental implementation, 2026-10-04. The first host-controlled
editor, bounded browser JS/TS debugger and homepage demo are implemented, with
browser-local revision-checked storage and a basic cross-origin iframe. S3
publication and a durable local sync outbox are implemented against
contract fixtures, with mandatory connection probes and explicit conflict
resolution; no real provider has been verified. Full engine semantics and
authenticated remote execution are still work in progress. See
[the embedding contract](../../developers/browser-embedding.md) and
[the portable-boundary inventory](../../developers/browser-portability.md).
Interface and package names below remain provisional where not shipped.*

**Requirement added 2026-10-04:** JavaScript and TypeScript must execute and be
debuggable entirely in the client browser. The homepage at hickorydocs.com must
demonstrate this with editable code and a real debugging session, served as
static assets without an engine server, extension, account, or local companion.

Audience: engineers extracting the existing Hickory document editor and engine
so another application can embed them without installing the desktop app.

A website should be able to open `notes/meeting.md`, edit it, display its
elements and recorded results, and save the document through the website's own
storage. Connecting an engine on another machine should add execution and IDE
services. Disconnecting it should leave a usable document editor.

The same component must support three useful deployments:

1. Entirely client-side, with documents supplied by the embedding application
   or saved in browser storage.
2. Client-side with an S3-compatible API providing durable storage.
3. Connected to a user-run Hickory engine with a working tree, toolchains,
   execution, terminals, language servers, debugging, and agent services.

Storage and execution are independent choices. An S3 workspace can gain remote
execution without moving its source of truth to the execution machine.
JavaScript/TypeScript debugging is a required local browser capability and does
not depend on either S3 storage or a remote machine.

## Product decisions and constraints

This direction widens [local-only.md](local-only.md)'s downloadable-only frontend
decision and its treatment of HTTP as an implementation detail. It preserves
the absence of a required Hickory-operated server, account, relay, subscription,
or product telemetry. A user-run engine and user-chosen bucket are supported
hosts. Building a managed hosting service is outside this plan.

Keep the purpose in [notes-ide.md](notes-ide.md), the language and execution
boundaries in [architecture.md](architecture.md), and the element seam in
[the-minimal-core.md](the-minimal-core.md). The document-extension amendment in
`AGENTS.md` takes precedence over older examples: documents and sessions are
`.md`, including browser-created documents and exports.

- Keep one parser and one authoritative implementation of document semantics.
  Extract Rust code in place and expose it through WASM; do not turn the demo's
  TypeScript weaver into a competing complete engine.
- Rendering and opening a document never execute it or call a model.
- Keep provenance coordinates as UTF-8 bytes, converting at the editor boundary.
- Preserve source bytes, unknown elements, and unsupported constructs when editing.
  A missing capability must not erase source or synthesize a successful result.
- Do not reintroduce the prohibited WASM container runtime. Browser-specific
  JavaScript/TypeScript execution and debugging are in scope through a dedicated
  language backend. This does not require a shell, OCI containers, or arbitrary
  native toolchains in the browser.
- Browser persistence does not become a new provenance family. Durable evidence
  must survive export or transfer to another host.
- Preserve the current desktop and CLI behavior while extracting shared code.

## What exists today

The inventory below is based on source inspection, not new WASM compile or
browser interoperability measurements.

| Existing seam | Reuse | Remaining work |
|---|---|---|
| `crates/hick-lang-wasm` | Real Rust parser in the browser | Exposes structure only; full engine bindings are absent |
| `crates/hick-blocks` | Registry separating element rendering from actions | Portable registrations and their run facts still need extraction |
| `apps/web/src/elements` | React views keyed by block kind | Inject host services instead of reaching for global services |
| `apps/web/src/landing/demos/DemoEditor.tsx` | Browser-only editing with shared decorations | Demo component is not a supported full editor API |
| `apps/web/src/lib/weave.ts` | Browser file/copy/paste weave, provenance, reverse edits | Subset implementation; replace covered behavior with shared Rust |
| `apps/web/src/api/realtime.ts` | Realtime interface and local Yjs state | Global instances and same-origin socket construction need isolation |
| `apps/web/src/debug` | Debug events, controls, gutter, variables and stack UI | Native channel coupling needs an injectable local debug transport |
| `apps/web/src/landing/demos/browserTsServer.ts` | TypeScript compiler already loaded in-browser | Compiler intelligence does not execute or debug code |
| `crates/hickory-executor` | Existing execution boundary | Trait crate also includes native process implementation |
| `crates/hickory-cli/src/serve` | Existing native API and working-tree behavior | CLI coupling, loopback trust, no remote authorization |
| `crates/hickory-peer` | Native pairing, grants, and peer tunnel | Not a ready browser transport or browser authentication mechanism |

`hick-literate` currently depends on native facilities including execution,
debugging, filesystem watching, and supporting native libraries. The mobile
experiments in [shipping-mobile-and-desktop.md](shipping-mobile-and-desktop.md)
are useful prior evidence, but native mobile compilation does not prove browser
WASM compatibility or runtime behavior.

## First deliverable

A separate host application embeds a document editor, supplies one `.md` source
and an asset resolver, receives source changes, and explicitly saves them. It
renders ordinary prose and supported elements without requesting `/api`, opening
a socket, running a command, or loading product analytics. Two instances on the
same page can edit different documents independently.

This proves the embedding boundary before adding a workspace, bucket, or remote
machine. Each subsequent phase has an independently reviewable exit condition.

## Phase 0 — establish the portable boundary

1. Trace full non-executing weave, block rendering, transform fingerprint checks,
   output lineage, and reverse edits. Record required inputs and dependencies.
2. Separate pure document operations from filesystem lookup, transcript-cache
   access, clocks, process supervision, and network calls. Identify dependencies
   that need feature gates or smaller shared types.
3. Inventory API calls and module-level state reachable from the editor, element
   views, asset loading, completions, and realtime setup.
4. Create a shared fixture corpus using existing documents and guarantees:
   bare markdown, frontmatter, rebound prefixes, Unicode, verbatim contents,
   file/copy/paste, includes, conditionals, transforms, sessions, cached and
   never-run cells, and unsupported elements.
5. Produce a measured browser capability matrix. Mark operations as working,
   requiring extraction, or requiring a native host. Do not infer support merely
   from a successful compilation.

**Exit:** a dependency inventory and a minimal WASM probe performing a real
non-executing document operation beyond parsing. Each portability blocker has
a concrete extraction task. Record bundle size and timing as baselines; do not
promise performance targets before measuring.

## Phase 1 — make the editor embeddable

1. Extract the shared editing surface from `DocumentEditor` and `DemoEditor`.
   Keep application menus, workspace panes, fleet, and desktop shell outside it.
2. Introduce instance-scoped host services for document changes, assets,
   capabilities, optional completions, and element actions. Desktop supplies
   its current services; the browser host supplies local ones.
3. Remove implicit `/api` requests and socket creation from the portable path.
   Asset URLs and imports must work under a non-root deployment path.
4. Offer a React entry point with controlled source/revision, read-only mode,
   change events, selection events, and explicit disposal. Specify whether
   each instance is host-controlled or room-controlled; do not mix both writers.
5. Scope styles and mutable registries so multiple editors do not share state
   accidentally. Include loading and WASM initialization errors.
6. Keep the source tree shared initially. Add a distributable package boundary
   when the external-host fixture proves what actually needs to be exported.

**Exit:** the first deliverable runs in a host fixture outside the desktop app,
including two simultaneous instances, mount/unmount, host-driven source updates,
read-only mode, and assets. Network inspection shows no implicit API traffic.

## Phase 1A — real browser JavaScript/TypeScript debugger and homepage demo

Prioritize this milestone ahead of S3 and remote-engine work. It can proceed
with the current parser/editor and a narrowly extracted local debug interface;
it need not wait for the entire portable weave engine or package distribution.
The resulting backend belongs to the product and is shared with the homepage.

### Choose a backend by proving a real pause

A normal webpage cannot assume access to its browser's native inspector.
The [DevTools protocol guidance](https://chromedevtools.github.io/devtools-protocol/index.html)
describes an extension bridge for web IDEs, and
[Chrome's debugger API](https://developer.chrome.com/docs/extensions/reference/api/debugger)
is an extension facility. Neither meets the zero-install static-page requirement.
Adding `debugger` statements alone does not give our UI control of execution.

Prototype the two viable local approaches against the same editable program:

| Approach | Benefit | Risk to prove before selecting |
|---|---|---|
| Instrument JS into resumable execution with explicit debug checkpoints | Native language operations where preserved; can use the existing TypeScript compiler for AST transformation | Correct scopes, call stacks, closures, evaluation order, exceptions, and async behavior require careful transformations |
| Execute JS in a step-capable interpreter with exposed frames and scopes | Execution state and pausing can be controlled directly | Language coverage, runtime APIs, speed, bundle size, and debugger hooks vary |

Prefer an existing permissively licensed runtime if it demonstrates the required
semantics and inspection hooks. For example,
[JS-Interpreter](https://github.com/NeilFraser/JS-Interpreter) explicitly targets
ES5; it is a candidate for a bounded prototype, not evidence of modern JavaScript
support. Transpilation changes syntax but does not supply every missing runtime
API. A runtime that only exposes evaluate/interrupt cannot supply stepping and
locals by itself. Select the backend after the probe and record its language
coverage. Do not quietly reduce the requested debugger to a prerecorded trace
or a demo-specific evaluator.

The prototype must stop before an executable statement, expose its live locals,
step into and out of a user function, continue to a second breakpoint, and stop
an infinite loop without freezing the page. Require actual edited source to
change both the computed result and inspected values. Compare execution results
against the browser's native JS engine for supported language fixtures.

### Build the shared local debug path

1. Inject a debug session transport into `useDebugger`/`DebugClient` instead of
   obtaining a workspace WebSocket implicitly. Reuse existing debug events,
   breakpoint states, controls, editor decorations, frames, and variable views.
   A local worker can adapt the existing wire contract; it needs no DAP server.
2. Compile TypeScript locally using the repo's pinned compiler and documented
   [compiler API](https://github.com/microsoft/TypeScript/wiki/Using-the-Compiler-API).
   Run JavaScript directly through the same execution backend. Supply a local
   entry function or module harness rather than assuming a Node process.
3. Compose mappings from `.md` source through generated files, TypeScript emit,
   and any instrumentation to executable locations. Use authoritative parser
   spans and weave lineage; do not extend the demo's single-block line scan into
   another parser. Resolve breakpoint locations back to the exact source revision.
4. Implement start/stop, pause/continue, step over/in/out, executable-line
   breakpoints, call stack, local variables and expandable values. Support
   bounded, read-only watch expressions and exception reporting; identify any
   unsupported expression explicitly. Avoid invoking getters or user functions
   implicitly during inspection. Advertise additional capabilities only when
   implemented; reverse stepping and arbitrary instruction jumps are optional.
5. Freeze inputs for each run. Editing a paused document marks the session as
   using the previous revision and offers restart; it does not move the paused
   location onto unrelated new bytes. Dynamic code and unavailable imports must
   be refused clearly until the backend can instrument and map them correctly.
6. Run the debuggee off the UI thread with execution/output limits and immediate
   worker termination on Stop. A worker provides responsiveness, not a security
   sandbox: constrain runtime host APIs and verify that debuggee code cannot
   access page credentials or perform arbitrary network requests. Scope any
   sandbox origin/CSP to the runner so marketing analytics stay outside it.
7. Capture real console output and exceptions. An interactive debugging run is
   not automatically a verified cell run: define which inputs, harness, edits,
   and host APIs enter the fingerprint before recording reusable evidence. Keep
   ad hoc debugger evaluations distinct from reproducible execution evidence.

### Make it the homepage's live demonstration

Integrate the shared backend into `LandingView` and its existing demos. The
visitor opens a `.md` document containing a small TypeScript program, clicks an
executable line to set a breakpoint, starts Debug, sees live variables, steps
into a function, and continues to actual output. They can edit an input or
calculation, restart, and observe different values. Include a JavaScript variant
to prove both languages; show compiler diagnostics for TypeScript separately
from runtime errors.

Use the current pricing example or another small program with an explicit entry
point and deterministic inputs. Replace recorded execution where the demo claims
live execution. Lazy-load compiler/runtime assets and keep the editor usable
while loading. Once static assets are loaded, disconnecting the network must
not prevent execution, pausing, inspection, stepping, or restart. Shipping a
static build does not mean the initial asset download happens without a host.

**Exit:** on a production-style static-site build, Playwright in Chromium,
Firefox, and WebKit executes the visitor flow above for JS and TS, with all API
requests and WebSockets blocked. Assert the paused source line, frames, locals,
changed result after editing, exceptions, rejected unsupported syntax, and Stop
on an infinite loop. Check multiple sessions, teardown, Unicode/multiline source
mapping, closures, and breakpoint changes. Produce a reviewable homepage preview;
deployment is a separate step. Record any language/API restrictions in the demo
and embedding contract rather than requiring a remote fallback to pass.

## Phase 2 — share deterministic document semantics through WASM

1. Extract a portable core around source bytes, a virtual workspace resolver,
   explicit evaluation facts, transcripts, and recorded outputs. Choose crate
   names after the dependency audit rather than creating a parallel engine.
2. Keep native executors and process supervision behind the host boundary.
   Split shared execution-result types from process implementations as needed.
3. Expose parse, render, non-executing weave, lineage, transform fingerprint
   checks, and reverse edits through a versioned WASM interface. Missing inputs
   return structured diagnostics and partial results where semantics permit.
4. Resolve includes and assets through an explicit workspace view. A render uses
   one consistent revision of its inputs. Cancel or discard obsolete results
   when the source changes; retain byte coordinates tied to their source revision.
5. Move expensive work into a worker when measurements justify it. Keep workers
   disposable and ensure stale responses cannot replace newer editor state.
6. Replace the browser weaver's covered behavior with calls to shared Rust.
   Port useful demo fixtures into parity checks, then delete duplicated semantics.

**Exit:** native and browser results agree on supported fixture outputs,
provenance, diagnostics, and reverse edits. A cached result cannot become fresh
merely because a browser loaded it. Unsupported cases retain source and explain
what is missing. Publish the supported subset if full extraction takes more
than one increment; do not advertise full parity before proving it.

## Phase 3 — browser workspace and honest degradation

1. Define storage operations for reading, listing, revision-checked writing,
   deleting, and assets. Define notifications separately; polling is valid.
   Start with host-supplied and in-memory adapters.
2. Add durable local persistence using an appropriate IndexedDB/OPFS design,
   with import/export of `.md` documents, assets, and required evidence. Detect
   quota failures and distinguish memory-only edits from durable local saves.
3. Add optional user-granted filesystem access after checking browser support
   and embedding restrictions. Import/export remains the fallback.
4. Add portable lexical search. Native semantic search remains an optional host
   service unless independently demonstrated in the browser.
5. Define structured capability results per service/action: available,
   unsupported, permission denied, disconnected, or missing prerequisites.
   Discover executor/toolchain support rather than using one `hasServer` flag.
6. Show separate editing, local persistence, remote synchronization, and run
   states. Missing execution disables Run with a reason while keeping editing
   and recorded results usable. Browser-supported JS/TS retains local Run/Debug
   when the remote engine is absent. Separate offline fingerprint checks from tests
   that require re-execution.

**Exit:** edit, reload, import, export, quota failure, and offline use work without
a server. Source/evidence round-trip to native Hickory. Missing includes, stale
results, and unsupported actions are visible without repeated failed requests.

## Phase 4 — S3-compatible storage

1. Specify the minimum backend requirements: HTTPS, browser CORS, object reads
   and writes, and verified conditional-write behavior. Treat ETags as opaque
   concurrency tokens, not provenance hashes. Probe each advertised provider.
2. Store immutable, content-addressed revisions and assets; publish an immutable
   workspace manifest referencing them. Commit a workspace change by updating
   one head object conditionally against its previous ETag. Readers follow only
   a published manifest, so an interrupted upload does not publish half a batch.
3. On a failed conditional update, fetch the new head and reconcile against the
   recorded base. Extract/call the existing merge machinery where portable.
   If a safe merge is unavailable, retain both versions and ask for resolution;
   never silently overwrite. Include deletions and renames in the manifest.
4. Keep an offline queue and the original merge base. Save locally first and
   report remote synchronization separately. Bound polling and retry behavior.
5. Accept scoped credentials or host-supplied signing/presigned-request callbacks.
   Specify renewal, expiry, and revocation. A presigning service is an additional
   component; claim S3-only operation only when provisioning supports it.
6. Leave unreachable-object cleanup out of the initial release; it needs an
   explicit retention and concurrent-reader policy.

**Exit:** two browser sessions cannot lose edits silently. Test conflicting
updates, interrupted batch uploads, offline edits, expired credentials, deleted
assets, and export of a complete workspace. Document provider prerequisites.

S3 supplies durability, not execution or git. Object revisions do not provide
git blame, branches, recipe commits, or the publication floor. Keep git-specific
surfaces unavailable unless an actual git service/library provides them. The
manifest is a storage mechanism and must not masquerade as git history.

## Phase 5 — supported remote engine

1. Extract the native server from CLI coupling as already proposed by the
   minimal-core plan. Add a supported headless launch path for one workspace.
   Decide the command spelling then; no command below is implied to exist today.
2. Version the embedding-facing protocol and publish capabilities. Configure HTTP
   and WebSocket endpoints independently of `location.host`, including base paths.
3. Provide a deployment path behind authenticated HTTPS/WebSocket. Check allowed
   origins, authenticate upgrades, authorize every operation and workspace path,
   and confine file/asset lookup to the selected root. Reuse peer-grant semantics
   where suitable; a paired native key is not automatically browser authentication.
4. Preserve the desktop's loopback mode. Remote publication must explicitly
   select its authentication policy and execution trust boundary. A permitted
   unsandboxed local executor still runs with host privileges; document that
   deployment assumption and do not imply multi-tenant isolation.
5. Initially make the engine's working tree authoritative for connected native
   workspaces. Identify workspace/revision on connect; never seed a remote room
   with an independently initialized copy of the same source.
6. Add execution, terminal, LSP, DAP, and agent services incrementally. Attach
   input revision/fingerprint and host identity to run results. Keep credentials
   at the configured credential owner; engine keys do not travel to the embed.
7. Allow local draft editing during disconnection. On reconnect, reconcile against
   the pre-disconnection base before adopting the live room. Do not blindly
   replay queued keystrokes or automatically re-run a disconnected command.

**Exit:** the browser performs a real run, observes its transcript, edits a file,
and uses supported IDE services on a remote machine. Permission rejection,
disconnect/reconnect, stale revisions, and socket authentication are verified.

SSH can provide access or tunneling for a person with an SSH client. A normal
browser speaks HTTPS/WebSocket to an engine or gateway; direct browser SSH is
not a prerequisite. Reaching a local companion from a hosted page requires a
separate interoperability check for browser network policies and permissions.

## Phase 6 — iframe integration and independent remote execution

1. Ship a static iframe entry point over the same component. Define a versioned
   `postMessage` handshake for capabilities, load/save, selection, errors, and
   disposal. Validate sender origin and window; never use wildcard credential
   delivery. The host supplies storage when iframe persistence is unavailable.
2. Test iframe sandbox/CSP settings, cross-origin storage restrictions, nested
   asset paths, focus, keyboard shortcuts, and resizing. Publish exact supported
   configurations rather than assuming parity with a top-level page.
3. Add optional execution for browser/S3-authoritative workspaces. Submit an
   immutable input manifest, required files, and requested actions. The remote
   engine executes in an explicit job workspace and returns output/evidence.
4. Persist accepted results through the authoritative storage provider. Apply
   document changes with revision checks; never promote results from old inputs
   to current results. Handle cancellation, output limits, and disconnected jobs
   without duplicate execution.

**Exit:** one embed reads/edits an S3 workspace offline, connects to an execution
host, runs a cell, stores its evidence, then remains usable after disconnection.
The same integration works through a supported iframe host contract.

## Validation and rollout

Keep changes incremental and use existing native guarantees as behavioral
requirements. Add tests where extraction crosses a real boundary:

- Native/WASM parity for byte-sensitive document operations and diagnostics.
- Browser flows in Chromium, Firefox, and WebKit for the advertised core;
  filesystem enhancements receive a separate, narrower support matrix.
- React multi-instance and iframe isolation, including cross-origin hosts.
- Storage conflict, failure, and export/import scenarios.
- Remote authorization, input revision, reconnect, and cancellation scenarios.
- Real browser-only JS/TS debugging and homepage flows with server traffic blocked.
- Existing desktop and CLI checks for each extraction affecting those paths.

Add guarantee documents as behavior ships; do not mark proposed capabilities
verified. Run repository-required formatting, lint, and file-length checks.
Report bundle size, initialization time, large-document latency, and persistence
limits with each supported release. Set budgets from the phase-0 measurements.

Recommended sequence is phase 0, phase 1 plus the prioritized phase 1A debugger
and homepage demo, then phases 2–3, S3 storage, the full remote engine, and
iframe/hybrid completion. The debugger proof must precede promises of client-only
debugging and need not wait for full core extraction. The basic iframe wrapper can ship after phase 1
if it is the immediate integration need. Native server extraction can proceed
independently after its API and trust boundary are defined.

Update product-shape docs and the short `AGENTS.md` entry point with the first
shipped browser capability. Keep this plan linked as work in progress until its
exit conditions are met. A remote or S3 mode may ship before another mode, but
its release copy must name which storage, semantics, and actions actually work.

## Decisions to settle during the first phase

| Decision | Proposed default | Evidence needed to change it |
|---|---|---|
| Public embedding surface | React component first, iframe wrapper over it | First consuming application's integration constraints |
| Portable engine boundary | Extract deterministic Rust operations | Dependency audit and a real WASM probe |
| Initial persistence | Host callbacks, then browser-local adapter | Required durability, quota, and browser support |
| S3 publication | Immutable revisions/manifest plus conditional head | Provider support and conflict tests |
| Remote transport | HTTPS/WebSocket; SSH-managed deployment optional | Deployment and authentication requirements |
| Browser computation | Required local JS/TS execution and debugging; no general command runtime | Backend probe, language coverage, source mapping, and homepage acceptance flow |
| Git without a native host | Unavailable initially | Portable git implementation and semantic compatibility proof |

## Browser and storage references

- [Directory picker support and restrictions](https://developer.mozilla.org/en-US/docs/Web/API/Window/showDirectoryPicker)
- [Origin-private filesystem persistence and limits](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system)
- [S3 conditional writes](https://docs.aws.amazon.com/AmazonS3/latest/userguide/conditional-writes.html)
- [S3 presigned requests and credential lifetime](https://docs.aws.amazon.com/AmazonS3/latest/userguide/using-presigned-url.html)
- [S3 CORS behavior](https://docs.aws.amazon.com/AmazonS3/latest/userguide/testing-cors.html)
