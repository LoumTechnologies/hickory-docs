# Deliver a verified final changeset from an ACP agent

For adapter authors integrating an ACP v1 agent with Hickory's projects, ordinary
workspaces, and unsaved editors. This contract lets the agent repair its final deliverable before its
turn is reported complete. It adds no write permission or execution sandbox.

## Complete one turn

1. On `initialize`, recognize the client's capability and advertise the same
   version in `agentCapabilities._meta`:

   ```json
   {"hickorydocs.com/changesets":{"version":1}}
   ```

2. When `session/prompt._meta["hickorydocs.com/changesets"].required` is true,
   retain its `boundary`. Hickory has already captured A, including dirty output
   bytes. Work normally through the configured tools.
3. Call MCP `changeset_status` on the supplied `hick` server. The result contains
   `boundary`, `startingFingerprint`, current `expected` fingerprint, output
   `differences`, and a replayable `response.md` template. Pass
   `include_snapshots: true` to also receive complete `starting` and `current`
   byte snapshots; the default avoids sending unchanged code twice. A template refusal does not prevent a
   supported explicit construction from being submitted.
4. Replace placeholder prose with the concise successful explanation. Submit:

   ```json
   {
     "name":"submit_changeset",
     "arguments":{
       "boundary":"TURN_FROM_PROMPT",
       "expected":"FINGERPRINT_FROM_STATUS",
       "entry":"response.md",
       "sources":{"response.md":"COMPLETE LITERATE DOCUMENT"}
     }
   }
   ```

5. Inspect the JSON result inside the MCP text content. `accepted: false`
   includes differences and repair guidance; tool errors carry validation or
   stale-state diagnostics. Repair and resubmit. Read status again after further
   output writes, including a build that changes captured outputs.
6. End the turn only after acceptance. Hickory rechecks B at completion. It sends
   at most three repair prompts if an `end_turn` arrives without acceptance.
   A non-completion stop reason is not successful artifact completion.

The document executes against captured A in the portable project evaluator.
Verification does not launch commands. Supply every included `.md` in `sources`;
verification cannot fetch missing documents from the working tree. Full entry
comparison detects omitted files and changed modes. Acceptance validates outputs,
not whether the explanation accurately describes why the change was made.

## Use ACP directly

After negotiating version 1, the agent can issue the client request
`_hickorydocs.com/changeset`. Every request requires the current `sessionId` and
`boundary`. Set `operation` to `status` or `submit`. Status accepts the same
`include_snapshots` flag as MCP. Submit uses the same
`expected`, `entry`, and `sources` fields as MCP. Both paths share acceptance
state and recording. Unknown sessions and expired turn boundaries are refused.
The capability is also exposed in Hickory's ACP connection snapshot.

Capability absence or an unsupported version leaves ordinary ACP behavior in
place. Do not assume the standard Codex or Claude adapters implement this
extension. They can use the MCP tools voluntarily; mandatory finalization needs
adapter participation. ACP v2 is not implemented here.

## Use standalone MCP

Call `begin_changeset` with `doc` before editing a declared project, or with
`workspace` naming an explicit directory before editing ordinary files. The MCP
server captures A and refuses a second begin so a failed artifact cannot be
"fixed" by recapturing B as A. Then use status and submission above. This
explicit task boundary survives engine forwarding for that MCP client. Relative
project defaults resolve against that client’s root, independently of the shared
engine’s working directory. The boundary does not survive an engine/server
restart; standalone MCP cannot enforce when another harness
ends its turn.

## Review and debug the result

Hickory records accepted artifacts separately from trial work and renders their
patches with captured whole-project context. Source equality does not resolve
ambiguous native-write attribution. Resolve that separately before incorporating
into maintained source. Accepted artifacts are immutable. Choose **Revise New** to create a separate
authored draft; its guarded application workflow is described below.

## Workspace and editor boundaries

