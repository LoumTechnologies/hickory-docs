# A protected workspace save is validated against the revision it read

Given an agent workspace frontend using `workspace_fs::Engine`, when a buffered
save replaces a source document or a generated text product, then the engine
checks the source revision and original bytes, maps product edits through the
existing lineage implementation, validates the candidate weave, and refuses a
save that cannot reproduce the proposed bytes before publishing the source.
The durable write and live-room update occur
under one room lock with a final source comparison; debounced room persistence
uses that lock too, so an older snapshot cannot finish saving over it.
A reverse save spanning several source documents is refused. Generated reads
are derived from the source view; they do not trust stale backing products.

Given a temporary-file replacement of a protected path, when its destination
was read earlier, then the save uses that read's revision. A newer source cannot
silently replace the base at rename time. Without a preceding destination read,
a protected replacement is refused.

Given filesystem reads, when a callback returns bytes, then the session records
access evidence with content identity, observed byte range, source revision,
and available lineage. This evidence does not claim the model saw those bytes.
Structured ACP text reads record the context actually returned; successful
reverse saves record the existing source-write evidence. A refused buffered
close retains its proposed bytes in the engine until the frontend is dropped.

Given native-workspace mode is requested, when the extension cannot mount, then
ACP connection fails with setup instructions and does not silently use the
backing directory. A build that enables the extension bundles it in Hickory and uses user-space
FSKit, without macFUSE or a kernel-extension installer.

---

Last LLM verification:

- Date: 2026-09-30
- Reviewer: Codex
- Result: partially verified
- Evidence: `serve/workspace_fs/{engine,view,host,native}.rs`, ACP `Client` cwd
  and file dispatch, `apps/desktop/fskit/{WorkspaceExtension,WorkspaceVolume}.swift`,
  `examples/fskit_bundle.rs`, `scripts/dist-desktop.sh`, `AgentSettings.tsx`,
  `hickory-collab::RoomRegistry::{replace_source_if_current,persist_now}`.
- Tests: `tests/workspace_fs.rs` exercises buffered reverse saves, live rooms,
  stale revisions, temporary-file replacements, invalid new/existing sources,
  synthetic-byte refusal, path/link confinement, read-only history, distinction
  between access and model-context records, and private-host error transport.
  The collaboration test
  `a_pending_old_persist_cannot_overwrite_validated_publication` verifies that
  an in-flight older save completes before validated publication can proceed.
  Actual Swift FSKit code compiles, with warnings as errors.
- Caveats: signed extension activation and mounted Codex have not been verified;
  this machine has no valid signing identity or authorized provisioning profile.
  Publication is one source save, not a multi-file filesystem transaction.
  The backing directory still uses the existing watcher. Native caching/descriptor lifetimes, crash recovery
  of pending buffers, large projects, permissions, links, and real build-tool
  compatibility require further validation. The mount is not a process sandbox
  and does not intercept explicit backing-path access. The frontend remains opt-in.
