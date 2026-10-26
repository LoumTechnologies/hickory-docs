<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/useRuntimeLogActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8e101a2bb928"></a>
## `apps/web/src/debug/useRuntimeLogActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-8e101a2bb928-useRuntimeLogActions"></a>
### `useRuntimeLogActions`

<storycode:copy id="f-8e101a2bb928-useRuntimeLogActions">
import { useId } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiField, UiValue } from "../lib/uiActions";
import type { RuntimeEvent, RuntimeLocation, RuntimeRun } from "./runtimeObservations";
import type { OutputLine } from "./runtimeLog";

export function useRuntimeLogActions(run: RuntimeRun, state: {
  open: boolean; pair: string | undefined; pairs: string[]; source: string | null;
  style: string; selected: string; lines: OutputLine[]; pending: boolean; error: string | null;
}, controls: {
  open(value: boolean): void; pair(value: string): void; source(value: string): void; style(value: "bands" | "braces"): void;
  navigate(event: RuntimeEvent, location: RuntimeLocation | null): void; challenge(event: RuntimeEvent, signal?: AbortSignal): Promise<boolean>;
}) {
  const parent = `runtime-log:${useId()}`;
  const revision = useUiRevision(() => [run, state]);
  const commands: { key: string; command: UiCommand }[] = [];
  const done = (value: UiValue) => ({ status: "completed" as const, value });
  const add = (id: string, label: string, parameters: Record<string, UiField>, execute: UiCommand["execute"], enabled = true) => commands.push({ key: `${parent}:${id}`, command: {
    parent, subject: `${parent}:${id === "open" ? "visibility" : id === "read" ? "reading" : id.endsWith(":review") ? "review" : "selection"}`, label, parameters, prerequisites: ["Captured execution relationships; navigation and review requests do not verify their correctness"],
    read: () => ({ revision: revision(), enabled, ...(id === "open" ? { value: { open: state.open, run: run.id } } : {}) }), execute,
  } });
  add("open", "Set runtime log visibility", { open: { type: "boolean" } }, async args => { controls.open(args.open as boolean); return done({ open: args.open }); });
  if (state.open) {
    add("read", "Read runtime log relationship metadata bytes", {
      offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true },
    }, async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ run: run.id, pairs: state.pairs, selected_pair: state.pair, source: state.source,
        sources: run.sources.map(source => ({ id: source.id, uri: source.uri, revision: source.revision })), style: state.style,
        lines: state.lines.map((line, index) => ({ line: index + 1, terminated: line.terminated, contributors: line.contributors.map(item => {
          const event = run.events.find(event => event.id === item.event);
          return { ...item, invocation: event?.invocation ?? "unknown", emission: event?.emission?.precision ?? "unknown", callers: event?.callers.map(caller => caller.precision) ?? [] };
        }) })), typed_events: run.events.filter(event => event.observation.kind !== "output").map(event => ({ id: event.id, kind: event.observation.kind })),
        selected_event: state.selected, pending_review: state.pending, error: state.error,
        scope: "Visible captured relationships. Source/log text has its own read-only editor. Captures are historical and are not current edit baselines or independent verification." }));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside current runtime log metadata" };
      return done({ bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null });
    });
    if (state.pairs.length) add("pair", "Select runtime log stream", { index: { type: "number", minimum: 0, maximum: state.pairs.length - 1, integer: true } }, async args => { controls.pair(state.pairs[args.index as number]); return done({ selected_index: args.index }); });
    if (run.sources.length) add("source", "Select runtime captured source", { index: { type: "number", minimum: 0, maximum: run.sources.length - 1, integer: true } }, async args => { controls.source(run.sources[args.index as number].id); return done({ selected_index: args.index }); });
    add("style", "Set runtime relationship connector style", { style: { type: "string", maxLength: 16, choices: ["bands", "braces"] } }, async args => { controls.style(args.style as "bands" | "braces"); return done({ style: args.style }); });
    const visible = new Set(state.lines.flatMap(line => line.contributors.map(item => item.event)));
    for (const event of run.events.filter(event => visible.has(event.id) || event.observation.kind !== "output")) {
      const location = event.observation.kind === "exception" ? event.observation.throwing : event.emission;
      add(`${event.id}:emission`, `Go to runtime emitting/throwing location: ${event.id}`, {}, async () => { controls.navigate(event, location); return done({ navigation_requested: true, event: event.id, location_coordinates_available: !!location?.bytes, captured_source_available: run.sources.some(source => source.id === location?.source) }); });
      for (const [index, caller] of event.callers.entries()) if (event.observation.kind === "output")
        add(`${event.id}:caller:${index}`, `Go to runtime calling context ${index + 1}: ${event.id}`, {}, async () => { controls.navigate(event, caller); return done({ navigation_requested: true, event: event.id }); });
      add(`${event.id}:review`, `Challenge runtime relationship: ${event.id}`, {}, async (_args, _actor, signal) => await controls.challenge(event, signal)
        ? done({ review_requested: true, event: event.id, review_completed: false })
        : { status: "unavailable", reason: "Review request failed, its capture changed, or another review is pending; inspect the runtime log" }, !state.pending);
    }
  }
  useUiCommands(commands);
}
</storycode:copy>

### Assemble `apps/web/src/debug/useRuntimeLogActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/useRuntimeLogActions.ts"><storycode:paste select="#f-8e101a2bb928-useRuntimeLogActions"/></storycode:file>

</storycode:doc>
