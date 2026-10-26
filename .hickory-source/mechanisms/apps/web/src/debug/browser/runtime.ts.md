<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/browser/runtime.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6c24fa24bf81"></a>
## `apps/web/src/debug/browser/runtime.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Browser debugging runs edited source](../../../../../../guarantees/debugging/browser-debugging-runs-edited-source/contract.md), [Observations read beside their captured source](../../../../../../guarantees/debugging/source-anchored-observation-reading/contract.md), [Browser-local prediction practice uses the shared gate](../../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md).

<a id="f-6c24fa24bf81-BrowserInput"></a>
### `BrowserInput`

<storycode:copy id="f-6c24fa24bf81-BrowserInput">
import type { RuntimeRun, RuntimeEvent, RuntimeLocation } from "../runtimeObservations";
import type { ObservationReading, ObservationAnnotation } from "../observations";
import Interpreter from "js-interpreter";
import type { IState, IScope, INode } from "js-interpreter";
import type { DebugBreakpoint, DebugEvent, Frame, Step } from "../client";
import { compileProgram } from "./compile";
import type { LiteralFile } from "../../editor/hickLang";
import { Inspector, preview } from "./inspect";

export interface BrowserInput { observationRevision?: string; source: string; revision: string; file: LiteralFile; }
</storycode:copy>
<a id="f-6c24fa24bf81-BrowserEvent"></a>
### `BrowserEvent`

<storycode:copy id="f-6c24fa24bf81-BrowserEvent">
export type BrowserEvent = DebugEvent | { event: "runtime_event"; session: string; observation: RuntimeEvent } | { event: "runtime"; session: string; run: RuntimeRun } | { event: "output"; session: string; text: string; category: "stdout" | "stderr" };
</storycode:copy>
<a id="f-6c24fa24bf81-BROWSER_CAPABILITIES"></a>
### `BROWSER_CAPABILITIES`

<storycode:copy id="f-6c24fa24bf81-BROWSER_CAPABILITIES">
export const BROWSER_CAPABILITIES = {
  frame_locals: true,
  conditional_breakpoints: false, hit_conditional_breakpoints: false, log_points: false,
  set_variable: false, restart_frame: false, step_in_targets: false, step_back: false,
  goto_targets: false, exception_filters: [],
};
</storycode:copy>
<a id="f-6c24fa24bf81-statements"></a>
### `statements`

<storycode:copy id="f-6c24fa24bf81-statements">
const statements = new Set(["VariableDeclaration", "ExpressionStatement", "ReturnStatement", "ThrowStatement",
  "IfStatement", "WhileStatement", "DoWhileStatement", "ForStatement", "ForInStatement", "SwitchStatement",
  "BreakStatement", "ContinueStatement", "DebuggerStatement"]);

</storycode:copy>
<a id="f-6c24fa24bf81-BrowserRun"></a>
### `BrowserRun`

Real interpreter state, not a trace or evaluator of the demonstration.

<storycode:copy id="f-6c24fa24bf81-BrowserRun">
/** Real interpreter state, not a trace or evaluator of the demonstration. */
export class BrowserRun {
  private reading: ObservationReading | null = null;
  private runtime: RuntimeRun | null = null;
  private invocations = new WeakMap<IState, string>();
  private invocationSequence = 0;
  private lastThrown: RuntimeEvent | null = null;
  private evidence = 0;
  private stopSequence = 0;
  private interpreter: Interpreter;
  private program: ReturnType<typeof compileProgram>;
  private inspector = new Inspector();
  private visited = new WeakSet<IState>();
  private builtins = new Set<string>();
  private executable = new Set<number>();
  private breakpoints: DebugBreakpoint[];
  private paused = false;
  private ended = false;
  private pauseRequested = false;
  private mode: Step = "continue";
  private depth = 0;
  private steps = 0;
  private outputBytes = 0;
  private frameScopes = new Map<number, IScope>();

