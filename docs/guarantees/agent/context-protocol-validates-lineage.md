# Context protocol validates declared byte lineage before integration

Given a self-contained context v1 witness, when it is checked by Rust
`hick_context_protocol::validate_bundle` or Python
`context_protocol.validate_bundle`, then unknown node references, changed text
hashes, modified request segments, coverage gaps/overlaps and false exact
transformation mappings are rejected with typed errors. Summaries and model/tool
outputs retain typed producer genealogy without claiming exact correspondence or
semantic support. Opaque lineage stays explicit. Immutable artifact descriptors
are checked without pretending the validator fetched their bytes.

Given version/capability offers, the v1 negotiation selects bounded cursor
polling and refuses incompatible peers. It does not enable unsolicited frames
on the existing synchronous Python Broker. Cursor pages preserve epoch, branch
and ordering. Request delivery receipts bind the exact host-controlled assembly;
capture, preparation, transport start, provider confirmation and cancellation
remain different facts. A cancelled/failed attempt cannot become confirmed, and
replays cannot change an idempotency key's bytes or attempt.

Given a Rust-owned location/stop/frame/invocation handle and expected revision
and execution scope, resolution rejects mismatches. Typed requested/bound/paused
annotations remain distinct; marker prose cannot manufacture a paused state.
Logs, profiles and external review evidence use the general annotation envelope
without fabricated debugger fields. Metrics retain their units, clock, interval,
aggregation and declared semantics separately from source precision.

This is a protocol-library guarantee, not a claim that the running agent already
uses context v1. Python continues to own context policy. No debugger, execution,
filesystem, refresh, or outbound capability is granted.

---

Last LLM verification:

- Date: 2026-10-18
- Reviewer: Codex
- Result: partially verified; contract and offline validation reviewed, live
  integration intentionally belongs to #49/#50 and observation providers to #64/#65.
- Originating requirement: GitHub #48, parent #47, and the context/location
  clarifications in #64/#65. Interpretation: implement the versioned contract
  before adapters, durable stores, UI disclosure or refresh policy experiments.
- Evidence: `protocols/context-v1/schema.json`; generated Rust `types.rs` and
  Python `context_types.py`; both `context_protocol` validators; the normative
  method, lifetime and ownership contract in `docs/developers/context-protocol.md`.
- Test coverage: `crates/hick-context-protocol/tests/conformance.rs` and
  `apps/agent/tests/test_context_protocol.py` consume the same checked-in fixtures.
  Python additionally exercises the actual `runtime.Broker.call` with matching,
  unsolicited and out-of-order responses. Generation drift is wired into
  `scripts/check-codegen.sh` and the focused `just test-context-protocol` recipe.
- Assumptions: the request is a self-contained declared assembly, not a discovery
  of hidden provider state. Source IDs are adapter-owned, independent of paths
  and content hashes. UI offsets are converted against the exact revision;
  normalization is explicitly CRLF-to-LF followed by NFC with span correspondence.
- Exclusions: the offline validator cannot detect a request part omitted from
  both the manifest and declared assembly. #50 must compare actual final provider
  assembly to the manifest. Source adapters must verify source-span boundaries
  against full snapshots; historical producer request identity, provider receipt
  authenticity, artifact bytes, durable idempotency, retention/GC and current
  execution scope require the Rust-owned stores/adapters. An in-memory replay
  registry does not guarantee exactly-once provider delivery. Provider payloads
  remain opaque until their own schemas are validated. These are integration
  boundaries, not evidence of implemented runtime support.
- Alternatives: unsolicited push was rejected for v1 because Broker.call expects
  its next matching response; a future negotiated multiplexing implementation
  could change that constraint. No refresh strategy was selected.
- Witnesses and challenges: fixtures include Unicode combining marks, a non-BMP
  character and CRLF; exact normalization/selection/concatenation; corrupt bytes;
  missing parents and producer receipts; summary and opaque coverage; role-bound
  request digests; branch/compaction genealogy; binary descriptors; cancelled
  deliveries; conflicting retries; expired cursors; fabricated markers; and stale
  stop/frame/invocation/source revision handles. A validator accepting any of
  those negative witnesses would disprove its stated behavior. The implementations
  and fixtures share a schema and were authored together, so they are not fully
  independent specifications or a proof of exhaustive corner-case coverage.
- Dependencies: exact implementation/schema/fixture hashes are recorded below.
  Earlier executed results become stale if these or generated types change.
- Executed checks: recorded below after focused verification. Broad regression
  checks and detached post-commit status monitoring are left to the user.
- Human judgment: no human correctness review or semantic approval recorded.

Executed checks on this change (2026-10-18):

- `CARGO_TARGET_DIR=target/context-protocol just test-context-protocol`: passed;
  90 shared fixtures in each language, Rust generated-type round trips, and
  two Python tests including the real Broker matching-response witness.
- `cargo clippy -p hick-context-protocol --all-targets --target-dir target/context-protocol -- -D warnings`: passed.
- Focused `rustfmt --check`, `bash -n scripts/check-codegen.sh`, and
  `git diff --check`: passed. No paid model, browser, full regression suite or
  post-commit status monitoring was run for this contract-only slice.

Inspected dependency SHA-256 values:

- `protocols/context-v1/schema.json`: `6e589e3177271678e450a675aae7ff490bb984ae7a7195a437a8ab7a70ecf0e7`
- `protocols/context-v1/fixtures.json`: `64808e8bd602ce74b1863ab10c1b667c40caa7b22b299e3512b766bdd48558ff`
- `crates/hick-context-protocol/src/lib.rs`: `568e264ff3ede9a5cee95239d6087fcf704912e936213e05bedb54ed627f9752`
- `crates/hick-context-protocol/src/types.rs`: `46d82bbaab45455afce651673a114963e84029b8d596b8349fc608bd4ee4a099`
- `apps/agent/src/hickory_agent_worker/context_protocol.py`: `0d395fe4962de5ea0dcf292372f0d0900052df66ed29466072d49f6542f01653`
- `apps/agent/src/hickory_agent_worker/context_types.py`: `b08c887eafb84bd5c4e018ac0307d41f8a471d19dc03715387100aed17f4e110`
