// The debugger's chrome, dissolved into the block it debugs.
//
// One line above the debugged editor — not a panel below the page. The editor
// carries almost everything (gutter, paused line, inline values, watches,
// inline eval); this strip is only the verbs, the stack as a combo box, and
// the words for what is happening: starting, running, finished, or why a
// session failed to start.
//
// Every control is gated on what the ADAPTER said it can do. A button that is
// present and silently does nothing is worse than one that is absent, and the
// two backwards controls are the sharp case: most adapters have exactly one.

import { useEffect } from "react";
import type { DebugCapabilities, Frame, Step } from "./client";
import { backwardsControl } from "./client";
import type { Watch } from "./useDebugger";

/** The combo box's line for one frame: who, and where it will return to. */
export function frameLabel(frame: Frame): string {
  return frame.in_document && frame.line !== null && frame.line !== undefined
    ? `${frame.name} — line ${frame.line + 1}`
    : `${frame.name} — external`;
}

/**
 * The step a function key asks for, or null when the key is not ours.
 *
 * The keys the old panel's tooltips promised and nothing bound: F5 continue,
 * F10 over, F11 in, Shift-F11 out. The caller guards WHEN these apply — only
 * while paused, never with a modifier held (Ctrl-F5 stays the browser's hard
 * reload), never on an event something else already handled.
 */
export function stepForKey(key: string, shift: boolean): Step | null {
  if (key === "F5" && !shift) return "continue";
  if (key === "F10" && !shift) return "over";
  if (key === "F11") return shift ? "out" : "in";
  return null;
}

/** Whether the function keys should act at all right now. */
export function debugKeysActive(status: DebugStripProps["status"]): boolean {
  // Only while paused: that is the only time a step means anything, and it
  // keeps F5 as the browser's own reload whenever no program is stopped.
  return status === "paused";
}

/**
 * One control: a glyph to press, a name for a screen reader, a sentence on
 * hover. The sentence says what the button DOES rather than repeating its
 * name — "Step over" twice helps nobody.
 */
function IconButton({
  glyph,
  label,
  hint,
  disabled,
  onClick,
  className,
}: {
  glyph: string;
  label: string;
  hint: string;
  disabled?: boolean;
  onClick: () => void;
  className?: string;
}) {
  return (
    <button
      type="button"
      className={`debug-icon${className ? ` ${className}` : ""}`}
      aria-label={label}
      data-tip={`${label} — ${hint}`}
      disabled={disabled}
      onClick={onClick}
    >
      <span aria-hidden="true">{glyph}</span>
    </button>
  );
}

export interface DebugStripProps {
  status: "idle" | "starting" | "paused" | "running" | "finished" | "failed";
  /** Which generated file this session is running. */
  program?: string | null;
  message: string | null;
  capabilities: DebugCapabilities | null;
  frames: Frame[];
  selectedFrame: number | null;
  watches: Watch[];
  /** The program's exit code once it finished, when the adapter said. */
  exitCode?: number | null;
  onSelectFrame: (id: number) => void;
  onStep: (how: Step) => void;
  /** Move the instruction pointer to the caret's line, where supported. */
  onJumpHere: () => void;
  /** Start a new session over the same file — the re-run after "finished". */
  onStart: () => void;
  onStop: () => void;
  /** Ask the person for an expression to watch (the strip's "+ watch"). */
  onAddWatch: () => void;
  onRemoveWatch: (expression: string) => void;
}

const STATUS_WORD: Record<DebugStripProps["status"], string> = {
  idle: "",
  starting: "starting…",
  running: "running…",
  paused: "paused",
  finished: "finished",
  failed: "failed",
};

/**
 * The status word, with the exit info a finished program earned. The session
 * behind it is already gone — reaped server-side the moment the program ended
 * — so this line and the local breakpoints are all that remains of it.
 */
export function statusWord(status: DebugStripProps["status"], exitCode?: number | null): string {
  if (status !== "finished") return STATUS_WORD[status];
  return exitCode === null || exitCode === undefined
    ? "finished"
    : `finished — exit code ${exitCode}`;
}

