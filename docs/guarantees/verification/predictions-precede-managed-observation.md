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

### Selected recorded debugger coordinates

Capture contexts now retain a distinct observation-run identity. Every prior hit
counted toward selected-hit settlement must belong to that run as well as the
same source/line/thread/function. A synthetic cross-run hit challenge and the
real configured Python witness pass. Selected publication retains only whitelisted
run/thread/frame/source/function/hit metadata from private captures; values,
evaluation errors, arbitrary context fields and problem text are excluded.
Selection checks its ledger generation again before reading that private metadata.
Public selection never records a reveal or returns current certification.

The explicit authorization evidence example now records runtime-code fingerprints
before/after its actual output/DAP runs. This is captured environment evidence,
not an immutable environment or complete OS fingerprint. Its exported predictions
are host-fixture declarations; they are not decisions by a paid model. The existing
publisher/viewer now displays that recorded pair without executing its target or
publishing actual answers, diagnostics, cell transcripts or assets. Missing
settlement evidence is refused, and source-revision navigation remains bound to
the selected index. Application strict mode and future-stop integration remain
separate outstanding work. See the active handoff for actual focused checks.

### Retained stops and shared document Verify — October 21

`RetainedDebugger` owns one host-private DAP session. Its initial seal precedes
launch and first observation. `seal_next` records the host's prior visible state
privately, its digest and the paused run/thread/frame/hit identity; the next seal
selects the immediately following hit. `resume` passes that exact binding through
`prediction/gate.rs` before Continue. An initial adapter cannot consume a future
seal. Changed context, another run or another owner refuses before dispatch.
Settled request retries return the retained summary without another Continue.
The existing capture evaluator and a bounded session time/hit budget remain in use.

An otherwise fresh successful future prediction is `informed_prediction_verified`,
never an independent first prediction. Existing correction, changed-expectation,
requirement and exposure rules still apply. Public history/selection retains only
answer-free stop coordinates and the prior-context digest; private prior visible
state and capture values remain withheld. The host must supply context actually
visible to the predictor and recognize scenario correspondence. Renaming an
already exposed question does not establish fresh evidence. The library cannot
independently establish that absence of prior exposure.

`EditSession::open_prediction` opens without weaving/importing caches. Its
host-selected policy routes Verify through the existing output/debugger/combined
gate. Generated file reads/edits/previews, general file reads and alternate tool
execution refuse under that policy. Source reads and captured source edits remain
available. A repair requires a new host-installed seal; tool arguments cannot
replace the policy. Verify returns only bounded comparisons and publishes no
runtime files, including after success. Promotion remains a separate exact-source
host action. Out-of-band source mutation refuses reads/verification/preview;
source metadata reports the approved captured hash and a current/unavailable
boolean, never the hash or content of the unapproved replacement.

LLM implementation review: **partially verified** for these seams. Inspected
`interactive_capture.rs`, `prediction/interactive.rs`, the shared gate and ledger,
and `hickory-agent/src/tools/{prediction,dispatch,preview}.rs`. The real Python
witness checks first/second-hit ordering, changed-context refusal, initial-adapter
refusal, unchanged-prediction repair and idempotent resume. Sixteen focused ledger
cases and the shared-tool real-executor witness pass. These are not attached
Python/MCP/ACP, native application, browser-worker or universal context quarantine
witnesses. Application enablement and strict coach remain unfinished. Ordinary
non-strict verification still follows its existing behavior. Human judgment is
not supplied. Exact dependency hashes and checks are retained in the handoff.

### Startup-bound authoritative HTTP service — October 21

`serve::prepare_with_prediction` accepts a trusted host binding before constructing
its router. Its existing document Run/Check handlers dispatch through the private
shared document gate, never the ordinary streaming/cache/publication path. Run
records retain typed, answer-free comparison summaries and bounded failure text.
Captured source tools reuse `EditSession`; out-of-band source writes refuse.
History is explicitly historical and grants no current certification.

The service's outer middleware refuses every ordinary API surface except the
bound Run/Check routes, its safe run-record reader, captured-source tools and
withheld history/state. Outputs/files, context/session/cache/report routes,
WebSocket streams, terminal/launch/debug and provider starts are unavailable.
There is no HTTP policy toggle or model-supplied actor. A repaired source requires
a new host seal and startup binding. This bounded host API is not an application
flag and does not automatically quarantine a separate engine/MCP process or
external tools. Those integrations remain required before product enablement.

LLM implementation review: partially verified for `serve/private_predictions.rs`,
`bootstrap.rs`, `LocalState::start_run_expected` and the outer router middleware.
A real HTTP/process witness rejects the faulty run, refuses alternate routes,
permits captured source repair, blocks stale/unsealed checking, refuses injected
source bytes and accepts the unchanged prediction after a reviewed host restart.
Failure/repair share the durable ledger; neither creates the runtime output file.
It does not witness an attached provider, native app or deployed session. Human
judgment is not supplied. The handoff retains exact dependency hashes.

## October 22 engine/context continuation review

Requirement: issue 72 and its handoff's authoritative no-copying clarification.
`PrivatePredictionStartup`, engine Attach and capability acknowledgement retain
one shared private source/Verify owner. The HTTP/MCP/stdio boundaries reuse that
owner. A durable root quarantine prevents restart from falling back to ordinary
reads or managed commands. `SourceQuestion` binds repair resealing to the existing
host selection; it neither proves semantic scenario identity nor captures the
execution environment. Only prediction/rationale can be submitted by the agent.

The explicit private Python conversation uses a fresh context store with live
adapters disabled, approved engine receipts and captured edit previews. It requires
an isolated review host; ordinary native Review is not yet admitted. Completion
is an investigation report without a changeset artifact or current certification.
Default ordinary conversations retain their source-output equality gate.

Executed checks: prediction_engine (actual daemon, stdio, confined Python worker
with scripted model/review decisions and daemon restart); two prediction_http
cases; private_snapshot_context_never_refreshes_from_files_or_editors; ordinary
CLI/window attach. These establish bounded managed interfaces, not native packaged
UI, real provider decisions, OS-wide secrecy or immutable environments. Host-only
syntax rebinding was separately witnessed by prediction_authorization source_question.
Human judgment: not supplied. LLM implementation review: these paths inspected;
not all remaining issue 72 interfaces integrated. Application flag remains absent.

