<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/execution/runtime-logs-preserve-contributor-relationships.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `9cee8664df2f835b1316e1ee7f204a5121421e77354cfc6b3dbd314d6eed0bdb`.

<book:entry path="docs/guarantees/execution/runtime-logs-preserve-contributor-relationships.md">
# Execution logs preserve contributor relationships and historical event identity

The execution log and inline view select the same run and actual invocation
occurrences. Output is LF-framed per stream/category with lossless per-event
chunk-byte and displayed-line-byte contributions. UTF-8 is decoded after
assembly; CR/escape controls are visibly escaped. Partial lines and lines with
multiple emitting statements retain all contributor identities, without text
matching or invented single origins.

The existing provenance overlay draws a distinct Execution family between
read-only captured log/source panes. Emission and calling-context navigation
remain separate and preserve captured source revisions. Exact mappings draw
exact connectors; unknown/block-only mappings stay explicit without fake
statement connectors. Multiple captured source files can be selected. Existing
brace/ribbon geometry, clipping and keyboard navigation are reused.

Each execution link retains its individual key even when repeated invocations
point to the same source line. Inline-to-log and log-to-inline navigation use
run/event IDs, not equal output text. Historical selections after edits keep
captured source and never attach old observations to changed live source.

Context-menu/Alt+Enter challenges use the existing review system with the
historical code quote/hash and queried event, run, contribution ranges and
source revision. Records without a displayed output contribution use an explicitly unavailable virtual record
address and a hash of the captured serialized record, not a fabricated current
file location. Execution provenance remains separate from byte lineage,
supplied model context and declared semantic support; no model infers links.

---

Last LLM verification:

- Date: 2026-10-19
- Reviewer: Codex
- Result: partially verified
- Originating requirement: #58/#47; execution-observation-provenance.md.
- Evidence: runtimeLog.outputLines/logLinks, RuntimeLog, runtimeNavigation,
  cmRuntime, RibbonOverlay/linkShapes/coalesceLinks and execution layer styling.
- Assumptions: receive order is not a cross-thread causal ordering. Framing
  deliberately separates stream/category, matching hick-observe::output_lines.
- Witnesses: runtimeLog.test.ts challenges split UTF-8, CRLF, multi-origin and
  partial lines, actual emitter/caller/review identities and unknown fallback.
  linkShapes tests preserve repeated-event keys/review targets under both styles;
  existing clipping/history checks remain applicable. Browser acceptance exercises
  actual provider events, repeated calls, mixed runs, stale source and review.
- Coverage boundary: only provider-supplied locations produce connectors. Unknown
  mappings remain navigable as event records; missing source artifacts do not
  become exact. Native review persistence reuses the existing append-only store;
  standalone browser hosts need their own review capability for durable posting.
- Dependencies: source/test files above in this commit; changes invalidate review.
- Executed checks: fifteen focused serial frontend safety/geometry/framing/layer
  tests and three serial real Chromium/Firefox/WebKit flows passed. Production
  site build (including TypeScript) and diff checks passed.
- Human judgment: not recorded.

### Shared captured-log relationships (issue #72)

Runtime-log disclosure now gates shared stream/source/style selectors, bounded
rendered-contributor metadata reading, emitting/throwing/caller navigation and
relationship challenges. Closing retires those detailed controls; source/log
text remains owned by the existing read-only editors. Only events visible in
the selected stream (plus the displayed typed observations) get relationship
actions. Changed run/stream/source state refuses old handles. Review results
report an opened review request, not a completed investigation or validation.

Missing retained source no longer crashes the log viewer: output remains readable
with an explicit absence message and no source chooser action. Fallback review
records hash the exact captured run and derive source revision from the event's
location, not the unrelated source selected in the adjacent pane. Pending hashes
cannot open reviews after capture replacement, unmount, or closing/reopening the
disclosure. Duplicate pending review requests are refused. Agent cancellation also refuses
a late review open, and disclosure/read controls have independent action locks
so a pending review does not prevent closing or inspecting it. Queued source jumps
are bound to their record/source; detached editors are not navigation targets.

Two focused captured-fixture UI cases pass, including native/model selectors,
stream-bound retirement, exact source review versus full-record fallback digest,
missing source, replaced-capture, agent-cancelled and closed/reopened-disclosure
late-hash refusal. Real
read-only CodeMirror editors are mounted; the review opener is mocked. Strict
TypeScript/file-length/diff checks pass. This does not witness a real execution,
provider, native release, completed review or deployed flow.
</book:entry>

</book:doc>
