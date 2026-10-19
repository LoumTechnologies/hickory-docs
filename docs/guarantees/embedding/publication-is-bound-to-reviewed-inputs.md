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