Assumptions: startup admission and review host are trusted; ledger stays outside
the predictor root; managed engine authentication is not an OS sandbox. Alternatives
rejected: silently downgrading old daemons, reading ordinary live context during
private completion, materializing answers to satisfy ordinary changeset acceptance,
or pretending a model's completion certifies execution. Falsifiers: restart permits
ordinary access to a retained private root; private context reads answer-bearing
live disk/editor bytes; a stale review preview writes; a tool changes the host
requirement/actor; or private completion produces ordinary acceptance. Remaining
gaps: host end/recovery lifecycle, narrower concurrent workspace behavior, native
review/coach, ACP/browser context integration, environment/input revalidation and
promotion. No guarantee of full application quarantine follows yet.

Inspected continuation dependencies (changed bytes stale this review):

| Path | SHA-256 |
| --- | --- |
| `crates/hickory-cli/src/engine/daemon.rs` | `3c147d5db58dbfe3ba4d0763c0cba48ce1bc217e67c1a7122597528b7683d1ab` |
| `crates/hickory-cli/src/engine/private_policy.rs` | `40922db10d17f98691762f18640ac192b7c80c041651af69022c6dad60774891` |
| `crates/hickory-cli/src/engine/client.rs` | `02d51a5e101198f00d30f49ef9fd5fa2e3c2c3943b75a3715957963031c02351` |
| `crates/hickory-cli/src/python_agent.rs` | `6f961d181eae17f0755dd2cdf8959080d994546f6f2b6f2d8a3ac735181a551b` |
| `crates/hickory-cli/src/python_agent/private.rs` | `8b4ab841aae6d15c8cf046095cb6bc96bcf257a6fdbca22ee89ff3127a303575` |
| `crates/hickory-cli/src/serve/private_predictions.rs` | `5bb6d1ffa67978c7e728cfabce8a53c76e32703b3db39176e902a64d41b0b175` |
| `crates/hick-literate/src/prediction/source_question.rs` | `21b56092b5639ac8455b2032210a94f18a3e29688dab579b715f18168df2be2a` |
| `crates/hick-context-store/src/host.rs` | `d03c0330dbef16802b4ef3488bb828440151d4e54b564fc264877551690d549c` |
| `crates/hickory-cli/tests/prediction_engine.rs` | `7734deaf0b5bdac6433d0ea624eec6e6dacbb2599b12bc474ecd6655198b0ca2` |

## October 22 private coach/runtime review

The startup-bound UI branch establishes private policy before mounting ordinary
workspace/context readers. Native private Review adapts the existing mandatory
Edits queue without ordinary editor capture. Source and gate receipts have bounded
MCP structuredContent; the coach's repair completion checks original prediction,
requirement, actor and current source/host identities rather than model assertions.
Ordinary completion and generic investigation reporting retain distinct policies.

Optional PythonDebugpy profiles revalidate the captured runtime before dispatch
and settlement through the same gate. Native promotion additionally requires
host-reviewed input anchors/value identity and rechecks the actual approved source
and runtime. The API accepts an attempt, not caller-declared current identities.
This is a scoped at-request witness. It does not freeze the host, prove program
semantics, certify every dependency or prevent transient same-user mutation.
Unknown runtime evidence and changed pinned inputs remain non-passing. Actuals
stay private after promotion. Shared UI descriptions/history are metadata; viewing
them cannot certify current execution.

Executed witnesses: private browser/engine/confined-worker/real output plus DAP
fault→reviewed correction→unchanged expectation→promotion→pinned-input refusal;
configured runtime drift/unavailable challenge; native engine/MCP and early false
completion refusal; ordinary browser review/recovery/Stop; focused component and
shared-control safety cases. Scripted provider/review choices are labeled. Failed
paid-provider attempts and provider-credit rejection are retained in the handoff;
no successful new private paid/native-package witness is inferred. LLM review:
these changed paths inspected, remaining application/future-stop/parity work not
verified. Human judgment: not supplied. No normal product enable flag is exposed.

Assumptions/alternatives: host scenario/input correspondence and runtime admission
remain trusted; private managed interfaces are the assurance boundary. Reusing the
shared gate, review queue and existing coach was chosen over a new evaluator or
demo loop. Copying provider/runtime error bodies to explain setup failures,
accepting a model's missing `execution` field as a pass, relaxing stale-handle
checks, or publishing held-out files for changeset equality were rejected.
Falsifiers include accepting a non-passing repair completion, changing input
anchors then resealing/promoting, post-run runtime loss yielding a pass, raw
private values entering structured receipts/UI, or ordinary readers mounting after
a failed private-policy handshake. Uncaptured dependencies and native lifecycle/
retained-stop/external-agent coverage remain explicit gaps, not waived criteria.


### October 22 safety review and final focused continuation checks

The live private-provider workflow remains blocked by OpenRouter HTTP 402
(insufficient credit), including the last Qwen coding-model request with the
2048-token reservation. The intervening smaller-model attempt misconstructed the
nested predicate and attempted repeated premature completion; the host refused it.
Recursive predicate schema, structured tool receipts and worker instructions now
make the actual protocol explicit. These failed witnesses are not passes. No
further paid requests were made after the credit failure; a funded provider or
credit restoration is required. Human judgment and selected publication assets
remain absent.

LLM implementation review found and corrected cross-predictor seal reuse and
history disclosure: admission binds the durable seal owner, legacy mismatched
owners stay Unknown, and active predictor history omits foreign predicates and
rationales plus unrelated exposure metadata. Complete human audit remains
retained. The first actual browser run after projection failed because filtering
inserted null context fields into non-seal events. The projection now preserves
those event schemas; a dedicated assertion and the repeated browser flow pass.
Queued native Run also captures a policy epoch: even an identical-source reseal
retires the queued intent before dispatch. Failed private startup releases its
workspace lock so corrected trusted startup can retry. Human-owned bindings cannot
be used as agent capabilities.

