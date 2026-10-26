<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/DebugStrip.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-09306f4ec1a5"></a>
## `apps/web/src/debug/DebugStrip.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Launch the artifact built from the selected compiled project](../../../../../guarantees/debugging/a-compiled-language-launches-what-a-build-produced/contract.md), [Install a supported missing debugger and resume the requested launch](../../../../../guarantees/debugging/a-missing-debugger-is-a-button/contract.md), [An Exception Breakpoint Uses The Adapter's Own Filters](../../../../../guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters/contract.md), [Choose a launch without replacing the editor being read](../../../../../guarantees/debugging/debug-controls-preserve-editor-space/contract.md), [Every Shortcut Is A Setting](../../../../../guarantees/editor-intelligence/every-shortcut-is-a-setting/contract.md).

<a id="f-09306f4ec1a5-frameLabel"></a>
### `frameLabel`

<storycode:copy id="f-09306f4ec1a5-frameLabel">
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";
import { ObservationReading } from "./ObservationReading";
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
import { useEffect, useId, useRef, useState, type ReactNode } from "react";
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

</storycode:copy>
<a id="f-09306f4ec1a5-stepForKey"></a>
### `stepForKey`

The step a function key asks for, or null when the key is not ours.
The keys the old panel's tooltips promised and nothing bound: F5 continue,
F10 over, F11 in, Shift-F11 out. The caller guards WHEN these apply — only
while paused, never with a modifier held (Ctrl-F5 stays the browser's hard
reload), never on an event something else already handled.

<storycode:copy id="f-09306f4ec1a5-stepForKey">
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

</storycode:copy>
<a id="f-09306f4ec1a5-debugKeysActive"></a>
### `debugKeysActive`

Whether the function keys should act at all right now.

<storycode:copy id="f-09306f4ec1a5-debugKeysActive">
/** Whether the function keys should act at all right now. */
export function debugKeysActive(status: DebugStripProps["status"]): boolean {
  // Only while paused: that is the only time a step means anything, and it
  // keeps F5 as the browser's own reload whenever no program is stopped.
  return status === "paused";
}

</storycode:copy>
<a id="f-09306f4ec1a5-installations"></a>
### `installations`

Missing catalogue adapters are a prerequisite of Debug. Share downloads and
resume only the request whose setup is still on screen.

<storycode:copy id="f-09306f4ec1a5-installations">
// Missing catalogue adapters are a prerequisite of Debug. Share downloads and
// resume only the request whose setup is still on screen.
const installations = new Map<string, Promise<void>>();
</storycode:copy>
<a id="f-09306f4ec1a5-setupEditors"></a>
### `setupEditors`

<storycode:copy id="f-09306f4ec1a5-setupEditors">
const setupEditors = new WeakMap<object, string>();
</storycode:copy>
<a id="f-09306f4ec1a5-setupEditorId"></a>
### `setupEditorId`

<storycode:copy id="f-09306f4ec1a5-setupEditorId">
function setupEditorId(view?: EditorView | null) { if (!view) return null; let id = setupEditors.get(view); if (!id) { id = crypto.randomUUID(); setupEditors.set(view, id); } return id; }
</storycode:copy>
<a id="f-09306f4ec1a5-MissingTool"></a>
### `MissingTool`

