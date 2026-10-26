<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Execution logs preserve contributor relationships and historical event identity

For: Authors running documents on their own machine and reviewers assessing their side effects.

The execution log and inline view select the same run and actual invocation
occurrences. Output is LF-framed per stream/category with lossless per-event
chunk-byte and displayed-line-byte contributions. UTF-8 is decoded after
assembly; CR/escape controls are visibly escaped. Partial lines and lines with
multiple emitting statements retain all contributor identities, without text
matching or invented single origins.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/runtime-logs-preserve-contributor-relationships.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-f70d869e5dff"></a>
## `apps/web/src/debug/runtimeLog.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Execution logs preserve contributor relationships and historical event identity](contract.md).

<a id="f-f70d869e5dff-source"></a>
### `source`

<storycode:copy id="f-f70d869e5dff-source">
// Guarantee: docs/guarantees/execution/runtime-logs-preserve-contributor-relationships.md
import { expect, it } from "vitest";
import { outputLines, logLinks, printable } from "./runtimeLog";
import { coalesceLinks } from "../shell/coalesceLinks";
import type { RuntimeRun, RuntimeEvent } from "./runtimeObservations";
const source = { id: "doc", uri: "example.md", revision: "a".repeat(64), bytes: [...new TextEncoder().encode("print one\nprint two\n")] };
</storycode:copy>
<a id="f-f70d869e5dff-event"></a>
### `event`

<storycode:copy id="f-f70d869e5dff-event">
const event = (id: string, bytes: number[], offset: number): RuntimeEvent => ({
  id, provider: "fixture-runtime", block: "main.js", invocation: id, parent_invocation: null, thread: null, task: null, time: null,
  emission: { source: "doc", block: "main.js", bytes: { start: offset, end: offset + 1 }, precision: "exact" },
  callers: [{ source: "doc", block: "main.js", bytes: { start: 10, end: 11 }, precision: "exact" }],
  observation: { kind: "output", stream: "stdout", category: "write", bytes },
});
</storycode:copy>
<a id="f-f70d869e5dff-fixture"></a>
### `fixture`

<storycode:copy id="f-f70d869e5dff-fixture">
const fixture = (): RuntimeRun => ({ schema_version: 1, id: "run-one", started_unix_ms: 1, status: "completed", pinned: false,
  sources: [source], truncated: false, missing_artifacts: [], detail: null,
  events: [event("event-one", [0xc3], 0), event("event-two", [0xa9, 0xf0, 0x9f, 0xa6, 0x80, 13, 10, 116, 97, 105, 108], 10)] });
</storycode:copy>
<a id="f-f70d869e5dff-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-f70d869e5dff-expression_statement">
it("decodes after assembly and preserves multi-origin partial-line byte contributions", () => {
  const lines = outputLines(fixture(), "stdout", "write");
  expect(printable(lines[0].bytes)).toBe("é🦀\\r\n");
  expect(lines[0].contributors).toEqual([
    { event: "event-one", chunk_bytes: { start: 0, end: 1 }, line_bytes: { start: 0, end: 1 } },
    { event: "event-two", chunk_bytes: { start: 0, end: 7 }, line_bytes: { start: 1, end: 8 } },
  ]);
  expect(lines[1]).toEqual({ bytes: [116, 97, 105, 108], terminated: false,
    contributors: [{ event: "event-two", chunk_bytes: { start: 7, end: 11 }, line_bytes: { start: 0, end: 4 } }] });
  expect(outputLines(fixture(), "stderr", "write")).toEqual([]);
});
</storycode:copy>
<a id="f-f70d869e5dff-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-f70d869e5dff-expression_statement-2">
it("keeps exact event/revision review targets and separate emitter/caller links without coalescing", () => {
  const run = fixture(); const lines = outputLines(run, "stdout", "write");
  const links = logLinks(run, lines, "log.txt", "source.txt");
  expect(links).toHaveLength(6);
  expect(coalesceLinks(links, [{ docPath: "log.txt", docSource: "first\nsecond" }])).toHaveLength(6);
  const emitter = links.find(link => link.title.includes("Execution emission"))!;
  expect(emitter.family).toBe("execution");
  expect(emitter.review?.source_hash).toBe(source.revision);
  expect(emitter.review?.relationship).toMatchObject({ run: run.id, event: { id: "event-one" }, source_revision: source.revision });
  expect(links.some(link => link.title.includes("calling context"))).toBe(true);
  run.events.forEach(event => { event.emission = null; event.callers = []; });
  expect(logLinks(run, lines, "log.txt", "source.txt")).toEqual([]);
});
</storycode:copy>

### Assemble `apps/web/src/debug/runtimeLog.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/runtimeLog.test.ts"><storycode:paste select="#f-f70d869e5dff-source"/><storycode:paste select="#f-f70d869e5dff-event"/><storycode:paste select="#f-f70d869e5dff-fixture"/><storycode:paste select="#f-f70d869e5dff-expression_statement"/><storycode:paste select="#f-f70d869e5dff-expression_statement-2"/></storycode:file>

## Implementation review

### Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../mechanisms/apps/web/e2e/browserEmbedding.spec.ts.md#output-31019d3567c7)
- [apps/web/src/debug/runtimeLog.test.ts](contract.md#output-f70d869e5dff)
- [apps/web/src/shell/linkShapes.test.ts](../../../mechanisms/apps/web/src/shell/linkShapes.ts.md#output-04fb400a1dba)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