The dock freezes A before starting agent work. With no declared project, the
open folder is the capture root. All descendants participate, including hidden,
binary, ignored and newly created files. The fixed exclusions are `.git/`,
`.hg/`, `.hick-cache/`, `.hick-journal/`, and `sessions/`. There is no build or
dependency exclusion inferred from file extensions or `.gitignore`. Folderless
windows capture editors only; the execution directory is never their workspace.
Standalone MCP requires an explicit directory and has no connected editor state.

Keep each editor's stable `id` for the whole turn. The captured `workspace`
context records `identities`, mapping IDs to backing paths or null, and
`provenance`, retaining each supplied buffer instance URI. Untitled instance
URIs include the window lifetime; an old record cannot bind to a reused tab ID. Every editor
has its own evaluator address: `__hickory_editor_buffers__/` followed by the
lowercase hexadecimal UTF-8 bytes of its ID. This reserved namespace cannot
coexist with a real workspace entry at that prefix. These are non-filesystem
addresses. Native project materialization and pipeline output publication refuse
them. Capturing never saves a buffer or creates a repository.

Disk and editor state are separate channels. A saved file can contain `disk A`
while its editor contains `dirty A`; both survive capture, and the editor channel
is authoritative for what the person sees. Native writes to disk cannot hide
behind an unchanged dirty editor. Deleting a backing file is captured as a disk
deletion while the still-open editor retains its own bytes and identity. Multiple editors for one backing path must
agree. Untitled editors need no filename, extension, folder or repository.

The browser advertises `context.live: true`. At status, submission and successful
completion the host requests fresh context through `edits.captureRequest` in the
ACP controls snapshot. Respond on the existing edits endpoint with
`capture_request` and `context`. The request token expires after ten seconds;
a disconnected window produces a diagnostic, never an accepted stale snapshot.
The browser reads every open editor again, including inactive editors. Existing
non-browser peers without editors need no refresh exchange. Older editor clients
that supply buffers without fresh-context support retain ordinary conversation;
their capture is recorded as incomplete and cannot be finalized. A negotiated
peer requires a usable capture before its work starts.

Opening, closing, saving or renaming an editor changes the frozen identity map
and is refused. Restore the original editors and paths and retry, or stop and
start a new turn. Saved editors outside the folder,
symlink-backed editors, excluded paths, duplicate IDs and conflicting editor
copies also refuse capture with diagnostics. File creations, deletions and
renames inside the folder remain ordinary complete-inventory changes; renames
replay as deletion plus creation. Executable bits and symlink targets use the
existing metadata comparison. Other filesystem metadata and empty directories
are outside the tree model. Sockets, devices and FIFOs refuse capture; keep
those entries outside the folder being captured. Unsupported generated textual transitions still
require explicit constructions or report an error.

Directory snapshots are read twice and compared. Moving writers cause a retry
diagnostic. This observes a stable tree; it is not an atomic filesystem
transaction and cannot detect an ABA write between reads. External writes are
still attribution questions even when equality verification passes.

Use the `entry` returned by status rather than hardcoding `response.md`: leading
underscores avoid collisions with ordinary workspace files of that name. Final
artifacts render with frozen A and the supplied closure. Disk patches in a
workspace capture offer the existing project tools against the complete captured
filesystem inventory. The native projection removes only reserved editor entries;
it never overlays buffer text onto a disk file or supplements dependencies from
the current working tree. Existing language/runtime discovery still determines
which tools can run. Editor-address patches and folderless captures have no
compatible executable project, so native tools stay disabled there. Workspace
incorporation remains unavailable without maintained literate source.
Built-in/Python turns share workspace capture and the manual finalizer; their
mandatory agent repair-loop integration remains separate work.

Run `just test-agent-changesets` for the real ACP/MCP peer and verifier cases,
and `just test-agent-changesets-web` for final-document rendering and proposal UI.
The peer is deterministic and tests the wire contract; it cannot establish
real-provider explanation quality or third-party adapter compliance.

