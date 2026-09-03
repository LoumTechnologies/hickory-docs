import type { TranscriptEvent } from "../api/types";

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
  /** Stop only when this is true, in the debuggee's own language. */
  condition?: string;
  /** Stop only on the Nth hit (`">5"`, `"%3"` — the adapter's own syntax). */
  hit_condition?: string;
  /**
   * Log this and continue, instead of stopping. JetBrains and VS Code both
   * spell it as a breakpoint that does not break; the server has carried it
   * since `hick-dap`'s `Breakpoint` was written, and `{expr}` inside it is
   * interpolated by the adapter.
   */
  log_message?: string;
}

export interface BreakpointStatus {
  line: number;
  /** False when the adapter could not bind it: drawn hollow in the gutter. */
  /** Whether the adapter has bound this line. Three answers, not two —
   * see `BindState` in `crates/hick-dap/src/session.rs` for why: an adapter
   * that has not confirmed a breakpoint yet has not refused it. */
  state: BindState;
  message?: string;
  /**
   * The line the adapter actually bound it to, when that is not the one asked
   * for. Adapters slide a breakpoint down to the next line that can hold one;
   * keeping the requested line makes the gutter disagree with where the
   * program stops.
   */
  moved_to?: number;
}

export interface Frame {
  id: number;
  name: string;
  /** 0-based document line, or null for a frame outside the document. */
  line: number | null;
  /**
   * The frame's file: root-relative when it is a file in the folder the app
   * opened (which the workspace can open as a tab), else as the adapter
   * named it.
   */
  source: string | null;
  /** 0-based line in `source`, as the adapter reported it — for opening a
   * frame that is in another file of the project. */
  source_line?: number | null;
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

/**
 * `bound` — the adapter confirmed it; the program will stop here.
 * `pending` — not confirmed yet. Normal for a compiled language, where
 *   nothing binds until the module loads.
 * `refused` — there is no code on this line at all. The only certain
 *   refusal, and hick makes it itself before any adapter is asked.
 */
export type BindState = "bound" | "pending" | "refused";

export type DebugEvent =
  | {
      event: "started";
      session: string;
      /** The file this session runs, exactly as `start` named it. Every
       * plain-file pane shares one socket, and this is how each tells its
       * own session from a neighbour's. */
      doc?: string;
      capabilities: DebugCapabilities;
      breakpoints: BreakpointStatus[];
    }
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
  | {
      /**
       * The program ran to its end. The server reaps the session with it —
       * the id in here no longer answers — so the client must send nothing
       * more to it; breakpoints and watches are kept locally for the next run.
       */
      event: "finished";
      session: string;
      /** The debuggee's exit code, when the adapter reported one. */
      exit_code?: number | null;
    }
  | { event: "ended"; session: string }
  // A build that had to happen before there was a program to launch. Its
  // events are `TranscriptEvent`-shaped so the watching terminal renders
  // them unchanged — a build tool's colour and its rewritten progress lines
  // are the reason that is a terminal and not a text card. It is not a
  // transcript: nothing here is recorded, woven, or compared.
  | { event: "build"; doc?: string; events: TranscriptEvent[] }
  | {
      event: "failed";
      session: string | null;
      /** The file a failed START was for, when the failure has no session
       * to name instead. */
      doc?: string;
      message: string;
      /** Which request failed, so its message can be shown where it belongs. */
      about?: string;
      /** The document lines it was about, for a breakpoint failure. */
      lines?: number[];
      /**
       * A missing tool this machine can fetch — the one debug failure a
       * person can fix from here. Present only when a catalogue can serve it,
       * so the app never offers a button for a language whose adapter comes
       * from its own ecosystem.
       */
      offer_install?: { kind: string; language: string };
    };

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

  start(doc: string, breakpoints: DebugBreakpoint[], program?: string) {
    // `program` names which generated file to run. A document with two
    // Python files has two answers, and picking the first is right only by
    // accident.
    this.send({ op: "start", doc, breakpoints, program });
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
