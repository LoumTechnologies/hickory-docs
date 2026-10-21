# Prediction history preserves order and distinguishes observation from verification

The host prediction ledger in `hick-observe::prediction` accepts a sealed runtime
prediction separately from its requirement. A managed attempt can be admitted
only against the seal's exact source, input, dependency and environment identities.
An output target names its execution; a debugger target binds an expression,
exact captured-source byte span and positive hit number. A result for another
source, expression, location or hit cannot settle that attempt.

Scenario identity uses the host's semantic family and concrete inputs, rather
than a filename, cell position, source revision or session. Renaming/copying the
same admitted scenario or repairing its code therefore does not erase its
history. Different concrete inputs have separate history. Host adapters must
establish the family/correspondence; unknown identity or unavailable earlier
exposure stays Unknown. The model cannot establish that admission by assertion.

Private results are absent from normal summaries. Agents and host tool responses
cannot request the held-out answer, even through an explicit reveal action.
Human-only inspection appends and syncs an actor/surface exposure event before
returning actual values or diagnostics. Host adapters must isolate those human
reports from agent context, cached transcripts, published artifacts accessible to
the predictor, and debugger/tool responses. Recording an accidental or pre-existing
exposure does not authorize delivery. Revealed target values cannot subsequently
serve as independent evidence of the predictor's understanding.
Previously revealed states are retained as the context of later seals, including
whether the same predictor received them or another actor saw them. Human views
and agent deliveries remain separate events. Salted event commitments preserve
ordering without publishing an enumerable unsalted hash of a small secret value.
They are not signatures or authenticated evidence against a same-user attacker.

Matching a runtime prediction of 160 does not satisfy a requirement of <=100.
Promotion checks both predicates, current identities and provenance. A failed,
stale, replayed, unknown or observed-adjustment attempt cannot be promoted.
Changing the requirement never silently replaces the original requirement.
Changing the predicted value also blocks promotion when no explicit exposure was
recorded: repeated guesses or copied expectations cannot earn verification credit.
Debug investigation must withhold the selected target's answer and indirect
answer-bearing diagnostics from the predictor. Already exposed targets require
honest contaminated provenance; assess understanding on fresh held-out targets
rather than treating a copied expectation as verification.
Corrected source passing the original prediction retains the failed attempt and
is classified separately from first-attempt success. Repeated attempts without
established code correction also have a separate classification.

Atomic journal replacement under a stable filesystem lock serializes cooperating
writers. Stale generation checks refuse conflicting seals. Exact execution
request retries reuse the attempt; altered actors/arguments cannot reuse its ID.
An unfinished attempt survives restart and blocks another execution of that
scenario until the authoritative executor records interruption. Interruption
means Unknown, never success; late results cannot overwrite it. Corrupt history
is refused rather than replaced with empty history. Historical promotion and
current certification are distinct; current certification requires supplied,
matching current identities.

## Historical checkpoint before continuation — 2026-10-21

Origin: #77 under #72, including the user's debugger-backed prediction
clarification. Reuse `hick:capture`, existing debugger evaluation and the existing
runner when connecting the ledger. The ledger itself executes no program and
does not generate tests.

LLM implementation review: **partially verified**. The state machine and durable
store are implemented. **No application strict mode is enabled by this change.**
Authoritative execution/promotion, capture/debugger, cached-output, context
delivery, native UI, Python/MCP/ACP, browser-tour and publication integrations
remain #77/#75/#63/#61/#76 work. A passing ledger test cannot establish those
interfaces are guarded. Ordinary run behavior remains unchanged.

Executed focused checks: `cargo test -p hick-observe prediction::tests --quiet`
passed eleven tests. They cover missing/stale seals, duplicate/concurrent requests,
private mismatch diagnostics, agent/host reveal refusal, human reveal ordering,
replacement-expectation refusal without explicit exposure, correct prediction
of an unacceptable bug, source correction, observed expectation/requirement
adjustment, independent novel inputs, prior exposure, replay, corruption,
debugger source/expression/location/hit matching, interruption/restart and
independent concurrent filesystem-lock holders. These supply synthetic host
results; no native execution, provider reasoning, actual capture adapter or
cross-platform filesystem witness is established by these tests.

Assumptions: a host adapter chooses the private durable directory outside
session/cache eviction, supplies trusted actor identity and independently
captured source/dependency/environment/results, and recognizes semantic scenario
families. It must use Unknown when correspondence cannot be established. The
ledger cannot establish an absence of unrestricted external shell/provider
observations, recover a deleted external state directory, authenticate a human,
or prevent a permitted same-user process from editing its files.

