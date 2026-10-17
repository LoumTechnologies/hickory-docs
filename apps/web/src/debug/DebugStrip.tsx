import { MoreActions } from "../components/MoreActions";
// The debugger's chrome, dissolved into the block it debugs.
//
// Controls follow the paused line in the right gutter. The editor
// carries almost everything (gutter, paused line, inline values, watches,
// inline eval); this strip is only the verbs, the stack as a combo box, and
// the words for what is happening: starting, running, finished, or why a
// session failed to start.
//
// Every control is gated on what the ADAPTER said it can do. A button that is
// present and silently does nothing is worse than one that is absent, and the
// two backwards controls are the sharp case: most adapters have exactly one.

import { DebugGutterControls, debugViewFocused } from "./DebugRail";
import type { EditorView } from "@codemirror/view";
import { PaneHeaderContent } from "../shell/PaneHeader";
import { bindingOf, formatBinding, isAction } from "../lib/keymap";
import { useEffect, useRef, useState, type ReactNode } from "react";
import type { TranscriptEvent } from "../api/types";
import { WatchingTerminal } from "../terminal/WatchingTerminal";
import type { DebugCapabilities, Frame, Step } from "./client";
import { backwardsControl } from "./client";
import type { Watch } from "./useDebugger";

/**
 * The combo box's line for one frame: who, and where it will return to.
 *
 * A frame in another file of the folder is named by that file and line —
 * it is one the app can open — and only a frame outside the folder (the
 * standard library, an installed package) is "external".
 */