export function DebugStrip(props: DebugStripProps) {
  const paused = props.status === "paused";
  // A terminal state: the session no longer exists, and the strip's job is
  // to say how it ended and offer the way back in.
  const over = props.status === "finished" || props.status === "failed";
  const back = backwardsControl(props.capabilities);
  const { onStep, status } = props;

  // The keys the tooltips promise. Bound only while a step can act, so the
  // browser keeps its own F5 the rest of the time.
  useEffect(() => {
    if (!debugKeysActive(status)) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.defaultPrevented || event.metaKey || event.ctrlKey || event.altKey) return;
      const how = stepForKey(event.key, event.shiftKey);
      if (!how) return;
      event.preventDefault();
      onStep(how);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [status, onStep]);

  // Idle means no session and nothing to say: the block's Debug chip is the
  // way in, and chrome for a debugger nobody started is noise.
  if (props.status === "idle") return null;

  return (
    <div className="debug-strip" role="toolbar" aria-label="Debugger">
      <span className={`debug-strip__dot debug-strip__dot--${props.status}`} aria-hidden="true" />
      <span className="debug-strip__status" role="status">
        {props.program ? `${props.program} · ` : ""}
        {statusWord(props.status, props.exitCode)}
      </span>
      {/* The way back in, once the session is over: the server reaped it
          with the program, so the only meaningful verb left is a fresh
          start — with the breakpoints kept locally for exactly this. */}
      {over && (
        <IconButton
          glyph="▶"
          label="Debug again"
          hint="Start a new session over the same file. Your breakpoints are kept."
          onClick={props.onStart}
        />
      )}
      {/* The stepping verbs, only while there is a program to step: after
          the end there is nothing they could act on, and a row of disabled
          buttons next to "finished" reads as a session that still exists. */}
      {!over && (
        <IconButton
          glyph="▶"
          label="Continue"
          hint="Run until the next breakpoint (F5)"
          disabled={!paused}
          onClick={() => props.onStep("continue")}
        />
      )}
      {!over && (
        <IconButton
          glyph="⤼"
          label="Step over"
          hint="Run this line, and stop on the next one in this function (F10)"
          disabled={!paused}
          onClick={() => props.onStep("over")}
        />
      )}
      {!over && (
        <IconButton
          glyph="↓"
          label="Step into"
          hint="Stop at the first line of the call on this line (F11)"
          disabled={!paused}
          onClick={() => props.onStep("in")}
        />
      )}
      {!over && (
        <IconButton
          glyph="↑"
          label="Step out"
          hint="Run to the end of this function and stop where it returns (Shift-F11)"
          disabled={!paused}
          onClick={() => props.onStep("out")}
        />
      )}
      {/* The backwards control, whichever one this adapter has. Where it has
          neither, nothing is shown rather than something disabled with no
          explanation. */}
      {!over && back && (
        <IconButton
          glyph={back.kind === "step_back" ? "◀" : back.kind === "drop_frame" ? "↺" : "⤒"}
          label={back.label}
          hint={back.hint}
          disabled={!paused}
          onClick={() => {
            // `jump` needs a target line and is not a step; the other two are
            // the server's own verbs.
            if (back.kind === "jump") props.onJumpHere();
            else props.onStep(back.kind === "step_back" ? "back" : "drop_frame");
          }}
        />
      )}
      {/* While a session lives, this ends it; once it is over there is
          nothing left to end — the server reaped it — so the same slot
          clears the strip instead. */}
      <IconButton
        glyph={over ? "✕" : "■"}
        label={over ? "Dismiss" : "Stop"}
        hint={
          over
            ? "Clear this. The session is already over and its scratch copy is deleted."
            : "End the session. The scratch copy it ran in is deleted with it."
        }
        className="debug-strip__stop"
        onClick={props.onStop}
      />
      {/* The stack, as a combo box: pick a frame, and the gutter marks, the
          paused line and the inline values follow it. */}
      {props.frames.length > 0 && (
        <select
          className="debug-strip__stack"
          aria-label="Call stack"
          value={props.selectedFrame ?? ""}
          onChange={(event) => props.onSelectFrame(Number(event.target.value))}
        >
          {props.frames.map((frame) => (
            <option key={frame.id} value={frame.id}>
              {frameLabel(frame)}
            </option>
          ))}
        </select>
      )}
      <button
        type="button"
        className="debug-strip__watch-add"
        onClick={props.onAddWatch}
        data-tip="Watch an expression — its value appears at the end of the line that mentions it"
      >
        + watch
      </button>
      {/* The watched expressions, each removable here. Their VALUES live in
          the editor, faded at the end of the lines that mention them. */}
      {props.watches.map((watch) => (
        <span key={watch.expression} className="debug-strip__watch mono">
          {watch.expression}
          <button
            type="button"
            aria-label={`Stop watching ${watch.expression}`}
            data-tip={`Stop watching ${watch.expression}`}
            onClick={() => props.onRemoveWatch(watch.expression)}
          >
            ×
          </button>
        </span>
      ))}
      {/* The failure, in the one place a session's chrome is. Its own element
          rather than appended to the status word, which read as one unbroken
          sentence of two different weights. */}
      {props.message && (
        <span className="debug-strip__error" role="alert" data-tip={props.message}>
          {props.message}
        </span>
      )}
    </div>
  );
}
