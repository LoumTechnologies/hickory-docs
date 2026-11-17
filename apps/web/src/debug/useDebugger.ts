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

/** One watched expression, and what it held the last time anyone asked. */
export interface Watch {
  expression: string;
  /** Null until the first answer arrives, and again after the program ends. */
  value: string | null;
}

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

/** Whether a breakpoint carries anything beyond "stop here". */
export function isConditional(breakpoint: HeldBreakpoint): boolean {
  return Boolean(breakpoint.condition || breakpoint.hit_condition || breakpoint.log_message);
}

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

// The workspace's one debug client, shared by every plain file.
//
// One, not one per pane, for the same reason the language client is one:
// the socket hands inbound frames to a single handler, and two clients on
// one connection would each take the other's answers. Each pane's hook
// filters the shared stream down to its own session (`eventIsOurs`).
let workspaceClient: DebugClient | null = null;
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

/** Test seam: forget the workspace debug client. */
export function resetWorkspaceDebugger(): void {
  workspaceClient = null;
}

/**
 * A debugger for a plain file — `src/main.rs`, `app.py` — over the
 * workspace connection. The same verbs, the same events, at the file's
 * own path; the server debugs a file that is not a document as itself.
 */
export function useWorkspaceDebugger(path: string,uiSource?:unknown,launchCurrent?:()=>boolean,sourceCapability?:DebugSourceCapability): DebugSession {
  const client = useMemo(() => workspaceDebugClient(), []);
  return useDebuggerOver(client, path,[],"new",uiSource,launchCurrent,sourceCapability);
}

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
  type TestPending={prepared:PreparedSourceTestDebug;current:()=>boolean;request:string|null;sent:boolean;resolve:(record:SourceTestDebugProcess)=>void;reject:(error:Error)=>void};
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
          const test=pendingTest.current;if(test&&test.request===event.request_id){const record:SourceTestDebugProcess={session:event.session,choice:retainedSourceTestDebugChoice(test.prepared.choice),binding:sourceTestDebugBinding(test.prepared),phase:'running',exitCode:null,message:null};testHistory.current.set(event.session,record);while(testHistory.current.size>32)testHistory.current.delete(testHistory.current.keys().next().value!);testContext.current=event.session;pendingTest.current=null;test.resolve(record);paintTest(value=>value+1);}
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
    try{if(client?.provider)throw Error('This browser-local provider does not implement source-test Debug admission.');prepareSourceTestDebug(prepared.choice,prepared.plan);if(uri!==prepared.choice.sourceUri)throw Error('This debugger belongs to a different authored input.');start(undefined,{},undefined,undefined,{}, {prepared,current,request:null,sent:false,resolve,reject});}catch(error){reject(error instanceof Error?error:Error(String(error)));}
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
