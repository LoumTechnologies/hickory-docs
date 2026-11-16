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

/** One watched expression, and what it held the last time anyone asked. */
export interface Watch {
  expression: string;
  /** Null until the first answer arrives, and again after the program ends. */
  value: string | null;
}

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
export function useWorkspaceDebugger(path: string): DebugSession {
  const client = useMemo(() => workspaceDebugClient(), []);
  return useDebuggerOver(client, path);
}

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
