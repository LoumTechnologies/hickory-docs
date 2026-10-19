# Runtime history keeps runs and invocation occurrences separate

The initial display policy selects the latest captured run, with explicit
history, pin/unpin and separate comparison. A newer failed/partial run exposes
its actual observations rather than retaining an older green result as current.
Historical selections remain explicit. Editing source preserves observations
but marks them stale and removes their live-editor decorations.

Inline CodeMirror widgets are view rows beneath actual source lines. They never
change document bytes or source/edit offsets. Real invocation IDs group events;
unknown invocation/location remains explicit. Expanding inline or shared source
observations selects those event IDs for the existing context projection,
including run/revision/status/truncation and original location precision.
Scrolling does not select evidence. Capturing output does not itself deliver it
to a model or establish semantic truth.

Desktop API retention uses the existing hick-observe LocalStore, preserving
immutable source/event prefixes, pins, recovery and retention rules. Standalone
browser hosts reuse transactional local workspace storage with the same run wire
model, digest checks, immutable prefixes and explicit quota/conflict errors.
Browser captures persist initial, each hundred-event prefix and terminal state;
unsaved suffixes can be lost on interruption. A saved running browser record
labels liveness unverified after restart and never claims success. Unpinned
terminal browser records use the native default 100-run/256-MiB/30-day policy;
pins and active captures are exempt. Browser storage can be cleared externally.

Live observations are view data by default. Explicit evidence capture downloads
a revision-bound JSON artifact the user can own alongside the document. It does
not silently insert generated output into authored Markdown. Existing show=output,
hick:output and ingested-recording persistence is unchanged. Captures may contain
private source/output and are not automatically uploaded or committed.

---

Last LLM verification:

- Date: 2026-10-19
- Reviewer: Codex
- Result: partially verified
- Originating requirement: #57/#47; execution-observation-provenance.md.
- Evidence: runtime_runs API and LocalState store, runRepository/runtimeReading,
  RuntimeHistory/useBrowserRuns/NativeRuntimeHistory, cmRuntime and
  DebuggableDocument callback wiring.
- Assumptions: latest means captured start timestamp with deterministic tie
  ordering, not a semantic winner. Browser IndexedDB durability depends on host
  storage availability. Native liveness/recovery belongs to the shared store.
- Witnesses: runtimeHistory.test.tsx (repository safety, source position
  preservation, selected context IDs, stale removal and run separation);
  serve_runtime_runs HTTP lifecycle; focused browser history/pin/reload/failed
  stream/keyboard/compare interaction.
- Challenges: equal output across invocations, stale source, newest failure,
  retained pin, corruption/refused rewrite, unknown locations and long streams.
- Coverage boundary: actual native Python emission is #56, not claimed by this
  presentation slice. Desktop HTTP evidence validates storage/API on real sockets;
  browser evidence uses the real browser provider. Context inclusion remains
  subject to Python request assembly/receipts; selected evidence is not delivery.
- Dependencies: files above in this commit; changes invalidate this review.
- Executed checks: two focused browser repository/renderer tests, one native
  HTTP lifecycle test and three serial real Chromium/Firefox/WebKit interaction
  flows passed. Production site build (including TypeScript) and diff checks
  passed. Initial browser failure was an exact-label test locator; the
  accessibility-role locator passed with the same captured run/options.
- Human judgment: not recorded; initial display policy remains open to feedback.
