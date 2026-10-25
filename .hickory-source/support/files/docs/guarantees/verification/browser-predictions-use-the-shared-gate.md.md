<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/verification/browser-predictions-use-the-shared-gate.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `7148277696fce44c65265f3d7775608873c5bb1dc1157cc3919788fbcd40c20a`.

<book:entry path="docs/guarantees/verification/browser-predictions-use-the-shared-gate.md">
# Browser-local prediction practice uses the shared gate

The optional browser branch of the existing workflow coach uses
`hick-observe::prediction::Ledger` compiled into the shipped parser/ledger WASM.
It does not implement a second comparison or promotion engine. The portable
journal uses the same validation, seal, admission, original requirement,
classification, interruption, actor-scoped history and promotion methods as the
native private ledger. Native private-file ledgers cannot export raw portable
snapshots through this API.

The browser host stores a bounded private journal under one fixed local-storage
key. A same-origin Web Lock serializes cooperating journal writers. Persistence
must succeed before an admitted attempt dispatches the existing interpreter.
Corrupt/oversized history is refused rather than silently reset. Retry IDs retain
their dispatch decision across snapshot restoration. Worker termination precedes
interruption recording; recovery requires the exclusive cross-tab runtime lease,
retains Unknown/interrupted evidence and never replays an old request.

Inputs are bounded whole-millisecond durations and a budget. The source contains
the same recorded faulty/corrected implementation as the existing coach. Source
bytes are frozen for execution and checked against the actual runtime source
hash. Worker-entry and mechanism artifact hashes are captured at sealing and
checked before admission and after execution. These are point-in-time checks,
not a frozen browser/import/dependency closure. That closure and prior exposure
are not established: browser seals set identity_known=false, native promotion is
unavailable, and every model-facing adapter response forces current_certification=false.
An independent short-attempt and empty-sequence check cannot be waived by a
matching output prediction. The coach cannot advance the correction by replacing
its original prediction: the shared ledger supplies an answer-free original
question/expectation/owner comparison even when Unknown masks other classifications.

Entering practice replaces the other homepage demos and monotonically confines
this managed UI registry for its lifetime. Ordinary descriptors, old handles,
cached results and results completing after confinement cannot deliver their
payloads. Hiding/restarting guidance cannot release the restriction. Explicitly
scoped source/metadata readers remain bounded and contain no actual values.
The source pane uses the same current generated source as sealing/execution.
Reversible human draft edits call the same field controller immediately; agent
edits retain dispatcher intents/results. Neither draft editing nor UI progress
admits execution. Seal/admission and private result records are authoritative.
Only validated question inputs restore from the draft cache; foreign predictions,
rationales and actuals are never auto-restored into model-readable drafts.

Raw output, errors, debugger events and portable snapshots stay inside the browser
host adapter. They are not added to ordinary runtime history or UI readings/tool
results. Human inspection persists its exposure before writing a separate window,
removes its opener, and returns only delivery metadata. An agent receives a
required-input result and no report URL. Changed owners/epochs cannot adopt late
seals or results. A human publication request never auto-uploads a session.

## Assurance boundary and assumptions

Requirement: #72/#74/#75/#77 shared handlers and withheld answers; #76 optional
input-changing browser practice without install/API keys. The existing native
private flow remains the supported retained-DAP/initial/future-stop path. This
browser branch checks output and independent cases; it is not a native/provider
witness, an authenticated history service, an OS sandbox or a freshness claim.
The same-origin runtime lease is an independent cooperative-host witness: all
managed practice workers hold it until settlement, and page destruction ends its
workers/releases its lease. Failure to establish that lease refuses recovery.

Browser storage, extensions, devtools, equivalent-origin parents and unrestricted
external harnesses are outside this typed broker boundary. Earlier public
observations cannot be unlearned; Unknown never becomes pristine through a reload,
renaming, copied inputs, deleted local storage or a new UI window. User-controlled
hash chains detect accidental corruption; they are not signatures. Manual human
copying of inspection data into predictor input is not authorized by inspection.
A matching bounded check is not semantic proof or general bug prevention.