Falsifiers include actual values returned before a committed observation,
source/hit drift settling a seal, adjusted expectations becoming pristine,
correctly predicting a bug satisfying the original requirement, a retry repeating
execution, or session reset erasing attempts. Human judgment: **not supplied**.
Broad regressions and post-commit monitoring were not run.

Evidence dependencies (SHA-256; changed bytes stale this review):

- `crates/hick-observe/src/prediction.rs`: `2563fdf2f1c65c41ab503ada664445d1a4f8e00e70c36e0d4632bddf878655ce`
- `crates/hick-observe/src/prediction_tests.rs`: `101f3a54eff12f77bdd88c2e805673f2079a94ca9042b5e3c242e294fe5e60a0`

## Bounded live adapters — October 21 continuation

`hick-literate::prediction::run_output` now admits one standalone exec through
the same `run_pipeline_live` implementation. Admission checks the exact source
bytes and selected output/decoding target before execution. The adapter accepts
no caller hooks/cache configuration and returns only a `ManagedOutput` summary.
Output files, stdout/stderr, failed expectation details and transcripts never
become that response, including after a successful comparison. They cannot be
published by a caller treating the response as a normal `PipelineResult`.
Private diagnostic mode removes answer-bearing expectation log messages.
Unsupported tags, captures, projects, volumes and frozen cells refuse admission;
these paths do not fall back to an ordinary run. Execution/decode failures produce
Unknown and cannot promote. Existing hick:expect failure is atomically retained
as ExecutionRejected, even if both the sealed prediction and requirement match.

`Ledger::admit_target` atomically returns one dispatch owner per durable request
ID. Concurrent/restarted retries return the same attempt with dispatch=false;
they cannot repeat execution. A dropped execution future remains unresolved
until the host confirms its executor stopped and records interruption. Transport,
execution or evidence failure records Unknown without releasing the dispatch
claim: an error alone does not prove a remote job/debuggee stopped. Only a late
valid result or explicit authoritative stop confirmation can settle that claim. Admission
binds the target before dispatch as well as at result intake. Settled independent
execution-check results cannot be rewritten. Existing journals without the new
execution-check field retain their original serialization/commitments and default
to their historical accepted-check semantics.

`hick-literate::prediction::debugger::run` uses existing hick:capture parsing,
Mapping and DAP watch evaluation for an initial Python selected-hit prediction.
It requires a sealed root-source line byte span, expression, positive bounded
hit and expected top-frame function. The actual thread/frame, generated source
coordinates and confirmed breakpoint state are retained privately with that
hit's evaluation. A moved/unbound/missing hit, mixed thread sequence, wrong
function/source/line, missing frame, evaluation error or unsupported value format
cannot pass. Private capture ends the debuggee at the selected bound. This does
not certify subsequent execution or final output. Only JSON-rendered numeric and
other JSON values are supported; arbitrary language representations are not
coerced. Watch context remains an adapter hint, not proof against expression
side effects.

`capture::run_private` suppresses build log delivery. Private session startup
suppresses adapter process stderr and TCP process stdout and suppresses raw
adapter-message diagnostics. Attached/multiplexed adapters refuse private
startup because their other output channels are outside this admission's control.
Normal capture/start behavior remains available separately.

LLM implementation review: **partially verified**. Real LocalExecutor output and
configured debugpy witnesses passed: faulty total-budget output fails; a source
repair passes the unchanged prediction; short and empty inputs pass separately;
the faulty second-hit allowance fails a prediction of 20, and the repaired
allowance passes that unchanged prediction. The output and debugger witnesses use
separate documents/scenario questions, **not one combined application evidence
chain**. Fixtures declare host input/environment identities and recognized
families; they do not independently fingerprint an immutable toolchain or prove
previous exposure absent. They establish actual execution/evaluation and the
tested binding/gate behavior, not full host admission correctness.

The host must provide a private, fresh executor, inaccessible durable ledger,
trusted actors, independently captured inputs/environment and scenario identity.
The supplied executor still owns all execution permissions. A scratch directory
does not sandbox LocalExecutor or DAP launch, isolate adapter-internal logging,
or stop a permitted same-user program from reading files or making external
observations. No application enable flag is exposed. Ordinary CLI/server runs,
UI, Python/MCP/ACP, cached/context delivery, future-stop interactive predictions,
coach, selected publication, and packaged/provider/deployed evidence still need
integration before application strict mode can be claimed.

Falsifiers include two retry dispatch owners, promoting an independent runner
check failure, inherited private adapter diagnostics, wrong-frame/hit settlement,
or returning a raw actual/error/file payload to the predictor. The focused commands,
observed results and current dependency hashes are in the
[continuation handoff](../../developers/handoffs/2026-10-21-issue-72.md).
Human judgment: **not supplied**. No broad regressions or post-commit status
monitoring were performed.


