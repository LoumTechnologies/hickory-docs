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
import { DebugClient,DebugThreads,DebugLaunches,debugAdapterId } from "./client";
import {documentUri} from "../lsp/documentUri";
import {withDebugProtocol} from "./launchProfiles";
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
  output?: { category: string; text: string; source?: string | null; line?: number | null }[];
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
  threads?:import("./client").DebugThread[];
  selectedThread?:number|null;
  threadStop?:number|null;
  threadPending?:boolean;
  threadSelecting?:boolean;
  threadMessage?:string|null;
  frameVariablesPending?:boolean;
  frameVariablesMessage?:string|null;
  stackMessage?:string|null;
  historicalFrames?:Frame[];
  observationScope?:import("./observations").ObservationScope|null;
  captureFrame?(id:number):{frame:Frame;current:()=>boolean}|null;
  readThreads?():void;
  selectThread?(id:number):void;
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

Thread reads share the document session controller but fail independently of paused frames and locals. Source/frame changes invalidate held callbacks. Retire thread scope immediately when execution resumes; a late list must not attach to a later pause. Confirmed thread changes clear old evaluation values and re-evaluate watches for the new stop. Evidence uses an opaque scope tuple rather than the numeric debugger pause generation: hold early readings by their complete scope and promote only the exact Stopped scope. Startup preflights the read-only protocol catalog even for direct action-search requests; a caller-owned launch ticket and captured owner/project/profile retire delayed replies before they can send Start.

