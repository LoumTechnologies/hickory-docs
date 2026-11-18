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
import {AdapterSourceReading} from "./AdapterSourceReading";
import {documentUri} from "../lsp/documentUri";
import {withDebugProtocol,launchCatalogNotices} from "./launchProfiles";
import {prepareSourceTestDebug,sourceTestDebugStarted,sourceTestDebugBinding,retainedSourceTestDebugChoice,type PreparedSourceTestDebug,type SourceTestDebugProcess} from "../simple/TestSources";
import {DebugBreakpointModel,savedDebugSources,stampDebugSource,type DebugSourceCapability,type DebugSourceReading,type DebugSourceProof,type CapturedDebugIntent} from "./SourceGuards";
import {DebugValueTree,type DebugValueScope,type DebugValueNode} from "./DebugValueTree";
import type {
  BreakpointStatus,
  DebugBreakpoint,
  DebugCapabilities,
  DebugEvent,
  Frame,
  Step,
  Variable,
} from "./client";

export type DebugStatus = "idle" | "starting" | "paused" | "running" | "unconfirmed" | "stopping" | "finished" | "failed";

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
  adapterSource?:AdapterSourceReading;
  readAdapterSource?(id?:number):void;
  /** Synchronous private lifecycle admission, before a React publication commits. */
  isActive?():boolean;
  startSourceTest?(prepared:PreparedSourceTestDebug,current:()=>boolean):Promise<SourceTestDebugProcess>;
  cancelSourceTestStart?(prepared:PreparedSourceTestDebug):void;
  sourceTestProcesses?():readonly SourceTestDebugProcess[];
  sourceTestInspectable?(session:string):boolean;
  sourceTest?:SourceTestDebugProcess|null;
  sourceTestIntent?:PreparedSourceTestDebug|null;
  releaseSourceTest?():void;
  breakpointRows?():HeldBreakpoint[];
  capturedBreakpointIntents?():Record<string,CapturedDebugIntent>;
  launchNotices?:string[];
  breakpointNotice?:string|null;
  breakpointIntentions?:{source:string;marks:import("./client").DebugBreakpoint[];namespace:string|null;hash?:string|null;historical?:boolean}[];
  variableRoots?:DebugValueNode[];
  valueNode?:(id:string)=>DebugValueNode|null;
  valueHistory?:string[];
  valueNotice?:string|null;
  expandValue?:(id:string)=>void;
  collapseValue?:(id:string)=>void;
  closeValues?:()=>void;
  refreshState?:()=>void;
  positionSourceBound?:boolean;
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
  start(program?: string, sources?: Record<string, HeldBreakpoint[]>, project?: string, profile?: import("./launchProfiles").LaunchSelection,origins?:Record<string,CapturedDebugIntent>): void;
  replaceBreakpoints(breakpoints: HeldBreakpoint[],provenance?:string): void;
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
  requested_line?:number;
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
// SourceGuards owns complete receipt admission and only then projects provider moves.
</storycode:copy>
<a id="f-597e991b795b-onTheWire"></a>
### `onTheWire`

Keep requested source lines independent of the adapter’s bound display positions. Conditions stay attached to their original intent.

<storycode:copy id="f-597e991b795b-onTheWire">
/**
 * The wire form: only what was actually set, so an adapter is never handed
 * an empty condition string to parse.
 */
