# From meeting evidence to a reviewed, executable requirement

Status: initial implementation added October 18, 2026. Anchored local review
threads, pluggable line attribution and author colors, the shared browser/app
tour, and viewer snapshot export/import are implemented. Transcript-import UI,
audio recording/transcription, an automated CI report index, automatic agent
investigation dispatch, and a tour that creates a native example repository
remain future work. The tour currently uses an isolated browser execution surface
in both environments and offers the same spec for local execution.
Origin: the user's October 18, 2026 request for provenance comments, authorship
coloring, meeting capture, executable requirements, published results, and a
shared homepage/application tour supporting an AI engineering demonstration.

## The workflow

One small example should connect the whole product: a meeting establishes a
total request deadline, an agent implements a per-attempt timeout, and the
engineer challenges the mismatch. A regression check exposes the mistake;
the agent fixes it; the engineer reviews the diff and resolves the challenge.
The processed requirement and its execution results are then published.

The requirement is a total deadline across retries. Use an injected clock and
deterministic transport to test elapsed-budget decisions without flaky sleeps.
Name the remaining boundary: that test does not establish actual network or
operating-system timing guarantees. Include a real integration witness where
the claimed requirement needs one.

## Existing foundations inspected

- `apps/web/src/shell/Ribbons.tsx`: lineage, context and declared links, drawn
  as ribbons or braces; click navigation already exists.
- `apps/web/src/components/AssumptionAudit.tsx`: user declarations and corrections
  through `/api/sessions/audit`; a separate form rather than anchored discussion.
- `crates/hickory-agent/src/audit.rs`: declarations, dependencies and superseding
  corrections; original entries remain available.
- `apps/web/src/elements/session/view.tsx`: agent answers, source chips and
  historical evidence passages.
- `crates/hickory-cli/src/ingest.rs` and `crates/hick-transcript/src/lib.rs`:
  deterministic transcript ingestion, content-based deduplication, VTT/SRT/SBV
  and line-based text parsing. Ingestion does not itself call a model. Audio
  and video are explicitly refused as non-transcripts.
- `hick:exec`, `hick:expect`, `hick test`: execution and expectation checks.
  Passing these establishes the specified checks, not arbitrary prose truth.
- `apps/web/src/editor/blameGutter.ts`: optional Git attribution. Git author
  identity and uncommitted state do not establish human or agent authorship.
- `apps/web/src/views/LandingView.tsx`: current homepage demonstrates debugging.
  The homepage and local app have separate entry points in the same web package.
- `.github/workflows/deploy-site.yml`: filtered pushes to master publish the
  static site to Cloudflare Pages. On inspection, the latest successful run
  reported the same SHA as local HEAD, `cfca02ef1f455f5a62f80b61cb59db2000e38ffc`.
  This is workflow evidence; the live site's delivered bundle was not compared.

## Review on relationships and claims

Offer Comment / challenge from a relationship, source chip, or selected agent
passage. Preserve normal click navigation and provide keyboard access.
The panel names whether the target is the statement, the source, or the
relationship between them. Store original quoted text, content hashes or
revision, session/turn where available, and both endpoints for relationships.
Rendering keys and line numbers alone are insufficient durable anchors.

New text creates a new revision of the target. Preserve the original; explicitly
show changed, unavailable and ambiguous targets. Never silently move a comment
to merely similar text. Reuse the audit infrastructure where appropriate,
adding structured targets and review events rather than encoding them in prose.
Non-session document comments must also work; session storage alone is not enough.

Posting and asking the agent to investigate are distinct actions. An agent
investigation includes the objection and recorded evidence, returns a proposed
change and checks, and links those results back to the thread. States are Open,
Agent responded, and Resolved by user. Model agreement or a passing test never
implies human resolution. Questioning source support does not erase the fact
that the source was supplied as context.

## Authorship display

User clarification: use Git blame by default in Git repositories, behind a
pluggable attribution-provider interface so Git is optional. The display
consumes normalized span/line attribution with identity, revision, provider,
and evidence kind. A Git provider uses the existing blame endpoint; other
providers can supply editor/session receipts or another version-control system.
Missing providers or history yield Unknown without disabling authoring.
Git attribution answers who is credited with a change. Classifying that change
as human or agent requires explicit metadata or recorded session evidence;
agent edits committed under a human identity must not be silently classified
as human-written. Keep the Git author visible alongside any origin evidence.

Keep language syntax colors. An optional Authorship view adds subtle background
marks or a margin stripe, labels, and a legend. Support Recorded human input,
Recorded agent write, Edited since recorded write, Imported, and Unknown.
Only label what the available capture actually establishes. In particular,
absence of an agent receipt is not evidence of human authorship; an imported
transcript is not a freshly human-authored passage; Git commit authorship is
separate from text origin. Human and agent edits can share a line, so use spans
where recorded precision permits it and disclose coarser attribution.

Marks must invalidate or become historical when the captured bytes change.
Missing sessions leave an explicit unknown state. Color is supplementary to
labels and must remain distinguishable with the provenance overlay enabled.

## Meetings

