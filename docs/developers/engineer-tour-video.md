# Capture the agent-first native workflow

Audience: engineers recording a supported native build with a real configured
provider for issue 72. Retain the exact build revision and platform. Install
Node.js and the local execution dependencies before starting. Release asset
existence does not establish the workflow works.

1. Open **AI engineer tour** and explicitly create an isolated practice repository.
   Show its new window/path and the transparent seeded meeting/proposal labels.
   Demonstrate the anchored coach’s Locate, dismissal and reopening with normal
   keyboard focus.
2. Copy the investigation task into the normal workspace conversation. Show the
   provider and explicitly Send. Let the agent inspect/setup the environment,
   reproduce the failure, bind a breakpoint and inspect a real paused stop through
   disclosed UI actions. Show source/run/stop/request identities. Fix actual
   errors; never substitute an example response.
3. Interpret two 80ms attempts consuming 160ms on the seeded source. The human
   selects the missed qualification and creates a real review thread requiring
   one total deadline including retries. Show the criterion in normal controls.
4. Copy and Send the correction task. Independently review and approve or replace
   the actual preview. Let the agent weave as necessary, invoke Verify saved
   document and inspect its receipt. Match the current root hash and retain the
   short-attempt and empty-sequence checks. Budget allocation does not establish
   real network or OS timing.
5. Review the diff/evidence and resolve the real thread yourself. Close/reopen the
   coach to demonstrate reconciliation. Skipped or missing witnesses must remain
   unestablished; saved progress cannot supply evidence.
6. Load reviewed publication JSON and compare source identity and tested revision.
   Preserve stale/failed results. Publication, local execution and deployment are
   separate facts.

Optionally profile hotspot.md through Runtime observations before and after a
reviewed optimization. Show separate source/run IDs, call counts, timing scope and
profiler overhead. Do not promise a fixed speedup.

## Record the optional private prediction workflow

Use **Create a private prediction practice** in the native coach for the strict
version. The earlier ordinary-workspace instructions do not enable strict policy.
Choose the local executor and install the configured Python/debugpy runtime first;
creation reports a setup request if either is unavailable. Keep the ordinary
workspace, the new private window and their identities visible in the recording.

1. Inspect the private request, source and explicit withholding notice. Choose a
   configured provider/model (or a separately configured private ACP profile),
   then explicitly Delegate. Let the agent investigate and seal the output and
   initial debugger predicates before execution. Do not enter held-out answers
   in its prompt or copy human-only evidence back into its context.
2. Show the retained non-passing attempt and blocked promotion. If demonstrating
   the future stop, seal the next predicate before Continue. A prior stop informs
   this prediction; it is not a pristine first-run prediction. Predictor-visible
   diagnostics withhold target values, including stdout/errors and variables.
3. Review the proposed remaining-budget correction in the actual private review
   controls. Human approval must be real and separately attributed. Let the agent
   revalidate unchanged predicates against the corrected source. Retain the
   original failure, corrected source identity and requirement/prediction results;
   changing an expectation cannot promote the repair.
4. Inspect selected evidence as a human through its separate evidence path.
   Include the 100ms requirement and faulty 160ms outcome in public footage only
   after selection/review; never deliver those human-only actuals to the predictor.
   Keep raw sessions, credentials and private roots out of the public assets.
5. End explicitly. Show retained history/private-root status. Then use Close window
   or the native close control. Active native close requests require End first;
   an agent cannot end/close its own quarantine. The post-End close path works
   independently of the retired project router. If a host cannot close, its
   explicit OS input request remains unresolved rather than claiming success.

A local scripted acceptance run is available without paid provider calls:

```sh
HICKORY_KEEP_UI_WITNESS=1 scripts/test-private-prediction-ui.sh --acp-fixture
```

This exercises the actual engine, ACP/MCP, debugger and browser with scripted
provider responses and test-driver review. Its retained footage is diagnostic,
not human-reviewed native/provider footage. A real direct DeepSeek Python flow
passed at 47353e3c with retained failure, reviewed correction, unchanged expectations
and promotion revalidation. Its review decisions were automated test-driver
decisions. The earlier OpenRouter HTTP 402 concerns a separate route; the real
Codex ACP custom-provider session setup succeeds, but the stock DeepSeek
Responses route has tool-routing/approval compatibility failures. See the current
[acceptance checklist](issue-72-acceptance-2026-10-22.md). For a bounded paid witness, provide
the configured DeepSeek credential to the host environment and run:

```sh
HICKORY_KEEP_UI_WITNESS=1 HICKORY_PRIVATE_PROVIDER=deepseek HICKORY_PRIVATE_MODEL=deepseek-chat scripts/test-private-prediction-ui.sh --live-provider
```

The script refreshes the installed worker before launch. It uses the configured
interpreter, retains held-out answers privately, and does not publish footage.
The managed private ACP profile is not an OS sandbox; see [its boundary](private-acp.md).

## Publish reviewed footage

Review footage and any redacted public session export for private content. Keep
raw sessions/ files private. Add the reviewed video and optional export to the
existing publications/selected.json entry’s assets, with their SHA-256 values in
reviewed_inputs. Add demonstration metadata:

```json
{
  "reviewed": true,
  "capture_kind": "native-provider",
  "captured_revision": "<40 lowercase hexadecimal characters>",
  "recording_asset": "<selected .mp4 or .webm path>",
  "cuts": ["Describe every omitted wait or edit"]
}
```

An optional session_asset must name a separately selected public export. Reuse the
existing publication command and CI path. The homepage verifies selected media
bytes before creating a playback URL; absent/invalid footage stays unavailable.

Retain the deployed URL, Build source revision, publication tested revision and
native build revision. Inspect playback, downloads, browser example and install
links on the deployed page. Record mismatches as failures.

This implementation session has not supplied a native/provider recording or a
matching deployed-build witness. Broad regression and post-commit monitoring
belong to the operator: use just checks-status and
[GitHub Actions](https://github.com/LoumTechnologies/hickory-docs/actions).

## Optional browser practice footage

The input-changing branch of the existing browser coach now executes through the
same Rust prediction ledger compiled to WASM. Record input/prediction/seal/source
identity, actual requirement comparisons, retained failure and recorded correction
with the original expectation. Also show the changed-expectation block and
Unknown/promotion boundary. The browser has no provider calls; it is not substitute
native/provider footage. Label test-driver predictions/review and human inspection
separately. Keep private raw journals and inspection windows out of predictor
context and unreviewed public assets. The browser guarantee records exact scope.

For the human-operated Linux take, use the [ready-to-run recording procedure](issue-72-native-recording.md). It verifies and launches the witnessed package in a fresh workspace.