Retained debugger owners now hold a cloned journal handle and can outlive the
startup handle. Optional configured runtime profiles revalidate before launch,
before each Continue and before settlement. Changed/unavailable runtime refuses
Continue before creating a new attempt and terminates the retained process.
This is still a scoped fingerprint observation, not immutable OS certification.

Executed focused checks: 18 ledger behavior/safety cases; the queued native Run
same-source reseal refusal; two engine tests with the installed confined worker
(including failed startup then same-root recovery), two HTTP cases; three live
retained-debugger cases including runtime loss before Continue; four standalone
debugger cases and one live runtime challenge in preceding focused runs; 48 UI
cases and targeted changed-entrypoint TypeScript. The repeated actual private
browser/provider-script/review-driver/runner/DAP flow passed in 14.4 seconds after
the history projection fix. File-length, whitespace and UI inventory generation
passed. No broad regression or post-commit monitoring was run.

These are executed checks and LLM review, not human judgment or current packaged
native/provider/release/deployed evidence. The remaining engineering list above
continues to apply; this checkpoint does not complete issue 72.

Exact implementation dependencies for this continuation (SHA-256; changes make these check results historical):

| Dependency | SHA-256 |
| --- | --- |
| `apps/agent/src/hickory_agent_worker/instructions.py` | `840b7c5500a4437dea9353ff2dc8c233d7ac1e2e7f46cb0d6e8749646b298887` |
| `apps/web/e2e/privatePredictionCoach.spec.ts` | `db08698e7cd1ed5319c6df405983e5fe3cc1662d97c550edfd1ed18405a844a6` |
| `apps/web/src/components/AgentChanges.test.tsx` | `4d105c6d4742635820f40b90b0272067f4a05f23102c682acf50dbd8a05909ad` |
| `apps/web/src/components/AgentChanges.tsx` | `936e4674f45e71e28636e8f04305386e42fee9ab7a6f64ab5b7d9f29f9e23e16` |
| `apps/web/src/components/NativeWorkflowTour.tsx` | `3ce321ec6b82200fb606e1911f015eb9fd02b1db14b78483a021c3745acffb09` |
| `apps/web/src/components/PrivatePredictionCoach.test.tsx` | `16ca0305ea294a94b831ce940f2d70f7cae5746f2d1760decb5a703f7ea37767` |
| `apps/web/src/components/PrivatePredictionCoach.tsx` | `3ecf7a6df3d552b31ec6583edb642457e3bdebb5beccdf18312dc7eaa9f33381` |
| `apps/web/src/lib/uiActionBridge.test.tsx` | `c6e12f05c5f3d96b8f9c5ca20dab241f2ad52bf1e5eddfe72c597a46fdc37971` |
| `apps/web/src/lib/uiActionBridge.tsx` | `80ad2279c40a1e3317261560e946deb967391530ca9b45437bafb651e07e7c2b` |
| `apps/web/src/main.tsx` | `800efa23eb7688bf4f76bb4303a10f56c575f45d5d25b1c760c790e5c5f96c43` |
| `crates/hick-literate/examples/authorization_evidence.rs` | `49fb5ab598cb122465a0a73a09afa2374ace5c07beec0fec279587da5fd35387` |
| `crates/hick-literate/src/prediction.rs` | `f20953d10517ff6661568a9ad1db6d30a936170d80ff8a10ab63b33814bbc09f` |
| `crates/hick-literate/src/prediction/combined.rs` | `03e1a1f125ed280d713a0a2b624622c41797fa74f1fffef3543c23f672eb68ef` |
| `crates/hick-literate/src/prediction/debugger.rs` | `2b21652dbdac17f0b67445699c881b9b1b9c70c3a75bd3ff4542c19dbe558db9` |
| `crates/hick-literate/src/prediction/debugger/tests.rs` | `aa994c6e6f393702cede1862c54ce0dc4f93b186dadd03b65095d93639eaccf5` |
| `crates/hick-literate/src/prediction/environment.rs` | `eb83a1e715b2fbda4fe104236c649464e8e43fb0e707c43293e0d585c8e7d790` |
| `crates/hick-literate/src/prediction/interactive.rs` | `4b588c988adf56f021e7b4e1327e40c80639e03f3afcb6452f7708870f569366` |
| `crates/hick-literate/src/prediction/python_identity.py` | `c78466ed1976c51baf85815843741ee52b505afa20fb0c88d70c08fb4ae2cdc3` |
| `crates/hick-literate/src/prediction/source_question.rs` | `64f834243136b489bc3ad11b74d717aad84817e855cceef25ad314405adef0bb` |
| `crates/hick-literate/tests/prediction_environment.rs` | `c27d8d2492964ee78d2959aa2bad779f699f9b10578e1bf2f0347d1644045058` |
| `crates/hick-observe/src/prediction.rs` | `6ee4b2bae923dba57d65488871554b631bb43f62e6962a849c473ff047579f1e` |
| `crates/hick-observe/src/prediction_publication.rs` | `c26f0da40d4aeb566197dbd42df5d5e3e8a41e9acb7224c1c6f3803758667dcd` |
| `crates/hick-observe/src/prediction_scope.rs` | `ef7fe0a165c27a087e4e5b027b90805accdd2b31949a8436e6bfe3c80dbed156` |
| `crates/hick-observe/src/prediction_scope_tests.rs` | `ae154fa68bda6e1b954aa98ea24a1250dc677acbb9b56c62efaa8aa553d8f2ff` |
| `crates/hick-observe/src/prediction_tests.rs` | `8c47ece5604b7f5e0b8ca294c0eae9c7ceeaac5b756c25337ea0dc6a327163ac` |
| `crates/hick-observe/src/prediction_types.rs` | `8eca89c4b79c86da4fdd58a0316611ffd34180d4514a6595051e6eb3587ca0fd` |
| `crates/hickory-agent/src/lib.rs` | `186b7498a0dfbbd355aa2b55c82ab742b9a11c461b9d331ecaf1db03ecdd7b52` |
| `crates/hickory-agent/src/provider.rs` | `900412af7b8019c463482d5320d58873c172541798337b89778329d0e6227e95` |
| `crates/hickory-agent/src/tools/prediction.rs` | `785f553cc53399995cb252b3dbf2a95803b203fee21696d703c68fc29cfefb68` |
| `crates/hickory-cli/examples/engine_client.rs` | `45c0d65cf0799b2d61770672265ff4f4fac33206f73fd8cb00749c410968ac46` |
| `crates/hickory-cli/src/engine/client_prediction_tests.rs` | `759f252e9e5fe6c178965646a125a2364d020c4da57feebcab7d667da7e668cf` |
| `crates/hickory-cli/src/engine/daemon.rs` | `a4a90970b493fd96f685c09f3454a9bb127e029355631aa79e7f17849edc9ac2` |
| `crates/hickory-cli/src/engine/mcp.rs` | `4d79ff7dbf735fe95c33e7c7011d9636a3fca7c3038d84d94dc4f3e3107fa434` |
| `crates/hickory-cli/src/python_agent.rs` | `2d45ce0ac85a2efa7fb87bef4869c53d9fabee4d5ab066520b2ffdbed8e2a52a` |
| `crates/hickory-cli/src/python_agent/host.rs` | `ff508325fa4682acc710736acfdba959653e8af47e2519f09aa41f8155d79a51` |
| `crates/hickory-cli/src/python_agent/private.rs` | `13072740bea99a198ebc2d840700f3f602f17648e31d8bf8cc70b8e0ec00915e` |
| `crates/hickory-cli/src/serve/agent.rs` | `48a0ee253d961df7d3af258138b0696890ade045ab452ac6c8f3ab4107b07fcf` |
| `crates/hickory-cli/src/serve/mod.rs` | `e16e672fbd076a90b136bc4e5c689af93d2120947b798e55638b36dcf52d0e0b` |
| `crates/hickory-cli/src/serve/private_prediction_agent.rs` | `78494700ff7bd5f4071f304ae4fcb1c61f4c31bf92ea7817deac55f59ab1ae86` |
| `crates/hickory-cli/src/serve/private_prediction_startup.rs` | `a34c8ce68ee360b5c9e4154aaa574cb56192f1c5ab4bbfcafbc487c2c4192302` |
| `crates/hickory-cli/src/serve/private_prediction_tests.rs` | `07053a9e7dba435167328935538ece716948b045fcbd0ce55353dfab0b69068d` |
| `crates/hickory-cli/src/serve/private_predictions.rs` | `254c7c034a7bd4c7a64d3e0652a267d6edb1b056b02e11915d2e6a0f92ce73ed` |
| `crates/hickory-cli/src/serve/python_agent.rs` | `d627b9c264bb9a86ec7c18cc2e8ee19afc99b6267fe4ff65b3ced1cdc95f2bb8` |
| `crates/hickory-cli/tests/prediction_engine.rs` | `de89b1cd118a886689d7ff659acad3e57e99da17fcc5381bb125064852650cf1` |
| `crates/hickory-cli/tests/prediction_http.rs` | `13d8227621c1a39b352c0c4891077dc6fbd97d930a1f229af098f2eb51d1cd5d` |
| `examples/ai-engineer-private-deadline.md` | `ae61814fa3292518be29ea788df39d2195aa2eb5de6d9673301c43024d230b3c` |
| `scripts/test-private-prediction-ui.sh` | `e9f50afd5f6339605fa50c5e0e7f78e6653e6b91bba36196f6d8883f7e1e870e` |