First make existing transcript import discoverable in the app. Preserve the
original transcript, timestamps, speaker labels where supplied, source hash and
links from extracted requirements back to particular utterances. Generated
summaries remain agent-authored declarations. Corrections preserve the machine
transcript rather than overwriting the evidence.

Add direct recording as a separate capture path feeding the same importer:
visible start/stop and recording state, save location, recoverable audio,
transcription progress/error/retry, and explicit selection of local or remote
transcription. Do not silently upload recordings. Speaker identification must
remain unknown unless supplied or confirmed. Audio retention and large-file
handling need a choice before committing binary recordings to Git; text and
an audio content hash can live in the repo with the audio stored separately.

## Executable requirements and published runs

The executable-spec engine already exists: requirement prose, implementation,
and behavioral tests can live in one literate document using `hick:exec` and
`hick:expect`, rerun with `hick test`. Do not build a second spec runner. The
new work is discoverability, requirement/check navigation and visualization,
report publication, and the shared tour. Tests still need to be written; the
system cannot derive adequate checks from arbitrary prose automatically.

A requirement document contains the requirement, scope, source references,
implementation reference, independent behavioral witnesses, counterexamples,
and executable checks. Link each requirement to the checks that exercise it.
Changing implementation behavior should make an appropriate witness fail;
changing the expectation to accept the regression must remain reviewable.

Publish an immutable run report keyed to the exact commit, with processed
documents, requirement-to-check links, command/environment, timestamps, and
per-check outcome. Keep Passed, Failed, Not run, Unavailable and Stale distinct.
Record whether checks actually executed or were verified from recordings.
Show historical success beside a changed revision rather than presenting it
as current success. Failed runs must still expose their reports.

Start with CI-produced static artifacts and a browser index; remote execution
and a hosted service are not prerequisites. Sanitize rendered content and
define which documents/output may be published: local session records are
ordinarily private and gitignored. Publishing a report must not expose them
implicitly. This is new reporting work, not a claim that `hick test` currently
provides such a dashboard.

## One tour, two execution environments

Share the scenario, fixture documents, steps and explanatory copy. Adapt actions
to each environment instead of maintaining two narratives.

1. Read a meeting excerpt and open the utterance behind a requirement.
2. Inspect human input and an agent-authored implementation.
3. Follow a declared citation; see why context is not semantic support.
4. Challenge the total-deadline claim on its brace or selected sentence.
5. Inspect a failing counterexample, then the proposed correction and diff.
6. Run the requirement checks and review their scope.
7. Resolve the thread and open the processed document with its run result.

The homepage uses browser-executable checks where feasible and explicitly labels
pre-recorded agent responses and desktop-only steps. It must not present replay
as a live model call, host process execution or actual microphone capture.
The application tour uses an isolated example repo, real edits and actual
execution. Provide restart, skip, resume and an easy way to reopen the tour.
Advance execution steps on observed success, not merely on clicking Next.
Do not automatically manipulate the user's current project.

Expose the deployed source revision so homepage freshness is inspectable.
Keep shared example and tour dependencies in the site's deployment trigger.

## Delivery order and acceptance

1. Anchored challenges and honest authorship display. Verify relationship and
   sentence targeting, changed/missing targets, mixed authorship and user resolution.
2. One executable deadline-requirement fixture with a demonstrably failing old
   implementation and passing corrected implementation; connect it to the review.
3. Shared tour using that fixture in the homepage and an isolated app project.
   Verify keyboard access, restart, and accurate live/replayed labels.
4. Immutable processed-document reports and a CI index, including failed and
   stale results. Verify that a previous green run cannot certify new bytes.
5. Transcript-import UI, then audio recording/transcription through that path.
   Verify import deduplication, preserved originals and transcription failures.

Implementation validation: review persistence, source changes, attribution-provider
replacement, attribution invalidation and tour gating have unit/HTTP coverage.
The tour and standalone snapshot viewer passed browser acceptance in Chromium,
Firefox and WebKit with local APIs blocked. See the corresponding guarantee files
for precise boundaries. Meeting recording and live deployment contents remain
unverified; this change does not add a transcription provider.


## Initial implementation verification

- Frontend: all 2,230 tests passed (247 files); subsequent targeted review,
  embedding, tour and conversation checks also passed (50 tests).
- Native: 2 review-store tests, 8 blame/review/snapshot HTTP tests, and the
  deadline requirement engine test passed. The engine test runs the actual
  generated JavaScript and rejects the original implementation.
- Browser: the tour, published viewer and non-Git attribution/review flow passed
  in Chromium, Firefox and WebKit (9 checks). Existing homepage debugger
  acceptance also passed in those three engines.
- TypeScript typecheck and production site builds passed.
- Visual inspection covered the tour, published failure report, and review panel
  with distinct human and agent origin stripes.
- Small fixes to the reused table renderer removed a native tooltip and raw
  theme fallback colors so the existing frontend invariant checks pass.

Review threads are accessed through **Comment on selection** or by right-clicking
an actual connection (Alt+Enter when keyboard-focused). **Lineage settings →
Color credited authors** enables attribution. **Export viewer snapshot** captures
the focused document for `snapshot.html`; hosting the JSON alongside that viewer
allows `snapshot.html?src=your-file.json`. Export does not upload or execute.
