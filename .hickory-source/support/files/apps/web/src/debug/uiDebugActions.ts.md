<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/uiDebugActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `9e230f2b6fab9ccc858f7d4ac458e9875b5b8c17229deff48b0758db02eb1ef4`.

<book:entry path="apps/web/src/debug/uiDebugActions.ts">
import { useId } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiField, UiValue } from "../lib/uiActions";
import type { DebugSession } from "./useDebugger";
import type { Step } from "./client";

const text = { type: "string" as const, maxLength: 8192 };
const line: UiField = { type: "number", minimum: 0, maximum: 100_000_000, integer: true };
const json = (value: unknown): UiValue => JSON.parse(JSON.stringify(value));

/** Registers the actual hook controller, shared by gutters, keys and strip. */
export function useDebugActions(path: string, debug: DebugSession, epoch: () => number, available: boolean, sourceIdentity?: unknown) {
  const instance = useId();
  const parent = `debug:${path}:${instance}`;
  const renderedEpoch = epoch();
  const revision = useUiRevision(() => [sourceIdentity, epoch(), debug.status, debug.reading?.source.revision, debug.reading?.scope, debug.selectedFrame, debug.breakpoints, debug.watches, debug.exceptionFilters]);
  const fresh = () => available && epoch() === renderedEpoch;
  const paused = () => fresh() && debug.status === "paused";
  const commands: { key: string; command: UiCommand }[] = [];
  function add(id: string, label: string, parameters: Record<string, UiField>, enabled: () => boolean, execute: UiCommand["execute"], value?: () => UiValue) {
    commands.push({ key: `${parent}/${id}`, command: {
      label, subject: parent, parent, parameters, prerequisites: ["Current debugger session, source, stop and frame; adapter capabilities apply"],
      read: () => ({ revision: revision(), enabled: enabled(), reason: !fresh() ? "Debugger unavailable or state is updating; observe again" : !enabled() ? "Operation unavailable at this debugger state or adapter capability" : undefined, ...(value ? { value: value() } : {}) }),
      execute,
    } });
  }
  const requested = (): { status: "completed"; value: UiValue } => ({ status: "completed", value: { dispatch: "requested", runtime_result: "Observe debugger state and retained run evidence" } });
  const collections = () => ({ frames: debug.frames, variables: debug.variables, breakpoints: debug.breakpoints, sourceBreakpoints: debug.sourceBreakpoints, watches: debug.watches, lastValue: debug.lastValue, buildOutput: debug.buildOutput, locations: debug.reading?.locations ?? [], annotations: debug.reading?.annotations ?? [], unsupported: debug.reading?.unsupported ?? [], capabilities: debug.capabilities });
  add("state", "Read debugger state", {}, fresh, async () => ({ status: "completed", value: snapshot() }), () => json({ status: debug.status, program: debug.program, selectedFrame: debug.selectedFrame, scope: debug.reading?.scope ?? null, detail: "Invoke state, then explicitly read bounded detail bytes" }));
  function snapshot() {
    return json({ status: debug.status, message: debug.message?.slice(0, 2048), program: debug.program, selectedFrame: debug.selectedFrame, exitCode: debug.exitCode,
      collections: Object.fromEntries(Object.entries(collections()).map(([key, value]) => [key, { items: Array.isArray(value) ? value.length : null, read: "Read debugger detail bytes" }])),
      evidence: debug.reading ? { source: { id: debug.reading.source.id, uri: debug.reading.source.uri, revision: debug.reading.source.revision }, scope: debug.reading.scope, annotations: debug.reading.annotations.filter(item => ["paused", "breakpoint_bound"].includes(item.payload.kind)).slice(0, 64), coverage: "At most 64 pause/bound annotations; request detail bytes for full collections" } : null });
  }
  add("detail", "Read debugger detail bytes", { collection: { ...text, choices: Object.keys(collections()) }, offset: { type: "number", minimum: 0, maximum: 100_000_000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, fresh, async args => {
    const collection = String(args.collection) as keyof ReturnType<typeof collections>;
    const bytes = new TextEncoder().encode(JSON.stringify(collections()[collection] ?? null));
    const offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds this debugger collection; observe the current stop" };
    return { status: "completed", value: { collection, encoding: "JSON encoded as UTF-8 bytes", scope: json(debug.reading?.scope ?? null), span: [offset, end], total_bytes: bytes.length, bytes: [...bytes.slice(offset, end)], next: end < bytes.length ? end : null } };
  });
  add("start", "Debug saved program", { program: text }, () => fresh() && ["idle", "finished", "failed"].includes(debug.status), async args => { debug.start(String(args.program) || undefined); return requested(); });
  add("stop", "Stop debugger", {}, () => fresh() && debug.status !== "idle", async () => { debug.stop(); return requested(); });
  for (const [how, label] of [["continue", "Continue"], ["over", "Step over"], ["in", "Step into"], ["out", "Step out"], ["back", "Reverse step"], ["drop_frame", "Restart frame"]] as [Step, string][]) {
    add(how, label, {}, () => paused() && (how !== "back" || !!debug.capabilities?.step_back) && (how !== "drop_frame" || !!debug.capabilities?.restart_frame), async () => { debug.step(how); return requested(); });
  }
  add("frame", "Select stack frame", { frame: line }, paused, async args => {
    const frame = Number(args.frame);
    if (!debug.frames.some(item => item.id === frame)) return { status: "unavailable", reason: "Frame does not belong to this stop" };
    debug.selectFrame(frame); return requested();
  });
  add("breakpoint", "Toggle document breakpoint", { line }, fresh, async args => {
    if (!Number.isInteger(args.line)) return { status: "unavailable", reason: "Line must be a zero-based integer in this document" };
    debug.toggleBreakpoint(Number(args.line)); return requested();
  });
  add("breakpoint-options", "Set breakpoint condition, hit count and log message", { line, condition: text, hit_condition: text, log_message: text }, () => fresh() && !!debug.capabilities, async args => {
    if (!debug.breakpoints.some(item => item.line === args.line)) return { status: "unavailable", reason: "Observe an existing breakpoint first" };
    if ((args.condition && !debug.capabilities?.conditional_breakpoints) || (args.hit_condition && !debug.capabilities?.hit_conditional_breakpoints) || (args.log_message && !debug.capabilities?.log_points)) return { status: "unavailable", reason: "Adapter does not support the requested breakpoint option" };
    debug.setBreakpointCondition(Number(args.line), { condition: String(args.condition), hit_condition: String(args.hit_condition), log_message: String(args.log_message) }); return requested();
  });
  add("evaluate", "Evaluate expression in selected frame", { expression: text }, paused, async args => {
    const value = await debug.query(String(args.expression));
    return value === null ? { status: "unavailable", reason: "No evaluation result received for this stop" } : { status: "completed", value: { expression: args.expression, value, requested_scope: json(debug.reading?.scope ?? null), freshness: "Evaluator response; observe the latest stop before using this as a current value" } };
  });
  for (const remove of [false, true]) add(remove ? "remove-watch" : "watch", remove ? "Remove watch" : "Add watch", { expression: text }, fresh, async args => { (remove ? debug.removeWatch : debug.addWatch)(String(args.expression)); return requested(); });
  add("exception-filter", "Toggle exception filter", { filter: text }, fresh, async args => {
    if (!debug.capabilities?.exception_filters.some(item => item.id === args.filter)) return { status: "unavailable", reason: "Unknown adapter exception filter" };
    debug.toggleExceptionFilter(String(args.filter)); return requested();
  });
  add("jump", "Move instruction pointer", { line }, () => paused() && !!debug.capabilities?.goto_targets, async args => { debug.jumpTo(Number(args.line)); return requested(); });
  add("run-to", "Run to document line", { line }, paused, async args => { debug.runTo(Number(args.line)); return requested(); });
  useUiCommands(commands);
}
</book:entry>

</book:doc>