Alternatives: duplicating ledger logic in TypeScript risks divergent gates;
reusing ordinary output/debug readers would leak selected answers; treating local
storage as authenticated or complete runtime identity would invent certification.
Falsifiers include leaked output/diagnostics/snapshot bytes, a cached ordinary
result delivered after confinement, changed-expectation advancement, native-file
snapshot export, replay during recovery, dispatch after failed persistence or an
asset change, and late-owner adoption after a source changes away and back.

## Verification evidence — October 22

LLM implementation review: partial, focused on the above transitions, source/
owner binding, actor projection and alternate managed UI paths. It does not
certify every rendering, platform, provider, import closure or adversarial host.

Executed checks: 21 focused ledger cases (including portable restore/deduplication,
corruption/rollback, native export refusal and original-expectation feedback under
Unknown); one WASM-adapter recovery case; 38 focused frontend cases; five real
Chromium flows including the unchanged legacy homepage flow, real faulty/corrected
custom-input comparisons, original-expectation blocking, corrupt history, separate
human inspection and cross-tab recovery refusal. Targeted strict TypeScript and
touched Rust formatting passed. All four generated WASM artifacts exactly matched
a focused rebuild with the repository's registry-path remapping.

The new human-flow challenge first caught stale per-keystroke receipt revisions;
immediate shared draft controllers fixed it. The first recovery-success check
raced browser cleanup after closing a lease-owning page; it now waits for an
independent read-only lease probe before requesting recovery. A rejected ordinary
handle was retired (unavailable), not merely forbidden; the fixture now asserts
that actual owner state. Synthetic component/driver behavior is distinguished
from actual interpreter execution. No human judgment is supplied.

The actual engine/worker/ACP/MCP/DAP/browser scripted six-attempt flow passed
(18.4s case, 19.0s total; private diagnostics /tmp/hickory-private-ui.y7O832).
That flow precedes the later answer-free original-expectation helper; subsequent
ledger/browser checks cover that helper. A current capped real Qwen provider
attempt failed for insufficient credit before reaching review (4.6s case), retained
at /tmp/hickory-private-ui.1vEhWL. It is not a pass. A read-only OpenRouter key
check returned HTTP 200 but could not establish account funding; no key material
or account balance was printed. The normal configured key was used only for its
intended service, with no credential-file contents exposed to the model.

The 7416bacf unpacked native package passed scoped window setup/minimize/restore/
active-close refusal/End/post-End closure with its engine stopped. It predates the
long-source CSS repair and this browser work. Current native packaging/visual
qualification is recorded separately after the matching build. Selected reviewed
footage, a supported release and matching deployed evidence remain absent.
No broad regression or post-commit status monitoring ran. Human judgment: absent.

