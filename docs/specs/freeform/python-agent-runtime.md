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
erase AI ancestry. An initial browser challenge also observed ordinary editor
paragraph reflow joining an inserted soft Markdown newline to the existing
paragraph. That later editor write is distinct from the approved tool write;
line-hash receipts alone cannot establish the ancestry of every remaining byte.
The browser acceptance fixture uses headings to isolate the agent review flow,
not to certify complete byte attribution through paragraph reflow.
The protection policy should live in the shared engine so
ACP and every other writer receive it, not only this new CLI.

## UI integration (2026-10-08)

The user's follow-up authorized UI adoption. Hickory Python is selectable beside
the Rust agent and ACP agents. A machine-local absolute virtual-environment path
is configured in Settings; saving it probes the actual confined worker without
calling a provider. No project-supplied executable is discovered or run implicitly.
The installed environment is still trusted, not bundled or authenticated by that
probe. Runtime packaging and promotion to the default remain separate work.

Both clients use `python_agent::run_conversation`. The UI supplies conversation
identity, prior branch prompts/answers, editor context and a per-turn Stop flag.
Raw model protocol replies are retained as inert context; the conversational
answer is the final summary labeled checks unrun. Usage and Rust-observed tools
stream through the existing run channel. Source reads and model request envelopes
feed the existing receipts; creating a document records its agent write hashes.

The Python approval queue is separate from native/ACP policy and rejects
Auto-accept. It flushes current room snapshots after approval so pending human
keystrokes participate in the freshness check. After a tool, existing CRDT
correspondence preserves concurrent typing when reconciling changed rooms.
Rejection reasons, invalidated approvals, cancellation and failures remain in the
session. Arbitrary outside editors can still race the final filesystem write.

Assumption: a stored runtime path expresses the user's choice of trusted code;
the alternative of an automatic project-local interpreter was rejected because
it would execute workspace code before the worker boundary was established.
Falsifier: a worker starts from an unconfigured project path, or native/ACP
Auto-accept causes a Python write without a separate review gesture.

Witnesses are real confined HTTP/WS tests and a real Chromium setup/review/reload/
Stop flow with only the paid model replaced. Those checks establish mechanics,
not semantic approval, complete assumption coverage or complete byte authorship.

## Runtime packaging (2026-10-10)

The earlier line that the installed environment is "not bundled" is now
narrowed: a **macOS desktop build** bundles a self-contained runtime at
`Contents/Resources/python-agent`, built by
`scripts/build-python-agent-runtime.sh` from the same locked environment, and
the app discovers it from its own location so no path has to be configured.
The probe still trusts what it is given; bundling packages the trusted
environment rather than authenticating it. The CLI archives still do not carry
the runtime, a cross-compiled Intel macOS bundle is skipped because `uv`
installs the host interpreter, and Linux/Windows desktop bundles are sequenced.
The guarantee is
[`a-desktop-app-bundles-its-python-agent.md`](../../guarantees/agent/a-desktop-app-bundles-its-python-agent.md).