  constructor(readonly input: BrowserInput, readonly session: string, readonly doc: string,
    breakpoints: DebugBreakpoint[], private emit: (event: BrowserEvent) => void) {
    this.program = compileProgram(input.file, input.source);
    this.breakpoints = breakpoints;
    if (input.observationRevision) this.runtime = { schema_version: 1, id: session,
      started_unix_ms: Date.now(), status: "running", pinned: false,
      sources: [{ id: "document", uri: doc, revision: input.observationRevision,
        bytes: [...new TextEncoder().encode(input.source)] }], events: [], truncated: false,
      missing_artifacts: [], detail: "Browser interpreter; receive order only; no external process streams" };
    this.interpreter = new Interpreter(this.program.code, (interpreter, global) => {
      const consoleObject = interpreter.createObjectProto(null);
      for (const category of ["log", "warn", "error", "write"]) {
        interpreter.setProperty(consoleObject, category, interpreter.createNativeFunction((...values) => {
          const text = values.map((value) => typeof value === "string" ? value : preview(value)).join(" ") + (category === "write" ? "" : "\n");
          this.outputBytes += new TextEncoder().encode(text).length;
          if (this.outputBytes > 65_536) throw new Error("Browser output limit exceeded (64 KiB)");
          this.observe({ kind: "output", stream: category === "error" ? "stderr" : "stdout",
            category, bytes: [...new TextEncoder().encode(text)] });
          this.emit({ event: "output", session, text, category: category === "error" ? "stderr" : "stdout" });
          return undefined;
        }));
      }
      interpreter.setProperty(global, "console", consoleObject);
      this.builtins = new Set(Object.keys(global.properties));
    });
    const throwException = this.interpreter.throwException;
    this.interpreter.throwException = (value, message) => {
      this.lastThrown = this.observe({ kind: "exception", type_name: typeof value,
        message: message ?? preview(value), handling: "throwing", throwing: this.location(this.interpreter.stateStack.at(-1)?.node), catching: null });
      return throwException.call(this.interpreter, value, message);
    };
    this.publishRuntime();
    // Polyfills initialize first; only then remove dynamic-code capabilities.
    for (const name of ["eval", "Function", "setTimeout", "setInterval", "clearTimeout", "clearInterval"]) delete this.interpreter.globalScope.object.properties[name];
    delete this.interpreter.FUNCTION_PROTO.properties["constructor"];
    const collect = (node: INode) => {
      if (statements.has(node.type)) { const line = this.program.lineAt(node.start); if (line !== null) this.executable.add(line); }
      for (const [key, child] of Object.entries(node)) {
        if (key === "loc") continue;
        if (Array.isArray(child)) for (const n of child) { if (n?.type) collect(n); }
        else if (child && typeof child === "object" && "type" in child) collect(child as INode);
      }
    };
    collect(this.interpreter.ast);
    this.emit({ event: "started", doc, session, capabilities: BROWSER_CAPABILITIES, breakpoints: this.bind() });
  }
  private bind() {
    return this.breakpoints.map((b) => ({ line: b.line,
      state: this.executable.has(b.line) && !b.condition && !b.hit_condition && !b.log_message ? "bound" as const : "refused" as const,
      message: !this.executable.has(b.line) ? "No executable statement on this line" : b.condition || b.hit_condition || b.log_message ? "Browser breakpoint conditions are unavailable" : undefined }));
  }
  setBreakpoints(breakpoints: DebugBreakpoint[]) { this.breakpoints = breakpoints; this.emit({ event: "breakpoints", session: this.session, breakpoints: this.bind() }); }
  pause() { this.pauseRequested = true; }
  step(how: Step) {
    if (!["continue", "in", "out", "over"].includes(how)) throw new Error("This browser backend cannot reverse or restart a frame");
    this.mode = how;
    this.depth = this.calls().length;
    this.paused = false;
  }
  stop() { this.ended = true; if (this.runtime) { this.runtime.status = "cancelled"; this.publishRuntime(); } }
  private calls() { return this.interpreter.stateStack.filter((s) => s.func_?.node && s.doneExec_); }
  /** Returns true while runnable, false while paused/finished. Worker schedules batches. */
  pump(budget = 1500): boolean {
    if (this.paused || this.ended) return false;
    try {
      for (let n = 0; n < budget; n++) {
        this.lastThrown = null;
        if (++this.steps > 2_000_000) throw new Error("Browser execution limit exceeded (2 million interpreter steps)");
        const top = this.interpreter.stateStack.at(-1);
        if (top) {
          const line = this.program.lineAt(top.node.start);
          const checkpoint = statements.has(top.node.type) && !this.visited.has(top);
          if (checkpoint) this.visited.add(top);
          const depth = this.calls().length;
          const stepping = this.mode === "in" || this.mode === "over" && depth <= this.depth || this.mode === "out" && depth < this.depth;
          const breakpoint = this.breakpoints.some((b) => b.line === line && !b.condition && !b.hit_condition && !b.log_message);
          if (line !== null && (this.pauseRequested || checkpoint && (stepping || breakpoint))) {
            const reason = this.pauseRequested ? "pause" : breakpoint ? "breakpoint" : "step";
            this.pauseRequested = false;
            this.paused = true;
            this.report(reason);
            return false;
          }
        }
        if (!this.interpreter.step()) {
          this.ended = true;
          if (this.runtime) { this.runtime.status = "completed"; this.publishRuntime(); }
          this.emit({ event: "finished", session: this.session, exit_code: 0 });
          return false;
        }
      }
    } catch (e) {
      this.ended = true;
      const message = e instanceof Error ? e.message : String(e);
      const original = this.lastThrown?.observation;
      this.observe({ kind: "exception", type_name: e instanceof Error ? e.name : "unknown",
        message, handling: original?.kind === "exception" ? "unhandled" : "unknown",
        throwing: original?.kind === "exception" ? original.throwing : this.location(this.interpreter.stateStack.at(-1)?.node), catching: null });
      if (this.runtime) { this.runtime.status = "failed"; this.runtime.truncated ||= message.includes("output limit exceeded"); this.publishRuntime(); }
      this.emit({ event: "output", session: this.session, category: "stderr", text: message + "\n" });
      this.emit({ event: "failed", session: this.session, message });
      this.emit({ event: "finished", session: this.session, exit_code: 1 });
      return false;
    }
    return true;
  }
  private location(node?: INode): RuntimeLocation | null {
    if (!node) return null;
    const start = this.program.sourceByteAt(node.start);
    if (start === null) return { source: "document", block: this.input.file.path, bytes: null, precision: "block_only" };
    // Exact anchor byte, not an invented full statement range across segments.
    const bytes = new TextEncoder().encode(this.input.source);
    let end = start + 1;
    while (end < bytes.length && (bytes[end] & 0xc0) === 0x80) end++;
    return { source: "document", block: this.input.file.path, bytes: { start, end }, precision: "exact" };
  }
  private invocation(state?: IState): string {
    if (!state) return `${this.session}:program`;
    let id = this.invocations.get(state);
    if (!id) { id = `${this.session}:invocation:${++this.invocationSequence}`; this.invocations.set(state, id); }
    return id;
  }
  private publishRuntime() {
    if (this.runtime) this.emit({ event: "runtime", session: this.session, run: structuredClone(this.runtime) });
  }
  private observe(observation: RuntimeEvent["observation"]): RuntimeEvent | null {
    const run = this.runtime;
    if (!run) return null;
    if (run.events.length >= 4096) { run.truncated = true; run.detail = "Observation limit reached (4096 events); later events omitted"; return null; }
    const calls = this.calls();
    const event: RuntimeEvent = { id: `${this.session}:event:${run.events.length + 1}`, provider: "browser-interpreter",
      block: this.input.file.path, invocation: this.invocation(calls.at(-1)),
      parent_invocation: calls.length ? this.invocation(calls.at(-2)) : null,
      thread: "browser-worker", task: null, time: { domain: this.session, unit: "ms", value: performance.now() },
      emission: this.location(this.interpreter.stateStack.at(-1)?.node),
      callers: calls.slice().reverse().map(call => this.location(call.node)).filter((location): location is RuntimeLocation => location !== null), observation };
    run.events.push(event);
    this.emit({ event: "runtime_event", session: this.session, observation: structuredClone(event) });
    return event;
  }
  report(reason = "frame", selected?: number) {
    if (!this.paused) throw new Error("Frame inspection requires a paused session");
    const top = this.interpreter.stateStack.at(-1);
    if (!top) return;
    this.inspector.reset();
    this.frameScopes.clear();
    const calls = this.calls().reverse();
    const frames: Frame[] = [];
    let scope = top.scope;
    let line = this.program.lineAt(top.node.start);
    for (let i = 0; i <= calls.length; i++) {
      const call = calls[i];
      const name = call ? call.func_?.node?.id?.name ?? this.program.code.slice(call.node.start, call.node.end).split("(")[0] : "<program>";
      frames.push({ id: i + 1, name, line, source: this.input.file.path, in_document: line !== null });
      this.frameScopes.set(i + 1, scope);
      if (call) { scope = call.scope; line = this.program.lineAt(call.node.start); }
    }
    const selectedScope = this.frameScopes.get(selected ?? 1) ?? top.scope;
    const variables = this.inspector.locals(selectedScope, this.builtins);
    this.captureReading(reason, frames, variables.map(v => [v.name, v.value]), selected ?? 1);
    this.emit({ event: "stopped", observation_scope: this.reading?.scope, session: this.session, reason, frames,
      selected_frame: selected ?? 1,
      variables, line: frames.find((f) => f.id === selected)?.line ?? frames[0].line });
  }
  private addAnnotation(line: number | null, frame: number | null, payload: ObservationAnnotation["payload"]) {
    const reading = this.reading;
    if (!reading) return;
    let location: string | null = null;
    if (line !== null) {
      location = `${reading.source.revision}:L${line}`;
      if (!reading.locations.some(l => l.id === location)) {
        const lines = this.input.source.split("\n");
        const start = new TextEncoder().encode(lines.slice(0, line).map(l => l + "\n").join("")).length;
        reading.locations.push({ id: location, revision: reading.source.revision, line, location: {
          source: "document", bytes: { start, end: start + new TextEncoder().encode(lines[line] ?? "").length }, precision: "exact" } });
      }
    }
    reading.annotations.push({ evidence: `${this.session}:E${this.evidence++}`, provider: "browser-interpreter", payload_schema: "hick.debug-annotation.v1",
      location, scope: { ...reading.scope, frame }, payload });
  }
  private captureReading(reason: string, frames: Frame[], values: string[][], selected: number) {
    if (!this.input.observationRevision) return; // Older direct hosts explicitly lack captured evidence.
    const previous = this.reading;
    const stop = reason === "frame" && previous ? previous.scope.stop : `${this.session}:S${++this.stopSequence}`;
    this.reading = { schema_version: 1, source: { id: "document", uri: this.doc, revision: this.input.observationRevision, bytes: [...new TextEncoder().encode(this.input.source)] },
      scope: { session: this.session, run: this.session, stop, frame: selected, invocation: null }, locations: [], annotations: [],
      unsupported: ["invocation identity", "reverse execution", "restart frame", "side-effectful evaluation"] };
    this.bind().forEach((bp, i) => {
      const breakpoint = `${this.session}:B${i}`;
      this.addAnnotation(bp.line, null, { kind: "breakpoint_requested", breakpoint, state: bp.state });
      if (bp.state === "bound") this.addAnnotation(bp.line, null, { kind: "breakpoint_bound", breakpoint });
    });
    frames.forEach((f, depth) => this.addAnnotation(f.line, f.id, { kind: "frame", name: f.name, depth }));
    // Selected caller values must never move the actual paused statement.
    this.addAnnotation(frames[0]?.line ?? null, frames[0]?.id ?? null, { kind: "paused", reason });
    this.addAnnotation(frames.find(f => f.id === selected)?.line ?? null, selected, { kind: "values", values });
    if (previous?.scope.stop === stop) {
      for (const annotation of previous.annotations.filter(a => a.payload.kind === "expression")) {
        const location = previous.locations.find(l => l.id === annotation.location);
        if (location && !this.reading.locations.some(l => l.id === location.id)) this.reading.locations.push(location);
        this.reading.annotations.push(annotation);
      }
    }
    this.emit({ event: "reading", session: this.session, reading: structuredClone(this.reading) });
  }
  evaluate(expression: string, frame?: number) {
    if (!this.paused) throw new Error("Watch expressions require a paused session");
    const scope = this.frameScopes.get(frame ?? 1);
    if (!scope) throw new Error("Unknown frame");
    const variable = this.inspector.variable(expression, this.inspector.evaluate(expression, scope));
    if (this.reading && this.reading.scope.frame === (frame ?? 1)) {
      const frameLocation = this.reading.annotations.find(a => a.payload.kind === "frame" && a.scope.frame === (frame ?? 1))?.location;
      const line = this.reading.locations.find(l => l.id === frameLocation)?.line ?? null;
      this.addAnnotation(line, frame ?? 1, { kind: "expression", expression, value: variable.value, receipt: `${this.session}:evaluation:${this.evidence}` });
      this.emit({ event: "reading", session: this.session, reading: structuredClone(this.reading) });
    }
    this.emit({ event: "value", session: this.session, expression, value: variable.value, type: variable.type ?? null, reference: variable.variables_reference });
  }
  children(reference: number) {
    if (!this.paused) throw new Error("Values belong to a paused session");
    this.emit({ event: "children", session: this.session, reference, variables: this.inspector.children(reference) });
  }
}
</storycode:copy>

