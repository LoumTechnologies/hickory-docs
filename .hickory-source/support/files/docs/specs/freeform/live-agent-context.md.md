<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/specs/freeform/live-agent-context.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `86dcef61c8712816d454de37cc6afd758c94e6d9a390c68bc696956f730a65d9`.

<book:entry path="docs/specs/freeform/live-agent-context.md">
# Live agent context: observations, changes and refresh policies

Status: context v1 contract implemented by #48, immutable source storage by #49,
and built-in Python request manifests/receipts by #50. Update UI, projections and
refresh experiments remain proposed. See [the wire contract](../../developers/context-protocol.md). Origin: the October 18, 2026 discussion of
progressive disclosure, execution provenance and stale agent context. The user
explicitly wants to compare refresh strategies rather than adopt one prematurely.

## Invariant: delivery schedule is not the data model

The user's clarification is authoritative: do not assume fresh context on each
user message, each model call, or any other fixed schedule. Every item actually
supplied needs tracked lineage, regardless of its capture time or delivery path.
Current editor recapture is an implementation observation, not a future contract.
Freshness is always relative to a particular immutable observation and its source
head. A document being open does not itself establish model exposure.

## Existing behavior inspected

`views/WorkspaceChat.tsx::editorContext` collects complete open editor buffers,
including unsaved content, and `components/ChatDock.tsx::send` invokes that capture
for each submitted user message. This is not solely a conversation-start snapshot.
It does not yet project the expanded document view or provide this proposed
staleness/update interface.

`apps/agent/src/hickory_agent_worker/core.py::run` builds the Python worker's
request history from prior prompts/answers, supplied editor context, the current
prompt and subsequent worker observations. Its model step sends the accumulated
messages; there is no generic source refresh before every model call.

`serve/acp/read_receipt.rs` records the exact returned lines and bytes of a file
read, together with a hash of the supplied file. `serve/audit_evidence.rs::enrich`
can mark a historical read changed or unavailable using the current whole-file
hash. That does not establish whether the particular passage read has changed.
`serve/agent_context.rs` already distinguishes editor snapshots from saved files.

Automatic capture of AGENTS.md by every adapter has not been established. Record
instruction material when it is actually supplied; a filename alone must not be
reported as evidence of model exposure. Adapter-owned, opaque context is a stated
coverage gap rather than an invented complete manifest.

## Separate three records

1. **Observation ledger:** immutable records of what was actually supplied or
   returned, including source identity, revision/hash, returned ranges and bytes,
   request/turn identity, material kind and visibility scope.
2. **Current comparison:** how those observed passages relate to current editor
   or saved state. A watcher is a trigger to compare bytes, not proof of change.
3. **Request manifest:** the context actually assembled for a particular model
   call, including any refreshed projections, notices or appended differences.

Refreshing a request must not rewrite an earlier receipt or retroactively change
what an earlier answer cites. “Supplied to request N” is observable; “the agent
understands the update” is not. Context may be absent from a later compacted or
adapter-managed request even though it remains in the historical ledger.

## Generic context items and baselines

Track material by actual exposure, irrespective of how it entered context:
initial editor snapshot, tool-read range, instruction file, expanded observation,
or an explicitly attached external resource. Do not depend on automatically
sending open files forever. Preserve distinct identities for the saved file,
unsaved editor buffer, and historical execution output.

Record the source revision and exact returned excerpt. Compare against the last
version actually supplied on the relevant conversation branch, not only the
conversation-start version or the last version that happened to exist on disk.
A successful agent edit can advance that baseline only if the worker received an
unambiguous result establishing the resulting bytes. Agent-authored intent alone
is insufficient; accepted edits may differ and rejected edits changed nothing.

For ranged reads, distinguish:

- the observed passage changed;
- only material elsewhere in the file changed;
- the passage moved without changing;
- the source was deleted, renamed, unavailable or has ambiguous correspondence.

Use revision-to-revision edit mappings where available. Never assume that old line
numbers still name the same text, or silently choose among duplicate quotations.
Show ambiguous mappings and offer a fresh bounded read. Expanding a read's scope
is explicit and participates in the context budget.

Instruction updates additionally retain their role, ordering and applicability
scope. A task moving under another directory can change the applicable instruction
set even if no instruction file was edited. Updating context must not silently
promote an ordinary tool result into a higher-priority instruction.

## Non-modal review and update

Display an agent-pane indicator such as “3 observed sources changed.” Expand it
inline to inspect patches between the last supplied passage and the current
candidate. Use the existing patch rendering rather than a second diff language.
Allow sending selected updates or all eligible updates, with explicit coverage
for instruction material. Keep unavailable and ambiguous items visible.