<storycode:copy id="f-597e991b795b-useDebuggerOver">
export function useDebuggerOver(client: DebugClient | null, docPath: string, initialBreakpoints: readonly number[] = [], projectContext = "new", uiSource?: unknown): DebugSession {
  const eventEpoch = useRef(0);
  const readingOwner = useRef(Symbol("debugger-capture"));
  const [, renderEventEpoch] = useState(0);
  const [reading, setReading] = useState<import("./observations").ObservationReading | null>(null);
  const [observationScope,setObservationScope]=useState<import("./observations").ObservationScope|null>(null),scopeRef=useRef<import("./observations").ObservationScope|null>(null),pendingReadings=useRef(new Map<string,import("./observations").ObservationReading>());
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
  const [frames, setFrames] = useState<Frame[]>([]),[historicalFrames,setHistoricalFrames]=useState<Frame[]>([]);
  const currentFrames=useRef<Frame[]>(frames);
  const [launches]=useState(()=>new DebugLaunches());
  const [threadModel]=useState(()=>new DebugThreads()),[threadRevision,paintThreads]=useState(0);
  const refreshThreads=()=>paintThreads(value=>value+1);
  const [variables, setVariables] = useState<Variable[]>([]);
  const localsByFrame=useRef(new Map<number,Variable[]>());
  const [frameVariablesPending,setFrameVariablesPending]=useState(false),[frameVariablesMessage,setFrameVariablesMessage]=useState<string|null>(null),[stackMessage,setStackMessage]=useState<string|null>(null);
  const ownerLifetime=useRef(0),ownerMounted=useRef(true);
  const frameRead=useRef<{session:string;stop:number;thread:number;frame:number;request:string}|null>(null),frameSequence=useRef(0);
  const [selectedFrame, setSelectedFrame] = useState<number | null>(null);
  const selectedFrameRef=useRef<number|null>(null);
  const evaluationSequence=useRef(0),evaluationRequests=useRef(new Map<string,{expression:string;stop:number;thread:number;frame:number|null}>()),latestEvaluation=useRef(new Map<string,string>());
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
  const [output, setOutput] = useState<NonNullable<DebugSession["output"]>>([]);

  const sessionRef = useRef<string | null>(null);
  // The status as of right now, for callbacks that must not close over a
  // stale one — a breakpoint toggled a moment after the program ended would
  // otherwise still be sent to a session that cannot answer.
  const statusRef = useRef<DebugStatus>("idle");
  // Hover asks for a value and wants it back; the channel answers with an
  // event, so the request is parked here until its answer arrives.
  const pendingValues = useRef(new Map<string, (value: string | null) => void>());

  function holdStatus(next:DebugStatus){statusRef.current=next;setStatus(next);}
  function observationKey(scope:import("./observations").ObservationScope){return JSON.stringify([scope.session??null,scope.run,scope.stop,scope.frame,scope.invocation]);}
  function retireObservationScope(){scopeRef.current=null;setObservationScope(null);pendingReadings.current.clear();}
  function retireFrameRead(){frameRead.current=null;setFrameVariablesPending(false);setFrameVariablesMessage(null);}
  function retireEvaluations(){evaluationRequests.current.clear();latestEvaluation.current.clear();for(const resolve of pendingValues.current.values())resolve(null);pendingValues.current.clear();setLastValue(null);}
  function askValue(session:string,expression:string,context:"hover"|"watch"|"repl",frame?:number){
    if(!client||threadModel.selecting)return;if(frame!==undefined&&!debugAdapterId(frame)){setMessage("Selected frame identity cannot be represented exactly by this connection.");return;}const scope=threadModel.scope;
    if(!scope){client.evaluate(session,expression,context,frame);return;}
    const request_id="eval-"+(++evaluationSequence.current),previous=latestEvaluation.current.get(expression);if(previous)evaluationRequests.current.delete(previous);
    if(evaluationRequests.current.size>=512){const oldest=evaluationRequests.current.keys().next().value;if(oldest)evaluationRequests.current.delete(oldest);}
    latestEvaluation.current.set(expression,request_id);evaluationRequests.current.set(request_id,{expression,stop:scope.stop,thread:scope.thread,frame:frame??null});client.evaluate(session,expression,context,frame,{stop:scope.stop,thread:scope.thread,request_id});
  }


  const uri = useMemo(() => documentUri(docPath), [docPath]);
  useEffect(() => { retainDebugReading(readingOwner.current, reading); return () => retainDebugReading(readingOwner.current, null); }, [reading]);

  useEffect(() => {
    if (!client) return;
    const off = client.on((event: DebugEvent) => {
      // A shared channel carries every pane's sessions; only ours is ours.
      // Before `started` there is no session to match, so a start that is
      // still in flight is recognised by the file it named.
      if (!eventIsOurs(event, uri, sessionRef.current)) return;
      if(event.event==="started"){
        if(!event.request_id&&launches.current&&statusRef.current==="starting"){setMessage("A debugger startup reply has no caller identity; this view has not adopted it and the request remains unconfirmed.");return;}
        const admission=launches.started(event.request_id,event.session);if(admission==="retired"){if(event.session!==sessionRef.current)client.stop(event.session);return;}if(admission!=="current")return;
      }else if(event.event==="build"||event.event==="failed"&&!event.session&&(event.about===undefined||event.about==="start")){
        if(!launches.matches(event.request_id))return;
      }
      eventEpoch.current++;
      renderEventEpoch(eventEpoch.current);
      switch (event.event) {
        case "run_output":
          setOutput(current => [...current, ...event.events].slice(-256));
          break;
        case "reading": {
          const key=observationKey(event.reading.scope);pendingReadings.current.set(key,event.reading);while(pendingReadings.current.size>16)pendingReadings.current.delete(pendingReadings.current.keys().next().value!);
          if(!scopeRef.current||observationKey(scopeRef.current)===key)setReading(event.reading);
          break;
        }
        case "build":
          // Replaces rather than appends: a build belongs to the start that
          // ran it, and the last one is the one on screen.
          setBuildOutput(event.events);
          break;
        case "started": {
          setReading(null);
          sessionRef.current = event.session;
          setOutput([]);
          setCapabilities(event.capabilities);
          setBreakpoints((current) => followMoves(current, event.breakpoints));
          threadModel.reset();retireObservationScope();currentFrames.current=[];setFrames([]);setHistoricalFrames([]);setVariables([]);selectedFrameRef.current=null;setSelectedFrame(null);setPausedLine(null);retireFrameRead();retireEvaluations();refreshThreads();holdStatus("running");
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
        case "threads":
          if(threadModel.accept(event))refreshThreads();else refreshThreads();
          break;
        case "stopped": {
          const previousThreadScope=threadModel.scope;if(!threadModel.stopped(event.session,event.stop,event.thread))break;refreshThreads();
          const samePause=!!previousThreadScope&&previousThreadScope===threadModel.scope,reportedFrame=event.selected_frame??event.frames[0]?.id??null;
          const chosenFrame=samePause&&event.frames.some(frame=>frame.id===selectedFrameRef.current)?selectedFrameRef.current:reportedFrame;
          if(!samePause||!!frameRead.current&&frameRead.current.frame!==chosenFrame)retireFrameRead();setStackMessage(event.stack_error??null);if(!samePause||chosenFrame===reportedFrame)setFrameVariablesMessage(event.variables_error??null);
          if(!samePause)localsByFrame.current.clear();if(reportedFrame!==null&&!event.variables_error)localsByFrame.current.set(reportedFrame,event.variables);
          if(previousThreadScope!==threadModel.scope){retireEvaluations();setWatches(current=>current.map(watch=>({...watch,value:null})));}
          const observed=event.observation_scope??(samePause?scopeRef.current:null);scopeRef.current=observed;setObservationScope(observed);
          setReading(current=>observed?(pendingReadings.current.get(observationKey(observed))??(current&&observationKey(current.scope)===observationKey(observed)?current:null)):null);
          holdStatus("paused");
          setPausedLine(chosenFrame===reportedFrame?event.line:event.frames.find(frame=>frame.id===chosenFrame)?.line??null);
          if(event.stack_error){const previous=currentFrames.current;setHistoricalFrames(held=>previous.length?previous:held);}else setHistoricalFrames([]);currentFrames.current=event.frames;setFrames(event.frames);
          setVariables(chosenFrame===reportedFrame?event.variables:localsByFrame.current.get(chosenFrame!)??[]);
          selectedFrameRef.current=chosenFrame;
          setSelectedFrame(selectedFrameRef.current);
          clearFailure();
          break;
        }
        case "breakpoints":
          setBreakpoints((current) => followMoves(current, event.breakpoints));
          break;
        case "source_breakpoints":
          holdSources(current=>({...current,[event.source]:followMoves(current[event.source] ?? [],event.breakpoints)}));
          break;
        case "frame_variables": {
          const requested=frameRead.current,scope=threadModel.scope;if(!requested||!scope||statusRef.current!=="paused"||event.request_id!==requested.request||event.stop!==scope.stop||event.thread!==scope.thread||event.frame!==selectedFrameRef.current||requested.stop!==scope.stop||requested.thread!==scope.thread||requested.frame!==event.frame)break;
          frameRead.current=null;setFrameVariablesPending(false);setFrameVariablesMessage(null);localsByFrame.current.set(event.frame,event.variables);setVariables(event.variables);break;
        }
        case "value": {
          if(statusRef.current!=="paused")break;
          const scope=threadModel.scope;
          if(scope){const request=event.request_id&&evaluationRequests.current.get(event.request_id);if(!request||request.expression!==event.expression||latestEvaluation.current.get(event.expression)!==event.request_id||event.stop!==scope.stop||event.thread!==scope.thread||event.frame!==selectedFrameRef.current||request.stop!==scope.stop||request.thread!==scope.thread||request.frame!==selectedFrameRef.current)break;evaluationRequests.current.delete(event.request_id!);latestEvaluation.current.delete(event.expression);}
          else if(event.stop!==undefined||event.thread!==undefined)break;
          setLastValue({ expression: event.expression, value: event.value, type: event.type_name??event.type??null });
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
          launches.retire();
          // The server reaped the session with the program: the id no longer
          // answers, so forget it NOW. A step, hover or stop sent after this
          // would be a request to a dead session, answered only by an error.
          sessionRef.current = null;
          threadModel.clear();retireObservationScope();retireFrameRead();retireEvaluations();refreshThreads();holdStatus("finished");
          setExitCode(event.exit_code ?? null);
          setPausedLine(null);
          currentFrames.current=[];setFrames([]);setHistoricalFrames([]);
          setVariables([]);
          setSelectedFrame(null);
          // The expressions survive — they are for the next run — but their
          // values belonged to a process that no longer exists.
          setWatches((current) => current.map((watch) => ({ ...watch, value: null })));
          break;
        case "ended":
          launches.retire();
          sessionRef.current = null;
          threadModel.clear();retireObservationScope();retireFrameRead();retireEvaluations();refreshThreads();holdStatus("idle");
          clearFailure();
          setPausedLine(null);
          currentFrames.current=[];setFrames([]);setHistoricalFrames([]);
          setVariables([]);
          setCapabilities(null);
          setWatches((current) => current.map((watch) => ({ ...watch, value: null })));
          break;
        case "failed": {
          if(event.about==="state"&&event.request_id){const requested=frameRead.current;if(!requested||requested.request!==event.request_id)break;frameRead.current=null;setFrameVariablesPending(false);setFrameVariablesMessage(event.message);break;}
          if(event.about==="threads"||event.about==="select_thread"){if(threadModel.failed(event.message,event.request_id))refreshThreads();break;}
          if (event.about === "source_breakpoints") {
            setMessage(`Could not update breakpoints in ${event.doc ?? "this source"}: ${event.message}`);
            const source=event.doc;
            if(source)holdSources(current=>({...current,[source]:(current[source] ?? []).map(bp=>({...bp,state:"refused" as const,message:event.message}))}));
            break;
          }
          if(event.about==="eval"&&event.request_id){const request=evaluationRequests.current.get(event.request_id);if(!request||latestEvaluation.current.get(request.expression)!==event.request_id)break;evaluationRequests.current.delete(event.request_id);latestEvaluation.current.delete(request.expression);const waiting=pendingValues.current.get(request.expression);pendingValues.current.delete(request.expression);waiting?.(null);setMessage(event.message);break;}
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
          if (!sessionRef.current && statusRef.current !== "finished") holdStatus("failed");
          break;
        }
      }
    });
    return off;
  }, [client, uri]);

  // A window that closes mid-session leaves a process running; the server
  // sweeps it, but saying so promptly is cheaper than waiting for the sweep.
  useEffect(() => {
    ownerMounted.current=true;ownerLifetime.current++;
    return () => {
      ownerMounted.current=false;ownerLifetime.current++;scopeRef.current=null;pendingReadings.current.clear();launches.retire();const session=sessionRef.current;sessionRef.current=null;statusRef.current="idle";currentFrames.current=[];frameRead.current=null;evaluationRequests.current.clear();latestEvaluation.current.clear();for(const resolve of pendingValues.current.values())resolve(null);pendingValues.current.clear();
      if (client && session) client.stop(session);
    };
  }, [client,uri]);

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
      const previous=sessionRef.current;const request_id=launches.begin();
      threadModel.clear();retireObservationScope();retireFrameRead();retireEvaluations();refreshThreads();holdStatus("starting");
      clearFailure();
      setProgram(program ?? null);
      const initial = {...sourceBreakpoints,...sources};
      holdSources(initial);
      launchContext.current = selection ?? launchContext.current;
      launchProfile.current = profile ?? (selection !== undefined ? undefined : launchProfile.current);
      setLaunchTarget(launchProfile.current?.name ?? null);
      const context=launchContext.current,chosen=launchProfile.current,lifetime=ownerLifetime.current;
      void withDebugProtocol({uri,project:context,program},()=>{if(!ownerMounted.current||ownerLifetime.current!==lifetime||launchContext.current!==context||launchProfile.current!==chosen||!launches.matches(request_id))return;sessionRef.current=null;if(previous)client.stop(previous);client.start(uri, onTheWire(breakpoints), program, launchContext.current, Object.fromEntries(Object.entries(initial).filter(([,bps])=>bps.length).map(([source,bps])=>[source,onTheWire(bps)])), chosen,request_id);}).catch(error=>{if(!ownerMounted.current||!launches.matches(request_id))return;launches.retire();holdStatus("failed");setMessage(error instanceof Error?error.message:String(error));});
    },
    [client, uri, breakpoints, sourceBreakpoints, clearFailure, projectContext],
  );

  const stop = useCallback(() => {
    launches.retire();
    if (client && statusRef.current === "starting") client.cancelStart(uri);
    if (client && sessionRef.current) {
      client.stop(sessionRef.current);
      return;
    }
    // No session to tell — the server already reaped it when the program
    // finished (or the start failed). Stop is then only a dismissal: clear
    // the strip's chrome locally, sending nothing to a session that is gone.
    threadModel.clear();retireObservationScope();retireFrameRead();retireEvaluations();refreshThreads();holdStatus("idle");
    clearFailure();
    setCapabilities(null);
    setExitCode(null);
  }, [client, uri]);

  const step = useCallback(
    (how: Step) => {
      if (!client || !sessionRef.current||threadModel.selecting) return;
      threadModel.clear();retireObservationScope();retireFrameRead();retireEvaluations();refreshThreads();holdStatus("running");
      client.step(sessionRef.current, how, selectedFrame ?? undefined);
    },
    [client, selectedFrame],
  );

  const jumpTo = useCallback(
    (line: number) => {
      if (!client || !sessionRef.current||threadModel.selecting) return;
      threadModel.clear();retireObservationScope();retireFrameRead();retireEvaluations();refreshThreads();holdStatus("running");
      client.jump(sessionRef.current, line);
    },
    [client],
  );

  const runTo = useCallback(
    (line: number) => {
      if (!client || !sessionRef.current||threadModel.selecting) return;
      threadModel.clear();retireObservationScope();retireFrameRead();retireEvaluations();refreshThreads();holdStatus("running");
      client.runTo(sessionRef.current, line);
    },
    [client],
  );

  const evaluate = useCallback(
    (expression: string, context: "hover" | "watch" | "repl" = "repl") => {
      if (!client || !sessionRef.current) return;
      askValue(sessionRef.current, expression, context, selectedFrame ?? undefined);
    },
    [client, selectedFrame],
  );

  const valueAt = useCallback(
    (expression: string): Promise<string | null> => {
      if (!client || !sessionRef.current || status !== "paused"||threadModel.selecting||selectedFrame===null||!debugAdapterId(selectedFrame)) return Promise.resolve(null);
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
        askValue(sessionRef.current!, expression, "hover", selectedFrame ?? undefined);
        // A hover that never answers must not leave the tooltip waiting
        // forever on a program that has moved on.
        setTimeout(() => {
          if (pendingValues.current.get(expression)===resolve){pendingValues.current.delete(expression);resolve(null);}
        }, 2000);
      });
    },
    [client, status, selectedFrame, variables],
  );

  const query = useCallback(
    (expression: string): Promise<string | null> => {
      if (!client || !sessionRef.current || statusRef.current !== "paused"||threadModel.selecting||selectedFrame===null||!debugAdapterId(selectedFrame)) {
        return Promise.resolve(null);
      }
      return new Promise((resolve) => {
        pendingValues.current.set(expression, resolve);
        // `repl` context: the person typed this, so a side effect is their
        // business — unlike a hover, which must never have one.
        askValue(sessionRef.current!, expression, "repl", selectedFrame ?? undefined);
        setTimeout(() => {
          if (pendingValues.current.get(expression)===resolve){pendingValues.current.delete(expression);resolve(null);}
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
      if (client && sessionRef.current && statusRef.current === "paused"&&selectedFrame!==null) {
        askValue(sessionRef.current, trimmed, "watch", selectedFrame ?? undefined);
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
    if (status !== "paused" || !client || !sessionRef.current||selectedFrame===null) return;
    for (const expression of watchKey ? watchKey.split("\n") : []) {
      askValue(sessionRef.current, expression, "watch", selectedFrame ?? undefined);
    }
  }, [status, selectedFrame, watchKey, client,threadModel.scope?.stop,threadModel.scope?.thread]);

  const readThreads=useCallback(()=>{if(!client||statusRef.current!=="paused"||!sessionRef.current)return;const request=threadModel.read();refreshThreads();if(request&&request.session===sessionRef.current)client.threads(request.session,request.request);},[client,threadModel]);
  const selectThread=useCallback((id:number)=>{if(!client||statusRef.current!=="paused"||!sessionRef.current)return;const request=threadModel.select(id);refreshThreads();if(request&&request.session===sessionRef.current)client.selectThread(request.session,id,request.stop,request.request);},[client,threadModel]);

  const captureFrame=useCallback((id:number)=>{const frame=currentFrames.current.find(item=>item.id===id),session=sessionRef.current,scope=threadModel.scope,lifetime=ownerLifetime.current;
    if(!frame||!session||statusRef.current!=="paused"||selectedFrameRef.current!==id||threadModel.selecting)return null;
    return {frame:Object.freeze({...frame}),current:()=>{const latest=currentFrames.current.find(item=>item.id===id);return ownerMounted.current&&ownerLifetime.current===lifetime&&sessionRef.current===session&&statusRef.current==="paused"&&!!latest&&["name","source","source_line","line","in_document","source_hash","source_reference"].every(key=>latest[key as keyof Frame]===frame[key as keyof Frame])&&selectedFrameRef.current===id&&!threadModel.selecting&&(!scope?threadModel.scope===null:threadModel.scope?.stop===scope.stop&&threadModel.scope.thread===scope.thread);}};
  },[threadModel]);

  const selectFrame = useCallback(
    (id: number) => {
      const frame = frames.find((candidate) => candidate.id === id);
      if(currentFrames.current!==frames||!frame||!debugAdapterId(id)||statusRef.current!=="paused"||threadModel.selecting)return;
      retireFrameRead();retireEvaluations();setVariables(localsByFrame.current.get(id)??[]);setWatches(current=>current.map(watch=>({...watch,value:null})));selectedFrameRef.current=id;setSelectedFrame(id);
      // Selecting a frame moves the paused marker with it, because the line
      // you are looking at should be the line whose values you are reading.
      if (frame?.line !== undefined) setPausedLine(frame.line);
      if (capabilities?.frame_locals && client && sessionRef.current){const scope=threadModel.scope;if(scope){const request="locals-"+(++frameSequence.current);frameRead.current={session:sessionRef.current,stop:scope.stop,thread:scope.thread,frame:id,request};setFrameVariablesPending(true);client.state(sessionRef.current,id,{stop:scope.stop,thread:scope.thread,request_id:request});}else client.state(sessionRef.current,id);}
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
      threads:threadModel.reading?.threads??[],selectedThread:threadModel.scope?.thread??null,threadStop:threadModel.scope?.stop??null,threadPending:threadModel.busy,threadSelecting:threadModel.selecting,threadMessage:threadModel.message,frameVariablesPending,frameVariablesMessage,stackMessage,historicalFrames,observationScope,captureFrame,readThreads,selectThread,
      variables,
      selectedFrame,
      breakpoints,
      sourceBreakpoints, setSourceBreakpoints,
      replaceBreakpoints: setBreakpoints,
      exitCode,
      buildOutput,
      output,
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
      threadRevision,readThreads,selectThread,frameVariablesPending,frameVariablesMessage,stackMessage,historicalFrames,observationScope,captureFrame,
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
      output,
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
import { describe, expect, it,vi } from "vitest";
import {withDebugProtocol} from "./launchProfiles";
vi.mock("./launchProfiles",()=>({withDebugProtocol:vi.fn((_query:unknown,effect:()=>void)=>{effect();return Promise.resolve();})}));
import type { Realtime } from "../api/realtime";
import { DebugClient,DebugThreads,DebugLaunches,debugAdapterId } from "./client";
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
    if(event.request_id===undefined&&(event.event==="started"||event.event==="build"||event.event==="failed"&&!event.session&&(event.about===undefined||event.about==="start"))){const start=[...this.sent].reverse().find(item=>(item as {op?:string}).op==="start") as {request_id?:string}|undefined;event={...event,request_id:start?.request_id};}
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
      client.handleFrame(frame({ ...STARTED, doc: "hick:///tools/a.py",request_id:(sent[0] as {request_id:string}).request_id }));
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
          about: "start",request_id:(sent.at(-1) as {request_id:string}).request_id,
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


describe("source-current debugger thread observations",()=>{
 it("admits only the latest exact stopped list and never selects optimistically",()=>{
  const model=new DebugThreads();model.stopped("session",3,1);const first=model.read()!,second=model.read()!;
  const event=(request:string,stop=3,selected=1)=>({event:"threads" as const,session:"session",stop,selected,request_id:request,threads:[{id:1,name:"main"},{id:2,name:"worker"}]});
  expect(model.accept(event(first.request))).toBe(false);expect(model.accept(event(second.request,2))).toBe(false);expect(model.accept({...event(second.request),session:"foreign"})).toBe(false);expect(model.accept(event(second.request))).toBe(true);
  const selected=model.select(2)!;expect(selected.stop).toBe(3);expect(model.scope?.thread).toBe(1);expect(model.select(1)).toBeNull();model.stopped("session",4,2);expect(model.reading).toBeNull();expect(model.accept(event(selected.request,4,2))).toBe(true);expect(model.scope?.thread).toBe(2);
  model.clear();expect(model.accept(event(selected.request,4,2))).toBe(false);expect(model.select(2)).toBeNull();
 });
 it("rejects malformed identities and retires unrelated stop generations while keeping optional failures local",()=>{
  const model=new DebugThreads();model.stopped("session",1,1);const read=model.read()!,event={event:"threads" as const,session:"session",stop:1,selected:1,request_id:read.request,threads:[{id:1,name:"main"}]};expect(model.accept(event)).toBe(true);
  const refresh=model.read()!;expect(model.failed("provider unavailable","old request")).toBe(false);expect(model.busy).toBe(true);expect(model.failed("provider unavailable",refresh.request)).toBe(true);expect(model.reading?.threads).toEqual([{id:1,name:"main"}]);
  const next=model.read()!;expect(model.accept({...event,request_id:next.request,threads:[{id:1,name:"a"},{id:1,name:"b"}]})).toBe(false);expect(model.reading?.threads).toEqual([{id:1,name:"main"}]);model.stopped("session",2,1);expect(model.accept({...event,request_id:next.request})).toBe(false);
  expect(model.stopped("session",Number.MAX_SAFE_INTEGER+1,1)).toBe(false);expect(model.scope).toEqual({session:"session",stop:2,thread:1});
 });
 it("routes wire thread reads and selection through the hook without failing the paused stack",async()=>{
  const {socket,hook}=open(),result=hook.result;act(()=>result.current.start());act(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:7,thread:1});});await waitFor(()=>expect(result.current.status).toBe("paused"));
  const frames=result.current.frames;act(()=>result.current.readThreads?.());const request=socket.sent.at(-1) as {op:string;request_id:string};expect(request.op).toBe("threads");
  act(()=>socket.deliver({event:"failed",session:"dbg-0",about:"threads",request_id:request.request_id,message:"adapter read unavailable"}));expect(result.current.status).toBe("paused");expect(result.current.frames).toBe(frames);expect(result.current.threadMessage).toContain("unavailable");
  act(()=>result.current.readThreads?.());const latest=socket.sent.at(-1) as {request_id:string};act(()=>socket.deliver({event:"threads",session:"dbg-0",stop:7,selected:1,request_id:latest.request_id,threads:[{id:1,name:"main"},{id:2,name:"worker"}]}));await waitFor(()=>expect(result.current.threads).toHaveLength(2));
  act(()=>result.current.selectThread?.(2));const selection=socket.sent.at(-1) as {op:string;stop:number;request_id:string};expect(selection.op).toBe("select_thread");expect(selection.stop).toBe(7);expect(result.current.selectedThread).toBe(1);const sent=socket.sent.length;act(()=>{result.current.step("over");result.current.readThreads?.();result.current.selectFrame(2);});expect(socket.sent).toHaveLength(sent);expect(await result.current.query("quantity")).toBeNull();
  act(()=>socket.deliver({...STOPPED,stop:8,thread:2}));act(()=>socket.deliver({event:"threads",session:"dbg-0",stop:8,selected:2,request_id:selection.request_id,threads:[{id:1,name:"main"},{id:2,name:"worker"}]}));await waitFor(()=>expect(result.current.selectedThread).toBe(2));expect(result.current.status).toBe("paused");
 });
 it("correlates values to the requested stop, thread, frame and request before settling current expressions",async()=>{
  const {socket,hook}=open(),result=hook.result;act(()=>result.current.start());act(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:10,thread:1});});
  let value!:Promise<string|null>;act(()=>{value=result.current.query("quantity");});const first=socket.sent.at(-1) as {request_id:string;stop:number;thread:number};expect(first.stop).toBe(10);expect(first.thread).toBe(1);
  const reply={event:"value",session:"dbg-0",expression:"quantity",value:"2",type_name:"int",type:"legacy",reference:0,frame:2,thread:1,stop:10,request_id:first.request_id};
  act(()=>socket.deliver({...reply,stop:9}));expect(result.current.lastValue).toBeNull();act(()=>socket.deliver({...reply,frame:99}));expect(result.current.lastValue).toBeNull();await act(async()=>{socket.deliver(reply);expect(await value).toBe("2");});expect(result.current.lastValue?.type).toBe("int");
  let stale!:Promise<string|null>;act(()=>{stale=result.current.query("quantity");});const old=socket.sent.at(-1) as {request_id:string};await act(async()=>{socket.deliver({...STOPPED,stop:11,thread:2});expect(await stale).toBeNull();});
  act(()=>socket.deliver({...reply,request_id:old.request_id}));expect(result.current.lastValue).toBeNull();
  act(()=>{value=result.current.query("quantity");});const latest=socket.sent.at(-1) as {request_id:string};act(()=>socket.deliver({event:"failed",session:"dbg-0",about:"eval",request_id:old.request_id,message:"old evaluation failed"}));await act(async()=>{socket.deliver({...reply,stop:11,thread:2,request_id:latest.request_id,value:"worker"});expect(await value).toBe("worker");});
 });

 it("does not relabel top-frame locals as another frame and keeps evaluation refusal local",async()=>{
  const {socket,hook}=open(),result=hook.result;act(()=>result.current.start());const frames=[...STOPPED.frames,{id:3,name:"worker",line:8,source:"orders.py",in_document:true}];act(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:30,thread:2,frames});});
  act(()=>result.current.selectFrame(3));expect(result.current.selectedFrame).toBe(3);expect(result.current.variables).toEqual([]);let value!:Promise<string|null>;act(()=>{value=result.current.query("worker_value");});const query=socket.sent.at(-1) as {request_id:string;frame:number};expect(query.frame).toBe(3);
  await act(async()=>{socket.deliver({event:"failed",session:"dbg-0",about:"eval",request_id:query.request_id,message:"Unable to find thread for evaluation"});expect(await value).toBeNull();});expect(result.current.status).toBe("paused");expect(result.current.frames).toEqual(frames);expect(result.current.selectedThread).toBe(2);
  act(()=>result.current.selectFrame(2));expect(result.current.variables).toEqual(STOPPED.variables);expect(result.current.status).toBe("paused");
 });

 it("admits only current selected-frame locals and keeps failed optional scopes separate from the stack",async()=>{
  const {socket,hook}=open(),result=hook.result,frames=[...STOPPED.frames,{id:3,name:"worker",line:8,source:"orders.py",in_document:true}];act(()=>result.current.start());act(()=>{socket.deliver({...STARTED,capabilities:{...STARTED.capabilities,frame_locals:true}});socket.deliver({...STOPPED,stop:40,thread:2,frames});});
  act(()=>result.current.selectFrame(3));const request=socket.sent.at(-1) as {op:string;request_id:string;stop:number;thread:number;frame:number};expect(request).toMatchObject({op:"state",stop:40,thread:2,frame:3});expect(result.current.frameVariablesPending).toBe(true);expect(result.current.variables).toEqual([]);
  const response={event:"frame_variables",session:"dbg-0",stop:40,thread:2,frame:3,request_id:request.request_id,variables:[{name:"worker_value",value:"worker",type:"str",variables_reference:0}]};act(()=>socket.deliver({...response,frame:2}));expect(result.current.variables).toEqual([]);act(()=>socket.deliver(response));expect(result.current.variables).toEqual(response.variables);expect(result.current.frameVariablesPending).toBe(false);
  act(()=>result.current.selectFrame(2));const top=socket.sent.at(-1) as {request_id:string};act(()=>socket.deliver({event:"failed",session:"dbg-0",about:"state",request_id:top.request_id,message:"adapter scopes unavailable"}));expect(result.current.status).toBe("paused");expect(result.current.frames).toEqual(frames);expect(result.current.variables).toEqual(STOPPED.variables);expect(result.current.frameVariablesMessage).toContain("unavailable");
  act(()=>result.current.step("over"));act(()=>socket.deliver(response));expect(result.current.status).toBe("running");expect(result.current.frameVariablesPending).toBe(false);
 });

 it("keeps captured frame lifetimes independent of locals arrival and rejects retired stack navigation",()=>{
  const {socket,hook}=open(),result=hook.result;act(()=>result.current.start());act(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:50,thread:1});});const captured=result.current.captureFrame?.(2)!;expect(captured.current()).toBe(true);
  act(()=>socket.deliver({event:"run_output",session:"dbg-0",events:[{category:"stdout",text:"independent output"}]}));expect(captured.current()).toBe(true);
  act(()=>socket.deliver({...STOPPED,stop:51,thread:1,frames:[],variables:[],stack_error:"stack provider unavailable"}));expect(captured.current()).toBe(false);expect(result.current.frames).toEqual([]);expect(result.current.historicalFrames).toEqual(STOPPED.frames);expect(result.current.stackMessage).toContain("unavailable");const sent=socket.sent.length;act(()=>result.current.selectFrame(2));expect(socket.sent).toHaveLength(sent);expect(result.current.captureFrame?.(2)).toBeNull();expect(result.current.status).toBe("paused");
 });

 it("does not regress the entire pause from held older stops and preserves selected caller on same-stop refresh",()=>{
  const {socket,hook}=open(),result=hook.result,frames=[...STOPPED.frames,{id:3,name:"caller",line:7,source:"orders.py",in_document:true}];act(()=>result.current.start());act(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:60,thread:1,frames});});act(()=>result.current.selectFrame(3));const captured=result.current.captureFrame?.(3)!;
  act(()=>socket.deliver({...STOPPED,stop:60,thread:1,frames:frames.map(frame=>({...frame}))}));expect(result.current.selectedFrame).toBe(3);expect(result.current.pausedLine).toBe(7);expect(result.current.variables).toEqual([]);expect(captured.current()).toBe(true);
  const accepted=result.current.frames;act(()=>socket.deliver({...STOPPED,stop:59,thread:2,frames:[],variables:[],line:null}));expect(result.current.frames).toBe(accepted);expect(result.current.selectedFrame).toBe(3);expect(result.current.selectedThread).toBe(1);
  act(()=>result.current.step("over"));act(()=>socket.deliver({...STOPPED,stop:60,thread:1,frames}));expect(result.current.status).toBe("running");expect(captured.current()).toBe(false);
 });
 it("binds launch receipts before sending and cleans only known retired same-file launches",()=>{
  const {socket,hook}=open(),result=hook.result;act(()=>result.current.start());const first=socket.sent.at(-1) as {request_id:string};expect(first.request_id).toBeTruthy();act(()=>result.current.stop());act(()=>result.current.start());const second=socket.sent.at(-1) as {request_id:string};expect(second.request_id).not.toBe(first.request_id);
  const before=socket.sent.length;act(()=>socket.deliver({...STARTED,session:"foreign",request_id:"another caller"}));expect(socket.sent).toHaveLength(before);expect(result.current.status).toBe("starting");
  act(()=>socket.deliver({...STARTED,session:"old",request_id:first.request_id}));expect(socket.sent.at(-1)).toEqual({op:"stop",session:"old"});expect(result.current.status).toBe("starting");
  act(()=>socket.deliver({event:"build",doc:"hick:///orders.md",request_id:first.request_id,events:[{type:"stdout",text:"old build"}]}));expect(result.current.buildOutput).toEqual([]);
  act(()=>socket.deliver({...STARTED,session:"new",request_id:second.request_id}));act(()=>socket.deliver({...STOPPED,session:"new",stop:1,thread:1}));const frames=result.current.frames;act(()=>socket.deliver({...STARTED,session:"new",request_id:second.request_id}));expect(result.current.status).toBe("paused");expect(result.current.frames).toBe(frames);
  act(()=>socket.deliver({event:"failed",session:null,about:"start",request_id:first.request_id,message:"old failed start"}));expect(result.current.status).toBe("paused");expect(result.current.message).toBeNull();
  const launches=new DebugLaunches(),old=launches.begin();for(let index=0;index<70;index++)launches.begin();expect(launches.started(old,"old")).toBe("foreign");
 });

 it("retains DAP zero and negative signed-int32 thread/frame identities without sentinel guessing",async()=>{
  const model=new DebugThreads();expect(model.stopped("scope",1,0)).toBe(true);const read=model.read()!;expect(model.accept({event:"threads",session:"scope",stop:1,selected:0,request_id:read.request,threads:[{id:0,name:"main"},{id:-7,name:"worker"}]})).toBe(true);expect(model.select(-7)?.thread).toBe(-7);expect(model.stopped("scope",2,-7)).toBe(true);expect(model.scope?.thread).toBe(-7);expect(debugAdapterId(true)).toBe(false);expect(debugAdapterId(0.5)).toBe(false);expect(debugAdapterId(2147483648)).toBe(false);
  const {socket,hook}=open(),result=hook.result,frames=[{...STOPPED.frames[0],id:0},{...STOPPED.frames[0],id:-3,name:"caller"}];act(()=>result.current.start());act(()=>{socket.deliver({...STARTED,capabilities:{...STARTED.capabilities,frame_locals:true}});socket.deliver({...STOPPED,stop:70,thread:0,frames});});expect(result.current.selectedFrame).toBe(0);expect(result.current.selectedThread).toBe(0);
  let value!:Promise<string|null>;act(()=>{value=result.current.query("quantity");});const expression=socket.sent.at(-1) as {request_id:string};expect(socket.sent.at(-1)).toMatchObject({thread:0,frame:0,stop:70});await act(async()=>{socket.deliver({event:"value",session:"dbg-0",expression:"quantity",value:"zero frame",type:null,reference:0,stop:70,thread:0,frame:0,request_id:expression.request_id});expect(await value).toBe("zero frame");});
  act(()=>result.current.selectFrame(-3));expect(socket.sent.at(-1)).toMatchObject({op:"state",frame:-3,thread:0});expect(result.current.captureFrame?.(-3)?.current()).toBe(true);
 });

 it("binds opaque observation scopes after Reading-first delivery and never downgrades to held old evidence",()=>{
  const {socket,hook}=open(),result=hook.result;act(()=>result.current.start());act(()=>socket.deliver(STARTED));const reading=(stop:string)=>({schema_version:1,source:{id:"orders",uri:"hick:///orders.md",revision:"captured",bytes:[97]},scope:{session:"dbg-0",run:"run",stop,frame:2,invocation:null},locations:[],annotations:[],unsupported:[]});
  act(()=>{socket.deliver({event:"reading",session:"dbg-0",reading:reading("opaque-stop-a")});socket.deliver({...STOPPED,stop:80,thread:0,observation_scope:reading("opaque-stop-a").scope});});expect(result.current.reading?.scope.stop).toBe("opaque-stop-a");
  act(()=>socket.deliver({event:"reading",session:"dbg-0",reading:reading("opaque-stop-b")}));expect(result.current.reading?.scope.stop).toBe("opaque-stop-a");act(()=>socket.deliver({...STOPPED,stop:81,thread:0,observation_scope:reading("opaque-stop-b").scope}));expect(result.current.reading?.scope.stop).toBe("opaque-stop-b");
  act(()=>{socket.deliver({event:"reading",session:"dbg-0",reading:reading("opaque-stop-a")});socket.deliver({...STOPPED,stop:80,thread:0,observation_scope:reading("opaque-stop-a").scope});});expect(result.current.reading?.scope.stop).toBe("opaque-stop-b");expect(result.current.observationScope?.stop).toBe("opaque-stop-b");
 });

});