### Assemble `apps/web/src/debug/browser/runtime.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/browser/runtime.ts"><storycode:paste select="#f-6c24fa24bf81-BrowserInput"/><storycode:paste select="#f-6c24fa24bf81-BrowserEvent"/><storycode:paste select="#f-6c24fa24bf81-BROWSER_CAPABILITIES"/><storycode:paste select="#f-6c24fa24bf81-statements"/><storycode:paste select="#f-6c24fa24bf81-BrowserRun"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-de529cc60d5d"></a>
## `apps/web/src/debug/browser/runtime.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Browser debugging runs edited source](../../../../../../guarantees/debugging/browser-debugging-runs-edited-source/contract.md), [Observations read beside their captured source](../../../../../../guarantees/debugging/source-anchored-observation-reading/contract.md), [Literal files use native bytes](../../../../../../guarantees/embedding/literal-files-use-native-bytes/contract.md), [Browser output carries actual runtime source and occurrence evidence](../../../../../../guarantees/execution/browser-output-has-runtime-source-evidence/contract.md).

<a id="f-de529cc60d5d-start"></a>
### `start`

<storycode:copy id="f-de529cc60d5d-start">
import { describe, expect, it } from "vitest";
import { BrowserRun } from "./runtime";
import type { BrowserEvent } from "./runtime";
import { materializeLiteralFiles } from "../../editor/hickLang";
import type { DebugBreakpoint } from "../client";