The button captures a specific revision. Show Queued, Included in request N,
Failed, and Changed again separately. Do not advance the delivered baseline on a
button click. If an inference request is already in flight, apply an update at
the next safe model-call boundary; it cannot alter a request already sent.
New edits after capture remain pending. Failed/cancelled deliveries remain pending.

Expanded document sections and expanded execution observations define a context
projection, independently of screen scrolling. Keep the full editing snapshot
and its concurrency guards separate from model-visible excerpts so a partial
context update can never become a whole-file replacement.

## Rust–Python API: proposed contract

Preserve `python-agent-runtime.md`: Python owns prompts, message construction,
context selection and refresh policy; Rust owns workspace access, confinement,
source evidence and credentialed provider transport. The host never silently
injects a refreshed prompt or rewrites Python's conversation. The standalone
Python core retains injectable context/model/tool interfaces with an in-memory
implementation for independent tests and experiments.

### Identity and lineage

Use a generic context graph, not a special changed-open-document endpoint.
A source identity is separate from its immutable revision and from its locator:
a rename changes a path without necessarily changing identity, and equal content
in two files does not make them one source. Content hashes establish bytes,
not source identity, authorship, trust or semantic support.

Every supplied segment references an immutable node with:

- a context reference and exact content/artifact digest;
- source identity, locator and captured revision when observable;
- UTF-8 byte ranges and line mapping for source-backed excerpts;
- typed parents and an operation describing selection, normalization,
  concatenation, redaction, rendering or another transformation;
- evidence kind, authority role, scope and capture/execution/model receipt;
- explicit unavailable or opaque lineage where coverage cannot be established.

Direct roots include user input, source captures, versioned agent prompt templates,
tool schemas, external resource captures and provider responses. A generated
summary links to its producing model request/response and supplied parents; that
records its genealogy without claiming the summary is accurate or that every
parent semantically supports it. A tool observation links to its actual execution
and known inputs. Python-generated context also registers its construction; it
cannot enter the tracked path as an unexplained string. Unknown dependency coverage
is explicit rather than an invented complete derivation.

Represent the whole model request: message roles, tool definitions, supplied
history, summaries, instruction material and non-text artifacts, not just editor
attachments. Byte-preserving transforms can carry exact correspondence; model
paraphrases carry declared semantic relationships. Convert UI UTF-16 positions
at the boundary; never use ambiguous bare integer offsets across languages.
Newline or Unicode normalization is a recorded transformation, not an invisible
change to the bytes used by a citation.

### Operations

Names below are proposed, not existing methods:

| Operation | Python supplies | Rust returns or records |
|---|---|---|
| `context.capture` | Source selector and bounded range | Immutable context reference, bytes/artifact descriptor, revision and source mapping |
| `context.derive` | Typed transformation, parent references and output | Validated context node; verified correspondence or explicitly declared derivation |
| `context.changes` | Branch identity, context references and last cursor | Ordered changes to observed source heads, comparison states and resumable cursor |
| `context.compare` | Old observation and chosen current capture | Diff/mapping references, moved/changed/ambiguous/unavailable state |
| `inference` extension | Python-built messages, context-segment manifest, policy/version and observed cursor | Validated request identity, delivery/failure receipt and model response references |

Capture/read tools return lineage metadata alongside results so the Python loop
does not have to infer ancestry by parsing text. Reuse the existing exact
read-receipt mechanism; convert it into the generic graph rather than creating
an unrelated second file-read history. Bootstrap supplies available context
references and capabilities, not a rule requiring their inclusion on every call.

Before transport, Rust validates references, segment hashes and message coverage.
In the strict tracked built-in path, unresolved references or unattributed bytes
are refused; deliberately opaque sources must be represented explicitly and are
not advertised as fully traced. The request receipt binds the actual validated
message assembly and ordered context references, including role and transformation.
Python's proposed manifest alone does not establish what was sent.

Keep both capture and delivery receipts: fetching content into Python does not
mean it was included in an LLM request. Distinguish prepared, transport-started,
provider-confirmed where observable, failed and cancelled/uncertain delivery.
Do not infer provider receipt merely because a request was queued. Provider-owned
hidden additions are outside observable lineage and must remain an explicit
boundary. Record final host-controlled request assembly, not a claim about hidden
provider internals or model understanding.

### Notifications and races

The existing `runtime.py::Broker.call` expects the next frame to be its matching
response. Unsolicited change frames would currently break that protocol. Begin
with cursor-based change retrieval at Python-selected safe boundaries and a
separate UI change subscription. If push is added later, negotiate a multiplexed
protocol with request IDs, queued events and cancellation; do not slip arbitrary
notifications into today's response stream.

