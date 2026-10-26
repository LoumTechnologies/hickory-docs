<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/useDebugger.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-597e991b795b"></a>
## `apps/web/src/debug/useDebugger.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Keystroke Redraws Only The Editor](../../../../../guarantees/authoring/a-keystroke-redraws-only-the-editor/contract.md), [A Breakpoint Can Carry A Condition](../../../../../guarantees/debugging/a-breakpoint-can-carry-a-condition/contract.md), [A Plain File Has The Same Debugger A Document Has](../../../../../guarantees/debugging/a-plain-file-has-the-same-debugger/contract.md), [An Exception Breakpoint Uses The Adapter's Own Filters](../../../../../guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters/contract.md).

<a id="f-597e991b795b-DebugStatus"></a>
### `DebugStatus`

One debug session per open document, and the editor state that follows it.
The hook owns three things the UI reads and the editor draws: where
execution is paused, what is in scope there, and which breakpoints the
adapter could actually bind. Everything arrives as an event, because a
`continue` takes as long as the program does.

<storycode:copy id="f-597e991b795b-DebugStatus">
// One debug session per open document, and the editor state that follows it.
//
// The hook owns three things the UI reads and the editor draws: where
// execution is paused, what is in scope there, and which breakpoints the
// adapter could actually bind. Everything arrives as an event, because a
// `continue` takes as long as the program does.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useDebugActions } from "./uiDebugActions";
import { retainDebugReading } from "./readingCapture";
import { getWorkspaceRealtime, type Realtime } from "../api/realtime";
import type { TranscriptEvent } from "../api/types";
import { DebugClient } from "./client";
import type {
  BreakpointStatus,
  DebugBreakpoint,
  DebugCapabilities,
  DebugEvent,
  Frame,
  Step,
  Variable,
} from "./client";

export type DebugStatus = "idle" | "starting" | "paused" | "running" | "finished" | "failed";

</storycode:copy>
<a id="f-597e991b795b-Watch"></a>
### `Watch`

One watched expression, and what it held the last time anyone asked.

<storycode:copy id="f-597e991b795b-Watch">
/** One watched expression, and what it held the last time anyone asked. */
export interface Watch {
  expression: string;
  /** Null until the first answer arrives, and again after the program ends. */
  value: string | null;
}

</storycode:copy>
<a id="f-597e991b795b-DebugSession"></a>
### `DebugSession`

<storycode:copy id="f-597e991b795b-DebugSession">
export interface DebugSession {
  reading?: import("./observations").ObservationReading | null;
  /** A tool this machine can fetch, when a missing one is why the last
   * attempt failed. Null the rest of the time. */
  offerInstall: { kind: string; language: string } | null;
  status: DebugStatus;
  message: string | null;
  /** The generated file being debugged, when one was named. */
  program: string | null;
  capabilities: DebugCapabilities | null;
  /** 0-based document line execution is paused on. */
  pausedLine: number | null;
  frames: Frame[];
  variables: Variable[];
  selectedFrame: number | null;
  /** Where the user has asked to stop, whether or not it bound. */
  breakpoints: HeldBreakpoint[];
  sourceBreakpoints: Record<string, HeldBreakpoint[]>;
  setSourceBreakpoints(source: string, breakpoints: HeldBreakpoint[]): void;
  /** How the program ended, once it has: its exit code, when reported. */
  exitCode: number | null;
  /**
   * The build that ran before there was a program to launch, when one did.
   *
   * Empty for Python, Node and Go, where the generated file IS the program.
   * Kept whether the start succeeded or failed — the failing case is the one
   * it exists for, because a build that fails says why in the compiler's own
   * words and "build failed" throws all of that away.
   */
  buildOutput: TranscriptEvent[];
  lastValue: { expression: string; value: string; type: string | null } | null;
  /** Expressions re-evaluated on every pause and frame change. */
  watches: Watch[];
  /**
   * The adapter's exception filters that are switched on.
   *
   * Ids, not names — `capabilities.exception_filters` carries the labels a
   * person reads. Kept across runs like breakpoints are: "stop on uncaught
   * exceptions" is a standing preference, not a property of one session.
   */
  exceptionFilters: string[];

  /** Debug one generated file. Omitted means the first debuggable one. */
  launchTarget?: string | null;
  start(program?: string, sources?: Record<string, HeldBreakpoint[]>, project?: string, profile?: import("./launchProfiles").LaunchSelection): void;
  replaceBreakpoints(breakpoints: HeldBreakpoint[]): void;
  stop(): void;
  step(how: Step): void;
  jumpTo(line: number): void;
  runTo(line: number): void;
  toggleBreakpoint(line: number): void;
  /** Turn one of the adapter's exception filters on or off. */
  toggleExceptionFilter(id: string): void;
  /**
   * Attach a condition, a hit count, or a log message to the breakpoint on
   * this line — or clear them, by passing empty strings.
   *
   * All three have been carried by `hick-dap::Breakpoint` and forwarded to
   * the adapter since the debugger was written; until this existed nothing
   * could set one, and the gutter's own `cm-bp-conditional` styling was
   * unreachable.
   */
  setBreakpointCondition(
    line: number,
    patch: { condition?: string; hit_condition?: string; log_message?: string },
  ): void;
  selectFrame(id: number): void;
  evaluate(expression: string, context?: "hover" | "watch" | "repl"): void;
  /** Ask for a value and get it back, for the hover tooltip. */
  valueAt(expression: string): Promise<string | null>;
  /**
   * Ask for a value the way a person typed it: any expression, no
   * known-name guard, answered as a promise for the inline eval widget.
   */
  query(expression: string): Promise<string | null>;
  addWatch(expression: string): void;
  removeWatch(expression: string): void;
}

</storycode:copy>
<a id="f-597e991b795b-HeldBreakpoint"></a>
### `HeldBreakpoint`

A breakpoint as this pane holds it: what the adapter said about it, plus
the three things only the person knows.
The adapter never reports a condition back, so it cannot come from
`BreakpointStatus` — it has to survive every status the server sends.

<storycode:copy id="f-597e991b795b-HeldBreakpoint">
/**
 * A breakpoint as this pane holds it: what the adapter said about it, plus
 * the three things only the person knows.
 *
 * The adapter never reports a condition back, so it cannot come from
 * `BreakpointStatus` — it has to survive every status the server sends.
 */
export interface HeldBreakpoint extends BreakpointStatus {
  /** Stop only when this is true, in the debuggee's own language. */
  condition?: string;
  /** Stop only on the Nth hit — the adapter's own syntax. */
  hit_condition?: string;
  /** Log this and continue instead of stopping. */
  log_message?: string;
}

</storycode:copy>
<a id="f-597e991b795b-isConditional"></a>
### `isConditional`

Whether a breakpoint carries anything beyond "stop here".

<storycode:copy id="f-597e991b795b-isConditional">
/** Whether a breakpoint carries anything beyond "stop here". */
export function isConditional(breakpoint: HeldBreakpoint): boolean {
  return Boolean(breakpoint.condition || breakpoint.hit_condition || breakpoint.log_message);
}

</storycode:copy>
<a id="f-597e991b795b-followMoves"></a>
### `followMoves`

Put each breakpoint where the adapter actually bound it, keeping what the
person attached to it.
A dot drawn on the line you asked for, while the program stops on the line
below, is a gutter that disagrees with the debugger — and the debugger is
the one that is right. But the move must carry the condition with it: a
breakpoint that slides one line down and silently becomes unconditional
would stop on every pass, which is the opposite of what was asked for.
That is why the match is made against the status BEFORE the move is
applied, where both the requested and the bound line are still in hand.

<storycode:copy id="f-597e991b795b-followMoves">
/**
 * Put each breakpoint where the adapter actually bound it, keeping what the
 * person attached to it.
 *
 * A dot drawn on the line you asked for, while the program stops on the line
 * below, is a gutter that disagrees with the debugger — and the debugger is
 * the one that is right. But the move must carry the condition with it: a
 * breakpoint that slides one line down and silently becomes unconditional
 * would stop on every pass, which is the opposite of what was asked for.
 * That is why the match is made against the status BEFORE the move is
 * applied, where both the requested and the bound line are still in hand.
 */