## Revise an accepted result

For contributors extending the final-artifact editor or publication handlers.
After finalization and attribution resolution, choose **Revise New**, edit a
patch's New code, and wait for C construction to succeed. Patch comparisons
need explicit patch IDs, as ordinary Markdown patches do. Other construction
kinds retain their captured read-only rendering. **Apply revision**
changes the captured destinations; **Discard draft** leaves them alone. Old and
the accepted artifact remain visible and read-only. Only one patch can have
unchecked edits at a time. Drafts are local to the mounted view and do not
survive closing it or restarting.

For example, the agent changed `old` (A) to `new` (B). Replacing New with
`revised` constructs C using the same WASM `regenerate_patch` as an ordinary
Markdown patch. The entire closure, including later patches, must evaluate.
Application checks for `new` at the destination and writes `revised`; it never
tries to replay the `old`-to-`revised` patch over `new`.

`POST /api/docs/{doc}/agent/projects/{turn}/draft/apply` receives `sources` and
fresh `context` from the workspace editor. It re-evaluates the supplied closure
with captured A, checks that atomic Old is unchanged and the accepted artifact
still reproduces B, and compares the current inventory to B. Workspace capture
checks the frozen ID/path map, so saving, renaming, closing or opening an editor, or changing its window instance,
requires restoring the original identities before applying. Disk and buffer
bytes remain separate channels. Standalone declared-project captures have no
editor channel; untitled revisions require the original connected workspace.

Disk differences use `project_publish::publish_text`, the text publication path
shared with literate source editing. Each write checks the expected bytes and
live room, preserves existing permissions, and uses the engine write lock.
Creations use no-clobber publication. Editor differences return identified
transactions and use `applyAgentEdit` with an exact old-text guard; the workspace
also rechecks the tab's backing path. Untitled text stays in its editor.

The response separates `constructed`, `runtime`, `applied`, `error`, `editors`
and `undo`. The browser reports each editor result independently and appends a separate
client-reported acknowledgement on `draft/result`. The displayed draft is read-only while Apply is in flight. Cross-file
and disk/editor application is **not atomic**. On failure, stop applying and keep
the application receipt: `undo` gives each landed disk path's before/after text.
Restore a path only after checking it still has those after bytes; use editor
undo for a landed buffer transaction where supported. No automatic rollback
runs over concurrent changes. A network failure can make the UI's outcome
unknown: inspect the separate `authored-changeset-revision` and
`authored-changeset-application` and `authored-changeset-editor-application`
session records and compare destinations before
recovering. The accepted artifact is never rewritten.

The tree model checks file bytes, kind, executable bits and symlink targets.
Other filesystem metadata and empty directories are outside it. New code edits
preserve kind/mode; binary, symlink and executable-mode revisions are refused
before publication. External processes are not locked by the engine: final
check/write races and ABA writes remain limitations of guarded filesystem
publication. A read-only target can produce a partial batch after earlier writes.

A draft has no inherited output-equality or runtime claim. Successful evaluation
establishes construction of C only. Native language/debug tools use the exact
complete evaluated draft project containing the displayed bytes. Unchecked
edits lose the preceding context. Editor-address patches still have no native
executable project; no working-tree dependency is borrowed to fill a gap.
Authored revision records describe an edit by the user workflow, not proof of
human authorship or correctness. Built-in/Python agent behavior and review
policy are unchanged.

Validation commands: `just test-agent-changesets`,
`just test-agent-changesets-web`, `just test-workspace-changesets-browser`, and
`just check-agent-changesets`.

The 2026-10-17 implementation checks passed 26 native cases, 72 focused web tests,
2,198 full web tests, both real Chromium workspace/folderless flows, and focused
Clippy/formatting/file-length checks. Native recipes used `CARGO_INCREMENTAL=0`
after rebuilding stalled local macOS compiler-library artifacts. Runtime checks
of arbitrary revised user programs remain explicit and unrun by Apply.
