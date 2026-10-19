# Context lineage protocol v1

Issue #48 defines this contract before source adapters (#49) and Python agent
integration (#50). The running agent does **not** advertise context v1 yet.
`protocols/context-v1/schema.json` is the wire source of truth. Run
`python3 scripts/generate-context-protocol.py` after changing it; `--check` checks
Rust types, Python types and the wheel's bundled schema for drift. Rust exposes
`hick-context-protocol`; Python exposes `hickory_agent_worker.context_protocol`.
`validate` checks wire shape; `validate_bundle`, `validate_page`, negotiation and
handle resolution apply the relevant cross-field rules. Use these validators at
the trust boundary: deserializing a generated Rust struct
or annotating a Python `TypedDict` alone does not validate lineage.

## Ownership and negotiation

Python owns messages, roles, history, inclusion, budget and refresh decisions.
Rust owns source identities, revision captures, confinement, evidence, reference
resolution and credentialed transport. Lineage grants no permissions.

Before using any `context.*` method, exchange `Hello` offers and call `negotiate`.
Both peers must offer version 1 and `context.changes`; choose the intersection
of capabilities and the smaller frame/page limits, capped at 1,048,576 bytes
(including the terminal newline) and 256 events. Incompatible versions return
`version_skew`, absent polling returns `capability_missing`, and unknown fields
or enum values fail closed with `invalid_schema`. V1 has **only cursor polling**.
There is no unsolicited event stream, multiplexing, or concurrent outstanding
call. Future push requires a separately negotiated transport version and a
queue/demultiplexer implementation, not merely a capability string.

Keep the current Broker's `{id, method, params}` / `{id, result}` or `{id, error}`
matching-response discipline. `validate_frame` checks bounded UTF-8 JSON with
exactly one terminal LF and, when supplied, the expected response ID. Typed
`ProtocolError` objects go inside `error`; existing legacy errors remain legacy
until the context handshake succeeds. A malformed or mismatched frame terminates
that channel; never discard arbitrary frames until a desired response arrives.
The v1 schema is additive to the hosted bootstrap, not a replacement for
`Config(**bootstrap)`: #50 must explicitly consume negotiation metadata before
constructing Config. No host refresh may rewrite Python's history.

| Method | Params | Result |
|---|---|---|
| `context.capture` | `CaptureRequest` | `Node` plus `CaptureReceipt` |
| `context.derive` | `DeriveRequest` | validated immutable `Node` |
| `context.changes` | `ChangesRequest` | `ChangePage` |
| `context.compare` | `CompareRequest` | `Comparison` |
| `context.resolve` | `ResolveRequest` | `Handle`, resolved by Rust |
| `inference` (negotiated extension) | `Bundle` containing Python's exact `Request` | response reference plus `DeliveryReceipt` history |

These are DTOs and validators, not installed RPC handlers. Each requested method
must be in the negotiated intersection; negotiation never creates a capability.
Capture bounds and pagination are applied before serializing a response. Large
artifacts travel by immutable descriptor, not oversized/base64 stdio payloads.
Oversized text must be explicitly captured in bounded chunks with exact mappings,
not silently truncated. Artifact bytes must be digest-verified by their owning
store/transport before use; this offline contract validator checks descriptors
and never dereferences a locator or claims to have inspected those bytes.

## Identity, coordinates and derivation

A `Source.id` is an adapter-owned stable identity, independent of locator and
content digest. A saved file, an unsaved editor buffer, execution output and an
external resource can be separate sources even when their bytes match. Revisions
are immutable and scoped to their source; rename does not necessarily replace
identity. References are session-scoped opaque IDs and cannot be fabricated from
paths or hashes. `Node.id` binds exactly one immutable descriptor for its lifetime.
A duplicate reference in a witness is rejected; #49 must also reject replacement
of an already registered reference across calls.

All spans are half-open, zero-based **UTF-8 byte** ranges, explicitly tagged
`utf8_bytes`. `Node.source_span` locates a captured excerpt in its revision;
node spans index the excerpt, not its full source. Capture must verify source
span boundaries against the full captured snapshot (not available to an offline
witness). `utf16_to_utf8` converts an absolute UI UTF-16 code-unit offset against
the exact revision text, rejecting split surrogates. Line/column UI positions
must first be resolved against that revision, preserving CRLF. Never normalize
newlines before computing offsets or substitute generated-file line numbers.

`validate_bundle` checks a topologically ordered DAG, exact text SHA-256,
references, and ordered contiguous request coverage. Roots include prompts,
user input, tool schemas, source captures and provider responses. All material
can be a root; a filename or role does not establish authority or actual exposure.

- `selection` and `concatenation` mappings reproduce each output byte exactly.
- `normalization` v1 means **CRLF to LF followed by Unicode NFC**, independently
  verified for each declared mapping. Original captures remain immutable. A
  mapping identifies a corresponding input/output span, not an interpolated
  one-to-one offset map inside a normalization that changes byte counts. Use
  smaller verified mappings or report ambiguity when translating inner offsets.
- `summary`, `model_output` and `tool_output` require parent references and a
  producing `ProductionReceipt`; they cannot claim exact coverage or provide byte
  maps. The typed receipt binds model/tool kind, ordered inputs, output and
  captured response references, plus the producing request identity. Input/output
  references must resolve in the witness; historical request resolution and
  receipt authenticity remain the durable adapter's responsibilities. A summary's parents do not prove it is correct.
- `redaction` and `rendering` retain declared parents with partial/opaque coverage.
  V1 does not validate arbitrary renderer/redactor algorithms as exact mappings.
- `opaque` is explicit, has no exact mappings, and remains opaque even if its
  output bytes were received and hashed. Hidden provider inputs are always
  outside the manifest's `host_controlled_only` boundary.

`Request.parts` is ordered. Each part has a unique assembly slot (for example
`messages/0/content/0`, `messages/0/role`, or `tools/0/schema`), authority role and
applicability scope, text or artifact, and ordered node segments. Include every
host-controlled component, including role/schema bytes, separators, history,
summary and non-text content. Multipart content uses distinct ordered slots;
roles are strings to accommodate provider-defined roles without implicit
promotion. JSON tool schemas can be registered as exact serialized text roots.
A slot is an address, not a path to dereference or an instruction to insert data.
The provider adapter must enumerate its actual final assembly and compare it to
these slots before transport (#50); the offline validator cannot discover a
part omitted from both the proposed request and witness.

A segment's bytes must equal its referenced node span, with no gap, overlap or
reordering. Artifacts bind the entire descriptor/range; partial binary mapping
is unsupported. Empty text has no bytes to cover. Branching or compaction creates
new requests/derivations with retained historical parents; it never edits an old
request. Cross-branch source comparisons are explicit, never inferred from a
conversation's newest message.

## Adapter handoff

Reuse `serve/acp/read_receipt.rs::read` and existing `ContextRead` evidence in
#49. Its `sha256` hashes the **whole supplied snapshot**, while `content` is only
the returned excerpt. Preserve the former as source revision evidence; compute
`Node.digest` from the latter. Derive `source_span` from that same snapshot's
actual UTF-8 bytes and returned range, never by multiplying line numbers. Empty
reads must not invent source ranges. Saved files and `serve/agent_context.rs`
editor buffers retain distinct identities; unsaved/untitled captures do not save.
Reuse `hickory-lineage::Provenance` and existing literate/DAP source maps for
known relationships; this contract adds no competing source-map engine or
transcript parser. Map unsupported/synthetic origins to explicit opaque/partial
coverage rather than inferring exact support from a citation or model output.

## Capture, transport and lifetime

`CaptureReceipt.state=captured` identifies the immutable host capture, not
Python consumption or provider delivery. Python receiving the matching response
is the observation that it fetched that capture; a host pipe write alone cannot
prove consumption. #50 must retain that distinction in its session records. A delivery receipt binds request ID, attempt, idempotency key and
SHA-256 of the **complete request object** encoded as recursively key-sorted,
compact UTF-8 JSON. Request objects contain safe integers, no floating-point
fields; do not use this encoding for arbitrary provider/metric JSON. All roles,
scopes, ordering, policy/version, cursor and artifact descriptors are bound.

Delivery states are an append-only history:

```text
prepared -> transport_started -> provider_confirmed
    |               |
    +-> failed      +-> failed
    +-> cancelled_uncertain
                    +-> cancelled_uncertain
```

Only an observable provider receipt permits `provider_confirmed`. This proves
neither hidden provider assembly nor model understanding. Cancellation retains
uncertainty; it never advances a delivered baseline. Terminal states do not
transition to confirmation. An identical receipt/operation replay is idempotent;
reusing a key for another assembly or attempt is `idempotency_conflict`.
`Idempotency.register` returns false for a replay; a caller must return its stored
result without initiating transport again. A failed/uncertain attempt needs an
explicit new key/attempt if Python chooses another delivery. This is not an
exactly-once provider guarantee. Persist keys and receipts atomically with the
session before enabling hosted retries; the contract's registry is in memory.
Cancellation of an in-flight synchronous call uses the existing out-of-band
supervisor cancellation; do not issue a second request on the blocked channel.

Retain references, source revision metadata, receipt history and artifact bytes
while a session or any retained request/child reference pins them. Releasing a
capture cannot invalidate a pinned historical request. Session deletion/explicit
retention expiry may make bytes unavailable: return `missing_reference`, preserve
historical descriptor/digest, and never rebind its ID to current content. Locators
are hints and may expire independently. Restart rotates cursor epoch; persisted
references may survive if the store retained them. This issue defines lifetime
requirements, not a garbage collector or durable store.

## Changes and typed observations

Poll at Python-selected safe boundaries. Cursors bind epoch, branch and monotonic
event position. A page has strictly increasing event sequences and a next cursor
at its last event; no empty page with `has_more=true`. Gaps in sequence numbers
are allowed for filtered events. The caller must chain the returned next cursor;
server code must refuse expired/unretained cursors with `cursor_expired` and
require reconciliation of retained references. Restart, watcher gaps or branch
mismatch do not mean unchanged. `cannot_check`, `ambiguous`, `unavailable` and
`invalidated` are distinct from unchanged; comparison does not choose a refresh
policy. Each request pins its captured revisions even if source heads advance.

`Annotation` is a provider/version/schema-typed evidence envelope. Its evidence
and parent IDs reference immutable nodes; its location references a typed handle.
Provider-specific `payload` is deliberately opaque until that provider's schema
is validated; its contents never grant actions or prove a marker corresponds to
execution. Keep source precision separate from metric semantics. Profiles carry
clock, units, interval, aggregation and semantics (for example exclusive function
time versus inclusive aggregate time), with run scope and no required stop/frame.
External review can have repository/PR/thread/diff-side details in a provider
schema without a fake run. Logs may carry run and source spans alone.

Debugger annotations distinguish `breakpoint_requested`, `breakpoint_bound` and
`paused`. Paused state, values and expressions require run/stop/frame/invocation
scope. `Handle` distinguishes location, stop, frame and invocation, tied to source
identity/revision and a byte span. Agents pass the handle and expected revision
and execution scope to Rust; `resolve_handle` refuses mismatches with
`stale_handle`. Runtime resolution must compare those expectations to **current**
Rust-owned state; matching an old stored handle to itself cannot certify freshness.
Display words such as “PAUSED HERE” carry no executable meaning, are not source
edits, and cannot create handles. No debugger/evaluation capability is added by
this protocol. #64/#65 supply provider validation and real observation surfaces.

## Focused verification

`just test-context-protocol` runs the same checked-in fixtures through both
languages and checks generated-file drift. Fixtures cover exact transformations,
Unicode/CRLF, offsets, modified/uncovered bytes, reference failures, opaque and
binary content, negotiation, paging/epoch expiry, response matching, frame byte
limits, replay/conflict, cancellation/confirmation and stale execution handles.
They exercise contract assembly/validation without a model, browser or capture
adapter. Real capture, durability, provider transport, UI usability and refresh
policies remain separate integration work; passing these witnesses certifies
none of those paths.
