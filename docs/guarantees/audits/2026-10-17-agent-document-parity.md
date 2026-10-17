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
| Recorded text | Protected prefix; only response draft is editable | Authored document source is editable | Preserves the executed conversation record; retain unless an explicit edited-session workflow is wanted. |
| Persistence | Harness writes the session; response draft is held in a process-local map | Document edits use the ordinary document storage path | Draft recovery across app restarts is a gap. `ChatDock.responseDrafts` is not durable. |
| Proposed changes | Review objects containing Old/New text, status, and a destination | Executable `hick:patch` inside a declared project, with predecessor and successor inventories | Full patch parity remains incomplete. A shared comparison widget alone does not provide project semantics. |
| Editing New | Revised text is submitted on Accept | Patch edits regenerate patch source and check later patches automatically | Approval is a useful explicit boundary, but validation and project regeneration differ. |
| Editing Old | Captured destination is fixed for stale-write checks | Authored predecessor may be edited through lineage | Do not make a captured baseline mutable without defining a fork/rebase operation. The desired interaction needs a product decision. |
| Language intelligence and debugging | Proposal review publishes no complete evaluated project or debug surface | `PatchEditor` supplies whole-project inventories, versions, local state and source identity | Missing parity. Do not infer compatible project identity from a path or attach the current workspace silently. |
| Applying a change | Accept writes to the identified editor or disk tool; stale destinations are refused | Edits update literate source and downstream patch construction | Both guard writes, but the destinations and checks differ. |
| Completed project changes | `AgentProject` is offered for the latest successful selected turn and loads after a click | A document renders its declared patches in place | Earlier-turn project reviews and automatic display remain gaps. Completed capture describes work already done, not a pending proposal. |
| Verification | Accept is approval, not execution or semantic verification | A document can be evaluated and its outputs checked | Retain honest verification labels; provide equivalent explicit actions once a proposal has complete project identity. |
| Native commands | ACP adapters and built-in scripts can write outside the document proposal queue | Document patches have explicit construction semantics | Not every possible agent write is a proposed patch today. UI changes cannot enforce this; tool/backend policy must change to satisfy that part of the request. |
| Review policy | Built-in/ACP support auto-accept; Python proposals require review | Authored edits ordinarily save directly | Existing execution-policy distinction; evaluate separately from visual rendering. |
| Receipts | Session links are remapped to the selected branch; response drafts have no receipts | Document lineage comes from evaluated source | Different evidence sources are intentional; shared appearance must not invent provenance. |
| Running cells | Conversation editor supplies no runnable cell bindings | Ordinary document editor binds execution actions | A rendered session is a record, not automatically a runnable authored program. Evaluate an explicit fork rather than rerunning recorded work implicitly. |

## Evidence and limits

LLM implementation review: partial. Inspected `ChatDock`, `ConversationEditor`, `conversationReading`, `protectedPrefix`, `responseSend`, `AgentChanges`, `DocumentReading`, `ComparisonEditor`, `elements/patch/Editor`, and `AgentProject`. These findings are stale if those paths change.

Executed checks: final `just test-agent-audit-web` passed typechecking and all 77 tests in nine files, including scroll-area placement and acceptance of revised proposal bytes. `git diff --check` passed. Tests use mocked providers; no live model or desktop visual inspection was performed.

Human judgment: the request establishes the desired parity, not acceptance of the remaining differences above. Full executable-patch parity and enforcement for arbitrary native agent writes are not implemented by this change.