<storycode:copy id="f-09306f4ec1a5-MissingTool">
function MissingTool({ offer, detail, onInstall, onReady, attempted, sourceIdentity }: {
  offer: { kind: string; language: string };
  detail: string | null;
  onInstall?: (offer: { kind: string; language: string }) => Promise<void>;
  onReady: () => void;
  attempted: Set<string>;
  sourceIdentity: () => string;
}) {
  const parent = `debugger-setup:${useId()}`;
  const [busy, setBusy] = useState(true);
  const [failed, setFailed] = useState<string | null>(null);
  const callbacks = useRef({ onInstall, onReady });
  callbacks.current = { onInstall, onReady };
  const active = useRef(false);
  const attempt = useRef(0);
  const inFlight = useRef(false);
  const currentOffer = useRef(offer); currentOffer.current = offer;
  const currentSource = useRef(sourceIdentity); currentSource.current = sourceIdentity;
  const run = async (signal?: AbortSignal): Promise<UiOutcome> => {
    if (inFlight.current || signal?.aborted) return { status: "unavailable", reason: "Debugger setup is already pending or this request was cancelled" };
    const install = callbacks.current.onInstall;
    if (!install) { setBusy(false); return { status: "unavailable", reason: "This debugger has no automatic installation handler" }; }
    const capturedSource = currentSource.current();
    const id = ++attempt.current;
    inFlight.current = true; setBusy(true); setFailed(null);
    const key = `${offer.kind}:${offer.language}`;
    let pending = installations.get(key);
    if (!pending) {
      pending = Promise.resolve().then(() => install(offer));
      installations.set(key, pending);
      const remove = () => { if (installations.get(key) === pending) installations.delete(key); };
      void pending.then(remove, remove);
    }
    try {
      await pending;
      if (!active.current || attempt.current !== id || signal?.aborted || currentSource.current() !== capturedSource) {
        const reason = "Installation may have completed, but its source/setup owner was dismissed, changed or cancelled; no debugger restart was requested";
        if (active.current && attempt.current === id) setFailed(reason);
        return { status: "unavailable", reason };
      }
      callbacks.current.onReady();
      return { status: "completed", value: { installation_completed: true, debugger_restart_requested: true, running_debuggee: "not established" } };
    } catch (cause) {
      const reason = cause instanceof Error ? cause.message : String(cause);
      if (active.current && attempt.current === id) setFailed(reason);
      return { status: "unavailable", reason };
    } finally {
      if (attempt.current === id) { inFlight.current = false; if (active.current) setBusy(false); }
    }
  };
  useEffect(() => {
    active.current = true;
    const key = `${offer.kind}:${offer.language}`;
    if (attempted.has(key) && !installations.has(key)) {
      setBusy(false);
      setFailed("The debugger is still unavailable. Retry installation to try again.");
    } else {
      attempted.add(key);
      void run();
    }
    return () => { active.current = false; attempt.current++; inFlight.current = false; };
  }, [offer.kind, offer.language]);
  const revision = useUiRevision(() => [offer.kind, offer.language, detail, currentSource.current(), busy, failed, attempt.current, active.current, inFlight.current]);
  useUiReading("debugger setup", { offer, detail, busy, error: failed, attempt: attempt.current, available: !!onInstall, scope: "Installation status; automatic restart is a request, not a running-session witness" });
  const invoke = useUiCommands(failed ? [{ key: parent, command: {
    label: `Retry ${offer.language} debugger installation`, parent, subject: parent, parameters: {}, prerequisites: ["Current failed debugger setup; existing host installer and configured execution policy apply"],
    read: () => ({ revision: revision(), enabled: active.current && !!callbacks.current.onInstall && !busy && !inFlight.current }),
    execute: async (_args, _actor, signal) => run(signal),
  } }] : []);
  const retry = () => { const captured = currentOffer.current; void invoke(parent).catch(cause => { if (active.current && currentOffer.current === captured) setFailed(cause instanceof Error ? cause.message : String(cause)); }); };
  const pretty = offer.language.charAt(0).toUpperCase() + offer.language.slice(1);
  return <span className="debug-strip__missing" role={failed ? "alert" : "status"}>
    <span data-tip={detail ?? undefined}>{busy ? `Installing ${pretty} debugger…` : failed ? `${pretty} debugger installation failed.` : `No ${pretty} debugger on this machine.`}</span>
    {failed && <><span className="debug-strip__error">{failed}</span>
      <button type="button" className="btn debug-strip__install" disabled={busy} onClick={retry}>Retry installation</button></>}
  </span>;
}

</storycode:copy>
<a id="f-09306f4ec1a5-IconButton"></a>
### `IconButton`

One control: a glyph to press, a name for a screen reader, a sentence on
hover. The sentence says what the button DOES rather than repeating its
name — "Step over" twice helps nobody.

<storycode:copy id="f-09306f4ec1a5-IconButton">
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