function followMoves(previous: HeldBreakpoint[], statuses: BreakpointStatus[]): HeldBreakpoint[] {
  return statuses.map((status) => {
    const held =
      previous.find((breakpoint) => breakpoint.line === status.line) ??
      (status.moved_to === undefined
        ? undefined
        : previous.find((breakpoint) => breakpoint.line === status.moved_to));
    const moved =
      status.moved_to === undefined || status.moved_to === status.line
        ? status
        : { ...status, line: status.moved_to, moved_to: undefined };
    return {
      ...moved,
      condition: held?.condition,
      hit_condition: held?.hit_condition,
      log_message: held?.log_message,
    };
  });
}

</storycode:copy>
<a id="f-597e991b795b-onTheWire"></a>
### `onTheWire`

The wire form: only what was actually set, so an adapter is never handed
an empty condition string to parse.

<storycode:copy id="f-597e991b795b-onTheWire">
/**
 * The wire form: only what was actually set, so an adapter is never handed
 * an empty condition string to parse.
 */
function onTheWire(breakpoints: HeldBreakpoint[]): DebugBreakpoint[] {
  return breakpoints.map((breakpoint) => ({
    line: breakpoint.line,
    ...(breakpoint.condition ? { condition: breakpoint.condition } : {}),
    ...(breakpoint.hit_condition ? { hit_condition: breakpoint.hit_condition } : {}),
    ...(breakpoint.log_message ? { log_message: breakpoint.log_message } : {}),
  }));
}

</storycode:copy>
<a id="f-597e991b795b-eventIsOurs"></a>
### `eventIsOurs`