function onTheWire(breakpoints: HeldBreakpoint[]): DebugBreakpoint[] {
  return breakpoints.map((breakpoint) => ({
    line: breakpoint.requested_line??breakpoint.line,
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
  if (event.event === "test_preparing") return event.doc === uri;
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
export function useDebugger(realtime: Realtime, docPath: string, projectContext = "new",sourceCapability?:DebugSourceCapability): DebugSession {
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

  return useDebuggerOver(client, docPath, [], projectContext,undefined,undefined,sourceCapability);
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
export function useWorkspaceDebugger(path: string,uiSource?:unknown,launchCurrent?:()=>boolean,sourceCapability?:DebugSourceCapability): DebugSession {
  const client = useMemo(() => workspaceDebugClient(), []);
  return useDebuggerOver(client, path,[],"new",uiSource,launchCurrent,sourceCapability);
}

</storycode:copy>
<a id="f-597e991b795b-useDebuggerOver"></a>
### `useDebuggerOver`

Thread reads share the document session controller but fail independently of paused frames and locals. Source/frame changes invalidate held callbacks. Retire thread scope immediately when execution resumes; a late list must not attach to a later pause. Confirmed thread changes clear old evaluation values and re-evaluate watches for the new stop. Evidence uses an opaque scope tuple rather than the numeric debugger pause generation: hold early readings by their complete scope and promote only the exact Stopped scope. Startup preflights the read-only protocol catalog even for direct action-search requests; a caller-owned launch ticket and captured owner/project/profile retire delayed replies before they can send Start.

<storycode:copy id="f-597e991b795b-useDebuggerOver">
export function useDebuggerOver(client: DebugClient | null, docPath: string, initialBreakpoints: readonly number[] = [], projectContext = "new", uiSource?: unknown,launchCurrent?:()=>boolean,sourceCapability?:DebugSourceCapability): DebugSession {
  sourceCapability=sourceCapability??client?.provider?.sources??savedDebugSources;
  const uri = useMemo(() => documentUri(docPath), [docPath]);
  const sourceAccess=useRef(sourceCapability);sourceAccess.current=sourceCapability;
  const launchAdmission=useRef(launchCurrent);launchAdmission.current=launchCurrent;
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
  const [capabilities, setCapabilities] = useState<DebugCapabilities | null>(null);const capabilitiesRef=useRef<DebugCapabilities|null>(null);
  const [pausedLine, setPausedLine] = useState<number | null>(null);
  const [frames, setFrames] = useState<Frame[]>([]),[historicalFrames,setHistoricalFrames]=useState<Frame[]>([]);
  const currentFrames=useRef<Frame[]>(frames);
  const [launches]=useState(()=>new DebugLaunches());
  const [threadModel]=useState(()=>new DebugThreads()),[threadRevision,paintThreads]=useState(0);
  const refreshThreads=()=>paintThreads(value=>value+1);
  const [variables, setVariables] = useState<Variable[]>([]);const variablesRef=useRef<Variable[]>([]);
  const holdVariables=(next:Variable[])=>{variablesRef.current=next;setVariables(next);};
  const localsByFrame=useRef(new Map<number,Variable[]>());
  const [frameVariablesPending,setFrameVariablesPending]=useState(false),[frameVariablesMessage,setFrameVariablesMessage]=useState<string|null>(null),[stackMessage,setStackMessage]=useState<string|null>(null);
  const ownerLifetime=useRef(0),ownerMounted=useRef(true);
  const frameRead=useRef<{session:string;stop:number;thread:number;frame:number;request:string}|null>(null),frameSequence=useRef(0);
  const [selectedFrame, setSelectedFrame] = useState<number | null>(null);
  const selectedFrameRef=useRef<number|null>(null);
  const evaluationSequence=useRef(0),evaluationRequests=useRef(new Map<string,{expression:string;context:"hover"|"watch"|"repl";stop:number;thread:number;frame:number|null}>()),latestEvaluation=useRef(new Map<string,string>());
  const [breakpointModel]=useState(()=>{const model=new DebugBreakpointModel();model.unknown(uri,initialBreakpoints.map(line=>({line})));return model;}),[,paintPolicy]=useState(0);
  type TestPending={prepared:PreparedSourceTestDebug;current:()=>boolean;request:string|null;sent:boolean;preparing:boolean;resolve:(record:SourceTestDebugProcess)=>void;reject:(error:Error)=>void};
  const testIntent=useRef<PreparedSourceTestDebug|null>(null),pendingTest=useRef<TestPending|null>(null),testHistory=useRef(new Map<string,SourceTestDebugProcess>()),testContext=useRef<string|null>(null),lastOwnedSession=useRef<string|null>(null),canceledTestRequests=useRef(new Set<string>()),testCleanup=useRef(new Set<string>());
  const [testEpoch,paintTest]=useState(0);
  function publishTest(session:string,phase:string,exitCode:number|null=null,message:string|null=null){const before=testHistory.current.get(session);if(before){testHistory.current.set(session,{...before,phase,exitCode,message});paintTest(value=>value+1);}}
  function rejectTest(error:Error){const held=pendingTest.current;if(held){pendingTest.current=null;held.reject(error);}}
  const sourceReadings=useRef(new Map<string,DebugSourceReading>()),sourceReadRequests=useRef(new Map<string,number>()),runProofs=useRef(new Map<string,DebugSourceProof>()),runHashes=useRef(new Map<string,string>()),sourceOperations=useRef(new Map<string,number>()),preparedStart=useRef<{request:string;proofs:Map<string,DebugSourceProof>}|null>(null);
  const [valueTree]=useState(()=>new DebugValueTree()),[treeRevision,paintTree]=useState(0),refreshTree=()=>paintTree(value=>value+1);
  const movement=useRef<{request:string;session:string;scope:{session:string;stop:number;thread:number};frames:Frame[];variables:Variable[];selected:number|null;observation:import("./observations").ObservationScope|null;roots:DebugValueNode[]}|null>(null),refreshTicket=useRef<string|null>(null);
  const [launchNotices,setLaunchNotices]=useState<string[]>([]);
  const [breakpoints,setBreakpoints]=useState<HeldBreakpoint[]>([]);
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

  const [adapterSourceVersion,paintAdapterSource]=useState(0);
  const adapterSource=useMemo(()=>new AdapterSourceReading(()=>paintAdapterSource(value=>value+1)),[]);
  function holdStatus(next:DebugStatus){statusRef.current=next;setStatus(next);if(sessionRef.current&&testHistory.current.has(sessionRef.current))publishTest(sessionRef.current,next);}
  function observationKey(scope:import("./observations").ObservationScope){return JSON.stringify([scope.session??null,scope.run,scope.stop,scope.frame,scope.invocation]);}
  function retireObservationScope(){scopeRef.current=null;setObservationScope(null);pendingReadings.current.clear();}
  function retireFrameRead(){adapterSource.retire();frameRead.current=null;setFrameVariablesPending(false);setFrameVariablesMessage(null);}
  function retireEvaluations(){evaluationRequests.current.clear();latestEvaluation.current.clear();for(const resolve of pendingValues.current.values())resolve(null);pendingValues.current.clear();setLastValue(null);}
  const currentRows=(source=uri)=>breakpointModel.rows(source,sourceReadings.current.get(source)??null,sessionRef.current,runHashes.current.get(source)??null) as HeldBreakpoint[];
  function repaintBreakpoints(){setBreakpoints(currentRows());holdSources(Object.fromEntries(breakpointModel.sources().filter(source=>source!==uri).map(source=>[source,currentRows(source)])));paintPolicy(value=>value+1);}
  function readSource(source:string):DebugSourceReading|Promise<DebugSourceReading>{const request=(sourceReadRequests.current.get(source)??0)+1,lifetime=ownerLifetime.current;sourceReadRequests.current.set(source,request);const result=sourceAccess.current.read(source);const admit=(reading:DebugSourceReading)=>{if(reading.uri!==source||!reading.current())throw Error('The current source reading changed before Debug. Observe it again.');if(ownerMounted.current&&ownerLifetime.current===lifetime&&sourceReadRequests.current.get(source)===request)sourceReadings.current.set(source,reading);return reading;};return result instanceof Promise?result.then(admit):admit(result);}
  function capturedIntents():Record<string,CapturedDebugIntent>{return Object.fromEntries(breakpointModel.sources().flatMap(source=>{const intent=breakpointModel.captured(source);return intent?[[source,intent]]:[];}));}
  function launchBound():boolean {const proof=runProofs.current.get(uri);return !!proof&&proof.hash===runHashes.current.get(uri);}
  function primaryBound():boolean {const proof=runProofs.current.get(uri);return !!proof&&proof.current()&&proof.hash===runHashes.current.get(uri)&&launchAdmission.current?.()!==false;}
  async function sendIntent(source:string,reading:DebugSourceReading,operation:number){
    const session=sessionRef.current;if(!client||!session||!['paused','running','unconfirmed'].includes(statusRef.current)||!reading.saved)return;
    const proof=await stampDebugSource(reading);if(!ownerMounted.current||sessionRef.current!==session||sourceOperations.current.get(source)!==operation||!proof.current())return;
    const launch=runProofs.current.get(source);if(!launch||launch.workspace!==proof.workspace||launch.namespace!==proof.namespace||launch.identity!==proof.identity||runHashes.current.get(source)!==proof.hash)return;
    const ticket=breakpointModel.request(source,proof,session);if(!ticket)return;repaintBreakpoints();if(source===uri)client.setBreakpoints(session,ticket.marks,proof.hash,ticket.request);else client.setSourceBreakpoints(session,source,ticket.marks,proof.hash,ticket.request);
  }
  function editIntent(source:string,transform:(marks:import('./client').DebugBreakpoint[])=>import('./client').DebugBreakpoint[]){
    const operation=(sourceOperations.current.get(source)??0)+1;sourceOperations.current.set(source,operation);
    const apply=(reading:DebugSourceReading)=>{if(sourceOperations.current.get(source)!==operation||!ownerMounted.current)return;breakpointModel.change(source,reading,transform);repaintBreakpoints();void sendIntent(source,reading,operation).catch(error=>{if(sourceOperations.current.get(source)===operation)setMessage(error instanceof Error?error.message:String(error));});};
    try{const reading=readSource(source);if(reading instanceof Promise)void reading.then(apply).catch(error=>{if(sourceOperations.current.get(source)===operation)setMessage(error instanceof Error?error.message:String(error));});else apply(reading);}catch(error){setMessage(error instanceof Error?error.message:String(error));}
  }
  const valueScope=():DebugValueScope|null=>{const scope=threadModel.scope,frame=selectedFrameRef.current;return sessionRef.current&&statusRef.current==='paused'&&!threadModel.selecting&&scope&&debugAdapterId(frame)&&currentFrames.current.some(value=>value.id===frame)?{...scope,frame}:null;};
  function replaceTreeLocals(values:Variable[]){const scope=valueScope();if(!scope)return;valueTree.setScope(scope);valueTree.replace('locals',scope,values);refreshTree();}
  useEffect(()=>sourceCapability.subscribe?.(()=>{for(const source of breakpointModel.sources()){try{const result=readSource(source);if(result instanceof Promise)void result.then(()=>{if(ownerMounted.current)repaintBreakpoints();}).catch(()=>{});}catch{/* Passive capability failure cannot discard readable wishes or block source. */}}repaintBreakpoints();}),[sourceCapability.subscribe,uri]);
  function askValue(session:string,expression:string,context:"hover"|"watch"|"repl",frame?:number){
    if(!client||threadModel.selecting)return;if(frame!==undefined&&!debugAdapterId(frame)){setMessage("Selected frame identity cannot be represented exactly by this connection.");return;}const scope=threadModel.scope;
    if(!scope||statusRef.current!=="paused"||frame===undefined||!currentFrames.current.some(value=>value.id===frame))return;
    valueTree.remove(`expression:${expression}`);refreshTree();
    const request_id="eval-"+(++evaluationSequence.current),previous=latestEvaluation.current.get(expression);if(previous)evaluationRequests.current.delete(previous);
    if(evaluationRequests.current.size>=512){const oldest=evaluationRequests.current.keys().next().value;if(oldest)evaluationRequests.current.delete(oldest);}
    latestEvaluation.current.set(expression,request_id);evaluationRequests.current.set(request_id,{expression,context,stop:scope.stop,thread:scope.thread,frame:frame??null});client.evaluate(session,expression,context,frame,{stop:scope.stop,thread:scope.thread,request_id});
  }


  useEffect(() => { retainDebugReading(readingOwner.current, reading); return () => retainDebugReading(readingOwner.current, null); }, [reading]);

  useEffect(() => {
    if (!client) return;
    const off = client.on((event: DebugEvent) => {
      // A shared channel carries every pane's sessions; only ours is ours.
      // Before `started` there is no session to match, so a start that is
      // still in flight is recognised by the file it named.
      if('session' in event&&typeof event.session==='string'&&testCleanup.current.has(event.session)){
        if(event.event==='ended'||event.event==='finished'){testCleanup.current.delete(event.session);if(!testCleanup.current.size&&!canceledTestRequests.current.size&&!sessionRef.current&&!launches.current){holdStatus('idle');setMessage('Canceled test-debug session stopped. No collected test verdict was recorded.');}}return;
      }
      if(event.event==='failed'&&event.request_id&&canceledTestRequests.current.delete(event.request_id)){if(!canceledTestRequests.current.size&&!testCleanup.current.size&&!sessionRef.current&&!launches.current){holdStatus('idle');setMessage('Canceled test-debug Start was refused; no session was adopted.');}return;}
      if (!eventIsOurs(event, uri, sessionRef.current)) return;
      if(event.event==="test_preparing"){
        const test=pendingTest.current;
        if(!test?.sent||test.preparing||!launches.matches(event.request_id)||test.request!==event.request_id||
          'session' in event||!Number.isInteger(event.wait_ms)||event.wait_ms<=30_000||event.wait_ms>180_000||
          !test.current()||!sourceTestDebugStarted(test.prepared,event.source_test))return;
        test.preparing=true;
        setMessage('Preparing the saved test. Waiting for the debugger session…');
        return;
      }
      if(event.event==="started"){
        if(!event.request_id&&launches.current&&statusRef.current==="starting"){setMessage("A debugger startup reply has no caller identity; this view has not adopted it and the request remains unconfirmed.");return;}
        const admission=launches.started(event.request_id,event.session);if(admission==="retired"){if(event.session!==sessionRef.current){if(event.request_id&&canceledTestRequests.current.delete(event.request_id))testCleanup.current.add(event.session);client.stop(event.session);}return;}if(admission!=="current")return;
        const test=pendingTest.current;if(test&&test.request===event.request_id&&(!test.current()||event.source_hash!==test.prepared.choice.inputHash||!sourceTestDebugStarted(test.prepared,event.source_test))){launches.retire();testCleanup.current.add(event.session);client.stop(event.session);rejectTest(Error('The engine did not confirm this exact test-debug selection. Its positively owned session was stopped without adoption.'));holdStatus('unconfirmed');setMessage('Test-debug receipt mismatch; waiting for the owned session to stop.');return;}
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
          sessionRef.current = event.session;lastOwnedSession.current=event.session;
          const test=pendingTest.current;if(test&&test.request===event.request_id){const record:SourceTestDebugProcess={session:event.session,choice:retainedSourceTestDebugChoice(test.prepared.choice),binding:sourceTestDebugBinding(test.prepared),phase:'running',exitCode:null,message:null};testHistory.current.set(event.session,record);while(testHistory.current.size>32)testHistory.current.delete(testHistory.current.keys().next().value!);testContext.current=event.session;pendingTest.current=null;setMessage(null);test.resolve(record);paintTest(value=>value+1);}
          setOutput([]);
          capabilitiesRef.current=event.capabilities;setCapabilities(event.capabilities);
          const prepared=preparedStart.current;runProofs.current=prepared&&prepared.request===event.request_id?new Map(prepared.proofs):new Map();runHashes.current.clear();
          if(typeof event.source_hash==='string')runHashes.current.set(uri,event.source_hash);
          breakpointModel.bindLaunch(event.session,event.request_id!);breakpointModel.admit(uri,event.session,event.request_id,event.source_hash,event.breakpoints);repaintBreakpoints();
          valueTree.retire();refreshTree();movement.current=null;refreshTicket.current=null;
          threadModel.reset();retireObservationScope();currentFrames.current=[];setFrames([]);setHistoricalFrames([]);holdVariables([]);selectedFrameRef.current=null;setSelectedFrame(null);setPausedLine(null);retireFrameRead();retireEvaluations();refreshThreads();holdStatus("running");
          clearFailure();if(!primaryBound())setMessage("The engine did not confirm the captured primary source hash. Session controls remain available, but source positions and breakpoint effects are unbound.");
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
        case "threads": {
          const selecting=threadModel.selecting;if(threadModel.accept(event)&&selecting)replaceTreeLocals(variablesRef.current);refreshThreads();break;
        }
        case "stopped": {
          const frameIds=new Set<number>();
          if(!Array.isArray(event.frames)||!Array.isArray(event.variables)||event.frames.length>4096||event.variables.length>16384||event.frames.some(frame=>!frame||!debugAdapterId(frame.id)||frameIds.has(frame.id)||(frameIds.add(frame.id),false)||typeof frame.name!=='string')||event.variables.some(value=>!value||typeof value.name!=='string'||typeof value.value!=='string'||!Number.isSafeInteger(value.variables_reference)||value.variables_reference<0)){
            setMessage('Malformed stopped observation refused; prior readable state retained.');break;
          }
          if(!Number.isSafeInteger(event.stop)||(event.stop as number)<0||!debugAdapterId(event.thread)){setMessage("The engine stop has no exact pause/thread identity; no positions or evaluation scope were adopted.");break;}
          const stopped={...event,frames:event.stack_error?[]:event.frames,variables:event.stack_error||event.variables_error?[]:event.variables};
          const previousThreadScope=threadModel.scope;if(!threadModel.stopped(stopped.session,stopped.stop,stopped.thread))break;refreshThreads();
          const samePause=!!previousThreadScope&&previousThreadScope===threadModel.scope,reportedFrame=stopped.selected_frame??stopped.frames[0]?.id??null;
          const chosenFrame=samePause&&stopped.frames.some(frame=>frame.id===selectedFrameRef.current)?selectedFrameRef.current:reportedFrame;
          if(!samePause||!!frameRead.current&&frameRead.current.frame!==chosenFrame)retireFrameRead();setStackMessage(stopped.stack_error??null);if(!samePause||chosenFrame===reportedFrame)setFrameVariablesMessage(stopped.variables_error??null);
          if(!samePause)localsByFrame.current.clear();if(reportedFrame!==null&&!stopped.variables_error)localsByFrame.current.set(reportedFrame,stopped.variables);
          if(previousThreadScope!==threadModel.scope){retireEvaluations();setWatches(current=>current.map(watch=>({...watch,value:null})));}
          const observed=stopped.observation_scope??(samePause?scopeRef.current:null);scopeRef.current=observed;setObservationScope(observed);
          setReading(current=>observed?(pendingReadings.current.get(observationKey(observed))??(current&&observationKey(current.scope)===observationKey(observed)?current:null)):null);
          holdStatus("paused");
          setPausedLine(chosenFrame===reportedFrame?stopped.line:stopped.frames.find(frame=>frame.id===chosenFrame)?.line??null);
          if(stopped.stack_error){const previous=currentFrames.current;setHistoricalFrames(held=>previous.length?previous:held);}else setHistoricalFrames([]);currentFrames.current=stopped.frames;setFrames(stopped.frames);
          holdVariables(chosenFrame===reportedFrame?stopped.variables:localsByFrame.current.get(chosenFrame!)??[]);
          selectedFrameRef.current=chosenFrame;
          setSelectedFrame(selectedFrameRef.current);
          movement.current=null;refreshTicket.current=null;valueTree.retire();if(!stopped.variables_error)replaceTreeLocals(variablesRef.current);else refreshTree();
          clearFailure();
          break;
        }
        case "breakpoints": {
          if(breakpointModel.admit(uri,event.session,event.request_id,event.source_hash,event.breakpoints)){const proof=breakpointModel.acceptedProof(uri);if(proof)runProofs.current.set(uri,proof);repaintBreakpoints();}break;
        }
        case "source_breakpoints": {
          const source=event.source||uri;if(breakpointModel.admit(source,event.session,event.request_id,event.source_hash,event.breakpoints)){if(typeof event.source_hash==='string')runHashes.current.set(source,event.source_hash);const proof=breakpointModel.acceptedProof(source);if(proof)runProofs.current.set(source,proof);repaintBreakpoints();}break;
        }
        case "adapter_source": {void adapterSource.accept(event);break;}
        case "frame_variables": {
          const requested=frameRead.current,scope=threadModel.scope;if(!requested||!scope||statusRef.current!=="paused"||event.request_id!==requested.request||event.stop!==scope.stop||event.thread!==scope.thread||event.frame!==selectedFrameRef.current||requested.stop!==scope.stop||requested.thread!==scope.thread||requested.frame!==event.frame)break;
          frameRead.current=null;setFrameVariablesPending(false);setFrameVariablesMessage(null);localsByFrame.current.set(event.frame,event.variables);holdVariables(event.variables);replaceTreeLocals(event.variables);break;
        }
        case "value": {
          if(statusRef.current!=="paused")break;
          const scope=threadModel.scope;
          if(!scope)break;{const request=event.request_id&&evaluationRequests.current.get(event.request_id);if(!request||request.expression!==event.expression||latestEvaluation.current.get(event.expression)!==event.request_id||event.stop!==scope.stop||event.thread!==scope.thread||event.frame!==selectedFrameRef.current||request.stop!==scope.stop||request.thread!==scope.thread||request.frame!==selectedFrameRef.current)break;evaluationRequests.current.delete(event.request_id!);latestEvaluation.current.delete(event.expression);}
          setLastValue({ expression: event.expression, value: event.value, type: event.type_name??event.type??null });
          const inspected=valueScope();if(inspected&&Number.isSafeInteger(event.reference)&&event.reference>=0){valueTree.setScope(inspected);valueTree.replace(`expression:${event.expression}`,inspected,[{name:event.expression,value:event.value,type:event.type_name??event.type??null,variables_reference:event.reference}]);refreshTree();}
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
        case "children":
          if(valueTree.admit(event))refreshTree();break;
        case "finished":
          publishTest(event.session,"process exited",event.exit_code??null);
          rejectTest(Error("Test-debug preparation stopped."));launches.retire();movement.current=null;refreshTicket.current=null;valueTree.retire();refreshTree();
          // The server reaped the session with the program: the id no longer
          // answers, so forget it NOW. A step, hover or stop sent after this
          // would be a request to a dead session, answered only by an error.
          sessionRef.current = null;
          threadModel.clear();retireObservationScope();retireFrameRead();retireEvaluations();refreshThreads();holdStatus("finished");
          setExitCode(event.exit_code ?? null);
          setPausedLine(null);
          currentFrames.current=[];setFrames([]);setHistoricalFrames([]);
          holdVariables([]);
          setSelectedFrame(null);
          // The expressions survive — they are for the next run — but their
          // values belonged to a process that no longer exists.
          setWatches((current) => current.map((watch) => ({ ...watch, value: null })));
          break;
        case "ended":
          publishTest(event.session,"stopped");
          movement.current=null;refreshTicket.current=null;valueTree.retire();refreshTree();
          launches.retire();
          sessionRef.current = null;
          threadModel.clear();retireObservationScope();retireFrameRead();retireEvaluations();refreshThreads();holdStatus("idle");
          clearFailure();
          setPausedLine(null);
          currentFrames.current=[];setFrames([]);setHistoricalFrames([]);
          holdVariables([]);
          setCapabilities(null);
          setWatches((current) => current.map((watch) => ({ ...watch, value: null })));
          break;
        case "failed": {
          if(event.about==='source'){adapterSource.failed(event.session,event.request_id,event.message);break;}
          if(!event.session&&pendingTest.current?.request===event.request_id)rejectTest(Error(event.message));
          if(event.about==='children'){if(valueTree.fail(event.request_id,event.message))refreshTree();break;}
          if(event.about==='step'||event.about==='jump'||event.about==='run_to'){
            const prior=movement.current;if(!prior||event.request_id!==prior.request||event.session!==prior.session)break;movement.current=null;setMessage(event.message);
            if(event.scope_retired===false&&sessionRef.current===prior.session&&threadModel.restore(prior.scope)){
              currentFrames.current=prior.frames;setFrames(prior.frames);holdVariables(prior.variables);selectedFrameRef.current=prior.selected;setSelectedFrame(prior.selected);setPausedLine(prior.frames.find(frame=>frame.id===prior.selected)?.line??null);scopeRef.current=prior.observation;setObservationScope(prior.observation);holdStatus('paused');
              const scope=valueScope();if(scope){valueTree.setScope(scope);const slots=new Map<string,Variable[]>();for(const root of prior.roots){const rows=slots.get(root.slot)??[];rows.push({name:root.name,value:root.value,type_name:root.type,variables_reference:root.reference});slots.set(root.slot,rows);}for(const [slot,rows] of slots)valueTree.replace(slot,scope,rows);}refreshTree();refreshThreads();
            }else {setHistoricalFrames(prior.frames);currentFrames.current=[];setFrames([]);holdVariables([]);selectedFrameRef.current=null;setSelectedFrame(null);setPausedLine(null);threadModel.clear();retireObservationScope();retireFrameRead();retireEvaluations();valueTree.retire();refreshTree();refreshThreads();holdStatus('unconfirmed');}
            break;
          }
          if(event.about==='state'&&typeof event.request_id==='string'&&event.request_id===refreshTicket.current){refreshTicket.current=null;setMessage(event.message);break;}
          if(event.about==='breakpoints'||event.about==='source_breakpoints'){
            const source=event.about==='breakpoints'?uri:event.doc||uri;if(breakpointModel.failed(source,event.session??'',event.request_id,event.message)){repaintBreakpoints();}break;
          }
          if(event.about==="state"&&event.request_id){const requested=frameRead.current;if(!requested||requested.request!==event.request_id)break;frameRead.current=null;setFrameVariablesPending(false);setFrameVariablesMessage(event.message);break;}
          if(event.about==="threads"||event.about==="select_thread"){if(threadModel.failed(event.message,event.request_id)){if(event.scope_retired){setHistoricalFrames(currentFrames.current);currentFrames.current=[];setFrames([]);holdVariables([]);selectedFrameRef.current=null;setSelectedFrame(null);setPausedLine(null);threadModel.clear();retireObservationScope();retireFrameRead();retireEvaluations();valueTree.retire();refreshTree();holdStatus("unconfirmed");}else if(event.about==="select_thread")replaceTreeLocals(variablesRef.current);refreshThreads();}break;}
          if(event.about==="state"&&!event.request_id)break;
          if(event.about==="eval"&&!event.request_id)break;
          if(event.about==="eval"&&event.request_id){const request=evaluationRequests.current.get(event.request_id);if(!request||latestEvaluation.current.get(request.expression)!==event.request_id)break;evaluationRequests.current.delete(event.request_id);latestEvaluation.current.delete(request.expression);const waiting=pendingValues.current.get(request.expression);pendingValues.current.delete(request.expression);waiting?.(null);if(request.context!=="hover")setMessage(event.message);break;}
          const stillWaiting = [...pendingValues.current.values()];
          pendingValues.current.clear();
          for (const resolve of stillWaiting) resolve(null);
          // A breakpoint that could not be set is a fact about that
          // breakpoint. It is marked broken, with the reason on hover, and
          // says nothing in the panel — where it would outlive its cause and
          // read as if it were about whatever you did next.
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
      ownerMounted.current=false;ownerLifetime.current++;adapterSource.retire();rejectTest(Error("Debugger owner closed before test admission."));movement.current=null;refreshTicket.current=null;valueTree.retire();scopeRef.current=null;pendingReadings.current.clear();launches.retire();const session=sessionRef.current;sessionRef.current=null;statusRef.current="idle";currentFrames.current=[];frameRead.current=null;evaluationRequests.current.clear();latestEvaluation.current.clear();for(const resolve of pendingValues.current.values())resolve(null);pendingValues.current.clear();
      if (client && session) client.stop(session);
    };
  }, [client,uri]);

  const toggleBreakpoint = useCallback((line:number)=>{
    const held=currentRows().find(row=>row.line===line),requested=held?.requested_line??line;
    editIntent(uri,marks=>marks.some(mark=>mark.line===requested)?marks.filter(mark=>mark.line!==requested):[...marks,{line:requested}]);
  },[client,uri]);
  const setSourceBreakpoints=useCallback((source:string,next:HeldBreakpoint[])=>editIntent(source,()=>onTheWire(next)),[client,uri]);

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

  const setBreakpointCondition=useCallback((line:number,patch:{condition?:string;hit_condition?:string;log_message?:string})=>{
    const held=currentRows().find(row=>row.line===line),requested=held?.requested_line??line;
    editIntent(uri,marks=>marks.map(mark=>mark.line===requested?{...mark,...patch}:mark));
  },[client,uri]);
  const replaceBreakpoints=useCallback((next:HeldBreakpoint[],provenance?:string)=>{
    breakpointModel.unknown(uri,onTheWire(next));repaintBreakpoints();if(!provenance)return;
    try{const reading=readSource(uri);void Promise.resolve(reading).then(value=>{if(!ownerMounted.current)return;breakpointModel.preferences(uri,value,onTheWire(next),provenance);repaintBreakpoints();}).catch(error=>setMessage(error instanceof Error?error.message:String(error)));}catch(error){setMessage(error instanceof Error?error.message:String(error));}
  },[uri]);

  const launchProfile = useRef<import("./launchProfiles").LaunchSelection>();
  const launchContext = useRef(projectContext);
  useEffect(() => { launchContext.current = projectContext; launchProfile.current = undefined; }, [uri, projectContext]);
  const start=useCallback((program?:string,sources:Record<string,HeldBreakpoint[]>={},selection?:string,profile?:import('./launchProfiles').LaunchSelection,origins:Record<string,CapturedDebugIntent>={},test?:TestPending)=>{
    if(!test&&testIntent.current){setMessage('This debugger retains a test selection. Release that context in Debug options before ordinary program Debug, or prepare a fresh test Debug choice.');return;}
    if(test&&(!test.current()||['starting','running','paused','unconfirmed','stopping'].includes(statusRef.current))){test.reject(Error('The source changed or this editor already owns an active debugger. No test launch was sent.'));return;}
    if(!client){test?.reject(Error('Debugger transport unavailable.'));return;}if(launchAdmission.current?.()===false){setMessage('This source address is no longer owned by this editor. Existing controls still address its captured launch.');test?.reject(Error('Test source ownership changed.'));return;}
    const previous=sessionRef.current,request_id=launches.begin(),lifetime=ownerLifetime.current;launchContext.current=selection??launchContext.current;launchProfile.current=profile??(selection!==undefined?undefined:launchProfile.current);
    if(test){testContext.current=null;testIntent.current=test.prepared;test.request=request_id;pendingTest.current=test;launchContext.current='new';launchProfile.current=undefined;}
    const context=launchContext.current,chosen=launchProfile.current;threadModel.clear();retireObservationScope();retireFrameRead();retireEvaluations();valueTree.retire();refreshTree();refreshThreads();holdStatus('starting');clearFailure();setProgram(program??null);setLaunchTarget(test?.prepared.plan.name??chosen?.name??null);
    for(const [source,marks] of Object.entries(sources))if(source!==uri)breakpointModel.unknown(source,onTheWire(marks));
    const candidates=new Set([...breakpointModel.sources(),...Object.keys(origins)]);candidates.delete(uri);
    let primary:DebugSourceReading|Promise<DebugSourceReading>;const additional=new Map<string,DebugSourceReading|Promise<DebugSourceReading>>();
    try{primary=readSource(uri);for(const source of candidates){const known=origins[source]??breakpointModel.captured(source);if(known&&(known.reading.current()||known.reading.provenance))additional.set(source,readSource(source));}}catch(error){launches.retire();holdStatus('failed');setMessage(error instanceof Error?error.message:String(error));rejectTest(error instanceof Error?error:Error(String(error)));return;}
    const initial=Promise.resolve(primary);initial.catch(()=>{});for(const reading of additional.values())Promise.resolve(reading).catch(()=>{});
    const local=client.provider;
    // The validated test-debug plan versions dedicated start_test admission,
    // including protocol3 source/scope guards. An ordinary launch catalog
    // would ask the reader to choose a file block the test plan already owns.
    const protocol=test?(action:(catalog:{workspace:string;notices?:unknown})=>Promise<void>)=>initial.then(reading=>action({workspace:reading.workspace})):local?(action:(catalog:{workspace:string;notices?:unknown})=>Promise<void>)=>local.protocol>=3?action({workspace:local.workspace}):Promise.reject(Error('This debugger provider does not implement guarded protocol3.')):(action:(catalog:{workspace:string;notices?:unknown})=>Promise<void>)=>withDebugProtocol({uri,project:context,program},action);
    void protocol(async catalog=>{
      if(ownerMounted.current&&ownerLifetime.current===lifetime&&launches.matches(request_id))setLaunchNotices([...launchCatalogNotices(catalog),...(test?.prepared.plan.notices??[])].slice(0,64));
      const reading=await initial;
      // Embedded startPausedAt is an explicit placement in this provider's
      // authored model, unlike unknown persisted path/line preferences.
      if(local&&initialBreakpoints.length&&!breakpointModel.captured(uri))breakpointModel.change(uri,reading,()=>initialBreakpoints.map(line=>({line})));
      if(reading.workspace!==catalog.workspace)throw Error(`Debugger source belongs to another workspace namespace (${JSON.stringify(reading.workspace)}; current ${JSON.stringify(catalog.workspace)}). Reload it before Debug.`);
      const proof=await stampDebugSource(reading),proofs=new Map<string,DebugSourceProof>([[uri,proof]]),sets:Record<string,import('./client').DebugBreakpoint[]>=Object.create(null),hashes:Record<string,string>=Object.create(null);
      for(const [source,pending] of additional){const current=await pending,captured=origins[source];if(captured&&!breakpointModel.adopt(source,current,captured))continue;const known=breakpointModel.captured(source);if(!known||!current.saved||!known.reading.current()&&!current.provenance)continue;
        const stamped=await stampDebugSource(current);if(stamped.workspace!==reading.workspace)throw Error('An additional breakpoint source belongs to another workspace.');const ticket=breakpointModel.request(source,stamped,null,request_id);if(!ticket)continue;sets[source]=ticket.marks;hashes[source]=stamped.hash;proofs.set(source,stamped);
      }
      const root=breakpointModel.request(uri,proof,null,request_id);
      if(!ownerMounted.current||ownerLifetime.current!==lifetime||launchContext.current!==context||launchProfile.current!==chosen||!launches.matches(request_id))return;
      if(test&&(!test.current()||proof.hash!==test.prepared.choice.inputHash||uri!==test.prepared.choice.sourceUri))throw Error('Test-debug source or selection changed during preparation.');
      if(launchAdmission.current?.()===false||[...proofs.values()].some(value=>!value.current()))throw Error('Source ownership changed during debugger preparation. Observe it again.');
      preparedStart.current={request:request_id,proofs};runHashes.current.clear();runProofs.current.clear();repaintBreakpoints();sessionRef.current=null;if(previous)client.stop(previous);
      if(test){test.sent=true;client.startTest(uri,root?.marks??[],sets,request_id,proof.hash,hashes,test.prepared.choice.request);}else client.start(uri,root?.marks??[],program,context,sets,chosen,request_id,proof.hash,hashes);
    }).catch(error=>{if(!ownerMounted.current||!launches.matches(request_id))return;launches.retire();holdStatus('failed');setMessage(error instanceof Error?error.message:String(error));rejectTest(error instanceof Error?error:Error(String(error)));});
  },[client,uri,projectContext]);

  const startSourceTest=useCallback((prepared:PreparedSourceTestDebug,current:()=>boolean)=>new Promise<SourceTestDebugProcess>((resolve,reject)=>{
    try{if(client?.provider)throw Error('This browser-local provider does not implement source-test Debug admission.');prepareSourceTestDebug(prepared.choice,prepared.plan);if(uri!==prepared.choice.sourceUri)throw Error('This debugger belongs to a different authored input.');start(undefined,{},undefined,undefined,{}, {prepared,current,request:null,sent:false,preparing:false,resolve,reject});}catch(error){reject(error instanceof Error?error:Error(String(error)));}
  }),[start,client,uri]);
  const cancelSourceTestStart=useCallback((prepared:PreparedSourceTestDebug)=>{const held=pendingTest.current;if(!held||held.prepared!==prepared)return;pendingTest.current=null;launches.retire();client?.cancelStart(uri);held.reject(Error(held.sent?'Test-debug admission canceled after dispatch; the owned late session will be stopped without adoption.':'Test-debug preparation canceled; no Start was sent.'));if(held.sent&&held.request){canceledTestRequests.current.add(held.request);holdStatus('unconfirmed');setMessage('Canceled test-debug request awaits its owned session cleanup.');}else {if(!testContext.current)testIntent.current=null;holdStatus('idle');paintTest(value=>value+1);}},[client,uri]);
  const releaseSourceTest=useCallback(()=>{if(['starting','running','paused','unconfirmed','stopping'].includes(statusRef.current))return;testContext.current=null;testIntent.current=null;paintTest(value=>value+1);},[]);

  const stop = useCallback(() => {
    if(pendingTest.current){cancelSourceTestStart(pendingTest.current.prepared);return;}
    if(!sessionRef.current&&(canceledTestRequests.current.size||testCleanup.current.size)){for(const owned of testCleanup.current)client?.stop(owned);client?.cancelStart(uri);holdStatus(testCleanup.current.size?'stopping':'unconfirmed');setMessage('Waiting for positively owned test-debug cleanup; no current process or test verdict was adopted.');return;}
    launches.retire();movement.current=null;refreshTicket.current=null;
    if(sessionRef.current){setHistoricalFrames(currentFrames.current);currentFrames.current=[];setFrames([]);holdVariables([]);selectedFrameRef.current=null;setSelectedFrame(null);setPausedLine(null);threadModel.clear();retireObservationScope();retireFrameRead();retireEvaluations();valueTree.retire();refreshTree();refreshThreads();holdStatus('stopping');setMessage('Stopping debugger…');}
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
  }, [client, uri,cancelSourceTestStart]);

  function moveDebug(kind:Step|'jump'|'run_to',line?:number){
    const session=sessionRef.current,scope=threadModel.scope;if(!client||!session||statusRef.current!=='paused'||!scope||threadModel.selecting||movement.current)return;
    const selected=selectedFrameRef.current,frame=currentFrames.current.find(value=>value.id===selected);
    if(kind==='drop_frame'&&(!frame||!capabilitiesRef.current?.restart_frame)){setMessage('A current reported frame is required to restart it.');return;}
    let hash:string|undefined;if(kind==='jump'||kind==='run_to'){
      const proof=runProofs.current.get(uri);if(!frame?.in_document||!primaryBound()||!proof||line===undefined||!Number.isInteger(line)||line<0||line>=proof.lines){setMessage('Choose a current captured primary-source frame and exact source line before positional movement.');return;}
      if(kind==='jump'&&!capabilitiesRef.current?.goto_targets||kind==='run_to'&&!capabilitiesRef.current?.run_to){setMessage('The current provider does not support this positional movement.');return;}hash=proof.hash;
    }
    const request=crypto.randomUUID();movement.current={request,session,scope:{...scope},frames:currentFrames.current,variables:variablesRef.current,selected,observation:scopeRef.current,roots:valueTree.roots()};
    setHistoricalFrames(currentFrames.current);currentFrames.current=[];setFrames([]);holdVariables([]);selectedFrameRef.current=null;setSelectedFrame(null);setPausedLine(null);threadModel.clear();retireObservationScope();retireFrameRead();retireEvaluations();valueTree.retire();refreshTree();refreshThreads();holdStatus('running');
    const ticket={stop:scope.stop,thread:scope.thread,request_id:request};if(kind==='jump')client.jump(session,line!,{...ticket,source_hash:hash!});else if(kind==='run_to')client.runTo(session,line!,{...ticket,source_hash:hash!});else client.step(session,kind,frame?.id,ticket);
  }
  const step=useCallback((how:Step)=>moveDebug(how),[client,uri]);
  const jumpTo=useCallback((line:number)=>moveDebug('jump',line),[client,uri]);
  const runTo=useCallback((line:number)=>moveDebug('run_to',line),[client,uri]);
  const refreshState=useCallback(()=>{const session=sessionRef.current;if(!client||!session||statusRef.current!=='unconfirmed'||refreshTicket.current)return;const request_id=crypto.randomUUID();refreshTicket.current=request_id;client.state(session,undefined,{request_id});setMessage('Reading an actual queued stop; the previous pause remains historical until confirmed.');},[client]);
  const expandValue=useCallback((id:string)=>{const scope=valueScope();if(!client||!scope)return;const request=valueTree.expand(id);refreshTree();if(request)client.children(request.session,request.reference,{stop:request.stop,thread:request.thread,frame:request.frame,request_id:request.request_id});},[client]);
  const collapseValue=useCallback((id:string)=>{valueTree.collapse(id);refreshTree();},[]);
  const closeValues=useCallback(()=>{if(valueTree.closeInspection())refreshTree();},[]);

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
    valueTree.remove(`expression:${expression}`);const request=latestEvaluation.current.get(expression);if(request)evaluationRequests.current.delete(request);latestEvaluation.current.delete(expression);const waiting=pendingValues.current.get(expression);pendingValues.current.delete(expression);waiting?.(null);refreshTree();
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
  const selectThread=useCallback((id:number)=>{if(!client||statusRef.current!=="paused"||!sessionRef.current)return;const request=threadModel.select(id);refreshThreads();if(request&&request.session===sessionRef.current){retireFrameRead();retireEvaluations();valueTree.retire();refreshTree();client.selectThread(request.session,id,request.stop,request.request);}},[client,threadModel]);

  const captureFrame=useCallback((id:number)=>{const frame=currentFrames.current.find(item=>item.id===id),session=sessionRef.current,scope=threadModel.scope,lifetime=ownerLifetime.current;
    if(!frame||!session||statusRef.current!=="paused"||selectedFrameRef.current!==id||threadModel.selecting)return null;
    return {frame:Object.freeze({...frame,source_hash:!launchBound()||frame.in_document&&!primaryBound()?null:frame.source_hash}),current:()=>{const latest=currentFrames.current.find(item=>item.id===id);return ownerMounted.current&&ownerLifetime.current===lifetime&&sessionRef.current===session&&statusRef.current==="paused"&&!!latest&&["name","source","source_line","line","in_document","source_hash","source_reference"].every(key=>latest[key as keyof Frame]===frame[key as keyof Frame])&&selectedFrameRef.current===id&&!threadModel.selecting&&(!scope?threadModel.scope===null:threadModel.scope?.stop===scope.stop&&threadModel.scope.thread===scope.thread);}};
  },[threadModel]);

  const readAdapterSource=useCallback((id:number|undefined=selectedFrameRef.current??undefined)=>{
    const session=sessionRef.current,scope=threadModel.scope,captured=id===undefined?null:captureFrame(id);
    if(capabilitiesRef.current?.source_content!==true){setMessage('This engine does not support reading adapter-owned source; workspace text was not substituted.');return;}
    if(!client||!session||!scope||id===undefined||!captured||!(captured.frame.source_reference&&captured.frame.source_reference>0)){setMessage('Select a current stopped frame with adapter-owned source before reading it.');return;}
    adapterSource.read({session,stop:scope.stop,thread:scope.thread,frame:id,reference:captured.frame.source_reference},captured.current,(held,request)=>client.source(held.session,held.reference,held.frame,{stop:held.stop,thread:held.thread,request_id:request}));
  },[client,threadModel,captureFrame,adapterSource]);

  const selectFrame = useCallback(
    (id: number) => {
      const frame = frames.find((candidate) => candidate.id === id);
      if(currentFrames.current!==frames||!frame||!debugAdapterId(id)||statusRef.current!=="paused"||threadModel.selecting)return;
      retireFrameRead();retireEvaluations();valueTree.retire();refreshTree();holdVariables(localsByFrame.current.get(id)??[]);setWatches(current=>current.map(watch=>({...watch,value:null})));selectedFrameRef.current=id;setSelectedFrame(id);if(localsByFrame.current.has(id))replaceTreeLocals(localsByFrame.current.get(id)!);
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
      startSourceTest,cancelSourceTestStart,releaseSourceTest,
      adapterSource,readAdapterSource,
      sourceTestIntent:testIntent.current,sourceTest:testContext.current?testHistory.current.get(testContext.current)??null:null,
      sourceTestProcesses:()=>[...testHistory.current.values()].map(record=>({...record})),sourceTestInspectable:(session:string)=>testContext.current===session&&(sessionRef.current===session||!['starting','running','paused','unconfirmed','stopping'].includes(statusRef.current)&&lastOwnedSession.current===session),
      isActive:()=>["starting","running","paused","unconfirmed","stopping"].includes(statusRef.current),
      launchNotices,breakpointRows:()=>currentRows(),capturedBreakpointIntents:capturedIntents,
      breakpointNotice:breakpointModel.notice(uri,sourceReadings.current.get(uri)??null,sessionRef.current,runHashes.current.get(uri)??null),
      breakpointIntentions:[...breakpointModel.archived.map(intent=>({...intent,historical:true})),...breakpointModel.sources().map(source=>({source,marks:breakpointModel.desired(source),namespace:breakpointModel.captured(source)?.reading.namespace??null,hash:breakpointModel.acceptedProof(source)?.hash??null,historical:false}))],
      variableRoots:valueTree.roots(),valueNode:(id:string)=>valueTree.node(id),valueHistory:[...valueTree.historical],valueNotice:valueTree.notice,expandValue,collapseValue,closeValues,refreshState,positionSourceBound:primaryBound(),
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
      breakpoints:currentRows(),
      sourceBreakpoints, setSourceBreakpoints,
      replaceBreakpoints,
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
      adapterSourceVersion,adapterSource,readAdapterSource,
      testEpoch,startSourceTest,cancelSourceTestStart,releaseSourceTest,launchNotices,treeRevision,expandValue,collapseValue,closeValues,refreshState,threadRevision,readThreads,selectThread,frameVariablesPending,frameVariablesMessage,stackMessage,historicalFrames,observationScope,captureFrame,
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
      sourceBreakpoints, setSourceBreakpoints,replaceBreakpoints,
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
import { describe, expect, it,vi,beforeAll } from "vitest";
import {withDebugProtocol} from "./launchProfiles";
vi.mock("./launchProfiles",()=>({launchCatalogNotices:(catalog:{notices?:string[]})=>catalog.notices??[],withDebugProtocol:vi.fn(async(_query:unknown,effect:(catalog:{workspace:string})=>void|Promise<void>)=>{await effect({workspace:"fixture-root"});})}));
import {savedDebugSources} from "./SourceGuards";
const fixtureSource=Array.from({length:100},(_,i)=>`line ${i}`).join('\n');
beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});vi.spyOn(savedDebugSources,'read').mockImplementation(uri=>({uri,workspace:'fixture-root',namespace:'fixture-root',identity:uri,text:fixtureSource,saved:true,current:()=>true}));});
async function settle(action:()=>void){await act(async()=>{action();await new Promise(resolve=>setTimeout(resolve,10));});}
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
    if(event.event==='value'||event.event==='failed'&&event.session){const op=event.event==='value'?'eval':event.about;const last=[...this.sent].reverse().find(item=>(item as {op?:string;expression?:string}).op===op&&(op!=='eval'||(item as {expression?:string}).expression===event.expression||event.event==='failed')) as {request_id?:string;stop?:number;thread?:number;frame?:number}|undefined;event={...event,request_id:event.request_id??last?.request_id,...(event.event==='value'?{stop:event.stop??last?.stop,thread:event.thread??last?.thread,frame:event.frame??last?.frame}:{})};}
    if(event.event==='started'&&event.source_hash===undefined){const start=[...this.sent].reverse().find(item=>(item as {op?:string}).op==='start') as {source_hash?:string}|undefined;event={...event,source_hash:start?.source_hash};}
    if(event.event==='breakpoints'||event.event==='source_breakpoints'){const op=event.event==='breakpoints'?'breakpoints':'source_breakpoints',last=[...this.sent].reverse().find(item=>(item as {op?:string;source?:string}).op===op&&(op==='breakpoints'||(item as {source?:string}).source===event.source)) as {request_id?:string;source_hash?:string}|undefined;event={...event,request_id:event.request_id??last?.request_id,source_hash:event.source_hash??last?.source_hash};}
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
  event: "stopped",stop:1,thread:1,
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
  it("retains newly placed callee intentions without rebinding an existing launch, then includes them in a fresh source-bound launch",async()=>{
    const {socket,hook}=open();await settle(()=>hook.result.current.start());await settle(()=>{socket.deliver(STARTED);socket.deliver(STOPPED);});const caller=hook.result.current.breakpoints,count=socket.sent.length;
    await settle(()=>hook.result.current.setSourceBreakpoints("hick:///helpers.py",[{line:25,state:"pending",condition:"x > 1"}]));expect(socket.sent).toHaveLength(count);expect(hook.result.current.breakpoints).toEqual(caller);expect(hook.result.current.sourceBreakpoints["hick:///helpers.py"]).toEqual([]);
    await settle(()=>socket.deliver({event:"finished",session:"dbg-0"}));await settle(()=>hook.result.current.start());const start=socket.sent.at(-1) as {request_id:string;source_hashes:Record<string,string>};expect(start).toMatchObject({op:"start",sources:{"hick:///helpers.py":[{line:25,condition:"x > 1"}]}});expect(start.source_hashes['hick:///helpers.py']).toMatch(/^[a-f0-9]{64}$/);
    await settle(()=>{socket.deliver(STARTED);socket.deliver({event:'source_breakpoints',session:'dbg-0',source:'hick:///helpers.py',request_id:start.request_id,source_hash:start.source_hashes['hick:///helpers.py'],breakpoints:[{line:25,state:'bound',moved_to:26}]});});expect(hook.result.current.sourceBreakpoints['hick:///helpers.py']).toMatchObject([{line:26,requested_line:25,state:'bound',condition:'x > 1'}]);
    await settle(()=>hook.result.current.setSourceBreakpoints('hick:///helpers.py',[{line:25,state:'pending',condition:'x > 2'}]));expect(socket.sent.at(-1)).toMatchObject({op:'source_breakpoints',source:'hick:///helpers.py',source_hash:start.source_hashes['hick:///helpers.py'],breakpoints:[{line:25,condition:'x > 2'}]});
  });

  it("leaves 'starting' when the engine answers", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    expect(hook.result.current.status).toBe("starting");
    expect(socket.sent[0]).toMatchObject({ op: "start", doc: "hick:///orders.md" });

    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));
    expect(hook.result.current.capabilities?.goto_targets).toBe(true);
  });

  it("shows the paused line and the frame's values", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));

    await waitFor(() => expect(hook.result.current.status).toBe("paused"));
    expect(hook.result.current.pausedLine).toBe(14);
    expect(hook.result.current.frames[0].name).toBe("line_total");
    expect(hook.result.current.variables[0].value).toBe("2");
    // The top frame is selected, so evaluating goes to the right one.
    expect(hook.result.current.selectedFrame).toBe(2);
  });

  it("sends the session and exact current scope with every admitted request",async()=>{
    const {socket,hook}=open();await settle(()=>hook.result.current.start());await settle(()=>socket.deliver(STARTED));await settle(()=>socket.deliver(STOPPED));
    await settle(()=>hook.result.current.step('over'));expect(socket.sent.at(-1)).toMatchObject({op:'step',session:'dbg-0',how:'over',frame:2,stop:1,thread:1,request_id:expect.any(String)});
    await settle(()=>socket.deliver({...STOPPED,stop:2}));await settle(()=>hook.result.current.evaluate('quantity * 2'));expect(socket.sent.at(-1)).toMatchObject({op:'eval',session:'dbg-0',expression:'quantity * 2',context:'repl',frame:2,stop:2,thread:1,request_id:expect.any(String)});
    await settle(()=>hook.result.current.jumpTo(12));expect(socket.sent.at(-1)).toMatchObject({op:'jump',session:'dbg-0',line:12,stop:2,thread:1,source_hash:expect.any(String),request_id:expect.any(String)});
  });

  it("does nothing before a session exists", async() => {
    // Stepping with no session would be a request the engine answers with an
    // error, and the error would be the UI's fault.
    const { socket, hook } = open();
    await settle(() => hook.result.current.step("over"));
    await settle(() => hook.result.current.evaluate("x"));
    expect(socket.sent).toMatchObject([]);
  });

  it("keeps breakpoints across a session, and sends them when one starts", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.toggleBreakpoint(14));
    await settle(() => hook.result.current.start());
    expect(socket.sent.at(-1)).toMatchObject({ breakpoints: [{ line: 14 }] });

    await settle(() => socket.deliver({...STARTED,breakpoints:[{line:14,state:"bound",moved_to:25}]}));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));
    // The engine's answer replaces the optimistic set: it knows which bound.
    expect(hook.result.current.breakpoints).toEqual([{ line:25,requested_line:14,state:"bound" }]);
  });

  it("reports a failure instead of waiting forever", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() =>
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
    await settle(() => hook.result.current.start());
    await settle(() =>
      socket.deliver({
        event: "failed",
        message: "no debug adapter for python on this machine.",
        offer_install: { kind: "dap", language: "python" },
      }),
    );
    await waitFor(() =>
      expect(hook.result.current.offerInstall).toEqual({ kind: "dap", language: "python" }),
    );

    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));
    expect(hook.result.current.offerInstall).toBeNull();
    expect(hook.result.current.message).toBeNull();
  });

  it("asks only about names the frame has", async () => {
    // Hovering prose, a tag name or a comment used to send the word to the
    // debugger and get a `NameError` back — an error about our question, not
    // about the program.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));
    const before = socket.sent.length;

    await act(async () => {
      expect(await hook.result.current.valueAt("order")).toBeNull();
      expect(await hook.result.current.valueAt("hick:file")).toBeNull();
    });
    expect(socket.sent.length).toBe(before);

    // A name that IS in the frame goes out, attribute access included.
    void hook.result.current.valueAt("quantity");
    void hook.result.current.valueAt("quantity.bit_length");
    expect(socket.sent.slice(before)).toMatchObject([
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
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));

    let answered: string | null = "unset";
    await settle(() => {
      void hook.result.current.valueAt("quantity").then((v) => {
        answered = v;
      });
    });
    await settle(() => socket.deliver({ event: "failed",session:"dbg-0",about:"eval",message:"NameError: nope" }));
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
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));
    await settle(() => hook.result.current.toggleBreakpoint(14));

    await settle(() =>
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
    // An unsolicited startup line is not an admitted intention or receipt.
    expect(hook.result.current.breakpoints.find((b) => b.line === 25)).toBeUndefined();
  });

  it("keeps a breakpoint local once the program has ended", async () => {
    // Setting one after the program finished used to send it to a session
    // that could only refuse: the breakpoint is for the next run.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver({ event: "finished", session: "dbg-0" }));
    await waitFor(() => expect(hook.result.current.status).toBe("finished"));
    const before = socket.sent.length;

    await settle(() => hook.result.current.toggleBreakpoint(14));
    expect(socket.sent.length).toBe(before);
    expect(hook.result.current.breakpoints.some((b) => b.line === 14)).toBe(true);
  });

  it("re-asks every watch at each pause and frame change", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));
    await waitFor(() => expect(hook.result.current.status).toBe("paused"));

    // Adding a watch while paused asks straight away, in `watch` context so
    // the adapter treats it as something evaluated repeatedly.
    await settle(() => hook.result.current.addWatch("quantity * 2"));
    expect(socket.sent.at(-1)).toMatchObject({
      op: "eval",
      session: "dbg-0",
      expression: "quantity * 2",
      context: "watch",
      frame: 2,
    });

    // The answer correlates by the expression it echoes back.
    await settle(() =>
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
    await settle(() => hook.result.current.step("over"));
    const before = socket.sent.length;
    await settle(() => socket.deliver({ ...STOPPED,stop:2, line:15 }));
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
    await settle(() => hook.result.current.addWatch("quantity"));
    await settle(() => hook.result.current.addWatch("quantity")); // and never twice
    expect(hook.result.current.watches).toEqual([{ expression: "quantity", value: null }]);

    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));
    await settle(() =>
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
    await settle(() => socket.deliver({ event: "finished", session: "dbg-0" }));
    await waitFor(() => expect(hook.result.current.watches).toEqual([
      { expression: "quantity", value: null },
    ]));

    await settle(() => hook.result.current.removeWatch("quantity"));
    expect(hook.result.current.watches).toEqual([]);
  });

  it("answers a typed expression as a promise, without the known-name guard", async () => {
    // `valueAt` refuses names the frame does not have (it exists for hover);
    // the inline eval is a person typing, and any expression is fair.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));
    await waitFor(() => expect(hook.result.current.status).toBe("paused"));

    let answered: string | null = null;
    await settle(() => {
      void hook.result.current.query("len('abc')").then((value) => {
        answered = value;
      });
    });
    expect(socket.sent.at(-1)).toMatchObject({
      op: "eval",
      session: "dbg-0",
      expression: "len('abc')",
      context: "repl",
      frame: 2,
    });
    await settle(() =>
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
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));
    await settle(() => socket.deliver({ event: "finished", session: "dbg-0", exit_code: 0 }));

    await waitFor(() => expect(hook.result.current.status).toBe("finished"));
    // The end says how it ended, and the paused chrome is gone.
    expect(hook.result.current.exitCode).toBe(0);
    expect(hook.result.current.pausedLine).toBeNull();
    expect(hook.result.current.frames).toEqual([]);
    expect(hook.result.current.variables).toEqual([]);

    // No request may reach the dead session: not a step, not an eval, not a
    // hover — each would be answered only by an error about a gone session.
    const before = socket.sent.length;
    await settle(() => hook.result.current.step("over"));
    await settle(() => hook.result.current.evaluate("quantity"));
    await settle(() => hook.result.current.jumpTo(12));
    await act(async () => {
      expect(await hook.result.current.valueAt("quantity")).toBeNull();
      expect(await hook.result.current.query("1 + 1")).toBeNull();
    });
    expect(socket.sent.length).toBe(before);

    // Stop is now a dismissal: nothing goes out, the strip clears locally.
    await settle(() => hook.result.current.stop());
    expect(socket.sent.length).toBe(before);
    await waitFor(() => expect(hook.result.current.status).toBe("idle"));
  });

  it("does not send a stop for an already-finished session on unmount", async () => {
    // The unmount cleanup exists for a window closed MID-session. After
    // "finished" there is nothing to stop, and sending one anyway is a
    // request to a session the server already reaped.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver({ event: "finished", session: "dbg-0", exit_code: 0 }));
    await waitFor(() => expect(hook.result.current.status).toBe("finished"));

    const before = socket.sent.length;
    hook.unmount();
    expect(socket.sent.length).toBe(before);
  });

  it("keeps 'finished' when a stray failure arrives afterwards", async () => {
    // A hover or watch answered late, after the program ended, must not
    // redress a clean end as a broken one.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver({ event: "finished", session: "dbg-0", exit_code: 0 }));
    await waitFor(() => expect(hook.result.current.status).toBe("finished"));

    await settle(() => socket.deliver({ event: "failed", session: "dbg-0", message: "too late" }));
    expect(hook.result.current.status).toBe("finished");
  });

  it("goes back to idle when the session ends", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));
    await settle(() => socket.deliver({ event: "ended", session: "dbg-0" }));

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

    await settle(() => a.result.current.start());
    expect(sent[0]).toMatchObject({ op: "start", doc: "hick:///tools/a.py" });
    await settle(() => {
      client.handleFrame(frame({ ...STARTED, doc: "hick:///tools/a.py",request_id:(sent[0] as {request_id:string}).request_id }));
    });
    await waitFor(() => expect(a.result.current.status).toBe("running"));
    expect(b.result.current.status).toBe("idle");

    await settle(() => {
      client.handleFrame(frame(STOPPED));
    });
    await waitFor(() => expect(a.result.current.status).toBe("paused"));
    expect(b.result.current.status).toBe("idle");
    expect(b.result.current.pausedLine).toBeNull();

    // A start that fails names the file it was for, and lands there only.
    await settle(() => b.result.current.start());
    await settle(() => {
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

  it("still takes an answer that names no file", async() => {
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
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    await settle(() => hook.result.current.toggleBreakpoint(10));
    await settle(() => hook.result.current.setBreakpointCondition(10, { condition: "i > 10" }));

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
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    await settle(() => hook.result.current.toggleBreakpoint(4));
    await settle(() =>
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
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    await settle(() => hook.result.current.toggleBreakpoint(7));
    await settle(() => hook.result.current.setBreakpointCondition(7, { condition: "x" }));
    await settle(() => hook.result.current.setBreakpointCondition(7, { condition: "  " }));

    const last = socket.sent.at(-1) as { breakpoints: Record<string, unknown>[] };
    expect(last.breakpoints.find((b) => b.line === 7)).toEqual({ line: 7 });
    expect(hook.result.current.breakpoints.find((b) => b.line === 7)?.condition).toBeUndefined();
  });

  it("keeps the condition when the adapter moves the breakpoint", async () => {
    // The subtle one. A breakpoint that slides to the next executable line
    // and silently loses its condition would stop on every pass — the exact
    // opposite of what was asked for.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    await settle(() => hook.result.current.toggleBreakpoint(12));
    await settle(() => hook.result.current.setBreakpointCondition(12, { condition: "n == 3" }));
    await settle(() =>
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
    await settle(() => hook.result.current.toggleBreakpoint(3));
    await settle(() => hook.result.current.setBreakpointCondition(3, { condition: "ready" }));
    // Nothing is running, so nothing was sent — the condition is for the run.
    expect(socket.sent).toHaveLength(0);

    await settle(() => hook.result.current.start());
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
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    await settle(() => hook.result.current.toggleExceptionFilter("uncaught"));
    expect(socket.sent.at(-1)).toMatchObject({
      op: "exception_breakpoints",
      session: "dbg-0",
      filters: ["uncaught"],
    });

    await settle(() => hook.result.current.toggleExceptionFilter("raised"));
    expect(socket.sent.at(-1)).toMatchObject({ filters: ["uncaught", "raised"] });

    await settle(() => hook.result.current.toggleExceptionFilter("uncaught"));
    expect(socket.sent.at(-1)).toMatchObject({ filters: ["raised"] });
    expect(hook.result.current.exceptionFilters).toEqual(["raised"]);
  });

  it("keeps them across runs and re-applies them to the next session", async () => {
    // A standing "stop on uncaught" that quietly stopped applying on the
    // second run would be worse than never having offered it.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));
    await settle(() => hook.result.current.toggleExceptionFilter("uncaught"));

    await settle(() => socket.deliver({ event: "finished", session: "dbg-0", exit_code: 0 }));
    await waitFor(() => expect(hook.result.current.status).toBe("finished"));
    expect(hook.result.current.exceptionFilters).toEqual(["uncaught"]);

    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver({ ...STARTED, session: "dbg-1" }));
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
it("keeps the clicked source context for restart and replaces it for a different surface", async() => {
  const { socket, hook } = open();
  const context = 'latest-file:["base","<hick:project/>source"]';
  await settle(() => hook.result.current.start("app.py", {}, context));
  expect(socket.sent.at(-1)).toMatchObject({ op: "start", program: "app.py", project: context });
  await settle(() => hook.result.current.start("app.py"));
  expect(socket.sent.at(-1)).toMatchObject({ project: context });
  await settle(() => hook.result.current.start("other.py", {}, "new"));
  expect(socket.sent.at(-1)).not.toHaveProperty("project");
  hook.unmount();
});


describe("source-current debugger thread observations",()=>{
 it("admits only the latest exact stopped list and never selects optimistically",async()=>{
  const model=new DebugThreads();model.stopped("session",3,1);const first=model.read()!,second=model.read()!;
  const event=(request:string,stop=3,selected=1)=>({event:"threads" as const,session:"session",stop,selected,request_id:request,threads:[{id:1,name:"main"},{id:2,name:"worker"}]});
  expect(model.accept(event(first.request))).toBe(false);expect(model.accept(event(second.request,2))).toBe(false);expect(model.accept({...event(second.request),session:"foreign"})).toBe(false);expect(model.accept(event(second.request))).toBe(true);
  const selected=model.select(2)!;expect(selected.stop).toBe(3);expect(model.scope?.thread).toBe(1);expect(model.select(1)).toBeNull();model.stopped("session",4,2);expect(model.reading).toBeNull();expect(model.accept(event(selected.request,4,2))).toBe(true);expect(model.scope?.thread).toBe(2);
  model.clear();expect(model.accept(event(selected.request,4,2))).toBe(false);expect(model.select(2)).toBeNull();
 });
 it("rejects malformed identities and retires unrelated stop generations while keeping optional failures local",async()=>{
  const model=new DebugThreads();model.stopped("session",1,1);const read=model.read()!,event={event:"threads" as const,session:"session",stop:1,selected:1,request_id:read.request,threads:[{id:1,name:"main"}]};expect(model.accept(event)).toBe(true);
  const refresh=model.read()!;expect(model.failed("provider unavailable","old request")).toBe(false);expect(model.busy).toBe(true);expect(model.failed("provider unavailable",refresh.request)).toBe(true);expect(model.reading?.threads).toEqual([{id:1,name:"main"}]);
  const next=model.read()!;expect(model.accept({...event,request_id:next.request,threads:[{id:1,name:"a"},{id:1,name:"b"}]})).toBe(false);expect(model.reading?.threads).toEqual([{id:1,name:"main"}]);model.stopped("session",2,1);expect(model.accept({...event,request_id:next.request})).toBe(false);
  expect(model.stopped("session",Number.MAX_SAFE_INTEGER+1,1)).toBe(false);expect(model.scope).toEqual({session:"session",stop:2,thread:1});
 });
 it("routes wire thread reads and selection through the hook without failing the paused stack",async()=>{
  const {socket,hook}=open(),result=hook.result;await settle(()=>result.current.start());await settle(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:7,thread:1});});await waitFor(()=>expect(result.current.status).toBe("paused"));
  const frames=result.current.frames;await settle(()=>result.current.readThreads?.());const request=socket.sent.at(-1) as {op:string;request_id:string};expect(request.op).toBe("threads");
  await settle(()=>socket.deliver({event:"failed",session:"dbg-0",about:"threads",request_id:request.request_id,message:"adapter read unavailable"}));expect(result.current.status).toBe("paused");expect(result.current.frames).toBe(frames);expect(result.current.threadMessage).toContain("unavailable");
  await settle(()=>result.current.readThreads?.());const latest=socket.sent.at(-1) as {request_id:string};await settle(()=>socket.deliver({event:"threads",session:"dbg-0",stop:7,selected:1,request_id:latest.request_id,threads:[{id:1,name:"main"},{id:2,name:"worker"}]}));await waitFor(()=>expect(result.current.threads).toHaveLength(2));
  await settle(()=>result.current.selectThread?.(2));const selection=socket.sent.at(-1) as {op:string;stop:number;request_id:string};expect(selection.op).toBe("select_thread");expect(selection.stop).toBe(7);expect(result.current.selectedThread).toBe(1);const sent=socket.sent.length;await settle(()=>{result.current.step("over");result.current.readThreads?.();result.current.selectFrame(2);});expect(socket.sent).toHaveLength(sent);expect(await result.current.query("quantity")).toBeNull();
  await settle(()=>socket.deliver({...STOPPED,stop:8,thread:2}));await settle(()=>socket.deliver({event:"threads",session:"dbg-0",stop:8,selected:2,request_id:selection.request_id,threads:[{id:1,name:"main"},{id:2,name:"worker"}]}));await waitFor(()=>expect(result.current.selectedThread).toBe(2));expect(result.current.status).toBe("paused");
 });
 it("correlates values to the requested stop, thread, frame and request before settling current expressions",async()=>{
  const {socket,hook}=open(),result=hook.result;await settle(()=>result.current.start());await settle(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:10,thread:1});});
  let value!:Promise<string|null>;await settle(()=>{value=result.current.query("quantity");});const first=socket.sent.at(-1) as {request_id:string;stop:number;thread:number};expect(first.stop).toBe(10);expect(first.thread).toBe(1);
  const reply={event:"value",session:"dbg-0",expression:"quantity",value:"2",type_name:"int",type:"legacy",reference:0,frame:2,thread:1,stop:10,request_id:first.request_id};
  await settle(()=>socket.deliver({...reply,stop:9}));expect(result.current.lastValue).toBeNull();await settle(()=>socket.deliver({...reply,frame:99}));expect(result.current.lastValue).toBeNull();await act(async()=>{socket.deliver(reply);expect(await value).toBe("2");});expect(result.current.lastValue?.type).toBe("int");
  let stale!:Promise<string|null>;await settle(()=>{stale=result.current.query("quantity");});const old=socket.sent.at(-1) as {request_id:string};await act(async()=>{socket.deliver({...STOPPED,stop:11,thread:2});expect(await stale).toBeNull();});
  await settle(()=>socket.deliver({...reply,request_id:old.request_id}));expect(result.current.lastValue).toBeNull();
  await settle(()=>{value=result.current.query("quantity");});const latest=socket.sent.at(-1) as {request_id:string};await settle(()=>socket.deliver({event:"failed",session:"dbg-0",about:"eval",request_id:old.request_id,message:"old evaluation failed"}));await act(async()=>{socket.deliver({...reply,stop:11,thread:2,request_id:latest.request_id,value:"worker"});expect(await value).toBe("worker");});
 });

 it("does not relabel top-frame locals as another frame and keeps evaluation refusal local",async()=>{
  const {socket,hook}=open(),result=hook.result;await settle(()=>result.current.start());const frames=[...STOPPED.frames,{id:3,name:"worker",line:8,source:"orders.py",in_document:true}];await settle(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:30,thread:2,frames});});
  await settle(()=>result.current.selectFrame(3));expect(result.current.selectedFrame).toBe(3);expect(result.current.variables).toEqual([]);let value!:Promise<string|null>;await settle(()=>{value=result.current.query("worker_value");});const query=socket.sent.at(-1) as {request_id:string;frame:number};expect(query.frame).toBe(3);
  await act(async()=>{socket.deliver({event:"failed",session:"dbg-0",about:"eval",request_id:query.request_id,message:"Unable to find thread for evaluation"});expect(await value).toBeNull();});expect(result.current.status).toBe("paused");expect(result.current.frames).toEqual(frames);expect(result.current.selectedThread).toBe(2);
  await settle(()=>result.current.selectFrame(2));expect(result.current.variables).toEqual(STOPPED.variables);expect(result.current.status).toBe("paused");
 });

 it("admits only current selected-frame locals and keeps failed optional scopes separate from the stack",async()=>{
  const {socket,hook}=open(),result=hook.result,frames=[...STOPPED.frames,{id:3,name:"worker",line:8,source:"orders.py",in_document:true}];await settle(()=>result.current.start());await settle(()=>{socket.deliver({...STARTED,capabilities:{...STARTED.capabilities,frame_locals:true}});socket.deliver({...STOPPED,stop:40,thread:2,frames});});
  await settle(()=>result.current.selectFrame(3));const request=socket.sent.at(-1) as {op:string;request_id:string;stop:number;thread:number;frame:number};expect(request).toMatchObject({op:"state",stop:40,thread:2,frame:3});expect(result.current.frameVariablesPending).toBe(true);expect(result.current.variables).toEqual([]);
  const response={event:"frame_variables",session:"dbg-0",stop:40,thread:2,frame:3,request_id:request.request_id,variables:[{name:"worker_value",value:"worker",type:"str",variables_reference:0}]};await settle(()=>socket.deliver({...response,frame:2}));expect(result.current.variables).toEqual([]);await settle(()=>socket.deliver(response));expect(result.current.variables).toEqual(response.variables);expect(result.current.frameVariablesPending).toBe(false);
  await settle(()=>result.current.selectFrame(2));const top=socket.sent.at(-1) as {request_id:string};await settle(()=>socket.deliver({event:"failed",session:"dbg-0",about:"state",request_id:top.request_id,message:"adapter scopes unavailable"}));expect(result.current.status).toBe("paused");expect(result.current.frames).toEqual(frames);expect(result.current.variables).toEqual(STOPPED.variables);expect(result.current.frameVariablesMessage).toContain("unavailable");
  await settle(()=>result.current.step("over"));await settle(()=>socket.deliver(response));expect(result.current.status).toBe("running");expect(result.current.frameVariablesPending).toBe(false);
 });

 it("keeps captured frame lifetimes independent of locals arrival and rejects retired stack navigation",async()=>{
  const {socket,hook}=open(),result=hook.result;await settle(()=>result.current.start());await settle(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:50,thread:1});});const captured=result.current.captureFrame?.(2)!;expect(captured.current()).toBe(true);
  await settle(()=>socket.deliver({event:"run_output",session:"dbg-0",events:[{category:"stdout",text:"independent output"}]}));expect(captured.current()).toBe(true);
  await settle(()=>socket.deliver({...STOPPED,stop:51,thread:1,frames:STOPPED.frames,variables:STOPPED.variables,stack_error:"stack provider unavailable"}));expect(captured.current()).toBe(false);expect(result.current.frames).toEqual([]);expect(result.current.historicalFrames).toEqual(STOPPED.frames);expect(result.current.stackMessage).toContain("unavailable");const sent=socket.sent.length;await settle(()=>result.current.selectFrame(2));expect(socket.sent).toHaveLength(sent);expect(result.current.captureFrame?.(2)).toBeNull();expect(result.current.status).toBe("paused");
 });

 it("does not regress the entire pause from held older stops and preserves selected caller on same-stop refresh",async()=>{
  const {socket,hook}=open(),result=hook.result,frames=[...STOPPED.frames,{id:3,name:"caller",line:7,source:"orders.py",in_document:true}];await settle(()=>result.current.start());await settle(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:60,thread:1,frames});});await settle(()=>result.current.selectFrame(3));const captured=result.current.captureFrame?.(3)!;
  await settle(()=>socket.deliver({...STOPPED,stop:60,thread:1,frames:frames.map(frame=>({...frame}))}));expect(result.current.selectedFrame).toBe(3);expect(result.current.pausedLine).toBe(7);expect(result.current.variables).toEqual([]);expect(captured.current()).toBe(true);
  const accepted=result.current.frames;await settle(()=>socket.deliver({...STOPPED,stop:59,thread:2,frames:[],variables:[],line:null}));expect(result.current.frames).toBe(accepted);expect(result.current.selectedFrame).toBe(3);expect(result.current.selectedThread).toBe(1);
  await settle(()=>result.current.step("over"));await settle(()=>socket.deliver({...STOPPED,stop:60,thread:1,frames}));expect(result.current.status).toBe("running");expect(captured.current()).toBe(false);
 });
 it("binds launch receipts before sending and cleans only known retired same-file launches",async()=>{
  const {socket,hook}=open(),result=hook.result;await settle(()=>result.current.start());const first=socket.sent.at(-1) as {request_id:string};expect(first.request_id).toBeTruthy();await settle(()=>result.current.stop());await settle(()=>result.current.start());const second=socket.sent.at(-1) as {request_id:string};expect(second.request_id).not.toBe(first.request_id);
  const before=socket.sent.length;await settle(()=>socket.deliver({...STARTED,session:"foreign",request_id:"another caller"}));expect(socket.sent).toHaveLength(before);expect(result.current.status).toBe("starting");
  await settle(()=>socket.deliver({...STARTED,session:"old",request_id:first.request_id}));expect(socket.sent.at(-1)).toMatchObject({op:"stop",session:"old"});expect(result.current.status).toBe("starting");
  await settle(()=>socket.deliver({event:"build",doc:"hick:///orders.md",request_id:first.request_id,events:[{type:"stdout",text:"old build"}]}));expect(result.current.buildOutput).toEqual([]);
  await settle(()=>socket.deliver({...STARTED,session:"new",request_id:second.request_id}));await settle(()=>socket.deliver({...STOPPED,session:"new",stop:1,thread:1}));const frames=result.current.frames;await settle(()=>socket.deliver({...STARTED,session:"new",request_id:second.request_id}));expect(result.current.status).toBe("paused");expect(result.current.frames).toBe(frames);
  await settle(()=>socket.deliver({event:"failed",session:null,about:"start",request_id:first.request_id,message:"old failed start"}));expect(result.current.status).toBe("paused");expect(result.current.message).toBeNull();
  const launches=new DebugLaunches(),old=launches.begin();for(let index=0;index<70;index++)launches.begin();expect(launches.started(old,"old")).toBe("foreign");
 });

 it("retains DAP zero and negative signed-int32 thread/frame identities without sentinel guessing",async()=>{
  const model=new DebugThreads();expect(model.stopped("scope",1,0)).toBe(true);const read=model.read()!;expect(model.accept({event:"threads",session:"scope",stop:1,selected:0,request_id:read.request,threads:[{id:0,name:"main"},{id:-7,name:"worker"}]})).toBe(true);expect(model.select(-7)?.thread).toBe(-7);expect(model.stopped("scope",2,-7)).toBe(true);expect(model.scope?.thread).toBe(-7);expect(debugAdapterId(true)).toBe(false);expect(debugAdapterId(0.5)).toBe(false);expect(debugAdapterId(2147483648)).toBe(false);
  const {socket,hook}=open(),result=hook.result,frames=[{...STOPPED.frames[0],id:0},{...STOPPED.frames[0],id:-3,name:"caller"}];await settle(()=>result.current.start());await settle(()=>{socket.deliver({...STARTED,capabilities:{...STARTED.capabilities,frame_locals:true}});socket.deliver({...STOPPED,stop:70,thread:0,frames});});expect(result.current.selectedFrame).toBe(0);expect(result.current.selectedThread).toBe(0);
  let value!:Promise<string|null>;await settle(()=>{value=result.current.query("quantity");});const expression=socket.sent.at(-1) as {request_id:string};expect(socket.sent.at(-1)).toMatchObject({thread:0,frame:0,stop:70});await act(async()=>{socket.deliver({event:"value",session:"dbg-0",expression:"quantity",value:"zero frame",type:null,reference:0,stop:70,thread:0,frame:0,request_id:expression.request_id});expect(await value).toBe("zero frame");});
  await settle(()=>result.current.selectFrame(-3));expect(socket.sent.at(-1)).toMatchObject({op:"state",frame:-3,thread:0});expect(result.current.captureFrame?.(-3)?.current()).toBe(true);
 });

 it("binds opaque observation scopes after Reading-first delivery and never downgrades to held old evidence",async()=>{
  const {socket,hook}=open(),result=hook.result;await settle(()=>result.current.start());await settle(()=>socket.deliver(STARTED));const reading=(stop:string)=>({schema_version:1,source:{id:"orders",uri:"hick:///orders.md",revision:"captured",bytes:[97]},scope:{session:"dbg-0",run:"run",stop,frame:2,invocation:null},locations:[],annotations:[],unsupported:[]});
  await settle(()=>{socket.deliver({event:"reading",session:"dbg-0",reading:reading("opaque-stop-a")});socket.deliver({...STOPPED,stop:80,thread:0,observation_scope:reading("opaque-stop-a").scope});});expect(result.current.reading?.scope.stop).toBe("opaque-stop-a");
  await settle(()=>socket.deliver({event:"reading",session:"dbg-0",reading:reading("opaque-stop-b")}));expect(result.current.reading?.scope.stop).toBe("opaque-stop-a");await settle(()=>socket.deliver({...STOPPED,stop:81,thread:0,observation_scope:reading("opaque-stop-b").scope}));expect(result.current.reading?.scope.stop).toBe("opaque-stop-b");
  await settle(()=>{socket.deliver({event:"reading",session:"dbg-0",reading:reading("opaque-stop-a")});socket.deliver({...STOPPED,stop:80,thread:0,observation_scope:reading("opaque-stop-a").scope});});expect(result.current.reading?.scope.stop).toBe("opaque-stop-b");expect(result.current.observationScope?.stop).toBe("opaque-stop-b");
 });

});