</storycode:copy>
<a id="f-09306f4ec1a5-DebugStripProps"></a>
### `DebugStripProps`

<storycode:copy id="f-09306f4ec1a5-DebugStripProps">
export interface DebugStripProps {
  readingChanged?: boolean;
  reading?: import("./observations").ObservationReading | null;
  locationActions?: ReactNode;
  editorView?: EditorView | null;
  /** Zero-based line in the displayed editor. */
  pausedLine?: number | null;
  status: "idle" | "starting" | "paused" | "running" | "finished" | "failed";
  /** Which generated file this session is running. */
  program?: string | null;
  launchTarget?: string | null;
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

</storycode:copy>
<a id="f-09306f4ec1a5-STATUS_WORD"></a>
### `STATUS_WORD`

<storycode:copy id="f-09306f4ec1a5-STATUS_WORD">
const STATUS_WORD: Record<DebugStripProps["status"], string> = {
  idle: "",
  starting: "starting…",
  running: "running…",
  paused: "paused",
  finished: "finished",
  failed: "failed",
};

</storycode:copy>
<a id="f-09306f4ec1a5-statusWord"></a>
### `statusWord`

The status word, with the exit info a finished program earned. The session
behind it is already gone — reaped server-side the moment the program ended
— so this line and the local breakpoints are all that remains of it.

<storycode:copy id="f-09306f4ec1a5-statusWord">
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

</storycode:copy>
<a id="f-09306f4ec1a5-DebugStrip"></a>
### `DebugStrip`

<storycode:copy id="f-09306f4ec1a5-DebugStrip">
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
      <span className="debug-strip__status" role="status" data-tip={props.launchTarget ? `Launch target: ${props.launchTarget}` : undefined}>
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
      <span>{props.launchTarget ?? props.program ?? ""} · {statusWord(props.status, props.exitCode)}</span>
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
          sourceIdentity={() => JSON.stringify([props.program, props.launchTarget, props.reading?.source.revision, props.reading?.scope, setupEditorId(props.editorView), props.editorView?.state.doc.toString()])}
        />
      )}
    </div>
  );
  return <>
    {props.reading && <ObservationReading reading={props.reading} changed={props.readingChanged} historical={props.status !== "paused" || props.selectedFrame !== props.reading.scope.frame} />}
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
</storycode:copy>

### Assemble `apps/web/src/debug/DebugStrip.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/DebugStrip.tsx"><storycode:paste select="#f-09306f4ec1a5-frameLabel"/><storycode:paste select="#f-09306f4ec1a5-stepForKey"/><storycode:paste select="#f-09306f4ec1a5-debugKeysActive"/><storycode:paste select="#f-09306f4ec1a5-installations"/><storycode:paste select="#f-09306f4ec1a5-setupEditors"/><storycode:paste select="#f-09306f4ec1a5-setupEditorId"/><storycode:paste select="#f-09306f4ec1a5-MissingTool"/><storycode:paste select="#f-09306f4ec1a5-IconButton"/><storycode:paste select="#f-09306f4ec1a5-DebugStripProps"/><storycode:paste select="#f-09306f4ec1a5-STATUS_WORD"/><storycode:paste select="#f-09306f4ec1a5-statusWord"/><storycode:paste select="#f-09306f4ec1a5-DebugStrip"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-1063c352c210"></a>
## `apps/web/src/debug/DebugStrip.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Launch the artifact built from the selected compiled project](../../../../../guarantees/debugging/a-compiled-language-launches-what-a-build-produced/contract.md), [Install a supported missing debugger and resume the requested launch](../../../../../guarantees/debugging/a-missing-debugger-is-a-button/contract.md), [A Plain File Has The Same Debugger A Document Has](../../../../../guarantees/debugging/a-plain-file-has-the-same-debugger/contract.md), [An Exception Breakpoint Uses The Adapter's Own Filters](../../../../../guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters/contract.md), [Choose a launch without replacing the editor being read](../../../../../guarantees/debugging/debug-controls-preserve-editor-space/contract.md), [Every Shortcut Is A Setting](../../../../../guarantees/editor-intelligence/every-shortcut-is-a-setting/contract.md).