// Guarantees: docs/guarantees/debugging/browser-debugging-runs-edited-source.md
// docs/guarantees/embedding/literal-files-use-native-bytes.md
function start(code: string, breakpoints: DebugBreakpoint[] = [], language = "js") {
  const source = `# 🦀 Live\n<hick:file path="main.${language}">\n${code}\n</hick:file>\n`;
  const events: BrowserEvent[] = [];
  const run = new BrowserRun({ source, revision: "1", file: materializeLiteralFiles(source)[0] }, "session", "hick:///note.md", breakpoints, (e) => events.push(e));
  const drain = () => { let n = 0; while (run.pump() && n++ < 2000) { /* bounded runtime */ } };
  return { run, events, drain };
}
</storycode:copy>
<a id="f-de529cc60d5d-pause"></a>
### `pause`

<storycode:copy id="f-de529cc60d5d-pause">
function pause(events: BrowserEvent[]) { const event = [...events].reverse().find((e) => e.event === "stopped"); if (!event || event.event !== "stopped") throw Error("Expected real pause"); return event; }

</storycode:copy>
<a id="f-de529cc60d5d-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-de529cc60d5d-expression_statement">
describe("browser interpreter", () => {
  it("pauses before the statement, steps into/out of a function and continues to output", () => {
    const { run, events, drain } = start("function price(n) {\n  var total = n * 3;\n  return total;\n}\nvar input = 4;\nvar result = price(input);\nconsole.log(result);", [{ line: 7 }, { line: 8 }]);
    drain();
    expect(pause(events).line).toBe(7);
    expect(pause(events).variables.find((v) => v.name === "input")?.value).toBe("4");
    expect(pause(events).variables.find((v) => v.name === "result")?.value).toBe("undefined");
    run.step("in"); drain();
    expect(pause(events).line).toBe(3);
    expect(pause(events).frames.map((f) => f.name)).toEqual(["price", "<program>"]);
    expect(pause(events).variables.find((v) => v.name === "n")?.value).toBe("4");
    run.step("out"); drain();
    expect(pause(events).line).toBe(8);
    expect(pause(events).variables.find((v) => v.name === "result")?.value).toBe("12");
    run.step("continue"); drain();
    expect(events).toContainEqual({ event: "output", session: "session", text: "12\n", category: "stdout" });
  });
  it.each(["js", "ts"])("matches native execution for edited %s, Unicode, multiline and closures", (language) => {
    const annotation = language === "ts" ? ": number" : "";
    const code = `function outer(n${annotation}) {\n  return function inner(x${annotation}) {\n    var label = 'é🦀';\n    return n +\n      x;\n  };\n}\nvar add = outer(7);\nconsole.log(add(5));`;
    const { events, drain } = start(code, [], language); drain();
    expect(events).toContainEqual({ event: "output", session: "session", text: "12\n", category: "stdout" });
    const native: number[] = [];
    Function("console", code.replaceAll(": number", ""))({ log: (v: number) => native.push(v) });
    expect(native).toEqual([12]);
    const edited = start(code.replace("outer(7)", "outer(9)"), [], language); edited.drain();
    expect(edited.events).toContainEqual({ event: "output", session: "session", text: "14\n", category: "stdout" });
  });
  it("inspects closures and objects without invoking getters or watch calls", () => {
    const { run, events, drain } = start("var touched = 0;\nvar object = { get value() { touched++; return 8; }, nested: { x: 3 } };\nconsole.log(touched);", [{ line: 4 }]); drain();
    run.evaluate("object.nested.x + 2");
    expect(events.at(-1)).toMatchObject({ event: "value", value: "5" });
    expect(() => run.evaluate("object.value")).toThrow("getter");
    expect(() => run.evaluate("console.log(5)")).toThrow("Unsupported");
    const variable = pause(events).variables.find((v) => v.name === "object")!;
    run.children(variable.variables_reference);
    expect(events.at(-1)).toMatchObject({ event: "children", variables: expect.arrayContaining([expect.objectContaining({ name: "value", value: "[Getter — not invoked]" })]) });
    run.evaluate("touched"); expect(events.at(-1)).toMatchObject({ value: "0" });
  });
  it("captures exceptions and denies page, network and indirect eval access", () => {
    for (const code of ["fetch('https://example.com');", "console.log(document.cookie);", "(function(){}).constructor('return 1')();", "throw new Error('broken');"]) {
      const test = start(code); test.drain();
      expect(test.events.at(-1)).toMatchObject({ event: "finished", exit_code: 1 });
    }
  });
  it("refuses unsupported syntax, type errors and dependent weave", () => {
    expect(() => start("var f = () => 1;")).toThrow("Unsupported");
    expect(() => start("async function f() {}", [], "ts")).toThrow("Unsupported");
    expect(() => start("var x: number = 'bad';", [], "ts")).toThrow("Compiler diagnostics");
    expect(() => materializeLiteralFiles('<hick:file path="a.js"><hick:paste select="#x" /></hick:file>')).toThrow("full weave");
  });
  it("bounds an infinite loop and permits pause between batches", () => {
    const { run, events } = start("var n = 0;\nwhile (true) { n++; }");
    expect(run.pump(100)).toBe(true);
    run.pause(); run.pump(); expect(pause(events).reason).toBeDefined();
    run.step("continue");
    // Accrued interpreter steps are the limit, not elapsed wall time. Place
    // the fixture at the boundary instead of spending the test's timeout on
    // two million real steps while other compiler/browser checks run.
    Reflect.set(run, "steps", 1_999_999);
    expect(run.pump(1)).toBe(true);
    expect(run.pump(1)).toBe(false);
    expect(events).toContainEqual(expect.objectContaining({
      event: "failed", message: "Browser execution limit exceeded (2 million interpreter steps)",
    }));
    expect(events.at(-1)).toMatchObject({ event: "finished", exit_code: 1 });
  });
});

