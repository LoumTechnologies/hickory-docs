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

## Verification and integration status — 2026-10-21

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