<a id="f-1063c352c210-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-1063c352c210-expression_statement">
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { isAction } from "../lib/keymap";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { DebugStrip, debugKeysActive, frameLabel, stepForKey } from "./DebugStrip";
import type { DebugStripProps } from "./DebugStrip";
import type { TranscriptEvent } from "../api/types";

describe("the combo box's line for a frame", () => {
  it("names the function and the line it will return to", () => {
    expect(
      frameLabel({ id: 2, name: "line_total", line: 14, source: "orders.py", in_document: true }),
    ).toBe("line_total — line 15");
  });

  it("says external for a frame outside the document", () => {
    // A library frame has no line HERE, and inventing one would point at
    // whatever prose happens to share the number.
    expect(
      frameLabel({ id: 9, name: "json.loads", line: null, source: "json/__init__.py", in_document: false }),
    ).toBe("json.loads — external");
    expect(
      frameLabel({ id: 3, name: "runner", line: null, source: "orders.py", in_document: true }),
    ).toBe("runner — external");
  });

  it("names a frame in another file of the folder by that file and line", () => {
    // Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
    // The callee in `tools/helpers.py` is not external: the app can open it,
    // and the stack should say where it is rather than shrug.
    expect(
      frameLabel({ id: 4, name: "double", line: null, source: "tools/helpers.py", source_line: 1, in_document: false }),
    ).toBe("double — tools/helpers.py:2");
    expect(
      frameLabel({ id: 5, name: "loads", line: null, source: "/usr/lib/python3/json/__init__.py", source_line: 3, in_document: false }),
    ).toBe("loads — external");
  });
});

</storycode:copy>
<a id="f-1063c352c210-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-1063c352c210-expression_statement-2">
describe("the function keys the tooltips promise", () => {
  it("maps F5, F10 and F11 to the steps their tooltips name", () => {
    // Continue is the keymap's (`debug.continue`), not this table's.
    expect(stepForKey("F5", false)).toBeNull();
    expect(isAction({ key: "F5", ctrlKey: false, metaKey: false, altKey: false, shiftKey: false }, "debug.continue")).toBe(true);
    expect(stepForKey("F10", false)).toBe("over");
    expect(stepForKey("F11", false)).toBe("in");
    expect(stepForKey("F11", true)).toBe("out");
  });

  it("claims no other key", () => {
    // Shift-F5 and Shift-F10 belong to the browser (reload, context menu);
    // taking them would be taking keys the tooltips never promised.
    expect(stepForKey("F5", true)).toBeNull();
    expect(stepForKey("F10", true)).toBeNull();
    expect(stepForKey("F12", false)).toBeNull();
    expect(stepForKey("Enter", false)).toBeNull();
  });

  it("acts only while paused", () => {
    // The guard that keeps F5 as the browser's reload the rest of the time.
    expect(debugKeysActive("paused")).toBe(true);
    for (const status of ["idle", "starting", "running", "finished", "failed"] as const) {
      expect(debugKeysActive(status)).toBe(false);
    }
  });
});

</storycode:copy>
<a id="f-1063c352c210-base"></a>
### `base`

Shared by every strip test, including the ones below this file's first
describe — it was scoped inside that one and the next block could not see it.

<storycode:copy id="f-1063c352c210-base">
// Shared by every strip test, including the ones below this file's first
// describe — it was scoped inside that one and the next block could not see it.
const base = (over: Partial<DebugStripProps> = {}): DebugStripProps => ({
  status: "paused",
  program: "orders.py",
  message: null,
  capabilities: null,
  exceptionFilters: [],
  onToggleExceptionFilter: () => {},
  frames: [
    { id: 2, name: "line_total", line: 14, source: "orders.py", in_document: true },
    { id: 3, name: "<module>", line: 20, source: "orders.py", in_document: true },
  ],
  selectedFrame: 2,
  watches: [],
  onSelectFrame: () => {},
  onStep: () => {},
  onJumpHere: () => {},
  onStart: () => {},
  onStop: () => {},
  onAddWatch: () => {},
  onRemoveWatch: () => {},
  ...over,
});