</storycode:copy>
<a id="f-de529cc60d5d-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-de529cc60d5d-expression_statement-2">
// Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
it("captures real recursive frame scopes without turning source markers into annotations", () => {
  const source = '<hick:file path="main.js">\n// BREAKPOINT HERE\nfunction recurse(n) {\n  if (n > 0) return recurse(n - 1);\n  return n;\n}\nvar result = recurse(2);\n</hick:file>\n';
  const events: BrowserEvent[] = [];
  const run = new BrowserRun({ source, revision: "1", observationRevision: "captured-digest", file: materializeLiteralFiles(source)[0] }, "recursive", "demo.md", [{ line: 4 }], e => events.push(e));
  while (run.pump()) { /* bounded fixture */ }
  const reading = events.find(e => e.event === "reading");
  expect(reading?.event).toBe("reading");
  if (reading?.event !== "reading") throw Error("missing reading");
  expect(new TextDecoder().decode(new Uint8Array(reading.reading.source.bytes))).toBe(source);
  const frames = reading.reading.annotations.filter(a => a.payload.kind === "frame");
  expect(frames.length).toBeGreaterThan(2);
  expect(new Set(frames.map(a => a.scope.frame)).size).toBe(frames.length);
  expect(reading.reading.annotations.filter(a => a.payload.kind === "paused")).toHaveLength(1);
  expect(reading.reading.unsupported).toContain("reverse execution");
  run.evaluate("n + 1");
  const next = events.filter(e => e.event === "reading").at(-1);
  if (next?.event !== "reading") throw Error("missing expression receipt");
  expect(next.reading.annotations.at(-1)).toMatchObject({ scope: { stop: reading.reading.scope.stop, frame: 1 }, payload: { kind: "expression", expression: "n + 1", value: "1" } });
  run.report("frame", 2);
  const callerReading = events.filter(e => e.event === "reading").at(-1);
  if (callerReading?.event !== "reading") throw Error("missing caller reading");
  expect(callerReading.reading.scope).toMatchObject({ stop: reading.reading.scope.stop, frame: 2 });
  expect(callerReading.reading.annotations.find(a => a.payload.kind === "expression")?.scope.frame).toBe(1);
  expect(callerReading.reading.annotations.find(a => a.payload.kind === "paused")?.scope.frame).toBe(1);
  expect(() => run.step("back")).toThrow();
  expect(() => run.evaluate("recurse(3)")).toThrow();
});