it("preflights direct startup and retires held negotiation before sending",async()=>{
 vi.mocked(withDebugProtocol).mockRejectedValueOnce(new Error("This engine needs protocol 2"));const {socket,hook}=open();await act(async()=>hook.result.current.start());expect(socket.sent).toEqual([]);expect(hook.result.current.message).toContain("protocol 2");
 let ready!:()=>void;vi.mocked(withDebugProtocol).mockImplementationOnce((_query,effect)=>new Promise<void>(resolve=>{ready=()=>{effect();resolve();};}));act(()=>hook.result.current.start());act(()=>hook.result.current.stop());await act(async()=>ready());expect(socket.sent.some(item=>(item as {op:string}).op==="start")).toBe(false);expect(hook.result.current.status).toBe("idle");
});
</storycode:copy>

### Assemble `apps/web/src/debug/useDebugger.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/useDebugger.test.ts"><storycode:paste select="#f-101aa3d58173-FakeSocket"/><storycode:paste select="#f-101aa3d58173-STARTED"/><storycode:paste select="#f-101aa3d58173-STOPPED"/><storycode:paste select="#f-101aa3d58173-open"/><storycode:paste select="#f-101aa3d58173-expression_statement"/><storycode:paste select="#f-101aa3d58173-expression_statement-2"/><storycode:paste select="#f-101aa3d58173-expression_statement-3"/><storycode:paste select="#f-101aa3d58173-expression_statement-4"/><storycode:paste select="#f-101aa3d58173-expression_statement-5"/></storycode:file>


</storycode:doc>
