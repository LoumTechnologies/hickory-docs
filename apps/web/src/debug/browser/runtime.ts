import Interpreter from "js-interpreter";
import type { IState, IScope, INode } from "js-interpreter";
import type { DebugBreakpoint, DebugEvent, Frame, Step } from "../client";
import { compileProgram } from "./compile";
import type { LiteralFile } from "../../editor/hickLang";
import { Inspector, preview } from "./inspect";

export interface BrowserInput { source: string; revision: string; file: LiteralFile; }
export type BrowserEvent = DebugEvent | { event: "output"; session: string; text: string; category: "stdout" | "stderr" };
export const BROWSER_CAPABILITIES = {
  frame_locals: true,
  conditional_breakpoints: false, hit_conditional_breakpoints: false, log_points: false,
  set_variable: false, restart_frame: false, step_in_targets: false, step_back: false,
  goto_targets: false, exception_filters: [],
};
const statements = new Set(["VariableDeclaration", "ExpressionStatement", "ReturnStatement", "ThrowStatement",
  "IfStatement", "WhileStatement", "DoWhileStatement", "ForStatement", "ForInStatement", "SwitchStatement",
  "BreakStatement", "ContinueStatement", "DebuggerStatement"]);

/** Real interpreter state, not a trace or evaluator of the demonstration. */
export class BrowserRun {
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
    this.interpreter = new Interpreter(this.program.code, (interpreter, global) => {
      const consoleObject = interpreter.createObjectProto(null);
      for (const category of ["log", "warn", "error"]) {
        interpreter.setProperty(consoleObject, category, interpreter.createNativeFunction((...values) => {
          const text = values.map((value) => typeof value === "string" ? value : preview(value)).join(" ") + "\n";
          this.outputBytes += new TextEncoder().encode(text).length;
          if (this.outputBytes > 65_536) throw new Error("Browser output limit exceeded (64 KiB)");
          this.emit({ event: "output", session, text, category: category === "error" ? "stderr" : "stdout" });
          return undefined;
        }));
      }
      interpreter.setProperty(global, "console", consoleObject);
      this.builtins = new Set(Object.keys(global.properties));
    });
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
  stop() { this.ended = true; }
  private calls() { return this.interpreter.stateStack.filter((s) => s.func_?.node && s.doneExec_); }
  /** Returns true while runnable, false while paused/finished. Worker schedules batches. */
  pump(budget = 1500): boolean {
    if (this.paused || this.ended) return false;
    try {
      for (let n = 0; n < budget; n++) {
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
          this.emit({ event: "finished", session: this.session, exit_code: 0 });
          return false;
        }
      }
    } catch (e) {
      this.ended = true;
      const message = e instanceof Error ? e.message : String(e);
      this.emit({ event: "output", session: this.session, category: "stderr", text: message + "\n" });
      this.emit({ event: "failed", session: this.session, message });
      this.emit({ event: "finished", session: this.session, exit_code: 1 });
      return false;
    }
    return true;
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
    this.emit({ event: "stopped", session: this.session, reason, frames,
      selected_frame: selected ?? 1,
      variables: this.inspector.locals(selectedScope, this.builtins), line: frames.find((f) => f.id === selected)?.line ?? frames[0].line });
  }
  evaluate(expression: string, frame?: number) {
    if (!this.paused) throw new Error("Watch expressions require a paused session");
    const scope = this.frameScopes.get(frame ?? 1);
    if (!scope) throw new Error("Unknown frame");
    const variable = this.inspector.variable(expression, this.inspector.evaluate(expression, scope));
    this.emit({ event: "value", session: this.session, expression, value: variable.value, type: variable.type ?? null, reference: variable.variables_reference });
  }
  children(reference: number) {
    if (!this.paused) throw new Error("Values belong to a paused session");
    this.emit({ event: "children", session: this.session, reference, variables: this.inspector.children(reference) });
  }
}