</storycode:copy>
<a id="f-de529cc60d5d-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-de529cc60d5d-expression_statement-3">
// Guarantee: docs/guarantees/execution/browser-output-has-runtime-source-evidence.md
it.each(["js", "ts"])("attributes %s output to emitting code and distinct recursive invocations", (language) => {
  const type = language === "ts" ? ": number" : "";
  const source = `# 🦀\n<hick:file path="main.${language}">\nfunction logger(n${type}) {\n console.write('é');\n console.write('🦀\\r\\nsecond\\n');\n if (n > 0) logger(n-1);\n}\nlogger(2);\nconsole.error('Error: this is text');\ntry { throw new Error('caught'); } catch (e) { console.log('handled'); }\nthrow new Error('unhandled');\n</hick:file>\n`;
  const events: BrowserEvent[] = [];
  const run = new BrowserRun({ source, revision: "editor-1", observationRevision: "a".repeat(64), file: materializeLiteralFiles(source)[0] }, "run-output", "example.md", [], e => events.push(e));
  while (run.pump()) { /* bounded fixture */ }
  const record = events.filter(e => e.event === "runtime").at(-1);
  if (record?.event !== "runtime") throw Error("Missing run evidence");
  expect(record.run.status).toBe("failed");
  expect(record.run.sources[0].revision).toBe("a".repeat(64));
  expect(new TextDecoder().decode(new Uint8Array(record.run.sources[0].bytes))).toBe(source);
  const output = record.run.events.filter(e => e.observation.kind === "output");
  const partial = output.filter(e => e.observation.kind === "output" && new TextDecoder().decode(new Uint8Array(e.observation.bytes)) === "é");
  expect(partial).toHaveLength(3);
  expect(new Set(partial.map(e => e.invocation)).size).toBe(3);
  expect(new Set(record.run.events.map(e => e.id)).size).toBe(record.run.events.length);
  for (const event of partial) {
    expect(event.emission?.precision).toBe("exact");
    const start = event.emission!.bytes!.start;
    expect(new TextDecoder().decode(new Uint8Array(record.run.sources[0].bytes).slice(start))).toMatch(/^console.write/);
    expect(event.callers.length).toBeGreaterThan(0);
  }
  const errors = record.run.events.filter(e => e.observation.kind === "exception");
  expect(errors.some(e => e.observation.kind === "exception" && e.observation.handling === "unhandled")).toBe(true);
  expect(output.some(e => e.observation.kind === "output" && e.observation.category === "error")).toBe(true);
});

