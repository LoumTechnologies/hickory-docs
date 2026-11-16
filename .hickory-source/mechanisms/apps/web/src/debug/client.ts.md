<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/client.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9c2c3f79dace"></a>
## `apps/web/src/debug/client.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Breakpoint Can Carry A Condition](../../../../../guarantees/debugging/a-breakpoint-can-carry-a-condition/contract.md), [An Exception Breakpoint Uses The Adapter's Own Filters](../../../../../guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters/contract.md).

<a id="f-9c2c3f79dace-CHANNEL_DEBUG"></a>
### `CHANNEL_DEBUG`

<storycode:copy id="f-9c2c3f79dace-CHANNEL_DEBUG">
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

</storycode:copy>
<a id="f-9c2c3f79dace-DebugBreakpoint"></a>
### `DebugBreakpoint`

<storycode:copy id="f-9c2c3f79dace-DebugBreakpoint">
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

</storycode:copy>
<a id="f-9c2c3f79dace-BreakpointStatus"></a>
### `BreakpointStatus`

<storycode:copy id="f-9c2c3f79dace-BreakpointStatus">
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

</storycode:copy>
<a id="f-9c2c3f79dace-Frame"></a>
### `Frame`

<storycode:copy id="f-9c2c3f79dace-Frame">
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
  /** Hash of authored bytes captured by the launch mapping; not proof of the executed runtime revision. */
  source_hash?:string|null;
  source_reference?:number|null;
  /** 0-based line in `source`, as the adapter reported it — for opening a
   * frame that is in another file of the project. */
  source_line?: number | null;
  in_document: boolean;
}

</storycode:copy>
<a id="f-9c2c3f79dace-Variable"></a>
### `Variable`

<storycode:copy id="f-9c2c3f79dace-Variable">
export interface Variable {
  name: string;
  value: string;
  /** Shared engine DAP variable wire name. */
  type_name?: string | null;
  /** Retained browser interpreter/legacy variable spelling. */
  type?: string | null;
  variables_reference: number;
}

</storycode:copy>
<a id="f-9c2c3f79dace-DebugCapabilities"></a>
### `DebugCapabilities`

What this adapter can do. Every control is gated on one of these.

<storycode:copy id="f-9c2c3f79dace-DebugCapabilities">
/** What this adapter can do. Every control is gated on one of these. */
export interface DebugCapabilities {
  /** Transport can return another frame's locals through state(frame). */
  frame_locals?: boolean;
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
  /** Engine-composed temporary-breakpoint movement. */
  run_to?:boolean;
  exception_filters: { id: string; label: string }[];
}

</storycode:copy>
<a id="f-9c2c3f79dace-BindState"></a>
### `BindState`

`bound` — the adapter confirmed it; the program will stop here.
`pending` — not confirmed yet. Normal for a compiled language, where
nothing binds until the module loads.
`refused` — there is no code on this line at all. The only certain
refusal, and hick makes it itself before any adapter is asked.

<storycode:copy id="f-9c2c3f79dace-BindState">
/**
 * `bound` — the adapter confirmed it; the program will stop here.
 * `pending` — not confirmed yet. Normal for a compiled language, where
 *   nothing binds until the module loads.
 * `refused` — there is no code on this line at all. The only certain
 *   refusal, and hick makes it itself before any adapter is asked.
 */
export type BindState = "bound" | "pending" | "refused";

</storycode:copy>
<a id="f-9c2c3f79dace-DebugEvent"></a>
### `DebugEvent`