### October 22 private restart and End lifecycle

Continuation after commit `940554a5`: trusted recovery metadata now lives under
engine private state, outside the predictor root. It retains the host question,
source-selection anchor, original actor/inputs/runtime/executor and last
controller-approved source hash. Ordinary opening of a known private root restores
that private binding before weaving. It selects only this actor's exact matching
seal; an approved repair without a new seal reopens unsealed and cannot execute
the old prediction. Changed disk bytes, inputs, capture selection, runtime or
executor refuse recovery; no execution-permission fallback is allowed. Interrupted
attempts remain historical and no execution is triggered by reopening. A crash
between source persistence and recovery-receipt persistence can require trusted
host recovery; arbitrary disk contents are never adopted as approval. Existing
root-only quarantine records without recovery metadata still refuse ordinary
fallback. Explicit stale startup profiles may still require a fresh host opening;
they are not silently acknowledged as a different profile.

The existing coach now has a human End command through the same action registry.
Agent invocation returns requires_input. The host refuses End while an
investigation, verification or source operation is busy, or another live client
holds the practice. Successful End retires execution/edit/seal admission and that
client identity, releases its workspace lock/global context quarantine, and keeps
the root's private admission and complete ledger. The connection stops its
heartbeat/reconnect loop. A fresh opening restores private policy and starts a
fresh context; the ended client cannot reconnect. Ordinary commands in a retained
private root remain refused after End. This controls known managed roots and
contexts; it does not confine arbitrary external shells or provider internals.

An actual browser challenge initially found that retiring the client too early
prevented retaining the End action result. The retired router now permits only
immutable action receipts and window cleanup through its normal remaining lease;
all other private readers/tools refuse. The repeated focused browser flow passes
in 14.9s, including End, with no page errors. This does not claim native-package
footage. Focused engine tests with the installed worker (2), HTTP tests (2), queued
Run/busy End/retired-context safety test (1), private coach cases (5), targeted
TypeScript including Vite/browser declarations, file-length, whitespace and
inventory generation pass. These are executed checks plus LLM implementation
review, not human judgment. Broader regression/post-commit monitoring remains with
the user.

Native creation of a new private practice, in-product retained future-stop
controllers, remaining universal UI/external-agent parity and the real
native/provider/released/deployed witnesses remain open. The provider-credit
blocker does not account for that independent engineering work. Issue 72 remains
incomplete.

Recovery/lifecycle exact dependency hashes (SHA-256; earlier evidence remains historical):