Whether an event from the channel is about THIS session.
The workspace socket is shared by every plain-file pane, and a pane that
took a neighbour's `stopped` would show itself paused on a line of a
program it never started. A session-bearing event is ours when the
session is the one we hold; the ones from before there is a session —
`started`, a `build`, a `failed` start — name the file instead, and are
ours when they name ours. An event with neither (an older server, a
test's bare event) is taken, so a document's own socket keeps working
unchanged.

<storycode:copy id="f-597e991b795b-eventIsOurs">
/**
 * Whether an event from the channel is about THIS session.
 *
 * The workspace socket is shared by every plain-file pane, and a pane that
 * took a neighbour's `stopped` would show itself paused on a line of a
 * program it never started. A session-bearing event is ours when the
 * session is the one we hold; the ones from before there is a session —
 * `started`, a `build`, a `failed` start — name the file instead, and are
 * ours when they name ours. An event with neither (an older server, a
 * test's bare event) is taken, so a document's own socket keeps working
 * unchanged.
 */
export function eventIsOurs(
  event: DebugEvent,
  uri: string,
  session: string | null,
): boolean {
  if (event.event === "started" || event.event === "build") {
    return event.doc === undefined || event.doc === uri;
  }
  if (event.event === "failed") {
    if (event.session) return event.session === session;
    return event.doc === undefined || event.doc === uri;
  }
  return session !== null && event.session === session;
}

</storycode:copy>
<a id="f-597e991b795b-IDLE_SESSION"></a>
### `IDLE_SESSION`

A session that does nothing: what a pane shows until its host has mounted.

<storycode:copy id="f-597e991b795b-IDLE_SESSION">
/** A session that does nothing: what a pane shows until its host has mounted. */
export const IDLE_SESSION: DebugSession = {
  sourceBreakpoints: {},
  setSourceBreakpoints() {},
  replaceBreakpoints() {},
  offerInstall: null,
  status: "idle",
  message: null,
  program: null,
  capabilities: null,
  pausedLine: null,
  frames: [],
  variables: [],
  selectedFrame: null,
  breakpoints: [],
  exitCode: null,
  buildOutput: [],
  lastValue: null,
  watches: [],
  exceptionFilters: [],
  start() {},
  stop() {},
  step() {},
  jumpTo() {},
  runTo() {},
  toggleBreakpoint() {},
  toggleExceptionFilter() {},
  setBreakpointCondition() {},
  selectFrame() {},
  evaluate() {},
  valueAt: () => Promise.resolve(null),
  query: () => Promise.resolve(null),
  addWatch() {},
  removeWatch() {},
};

</storycode:copy>
<a id="f-597e991b795b-useDebugger"></a>
### `useDebugger`

A debugger for one document, over the document's own socket.

<storycode:copy id="f-597e991b795b-useDebugger">
/** A debugger for one document, over the document's own socket. */
export function useDebugger(realtime: Realtime, docPath: string, projectContext = "new"): DebugSession {
  const client = useMemo(() => {
    const channel = realtime.debug?.();
    return channel ? new DebugClient(channel) : null;
  }, [realtime]);

  // The way back. A client that can send but is never handed the socket's
  // inbound frames sits at "starting…" forever while the engine answers into
  // nothing, so this is registered as soon as the client exists.
  useEffect(() => {
    if (!client || !realtime.onDebugFrame) return;
    realtime.onDebugFrame((frame) => client.handleFrame(frame));
    return () => realtime.onDebugFrame?.(() => false);
  }, [client, realtime]);

  return useDebuggerOver(client, docPath, [], projectContext);
}

</storycode:copy>
<a id="f-597e991b795b-workspaceClient"></a>
### `workspaceClient`

The workspace's one debug client, shared by every plain file.
One, not one per pane, for the same reason the language client is one:
the socket hands inbound frames to a single handler, and two clients on
one connection would each take the other's answers. Each pane's hook
filters the shared stream down to its own session (`eventIsOurs`).

<storycode:copy id="f-597e991b795b-workspaceClient">
// The workspace's one debug client, shared by every plain file.
//
// One, not one per pane, for the same reason the language client is one:
// the socket hands inbound frames to a single handler, and two clients on
// one connection would each take the other's answers. Each pane's hook
// filters the shared stream down to its own session (`eventIsOurs`).
let workspaceClient: DebugClient | null = null;
</storycode:copy>
<a id="f-597e991b795b-workspaceDebugClient"></a>
### `workspaceDebugClient`

<storycode:copy id="f-597e991b795b-workspaceDebugClient">
function workspaceDebugClient(): DebugClient | null {
  if (workspaceClient) return workspaceClient;
  const realtime = getWorkspaceRealtime();
  const channel = realtime?.debug?.();
  if (!realtime || !channel) return null;
  const client = new DebugClient(channel);
  realtime.onDebugFrame?.((frame) => client.handleFrame(frame));
  workspaceClient = client;
  return client;
}

</storycode:copy>
<a id="f-597e991b795b-resetWorkspaceDebugger"></a>
### `resetWorkspaceDebugger`

Test seam: forget the workspace debug client.

<storycode:copy id="f-597e991b795b-resetWorkspaceDebugger">
/** Test seam: forget the workspace debug client. */
export function resetWorkspaceDebugger(): void {
  workspaceClient = null;
}

</storycode:copy>
<a id="f-597e991b795b-useWorkspaceDebugger"></a>
### `useWorkspaceDebugger`

A debugger for a plain file — `src/main.rs`, `app.py` — over the
workspace connection. The same verbs, the same events, at the file's
own path; the server debugs a file that is not a document as itself.

<storycode:copy id="f-597e991b795b-useWorkspaceDebugger">
/**
 * A debugger for a plain file — `src/main.rs`, `app.py` — over the
 * workspace connection. The same verbs, the same events, at the file's
 * own path; the server debugs a file that is not a document as itself.
 */
export function useWorkspaceDebugger(path: string): DebugSession {
  const client = useMemo(() => workspaceDebugClient(), []);
  return useDebuggerOver(client, path);
}

</storycode:copy>
<a id="f-597e991b795b-useDebuggerOver"></a>
### `useDebuggerOver`

<storycode:copy id="f-597e991b795b-useDebuggerOver">
export function useDebuggerOver(client: DebugClient | null, docPath: string, initialBreakpoints: readonly number[] = [], projectContext = "new", uiSource?: unknown): DebugSession {
  const eventEpoch = useRef(0);
  const readingOwner = useRef(Symbol("debugger-capture"));
  const [, renderEventEpoch] = useState(0);
  const [reading, setReading] = useState<import("./observations").ObservationReading | null>(null);
  const [status, setStatus] = useState<DebugStatus>("idle");
  const [message, setMessage] = useState<string | null>(null);
  // A missing tool this machine can fetch, carried out of the failure so the
  // strip can offer a button instead of a command to go and type somewhere
  // else.
  //
  // It belongs to `message` and is cleared with it — `clearFailure` exists so
  // the two cannot come apart. They did once: installing the adapter started
  // the session successfully and left "No Python debugger on this machine"
  // sitting beside "finished — exit code 0".
  const [offerInstall, setOfferInstall] = useState<{
    kind: string;
    language: string;
  } | null>(null);
  const clearFailure = useCallback(() => {
    setMessage(null);
    setOfferInstall(null);
  }, []);
  const [capabilities, setCapabilities] = useState<DebugCapabilities | null>(null);
  const [pausedLine, setPausedLine] = useState<number | null>(null);
  const [frames, setFrames] = useState<Frame[]>([]);
  const [variables, setVariables] = useState<Variable[]>([]);
  const [selectedFrame, setSelectedFrame] = useState<number | null>(null);
  const [breakpoints, setBreakpoints] = useState<HeldBreakpoint[]>(() => initialBreakpoints.map((line) => ({ line, state: "bound" })));
  const [sourceBreakpoints, holdSources] = useState<Record<string, HeldBreakpoint[]>>({});
  // Which file this session is running, for the panel to say so: "paused" is
  // ambiguous in a document that generates three programs.
  const [launchTarget,setLaunchTarget] = useState<string|null>(null);
  const [program, setProgram] = useState<string | null>(null);
  const [lastValue, setLastValue] =
    useState<{ expression: string; value: string; type: string | null } | null>(null);
  const [watches, setWatches] = useState<Watch[]>([]);
  // Kept across runs, like breakpoints: "stop on uncaught exceptions" is a
  // standing preference, not a property of one session.
  const [exceptionFilters, setExceptionFilters] = useState<string[]>([]);
  // Read from the `started` handler, which is not re-created per render.
  const exceptionFiltersRef = useRef<string[]>([]);
  exceptionFiltersRef.current = exceptionFilters;
  const [exitCode, setExitCode] = useState<number | null>(null);
  const [buildOutput, setBuildOutput] = useState<TranscriptEvent[]>([]);

  const sessionRef = useRef<string | null>(null);
  // The status as of right now, for callbacks that must not close over a
  // stale one — a breakpoint toggled a moment after the program ended would
  // otherwise still be sent to a session that cannot answer.
  const statusRef = useRef<DebugStatus>("idle");
  // Hover asks for a value and wants it back; the channel answers with an
  // event, so the request is parked here until its answer arrives.
  const pendingValues = useRef(new Map<string, (value: string | null) => void>());

  statusRef.current = status;

  const uri = useMemo(() => docPath.startsWith("hick-project:///") ? docPath : `hick:///${docPath.replace(/^\/+/, "")}`, [docPath]);
  useEffect(() => { retainDebugReading(readingOwner.current, reading); return () => retainDebugReading(readingOwner.current, null); }, [reading]);

  useEffect(() => {
    if (!client) return;
    const off = client.on((event: DebugEvent) => {
      // A shared channel carries every pane's sessions; only ours is ours.
      // Before `started` there is no session to match, so a start that is
      // still in flight is recognised by the file it named.
      if (!eventIsOurs(event, uri, sessionRef.current)) return;
      eventEpoch.current++;
      renderEventEpoch(eventEpoch.current);
      switch (event.event) {
        case "reading":
          setReading(event.reading);
          break;
        case "build":
          // Replaces rather than appends: a build belongs to the start that
          // ran it, and the last one is the one on screen.
          setBuildOutput(event.events);
          break;
        case "started": {
          setReading(null);
          sessionRef.current = event.session;
          setCapabilities(event.capabilities);
          setBreakpoints((current) => followMoves(current, event.breakpoints));
          setStatus("running");
          clearFailure();
          setExitCode(null);
          // Re-applied to the new session. A standing "stop on uncaught"
          // that quietly stopped applying on the second run would be worse
          // than never having offered it.
          const standing = exceptionFiltersRef.current;
          if (client && standing.length > 0) {
            client.setExceptionBreakpoints(event.session, standing);
          }
          break;
        }
        case "stopped":
          setReading(current => event.observation_scope && current && ["session", "run", "stop", "frame", "invocation"].every(key => current.scope[key as keyof typeof current.scope] === event.observation_scope![key as keyof typeof current.scope]) ? current : null);
          setStatus("paused");
          setPausedLine(event.line);
          setFrames(event.frames);
          setVariables(event.variables);
          setSelectedFrame(event.selected_frame ?? event.frames[0]?.id ?? null);
          clearFailure();
          break;
        case "breakpoints":
          setBreakpoints((current) => followMoves(current, event.breakpoints));
          break;
        case "source_breakpoints":
          holdSources(current=>({...current,[event.source]:followMoves(current[event.source] ?? [],event.breakpoints)}));
          break;
        case "value": {
          setLastValue({ expression: event.expression, value: event.value, type: event.type });
          // Answers carry the expression they answer, and the watch list is
          // keyed by expression — that string IS the correlation id. A watch
          // and a hover for the same expression both get the answer, which
          // is fine: it is the same answer.
          setWatches((current) =>
            current.map((watch) =>
              watch.expression === event.expression ? { ...watch, value: event.value } : watch,
            ),
          );
          const waiting = pendingValues.current.get(event.expression);
          if (waiting) {
            waiting(event.value);
            pendingValues.current.delete(event.expression);
          }
          break;
        }
        case "finished":
          // The server reaped the session with the program: the id no longer
          // answers, so forget it NOW. A step, hover or stop sent after this
          // would be a request to a dead session, answered only by an error.
          sessionRef.current = null;
          setStatus("finished");
          setExitCode(event.exit_code ?? null);
          setPausedLine(null);
          setFrames([]);
          setVariables([]);
          setSelectedFrame(null);
          // The expressions survive — they are for the next run — but their
          // values belonged to a process that no longer exists.
          setWatches((current) => current.map((watch) => ({ ...watch, value: null })));
          break;
        case "ended":
          sessionRef.current = null;
          setStatus("idle");
          clearFailure();
          setPausedLine(null);
          setFrames([]);
          setVariables([]);
          setCapabilities(null);
          setWatches((current) => current.map((watch) => ({ ...watch, value: null })));
          break;
        case "failed": {
          if (event.about === "source_breakpoints") {
            setMessage(`Could not update breakpoints in ${event.doc ?? "this source"}: ${event.message}`);
            const source=event.doc;
            if(source)holdSources(current=>({...current,[source]:(current[source] ?? []).map(bp=>({...bp,state:"refused" as const,message:event.message}))}));
            break;
          }
          const stillWaiting = [...pendingValues.current.values()];
          pendingValues.current.clear();
          for (const resolve of stillWaiting) resolve(null);
          // A breakpoint that could not be set is a fact about that
          // breakpoint. It is marked broken, with the reason on hover, and
          // says nothing in the panel — where it would outlive its cause and
          // read as if it were about whatever you did next.
          if (event.about === "breakpoints" || event.about === "start") {
            const broken = new Set(event.lines ?? []);
            if (broken.size > 0) {
              setBreakpoints((current) =>
                current.map((breakpoint) =>
                  broken.has(breakpoint.line)
                    ? { ...breakpoint, state: "refused" as const, message: event.message }
                    : breakpoint,
                ),
              );
              if (event.about === "breakpoints") break;
            }
          }
          // A hover that could not be answered is not news. It resolves to
          // nothing and the tooltip shows the type alone; putting the
          // adapter's `NameError` in the panel would report our own question
          // back to the person as if their program were wrong.
          if (stillWaiting.length === 0) {
            setMessage(event.message);
            setOfferInstall(event.offer_install ?? null);
          }
          // A failed STEP leaves the program where it was — still paused —
          // so only a failure with no session at all is fatal to the UI.
          // "Finished" already IS a terminal state: a stray failure arriving
          // after the program ended must not redress a clean end as a broken
          // one.
          if (!sessionRef.current && statusRef.current !== "finished") setStatus("failed");
          break;
        }
      }
    });
    return off;
  }, [client, uri]);

  // A window that closes mid-session leaves a process running; the server
  // sweeps it, but saying so promptly is cheaper than waiting for the sweep.
  useEffect(() => {
    return () => {
      if (client && sessionRef.current) client.stop(sessionRef.current);
    };
  }, [client]);

  const toggleBreakpoint = useCallback(
    (line: number) => {
      setBreakpoints((current) => {
        const without = current.filter((breakpoint) => breakpoint.line !== line);
        const next =
          without.length === current.length
            ? // Optimistically bound: the adapter's answer replaces this a
              // moment later, and a dot that appears only after a round trip
              // feels broken. Not "pending", which would flash a half-filled
              // dot on every click in a language that binds instantly.
              [...current, { line, state: "bound" as const }]
            : without;
        // Only a session that is still running can be told. After the
        // program ends the adapter answers `setBreakpoints` with "Server
        // disconnected unexpectedly", which is true and useless: the
        // breakpoint is for the NEXT run, and it is kept here until then.
        const session = sessionRef.current;
        if (client && session && (statusRef.current === "paused" || statusRef.current === "running")) {
          clearFailure();
          client.setBreakpoints(session, onTheWire(next));
        }
        return next.sort((a, b) => a.line - b.line);
      });
    },
    [client],
  );

  const setSourceBreakpoints = useCallback((source: string, next: HeldBreakpoint[]) => {
    const prepared = next.map(bp=>({...bp, condition:bp.condition?.trim() || undefined,
      hit_condition:bp.hit_condition?.trim() || undefined, log_message:bp.log_message?.trim() || undefined,
      state:"pending" as const,message:undefined}));
    holdSources(current=>({...current,[source]:prepared}));
    const session = sessionRef.current;
    if (client && session && ["paused","running"].includes(statusRef.current))
      client.setSourceBreakpoints(session,source,onTheWire(prepared));
  }, [client]);

  const toggleExceptionFilter = useCallback(
    (id: string) => {
      setExceptionFilters((current) => {
        const next = current.includes(id)
          ? current.filter((one) => one !== id)
          : [...current, id];
        const session = sessionRef.current;
        if (
          client &&
          session &&
          (statusRef.current === "paused" || statusRef.current === "running")
        ) {
          client.setExceptionBreakpoints(session, next);
        }
        return next;
      });
    },
    [client],
  );

  const setBreakpointCondition = useCallback(
    (
      line: number,
      patch: { condition?: string; hit_condition?: string; log_message?: string },
    ) => {
      setBreakpoints((current) => {
        // An empty string clears; `undefined` in the patch leaves that field
        // alone, so a popover can send only what it edited. Trimmed here
        // rather than in the caller: a condition of three spaces is not a
        // condition, and an adapter handed one reports a parse error about
        // code the person never wrote.
        const clean = (value: string | undefined) => {
          const trimmed = value?.trim();
          return trimmed ? trimmed : undefined;
        };
        const next = current.map((breakpoint) =>
          breakpoint.line === line
            ? {
                ...breakpoint,
                condition:
                  "condition" in patch ? clean(patch.condition) : breakpoint.condition,
                hit_condition:
                  "hit_condition" in patch
                    ? clean(patch.hit_condition)
                    : breakpoint.hit_condition,
                log_message:
                  "log_message" in patch ? clean(patch.log_message) : breakpoint.log_message,
              }
            : breakpoint,
        );
        // Same rule as toggling: only a live session can be told, and the
        // condition is kept here for the next run either way.
        const session = sessionRef.current;
        if (
          client &&
          session &&
          (statusRef.current === "paused" || statusRef.current === "running")
        ) {
          clearFailure();
          client.setBreakpoints(session, onTheWire(next));
        }
        return next;
      });
    },
    [client, clearFailure],
  );

  const launchProfile = useRef<import("./launchProfiles").LaunchSelection>();
  const launchContext = useRef(projectContext);
  useEffect(() => { launchContext.current = projectContext; launchProfile.current = undefined; }, [uri, projectContext]);
  const start = useCallback(
    (program?: string, sources: Record<string, HeldBreakpoint[]> = {}, selection?: string, profile?: import("./launchProfiles").LaunchSelection) => {
      if (!client) return;
      setStatus("starting");
      clearFailure();
      setProgram(program ?? null);
      const initial = {...sourceBreakpoints,...sources};
      holdSources(initial);
      launchContext.current = selection ?? launchContext.current;
      launchProfile.current = profile ?? (selection !== undefined ? undefined : launchProfile.current);
      setLaunchTarget(launchProfile.current?.name ?? null);
      client.start(uri, onTheWire(breakpoints), program, launchContext.current, Object.fromEntries(Object.entries(initial).filter(([,bps])=>bps.length).map(([source,bps])=>[source,onTheWire(bps)])), launchProfile.current);
    },
    [client, uri, breakpoints, sourceBreakpoints, clearFailure, projectContext],
  );

  const stop = useCallback(() => {
    if (client && statusRef.current === "starting") client.cancelStart(uri);
    if (client && sessionRef.current) {
      client.stop(sessionRef.current);
      return;
    }
    // No session to tell — the server already reaped it when the program
    // finished (or the start failed). Stop is then only a dismissal: clear
    // the strip's chrome locally, sending nothing to a session that is gone.
    setStatus("idle");
    clearFailure();
    setCapabilities(null);
    setExitCode(null);
  }, [client, uri]);

  const step = useCallback(
    (how: Step) => {
      if (!client || !sessionRef.current) return;
      setStatus("running");
      client.step(sessionRef.current, how, selectedFrame ?? undefined);
    },
    [client, selectedFrame],
  );

  const jumpTo = useCallback(
    (line: number) => {
      if (!client || !sessionRef.current) return;
      setStatus("running");
      client.jump(sessionRef.current, line);
    },
    [client],
  );

  const runTo = useCallback(
    (line: number) => {
      if (!client || !sessionRef.current) return;
      setStatus("running");
      client.runTo(sessionRef.current, line);
    },
    [client],
  );

  const evaluate = useCallback(
    (expression: string, context: "hover" | "watch" | "repl" = "repl") => {
      if (!client || !sessionRef.current) return;
      client.evaluate(sessionRef.current, expression, context, selectedFrame ?? undefined);
    },
    [client, selectedFrame],
  );

  const valueAt = useCallback(
    (expression: string): Promise<string | null> => {
      if (!client || !sessionRef.current || status !== "paused") return Promise.resolve(null);
      // Only names this frame has. The debugger is not a spell-checker: asking
      // it about every word the pointer crosses — a word in the prose, a tag
      // name, a comment — earns a `NameError` per hover, and the error is
      // about our question rather than about the program. The frame's own
      // variables are the static analysis that matters here, and they are
      // already in hand.
      const root = expression.split(/[.[]/)[0];
      if (!variables.some((variable) => variable.name === root)) {
        return Promise.resolve(null);
      }
      return new Promise((resolve) => {
        pendingValues.current.set(expression, resolve);
        client.evaluate(sessionRef.current!, expression, "hover", selectedFrame ?? undefined);
        // A hover that never answers must not leave the tooltip waiting
        // forever on a program that has moved on.
        setTimeout(() => {
          if (pendingValues.current.delete(expression)) resolve(null);
        }, 2000);
      });
    },
    [client, status, selectedFrame, variables],
  );

  const query = useCallback(
    (expression: string): Promise<string | null> => {
      if (!client || !sessionRef.current || statusRef.current !== "paused") {
        return Promise.resolve(null);
      }
      return new Promise((resolve) => {
        pendingValues.current.set(expression, resolve);
        // `repl` context: the person typed this, so a side effect is their
        // business — unlike a hover, which must never have one.
        client.evaluate(sessionRef.current!, expression, "repl", selectedFrame ?? undefined);
        setTimeout(() => {
          if (pendingValues.current.delete(expression)) resolve(null);
        }, 5000);
      });
    },
    [client, selectedFrame],
  );

  const addWatch = useCallback(
    (expression: string) => {
      const trimmed = expression.trim();
      if (!trimmed) return;
      setWatches((current) =>
        current.some((watch) => watch.expression === trimmed)
          ? current
          : [...current, { expression: trimmed, value: null }],
      );
      // Answer immediately when there is a frame to ask; otherwise the
      // re-evaluate effect covers it at the next pause.
      if (client && sessionRef.current && statusRef.current === "paused") {
        client.evaluate(sessionRef.current, trimmed, "watch", selectedFrame ?? undefined);
      }
    },
    [client, selectedFrame],
  );

  const removeWatch = useCallback((expression: string) => {
    setWatches((current) => current.filter((watch) => watch.expression !== expression));
  }, []);

  // Every pause and every frame change re-asks every watch: a watch showing
  // the value from two steps ago is worse than no watch at all. Keyed on the
  // joined expressions rather than the array so an answer arriving (which
  // replaces the array) does not re-ask the question it answers.
  const watchKey = watches.map((watch) => watch.expression).join("\n");
  useEffect(() => {
    if (status !== "paused" || !client || !sessionRef.current) return;
    for (const expression of watchKey ? watchKey.split("\n") : []) {
      client.evaluate(sessionRef.current, expression, "watch", selectedFrame ?? undefined);
    }
  }, [status, selectedFrame, watchKey, client]);

  const selectFrame = useCallback(
    (id: number) => {
      setSelectedFrame(id);
      const frame = frames.find((candidate) => candidate.id === id);
      // Selecting a frame moves the paused marker with it, because the line
      // you are looking at should be the line whose values you are reading.
      if (frame?.line !== undefined) setPausedLine(frame.line);
      if (capabilities?.frame_locals && client && sessionRef.current) client.state(sessionRef.current, id);
    },
    [frames, capabilities, client],
  );

  // One object per distinct state, not one per render: the document session
  // carries this as a field and is published by identity, so a fresh object
  // on every render would re-publish the session — and re-render the whole
  // workspace — on every keystroke of the document beside it.
  const controller = useMemo(
    () => ({
      reading,
      status,
      message,
      offerInstall,
      program,
      launchTarget,
      capabilities,
      pausedLine,
      frames,
      variables,
      selectedFrame,
      breakpoints,
      sourceBreakpoints, setSourceBreakpoints,
      replaceBreakpoints: setBreakpoints,
      exitCode,
      buildOutput,
      lastValue,
      watches,
      exceptionFilters,
      start,
      stop,
      step,
      jumpTo,
      runTo,
      toggleBreakpoint,
      toggleExceptionFilter,
      setBreakpointCondition,
      selectFrame,
      evaluate,
      valueAt,
      query,
      addWatch,
      removeWatch,
    }),
    [
      reading,
      status,
      message,
      offerInstall,
      program,
      launchTarget,
      capabilities,
      pausedLine,
      frames,
      variables,
      selectedFrame,
      breakpoints,
      sourceBreakpoints, setSourceBreakpoints,
      exitCode,
      buildOutput,
      lastValue,
      watches,
      exceptionFilters,
      start,
      stop,
      step,
      jumpTo,
      runTo,
      toggleBreakpoint,
      toggleExceptionFilter,
      setBreakpointCondition,
      selectFrame,
      evaluate,
      valueAt,
      query,
      addWatch,
      removeWatch,
    ],
  );
  useDebugActions(docPath, controller, () => eventEpoch.current, !!client, uiSource);
  return controller;
}
</storycode:copy>

### Assemble `apps/web/src/debug/useDebugger.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/useDebugger.ts"><storycode:paste select="#f-597e991b795b-DebugStatus"/><storycode:paste select="#f-597e991b795b-Watch"/><storycode:paste select="#f-597e991b795b-DebugSession"/><storycode:paste select="#f-597e991b795b-HeldBreakpoint"/><storycode:paste select="#f-597e991b795b-isConditional"/><storycode:paste select="#f-597e991b795b-followMoves"/><storycode:paste select="#f-597e991b795b-onTheWire"/><storycode:paste select="#f-597e991b795b-eventIsOurs"/><storycode:paste select="#f-597e991b795b-IDLE_SESSION"/><storycode:paste select="#f-597e991b795b-useDebugger"/><storycode:paste select="#f-597e991b795b-workspaceClient"/><storycode:paste select="#f-597e991b795b-workspaceDebugClient"/><storycode:paste select="#f-597e991b795b-resetWorkspaceDebugger"/><storycode:paste select="#f-597e991b795b-useWorkspaceDebugger"/><storycode:paste select="#f-597e991b795b-useDebuggerOver"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-101aa3d58173"></a>
## `apps/web/src/debug/useDebugger.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Breakpoint Can Carry A Condition](../../../../../guarantees/debugging/a-breakpoint-can-carry-a-condition/contract.md), [Install a supported missing debugger and resume the requested launch](../../../../../guarantees/debugging/a-missing-debugger-is-a-button/contract.md), [A Plain File Has The Same Debugger A Document Has](../../../../../guarantees/debugging/a-plain-file-has-the-same-debugger/contract.md), [An Exception Breakpoint Uses The Adapter's Own Filters](../../../../../guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters/contract.md), [Debug the latest atomic project that still contains the displayed bytes](../../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md).

<a id="f-101aa3d58173-FakeSocket"></a>
### `FakeSocket`

<storycode:copy id="f-101aa3d58173-FakeSocket">
import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { Realtime } from "../api/realtime";
import { DebugClient } from "./client";
import { eventIsOurs, useDebugger, useDebuggerOver } from "./useDebugger";

/**
 * A socket that records what went out and can push frames back in.
 *
 * The point of testing through this rather than through `DebugClient`
 * directly: the bug this file exists for was that nothing ever registered the
 * client with the socket. Requests went out, every answer was dropped, and the
 * window sat on "starting…" — invisible to any test of the pieces on their own.
 */
class FakeSocket {
  readonly serverAuthoritative = true;
  sent: unknown[] = [];
  private handler: ((frame: Uint8Array) => boolean) | null = null;

  // The rest of the Realtime surface, which the debugger never touches.
  lsp() {
    return null;
  }
  onRunEvent() {
    return () => {};
  }
  bindDoc() {}
  whenSynced() {
    return Promise.resolve();
  }
  close() {}

  debug() {
    return {
      send: (frame: Uint8Array) => {
        this.sent.push(JSON.parse(new TextDecoder().decode(frame.subarray(1))));
      },
    };
  }

  onDebugFrame(handler: (frame: Uint8Array) => boolean) {
    this.handler = handler;
  }

  /** What the engine says, arriving as a real `0x03` frame. */
  deliver(event: Record<string, unknown>) {
    const body = new TextEncoder().encode(JSON.stringify(event));
    const frame = new Uint8Array(body.length + 1);
    frame[0] = 0x03;
    frame.set(body, 1);
    this.handler?.(frame);
  }
}

</storycode:copy>
<a id="f-101aa3d58173-STARTED"></a>
### `STARTED`

<storycode:copy id="f-101aa3d58173-STARTED">
const STARTED = {
  event: "started",
  session: "dbg-0",
  capabilities: {
    conditional_breakpoints: true,
    hit_conditional_breakpoints: true,
    log_points: true,
    set_variable: true,
    restart_frame: false,
    step_in_targets: true,
    goto_targets: true,
    step_back: false,
    exception_filters: [],
  },
  breakpoints: [{ line: 25, state: "bound" }],
};

</storycode:copy>
<a id="f-101aa3d58173-STOPPED"></a>
### `STOPPED`

<storycode:copy id="f-101aa3d58173-STOPPED">
const STOPPED = {
  event: "stopped",
  session: "dbg-0",
  reason: "breakpoint",
  line: 14,
  frames: [{ id: 2, name: "line_total", line: 14, source: "orders.py", in_document: true }],
  variables: [{ name: "quantity", value: "2", type_name: "int", variables_reference: 0 }],
};

</storycode:copy>
<a id="f-101aa3d58173-open"></a>
### `open`

<storycode:copy id="f-101aa3d58173-open">
function open() {
  const socket = new FakeSocket();
  const hook = renderHook(() => useDebugger(socket as unknown as Realtime, "orders.md"));
  return { socket, hook };
}

</storycode:copy>
<a id="f-101aa3d58173-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-101aa3d58173-expression_statement">
describe("the debugger, over the socket", () => {
  it("keeps source breakpoint replies separate from the caller and includes them in a fresh launch", async () => {
    const {socket,hook}=open();
    act(()=>hook.result.current.start());
    act(()=>{socket.deliver(STARTED);socket.deliver(STOPPED);});
    const caller=hook.result.current.breakpoints;
    act(()=>hook.result.current.setSourceBreakpoints("hick:///helpers.py",[{line:25,state:"pending",condition:"x > 1"}]));
    expect(socket.sent.at(-1)).toEqual({op:"source_breakpoints",session:"dbg-0",source:"hick:///helpers.py",breakpoints:[{line:25,condition:"x > 1"}]});
    act(()=>socket.deliver({event:"source_breakpoints",session:"dbg-0",source:"hick:///helpers.py",breakpoints:[{line:25,state:"bound",moved_to:26}]}));
    expect(hook.result.current.sourceBreakpoints["hick:///helpers.py"]).toMatchObject([{line:26,state:"bound",condition:"x > 1"}]);
    expect(hook.result.current.breakpoints).toBe(caller);
    act(()=>socket.deliver({event:"failed",session:"dbg-0",doc:"hick:///helpers.py",about:"source_breakpoints",message:"adapter rejected the update"}));
    expect(hook.result.current.status).toBe("paused");
    expect(hook.result.current.breakpoints).toBe(caller);
    expect(hook.result.current.sourceBreakpoints["hick:///helpers.py"][0].state).toBe("refused");
    expect(hook.result.current.message).toContain("helpers.py");
    act(()=>socket.deliver({event:"finished",session:"dbg-0"}));
    act(()=>hook.result.current.start());
    expect(socket.sent.at(-1)).toMatchObject({op:"start",sources:{"hick:///helpers.py":[{line:26,condition:"x > 1"}]}});
    act(()=>{socket.deliver(STARTED);socket.deliver({event:"finished",session:"dbg-0"});});
    act(()=>hook.result.current.start(undefined,{"hick:///helpers.py":[]}));
    expect(socket.sent.at(-1)).not.toHaveProperty("sources");
  });
  it("leaves 'starting' when the engine answers", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    expect(hook.result.current.status).toBe("starting");
    expect(socket.sent[0]).toMatchObject({ op: "start", doc: "hick:///orders.md" });

    act(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));
    expect(hook.result.current.capabilities?.goto_targets).toBe(true);
  });

  it("shows the paused line and the frame's values", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver(STOPPED));

    await waitFor(() => expect(hook.result.current.status).toBe("paused"));
    expect(hook.result.current.pausedLine).toBe(14);
    expect(hook.result.current.frames[0].name).toBe("line_total");
    expect(hook.result.current.variables[0].value).toBe("2");
    // The top frame is selected, so evaluating goes to the right one.
    expect(hook.result.current.selectedFrame).toBe(2);
  });

  it("sends the session id with every later request", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver(STOPPED));

    act(() => hook.result.current.step("over"));
    act(() => hook.result.current.evaluate("quantity * 2"));
    act(() => hook.result.current.jumpTo(12));

    expect(socket.sent.slice(1)).toEqual([
      { op: "step", session: "dbg-0", how: "over", frame: 2 },
      { op: "eval", session: "dbg-0", expression: "quantity * 2", context: "repl", frame: 2 },
      { op: "jump", session: "dbg-0", line: 12 },
    ]);
  });

  it("does nothing before a session exists", () => {
    // Stepping with no session would be a request the engine answers with an
    // error, and the error would be the UI's fault.
    const { socket, hook } = open();
    act(() => hook.result.current.step("over"));
    act(() => hook.result.current.evaluate("x"));
    expect(socket.sent).toEqual([]);
  });

  it("keeps breakpoints across a session, and sends them when one starts", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.toggleBreakpoint(14));
    act(() => hook.result.current.start());
    expect(socket.sent.at(-1)).toMatchObject({ breakpoints: [{ line: 14 }] });

    act(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));
    // The engine's answer replaces the optimistic set: it knows which bound.
    expect(hook.result.current.breakpoints).toEqual([{ line: 25, state: "bound" }]);
  });

  it("reports a failure instead of waiting forever", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() =>
      socket.deliver({
        event: "failed",
        message: "no debug adapter for python on this machine.",
      }),
    );
    await waitFor(() => expect(hook.result.current.status).toBe("failed"));
    expect(hook.result.current.message).toContain("no debug adapter");
  });

  it("carries a fetchable tool out of the failure, and drops it when one starts", async () => {
    // Protects docs/guarantees/debugging/a-missing-debugger-is-a-button.md
    //
    // The second half is the bug this pairing exists to prevent: installing
    // the adapter started the session successfully and left "No Python
    // debugger on this machine" sitting beside "finished — exit code 0".
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() =>
      socket.deliver({
        event: "failed",
        message: "no debug adapter for python on this machine.",
        offer_install: { kind: "dap", language: "python" },
      }),
    );
    await waitFor(() =>
      expect(hook.result.current.offerInstall).toEqual({ kind: "dap", language: "python" }),
    );

    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));
    expect(hook.result.current.offerInstall).toBeNull();
    expect(hook.result.current.message).toBeNull();
  });

  it("asks only about names the frame has", async () => {
    // Hovering prose, a tag name or a comment used to send the word to the
    // debugger and get a `NameError` back — an error about our question, not
    // about the program.
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver(STOPPED));
    const before = socket.sent.length;

    await act(async () => {
      expect(await hook.result.current.valueAt("order")).toBeNull();
      expect(await hook.result.current.valueAt("hick:file")).toBeNull();
    });
    expect(socket.sent.length).toBe(before);

    // A name that IS in the frame goes out, attribute access included.
    void hook.result.current.valueAt("quantity");
    void hook.result.current.valueAt("quantity.bit_length");
    expect(socket.sent.slice(before)).toEqual([
      { op: "eval", session: "dbg-0", expression: "quantity", context: "hover", frame: 2 },
      {
        op: "eval",
        session: "dbg-0",
        expression: "quantity.bit_length",
        context: "hover",
        frame: 2,
      },
    ]);
  });

  it("keeps a hover's failure out of the panel", async () => {
    // The panel is for what happened to the program. An unanswerable hover
    // resolves to nothing and the tooltip falls back to the type.
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver(STOPPED));

    let answered: string | null = "unset";
    act(() => {
      void hook.result.current.valueAt("quantity").then((v) => {
        answered = v;
      });
    });
    act(() => socket.deliver({ event: "failed", session: "dbg-0", message: "NameError: nope" }));
    await waitFor(() => expect(answered).toBeNull());
    expect(hook.result.current.message).toBeNull();
    // And the session is still alive: a hover failing is not a session failing.
    expect(hook.result.current.status).toBe("paused");
  });

  it("marks the breakpoint broken instead of shouting in the panel", async () => {
    // The reported bug: a refused `setBreakpoints` left the dot looking set,
    // put the reason in the panel, and left it there — so the next breakpoint
    // inherited a message about the previous one.
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver(STOPPED));
    act(() => hook.result.current.toggleBreakpoint(14));

    act(() =>
      socket.deliver({
        event: "failed",
        session: "dbg-0",
        about: "breakpoints",
        lines: [14],
        message: "the debug adapter refused `setBreakpoints`: Server disconnected unexpectedly",
      }),
    );

    await waitFor(() =>
      expect(hook.result.current.breakpoints.find((b) => b.line === 14)?.state).toBe("refused"),
    );
    // The reason lives on the breakpoint, for its hover.
    expect(hook.result.current.breakpoints.find((b) => b.line === 14)?.message).toContain(
      "disconnected",
    );
    // And nowhere else.
    expect(hook.result.current.message).toBeNull();
    // Only that line is affected.
    expect(hook.result.current.breakpoints.find((b) => b.line === 25)?.state).toBe("bound");
  });

  it("keeps a breakpoint local once the program has ended", async () => {
    // Setting one after the program finished used to send it to a session
    // that could only refuse: the breakpoint is for the next run.
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver({ event: "finished", session: "dbg-0" }));
    await waitFor(() => expect(hook.result.current.status).toBe("finished"));
    const before = socket.sent.length;

    act(() => hook.result.current.toggleBreakpoint(14));
    expect(socket.sent.length).toBe(before);
    expect(hook.result.current.breakpoints.some((b) => b.line === 14)).toBe(true);
  });

  it("re-asks every watch at each pause and frame change", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver(STOPPED));
    await waitFor(() => expect(hook.result.current.status).toBe("paused"));

    // Adding a watch while paused asks straight away, in `watch` context so
    // the adapter treats it as something evaluated repeatedly.
    act(() => hook.result.current.addWatch("quantity * 2"));
    expect(socket.sent.at(-1)).toEqual({
      op: "eval",
      session: "dbg-0",
      expression: "quantity * 2",
      context: "watch",
      frame: 2,
    });

    // The answer correlates by the expression it echoes back.
    act(() =>
      socket.deliver({
        event: "value",
        session: "dbg-0",
        expression: "quantity * 2",
        value: "4",
        type: "int",
        reference: 0,
      }),
    );
    await waitFor(() =>
      expect(hook.result.current.watches).toEqual([{ expression: "quantity * 2", value: "4" }]),
    );

    // A step that pauses again re-asks, unprompted: a watch showing the value
    // from two steps ago is worse than no watch at all.
    act(() => hook.result.current.step("over"));
    const before = socket.sent.length;
    act(() => socket.deliver({ ...STOPPED, line: 15 }));
    await waitFor(() =>
      expect(
        socket.sent
          .slice(before)
          .some((sent) => (sent as { context?: string }).context === "watch"),
      ).toBe(true),
    );
  });

  it("keeps watch expressions but drops their values when the program ends", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.addWatch("quantity"));
    act(() => hook.result.current.addWatch("quantity")); // and never twice
    expect(hook.result.current.watches).toEqual([{ expression: "quantity", value: null }]);

    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver(STOPPED));
    act(() =>
      socket.deliver({
        event: "value",
        session: "dbg-0",
        expression: "quantity",
        value: "2",
        type: "int",
        reference: 0,
      }),
    );
    await waitFor(() => expect(hook.result.current.watches[0].value).toBe("2"));

    // The expression is for the next run; the value belonged to this one.
    act(() => socket.deliver({ event: "finished", session: "dbg-0" }));
    await waitFor(() => expect(hook.result.current.watches).toEqual([
      { expression: "quantity", value: null },
    ]));

    act(() => hook.result.current.removeWatch("quantity"));
    expect(hook.result.current.watches).toEqual([]);
  });

  it("answers a typed expression as a promise, without the known-name guard", async () => {
    // `valueAt` refuses names the frame does not have (it exists for hover);
    // the inline eval is a person typing, and any expression is fair.
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver(STOPPED));
    await waitFor(() => expect(hook.result.current.status).toBe("paused"));

    let answered: string | null = null;
    act(() => {
      void hook.result.current.query("len('abc')").then((value) => {
        answered = value;
      });
    });
    expect(socket.sent.at(-1)).toEqual({
      op: "eval",
      session: "dbg-0",
      expression: "len('abc')",
      context: "repl",
      frame: 2,
    });
    act(() =>
      socket.deliver({
        event: "value",
        session: "dbg-0",
        expression: "len('abc')",
        value: "3",
        type: "int",
        reference: 0,
      }),
    );
    await waitFor(() => expect(answered).toBe("3"));
  });

  it("forgets the session when the program finishes, and sends it nothing more", async () => {
    // The bug's visible half: the program ran to its end, the server reaped
    // the session — and the UI kept the id, a live-looking strip, and a Stop
    // that earned an error from a session that no longer existed.
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver(STOPPED));
    act(() => socket.deliver({ event: "finished", session: "dbg-0", exit_code: 0 }));

    await waitFor(() => expect(hook.result.current.status).toBe("finished"));
    // The end says how it ended, and the paused chrome is gone.
    expect(hook.result.current.exitCode).toBe(0);
    expect(hook.result.current.pausedLine).toBeNull();
    expect(hook.result.current.frames).toEqual([]);
    expect(hook.result.current.variables).toEqual([]);

    // No request may reach the dead session: not a step, not an eval, not a
    // hover — each would be answered only by an error about a gone session.
    const before = socket.sent.length;
    act(() => hook.result.current.step("over"));
    act(() => hook.result.current.evaluate("quantity"));
    act(() => hook.result.current.jumpTo(12));
    await act(async () => {
      expect(await hook.result.current.valueAt("quantity")).toBeNull();
      expect(await hook.result.current.query("1 + 1")).toBeNull();
    });
    expect(socket.sent.length).toBe(before);

    // Stop is now a dismissal: nothing goes out, the strip clears locally.
    act(() => hook.result.current.stop());
    expect(socket.sent.length).toBe(before);
    await waitFor(() => expect(hook.result.current.status).toBe("idle"));
  });

  it("does not send a stop for an already-finished session on unmount", async () => {
    // The unmount cleanup exists for a window closed MID-session. After
    // "finished" there is nothing to stop, and sending one anyway is a
    // request to a session the server already reaped.
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver({ event: "finished", session: "dbg-0", exit_code: 0 }));
    await waitFor(() => expect(hook.result.current.status).toBe("finished"));

    const before = socket.sent.length;
    hook.unmount();
    expect(socket.sent.length).toBe(before);
  });

  it("keeps 'finished' when a stray failure arrives afterwards", async () => {
    // A hover or watch answered late, after the program ended, must not
    // redress a clean end as a broken one.
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver({ event: "finished", session: "dbg-0", exit_code: 0 }));
    await waitFor(() => expect(hook.result.current.status).toBe("finished"));

    act(() => socket.deliver({ event: "failed", session: "dbg-0", message: "too late" }));
    expect(hook.result.current.status).toBe("finished");
  });

  it("goes back to idle when the session ends", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver(STOPPED));
    act(() => socket.deliver({ event: "ended", session: "dbg-0" }));

    await waitFor(() => expect(hook.result.current.status).toBe("idle"));
    expect(hook.result.current.pausedLine).toBeNull();
    expect(hook.result.current.frames).toEqual([]);
  });
});