## Unified gate and same-revision evidence

`prediction/gate.rs` now owns admission, idempotent retry feedback, unresolved
Unknown handling, private result settlement and exact single-cell output intake
for output-only, debugger-only and combined adapters. The adapters supply actual
observations and existing runner-check verdicts; the same ledger decides
prediction/requirement matching and promotion eligibility. They do not implement
separate acceptance rules. Ordinary verified-publication policy remains separate
because it deliberately preserves ordinary diagnostics/default behavior.

`prediction::combined::run` seals one Combined target and retains one observation
with output and debugger fields from the **same document revision**. Fields
predicates require every named field, reject missing/extra fields and have bounded
size/depth. The original requirement can independently constrain total output and
second-hit allowance. One passing field cannot promote a partially passing result.
The live pipeline's private capture sink reuses the existing exec + scratch DAP
capture sequence; it delivers capture evidence privately instead of appending a
held-out table to stdout or an executor transcript. Ordinary capture behavior
remains unchanged with no private sink.

The supported combined adapter admits a single literal Python file in an empty
private input volume and a fixed command invoking that file, so an unrelated
stdout program cannot stand in for the captured program. External volume inputs,
extra executables/files, changed source/targets and unsupported paths refuse
before execution. This remains two executions of one captured source, matching
the existing capture architecture, not simultaneous output and debugger evidence
from one process.

Executed focused witnesses: the real combined exec/debugpy test retains the
original faulty attempt, rejects a final-output-only cap while the allowance
remains wrong, and accepts the remaining-budget repair with both original
predictions unchanged. Two combined tests, fourteen ledger tests, four output
adapter tests and two debugger tests passed; CLI compilation passed. The privacy
assertion checks JSON values/keys rather than UUID substrings, because a random
opaque ID can coincidentally contain a short numeric answer. Full commands,
remaining application integration and current hashes are in the continuation
handoff. No application strict-mode flag, native/provider or deployed witness is
established by this consolidation. Human judgment remains not supplied.


## Continuation review correction and selected history

On October 21, a fresh review found that the combined adapter passed its outer
Combined target directly to the debugger-only validator. Those bytes could not
reach execution, despite the preceding checkpoint's passing-witness claim.
That claim is historical and does not verify the reviewed pre-fix worktree.
The combined adapter now validates an explicit debugger-arm request, retaining
the original Combined target for the one authoritative ledger admission.
A non-ignored missing-seal test establishes that supported combined sources
reach that shared gate without requiring an installed adapter. The real
fault/output-only-fix/repair witness also passed after this correction.

Selected-hit intake now validates every counted prior hit's positive frame and
raw generated source/line against the selected hit. A prior hit with another
source, line or missing frame cannot contribute to the requested hit count.
The focused binding challenge covers those three counterexamples; the real
configured Python fault/repair witness passes with this stronger intake.

`Ledger::select_history` selects whole scenario histories from one consistent
public-history snapshot. It retains failed and repaired attempts, original
global sequence/hash references, sealed requirements/predictions, identities,
actors, exposure events and existing classifications. It declares omitted event,
attempt and prior-context counts; unselected scenario context is removed. It
never returns private actual values or raw runtime/debugger evidence, records no
reveal and grants no upload capability. Its summaries are historical, without
current-source certification. Partial hash references are commitments, not an
independently authenticated or publicly verifiable full journal.

The selection test witnesses retained failure plus repair, omitted unrelated
private context, round-trip serialization, unchanged journal generation, missing
selection refusal and continued agent-reveal refusal. This establishes a host
serialization seam, not integration with the selected static snapshot viewer or
isolation of a human-answer publication from agent access. Those integrations
remain required before application strict mode is available.

`prediction_authorization` additionally runs the synthetic vulnerable and patched
sources through the same combined adapter and stable scenario/requirement/input
identity. Both retain legitimate-own-record and missing-record assertions.
The vulnerable side fails output/debugger acceptance; the patch passes unchanged
predictions and preserves the failure. A separate witness correctly predicts the
vulnerability and is still refused promotion. These are actual host-process and
configured-DAP witnesses, not selected publication assets or a security audit.
The [example instructions](../../../examples/security-authorization/README.md)
state prerequisites and scope.

LLM implementation review: **partially verified** for these host-library seams;
authoritative application isolation, future-stop prediction and selected browser
publication remain incomplete. Executed checks and exact reviewed hashes are
recorded in the continuation handoff. Human judgment: **not supplied**.