| Dependency | SHA-256 |
| --- | --- |
| `apps/web/e2e/privatePredictionCoach.spec.ts` | `46087f5b29f22ec328897400311506b727857c857a94b50056bb43eed881eba7` |
| `apps/web/src/components/PrivatePredictionCoach.test.tsx` | `eeb8ceca842c2efd6e065b11eb9b64f5388b23f2c51875e95f8d39b5cdce8526` |
| `apps/web/src/components/PrivatePredictionCoach.tsx` | `4f2ad0fcde9e8f463b9e7a2d9bf9069a9e2b6d85b7c797ced46020f48751615a` |
| `crates/hick-literate/src/prediction/source_question.rs` | `06462887b0739637fdf473b77ff22b82364829f55653c24956c6e1e61b97b81a` |
| `crates/hickory-cli/src/engine/client.rs` | `5f2f12e36b1bf88e3b9081448c733f65465a0bcb3f67897879c913299487baa1` |
| `crates/hickory-cli/src/engine/daemon.rs` | `e93d26fa02d51ad10fee9ceb9f8ca1705f22dedf7aaba204a3a56ff8d85c7eb3` |
| `crates/hickory-cli/src/engine/mod.rs` | `55c842e4ea9cb4cecae7ece75ed4037ffaa7c068e345367a0f9fc77f6a348a27` |
| `crates/hickory-cli/src/engine/private_policy.rs` | `179bc1522be5a8a61d9c0712a6eb8d15f0116e869133bd5f5bae57a47c6e4ae5` |
| `crates/hickory-cli/src/engine/proxy.rs` | `26ccae4b3955e63fbadbcca28aec3838941a73de228ca3d765d6f4e61b42590c` |
| `crates/hickory-cli/src/serve/mod.rs` | `e3c700d3a1a1defbe441eac6c2773fded0a85d2874e6309d4100d0466b75b999` |
| `crates/hickory-cli/src/serve/private_prediction_agent.rs` | `9bf2a29677465668213c28b4bbb95a70ba315b83e0e59293760f2365a45f79d4` |
| `crates/hickory-cli/src/serve/private_prediction_recovery.rs` | `5fe2036c1f569afea80f1c500b0a3a212d4a4c56706563058ea1c009c88a2194` |
| `crates/hickory-cli/src/serve/private_prediction_tests.rs` | `d910069a45ce748c2c5b5fe1f70c6a3d3b6f3316edf17859b4ffcc132639ecd1` |
| `crates/hickory-cli/src/serve/private_predictions.rs` | `63f2090d9cf9de0ab73e5af83b7e1dfb3899b8018e3bf98540f6d044587a5dc2` |
| `crates/hickory-cli/tests/prediction_engine.rs` | `726efb6d2530277623fc387244e9108f448dfc61fc083565e31f029155abb58c` |


### October 22 retained-stop controls and native private creation

The existing native tour now offers explicit human creation of a separate private
Python practice. It seeds a Git repository, independently captures the configured
runtime and fixed inputs, and installs the trusted engine binding/recovery record
before invoking the existing native open-window callback. No prediction, target
execution or provider request is started by creation. The original project is not
edited. The current supported private DAP profile requires an explicitly selected
local executor and an already available Python/debugpy environment; unsupported
executor/platform/environment setup returns an agent-addressable requires_input
receipt. Creation does not select weaker execution permissions or install packages.
An agent cannot create a fresh practice to reset its own history. A failed native
open ends the temporary private context while retaining private-root admission, so
it does not leave an invisible active context. Fresh fixture Git commits disable
hooks; this is confined to the newly seeded repository.

The trusted startup may carry a two-stop plan: initial and next semantic question
families are fixed separately from the combined output question, with the same
host inputs. The coach, built-in Python and external MCP expose
seal_debugger_prediction and continue_debugger_prediction only when that plan is
present. Initial sealing precedes launch; future sealing binds the actual paused
run/thread/frame/hit before Continue. It reuses RetainedDebugger, PrivateCaptureSession
and prediction/gate.rs. The combined-source extension first validates the existing
combined source/volume constraints; it grants no arbitrary-volume/evaluator path.
Output verification remains its separate authoritative output plus scratch-DAP
combined run. Completion and promotion also require current passing original
initial/future questions; a phase named complete is not a passing verdict.

The future seal privately retains the full captured source, host questions/target,
prior comparison and this actor's selected ledger history at that seal. Its digest
and stop coordinates are public; the retained private state is not. The confined
supervisor separately retains request/delivery/session context. External-client
and provider-internal context remain unestablished: the host-state checkpoint is
not a proof that every outside prompt, human memory or provider state was captured.
The existing bounded private-context size may refuse a very large history rather
than truncate it into a claim of completeness. No held-out target values are
returned. Source edits terminate the old paused process. A scoped timer expires
an abandoned paused owner after two minutes and cannot retire a newer sequence;
End and orderly daemon shutdown also terminate retained owners. Abrupt OS process
termination is not an independently witnessed process-tree cleanup guarantee.

LLM safety review found two additional alternate paths and corrected them:
ordinary-window APIs and already-open debug sockets now pause/close while private
policy is active. Admission drains previously admitted requests/debug operations
and refuses startup if ordinary jobs, terminals or agent connections remain.
UI action receipt/owner cleanup remains available, including creation's final
receipt. Busy admission failure retains the private root/recovery identity but
releases its active global contribution. Private evidence storage is durably
protected alongside the source root, including after End. Ordinary workspace,
command, Python-context and worker-marker MCP paths cannot open it. Older End
records recover that store identity from trusted recovery metadata; missing or
corrupt identities fail closed. A private ledger cannot be placed inside an
already ordinary workspace. This is managed-interface/root admission, not OS
confinement of arbitrary external shells or opaque provider internals.

