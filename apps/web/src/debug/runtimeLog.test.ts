// Guarantee: docs/guarantees/execution/runtime-logs-preserve-contributor-relationships.md
import { expect, it } from "vitest";
import { outputLines, logLinks, printable } from "./runtimeLog";
import { coalesceLinks } from "../shell/coalesceLinks";
import type { RuntimeRun, RuntimeEvent } from "./runtimeObservations";
const source = { id: "doc", uri: "example.md", revision: "a".repeat(64), bytes: [...new TextEncoder().encode("print one\nprint two\n")] };
const event = (id: string, bytes: number[], offset: number): RuntimeEvent => ({
  id, provider: "fixture-runtime", block: "main.js", invocation: id, parent_invocation: null, thread: null, task: null, time: null,
  emission: { source: "doc", block: "main.js", bytes: { start: offset, end: offset + 1 }, precision: "exact" },
  callers: [{ source: "doc", block: "main.js", bytes: { start: 10, end: 11 }, precision: "exact" }],
  observation: { kind: "output", stream: "stdout", category: "write", bytes },
});
const fixture = (): RuntimeRun => ({ schema_version: 1, id: "run-one", started_unix_ms: 1, status: "completed", pinned: false,
  sources: [source], truncated: false, missing_artifacts: [], detail: null,
  events: [event("event-one", [0xc3], 0), event("event-two", [0xa9, 0xf0, 0x9f, 0xa6, 0x80, 13, 10, 116, 97, 105, 108], 10)] });
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
