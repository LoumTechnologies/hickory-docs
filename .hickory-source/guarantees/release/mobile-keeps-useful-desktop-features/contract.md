<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Use Hickory on a phone, with desktop tools when connected

For: a Hickory user reading, editing and reviewing work on iOS or Android, and
maintainers extending those workflows without complicating the desktop app.

Requirement decision, 2026-10-26: target the desktop feature set wherever it is
useful on mobile. Use Tauri v2 for the mobile shell and the existing Iroh 1.0
peer channel to reach the desktop. Share code where it keeps both applications
simple; sharing every shell or layout is not a requirement. “Tari V2” in the
request is interpreted as Tauri v2, consistent with the existing desktop shell.

Status: initial implementation, with host and browser evidence below. The
separate mobile shell, local recovery storage and Iroh document client now
exist. The feature table remains the target scope; complete desktop parity and
acceptance on real iOS/Android devices are still pending.

## Keep working as capabilities change

A phone must remain useful without a desktop connection. It must open locally
available documents, edit and save them, render supported content and inspect
cached results. When a paired desktop is reachable, it should expose useful
desktop tools through that session, within that machine's grants. A small screen
alone is not a reason to remove an editor or a review workflow.

The interface must distinguish available locally, available on a named desktop,
temporarily disconnected, permission denied and unsupported. A disabled action
must explain the reason and the useful next step. Capability comes from the
selected session and platform adapter, rather than screen width or user agent.
These capability states do not replace authorization on the serving desktop.

| Workflow | Phone alone | Paired desktop connected |
|---|---|---|
| Read, edit, search and navigate documents | Local files and supported local indexes; retain edits across restart | Also access the selected desktop session with its view/edit grants |
| Render blocks, outputs, transcripts and lineage | Render supported local content and cached evidence; show stale, unavailable and never-run states truthfully | Retrieve current session data where the peer API permits it |
| Run cells, checks and process-based agent tools | Explain that the local execution environment is unavailable; retain the document | Request execution on the named desktop only with its execute grant |
| Terminal, language server and debugger | No local process-backed service; retain useful static navigation | Offer suitable remote controls after their protocol and grants support them |
| Git history, diffs and merges | Cached inspection first; local operations require an in-process backend | Use supported desktop operations; do not assume every desktop route is peer accessible |
| Import, capture and export | App storage, document picker and platform share integration | Explicit transfer or session access; never imply access to every desktop file |
| Workspace layout | Touch navigation and a usable editor with the software keyboard open; accommodate larger screens and hardware keyboards | Same navigation, with the selected machine and operation location visible |

This table groups the work; it is not an exhaustive audit of desktop features.
Before advertising parity, inventory the actual desktop actions and record for
each one its local implementation, remote implementation or justified omission,
along with acceptance evidence. Prioritize document authoring and review over
reproducing desktop window chrome.

## Share behavior at existing boundaries

Keep `apps/desktop/` as the full-engine Tauri shell. Add a separate
`apps/mobile/` Tauri v2 shell that does not depend on `hickory-cli` and therefore
does not acquire its PTY and process-management dependencies. Both shells consume
ordinary Rust crates and the existing web package; no new plugin framework is
needed to achieve sharing.

Reuse the parser, document model, transcript derivation and other portable
mechanisms as inspected. Reuse React editor, renderer and review components
where they work with touch. A mobile entry point may compose those components
differently. Keep platform storage, picker access, secure credentials, lifecycle
and keyboard integration at the mobile boundary.

Extract a shared application service only when a concrete mobile workflow needs
it. Do not move the entire CLI server into a new framework up front, thread
mobile checks through desktop handlers, or require the desktop to use mobile
storage. If a small amount of shell glue is easier to maintain than an
abstraction shared by two platforms, keep the glue separate. Shared backend
changes need a focused desktop witness for the behavior they affect.

Keep generated API models and adapters in the existing generation workflow.
Mobile transport must carry bodies, errors, cancellation and live updates, not
just GET requests. Start by evaluating the existing loopback forwarding path;
its WebView origin, local authorization and lifecycle behavior need explicit
mobile verification before reuse. If that path does not fit, place the adapter
at the client transport boundary and preserve route and model definitions.

## Reuse Iroh without expanding trust

The checkout already declares Iroh `1.0` in `hickory-peer` and locks `1.0.3`.
Reuse its public-key identity, pairing, reachability and per-request grant checks.
Keep Iroh in Rust; Tauri does not require a second Swift or Kotlin networking
implementation. Cross-compilation and real-device connectivity of the complete
peer dependency graph remain to be established.

Follow the [peer-channel contract](../../../guarantees/collaboration/the-peer-channel-carries-grants/contract.md)
and [identity contract](../../../guarantees/collaboration/a-machine-is-a-keypair/contract.md).
Pairing does not grant execution. A phone cannot grant itself additional rights
or read desktop provider keys. Remote tool support is conditional on explicit
route/protocol support; unknown routes remain denied. Capability negotiation must
be versioned and must not open a general proxy to desktop APIs.