Executed focused evidence: the actual browser/engine/confined worker/provider-
script/review-driver/output/DAP flow passed with six retained attempts: initial and
future plus combined checks on the faulty revision, then the same expectations
on the repaired revision. It proves future-seal-before-Continue ordering, retained
fault/correction, missing-seal/stale-source refusal, promotion, reload, fixed-input
refusal and End. The latest completed run before the final client-schema change
passed in 32.1s; the following focused rerun is recorded below. Three engine tests
with the real confined worker and two HTTP tests passed, including actual trusted
creation/open-callback/fresh-root attachment, original-source preservation,
ordinary-socket closure, cross-window Run/terminal refusal, and ended evidence-store
MCP refusal. Thirteen coach/component cases passed, including mixed agent seal /
human Continue and human-only creation/setup. Two creation/permission/expiry cases
passed with the real adapter. The timer callback was challenged directly on an
actual paused process, including a stale owner; no two-minute wall-clock test is
claimed. The ordinary browser review/recovery/Stop witness passed in 17.2s before
the final root-guard change; its focused rerun is recorded below. The existing
ordinary practice-seed and queued native Run safety tests passed.

Native desktop cargo check --locked initially refused its stale generated lock.
Offline Cargo resolution added 22 lines for already present local dependency edges
and hick-observe; no third-party version changed. Native compilation then passed,
and the repeated --locked check passed. This is compilation, not a packaged/native
UI/provider/installer witness. No package, upload or release was created.

Remaining issue-72 work still includes the full per-control/dynamic/platform parity
audit and any uncovered adapters; isolated private ACP integration (currently
refused, with no unsafe fallback); reviewed selection/recording and current packaged
native/provider/released/deployed witnesses. The private paid-provider witness
remains blocked by OpenRouter credit (HTTP 402). No further paid requests were made.
Human judgment is not supplied. No issue was closed, external issue message sent,
broad regression run or detached post-commit status monitored.


Final focused results for this slice: the private six-attempt browser flow passed
in 31.3s after the client response-schema guard. The ordinary browser rerun then
failed on an unhandled retired-command result: Escape could unmount AgentPicker
while its persisted human preference intent was pending. FeatureSettings now waits
for pending human form receipts before dismissal, excludes its own visibility
request to avoid self-wait, and binds late dismissal to the current owner. Picker
failures are visible and owner-scoped. The registry still refuses removed/stale
controls; no error is filtered out of the browser test. A deterministic delayed
preference test confirms the actual change executes before Escape closes it. The
actual ordinary browser flow then passed in 16.7s, with no page errors, and retained
its private witness at /tmp/hickory-python-ui.m1coLC (not a publication selection).
Thirty focused registry/bridge/settings cases passed before the extra own-command
wait challenge; its result is recorded with the final check below. These changes
are also a shared UI-parity correction, not a private-only demo workaround.


The final settings/registry/bridge challenge passed 31 focused cases, including
visibility dismissal that excludes its own pending request and waits for the
actual child preference. Targeted TypeScript passed. The native locked compile
passed with the generated dependency-only lock update. No broad regression or
post-commit monitoring was performed. Ending retires the private host context;
normal-mode unrestricted programs afterwards are not an OS confidentiality
boundary for same-user ledger bytes. Native private contexts cannot resume as
ordinary contexts, and ordinary managed root/file/context admission continues to
refuse retained private source and evidence roots. External client/provider state
cannot be independently reset or authenticated by those host receipts.

Retained-controller/native-creation/quarantine exact dependencies (SHA-256):