</storycode:copy>
<a id="f-101aa3d58173-expression_statement-2"></a>
### `expression_statement`

Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md

<storycode:copy id="f-101aa3d58173-expression_statement-2">
// Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
describe("two panes on one socket", () => {
  const frame = (event: Record<string, unknown>) => {
    const body = new TextEncoder().encode(JSON.stringify(event));
    const out = new Uint8Array(body.length + 1);
    out[0] = 0x03;
    out.set(body, 1);
    return out;
  };

  it("each takes only its own session", async () => {
    // Every plain file's pane shares the workspace connection, so the
    // stream carries everybody's sessions. A pane that took a neighbour's
    // `stopped` would show itself paused in a program it never started.
    const sent: unknown[] = [];
    const client = new DebugClient({
      send: (bytes) => sent.push(JSON.parse(new TextDecoder().decode(bytes.subarray(1)))),
    });
    const a = renderHook(() => useDebuggerOver(client, "tools/a.py"));
    const b = renderHook(() => useDebuggerOver(client, "tools/b.py"));

    act(() => a.result.current.start());
    expect(sent[0]).toMatchObject({ op: "start", doc: "hick:///tools/a.py" });
    act(() => {
      client.handleFrame(frame({ ...STARTED, doc: "hick:///tools/a.py" }));
    });
    await waitFor(() => expect(a.result.current.status).toBe("running"));
    expect(b.result.current.status).toBe("idle");

    act(() => {
      client.handleFrame(frame(STOPPED));
    });
    await waitFor(() => expect(a.result.current.status).toBe("paused"));
    expect(b.result.current.status).toBe("idle");
    expect(b.result.current.pausedLine).toBeNull();

    // A start that fails names the file it was for, and lands there only.
    act(() => b.result.current.start());
    act(() => {
      client.handleFrame(
        frame({
          event: "failed",
          session: null,
          doc: "hick:///tools/b.py",
          message: "no debug adapter for python on this machine",
          about: "start",
        }),
      );
    });
    await waitFor(() => expect(b.result.current.status).toBe("failed"));
    expect(a.result.current.status).toBe("paused");
    expect(a.result.current.message).toBeNull();
  });

  it("still takes an answer that names no file", () => {
    // A document's own socket carries one session and an older engine may
    // not say which; that must keep working.
    expect(eventIsOurs({ ...STARTED, event: "started" } as never, "hick:///x.md", null)).toBe(true);
    expect(eventIsOurs(STOPPED as never, "hick:///x.md", "dbg-0")).toBe(true);
    expect(eventIsOurs(STOPPED as never, "hick:///x.md", null)).toBe(false);
    expect(eventIsOurs(STOPPED as never, "hick:///x.md", "dbg-9")).toBe(false);
  });
});