</storycode:copy>
<a id="f-1063c352c210-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-1063c352c210-expression_statement-3">
describe("the strip on the debugged block", () => {
  // Without cleanup, a strip from an earlier test keeps its key listener and
  // preventDefaults the F10 the next test fires.
  afterEach(cleanup);


  it("shows the build in a terminal while starting, and keeps it when the start failed", () => {
    // The spinner is the wrong answer: a build that fails says why in its
    // own output, and a `Building…` label throws that away.
    const built: TranscriptEvent[] = [
      { t: 0, kind: "cmd", data: "dotnet build --nologo --configuration Debug (app/app.csproj)" },
      { t: 400, kind: "err", data: "Program.cs(1,25): error CS1519: Invalid token\n" },
      { t: 500, kind: "exit", code: 1 },
    ];
    const { rerender } = render(
      <DebugStrip {...base({ status: "starting", buildOutput: built })} />,
    );
    expect(screen.getByTestId("debug-build")).toBeTruthy();
    expect(screen.getByTestId("watch-terminal")).toBeTruthy();

    // The failing case is the one it exists for.
    rerender(
      <DebugStrip {...base({ status: "failed", buildOutput: built, message: "build failed" })} />,
    );
    expect(screen.getByTestId("debug-build")).toBeTruthy();

    // Once the program is running, a successful build's log is a wall of
    // text between a person and the frame they are looking at.
    rerender(<DebugStrip {...base({ status: "paused", buildOutput: built })} />);
    expect(screen.queryByTestId("debug-build")).toBeNull();
  });

  it("says the build is not evidence, where the build is shown", () => {
    // The transcript is the record; the terminal is the run happening.
    // Nothing is ever verified against what a terminal showed, and the one
    // place that could be misread is beside a debugger.
    render(
      <DebugStrip
        {...base({ status: "starting", buildOutput: [{ t: 0, kind: "out", data: "ok\n" }] })}
      />,
    );
    expect(screen.getByTestId("debug-build").textContent).toContain("not a transcript");
  });

  it("shows no build chrome for a language whose generated file IS the program", () => {
    render(<DebugStrip {...base({ status: "starting", buildOutput: [] })} />);
    expect(screen.queryByTestId("debug-build")).toBeNull();
  });

  it("is nothing while idle: the Debug chip is the way in", () => {
    const { container } = render(<DebugStrip {...base({ status: "idle" })} />);
    expect(container.firstChild).toBeNull();
  });

  it("offers the stack as a combo box that selects a frame", () => {
    const onSelectFrame = vi.fn();
    render(<DebugStrip {...base({ onSelectFrame })} />);
    const select = screen.getByLabelText("Call stack") as HTMLSelectElement;
    expect([...select.options].map((option) => option.text)).toEqual([
      "line_total — line 15",
      "<module> — line 21",
    ]);
    fireEvent.change(select, { target: { value: "3" } });
    expect(onSelectFrame).toHaveBeenCalledWith(3);
  });

  it("steps on F10 while paused, and never while running", () => {
    const onStep = vi.fn();
    const { rerender } = render(<DebugStrip {...base({ onStep })} />);
    fireEvent.keyDown(window, { key: "F10" });
    expect(onStep).toHaveBeenCalledWith("over");

    onStep.mockClear();
    rerender(<DebugStrip {...base({ onStep, status: "running" })} />);
    fireEvent.keyDown(window, { key: "F10" });
    expect(onStep).not.toHaveBeenCalled();
  });

  it("leaves a modified chord to the browser", () => {
    // Ctrl-F5 is a hard reload; a debugger that eats it is a debugger that
    // broke the browser.
    const onStep = vi.fn();
    render(<DebugStrip {...base({ onStep })} />);
    fireEvent.keyDown(window, { key: "F5", ctrlKey: true });
    expect(onStep).not.toHaveBeenCalled();
  });

  it("shows why a session failed, where its controls are", () => {
    // The panel's last duty, moved here with it.
    render(
      <DebugStrip
        {...base({ status: "failed", message: "no debug adapter for python on this machine." })}
      />,
    );
    expect(screen.getByRole("alert").textContent).toContain("no debug adapter");
  });

  it("installs a missing debugger and resumes without another click", async () => {
    // Protects docs/guarantees/debugging/a-missing-debugger-is-a-button.md
    //
    // The failure a person CAN fix from here. The old strip printed the whole
    // sentence — "…Install one with `hick dap install python`, or…" — and
    // truncated it at the strip's width, so the actionable half was off the
    // end of the control that could have done it.
    const onInstall = vi.fn().mockResolvedValue(undefined);
    const onStart = vi.fn();
    render(
      <DebugStrip
        {...base({
          status: "failed",
          message: "no debug adapter for python on this machine. Install one with…",
          offerInstall: { kind: "dap", language: "python" },
          onInstall,
          onStart,
        })}
      />,
    );
    expect(screen.queryByRole("button", { name: "Install it" })).toBeNull();
    await waitFor(() => expect(onInstall).toHaveBeenCalledWith({ kind: "dap", language: "python" }));
    await waitFor(() => expect(onStart).toHaveBeenCalledOnce());
  });

  it("still says the whole sentence when nothing here can install it", () => {
    // Go's adapter comes from `go install`. There is no button to offer, so
    // the prose is all there is — and it must not be swallowed.
    render(
      <DebugStrip
        {...base({
          status: "failed",
          message: "no debug adapter for go on this machine. delve is the Go debugger: …",
        })}
      />,
    );
    expect(screen.getByRole("alert").textContent).toContain("delve");
  });

  it("says how a finished program ended, and offers the way back in", () => {
    // The session behind a finished strip no longer exists — the server
    // reaped it with the program — so the strip's job is the exit info, a
    // re-run, and a dismissal. Not a row of step buttons for a dead session.
    const onStart = vi.fn();
    const onStop = vi.fn();
    render(
      <DebugStrip
        {...base({ status: "finished", exitCode: 0, frames: [], selectedFrame: null, onStart, onStop })}
      />,
    );
    expect(screen.getByRole("status").textContent).toContain("finished — exit code 0");
    // Stepping verbs are gone: there is nothing they could act on.
    expect(screen.queryByLabelText("Continue")).toBeNull();
    expect(screen.queryByLabelText("Step over")).toBeNull();
    // The re-run, with the breakpoints kept locally for it.
    fireEvent.click(screen.getByLabelText("Debug again"));
    expect(onStart).toHaveBeenCalled();
    // And Stop has become a dismissal of chrome, not of a session.
    expect(screen.queryByLabelText("Stop")).toBeNull();
    fireEvent.click(screen.getByLabelText("Dismiss"));
    expect(onStop).toHaveBeenCalled();
  });

  it("shows a failing exit code, and plain 'finished' when none was reported", () => {
    const { rerender } = render(
      <DebugStrip {...base({ status: "finished", exitCode: 3, frames: [], selectedFrame: null })} />,
    );
    expect(screen.getByRole("status").textContent).toContain("finished — exit code 3");
    rerender(
      <DebugStrip {...base({ status: "finished", exitCode: null, frames: [], selectedFrame: null })} />,
    );
    expect(screen.getByRole("status").textContent).toContain("finished");
    expect(screen.getByRole("status").textContent).not.toContain("exit code");
  });

  it("lists watches and lets each be removed", () => {
    const onRemoveWatch = vi.fn();
    render(
      <DebugStrip
        {...base({ watches: [{ expression: "quantity", value: "2" }], onRemoveWatch })}
      />,
    );
    fireEvent.click(screen.getByLabelText("Stop watching quantity"));
    expect(onRemoveWatch).toHaveBeenCalledWith("quantity");
  });
});