| Dependency | SHA-256 |
| --- | --- |
| `apps/desktop/src-tauri/Cargo.lock` | `f7f09fa56912d208b44719e8ba02beb5497c467155fa8698b9ac4906491a6a07` |
| `apps/web/e2e/privatePredictionCoach.spec.ts` | `6c056c326efa565a3f4e907b9ca92ce029f91fa56ca5e91f345a957d6b7eae9a` |
| `apps/web/src/components/AgentPicker.tsx` | `e4f94c64b590f3d9ef0e18fa56ecacd6d49ab1d3e934a1afea5a8fd8cdc2b4b1` |
| `apps/web/src/components/ChatDock.tsx` | `7b0157b7762e1b5ea9bf420cf414bcb5fdd5eb7024a431a675029649e84d6477` |
| `apps/web/src/components/FeatureSettings.test.tsx` | `423c4559f739f0d09f8a5f2220503a9901ef721df4dd7479a4430d205639cc44` |
| `apps/web/src/components/FeatureSettings.tsx` | `c2851465b5e59b692d135a3c522ccaeaf57c3e74d138434cb6c63db9f321173f` |
| `apps/web/src/components/NativeWorkflowTour.test.tsx` | `0d00d15ca8a926002e8ecebd4ebc492e99a0946b0471f4bd931be1867c18ea01` |
| `apps/web/src/components/NativeWorkflowTour.tsx` | `61c70dd5f62b0517c2a63b1d1d16072271ecbf38658db98414f7d3a758a9c646` |
| `apps/web/src/components/PrivatePredictionCoach.test.tsx` | `227ee0c06fbde299d711853b4575b355a0c006f7a0f5901b028d0f03bb0ff6d3` |
| `apps/web/src/components/PrivatePredictionCoach.tsx` | `5f114ddbae10915075e45cf78b1c6b0b1e7688b38643466a742c34795cd5a0b3` |
| `apps/web/src/lib/uiActionBridge.tsx` | `203c89ebf36a4de6ef502cad14a032cbcf5ae589ed2aee8abb69d4d342b772a1` |
| `apps/web/src/lib/uiActions.ts` | `2d6cab9feaf0efc115f61935655846145715dbe43ffb8fc98f653e1c77d8070c` |
| `crates/hick-literate/src/prediction.rs` | `f2c713f85ed453272c5f73aa7d77bef0925d7055e32c680c0de997c33002d3e1` |
| `crates/hick-literate/src/prediction/debugger/tests.rs` | `63ac4c51344bf2819b37aef53a5ec0012c00ab5fe3284786df94cfd9277b1511` |
| `crates/hick-literate/src/prediction/interactive.rs` | `883693670d15ce8942b8dcb03e48f823aa11c463bb154d3861cfdca660ec50c0` |
| `crates/hickory-cli/src/engine/client.rs` | `eaf53259815eab567a92785a7d45f5cc317ce84f034412f8df50d0faf5882eca` |
| `crates/hickory-cli/src/engine/client_prediction_tests.rs` | `6f84d1aed0854bdb88b9e60114bbf2c2dc2b9844d0f68ecc714526ec54f7441e` |
| `crates/hickory-cli/src/engine/daemon.rs` | `6e9e162deab3d6931993b4ec78f84ad9e10935d506c0e2f2368141c73b641e76` |
| `crates/hickory-cli/src/engine/mcp.rs` | `97ac1bc8c47f69b45ce0c919b9be4a4e8a41559f6ff6432ba97007fa3b598b32` |
| `crates/hickory-cli/src/engine/mod.rs` | `2167ecd65a1ec55bff58c78a15cbd4072c7cb7059a2bc15fda37db3e68e04d7a` |
| `crates/hickory-cli/src/engine/private_policy.rs` | `7f0751bf38a599571654d54a9f5caeb374589066aab732124ab7c7dcb63bd5ad` |
| `crates/hickory-cli/src/mcp_stdio.rs` | `bacef15638853c66f7e11dd3571973229cf9b0134a7f1f6ad5403d9baf3a1336` |
| `crates/hickory-cli/src/python_agent.rs` | `b53f8aa9505330c40d67937492c7e2781ffdbc5417248ec189826f4b19d086fa` |
| `crates/hickory-cli/src/python_agent/host.rs` | `a5bd64c5a32fdbc85789f959625ba139a8fc1c6aa4ec21db9c4cb9b66f688d6d` |
| `crates/hickory-cli/src/python_agent/private.rs` | `8c4aa161c9a547b8f65d0054f6b4de0f7564c76bc579934c519de822cd529cda` |
| `crates/hickory-cli/src/serve/acp/mod.rs` | `9c6639977a46cb1c9e22328b4bb064e92929868fcb778a555705954e321ea4a3` |
| `crates/hickory-cli/src/serve/agent.rs` | `39b5f089f95194dd08a5107068a7b193f27f7a0affe29dc93abc31ac618b1f46` |
| `crates/hickory-cli/src/serve/engineer_tour.rs` | `2ad6be952ea53716c7375366cd73b5c52da011c1284f88ca7dac0a04fc3d7fbc` |
| `crates/hickory-cli/src/serve/mod.rs` | `e250885c05f981227f73c63b709c968c5e5eff780878aeae5c6b3d80fa8fdd33` |
| `crates/hickory-cli/src/serve/private_practice.rs` | `c8a0186132c1573bf4369774152825f61d605c87b2a8934768a764b7a8a9131b` |
| `crates/hickory-cli/src/serve/private_prediction_debugger.rs` | `c39cfb2f2b4de1e1636eca8098dee37d7710bb6089bb1a27872c7b513137936d` |
| `crates/hickory-cli/src/serve/private_prediction_recovery.rs` | `ea4e77588e033124fd06029efa150606c46061fc2d87533a41d6adf6af4c0fc2` |
| `crates/hickory-cli/src/serve/private_prediction_startup.rs` | `7ad28b3e9fd0bde75e64510c5d0aeabb8bb41231f5b8c0d1fa9b6e71b41bde4f` |
| `crates/hickory-cli/src/serve/private_prediction_tests.rs` | `d67915d68d7c5eeb2c267a498ac9067bd1915a33381c77ba8a5360e44ea48b15` |
| `crates/hickory-cli/src/serve/private_predictions.rs` | `365648e436385d1e2e5252d53ecfeb21ae0dcdb1e056c51936d5fb902def9506` |
| `crates/hickory-cli/src/serve/socket.rs` | `baeb385c0ed99b0df63f66400783b6921b2ce32497761fa5ca51be192a996b5d` |
| `crates/hickory-cli/tests/prediction_engine.rs` | `4b6c0bd9fdf585212b4a6304f912bb1e782a144aa756962094209f02c2328f64` |
| `crates/hickory-cli/tests/prediction_http.rs` | `d057a803d0962c9c9b1424c6b024243e68e8c14fa771e5b9167c9a1357b4da6e` |
| `scripts/test-private-prediction-ui.sh` | `2669b251f44ecfcb3badbe71a2863c11903c7c89b67ee4bda9e7c172b4b03710` |

The two focused ChatDock settings/registered-action cases also passed (27 unrelated cases skipped); the final changed-entrypoint TypeScript check passed.


### ACP refusal at internal admission — 2026-10-22

Ordinary ACP process construction and MCP Host startup independently refuse private
LocalState before executable lookup, capture/recording and executor setup. Global
private policy also pauses ordinary ACP startup. The existing HTTP quarantine
remains separate. This is refusal, not isolated ACP support or confinement of
external provider/harness internals. A prepared-private session safety test now
challenges direct Client and Host entry; it passes without creating its recorder.
The ordinary desktop-proxy ACP protocol flow also passes against rebuilt local
binaries. LLM implementation review is partial; human judgment is not supplied.
Exact dependency hashes, scope and remaining adapter work are in the issue-72
handoff. No new paid-provider or packaged-native witness is inferred.


### Separate managed private ACP path — 2026-10-22

The later adapter supersedes blanket private ACP refusal only for an explicitly
configured separate private argument profile. Ordinary ACP remains refused.
Private MCP uses the existing source/seal/Verify/retained-stop gate and mandatory
fresh exact-edit review; old source approval refuses, human replacements apply,
and cancelled/ended contexts cannot start another tool. A fresh conversation and
home are supplied, no ordinary history is hydrated, and client filesystem,
terminal, permission and extension requests refuse. Only an advertised configured
authentication method is requested. Raw adapter notifications/diagnostics are not
verification and are not returned to the coach. Actual source execution and
promotion retain the same independent host checks and answer withholding.

Consequential assumption: the external executable and explicit private argument
profile are trusted configuration and do not import ordinary history. Hickory's
managed boundary does not sandbox an external harness's internal OS access or
provider context. There is no private ACP default or claim of generic confinement.
The fresh managed context alone cannot prove independent model understanding.

