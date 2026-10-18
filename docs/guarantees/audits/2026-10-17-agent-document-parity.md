# Agent pane and document behavior differences

Requirement: on 2026-10-17 the user asked for agent-proposed changes to appear as patches with the capabilities of `.md` patches, and for every difference between the Agent pane and documents to be identified for evaluation.

This is a source review of the conversation editor, proposal queue UI, shared comparison editor, literate patch editor, and captured-project review. It is not an exhaustive audit of every element, provider, or platform.

## Changes made

- `AgentChanges` now embeds `DocumentReading` for each proposal in the Agent
  pane's existing scroll area. A separate review tab remains optional; proposals
  no longer automatically open another pane.
- The embedded review uses `ComparisonEditor`, also used by `.md` patches.
  Its comparison presentations and proposed-New editing are shared. Accept
  applies the revised New text; Reject leaves the destination unchanged.
- `responseSend` places Send in a block widget after the response. Typing no
  longer pushes it horizontally, and the widget adds no session bytes.

## Differences requiring evaluation

| Surface | Agent pane | Ordinary `.md` document / patch | Assessment |
|---|---|---|---|
| Recorded text | Protected prefix; only response draft is editable | Authored document source is editable | Preserves the executed conversation record; retain unless an explicit edited-session workflow is wanted. *Agreed* |
| Persistence | Harness writes the session; response draft is held in a process-local map | Document edits use the ordinary document storage path | Draft recovery across app restarts is a gap. `ChatDock.responseDrafts` is not durable. *Let's deal with this now. Drafts should indeed be persisted.* |
| Proposed changes | Review objects containing Old/New text, status, and a destination | Executable `hick:patch` inside a declared project, with predecessor and successor inventories | Full patch parity remains incomplete. A shared comparison widget alone does not provide project semantics. *We need to deal with this now.* |
| Editing New | Revised text is submitted on Accept | Patch edits regenerate patch source and check later patches automatically | Approval is a useful explicit boundary, but validation and project regeneration differ. *We need to deal with this now* |
| Editing Old | Captured destination is fixed for stale-write checks | Authored predecessor may be edited through lineage | Do not make a captured baseline mutable without defining a fork/rebase operation. The desired interaction needs a product decision. *I do not want to support editing old right now, although I don't necessarily want to rule out ever being able to do this.* |
| Language intelligence and debugging | Proposal review publishes no complete evaluated project or debug surface | `PatchEditor` supplies whole-project inventories, versions, local state and source identity | Missing parity. Do not infer compatible project identity from a path or attach the current workspace silently. *We NEED this now.* |
| Applying a change | Accept writes to the identified editor or disk tool; stale destinations are refused | Edits update literate source and downstream patch construction | Both guard writes, but the destinations and checks differ. *Can these be made more similar? Share the same code?* |
| Completed project changes | `AgentProject` is offered for the latest successful selected turn and loads after a click | A document renders its declared patches in place | Earlier-turn project reviews and automatic display remain gaps. Completed capture describes work already done, not a pending proposal. *I want the agent to be told (maybe in a system prompt or however ACP does this normally) to format its final response at the end of the ReAct loop using hickory docs .md file that reproduces the entire changeset without dead ends / bunny trails. And in that final response is the only place where I really want debugging to work.* |
| Verification | Accept is approval, not execution or semantic verification | A document can be evaluated and its outputs checked | Retain honest verification labels; provide equivalent explicit actions once a proposal has complete project identity. *Sounds good* |
| Native commands | ACP adapters and built-in scripts can write outside the document proposal queue | Document patches have explicit construction semantics | Not every possible agent write is a proposed patch today. UI changes cannot enforce this; tool/backend policy must change to satisfy that part of the request. *Is this something we can enforce with our own hickory docs agent(s) but can't with codex or claude code?* |
| Review policy | Built-in/ACP support auto-accept; Python proposals require review | Authored edits ordinarily save directly | Existing execution-policy distinction; evaluate separately from visual rendering. *We need to support workflows where the user reviews every change by the ai but the primary mode is auto-approve* |
| Receipts | Session links are remapped to the selected branch; response drafts have no receipts | Document lineage comes from evaluated source | Different evidence sources are intentional; shared appearance must not invent provenance. *Sounds good* |
| Running cells | Conversation editor supplies no runnable cell bindings | Ordinary document editor binds execution actions | A rendered session is a record, not automatically a runnable authored program. Evaluate an explicit fork rather than rerunning recorded work implicitly. *Yes, that is accurate* |

## Evidence and limits

LLM implementation review: partial. Inspected `ChatDock`, `ConversationEditor`, `conversationReading`, `protectedPrefix`, `responseSend`, `AgentChanges`, `DocumentReading`, `ComparisonEditor`, `elements/patch/Editor`, and `AgentProject`. These findings are stale if those paths change.

