import { DebugClient, CHANNEL_DEBUG } from "../client";
import type { DebugEvent } from "../client";
import { loadHickLang, materializeLiteralFiles } from "../../editor/hickLang";
import type { BrowserEvent } from "./runtime";
import workerAsset from "./worker.ts?worker&url";

export interface BrowserDebuggerOptions {
  /** Read only at Start. The worker receives an immutable revision. */
  readDocument: () => { source: string; revision: string };
  onOutput?: (text: string, category: "stdout" | "stderr") => void;
  onRevision?: (revision: string) => void;
}

/** Instance-scoped transport for the existing product debug client. */
export function createBrowserDebugger(options: BrowserDebuggerOptions) {
  let worker: Worker | null = null;
  let session: string | null = null;
  let generation = 0;
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
  const client = new DebugClient({ cancelStart() { terminate(); session = null; }, send(frame) {
    if (disposed) return;
    const request = JSON.parse(new TextDecoder().decode(frame.subarray(1))) as Record<string, unknown>;
    if (request.op === "stop") {
      const previous = session;
      terminate(); session = null;
      if (previous) queueMicrotask(() => emit({ event: "ended", session: previous }));
      return;
    }
    if (request.op !== "start") { worker?.postMessage(request); return; }
    terminate();
    const current = generation;
    const frozen = options.readDocument();
    const previous = session;
    if (previous) emit({ event: "ended", session: previous });
    session = null;
    void Promise.all([loadHickLang(), loadRunner()]).then(([, url]) => {
      if (disposed || current !== generation) return;
      const files = materializeLiteralFiles(frozen.source);
      const file = request.program ? files.find((f) => f.path === request.program) : files.find((f) => /\.(js|ts)$/.test(f.path));
      if (!file) throw new Error("No literal .js or .ts file in this document");
      session = `browser-${crypto.randomUUID()}`;
      options.onRevision?.(frozen.revision);
      let started = false;
      const startRequest = { ...request, session, input: { ...frozen, file } };
      function launch(url: string, fallback: boolean) {
        worker = new Worker(url, import.meta.env.DEV ? { type: "module" } : undefined);
        worker.onmessage = (message: MessageEvent<BrowserEvent>) => {
          if (current !== generation || disposed) return;
          const event = message.data;
          if (event.event === "output") options.onOutput?.(event.text, event.category);
          else {
            if (event.event === "started") started = true;
            // A constructor/compilation failure is a failed start, not a live session.
            if (event.event === "failed" && event.about === "start") session = null;
            emit(event);
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
          emit({ event: "failed", session: null, doc: String(request.doc), message: error.message || "Browser runtime could not load (check the host worker-src policy)" });
          terminate(); session = null;
        };
        worker.postMessage(startRequest);
      }
      launch(url, true);
    }).catch((e: unknown) => {
      if (current === generation && !disposed) emit({ event: "failed", session: null, doc: String(request.doc), about: "start", message: String(e) });
    });
  } });
  return { client,
    pause() { if (session) worker?.postMessage({ op: "pause", session }); },
    dispose() { disposed = true; terminate(); session = null; if (runnerUrl) URL.revokeObjectURL(runnerUrl); runnerUrl = null; runnerBytes = null; dataRunner = null; },
  };
}