it('retains source inventory notices from automatic direct startup',async()=>{vi.mocked(withDebugProtocol).mockImplementationOnce(async(_query,effect)=>{await effect({workspace:'fixture-root',notices:['Unreadable input retained as unsupported.']});});const {hook}=open();await settle(()=>hook.result.current.start());expect(hook.result.current.launchNotices).toEqual(['Unreadable input retained as unsupported.']);});

it("preflights direct startup and retires held negotiation before sending",async()=>{
 vi.mocked(withDebugProtocol).mockRejectedValueOnce(new Error("This engine needs protocol 2"));const {socket,hook}=open();await act(async()=>hook.result.current.start());expect(socket.sent).toMatchObject([]);expect(hook.result.current.message).toContain("protocol 2");
 let ready!:()=>void;vi.mocked(withDebugProtocol).mockImplementationOnce((_query,effect)=>new Promise<void>(resolve=>{ready=()=>{void effect({workspace:"fixture-root"});resolve();};}));await settle(()=>hook.result.current.start());await settle(()=>hook.result.current.stop());await act(async()=>ready());expect(socket.sent.some(item=>(item as {op:string}).op==="start")).toBe(false);expect(hook.result.current.status).toBe("idle");
});

it("refuses new startup from a retired source identity while retaining its existing session controls",async()=>{
 const socket=new FakeSocket(),client=new DebugClient(socket.debug()),identity={};let current=false;socket.onDebugFrame(bytes=>client.handleFrame(bytes));
 const hook=renderHook(()=>useDebuggerOver(client,"old.py",[],"new",identity,()=>current));await settle(()=>hook.result.current.start());expect(socket.sent).toHaveLength(0);expect(hook.result.current.message).toContain("no longer owned");
 current=true;const held=hook.result.current;await settle(()=>{held.start();expect(held.isActive?.()).toBe(true);});await settle(()=>{socket.deliver({...STARTED,doc:"hick:///old.py"});socket.deliver(STOPPED);});const frames=hook.result.current.frames,currentCount=socket.sent.length;current=false;
 await settle(()=>hook.result.current.start());expect(socket.sent).toHaveLength(currentCount);expect(hook.result.current.status).toBe("paused");expect(hook.result.current.frames).toBe(frames);await settle(()=>hook.result.current.stop());expect(socket.sent.at(-1)).toMatchObject({op:"stop",session:"dbg-0"});
});
// These controller witnesses use a provider-owned source capability. Legacy
// wire fixtures above retain their prior protocol assumptions until migrated.
describe('guarded protocol3 controller',()=>{
 async function fixture(){
  const moduleName='node:'+'crypto';const {webcrypto}=await import(moduleName);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});
  const socket=new FakeSocket(),identity={},state={text:Array.from({length:100},(_,index)=>`line${index}`).join('\n'),current:true};
  const sources={read:(uri:string)=>{const text=state.text;return {uri,workspace:'guarded-root',namespace:'guarded-root',identity,text,saved:true,current:()=>state.current&&state.text===text};}};
  const client=new DebugClient({...socket.debug(),provider:{protocol:3,workspace:'guarded-root',sources}});socket.onDebugFrame(frame=>client.handleFrame(frame));
  const hook=renderHook(()=>useDebuggerOver(client,'orders.md',[],'new',identity,()=>state.current));
  async function launch(){await act(async()=>{hook.result.current.start();});await waitFor(()=>expect(socket.sent.some(item=>(item as {op:string}).op==='start')).toBe(true));const request=socket.sent.find(item=>(item as {op:string}).op==='start') as {request_id:string;source_hash:string};await settle(()=>socket.deliver({...STARTED,request_id:request.request_id,source_hash:request.source_hash,breakpoints:[]}));await settle(()=>socket.deliver({...STOPPED,stop:1,thread:0,frames:STOPPED.frames.map(frame=>({...frame,source_hash:request.source_hash}))}));return request;}
  return {socket,hook,state,launch};
 }
 it('keeps unconfirmed effects owned and rejects old pause resurrection through resource readdress',async()=>{
  const {socket,hook,state,launch}=await fixture();await launch();expect(hook.result.current.status).toBe('paused');
  await settle(()=>hook.result.current.step('over'));const movement=socket.sent.at(-1) as {request_id:string};expect(movement).toMatchObject({op:'step',stop:1,thread:0,frame:2});
  await settle(()=>socket.deliver({event:'failed',session:'dbg-0',about:'step',request_id:movement.request_id,scope_retired:true,message:'stop observation unavailable'}));expect(hook.result.current.status).toBe('unconfirmed');expect(hook.result.current.isActive?.()).toBe(true);expect(hook.result.current.historicalFrames).toEqual(STOPPED.frames.map(frame=>({...frame,source_hash:expect.any(String)})));
  state.current=false;const count=socket.sent.length;await settle(()=>{hook.result.current.start();hook.result.current.step('continue');hook.result.current.jumpTo(3);});expect(socket.sent).toHaveLength(count);expect(await hook.result.current.query('quantity')).toBeNull();
  await settle(()=>socket.deliver({...STOPPED,stop:1,thread:0}));expect(hook.result.current.status).toBe('unconfirmed');expect(hook.result.current.captureFrame?.(2)).toBeNull();await settle(()=>hook.result.current.stop());expect(socket.sent.at(-1)).toMatchObject({op:'stop',session:'dbg-0'});expect(hook.result.current.isActive?.()).toBe(true);await settle(()=>socket.deliver({event:'ended',session:'dbg-0'}));expect(hook.result.current.isActive?.()).toBe(false);
 });
 it('restores only a correlated pre-effect pause and rejects malformed stop contents before advancing generation',async()=>{
  const {socket,hook,launch}=await fixture();await launch();await settle(()=>hook.result.current.step('over'));const movement=socket.sent.at(-1) as {request_id:string};await settle(()=>socket.deliver({event:'failed',session:'dbg-0',about:'step',request_id:'foreign',scope_retired:false,message:'foreign'}));expect(hook.result.current.status).toBe('running');await settle(()=>socket.deliver({event:'failed',session:'dbg-0',about:'step',request_id:movement.request_id,scope_retired:false,message:'not admitted'}));expect(hook.result.current.status).toBe('paused');expect(hook.result.current.selectedFrame).toBe(2);
  await settle(()=>socket.deliver({...STOPPED,stop:99,thread:0,frames:[{id:true,name:'bad'}]}));expect(hook.result.current.selectedFrame).toBe(2);await settle(()=>socket.deliver({...STOPPED,stop:2,thread:0}));expect(hook.result.current.status).toBe('paused');expect(hook.result.current.captureFrame?.(2)?.current()).toBe(true);
 });
 it('does not let a late recursive reply survive closing the same-frame inspection',async()=>{
  const {socket,hook,launch}=await fixture();await launch();await settle(()=>socket.deliver({...STOPPED,stop:2,thread:0,variables:[{name:'object',value:'reported',type_name:'dict',variables_reference:7}]}));const root=hook.result.current.variableRoots![0];await settle(()=>hook.result.current.expandValue?.(root.id));const request=socket.sent.at(-1) as {request_id:string};expect(request).toMatchObject({op:'children',frame:2,stop:2,thread:0,reference:7});await settle(()=>hook.result.current.closeValues?.());await settle(()=>socket.deliver({event:'children',session:'dbg-0',frame:2,stop:2,thread:0,reference:7,request_id:request.request_id,variables:[{name:'late',value:'not current',variables_reference:0}]}));expect(hook.result.current.valueNode?.(root.id)).toBeNull();expect(hook.result.current.variableRoots?.[0].children).toBeNull();
 });
});