</storycode:copy>
<a id="f-1063c352c210-expression_statement-4"></a>
### `expression_statement`

Protects docs/guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters.md

<storycode:copy id="f-1063c352c210-expression_statement-4">
// Protects docs/guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters.md
describe("stopping on exceptions", () => {
  afterEach(cleanup);

  const caps = (filters: { id: string; label: string }[]) => ({
    conditional_breakpoints: true,
    hit_conditional_breakpoints: true,
    log_points: true,
    set_variable: true,
    restart_frame: false,
    step_in_targets: true,
    goto_targets: true,
    step_back: false,
    exception_filters: filters,
  });

  it("offers the adapter's own filters, in the adapter's own words", () => {
    render(
      <DebugStrip
        {...base({
          capabilities: caps([
            { id: "raised", label: "Raised Exceptions" },
            { id: "uncaught", label: "Uncaught Exceptions" },
          ]),
        })}
      />,
    );
    fireEvent.click(screen.getByLabelText("Debug options"));
    const switches = screen.getAllByRole("switch");
    expect(switches.map((s) => s.textContent)).toEqual([
      "Raised Exceptions",
      "Uncaught Exceptions",
    ]);
    expect(switches.every((s) => s.getAttribute("aria-checked") === "false")).toBe(true);
  });

  it("draws nothing for an adapter that has none", () => {
    render(<DebugStrip {...base({ capabilities: caps([]) })} />);
    expect(screen.queryAllByRole("switch")).toHaveLength(0);
  });

  it("shows which are on, and toggles by id", () => {
    const toggled: string[] = [];
    render(
      <DebugStrip
        {...base({
          capabilities: caps([
            { id: "raised", label: "Raised Exceptions" },
            { id: "uncaught", label: "Uncaught Exceptions" },
          ]),
          exceptionFilters: ["uncaught"],
          onToggleExceptionFilter: (id) => toggled.push(id),
        })}
      />,
    );
    fireEvent.click(screen.getByLabelText("Debug options"));
    const on = screen.getByRole("switch", { name: "Uncaught Exceptions" });
    expect(on.getAttribute("aria-checked")).toBe("true");
    fireEvent.click(screen.getByRole("switch", { name: "Raised Exceptions" }));
    // The id, never the label: the label is what a person reads and the id is
    // what the adapter answers to.
    expect(toggled).toEqual(["raised"]);
  });
});

