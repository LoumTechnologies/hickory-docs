# A Python agent with a Rust authority boundary

Decision of record, 2026-10-08. Origin: the user's request to run a standalone
Python agent in Hickory-managed confinement, use Pydantic Monty for model-written
code, retain consequential assumptions for later debugging, and require extra
permission to modify observed human input.

The first implementation is `hickory-agent`, a Rust CLI supervising an installed
Python package. LangGraph orchestrates the loop. The worker requests model calls
and workspace tools over bounded JSON-lines stdio. Rust owns the conversation
sent to providers, the provider keys and actual tool receipts. A worker cannot
replace those receipts with its own test verdict. Model replies select either
one Python snippet or a declared completion summary. Rust binds the requested
snippet and summary to the recorded response; Monty interprets snippets without
filesystem mounts. No snippet falls back to CPython or a shell.

The LSP/DAP comparison needed a correction: their installers are confined, but
the server processes themselves currently launch without that confinement. This
runtime reuses Hickory's sandbox detection and OS mechanisms through a separate
process profile. The cell profile's broader read access is unsuitable here.
This is OS process confinement, not a VM. Windows support remains unimplemented
and refuses to launch.

## Consequential assumptions and choices

1. **Python-only means executable agent snippets.** Project artifacts may use
   the team's language, consistent with the existing generator guarantee. The
   optional clarification was unanswered; this interpretation preserves that
   established contract. Revisit if the user wants all authored code restricted.
2. **Rust remains authoritative.** Python orchestration may change without
   moving workspace permissions, approvals or provider keys into the worker.
   Stdio RPC gives the CLI and UI the same possible seam without embedding
   CPython into the engine. The initial CLI borrows the existing MCP tool engine.
3. **Installed runtime code is trusted.** The non-editable venv and base Python
   installation are readable. Workspace content and home-directory credentials
   are not direct process inputs. Credentials deliberately placed in a trusted
   runtime directory fall outside this separation. The alternative of mounting
   the whole repository was rejected because it exposes unmediated reads.
4. **Approval is an independent input.** Every exposed write is proposed and
   reviewed in the parent process. Unknown authorship is protected, too. The
   current mechanism checks the proposed edit and source closure again after
   approval. This detects changes during review; it is not an atomic compare-and-
   swap against arbitrary editors racing the final engine write. Shared writer
   coordination remains necessary for that stronger guarantee.
5. **Checks must describe what happened.** Existing `verify` writes outputs;
   the bridge withholds it and debugger execution instead of giving them implicit
   write authority. Completion is labeled unrun. A later read-only verification
   path or staged derived-output diff should provide executable feedback without
   weakening protected-content permission.
6. **Audits record discovered assumptions.** Neither a prompt nor a skill proves
   every hidden assumption was found. The guarantee-maintainer tools remain
   available, preserve superseded entries and pin declared evidence dependencies.
   The session also retains snippets and proposals, including refused work.

The guarantee-maintainer skill can use `record_assumption`, `record_decision`,
`record_guarantee_review` and `audit_assumptions` through Monty's `tool` callback.
Its judgments remain model declarations. Executed checks and human judgment are
separate. A clone still needs current guarantees and their evidence references
in the repository; a private session is an optional historical account.

## Authorship still to build

This change records observed proposals, approvals and existing tool write
receipts. It does not implement a complete per-byte writer ledger or claim that
git blame authenticates human composition. The next layer needs durable writer
events and byte-range ancestry that survive acceptance, copying, undo, external
edits and synchronization. The display must distinguish observed human input,
observed agent writes, derived bytes and unknown origin. Approval must never
erase AI ancestry. The protection policy should live in the shared engine so
ACP and every other writer receive it, not only this new CLI.

UI adoption and packaged environments remain separate work. Promoting this
runtime to the default requires the existing conversation rendering, cancellation,
approval queue, evidence states and release packaging to preserve these boundaries.