describe('source-test Debug guarded host',()=>{
 async function testHost(){const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});const hash=[...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(fixtureSource)))].map(value=>value.toString(16).padStart(2,'0')).join('');const {sourceTestDebugChoice,prepareSourceTestDebug,sourceTestDebugBinding}=await import('../simple/TestSources');const snapshot={workspace:'fixture-root',request:{path:'orders.md',source:fixtureSource}},facts={version:1,workspace:'fixture-root',workspace_id:'test-workspace',input_path:'orders.md',input_hash:hash,scope:'plain',project_entry:null,fingerprint:'test-facts',source_hashes:{'orders.md':hash},candidates:[{id:'case',name:'TestFamily::test_selected',language:'python',run_scope:'file_filter',name_filter:'TestFamily::test_selected',confidence:'syntax',source:null,target:{path:'test_cases.py',line:3,character:0,hash}}],syntax_complete:true,notices:[],unsupported_languages:[],unclassified_outputs:[],scanned_files:1,unused_sources:[],project_context:null} as import('../api/testSources').SourceTestScan;const choice=sourceTestDebugChoice(snapshot,facts,snapshot,'case','name_filter'),prepared=prepareSourceTestDebug(choice,{version:1,test_debug_protocol:1,available:true,workspace_id:'test-workspace',input_path:'orders.md',input_hash:hash,analysis_fingerprint:'test-facts',candidate_id:'case',candidate_name:choice.candidateName,selection:choice.selection,source_uri:choice.sourceUri,source_hash:hash,language:'python',framework:'pytest',name:'Pytest selected family',notices:[]}),binding=sourceTestDebugBinding(prepared),opened=open();return {...opened,prepared,binding,hash};}
 async function dispatch(host:Awaited<ReturnType<typeof testHost>>,current=()=>true){let admitted!:Promise<import('../simple/TestSources').SourceTestDebugProcess>;await settle(()=>{admitted=host.hook.result.current.startSourceTest!(host.prepared,current);});await waitFor(()=>expect(host.socket.sent.some(item=>(item as {op:string}).op==='start_test')).toBe(true));const request=host.socket.sent.find(item=>(item as {op:string}).op==='start_test') as {request_id:string;breakpoints:unknown[]};return {admitted,request};}
 it('uses distinct start_test without argv/profile/project or hidden entry breakpoints and retains the exact session after close',async()=>{const host=await testHost(),catalogs=vi.mocked(withDebugProtocol).mock.calls.length;await settle(()=>host.hook.result.current.toggleBreakpoint(14));const {admitted,request}=await dispatch(host);expect(vi.mocked(withDebugProtocol).mock.calls.length).toBe(catalogs);expect(request).toMatchObject({op:'start_test',doc:'hick:///orders.md',source_hash:host.hash,breakpoints:[{line:14}],source_test:host.prepared.choice.request});for(const key of ['program','profile','project'])expect(request).not.toHaveProperty(key);await settle(()=>host.socket.deliver({...STARTED,request_id:request.request_id,source_hash:host.hash,source_test:host.binding,breakpoints:[{line:14,state:'bound'}]}));expect((await admitted).session).toBe('dbg-0');await settle(()=>host.hook.result.current.cancelSourceTestStart?.(host.prepared));expect(host.socket.sent.filter(item=>(item as {op:string}).op==='stop')).toEqual([]);expect(host.hook.result.current.sourceTestInspectable?.('dbg-0')).toBe(true);await settle(()=>{host.socket.deliver(STOPPED);host.hook.result.current.stop();});await settle(()=>host.socket.deliver({event:'ended',session:'dbg-0'}));expect(host.hook.result.current.sourceTestProcesses?.()[0]).toMatchObject({phase:'stopped',exitCode:null});expect(host.hook.result.current.sourceTestProcesses?.()[0].choice).not.toHaveProperty('request');const count=host.socket.sent.length;await settle(()=>host.hook.result.current.start());expect(host.socket.sent).toHaveLength(count);await settle(()=>host.hook.result.current.releaseSourceTest?.());expect(host.hook.result.current.sourceTestInspectable?.('dbg-0')).toBe(false);});
 it('stops only a positively owned mismatched test Started and never adopts its partial contributor receipt',async()=>{const host=await testHost(),{admitted,request}=await dispatch(host),rejected=admitted.catch(error=>String(error));await settle(()=>host.socket.deliver({...STARTED,request_id:'foreign',source_hash:host.hash,source_test:host.binding}));expect(host.socket.sent.filter(item=>(item as {op:string}).op==='stop')).toEqual([]);await settle(()=>host.socket.deliver({...STARTED,request_id:request.request_id,source_hash:host.hash,source_test:{...host.binding,checked_sources:{}}}));expect(await rejected).toContain('exact test-debug selection');expect(host.hook.result.current.sourceTestProcesses?.()).toEqual([]);expect(host.hook.result.current.isActive?.()).toBe(true);expect(host.socket.sent.at(-1)).toMatchObject({op:'stop',session:'dbg-0'});await settle(()=>host.socket.deliver({event:'ended',session:'dbg-0'}));expect(host.hook.result.current.isActive?.()).toBe(false);});
 it('retires close-after-dispatch and waits for owned late cleanup without claiming a session or a test verdict',async()=>{const host=await testHost(),{admitted,request}=await dispatch(host),rejected=admitted.catch(error=>String(error));await settle(()=>host.hook.result.current.cancelSourceTestStart?.(host.prepared));expect(await rejected).toContain('after dispatch');expect(host.hook.result.current.status).toBe('unconfirmed');await settle(()=>host.hook.result.current.stop());expect(host.hook.result.current.isActive?.()).toBe(true);await settle(()=>host.socket.deliver({...STARTED,request_id:request.request_id,source_hash:host.hash,source_test:host.binding}));expect(host.socket.sent.at(-1)).toMatchObject({op:'stop',session:'dbg-0'});expect(host.hook.result.current.sourceTestProcesses?.()).toEqual([]);await settle(()=>host.socket.deliver({event:'ended',session:'dbg-0'}));expect(host.hook.result.current.status).toBe('idle');});
 it('displays exact preparation without adopting a session or replaying Start',async()=>{
  const host=await testHost();
  const {request,admitted}=await dispatch(host);
  const receipt={event:'test_preparing',doc:'hick:///orders.md',request_id:request.request_id,
    source_test:host.binding,wait_ms:180_000};
  await settle(()=>{
   host.socket.deliver({...receipt,wait_ms:180_001});
   host.socket.deliver({...receipt,request_id:'foreign'});
   host.socket.deliver({...receipt,session:'fake'});
   host.socket.deliver({...receipt,source_test:{...host.binding,candidate_id:'wrong'}});
  });
  expect(host.hook.result.current.message??'').not.toMatch(/Preparing the saved test/);
  await settle(()=>{host.socket.deliver(receipt);host.socket.deliver(receipt);});
  expect(host.hook.result.current.message).toMatch(/Preparing the saved test/);
  expect(host.hook.result.current.status).toBe('starting');
  expect(host.hook.result.current.sourceTestProcesses?.()).toEqual([]);
  await settle(()=>host.socket.deliver({...STARTED,request_id:request.request_id,
   doc:'hick:///orders.md',source_hash:host.hash,source_test:host.binding}));
  expect((await admitted).session).toBe(STARTED.session);
  expect(host.hook.result.current.message??'').not.toMatch(/Preparing the saved test/);
  expect(host.socket.sent.filter(item=>(item as {op:string}).op==='start_test')).toHaveLength(1);
 });
 it('cancels held preparation before any effect and does not turn a legacy websocket refusal into ordinary Start',async()=>{const host=await testHost();let pending!:Promise<unknown>;await settle(()=>{pending=host.hook.result.current.startSourceTest!(host.prepared,()=>true).catch(error=>String(error));host.hook.result.current.cancelSourceTestStart?.(host.prepared);});expect(await pending).toContain('no Start');expect(host.socket.sent).toEqual([]);const next=await dispatch(host),rejected=next.admitted.catch(error=>String(error));await settle(()=>host.socket.deliver({event:'failed',doc:'hick:///orders.md',about:'start',request_id:next.request.request_id,message:'unknown variant start_test'}));expect(await rejected).toContain('unknown variant');expect(host.socket.sent.every(item=>(item as {op:string}).op!=='start')).toBe(true);});
 it('refuses another test choice while the source owns an active ordinary session',async()=>{const host=await testHost();await settle(()=>host.hook.result.current.start());await settle(()=>{host.socket.deliver(STARTED);host.socket.deliver(STOPPED);});const count=host.socket.sent.length,frames=host.hook.result.current.frames;await expect(host.hook.result.current.startSourceTest!(host.prepared,()=>true)).rejects.toThrow('active debugger');expect(host.socket.sent).toHaveLength(count);expect(host.hook.result.current.frames).toBe(frames);expect(host.hook.result.current.status).toBe('paused');});
});
</storycode:copy>