</storycode:copy>
<a id="f-1063c352c210-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-1063c352c210-expression_statement-5">
describe("automatic debugger setup", () => {
  afterEach(cleanup);
  const offerInstall = { kind: "dap", language: "python" };
  it("reports installation failure and retries only when requested", async () => {
    const onInstall = vi.fn().mockRejectedValueOnce(new Error("Download failed")).mockResolvedValueOnce(undefined);
    const onStart = vi.fn();
    render(<DebugStrip {...base({ status: "failed", offerInstall, onInstall, onStart })} />);
    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("Download failed"));
    expect(onInstall).toHaveBeenCalledTimes(1);
    expect(onStart).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Retry installation" }));
    await waitFor(() => expect(onStart).toHaveBeenCalledOnce());
    expect(onInstall).toHaveBeenCalledTimes(2);
  });
  it("does not start a session after setup is dismissed", async () => {
    let finish!: () => void;
    const pending = new Promise<void>(resolve => { finish = resolve; });
    const onInstall = vi.fn(() => pending);
    const onStart = vi.fn();
    const { rerender } = render(<DebugStrip {...base({ status: "failed", offerInstall, onInstall, onStart })} />);
    await waitFor(() => expect(onInstall).toHaveBeenCalledOnce());
    rerender(<DebugStrip {...base({ status: "idle", onStart })} />);
    finish();
    await pending;
    expect(onStart).not.toHaveBeenCalled();
  });
  it("does not loop installations if starting still reports a missing adapter", async () => {
    const onInstall = vi.fn().mockResolvedValue(undefined);
    const onStart = vi.fn();
    const { rerender } = render(<DebugStrip {...base({ status: "failed", offerInstall, onInstall, onStart })} />);
    await waitFor(() => expect(onStart).toHaveBeenCalledOnce());
    rerender(<DebugStrip {...base({ status: "starting", onInstall, onStart })} />);
    rerender(<DebugStrip {...base({ status: "failed", offerInstall, onInstall, onStart })} />);
    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("still unavailable"));
    expect(onInstall).toHaveBeenCalledOnce();
  });
});