Keep the current direct, default-relay and own-relay choices. Say when traffic is
relayed and when discovery publishes addresses; direct-only mode may report a
peer unreachable. “Peer-to-peer” does not promise that every network permits a
direct connection. Mobile settings must expose the relevant choices without
requiring a user to configure shell environment variables.

## Preserve work when the connection disappears

Local saves must complete inside app-controlled storage independently of peer
availability. Distinguish a local document from a window onto a desktop session.
For remote editing, either persist a recoverable local draft with its source
revision, or clearly provide a connected-only view until that recovery path is
built. Never silently replace newer desktop content when reconnecting. An
unacknowledged remote write is not shown as saved on that desktop.

The current peer channel carries live session traffic, not durable repository
replication. The phone must not describe a successful connection as a completed
sync. Git synchronization and durable document transfer need their own explicit
protocol, conflict handling and persistence evidence before they are offered;
adding a mobile client does not silently change the existing channel's purpose.
The earlier HTTPS Git experiments remain possible groundwork, not a prerequisite
for the first connected mobile workflow.

Do not queue execution, terminal input or other side-effecting commands for
automatic replay. If a connection drops after a request was sent, report that its
outcome may be unknown and refresh authoritative state before offering a retry.
Reconnect must not duplicate a run or an agent turn. Outputs retained from an
earlier connection remain cached evidence with their provenance and freshness.

Treat suspension and process termination as normal. Save while the application
is active, release or pause connections as appropriate, and reconcile state on
resume. Background work is opportunistic, not a promise of an always-running
peer daemon. A desktop going to sleep removes remote capabilities while leaving
local editing available.

## Build and accept one complete workflow at a time

1. Establish the mobile dependency boundary on iOS and Android. Build a separate
   Tauri shell using a minimal portable document path. Verify that its actual
   dependency graph and exercised paths do not require desktop executables.
   Pin supported OS versions explicitly; historical SDK defaults are not a
   support policy.
2. Deliver local open, edit, save, reopen and cached rendering using shared
   components. Check keyboard layout, scrolling, safe areas and narrow-screen
   controls. Demonstrate local persistence after app restart and truthful
   never-run/stale states without a desktop.
3. Pair with an existing desktop over Iroh. Open a real desktop document and
   complete one permitted edit through the existing session protocol. Exercise
   disconnect, draft recovery and resume; display the machine and reachability.
4. Add a capability response and one remote execution flow. Prove that view/edit
   grants refuse execution, execute permits it on the selected desktop, and a
   disconnect cannot cause automatic replay. Test revocation while connected.
5. Extend the desktop feature inventory through review, Git and remote tools.
   Add durable synchronization only with a separate persistence design. Finish
   device signing, store packaging and real-device acceptance for each platform.

At each step, run only focused behavior and safety witnesses. Shared document
behavior needs its relevant desktop witness; mobile lifecycle and WebView
integration need an emulator/simulator and then real-device checks. The first
end-to-end acceptance is: edit and reopen locally offline, pair, view and edit a
desktop document, request an authorized desktop run, lose connectivity, recover
the draft without overwriting newer work, and resume without repeating the run.

## What inspection establishes, and what it does not

The [current peer implementation](../../../mechanisms/crates/hickory-peer/src/tunnel.rs.md)
already has authenticated connections, live upgrade forwarding and a loopback
forwarder. Its small request helper does not by itself implement the full editing
client. Its narrative ledger still contains explanation gaps; ownership does not
establish mobile correctness.

The [portability witness](../../../mechanisms/crates/hickory-cli/tests/portability.rs.md)
checks five crates for selected process-spawn spellings. It does not audit their
entire dependency graph or establish device behavior. The
[iOS core probe](../../../mechanisms/experiments/ios-core/src/main.rs.md)
and the preserved shipping design contain historical simulator findings, not
fresh acceptance of this feature set. No probes were rerun for this decision.

This requirement supersedes the older “phone: read, capture, sync” product scope
and its unconditional claim that the entire frontend must be shared. The
[captured shipping design](../../../support/files/docs/specs/freeform/shipping-mobile-and-desktop.md.md)
remains history; its dated platform and dependency findings require rechecking
as each implementation step reaches them. Existing collaboration authorization
and durable-state boundaries continue to apply.