### Assemble `apps/web/src/debug/useDebugger.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

## Keep adapter-source failures independent of debugger movement

These focused socket-hook cases drive real channel frames through the client and
hook, not a renderer flag. Exact source receipt identity is required; motion
retires its scope while retaining historical text. Missing old-engine support
sends no unknown operation, and a provider source refusal leaves the paused stack
usable. Owned adapter/host integration remains separate acceptance.

<storycode:copy id="debug-adapter-source-hook-tests">
describe("adapter-owned source hook integration",()=>{
 it("reads only the exact current frame reference and retires source authority on motion",async()=>{
  const {socket,hook}=open(),result=hook.result,frames=[{...STOPPED.frames[0],line:null,source:'same-name.py',in_document:false,source_hash:null,source_reference:123,source_line:0}];
  await settle(()=>result.current.start());await settle(()=>{socket.deliver({...STARTED,capabilities:{...STARTED.capabilities,source_content:true}});socket.deliver({...STOPPED,stop:60,thread:1,frames,variables:[]});});
  await settle(()=>result.current.readAdapterSource?.());const request=socket.sent.at(-1) as {op:string;session:string;stop:number;thread:number;frame:number;reference:number;request_id:string};expect(request).toMatchObject({op:'source',session:'dbg-0',stop:60,thread:1,frame:frames[0].id,reference:123});
  const content='<b>literal source</b>\n😀',source_hash=[...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(content)))].map(byte=>byte.toString(16).padStart(2,'0')).join(''),reply={...request,event:'adapter_source',content,source_hash,mime_type:'text/html'};
  await settle(()=>socket.deliver({...reply,reference:456}));expect(result.current.adapterSource?.value).toBeNull();await settle(()=>socket.deliver(reply));expect(result.current.adapterSource?.value?.content).toBe(content);expect(result.current.adapterSource?.current()).toBe(true);expect(result.current.status).toBe('paused');expect(result.current.frames).toEqual(frames);
  await settle(()=>result.current.step('over'));expect(result.current.adapterSource?.current()).toBe(false);await settle(()=>socket.deliver(reply));expect(result.current.status).toBe('running');expect(result.current.adapterSource?.value?.content).toBe(content);hook.unmount();
 });
 it("keeps old-engine support and optional provider failures local to the source reading",async()=>{
  const {socket,hook}=open(),result=hook.result,frames=[{...STOPPED.frames[0],source_reference:123,in_document:false,source:'runtime.py',line:null}];await settle(()=>result.current.start());await settle(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:70,thread:1,frames,variables:[]});});
  const sent=socket.sent.length;await settle(()=>result.current.readAdapterSource?.());expect(socket.sent).toHaveLength(sent);expect(result.current.status).toBe('paused');expect(result.current.message).toContain('does not support');
  await settle(()=>result.current.start());await settle(()=>{socket.deliver({...STARTED,capabilities:{...STARTED.capabilities,source_content:true}});socket.deliver({...STOPPED,stop:71,thread:1,frames,variables:[]});});await settle(()=>result.current.readAdapterSource?.());const request=socket.sent.at(-1) as {request_id:string};await settle(()=>socket.deliver({event:'failed',session:'dbg-0',about:'source',request_id:request.request_id,message:'Adapter source unavailable',scope_retired:false}));expect(result.current.adapterSource?.notice).toContain('unavailable');expect(result.current.status).toBe('paused');expect(result.current.frames).toEqual(frames);hook.unmount();
 });
});
</storycode:copy>

<storycode:file path="apps/web/src/debug/useDebugger.test.ts"><storycode:paste select="#f-101aa3d58173-FakeSocket"/><storycode:paste select="#f-101aa3d58173-STARTED"/><storycode:paste select="#f-101aa3d58173-STOPPED"/><storycode:paste select="#f-101aa3d58173-open"/><storycode:paste select="#f-101aa3d58173-expression_statement"/><storycode:paste select="#f-101aa3d58173-expression_statement-2"/><storycode:paste select="#f-101aa3d58173-expression_statement-3"/><storycode:paste select="#f-101aa3d58173-expression_statement-4"/><storycode:paste select="#f-101aa3d58173-expression_statement-5"/><storycode:paste select="#debug-adapter-source-hook-tests"/></storycode:file>


</storycode:doc>
