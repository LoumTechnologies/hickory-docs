import type { RuntimeRun } from "../runtimeObservations";
import { DebugClient, CHANNEL_DEBUG } from "../client";
import type { DebugEvent } from "../client";
import { loadHickLang, materializeLiteralFiles } from "../../editor/hickLang";
import {debugUTF8,DebugSourceSignal} from "../SourceGuards";
import type { BrowserEvent } from "./runtime";
import workerAsset from "./worker.ts?worker&url";

export interface BrowserDebuggerOptions {
  /** Read the current authored model without side effects; Start captures an immutable revision. */
  readDocument: () => { uri:string;source: string; revision: string };
  onCapture?: (source:{source:string;revision:string})=>void;
  onOutput?: (text: string, category: "stdout" | "stderr") => void;
  onRevision?: (revision: string) => void;
  onRuntime?: (run: RuntimeRun) => void;
}

/** Instance-scoped transport for the existing product debug client. */
export function createBrowserDebugger(options: BrowserDebuggerOptions) {
  const workspace=`browser-provider:${crypto.randomUUID()}`,identity={},sourceSignal=new DebugSourceSignal();
  const sources={subscribe:sourceSignal.subscribe,read:(uri:string)=>{const held=options.readDocument();if(uri!==held.uri)throw Error("Browser source address no longer belongs to this document");return {uri,workspace,namespace:workspace,identity,text:held.source,saved:true,current:()=>{const latest=options.readDocument();return !disposed&&latest.uri===held.uri&&latest.source===held.source&&latest.revision===held.revision;}};}};
  let worker: Worker | null = null;
  let session: string | null = null;
  let generation = 0;
  let runtime: RuntimeRun | null = null;
  let disposed = false;
  // Keep the runner bytes in this instance, so terminating a worker on Stop
  // never makes the next run depend on another network request.
  let runnerUrl: string | null = null;
  let runnerLoading: Promise<string> | null = null;
  let runnerBytes: Uint8Array | null = null;
  let dataRunner: string | null = null;
  function loadRunner(): Promise<string> {
    if (runnerUrl) return Promise.resolve(runnerUrl);
    if (import.meta.env.DEV) return Promise.resolve(new URL(workerAsset, location.href).href);
    runnerLoading ??= fetch(workerAsset).then(async (response) => {
      if (!response.ok) throw new Error(`Browser runtime download failed: ${response.status}`);
      const bytes = new Uint8Array(await response.arrayBuffer());
      if (disposed) throw new Error("Browser debugger disposed");
      runnerBytes = bytes;
      runnerUrl = URL.createObjectURL(new Blob([bytes], { type: "text/javascript" }));
      return runnerUrl;
    }).catch((error) => { runnerLoading = null; throw error; });
    return runnerLoading;
  }
  function emit(event: DebugEvent) {
    const body = new TextEncoder().encode(JSON.stringify(event));
    const frame = new Uint8Array(body.length + 1);
    frame[0] = CHANNEL_DEBUG; frame.set(body, 1); client.handleFrame(frame);
  }
  function terminate() { worker?.terminate(); worker = null; generation++; }
  const client = new DebugClient({ provider:{protocol:3,workspace,sources},cancelStart() { terminate(); session = null; }, send(frame) {
    if (disposed) return;
    const request = JSON.parse(new TextDecoder().decode(frame.subarray(1))) as Record<string, unknown>;
    if (request.op === "stop") {
      const previous = session;
      if (runtime?.status === "running") { runtime = { ...runtime, status: "cancelled" }; options.onRuntime?.(runtime); }
      terminate(); session = null;
      if (previous) queueMicrotask(() => emit({ event: "ended", session: previous }));
      return;
    }
    if (request.op !== "start") { worker?.postMessage(request); return; }
    if (runtime?.status === "running") { runtime = { ...runtime, status: "cancelled" }; options.onRuntime?.(runtime); }
    terminate();
    const current = generation;
    const frozen = options.readDocument();
    const previous = session;
    if (previous) emit({ event: "ended", session: previous });
    session = null;
    void Promise.all([loadHickLang(), loadRunner(), crypto.subtle.digest("SHA-256", debugUTF8(frozen.source))]).then(([, url, digest]) => {
      if (disposed || current !== generation) return;
      const sourceHash=[...new Uint8Array(digest)].map(b=>b.toString(16).padStart(2,"0")).join("");
      if(request.doc!==frozen.uri)throw Error("Browser document source address changed before capture.");
      if(request.source_hash!==sourceHash)throw Error("Browser authored source changed before capture.");
      if(Object.keys((request.source_hashes??{}) as object).length)throw Error("Browser interpreter cannot prove additional authored source mappings.");
      options.onCapture?.(frozen);
      const files = materializeLiteralFiles(frozen.source);
      const file = request.program ? files.find((f) => f.path === request.program) : files.find((f) => /\.(js|ts)$/.test(f.path));
      if (!file) throw new Error("No literal .js or .ts file in this document");
      session = `browser-${crypto.randomUUID()}`;
      options.onRevision?.(frozen.revision);
      let started = false;
      const startRequest = { ...request, session, input: { ...frozen, file, sourceHash, observationRevision: [...new Uint8Array(digest)].map(b => b.toString(16).padStart(2, "0")).join("") } };
      function launch(url: string, fallback: boolean) {
        worker = new Worker(url, import.meta.env.DEV ? { type: "module" } : undefined);
        worker.onmessage = (message: MessageEvent<BrowserEvent>) => {
          if (current !== generation || disposed) return;
          const event = message.data;
          if (event.event === "runtime") { runtime = event.run; options.onRuntime?.(event.run); }
          else if (event.event === "runtime_event") {
            if (runtime?.id === event.session) {
              runtime = { ...runtime, events: [...runtime.events, event.observation] };
              options.onRuntime?.(runtime);
            }
          }
          else if (event.event === "output") options.onOutput?.(event.text, event.category);
          else {
            if (event.event === "started") started = true;
            // A constructor/compilation failure is a failed start, not a live session.
            if (event.event === "failed" && event.about === "start") session = null;
            emit((event.event==="started"||event.event==="build"||event.event==="failed"&&event.about==="start")&&typeof request.request_id==="string"?{...event,request_id:request.request_id}:event);
            if (event.event === "finished" || event.event === "failed" && event.about === "start") { worker?.terminate(); worker = null; session = null; }
          }
        };
        worker.onerror = (error) => {
          if (current !== generation || disposed) return;
          error.preventDefault();
          // WebKit's offline network switch also rejects fresh blob workers.
          // A data runner has no resource lookup. The large data runner failed
          // in our Chromium probe, so prefer blob and fall back on load failure.
          if (!started && fallback && runnerBytes) {
            worker?.terminate();
            if (!dataRunner) {
              let binary = "";
              for (let at = 0; at < runnerBytes.length; at += 32768) binary += String.fromCharCode(...runnerBytes.subarray(at, at + 32768));
              dataRunner = "data:text/javascript;base64," + btoa(binary);
            }
            launch(dataRunner, false);
            return;
          }
          if(started&&session){const failed=session;emit({event:'failed',session:failed,message:error.message||'Browser worker stopped unexpectedly'});emit({event:'finished',session:failed,exit_code:1});terminate();session=null;return;}
          emit({ event: "failed", session: null, doc: String(request.doc),request_id:typeof request.request_id==="string"?request.request_id:undefined,about:"start", message: error.message || "Browser runtime could not load (check the host worker-src policy)" });
          terminate(); session = null;
        };
        worker.postMessage(startRequest);
      }
      launch(url, true);
    }).catch((e: unknown) => {
      if (current === generation && !disposed) emit({ event: "failed", session: null, doc: String(request.doc), about: "start",request_id:typeof request.request_id==="string"?request.request_id:undefined, message: String(e) });
    });
  } });
  return { client,sourceChanged:()=>sourceSignal.changed(),
    pause() { if (session) worker?.postMessage({ op: "pause", session }); },
    dispose() { if (runtime?.status === "running") options.onRuntime?.({ ...runtime, status: "cancelled" }); disposed = true; terminate(); session = null; if (runnerUrl) URL.revokeObjectURL(runnerUrl); runnerUrl = null; runnerBytes = null; dataRunner = null; },
  };
}

/** Actual worker asset bytes only; development imports/browser platform are not a complete environment closure. */
export async function browserRuntimeAssetHash(): Promise<string> {
  const response = await fetch(workerAsset);
  if (!response.ok) throw Error("Browser runtime identity unavailable");
  return [...new Uint8Array(await crypto.subtle.digest("SHA-256", await response.arrayBuffer()))].map(byte => byte.toString(16).padStart(2, "0")).join("");
}