Dependencies (SHA-256; changes stale this review):
- `Cargo.lock`: `66313c886b1bf756c34f51f6592fdd1e8be88c6f21710cf856ed2e63aa286036`.
- `crates/hick-observe/Cargo.toml`: `72eaece8cbde99befbe339c95d57c13bbe3d5516d4b2ba89a99bd9fb77fed286`.
- `crates/hick-observe/src/lib.rs`: `52cc09d7a9ff1b66071d08c9d362d9764d1eb04fb5f99dd890a00c1a8c5217b3`.
- `crates/hick-observe/src/prediction.rs`: `2082dee2693b62dc27444a084d665bdebc971a2e95a00e47be2d8c987ab22c09`.
- `crates/hick-observe/src/prediction_tests.rs`: `41ec74131af7aab89ecf16d28119518add88d0b8412850c89332a9a96b8159a8`.
- `crates/hick-lang-wasm/Cargo.toml`: `794a94dc0239d352cb4d87dc1f986a59b3df2d93b0dce4782f176604c44bf3db`.
- `crates/hick-lang-wasm/src/lib.rs`: `969265ce60c611813a0302aee809e07d52beaea305c8501331b96fc3fe396326`.
- `crates/hick-lang-wasm/src/prediction.rs`: `bc7b3f2a859481bfbce4d407f6f09649c451d1542c1f2c8bf87a5974853ae79a`.
- `apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm`: `d2b0f21e57d4ddcedd0bb1319028d9d05af92c5c3e0da29913db7e641ea6639d`.
- `apps/web/src/editor/generated/hick-lang/hick_lang.js`: `fbb83326740499ad0d15389faa655e261fb90728ba0c1476cc222b113834b573`.
- `apps/web/src/editor/hickLang.ts`: `2921e22374ee79aa59db038f3e87d77156634b632650f16b43b7a7755f4293c7`.
- `apps/web/src/debug/browser/transport.ts`: `5c3c0883c479cb16985826eca337aa313b8557437906d6d02cab1f8a10c4b2c2`.
- `apps/web/src/tour/browserPrediction.ts`: `99ede00a7604cdcfeffdcd06081c04c9b88d9333b5e529c672364b3867964a33`.
- `apps/web/src/tour/browserPrediction.test.ts`: `39e1af5efaf490de16b38e56fbe45f6d59a686428e7c7c2908e027f99cff89a5`.
- `apps/web/src/components/BrowserPredictionPractice.tsx`: `3b638868edfdbdbb40149ce0c5626c5cbe151f93a61004960297db4769f19c55`.
- `apps/web/src/components/BrowserPredictionPractice.test.tsx`: `c233a6ce449d886e336c364e177b668002614fcc6a9af56e9e0a563aea924432`.
- `apps/web/src/components/WorkflowTour.tsx`: `bcbfdf0e57d7f60afd0b032f1bb27482f7e17086c88e916a886cb749baa55b4d`.
- `apps/web/src/views/LandingView.tsx`: `863ae5ff99fb129f66ba01271709e201a30b708e47af4405aa34f1dd28d5091c`.
- `apps/web/src/lib/uiActions.ts`: `d7978dbcaa7318cf257610e1d4dba3e845b9f81d782fd8bf6abf4877b3aaecfc`.
- `apps/web/src/lib/uiActions.test.ts`: `c23b5912982c06c2651036ba7ddace210e370184f6657f6222fb6167ed34894f`.
- `apps/web/src/lib/uiReading.ts`: `0106a04c688187a2f26b674fae8bbb4bf98922c0b30202e3d7ead40d74d50363`.
- `apps/web/e2e/staticUiParity.spec.ts`: `2d7af96b6b46d06358e2721561afc42ea4a7256d1f886195324afc3eab294b2f`.
- `apps/web/src/editor/generated/hick-lang/hick_lang.d.ts`: `afd96435fdcf71c7238398b8f606c55c077f3aa6032ae183d16abff3514733df`.
- `apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm.d.ts`: `db50d427f3aa46d96c660d762d989732841cd76838ce21e8db6cd1a1997854d7`.
- `apps/web/src/embed/uiHostSession.ts`: `e7011b60d2e64b931391984bc7770fe0a1bc67f98b6cff5d52010c43ab8c9711`.
- `apps/web/src/embed/StaticUiSurface.tsx`: `ad17f0f8466c792dafe7883d1670c960fb954255bf6f99e73d48ec97e20d3a31`.
- `apps/web/src/debug/browser/runtime.ts`: `edf2e1041bb908e55a7efcb7138cdcefe6a06b706b543b1cf994a368d642c80c`.

Package-build challenge: c522b1b0 compiled the native CLI, then its frontend
production compilation rejected an unused type import and unused inspection
parameter. Both declarations were removed without changing the inspection
transition. That failed build produced no accepted current package.

Matching native evidence: the 530df638 local .deb built successfully and passed
two owned-window setup/lifecycle witnesses, including minimize/restore, active
close refusal, End and actual closure with the engine stopped. A source-pane
screenshot and native scrollbar geometry confirm the scoped long-line containment
repair. See the final matching-package section of the issue-72 handoff for exact
artifact/evidence hashes. No provider/target execution or human judgment occurred;
this is not a released/deployed or authentic-provider workflow witness.
</book:entry>

</book:doc>