</storycode:copy>
<a id="f-de529cc60d5d-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-de529cc60d5d-expression_statement-4">
// Guarantee: docs/guarantees/execution/browser-output-has-runtime-source-evidence.md
it("bounds evidence independently of empty output chunks", () => {
  const source = '<hick:file path="main.js">var i; for(i=0;i<4100;i++) console.write(\'\');</hick:file>';
  const events: BrowserEvent[] = [];
  const run = new BrowserRun({ source, revision: "1", observationRevision: "a".repeat(64), file: materializeLiteralFiles(source)[0] }, "bounded", "example.md", [], e => { if (e.event === "runtime") events.push(e); });
  while (run.pump()) { /* bounded empty writes */ }
  const evidence = events.at(-1);
  if (evidence?.event !== "runtime") throw Error("missing evidence");
  expect(evidence.run.truncated).toBe(true);
  expect(evidence.run.events).toHaveLength(4096);
  expect(evidence.run.status).toBe("completed");
});
</storycode:copy>

### Assemble `apps/web/src/debug/browser/runtime.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/browser/runtime.test.ts"><storycode:paste select="#f-de529cc60d5d-start"/><storycode:paste select="#f-de529cc60d5d-pause"/><storycode:paste select="#f-de529cc60d5d-expression_statement"/><storycode:paste select="#f-de529cc60d5d-expression_statement-2"/><storycode:paste select="#f-de529cc60d5d-expression_statement-3"/><storycode:paste select="#f-de529cc60d5d-expression_statement-4"/></storycode:file>


</storycode:doc>
