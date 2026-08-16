// The debugger's controls, stack and variables.
//
// Deliberately small, because the editor carries most of the debugger: the
// gutter shows the breakpoints, the paused line shows where you are, and the
// inline values show what things hold. This is what is left — the verbs, the
// frames you can switch to, and a box to ask a question in.
//
// Every control is gated on what the ADAPTER said it can do. A button that is
// present and silently does nothing is worse than one that is absent, and the
// two backwards controls are the sharp case: most adapters have exactly one.

import { useState } from "react";
import type { DebugCapabilities, Frame, Step, Variable } from "./client";
import { backwardsControl } from "./client";

export interface DebugPanelProps {
  status: "idle" | "starting" | "paused" | "running" | "finished" | "failed";
  /** Which generated file this session is running. */
  program?: string | null;
  message: string | null;
  capabilities: DebugCapabilities | null;
  frames: Frame[];
  variables: Variable[];
  /** The frame whose variables are shown. */
  selectedFrame: number | null;
  onSelectFrame: (id: number) => void;
  onStep: (how: Step) => void;
  /** Move the instruction pointer to the caret's line, where supported. */
  onJumpHere: () => void;
  onEvaluate: (expression: string) => void;
  /** The last expression answered, shown under the box. */
  lastValue: { expression: string; value: string; type: string | null } | null;
  onStart: (program?: string) => void;
  onStop: () => void;
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
      title={`${label} — ${hint}`}
      disabled={disabled}
      onClick={onClick}
    >
      <span aria-hidden="true">{glyph}</span>
    </button>
  );
}

export function DebugPanel(props: DebugPanelProps) {
  const [expression, setExpression] = useState("");
  const paused = props.status === "paused";
  const back = backwardsControl(props.capabilities);

  return (
    <aside className="debug-panel" aria-label="Debugger">
      <header className="debug-panel__bar">
        {props.status === "idle" || props.status === "finished" || props.status === "failed" ? (
          <button type="button" onClick={() => props.onStart()} className="debug-panel__go">
            Debug this document
          </button>
        ) : (
          <>
            {/* Icons, because these five are pressed constantly and read
                once. Every one carries its name and its key in the tooltip:
                an icon nobody can name is a puzzle, and a puzzle in a
                debugger is worse than a word. */}
            <IconButton
              glyph="▶"
              label="Continue"
              hint="Run until the next breakpoint (F5)"
              disabled={!paused}
              onClick={() => props.onStep("continue")}
            />
            <IconButton
              glyph="⤼"
              label="Step over"
              hint="Run this line, and stop on the next one in this function (F10)"
              disabled={!paused}
              onClick={() => props.onStep("over")}
            />
            <IconButton
              glyph="↓"
              label="Step into"
              hint="Stop at the first line of the call on this line (F11)"
              disabled={!paused}
              onClick={() => props.onStep("in")}
            />
            <IconButton
              glyph="↑"
              label="Step out"
              hint="Run to the end of this function and stop where it returns (Shift-F11)"
              disabled={!paused}
              onClick={() => props.onStep("out")}
            />
            {/* The backwards control, whichever one this adapter has. Where
                it has neither, nothing is shown rather than something
                disabled with no explanation. */}
            {back && (
              <IconButton
                glyph={back.kind === "step_back" ? "◀" : back.kind === "drop_frame" ? "↺" : "⤒"}
                label={back.label}
                hint={back.hint}
                disabled={!paused}
                onClick={() => {
                  // `jump` needs a target line and is not a step; the other
                  // two are the server's own verbs.
                  if (back.kind === "jump") props.onJumpHere();
                  else props.onStep(back.kind === "step_back" ? "back" : "drop_frame");
                }}
              />
            )}
            <IconButton
              glyph="■"
              label="Stop"
              hint="End the session. The scratch copy it ran in is deleted with it."
              className="debug-panel__stop"
              onClick={props.onStop}
            />
          </>
        )}
        <span className="debug-panel__status" role="status">
          {props.program && props.status !== "idle" ? `${props.program} · ` : ""}
          {props.status === "starting" && "starting…"}
          {props.status === "running" && "running…"}
          {props.status === "paused" && "paused"}
          {props.status === "finished" &&
            "the program ran to the end — set a breakpoint in the gutter and debug again"}
        </span>
      </header>

      {/* Its own row, because it is a sentence. Appended to the status word it
          read as "pausedthe debug adapter refused…", which is both unreadable
          and the wrong weight: the status is a word, this is an explanation. */}
      {props.message && (
        <p className="debug-panel__error" role="alert">
          {props.message}
        </p>
      )}

      {/* Why the editor has gone quiet. Values belong to a frame of a live
          process: when it ends there is nothing left to ask, and saying so
          beats hovering a variable and getting only its type back. */}
      {props.status === "finished" && (
        <p className="debug-panel__hint">
          Nothing is paused, so there are no values to show. Click the gutter beside a line to
          leave a red dot, then debug again — values appear inline and in hovers while the program
          is stopped there.
        </p>
      )}
      {props.status === "idle" && (
        <p className="debug-panel__hint">
          Click the gutter to the left of a line of code to set a breakpoint, then start.
        </p>
      )}

      {props.frames.length > 0 && (
        <div className="debug-panel__body">
          <section className="debug-panel__frames" aria-label="Call stack">
            <h4>Stack</h4>
            <ul>
              {props.frames.map((frame) => (
                <li key={frame.id}>
                  <button
                    type="button"
                    aria-pressed={props.selectedFrame === frame.id}
                    className={frame.in_document ? "" : "debug-panel__foreign"}
                    onClick={() => props.onSelectFrame(frame.id)}
                    // A frame outside the document is shown as what it is
                    // rather than given a line it does not have here.
                    title={frame.in_document ? `line ${(frame.line ?? 0) + 1}` : (frame.source ?? "outside this document")}
                  >
                    {frame.name}
                    {frame.in_document ? (
                      <span className="debug-panel__line"> line {(frame.line ?? 0) + 1}</span>
                    ) : (
                      <span className="debug-panel__line"> external</span>
                    )}
                  </button>
                </li>
              ))}
            </ul>
          </section>

          <section className="debug-panel__vars" aria-label="Variables">
            <h4>Variables</h4>
            <ul>
              {props.variables.map((variable) => (
                <li key={variable.name}>
                  <span className="debug-panel__name">{variable.name}</span>
                  <span className="debug-panel__value">{variable.value}</span>
                  {variable.type && <span className="debug-panel__type">{variable.type}</span>}
                </li>
              ))}
            </ul>
          </section>
        </div>
      )}

      <form
        className="debug-panel__eval"
        onSubmit={(event) => {
          event.preventDefault();
          if (expression.trim()) props.onEvaluate(expression);
        }}
      >
        <input
          value={expression}
          onChange={(event) => setExpression(event.target.value)}
          placeholder={paused ? "Evaluate in this frame…" : "Evaluate (paused only)"}
          disabled={!paused}
          aria-label="Evaluate an expression in the selected frame"
        />
        <button type="submit" disabled={!paused}>
          Evaluate
        </button>
      </form>

      {props.lastValue && (
        <p className="debug-panel__result">
          <code>{props.lastValue.expression}</code> = <strong>{props.lastValue.value}</strong>
          {props.lastValue.type && <span className="debug-panel__type"> {props.lastValue.type}</span>}
        </p>
      )}
    </aside>
  );
}