</storycode:copy>
<a id="f-101aa3d58173-expression_statement-3"></a>
### `expression_statement`

Protects docs/guarantees/debugging/a-breakpoint-can-carry-a-condition.md
`hick-dap::Breakpoint` has carried `condition`, `hit_condition` and
`log_message` since the debugger was written, and the server deserializes
all three. Nothing could set one: the hook sent `{ line }` and nothing else,
and `debugStateEffects` hardcoded `conditional: false`.

<storycode:copy id="f-101aa3d58173-expression_statement-3">
// Protects docs/guarantees/debugging/a-breakpoint-can-carry-a-condition.md
//
// `hick-dap::Breakpoint` has carried `condition`, `hit_condition` and
// `log_message` since the debugger was written, and the server deserializes
// all three. Nothing could set one: the hook sent `{ line }` and nothing else,
// and `debugStateEffects` hardcoded `conditional: false`.
describe("a breakpoint carries what the person attached to it", () => {
  it("sends a condition on the wire, and only what was set", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    act(() => hook.result.current.toggleBreakpoint(10));
    act(() => hook.result.current.setBreakpointCondition(10, { condition: "i > 10" }));

    const last = socket.sent.at(-1) as { op: string; breakpoints: Record<string, unknown>[] };
    expect(last.op).toBe("breakpoints");
    const sent = last.breakpoints.find((b) => b.line === 10)!;
    expect(sent).toEqual({ line: 10, condition: "i > 10" });
    // An adapter must never be handed an empty string to parse.
    expect(sent).not.toHaveProperty("hit_condition");
    expect(sent).not.toHaveProperty("log_message");
  });

  it("carries a hit count and a log message too", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    act(() => hook.result.current.toggleBreakpoint(4));
    act(() =>
      hook.result.current.setBreakpointCondition(4, {
        hit_condition: "> 5",
        log_message: "here with i={i}",
      }),
    );
    const last = socket.sent.at(-1) as { breakpoints: Record<string, unknown>[] };
    expect(last.breakpoints.find((b) => b.line === 4)).toEqual({
      line: 4,
      hit_condition: "> 5",
      log_message: "here with i={i}",
    });
  });

  it("clears a condition when it is emptied", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    act(() => hook.result.current.toggleBreakpoint(7));
    act(() => hook.result.current.setBreakpointCondition(7, { condition: "x" }));
    act(() => hook.result.current.setBreakpointCondition(7, { condition: "  " }));

    const last = socket.sent.at(-1) as { breakpoints: Record<string, unknown>[] };
    expect(last.breakpoints.find((b) => b.line === 7)).toEqual({ line: 7 });
    expect(hook.result.current.breakpoints.find((b) => b.line === 7)?.condition).toBeUndefined();
  });

  it("keeps the condition when the adapter moves the breakpoint", async () => {
    // The subtle one. A breakpoint that slides to the next executable line
    // and silently loses its condition would stop on every pass — the exact
    // opposite of what was asked for.
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    act(() => hook.result.current.toggleBreakpoint(12));
    act(() => hook.result.current.setBreakpointCondition(12, { condition: "n == 3" }));
    act(() =>
      socket.deliver({
        event: "breakpoints",
        session: "dbg-0",
        breakpoints: [{ line: 12, state: "bound", moved_to: 13 }],
      }),
    );

    const held = hook.result.current.breakpoints;
    expect(held).toHaveLength(1);
    expect(held[0].line).toBe(13);
    expect(held[0].condition).toBe("n == 3");
  });

  it("keeps a condition set before the program runs, and starts with it", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.toggleBreakpoint(3));
    act(() => hook.result.current.setBreakpointCondition(3, { condition: "ready" }));
    // Nothing is running, so nothing was sent — the condition is for the run.
    expect(socket.sent).toHaveLength(0);

    act(() => hook.result.current.start());
    expect(socket.sent[0]).toMatchObject({
      op: "start",
      breakpoints: [{ line: 3, condition: "ready" }],
    });
  });
});

