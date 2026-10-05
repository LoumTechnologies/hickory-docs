import { BrowserRun } from "./runtime";
import type { BrowserInput } from "./runtime";
import type { DebugBreakpoint, Step } from "../client";

let run: BrowserRun | null = null;
let timer: ReturnType<typeof setTimeout> | null = null;
function schedule() {
  if (timer !== null) return;
  timer = setTimeout(() => { timer = null; if (run?.pump()) schedule(); }, 0);
}
globalThis.onmessage = (event: MessageEvent<Record<string, unknown> & { input: BrowserInput }>) => {
  const request = event.data;
  try {
    if (request.op === "start") {
      run?.stop();
      run = new BrowserRun(request.input, String(request.session), String(request.doc), request.breakpoints as DebugBreakpoint[], (e) => globalThis.postMessage(e));
      schedule();
      return;
    }
    if (!run || request.session !== run.session) throw new Error("Unknown browser session");
    switch (request.op) {
      case "step": run.step(request.how as Step); schedule(); break;
      case "pause": run.pause(); schedule(); break;
      case "state": run.report("frame", request.frame as number | undefined); break;
      case "breakpoints": run.setBreakpoints(request.breakpoints as DebugBreakpoint[]); break;
      case "eval": run.evaluate(String(request.expression), request.frame as number | undefined); break;
      case "children": run.children(Number(request.reference)); break;
      default: throw new Error(`Unsupported browser debug action: ${request.op}`);
    }
  } catch (e) {
    globalThis.postMessage({ event: "failed", session: run?.session ?? null, doc: request.doc, about: request.op,
      message: e instanceof Error ? e.message : String(e) });
  }
};