export function frameLabel(frame: Frame): string {
  if (frame.in_document && frame.line !== null && frame.line !== undefined) {
    return `${frame.name} — line ${frame.line + 1}`;
  }
  const inFolder = !!frame.source && !/^([A-Za-z]:)?[\\/]/.test(frame.source);
  if (inFolder && typeof frame.source_line === "number") {
    return `${frame.name} — ${frame.source}:${frame.source_line + 1}`;
  }
  return `${frame.name} — external`;
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

// Missing catalogue adapters are a prerequisite of Debug. Share downloads and
// resume only the request whose setup is still on screen.
const installations = new Map<string, Promise<void>>();
function MissingTool({ offer, detail, onInstall, onReady, attempted }: {
  offer: { kind: string; language: string };
  detail: string | null;
  onInstall?: (offer: { kind: string; language: string }) => Promise<void>;
  onReady: () => void;
  attempted: Set<string>;
}) {
  const [busy, setBusy] = useState(true);
  const [failed, setFailed] = useState<string | null>(null);
  const callbacks = useRef({ onInstall, onReady });
  callbacks.current = { onInstall, onReady };
  const active = useRef(false);
  const attempt = useRef(0);
  const run = () => {
    const install = callbacks.current.onInstall;
    if (!install) { setBusy(false); return; }
    const id = ++attempt.current;
    setBusy(true); setFailed(null);
    const key = `${offer.kind}:${offer.language}`;
    let pending = installations.get(key);
    if (!pending) {
      pending = Promise.resolve().then(() => install(offer));
      installations.set(key, pending);
      const remove = () => { if (installations.get(key) === pending) installations.delete(key); };
      void pending.then(remove, remove);
    }
    void pending.then(() => {
      if (active.current && attempt.current === id) callbacks.current.onReady();
    }, e => {
      if (active.current && attempt.current === id) setFailed(e instanceof Error ? e.message : String(e));
    }).finally(() => {
      if (active.current && attempt.current === id) setBusy(false);
    });
  };
  useEffect(() => {
    active.current = true;
    const key = `${offer.kind}:${offer.language}`;
    if (attempted.has(key) && !installations.has(key)) {
      setBusy(false);
      setFailed("The debugger is still unavailable. Retry installation to try again.");
    } else {
      attempted.add(key);
      run();
    }
    return () => { active.current = false; attempt.current++; };
  }, [offer.kind, offer.language]);
  const pretty = offer.language.charAt(0).toUpperCase() + offer.language.slice(1);
  return <span className="debug-strip__missing" role={failed ? "alert" : "status"}>
    <span data-tip={detail ?? undefined}>{busy ? `Installing ${pretty} debugger…` : failed ? `${pretty} debugger installation failed.` : `No ${pretty} debugger on this machine.`}</span>
    {failed && <><span className="debug-strip__error">{failed}</span>
      <button type="button" className="btn debug-strip__install" onClick={run}>Retry installation</button></>}
  </span>;
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
  locationActions?: ReactNode;
  editorView?: EditorView | null;
  /** Zero-based line in the displayed editor. */
  pausedLine?: number | null;
  status: "idle" | "starting" | "paused" | "running" | "finished" | "failed";
  /** Which generated file this session is running. */
  program?: string | null;
  message: string | null;
  /** A tool this machine can fetch, when that is why the session failed. */
  offerInstall?: { kind: string; language: string } | null;
  /** Install the missing adapter. The strip restarts only while still mounted. */
  onInstall?: (offer: { kind: string; language: string }) => Promise<void>;
  capabilities: DebugCapabilities | null;
  frames: Frame[];
  selectedFrame: number | null;
  watches: Watch[];
  /** The program's exit code once it finished, when the adapter said. */
  exitCode?: number | null;
  /**
   * The build that ran before there was a program to launch, when one did.
   *
   * Empty for every language whose generated file IS the program. Shown as a
   * terminal rather than summarised by a spinner: a build that fails says
   * why in its own output — MSBuild's errors, the missing package, the
   * syntax error on line 12 — and `Building…` throws all of that away and
   * replaces it with the one fact the person already knew.
   */
  buildOutput?: TranscriptEvent[];
  onSelectFrame: (id: number) => void;
  onStep: (how: Step) => void;
  /** Move the instruction pointer to the caret's line, where supported. */
  onJumpHere: () => void;
  /** Start a new session over the same file — the re-run after "finished". */
  onStart: () => void;
  onStop: () => void;
  /** Ask the person for an expression to watch (the strip's "+ watch"). */
  onAddWatch: () => void;
  /** The adapter's exception filters that are switched on, by id. */
  exceptionFilters: string[];
  onToggleExceptionFilter: (id: string) => void;
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
  const attemptedInstallations = useRef(new Set<string>());
  useEffect(() => {
    if (["idle", "paused", "running", "finished"].includes(props.status)) attemptedInstallations.current.clear();
  }, [props.status]);
  const paused = props.status === "paused";
  // A terminal state: the session no longer exists, and the strip's job is
  // to say how it ended and offer the way back in.
  const over = props.status === "finished" || props.status === "failed";
  const back = backwardsControl(props.capabilities);
  const { onStep, status } = props;

  // The keys the tooltips promise. Bound only while a step can act, so the
  // browser keeps its own F5 the rest of the time.
  useEffect(() => {
    if (!["paused", "running", "starting"].includes(status)) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.defaultPrevented || (props.editorView && !debugViewFocused(props.editorView))) return;
      if (isAction(event, "debug.stop")) { event.preventDefault(); props.onStop(); return; }
      if (!debugKeysActive(status)) return;
      const how = isAction(event, "debug.continue") ? "continue"
        : isAction(event, "debug.over") ? "over" : isAction(event, "debug.in") ? "in" : isAction(event, "debug.out") ? "out" : null;
      if (!how) return;
      event.preventDefault();
      onStep(how);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [status, onStep, props.editorView, props.onStop]);

  // Idle means no session and nothing to say: the script's gutter icon is the
  // way in, and chrome for a debugger nobody started is noise.
  if (props.status === "idle") return null;

  // Shown while the build is the thing happening, and kept when the start
  // failed — which is the case it exists for. Once the program is running,
  // a successful build's log is a wall of text between a person and the
  // frame they are looking at.
  const build = props.buildOutput ?? [];
  const showBuild =
    build.length > 0 && (props.status === "starting" || props.status === "failed");

  const controls = (
    <div className="debug-strip" role="toolbar" aria-label="Debugger">
      <span className={`debug-strip__dot debug-strip__dot--${props.status}`} aria-hidden="true" />
      <span className="debug-strip__status" role="status">
        {props.program ? `${props.program} · ` : ""}
        {props.offerInstall ? "Debugger setup" : statusWord(props.status, props.exitCode)}
      </span>
      {/* The way back in, once the session is over: the server reaped it
          with the program, so the only meaningful verb left is a fresh
          start — with the breakpoints kept locally for exactly this. */}
      {over && !props.offerInstall && (
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
      {!over && paused && (
        <IconButton
          glyph="▶"
          label="Continue"
          hint={`Run until the next breakpoint (${formatBinding(bindingOf("debug.continue"))})`}
          disabled={!paused}
          onClick={() => props.onStep("continue")}
        />
      )}
      {!over && paused && (
        <IconButton
          glyph="⤼"
          label="Step over"
          hint={`Run this line and stop on the next one (${formatBinding(bindingOf("debug.over"))})`}
          disabled={!paused}
          onClick={() => props.onStep("over")}
        />
      )}
      {!over && paused && (
        <IconButton
          glyph="↓"
          label="Step into"
          hint={`Stop inside this call (${formatBinding(bindingOf("debug.in"))})`}
          disabled={!paused}
          onClick={() => props.onStep("in")}
        />
      )}
      {!over && paused && (
        <IconButton
          glyph="↑"
          label="Step out"
          hint={`Stop where this function returns (${formatBinding(bindingOf("debug.out"))})`}
          disabled={!paused}
          onClick={() => props.onStep("out")}
        />
      )}
      {/* The backwards control, whichever one this adapter has. Where it has
          neither, nothing is shown rather than something disabled with no
          explanation. */}
      {!over && paused && back && (
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
      {props.locationActions}
      {/* The stack, as a combo box: pick a frame, and the gutter marks, the
          paused line and the inline values follow it. */}
      <MoreActions label="Debug options">
      <span>{props.program ? `${props.program} · ` : ""}{statusWord(props.status, props.exitCode)}</span>
      {!over && <>
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
      {/* The adapter's OWN exception filters. Never a list written here:
          Python offers "raised" and "uncaught", a JVM offers caught and
          uncaught, and an adapter with none draws nothing. */}
      {(props.capabilities?.exception_filters ?? []).map((filter) => {
        const on = props.exceptionFilters.includes(filter.id);
        return (
          <button
            key={filter.id}
            type="button"
            role="switch"
            aria-checked={on}
            className={`debug-strip__exception${on ? " on" : ""}`}
            data-tip={`Stop when an exception is ${filter.label.toLowerCase()}`}
            onClick={() => props.onToggleExceptionFilter(filter.id)}
          >
            {filter.label}
          </button>
        );
      })}
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
      </>}
      </MoreActions>
      {/* The failure, in the one place a session's chrome is. Its own element
          rather than appended to the status word, which read as one unbroken
          sentence of two different weights.

          When the failure is a missing tool this machine can fetch, the
          button IS the message: printing "install one with `hick dap install
          python`" next to a control that does exactly that is telling
          somebody to go and do by hand what is in front of them. */}
      {props.message && !props.offerInstall && (
        <span className="debug-strip__error" role="alert" data-tip={props.message}>
          {props.message}
        </span>
      )}
      {props.offerInstall && (
        <MissingTool
          offer={props.offerInstall}
          detail={props.message}
          onInstall={props.onInstall}
          onReady={props.onStart}
          attempted={attemptedInstallations.current}
        />
      )}
    </div>
  );
  return <>
    {props.editorView ? <DebugGutterControls view={props.editorView} line={props.pausedLine} program={props.program}>{controls}</DebugGutterControls> : <PaneHeaderContent part="actions">{controls}</PaneHeaderContent>}
    {showBuild && (
      <div className="debug-build" data-testid="debug-build">
        <div className="debug-build__title">
          the build, as it ran — this is not a transcript and nothing is
          verified against it
        </div>
        <WatchingTerminal events={build} live={props.status === "starting"} />
      </div>
    )}
    </>;
}
