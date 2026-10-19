import { useId } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiField, UiValue } from "../lib/uiActions";
import type { RuntimeRun } from "./runtimeObservations";

const text = { type: "string" as const, maxLength: 4096 };
const offset: UiField = { type: "number", minimum: 0, maximum: 100_000_000, integer: true };
const json = (value: unknown): UiValue => JSON.parse(JSON.stringify(value));
export function useRuntimeActions(runs: RuntimeRun[], selected: RuntimeRun | undefined, invocation: string, comparison: string, stale: boolean, controls: {
  select(id: string): void; occurrence(id: string): void; compare(id: string): void;
  pin(run: RuntimeRun): Promise<void>; remove(run: RuntimeRun): Promise<void>;
}) {
  const instance = useId(); const parent = `runtime-history:${instance}`;
  const revision = useUiRevision(() => [runs.map(run => [run.id, run.status, run.events.length, run.pinned]), selected?.id, invocation, comparison, stale]);
  const commands: { key: string; command: UiCommand }[] = [];
  function add(id: string, label: string, parameters: Record<string, UiField>, execute: UiCommand["execute"], value?: UiValue) {
    commands.push({ key: `${parent}/${id}`, command: { label, subject: parent, parent, parameters, prerequisites: ["Retained run/source identity; historical evidence remains historical"], read: () => ({ revision: revision(), enabled: true, ...(value === undefined ? {} : { value }) }), execute } });
  }
  const done = (value: UiValue = null) => ({ status: "completed" as const, value });
  const page: Record<string, UiField> = { offset, limit: { type: "number", minimum: 1, maximum: 64, integer: true } };
  const summary = { selected: selected?.id ?? null, invocation, comparison, source_changed: stale, total_runs: runs.length };
  add("runs", "Read runtime run index", page, async args => {
    const first = Number(args.offset), end = Math.min(runs.length, first + Number(args.limit));
    if (first > runs.length) return { status: "unavailable", reason: "Offset exceeds the retained run count" };
    return done(json({ ...summary, runs: runs.slice(first, end).map(run => ({ id: run.id, status: run.status, pinned: run.pinned ?? false, events: run.events.length, sources: run.sources.length })), next: end < runs.length ? end : null }));
  }, summary);
  for (const compare of [false, true]) add(compare ? "compare" : "select", compare ? "Compare retained runtime run" : "Select retained runtime run", { run: text }, async args => {
    const id = String(args.run);
    if (id && !runs.some(run => run.id === id)) return { status: "unavailable", reason: "Run does not belong to this retained history" };
    (compare ? controls.compare : controls.select)(id); return done({ [compare ? "comparison" : "selected"]: id || (compare ? null : "latest") });
  });
  if (selected) {
    const run = selected;
    const occurrences = [...new Set(run.events.map(event => event.invocation ?? "unknown"))];
    add("occurrence", "Select runtime occurrence", { invocation: text }, async args => {
      const id = String(args.invocation);
      if (id && !occurrences.includes(id)) return { status: "unavailable", reason: "Invocation does not belong to this retained run" };
      controls.occurrence(id); return done({ invocation: id || "all" });
    });
    add("sources", "Read retained runtime source index", page, async args => {
      const first = Number(args.offset), end = Math.min(run.sources.length, first + Number(args.limit));
      if (first > run.sources.length) return { status: "unavailable", reason: "Offset exceeds the retained source count" };
      return done(json({ run: run.id, sources: run.sources.slice(first, end).map((source, index) => ({ index: first + index, id: source.id, uri: source.uri, revision: source.revision, bytes: source.bytes.length })), total: run.sources.length, next: end < run.sources.length ? end : null }));
    });
    add("events", "Read retained runtime events", { offset, limit: { type: "number", minimum: 1, maximum: 64, integer: true } }, async args => {
      const events = invocation ? run.events.filter(event => (event.invocation ?? "unknown") === invocation) : run.events;
      const first = Number(args.offset), end = Math.min(events.length, first + Number(args.limit));
      if (first > events.length) return { status: "unavailable", reason: "Offset exceeds this run's selected event count" };
      return done(json({ run: run.id, status: run.status, source_changed: stale, events: events.slice(first, end), total: events.length, next: end < events.length ? end : null }));
    });
    if (run.sources.length) add("source", "Read retained runtime source bytes", { source: { type: "number", minimum: 0, maximum: run.sources.length - 1, integer: true }, offset, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
      const source = run.sources[Number(args.source)]; const first = Number(args.offset), end = Math.min(source.bytes.length, first + Number(args.limit));
      if (first > source.bytes.length) return { status: "unavailable", reason: "Offset is outside the retained source" };
      return done(json({ run: run.id, source: { id: source.id, uri: source.uri, revision: source.revision }, span: { start: first, end }, bytes: source.bytes.slice(first, end), total: source.bytes.length, next: end < source.bytes.length ? end : null, edit_baseline: false }));
    });
    add("pin", run.pinned ? "Unpin retained run" : "Pin retained run", {}, async () => { await controls.pin(run); return done({ run: run.id, pinned: !run.pinned }); });
    add("delete", "Delete retained runtime run", {}, async () => { await controls.remove(run); return done({ deleted: run.id }); });
    add("export", "Export selected runtime evidence JSON", {}, async () => done({ name: `runtime-${run.id}.json`, mime: "application/json", text: JSON.stringify(run, null, 2), sharing: "Local result only; review source/output before publishing" }));
  }
  useUiCommands(commands);
}