Upstream references checked for this plan: [Tauri mobile development](https://v2.tauri.app/develop/),
[mobile platform plugins](https://v2.tauri.app/develop/plugins/develop-mobile/) and
[Iroh 1.0](https://www.iroh.computer/blog/v1). These establish framework
capabilities, not acceptance of Hickory on either mobile platform.


## Try the initial mobile implementation

Install the existing web dependencies, then build or preview the mobile page:

```sh
npm --prefix apps/web ci
npm --prefix apps/web run build:mobile
npm --prefix apps/web run dev:mobile
```

The browser preview supports local authoring and restart recovery. It labels its
browser storage explicitly and disables native-only pairing. It is not evidence
of native mobile lifecycle behavior. From `apps/mobile/`, a configured mobile
SDK can initialize and run the Tauri platform shell:

```sh
cargo tauri android init
cargo tauri android dev
# On a Mac with Xcode and signing configured instead:
cargo tauri ios init
cargo tauri ios dev
```

The host currently stores identity as an owner-readable fleet key file in its
app container, reusing the existing identity implementation. Native keychain
integration, a document picker/share sheet, durable Git sync, remote terminal,
LSP/debug controls and store delivery remain implementation work. App-private
drafts can be lost by uninstalling the app; this slice has no export workflow.

For an existing desktop session, `hick fleet pair --new` creates the phrase the
phone joins. Exchange invitations in both directions as an alternative. Start
`hick fleet serve <loopback-session-address>` on the desktop, then paste the
printed session address JSON into the phone. The address includes the desktop
key and is checked against enrollment. Grant remote execution deliberately on
that desktop with `hick fleet grant phone execute`; pairing alone cannot do so.
The native phone identity is currently named `phone`, so a second phone needs a
separate naming workflow before both can be enrolled under distinct names.

The implemented phone flow opens a desktop document through its existing Yjs
room, retains recoverable CRDT state locally and restores that state on reopen.
Remote outputs fetched by the user persist with the recovery copy and are always
labeled cached. Desktop persistence acknowledgment is not part of this client;
a local save message must not be interpreted as a desktop save acknowledgment.
Remote execution uses a source hash at admission and a one-shot request, with
explicit run-status refresh after an uncertain result.

## Focused evidence and remaining acceptance

On 2026-10-26, the mobile web production build, Linux Tauri host compilation,
two native draft-store tests, the direct-only Iroh HTTP/WebSocket grant flow,
eight frontend persistence/transport tests, fourteen desktop fleet-pane tests,
the focused fleet and CLI phone-grant witnesses, and a phone-sized browser
authoring flow passed. Those checks establish the exercised host and browser boundaries,
not app installation or lifecycle behavior on iOS/Android. The initial host test
uses a loopback HTTP/WebSocket stand-in; it does not establish every desktop API
or tool works through the phone. Device toolchains are absent in this session.

The focused commands are:

```sh
cargo test --locked --manifest-path apps/mobile/Cargo.toml -p hickory-mobile-core --lib
cargo test --locked --manifest-path apps/mobile/Cargo.toml -p hickory-mobile-core --test peer
cargo check --locked --manifest-path apps/mobile/Cargo.toml -p hickory-mobile
npm --prefix apps/web test -- src/mobile/bridge.test.ts src/mobile/transport.test.ts src/mobile/recovery.test.ts src/api/realtime.test.ts
# With the mobile Vite server running; Chromium must be installed for Playwright:
HICKORY_E2E_URL=http://127.0.0.1:1420 HICKORY_E2E_CHANNEL=chromium npm --prefix apps/web run test:e2e -- mobile.spec.ts
```

Run the complete native document-edit-to-desktop-run flow on both real platforms
before claiming mobile support. Test suspension, process termination, a lost
network, grant revocation and the software keyboard there. Broad regression
execution and monitoring remain the maintainer's responsibility.

## Maintained mobile implementation

- [asset-apps-mobile-Cargo.lock](asset-apps-mobile-Cargo.lock.md).
<book:include file="asset-apps-mobile-Cargo.lock.md" />
- [asset-apps-mobile-Cargo.toml](asset-apps-mobile-Cargo.toml.md).
<book:include file="asset-apps-mobile-Cargo.toml.md" />
- [asset-apps-mobile-core-Cargo.toml](asset-apps-mobile-core-Cargo.toml.md).
<book:include file="asset-apps-mobile-core-Cargo.toml.md" />
- [asset-apps-mobile-src-tauri-Cargo.toml](asset-apps-mobile-src-tauri-Cargo.toml.md).
<book:include file="asset-apps-mobile-src-tauri-Cargo.toml.md" />
- [asset-apps-mobile-src-tauri-capabilities-default.json](asset-apps-mobile-src-tauri-capabilities-default.json.md).
<book:include file="asset-apps-mobile-src-tauri-capabilities-default.json.md" />
- [asset-apps-mobile-src-tauri-tauri.conf.json](asset-apps-mobile-src-tauri-tauri.conf.json.md).
<book:include file="asset-apps-mobile-src-tauri-tauri.conf.json.md" />
- [asset-apps-web-mobile.html](asset-apps-web-mobile.html.md).
<book:include file="asset-apps-web-mobile.html.md" />
- [asset-apps-web-src-mobile-mobile.css](asset-apps-web-src-mobile-mobile.css.md).
<book:include file="asset-apps-web-src-mobile-mobile.css.md" />
- [bridge](bridge.md).
<book:include file="bridge.md" />
- [drafts](drafts.md).
<book:include file="drafts.md" />
- [frontend-witnesses](frontend-witnesses.md).
<book:include file="frontend-witnesses.md" />
- [host](host.md).
<book:include file="host.md" />
- [interface](interface.md).
<book:include file="interface.md" />
- [peer-witnesses](peer-witnesses.md).
<book:include file="peer-witnesses.md" />
- [peer](peer.md).
<book:include file="peer.md" />
- [room-recovery](room-recovery.md).
<book:include file="room-recovery.md" />
- [web-build](web-build.md).
<book:include file="web-build.md" />

</book:doc>