</storycode:copy>
<a id="f-101aa3d58173-expression_statement-4"></a>
### `expression_statement`

Protects docs/guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters.md

<storycode:copy id="f-101aa3d58173-expression_statement-4">
// Protects docs/guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters.md
describe("exception breakpoints", () => {
  it("sends the filters that are on, by id", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    act(() => hook.result.current.toggleExceptionFilter("uncaught"));
    expect(socket.sent.at(-1)).toEqual({
      op: "exception_breakpoints",
      session: "dbg-0",
      filters: ["uncaught"],
    });

    act(() => hook.result.current.toggleExceptionFilter("raised"));
    expect(socket.sent.at(-1)).toMatchObject({ filters: ["uncaught", "raised"] });

    act(() => hook.result.current.toggleExceptionFilter("uncaught"));
    expect(socket.sent.at(-1)).toMatchObject({ filters: ["raised"] });
    expect(hook.result.current.exceptionFilters).toEqual(["raised"]);
  });

  it("keeps them across runs and re-applies them to the next session", async () => {
    // A standing "stop on uncaught" that quietly stopped applying on the
    // second run would be worse than never having offered it.
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));
    act(() => hook.result.current.toggleExceptionFilter("uncaught"));

    act(() => socket.deliver({ event: "finished", session: "dbg-0", exit_code: 0 }));
    await waitFor(() => expect(hook.result.current.status).toBe("finished"));
    expect(hook.result.current.exceptionFilters).toEqual(["uncaught"]);

    act(() => hook.result.current.start());
    act(() => socket.deliver({ ...STARTED, session: "dbg-1" }));
    await waitFor(() =>
      expect(socket.sent).toContainEqual({
        op: "exception_breakpoints",
        session: "dbg-1",
        filters: ["uncaught"],
      }),
    );
  });
});

