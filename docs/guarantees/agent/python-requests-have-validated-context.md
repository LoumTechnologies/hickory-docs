# Python model requests carry validated context manifests

Given the built-in confined Python agent,
when Python selects messages at a model boundary,
then it registers Python constructions, preserves captured read/editor references,
and submits an ordered manifest covering each message's exact UTF-8 bytes, role,
branch, refresh policy and observed cursor. Rust resolves references against its
immutable store and validates complete segment coverage before constructing the
messages passed to the provider adapter. Rust adds no prompt or refresh policy.

Captured references alone do not establish delivery. The host records the actual
validated assembly and separate prepared, transport-started and completed-response
receipts. Provider confirmation means this host observed a completed response
stream; its receipt names a retained response node, not a provider-assigned ID.
A request ID cannot start transport twice in one worker lifetime. An identical
successful replay returns the retained result; a conflicting assembly or an
uncertain/failed replay is refused. Python's retries are bounded and reuse the
pinned assembly. Cancellation of active transport records uncertain cancellation;
timeouts are uncertain, while reported transport errors are failed.

Model outputs retain typed production receipts linking their producing request,
response bytes and supplied parents. This establishes genealogy, not semantic
support. Python-rendered observations and supplied editor wrappers declare opaque
parentage rather than invent exact mappings. Prompt templates, schemas and
supplied prior history are explicit Python roots. Prior history lacks original
producer receipts in the existing bootstrap API; it is supplied history, not a
reconstructed claim of earlier provider delivery.

The injectable in-memory context client needs no Rust process. Its append-update
baseline drains queued updates only at Python-selected model boundaries. Updates
arriving in flight remain queued for a later request; retrying does not recapture
or mutate the prepared request. Hosted polling takes one bounded page per boundary,
appends change notices and asks the model to reread changed inputs. It never waits
for editing to stop. Polling monitors at most 256 selected references; unmonitored
references retain lineage but carry no freshness guarantee. Cursor failure triggers
explicit reconciliation, never an assertion that nothing changed.

Context references and model excerpts never populate complete editable buffers.
Existing full-buffer capture, review and stale-write guards remain authoritative.
The update button and expanded-view projection belong to #51 and #52.

---

Last LLM verification:

- Date: 2026-10-18
- Reviewer: Codex
- Result: verified for the stated built-in path and explicit coverage boundaries.
- Originating requirement: GitHub #50 and #47; `docs/specs/freeform/live-agent-context.md`.
- Evidence: `apps/agent/src/hickory_agent_worker/context.py`, `core.py`,
  `hosted.py`, `runtime.py`; `crates/hickory-cli/src/python_agent.rs::Host::handle`;
  `crates/hick-context-store/src/lib.rs::Store::model_response`.
- Witnesses: `apps/agent/tests/test_context_runtime.py` covers Unicode/tampering,
  immutable late updates, bounded retries/polling and compaction ancestry.
  `serve_python_agent::python_turn_requires_review_streams_receipts_and_rehydrates_its_branch`
  compares retained manifests with actual adapter submissions, receipt transitions,
  reviewed writes and branch continuation. Existing stale-proposal test protects
  full editable buffers independently of model context.
- Assumptions and exclusions: provider adapter internals/hidden prompts are opaque;
  completed-stream receipt is host observation. No live paid-provider check.
  Generic rendered tool observations retain conservative opaque parentage; exact
  read nodes retain original bytes/ranges. Root history does not recover old
  request genealogy. Deduplication is scoped to a worker; crashed workers are not
  resumed with inference retries. No claim about semantic understanding or truth.
- Challenges/falsifier: uncovered or changed request bytes accepted by transport;
  a late update mutating an old manifest; context excerpts replacing full buffers;
  or a replay starting transport twice would violate this guarantee.
- Dependencies: #48 protocol (`ff01e02`) and #49 immutable store (`3d7e429`); changing request construction,
  adapters or snapshot handling invalidates this review.
- Executed checks: focused Python worker tests (10 passed) and context runtime
  tests (6 passed). Focused Rust tests passed: model-response restart genealogy,
  manifest/actual-submission comparison with branch continuation, stale-proposal
  preservation, and active-inference cancellation. Initial confined startup failed
  with an editable install; installing the locked non-editable package resolved it.
  `cargo check -p hickory-cli --quiet` passed. Broad regressions are left to the user.
- Human judgment: not recorded.
