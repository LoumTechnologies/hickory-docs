<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `7b3b8f50635d96f9bfb28e6fa4cf9d354914be05e894d4ba4670e969982455d9`.

<book:entry path="docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md">
# Published executable results identify reviewed inputs

Given the explicit publication selection,
when CI generates public verification artifacts, then it copies only selected
source/dependencies/assets whose SHA-256 values match their recorded review.
It runs the existing native weave/test commands in an isolated temporary input
tree with an isolated home and no inherited provider credentials. It preserves
failed command details and still generates the static index and snapshot.

The browser uses the existing document and cell renderer without a local backend.
It shows the tested revision, source/dependency identities, exact commands and
results, requirement-to-witness links, absent witnesses and the declared scope.
Command execution, document recording/replay evidence and unavailable origin
metadata are distinct. Historical results cannot attest changed source or prove
arbitrary prose. Opening the viewer executes no native checks or model calls.

Assets require explicit selection and a matching hash. External assets and
unselected/private/session inputs are refused. Remote provider context is not
published. Output publication is explicitly approved for each selected fixture;
source/dependency changes require renewed input review.

---

Last LLM verification:

- Date: 2026-10-19
- Reviewer: Codex
- Result: partially verified pending deployed CI/browser witness.
- Originating requirement: #61/#47, extending the manual snapshot viewer.
- Evidence: `scripts/publish-spec-results.py`, `publications/selected.json`,
  `PublishedSnapshot.tsx`, `snapshot.tsx`, CLI `block_model_json`, CI/site workflows.
- Assumptions: only reviewed deterministic public fixtures are selected. The
  local executor runs trusted selected code; this is not a hosted execution API.
  A content hash or hosted result is not an authenticated semantic attestation.
- Exclusions: generic per-cell cache-hit origin is unavailable; retained document
  replay identities and recording keys are projected when the native CLI provides
  them. Browser-only snapshots cannot discover the latest private source revision.
- Witnesses: native deadline fixture weave/test passed with its actual transcript;
  publisher tests retain failure diagnostics and reject private/escaped inputs.
  Focused browser witness checks failed results, revision/witness navigation,
  missing evidence and source tampering without a backend.
- Challenges/falsifier: an unselected source uploaded, changed approved bytes
  silently published, absent witnesses labeled passed, a viewer triggering native
  execution, or historical success shown as verification of changed source.
- Dependencies: existing CLI runner, CellPanel/DocumentEmbed/PublishedSnapshot and
  static Pages deployment. Changes to source selection, capture or rendering
  invalidate this review.
- Executed checks: two focused Python publisher tests passed; native deadline
  publication passed. Chromium offline-publication witness, snapshot validation unit test and
  TypeScript compilation passed. New CI deployment was not monitored.
- Human judgment: no new public material beyond the explicit selected fixture;
  no live deployment acceptance recorded in this session.

Issue 72 reviewed recording extension (2026-10-19):

- Optional demonstration metadata names a separately selected, hash-reviewed
  .mp4/.webm, exact captured build revision, declared cuts and optional selected
  public session export. Raw sessions remain excluded by the existing publisher.
- WorkflowRecording loads only on request, correlates index/snapshot identity,
  and verifies selected media bytes before playback. Missing footage, failed
  verification and differing capture/test revisions remain visible.
- Asset resolution rejects absolute/scheme/query/fragment paths and encoded
  traversal before fetching. Changed media fails SHA-256 comparison before any
  object URL is created. Retrying clears prior video/publication state.
- Executed checks: three publisher tests, three recording-loader tests and the
  selected-byte/tampered-byte/encoded-escape asset test passed. Test media is
  synthetic test bytes, not native/provider footage or playable-video evidence.
- No video has been selected or published; native/provider/deployed playback
  remains unverified. No deployment or regression status was monitored.

## Selected answer-withheld prediction reports

A separately selected prediction export must match its reviewed SHA-256. The
publisher accepts the ledger's historical verdicts without re-comparing private
actual values or granting promotion. It does not execute the held-out target for
that projection. Cell transcripts, command diagnostics, recording payloads and
assets are excluded. Private capture values/errors are excluded by the host
exporter; only whitelisted observed debugger coordinates may be selected.

The authorization negative fixture explicitly expects failure while retaining a
visible failed result. That declaration needs a reason and cannot waive missing,
stale or unavailable source evidence. The repaired source has its own exact hash.
Both share stable scenario/requirement/input identities and preserve failure next
to repair. Source-revision navigation uses the existing index identity checks.

The browser validates ordered seal/admission/settlement references and refuses
comparison summaries whose settlement is missing. Supplied verdicts remain
historical host reports, not authenticated semantic truth or current certification.
Absent debugger metadata and unavailable/ambiguous revision navigation stay
explicit. No public report provides execution authority.

Executed focused checks: six publisher cases, eight snapshot/history/controller
cases, real Chrome backend-unavailable paired navigation/missing-evidence refusal,
and local full selected publication passed. The pair comes from actual host
process/DAP execution with synthetic fixture predictions; it is not a provider
or packaged/deployed witness. No private QA footage was selected. Human judgment
and a new deployed witness have not been supplied. Broad regressions and their
monitoring remain with the user.
</book:entry>

</book:doc>