LLM review is partial. Three focused Rust cases, eleven frontend cases, targeted
TypeScript, a scripted actual ACP process/MCP/browser six-attempt repair flow and
the existing Python equivalent passed. The actual pinned Codex adapter initialized
v1/HTTP MCP, but new-session creation required authentication in the deliberately
credential-free probe. No paid/model request or human judgment was supplied.
Falsifiers include stale approval overwriting newer source, ordinary client file
access succeeding, unverified completion passing, or raw target diagnostics in an
ACP-visible result. Exact dependency hashes, failures and fixture paths are in
the handoff's separate managed private ACP checkpoint. Later source changes stale
that evidence; native/released/deployed and real authenticated-provider acceptance
remain distinct required witnesses.

## 2026-10-22: native window controls and post-End close

Requirement: issue 72/#74 requires actual shared OS controls or an explicit
tracked OS limitation, and #77 retains a human-owned End/quarantine boundary.
The native Window menu and UI registry now invoke the same current-window
minimize/restore controller. Explicit status reads return host-observed platform
and minimized state only. No caller selects another window, file or arbitrary IPC.
Browser/missing callbacks return a required-input result; uncertain host failures
are not successful closure/minimization claims. Predefined macOS About, Services,
Hide, Hide Others and Show All remain a tracked #74 platform limitation, with an
addressable operation request and retained pending result. Human conversation
reports are not authenticated OS completion. macOS execution is unavailable on
this Linux host and must be qualified separately.

Review discovered that the private page bypasses App's close listener. It now
shares a close controller between the native event, ended-page button and registry.
An agent cannot end/close its own quarantine. Human close before End is refused.
After durable End, local host close bypasses the retired project router/receipt
lease; it does not reopen project capabilities. The human button/event call the
same controller directly because persisting an action receipt through an expired
project lease would prevent closing. An agent-visible close request still returns
requires_input. Host acknowledgment does not prove window destruction.

LLM implementation review: partial, covering the finite callbacks, origin/body
validation, ended-only local close and retired-router independence. Assumption:
the current native host owns the one main window; SDK state is a point-in-time
report and OS scheduling can change it immediately. Falsifiers include a foreign
origin closing a window, active quarantine closing, arbitrary target parameters,
a late receipt preventing human close, or a held-out field entering window output.
Executed checks: 12 focused frontend cases (window responses, OS requests, private
coach), 7 existing bridge cases, one real local proxy/Connection lifecycle test;
CLI and native desktop cargo check and targeted strict TypeScript passed.
The first frontend invocation used the repository root and lacked jsdom; rerunning
from apps/web passed. The first standalone TypeScript command omitted dom.iterable;
including that configured DOM library passed. No broad regression was run.
Human judgment: not supplied. These checks do not establish paid-provider,
reviewed footage/publication, deployed or other-platform acceptance.

Dependencies (SHA-256; edits stale this review):
- `apps/web/src/lib/nativeWindowActions.ts`: `8d673546ea02e309bb03bcb68bb6e3c0a62cfe8a16e23e72913c4d156872c9b5`.
- `apps/web/src/lib/nativeWindowActions.test.ts`: `d6e41286eb25d2013435967ae489774e769ea68621317c04a00d55de79c6468c`.
- `apps/web/src/components/PrivatePredictionCoach.tsx`: `f52c09d0da7e16b44509e166fcb18b54e3bd3fd86950714e0fbf97c14036a71a`.
- `apps/web/src/components/PrivatePredictionCoach.test.tsx`: `f49321378846ac4355ec43e1e1f0556b82af284a4966f5d40899858ae240ab37`.
- `apps/web/src/lib/uiActionBridge.tsx`: `ad5ddfecc981c32de3f284e466ca34c1bc48645c23519d20311f9bb16873e561`.
- `crates/hickory-cli/src/engine/client.rs`: `bc4caa99206d1fc157ddfdbf5a4917007ed3b957bd9488914081bf48afbd2b7b`.
- `crates/hickory-cli/src/engine/proxy.rs`: `46cb41135332a9b6cdabedde7d4b60aab39c3730950bd804912c950a45608e23`.
- `crates/hickory-cli/src/engine/client_prediction_tests.rs`: `01aaaef8922155f615732939a69e11db40f5468b368d87ea246791997d653158`.
- `crates/hickory-cli/src/serve/shell.rs`: `2f3be29957046bc545826b92892ade11077c5e59f924b40dbceebfd8871b9eed`.
- `apps/desktop/src-tauri/src/lib.rs`: `d8ce0cd736db0f367e9560e797e46efdb265b954333a694c7425868243e6ad73`.
- `apps/desktop/src-tauri/src/server.rs`: `1b82b61fea2c1f12cc417e823881ccfe19fa91fcd1dde9c4f9b78cf41b2abb6b`.

### Executed ACP continuation after fixture correction

The actual engine/worker/MCP/DAP/browser private ACP flow passed (18.3s case,
19.0s reported total). Six initial/future/combined faulty/corrected attempts,
review, unchanged-expectation promotion, changed-input refusal and End ran with
scripted provider responses. `/tmp/hickory-private-ui.zL2PZf` and
`/tmp/hickory-issue72-window-private-acp-retry.log` retain private diagnostics.
This is not real authenticated-provider or human-review/publication evidence.
The package being built is functional code checkpoint `49a2a491`; no source
changes follow it, only focused safety tests/documentation.

### Close failure recovery correction

The post-End native-close failure witness initially failed: polling cleanup had
set its owner flag false, suppressing the visible close error and busy cleanup.
A separate mounted close-controller owner now outlives polling and is withdrawn
only at unmount. The focused challenge verifies actual failure visibility, no
raw host-error disclosure, no automatic replay, restored button availability
and removal of the native listener at unmount. Thirteen focused frontend cases
and targeted strict TypeScript passed after the fix. The 49a2a491 package build
precedes this final recovery fix and cannot attest it. The scripted ACP witness
above also precedes this recovery-only change; its execution/gate paths are
unchanged. LLM review remains partial; human judgment remains absent.

Current recovery dependencies (SHA-256):
- `apps/web/src/components/PrivatePredictionCoach.tsx`: `3c289c11702eb623a8fc169202f844fd667130995f5eb60401999ca8d527e9d9`.
- `apps/web/src/components/PrivatePredictionCoach.test.tsx`: `ecef641a955243bea19b7262cffdc7231390c0143a55384358dfe2abe1e023de`.