</storycode:copy>
<a id="f-101aa3d58173-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-101aa3d58173-expression_statement-5">
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
it("keeps the clicked source context for restart and replaces it for a different surface", () => {
  const { socket, hook } = open();
  const context = 'latest-file:["base","<hick:project/>source"]';
  act(() => hook.result.current.start("app.py", {}, context));
  expect(socket.sent.at(-1)).toMatchObject({ op: "start", program: "app.py", project: context });
  act(() => hook.result.current.start("app.py"));
  expect(socket.sent.at(-1)).toMatchObject({ project: context });
  act(() => hook.result.current.start("other.py", {}, "new"));
  expect(socket.sent.at(-1)).not.toHaveProperty("project");
  hook.unmount();
});
</storycode:copy>

### Assemble `apps/web/src/debug/useDebugger.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/useDebugger.test.ts"><storycode:paste select="#f-101aa3d58173-FakeSocket"/><storycode:paste select="#f-101aa3d58173-STARTED"/><storycode:paste select="#f-101aa3d58173-STOPPED"/><storycode:paste select="#f-101aa3d58173-open"/><storycode:paste select="#f-101aa3d58173-expression_statement"/><storycode:paste select="#f-101aa3d58173-expression_statement-2"/><storycode:paste select="#f-101aa3d58173-expression_statement-3"/><storycode:paste select="#f-101aa3d58173-expression_statement-4"/><storycode:paste select="#f-101aa3d58173-expression_statement-5"/></storycode:file>


</storycode:doc>