<storycode:copy id="f-9c2c3f79dace-DebugEvent">
export type DebugEvent =
  | DebugThreadEvent
  | {event:"frame_variables";session:string;stop:number;thread:number;frame:number;request_id?:string|null;variables:Variable[]}
  | { event: "run_output"; session: string; events: { category: string; text: string; source?: string | null; line?: number | null }[] }
  | { event: "reading"; session: string; reading: import("./observations").ObservationReading }
  | {
      event: "started";
      source_hash?:string|null;
      request_id?:string|null;
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
      stop?:number;
      thread?:number;
      stack_error?:string|null;
      variables_error?:string|null;
      observation_scope?: import("./observations").ObservationScope | null;
      session: string;
      reason: string;
      frames: Frame[];
      variables: Variable[];
      line: number | null;
      selected_frame?: number;
    }
  | { event: "breakpoints"; session: string; breakpoints: BreakpointStatus[];source_hash?:string|null;request_id?:string|null }
  | { event: "source_breakpoints"; session: string; source: string; breakpoints: BreakpointStatus[];source_hash?:string|null;request_id?:string|null }
  | { event: "value"; session: string; expression: string; value: string; type_name?: string | null; type?: string | null; reference: number;stop?:number;thread?:number|null;frame?:number|null;request_id?:string|null }
  | { event: "children"; session: string; reference: number; variables: Variable[];stop?:number;thread?:number|null;frame?:number|null;request_id?:string|null }
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
  | { event: "build"; doc?: string;request_id?:string|null; events: TranscriptEvent[] }
  | {
      event: "failed";
      scope_retired?:boolean;
      session: string | null;
      /** The file a failed START was for, when the failure has no session
       * to name instead. */
      doc?: string;
      message: string;
      /** Which request failed, so its message can be shown where it belongs. */
      about?: string;
      request_id?:string|null;
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

</storycode:copy>
<a id="f-9c2c3f79dace-Step"></a>
### `Step`

The server's own verbs, spelled exactly as it deserializes them.
`back` is true reverse execution (almost nothing has it) and `drop_frame`
re-enters the current function. `jump` is not here because it takes a line
and is its own request rather than a step.

<storycode:copy id="f-9c2c3f79dace-Step">
/**
 * The server's own verbs, spelled exactly as it deserializes them.
 *
 * `back` is true reverse execution (almost nothing has it) and `drop_frame`
 * re-enters the current function. `jump` is not here because it takes a line
 * and is its own request rather than a step.
 */
export type Step = "over" | "in" | "out" | "continue" | "drop_frame" | "back";

</storycode:copy>
<a id="f-9c2c3f79dace-DebugWire"></a>
### `DebugWire`

The raw-frame transport, shared with the LSP channel.

<storycode:copy id="f-9c2c3f79dace-DebugWire">
/** The raw-frame transport, shared with the LSP channel. */
export interface DebugWire {
  send(frame: Uint8Array): void;
  cancelStart?: (doc: string) => void;
  /** Independent providers negotiate their own guarded protocol and source authority. */
  provider?: {protocol:number;workspace:string;sources:import("./SourceGuards").DebugSourceCapability};
}

</storycode:copy>
<a id="f-9c2c3f79dace-DebugClient"></a>
### `DebugClient`

<storycode:copy id="f-9c2c3f79dace-DebugClient">
export class DebugClient {
  private listeners = new Set<(event: DebugEvent) => void>();

  constructor(private wire: DebugWire) {}
  get provider(){return this.wire.provider;}

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

  start(doc: string, breakpoints: DebugBreakpoint[], program?: string, project?: string, sources?: Record<string, DebugBreakpoint[]>, profile?: import("./launchProfiles").LaunchSelection,request_id?:string,source_hash?:string,source_hashes?:Record<string,string>) {
    // `program` names which generated file to run. A document with two
    // Python files has two answers, and picking the first is right only by
    // accident.
    this.send({ op: "start", doc, breakpoints, program,source_hash,source_hashes,...(request_id?{request_id}:{}), ...(profile ? {profile:{id:profile.id,fingerprint:profile.fingerprint}} : {}), ...(project && project !== "new" ? {project} : {}), ...(sources && Object.keys(sources).length ? {sources} : {}) });
  }

  located(session: string, location: string, scope: import("./observations").ObservationScope, how: "jump" | "run_to") {
    this.send({ op: "located", session, location, scope, how });
  }

  setBreakpoints(session: string, breakpoints: DebugBreakpoint[],source_hash?:string,request_id?:string) {
    this.send({ op: "breakpoints", session, breakpoints,source_hash,request_id });
  }

  setSourceBreakpoints(session: string, source: string, breakpoints: DebugBreakpoint[],source_hash?:string,request_id?:string) {
    this.send({ op: "source_breakpoints", session, source, breakpoints,source_hash,request_id });
  }

  threads(session:string,request_id:string){this.send({op:"threads",session,request_id});}

  selectThread(session:string,thread:number,stop:number,request_id:string){this.send({op:"select_thread",session,thread,stop,request_id});}

  state(session: string, frame?: number,scope?:{stop?:number;thread?:number;request_id:string}) {
    this.send({ op: "state", session, ...(frame === undefined ? {} : { frame }),...scope });
  }

  cancelStart(doc: string) { this.wire.cancelStart?.(doc); }

  /**
   * Evaluate an expression.
   *
   * `context` is DAP's own and adapters treat it differently: `hover` for a
   * tooltip, `watch` for something evaluated repeatedly, `repl` for what a
   * person typed — which is the only one where a side effect is their
   * business.
   */
  evaluate(session: string, expression: string, context: "hover" | "watch" | "repl" = "repl", frame?: number, scope?:{stop:number;thread:number;request_id:string}) {
    this.send({ op: "eval", session, expression, context, frame,...scope });
  }

  step(session: string, how: Step, frame?: number,scope?:{stop:number;thread:number;request_id:string}) {
    this.send({ op: "step", session, how, frame,...scope });
  }

  /** Move the instruction pointer to a line in the current frame. */
  jump(session: string, line: number,scope?:{stop:number;thread:number;request_id:string;source_hash:string}) {
    this.send({ op: "jump", session, line,...scope });
  }

  runTo(session: string, line: number,scope?:{stop:number;thread:number;request_id:string;source_hash:string}) {
    this.send({ op: "run_to", session, line,...scope });
  }

  children(session: string, reference: number,scope?:{stop:number;thread:number;frame:number;request_id:string}) {
    this.send({ op: "children", session, reference,...scope });
  }

  /**
   * Stop when an exception is thrown, by the adapter's OWN filter ids.
   *
   * The ids come from `capabilities.exception_filters` and are never
   * invented here: "raised" and "uncaught" are Python's words, and another
   * adapter's are its own.
   */
  setExceptionBreakpoints(session: string, filters: string[]) {
    this.send({ op: "exception_breakpoints", session, filters });
  }

  setVariable(session: string, container: number, name: string, value: string) {
    this.send({ op: "set_variable", session, container, name, value });
  }

  stop(session: string) {
    this.send({ op: "stop", session });
  }
}

</storycode:copy>
<a id="f-9c2c3f79dace-backwardsControl"></a>
### `backwardsControl`

Which backwards control this adapter can offer, if any.
Two exist and most adapters have exactly one, so the UI asks this rather
than showing both and letting one fail. debugpy has `jump`; js-debug and
the JVM adapters have `drop frame`.

<storycode:copy id="f-9c2c3f79dace-backwardsControl">
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
</storycode:copy>

## Admit one launch identity before accepting its session

A delayed launch can answer after Stop and a same-file relaunch. Give every caller launch an unpredictable identity before sending it. Accept startup/build/failure only for that current identity; remember a bounded set of retired own launches solely to stop their late sessions. An unknown caller identity never authorizes stopping a neighbour's session. A repeated accepted session receipt cannot reset an already paused inspector.

<storycode:copy id="debug-launch-observations-browser">
export class DebugLaunches {
 current:string|null=null;private accepted:string|null=null;private retired=new Set<string>();
 begin(){if(this.current)this.remember(this.current);this.current=crypto.randomUUID();this.accepted=null;return this.current;}
 retire(){if(this.current)this.remember(this.current);this.current=null;this.accepted=null;}
 private remember(id:string){this.retired.add(id);while(this.retired.size>64)this.retired.delete(this.retired.values().next().value!);}
 matches(id:string|null|undefined){return !!id&&id===this.current;}
 started(id:string|null|undefined,session:string):'current'|'duplicate'|'retired'|'foreign'{
  if(this.matches(id)){if(this.accepted===session)return 'duplicate';if(this.accepted)return 'retired';this.accepted=session;return 'current';}
  return id&&this.retired.has(id)?'retired':'foreign';
 }
}
</storycode:copy>

## Bind reported threads to a stopped observation

An adapter thread is an observation, not a declaration that every thread is paused. Keep one workspace-session stop scope and only admit the latest explicitly requested list for that scope. Selection sends the captured stop token and retains the old stack until the engine confirms another stopped state. A failed optional read cannot terminate the session or erase frames. The backend may advance the stop after a successful selection; retain only that matching selection request for the immediately following list.

<storycode:copy id="debug-thread-observations-browser">
export function debugAdapterId(value:unknown):value is number{return typeof value==="number"&&Number.isInteger(value)&&value>=-2147483648&&value<=2147483647;}
export interface DebugThread {id:number;name:string}
export interface DebugThreadEvent {event:'threads';session:string;stop:number;selected:number;threads:DebugThread[];request_id?:string|null}
interface DebugThreadScope {session:string;stop:number;thread:number}
/** An adapter list is scoped to an observed stop and the latest request, never a global thread catalog. */
export class DebugThreads {
 scope:DebugThreadScope|null=null;reading:DebugThreadEvent|null=null;message:string|null=null;busy=false;
 get selecting(){return this.busy&&this.pending?.target!==undefined;}
 private latest:DebugThreadScope|null=null;
 private sequence=0;private pending:{request:string;session:string;stop:number;target?:number}|null=null;
 stopped(session:string,stop:unknown,thread:unknown):boolean{
  if(!session||!Number.isSafeInteger(stop)||(stop as number)<0||!debugAdapterId(thread)){if(this.latest?.session===session)return false;this.clear();this.message='Thread inspection needs a current stop identity from the engine.';return true;}
  const next={session,stop:stop as number,thread:thread as number};if(this.latest?.session===session&&(next.stop<this.latest.stop||next.stop===this.latest.stop&&(!this.scope||next.thread!==this.latest.thread)))return false;
  this.latest=next;const same=this.scope?.session===session&&this.scope.stop===stop&&this.scope.thread===thread;
  if(same)return true;
  const selected=this.pending?.session===session&&this.pending.target===thread&&next.stop>this.pending.stop;
  this.scope=next;this.reading=null;this.message=null;if(!selected){this.pending=null;this.busy=false;}return true;
 }
 restore(scope:{session:string;stop:number;thread:number}):boolean {if(!this.latest||this.latest.session!==scope.session||this.latest.stop!==scope.stop||this.latest.thread!==scope.thread)return false;this.scope={...scope};this.reading=null;this.pending=null;this.busy=false;this.message=null;return true;}
 reset(){this.latest=null;this.clear();}
 clear(){this.scope=null;this.reading=null;this.pending=null;this.busy=false;this.message=null;}
 read():{session:string;request:string}|null {if(this.selecting){this.message='Wait for the selected thread stack before another thread read.';return null;}if(!this.scope){this.message='Pause the debugger before reading threads.';return null;}const request='threads-'+(++this.sequence);this.pending={request,session:this.scope.session,stop:this.scope.stop};this.busy=true;this.message=null;return {session:this.scope.session,request};}
 select(thread:number):{session:string;thread:number;stop:number;request:string}|null {
  const scope=this.scope,reading=this.reading;if(!scope||!reading||this.busy||reading.session!==scope.session||reading.stop!==scope.stop||!reading.threads.some(item=>item.id===thread)){this.message='Read the current stopped thread list before selecting a thread.';return null;}
  const request='threads-'+(++this.sequence);this.pending={request,session:scope.session,stop:scope.stop,target:thread};this.busy=true;this.message=null;return {session:scope.session,thread,stop:scope.stop,request};
 }
 accept(event:DebugThreadEvent):boolean {
  const pending=this.pending,scope=this.scope;if(!pending||!scope||event.request_id!==pending.request||event.session!==pending.session||event.session!==scope.session||event.stop!==scope.stop)return false;
  if(!Array.isArray(event.threads)||event.threads.length>4096||event.threads.some(item=>!item||typeof item!=="object"||!debugAdapterId(item.id)||typeof item.name!=='string'||item.name.length>4096)||new Set(event.threads.map(item=>item.id)).size!==event.threads.length||event.selected!==scope.thread||!event.threads.some(item=>item.id===event.selected)||(pending.target!==undefined&&event.selected!==pending.target)){
   this.busy=false;this.message='The adapter thread list does not match the current stopped observation. Read threads again.';return false;
  }
  this.reading={...event,threads:event.threads.map(item=>({id:item.id,name:item.name}))};this.pending=null;this.busy=false;this.message=null;return true;
 }
 failed(message:string,request?:string|null):boolean{if(typeof request!=='string'||this.pending?.request!==request)return false;this.pending=null;this.busy=false;this.message=message;return true;}
}
</storycode:copy>

### Assemble `apps/web/src/debug/client.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/client.ts"><storycode:paste select="#f-9c2c3f79dace-CHANNEL_DEBUG"/><storycode:paste select="#f-9c2c3f79dace-DebugBreakpoint"/><storycode:paste select="#f-9c2c3f79dace-BreakpointStatus"/><storycode:paste select="#f-9c2c3f79dace-Frame"/><storycode:paste select="#f-9c2c3f79dace-Variable"/><storycode:paste select="#f-9c2c3f79dace-DebugCapabilities"/><storycode:paste select="#f-9c2c3f79dace-BindState"/><storycode:paste select="#f-9c2c3f79dace-DebugEvent"/><storycode:paste select="#debug-launch-observations-browser"/><storycode:paste select="#debug-thread-observations-browser"/><storycode:paste select="#f-9c2c3f79dace-Step"/><storycode:paste select="#f-9c2c3f79dace-DebugWire"/><storycode:paste select="#f-9c2c3f79dace-DebugClient"/><storycode:paste select="#f-9c2c3f79dace-backwardsControl"/></storycode:file>

</storycode:doc>
