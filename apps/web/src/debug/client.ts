// The app's side of the debug channel (0x03).
//
// Everything here speaks in DOCUMENT lines, because the server already mapped
// them: the bridge carries the session API's verbs rather than raw DAP
// precisely so this file never has to know what a virtual file is.
//
// Requests are fire-and-forget and answers arrive as events, which is not
// laziness — a `continue` takes as long as the program does, and a promise
// that resolves in four minutes is a promise nothing can be built on. The UI
// reacts to `stopped` the way it reacts to any other state change.

export const CHANNEL_DEBUG = 0x03;

export interface DebugBreakpoint {
  /** 0-based document line. */
  line: number;
  condition?: string;
  hit_condition?: string;
}

export interface BreakpointStatus {
  line: number;
  /** False when the adapter could not bind it: drawn hollow in the gutter. */
  verified: boolean;
  message?: string;
}

export interface Frame {
  id: number;
  name: string;
  /** 0-based document line, or null for a frame outside the document. */
  line: number | null;
  source: string | null;
  in_document: boolean;
}

export interface Variable {
  name: string;
  value: string;
  type?: string | null;
  variables_reference: number;
}

/** What this adapter can do. Every control is gated on one of these. */
export interface DebugCapabilities {
  conditional_breakpoints: boolean;
  hit_conditional_breakpoints: boolean;
  log_points: boolean;
  set_variable: boolean;
  /** Re-enter the current function from its first line. */
  restart_frame: boolean;
  step_in_targets: boolean;
  /** True reverse execution. Almost nothing has it. */
  step_back: boolean;
  /** Move the instruction pointer within the frame — the other way back. */
  goto_targets: boolean;
  exception_filters: { id: string; label: string }[];
}

export type DebugEvent =
  | { event: "started"; session: string; capabilities: DebugCapabilities; breakpoints: BreakpointStatus[] }
  | {
      event: "stopped";
      session: string;
      reason: string;
      frames: Frame[];
      variables: Variable[];
      line: number | null;
    }
  | { event: "breakpoints"; session: string; breakpoints: BreakpointStatus[] }
  | { event: "value"; session: string; expression: string; value: string; type: string | null; reference: number }
  | { event: "children"; session: string; reference: number; variables: Variable[] }
  | { event: "finished"; session: string }
  | { event: "ended"; session: string }
  | { event: "failed"; session: string | null; message: string };

/**
 * The server's own verbs, spelled exactly as it deserializes them.
 *
 * `back` is true reverse execution (almost nothing has it) and `drop_frame`
 * re-enters the current function. `jump` is not here because it takes a line
 * and is its own request rather than a step.
 */
export type Step = "over" | "in" | "out" | "continue" | "drop_frame" | "back";

/** The raw-frame transport, shared with the LSP channel. */
export interface DebugWire {
  send(frame: Uint8Array): void;
}

export class DebugClient {
  private listeners = new Set<(event: DebugEvent) => void>();

  constructor(private wire: DebugWire) {}

  /** Feed a socket frame in. True when it was ours. */
  handleFrame(frame: Uint8Array): boolean {
    if (frame.length < 2 || frame[0] !== CHANNEL_DEBUG) return false;
    try {
      const event = JSON.parse(new TextDecoder().decode(frame.subarray(1))) as DebugEvent;
      for (const listener of this.listeners) listener(event);
    } catch {
      // A frame we cannot read is dropped rather than thrown: one bad message
      // must not take down a debugging session mid-step.
    }
    return true;
  }

  on(listener: (event: DebugEvent) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private send(request: Record<string, unknown>) {
    const body = new TextEncoder().encode(JSON.stringify(request));
    const frame = new Uint8Array(body.length + 1);
    frame[0] = CHANNEL_DEBUG;
    frame.set(body, 1);
    this.wire.send(frame);
  }

  start(doc: string, breakpoints: DebugBreakpoint[]) {
    this.send({ op: "start", doc, breakpoints });
  }

  setBreakpoints(session: string, breakpoints: DebugBreakpoint[]) {
    this.send({ op: "breakpoints", session, breakpoints });
  }

  state(session: string) {
    this.send({ op: "state", session });
  }

  /**
   * Evaluate an expression.
   *
   * `context` is DAP's own and adapters treat it differently: `hover` for a
   * tooltip, `watch` for something evaluated repeatedly, `repl` for what a
   * person typed — which is the only one where a side effect is their
   * business.
   */
  evaluate(session: string, expression: string, context: "hover" | "watch" | "repl" = "repl", frame?: number) {
    this.send({ op: "eval", session, expression, context, frame });
  }

  step(session: string, how: Step, frame?: number) {
    this.send({ op: "step", session, how, frame });
  }

  /** Move the instruction pointer to a line in the current frame. */
  jump(session: string, line: number) {
    this.send({ op: "jump", session, line });
  }

  runTo(session: string, line: number) {
    this.send({ op: "run_to", session, line });
  }

  children(session: string, reference: number) {
    this.send({ op: "children", session, reference });
  }

  setVariable(session: string, container: number, name: string, value: string) {
    this.send({ op: "set_variable", session, container, name, value });
  }

  stop(session: string) {
    this.send({ op: "stop", session });
  }
}

/**
 * Which backwards control this adapter can offer, if any.
 *
 * Two exist and most adapters have exactly one, so the UI asks this rather
 * than showing both and letting one fail. debugpy has `jump`; js-debug and
 * the JVM adapters have `drop frame`.
 */
export function backwardsControl(
  capabilities: DebugCapabilities | null,
): { kind: "drop_frame" | "jump" | "step_back"; label: string; hint: string } | null {
  if (!capabilities) return null;
  if (capabilities.step_back) {
    return {
      kind: "step_back",
      label: "Step back",
      hint: "Reverse execution: the previous statement, undone.",
    };
  }
  if (capabilities.restart_frame) {
    return {
      kind: "drop_frame",
      label: "Restart frame",
      hint: "Re-enter this function from its first line. It re-runs rather than rewinds, so anything already written stays written.",
    };
  }
  if (capabilities.goto_targets) {
    return {
      kind: "jump",
      label: "Move here",
      hint: "Move the instruction pointer to the selected line in this frame. The lines between run again; anything already written stays written.",
    };
  }
  return null;
}