Executed checks: final `just test-agent-audit-web` passed typechecking and all 77 tests in nine files, including scroll-area placement and acceptance of revised proposal bytes. `git diff --check` passed. Tests use mocked providers; no live model or desktop visual inspection was performed.

Human judgment: the request establishes the desired parity, not acceptance of the remaining differences above. Full executable-patch parity and enforcement for arbitrary native agent writes are not implemented by this change.

## Implementation continuation after the comments

The user's italic comments above are retained as decisions and requirements.
The follow-up approval authorizes the ACP/MCP changeset contract described in
`docs/developers/agent-changeset-contract.md` and protected by
`docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md`.

Implemented in this continuation:

- MCP task capture, status and final-artifact submission use complete inventory
  replay over frozen A. The agent receives repair diagnostics. No intermediate
  or abandoned output is needed to reproduce B.
- Negotiated ACP v1 peers share that verifier and receive bounded completion
  repair prompts. Legacy adapters remain usable without claiming acceptance.
- Accepted closures are recorded and appear automatically for successful turns
  on the selected branch. They render through ordinary `.md` patch views with
  frozen project input, evaluated versions, language tools and debug context.
- Old and accepted artifacts remain immutable. Source equality remains separate
  from attribution, user approval, explanatory accuracy and runtime checks.

Still unimplemented from the discussion:

- Durable response drafts. `ChatDock.responseDrafts` remains process-local.
- Auto-approve as the default, and unified review policy for the Python agent.
- Built-in and Python harness participation in the mandatory finalization repair
  lifecycle. They retain the earlier post-turn A/B capture and manual finalizer;
  exposing tools in a catalogue alone does not constitute loop integration.
- Adapter adoption by installed Codex/Claude peers, ACP v2 support, or an upstream
  ACP proposal. The new v1 capability is implemented and tested with the existing
  deterministic subprocess peer, not authenticated external adapters.
- Whole-workspace and untitled-editor capture are implemented in the workspace/editor
  continuation of `changeset-completion-is-negotiated-and-checked.md`; editor identity
  transitions currently require a new boundary.
- Editable forks of accepted final artifacts and application of their revised
  New code through the same guarded project/source write pipeline. Pending text
  proposals can be revised before Accept; this is still a separate workflow.
- Enforcement that every native shell write passes through review. No new
  filesystem isolation or execution boundary is introduced.

Executable-mode and binary changes can be checked through explicit constructions;
the convenient generated textual template still refuses transitions it cannot
represent. Checking outputs cannot judge whether a human-readable explanation
has omitted an important reason or contains a dead end.

Continuation checks: `just test-agent-changesets` passed 13 native cases,
`just test-agent-changesets-web` passed typechecking and 65 tests, and
`just test-agent-audit-web` passed typechecking and 77 tests. Focused Clippy,
formatting, file-length and whitespace checks passed. The verifier's status
returns fingerprints, differences and the response template by default; full
A/B byte snapshots require `include_snapshots: true` to avoid duplicating
unchanged source in the agent's context.

Final regression result: the native ACP stages passed 44 tests (two paid/live
provider cases intentionally ignored). After correcting the included-source
debug identity and its asynchronous assertion, `just test-web` passed
typechecking and all 2,180 tests in 236 files. This supersedes the earlier
full-web development run that reported that mismatch.

## Editable final-artifact continuation (2026-10-17)

The earlier editable-fork gap is superseded by
[the final-revision guarantee](../agent/final-changeset-revisions-apply-b-to-c.md).
Accepted artifacts remain immutable. Revise New creates an authored draft using
ordinary patch regeneration and full downstream validation against captured A.
Explicit Apply guards B-to-C disk publication and identified editor transactions;
Discard writes nothing. Tools use captured evaluated draft projects. The developer
contract records per-target recovery, metadata and non-atomic batch limits.
Durable response drafts, harness repair-loop adoption and policy defaults remain
separate work; this continuation does not change them.

Continuation evidence: the native recipe passed 26 cases, the focused web recipe
passed typechecking and 72 tests, and the full web recipe passed 2,198 tests in
239 files. The real Chromium recipe passed both folder and folderless flows.
Focused Clippy, formatting, file lengths and whitespace checks passed. Native
compilation initially stalled on macOS compiler-library loading; affected local
build artifacts were rebuilt. A real process test found and fixed plain-file
publication incorrectly entering the document-room lookup. Browser undo exposed
that applied editor transactions lacked the local user-event marker; they now
enter editor history as isolated transactions. The standalone MCP witness uses
an isolated engine state directory so a running desktop engine cannot supply
older code to the test. Human visual acceptance and real-provider compliance
remain unrecorded.