Change detection belongs to source adapters, not an assumption about Git or
files. Rust's filesystem/editor/run adapters report actual observed changes;
external context adapters may expose a version, explicit invalidation, or no
freshness capability. Reconcile content after watcher gaps and cursor expiration.
“Cannot check” is not “unchanged.” Dependency changes can invalidate derived
context without mutating any historical node.

Each inference request is pinned to exact context revisions. Source heads may
advance during capture or while an inference is running; report the newer event
for the next request rather than pretending to mutate an in-flight call. The
API can return a typed context-advanced result at a preflight boundary, but Python
chooses whether to refresh, continue with the pinned historical context, replan
or ask the user. Bound retries so continuous human editing cannot starve the
agent indefinitely. There is no implied global filesystem transaction, and
external processes can change bytes after any observation.

Lineage is never authorization. A context refresh neither expands filesystem
access nor approves an edit. Existing full-buffer snapshots, review and edit
concurrency guards remain independent of model-visible excerpts and context epochs.

### Versioning and protocol witnesses

Use a versioned capability handshake and a shared schema with generated, validated
Rust/Python representations, rather than parallel dictionaries that drift. Define
bounded frames/payloads, paginated changes, reference lifetimes, typed errors,
idempotency keys, resumable cursors and failure behavior. The existing bounded
stdio capability channel is the transport substrate; this proposal does not
require a server or direct Python filesystem access.

Compatibility tests must drive both implementations: references and digests
round-trip; modified or uncovered message content is refused; request IDs and
out-of-order events obey the negotiated protocol; restart/cursor expiry triggers
reconciliation; Unicode and CRLF preserve source coordinates; branching and
compaction preserve ancestry; retries do not duplicate delivery claims; cancelled
requests retain honest uncertainty. A complete root-to-request manifest must be
inspectable without asking an LLM to reconstruct it.

## Refresh policies to compare

Implement a policy interface over the same ledger and request builder. Record
policy/version and exact chosen material for each request. Initial candidates:

- **Append update:** preserve earlier messages and append a bounded patch or
  replacement passage, with old/new revisions and a clear notice.
- **Refresh projection:** replace selected old reads in the newly assembled
  request with current passages, and append a notice describing the refresh.
  Preserve original requests and receipts separately. Honor provider message and
  tool-call constraints; mark unsupported adapters rather than forging history.
- **Notify and reread:** announce changed observed sources and let the agent use
  tools to request the passages it needs.

Manual and automatic refresh are separate experimental dimensions from how an
update is encoded. Compare them separately. A simple manual append policy is an
initial baseline, not a conclusion about the best policy.

A refreshed source passage may invalidate earlier derived conclusions, plans,
run results and summaries. Do not refresh a read while retaining a contradictory
summary as though it were current. Preserve the old reasoning as historical and
make its input change explicit; compare reset/replan behavior where appropriate.

## Evaluation

Use repeatable tasks with controlled edits between reads and subsequent model
calls. Exercise human edits, Git pulls, unsaved buffers, changed AGENTS.md scope,
inserted lines above a read, edits outside the read, rename/deletion, conflicting
agent edits, and multiple overlapping reads. Include a changed requirement that
invalidates a previously sensible plan, not just a changed constant.

Hold task, edit schedule, context scope, model configuration and checks stable
while comparing policies. Use isolated replay workspaces and paired runs; record
repetitions rather than presenting one stochastic success as a general result.

Measure task success with behavioral witnesses; stale assumptions and attempted
stale edits; user interventions; supplied input tokens, cached/uncached usage
where reported, output tokens, tool calls, latency and actual reported cost.
Changing an early request prefix may affect prompt-cache reuse, so raw request
length alone is not sufficient. Do not infer unavailable token/cost metrics.

Capture the request manifest and the update's delivery point so a result can be
attributed to an actual exposure. External adapters may expose only notification
acceptance, not their complete model requests; report that narrower measurement.
Keep experiment records local by default. No additional telemetry is implied.

## Required behavioral witnesses before implementation is called complete

- A change to an observed passage produces the correct old/new patch and source
  coordinates; a change elsewhere does not masquerade as a changed passage.
- Two edits while an update is queued leave the unsent revision visibly pending.
- Failure or cancellation never marks an update delivered.
- Refreshing a request never alters an earlier receipt or its citations.
- Partial context cannot overwrite omitted source; a stale edit remains guarded.
- Deleted and ambiguously relocated passages are explicit, not silently guessed.
- Instruction scope and conversation branch are preserved through refresh.
- Each supported policy produces an inspectable request manifest; opaque adapter
  behavior remains an explicit gap.

The #48 follow-up adds a shared schema, generated Rust/Python types, offline
validators and cross-language contract fixtures. It does not install runtime
handlers, compare policies, or claim that live context refresh is available.
</book:entry>

</book:doc>
