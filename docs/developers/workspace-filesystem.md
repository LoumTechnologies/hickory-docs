# Work on the agent workspace filesystem

For engineers changing native agent file access or adding another frontend.

The native frontend is a **preview**. Its Swift code compiles against Apple's
FSKit SDK and the Rust engine is tested. A signed, enabled FSKit mount and a
Codex turn through it have not yet been verified. Do not advertise full native
filesystem support until that smoke test passes.

## One engine, several frontends

`crates/hickory-cli/src/serve/workspace_fs` is the facade. `Engine` owns open
handles and text-client reads; `View` derives source bytes, products, and
lineage; `Host` exposes the engine on a private random loopback endpoint;
`Mount` is the macOS frontend's lifecycle. ACP file callbacks and the FSKit
extension use the same request operations when native-workspace mode is on.
The MCP bridge shares its operation gate. MCP remains the existing document
implementation; it is not reimplemented in Swift.

A mount belongs to one ACP conversation, so its access record has a known
session even though FSKit does not supply a caller identity here. Prompt starts
supply the turn id. OS cache hits need not reach the extension: these are
**observed filesystem accesses**, not an exhaustive syscall log or proof that
the model saw the bytes. Only the structured `read_text` response records
model context. Reverse saves record the source's existing `wrote` evidence.

Reads pin bytes and lineage to the open revision. Writes and truncation buffer
until close or sync. Before a protected save, the engine checks the revision,
maps output differences with `up::reverse::source_edits_for_save`, validates the
candidate weave, and checks that it reproduces the proposed bytes. The final source comparison,
durable write, and live-room update share the room lock. A stale or
invalid save is refused; the handle keeps the buffer and records the refusal.
A temporary-file rename onto a protected path uses the revision of the
preceding destination read, not the newer revision at rename time.

The mount projects generated products from the source, not from backing output
files. Backing disk products still converge through the existing watcher; this
is not atomic publication to unrelated programs reading the backing directory.
Existing handles continue to read their pinned revision. Multi-document reverse
saves are refused rather than claiming a multi-file atomic transaction.

The adapter's cwd and ACP session cwd are the mount. **This is not a process
sandbox**: an agent that explicitly accesses the backing directory or another
host path can bypass the mount. Agent-owned command confinement still applies.
Do not claim comprehensive provenance until that separate boundary is enforced.

## Build and bundle

Run `just build-fskit`. It locates full Xcode (or uses `DEVELOPER_DIR`), compiles
with warnings as errors, and produces `.dev/fskit/HickoryWorkspace.appex`.
An ad-hoc signature permits artifact inspection, not restricted-entitlement
activation. No account or mock FSKit implementation substitutes for activation.

For a distributable extension, provide `APPLE_SIGNING_IDENTITY` and
`HICKORY_FSKIT_PROFILE`. The latter is a local provisioning-profile path for
`com.loumtechnologies.hickorydocs.workspace`, with Apple's FSKit module
entitlement authorized. With those available, set `HICKORY_BUNDLE_FSKIT=1`
when running the existing `just dist-desktop` recipe. The extension is embedded
under `Contents/PlugIns` before Tauri signs and packages the containing app.
No macFUSE package, kernel extension, installer helper, or Recovery-mode setup
is involved. This packaging path still needs signed-artifact verification.

After installation, enable **Hickory Workspace** in System Settings → General
→ Login Items & Extensions → File System Extensions. In Hickory's agent
settings, enable the native workspace for the chosen agent, save, and start a
new thread. A requested mount failure fails connection; it never falls back to
the ordinary working directory. The current deployment target is macOS 26.

## Verify

`just test-workspace-fs` exercises the real Rust engine and private HTTP host.
`just test-acp` includes those checks and existing agent/UI regressions.
`just build-fskit` compiles the actual frontend; it does not mount anything.

For a real filesystem smoke test, set `HICKORY_FSKIT_APP` to that signed app's
path and run `just test-fskit-live`. It invokes the app's own smoke entry point,
using a temporary workspace to read, reverse-save, update a live room, and
check persistence and a fresh mounted read. It neither opens a window nor
uses a model account. It must fail when mounting is unavailable.

Before calling the native frontend verified, also use a signed bundled app to test:

1. Enable the extension and connect Codex with native-workspace mode on.
2. Read a generated file with an ordinary shell command, patch it, and verify
   that the source and editor update and a fresh read reproduces the result.
3. Repeat with a temporary-file-and-rename save, a concurrent source edit, an
   invalid document, and a synthetic output. Inspect the refusal and record.
4. Stop/reconnect, close the app, and restart. Confirm mounts clean up without
   traversing a busy mount or losing the source.
5. Exercise real builds and search over a large project before changing defaults.

The mountpoint is outside the descriptor's temporary directory. Cleanup must
never recurse through a mounted workspace after unmount fails.

## Current limits

Files buffer at most 64 MiB. Symbolic and hard links are refused; repository
history, caches, and session records are read-only. Document/output moves and
deletes belong to Hickory's Files pane. Filesystem permissions and extended
attributes are not implemented by the Swift frontend. Those restrictions may
prevent some build tools from working and need real-agent compatibility tests.
Native activation, packaging, cache invalidation, simultaneous descriptors,
mount cleanup on process death, and large-project performance remain unverified.