</storycode:copy>
<a id="f-1063c352c210-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-1063c352c210-expression_statement-6">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares actual installer retry and refuses to restart a replacement target after a late installation", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let release!: () => void;
  const install = vi.fn().mockRejectedValueOnce(Error("first download failed")).mockImplementationOnce(() => new Promise<void>(resolve => { release = resolve; }));
  const start = vi.fn();
  const ui = (program: string) => <UiActionContext.Provider value={registry}><DebugStrip {...base({ program, status: "failed", offerInstall: { kind: "dap", language: "python" }, onInstall: install, onStart: start })} /></UiActionContext.Provider>;
  const rendered = render(ui("first.py"));
  const action = () => registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === "Retry python debugger installation")!;
  try {
    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("first download failed"));
    const descriptor = action(); let pending!: ReturnType<typeof registry.act>;
    await act(async () => { pending = registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: descriptor.handle, arguments: {}, request_id: "retry" }); });
    expect(install).toHaveBeenCalledTimes(2); expect(start).not.toHaveBeenCalled();
    rendered.rerender(ui("second.py"));
    await act(async () => { release(); expect(await pending).toMatchObject({ outcome: { status: "unavailable", reason: expect.stringContaining("source/setup owner") } }); });
    expect(start).not.toHaveBeenCalled();
    expect(screen.getByRole("alert").textContent).toContain("no debugger restart was requested");
    expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: descriptor.handle, arguments: {}, request_id: "old-target" })).toMatchObject({ status: "unavailable" });
    install.mockResolvedValue(undefined);
    await act(async () => expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: action().handle, arguments: {}, request_id: "current-target" })).toMatchObject({ outcome: { value: { installation_completed: true, debugger_restart_requested: true, running_debuggee: "not established" } } }));
    expect(start).toHaveBeenCalledTimes(1);
  } finally { rendered.unmount(); }
});

</storycode:copy>
<a id="f-1063c352c210-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-1063c352c210-expression_statement-7">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("cancels the originating setup request without claiming to undo a shared download", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let release!: () => void;
  const install = vi.fn().mockRejectedValueOnce(Error("download failed")).mockImplementationOnce(() => new Promise<void>(resolve => { release = resolve; }));
  const start = vi.fn();
  const rendered = render(<UiActionContext.Provider value={registry}><DebugStrip {...base({ status: "failed", offerInstall: { kind: "dap", language: "python" }, onInstall: install, onStart: start })} /></UiActionContext.Provider>);
  try {
    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("download failed"));
    const descriptor = registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === "Retry python debugger installation")!;
    const actor = { id: "agent", kind: "agent" as const }; let pending!: ReturnType<typeof registry.act>;
    await act(async () => { pending = registry.act(registry.scope, actor, { handle: descriptor.handle, arguments: {}, request_id: "cancel-setup" }); });
    expect(registry.cancel(registry.scope, actor, "cancel-setup")).toBe(true);
    await act(async () => { release(); expect(await pending).toMatchObject({ outcome: { status: "unavailable", reason: expect.stringContaining("no debugger restart") } }); });
    expect(start).not.toHaveBeenCalled(); expect(install).toHaveBeenCalledTimes(2);
    expect(screen.getByRole("alert").textContent).toContain("Installation may have completed");
  } finally { rendered.unmount(); }
});
</storycode:copy>

### Assemble `apps/web/src/debug/DebugStrip.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/DebugStrip.test.tsx"><storycode:paste select="#f-1063c352c210-expression_statement"/><storycode:paste select="#f-1063c352c210-expression_statement-2"/><storycode:paste select="#f-1063c352c210-base"/><storycode:paste select="#f-1063c352c210-expression_statement-3"/><storycode:paste select="#f-1063c352c210-expression_statement-4"/><storycode:paste select="#f-1063c352c210-expression_statement-5"/><storycode:paste select="#f-1063c352c210-expression_statement-6"/><storycode:paste select="#f-1063c352c210-expression_statement-7"/></storycode:file>


</storycode:doc>
