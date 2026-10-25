<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/tour/browserPrediction.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Browser-local prediction practice uses the shared gate](../../../../../contract.md)

Captured SHA-256: `99ede00a7604cdcfeffdcd06081c04c9b88d9333b5e529c672364b3867964a33`.

<book:entry path="apps/web/src/tour/browserPrediction.ts">
// Host-private adapter. Never return a portable snapshot, output or raw runtime
// diagnostics to ui_act/ui_observe or the ordinary browser runtime history.
import { browserPredictionTransition, loadHickLang, hickLangArtifactHash } from "../editor/hickLang";
import { browserRuntimeAssetHash, createBrowserDebugger } from "../debug/browser/transport";
import type { RuntimeRun } from "../debug/runtimeObservations";
import type { UiActor } from "../lib/uiActions";
import { scenarioSource } from "./scenario";
export const BROWSER_PREDICTION_KEY = "hickory.browserPrediction.v1";
export type BrowserInputs = { attempts: number[]; budget: number };
export type BrowserIdentity = { source: string; inputs: string; dependencies: Record<string, string>; environment: Record<string, string> };
export type BrowserSeal = { receipt: { id: string; generation: number }; scenario: string; identity: BrowserIdentity; actor: UiActor; source: string };
export type BrowserSummary = { attempt: string; classification: string; prediction_matched: boolean | null; original_requirement_satisfied: boolean | null; current_certification: boolean; original_expectation_unchanged?: boolean; independent_checks_accepted?: boolean };
export async function hash(text: string) { return [...new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text)))].map(value => value.toString(16).padStart(2, "0")).join(""); }
export function parseBrowserInputs(attempts: string, budget: string): BrowserInputs {
  const values: unknown = JSON.parse(attempts), limit = Number(budget);
  if (!Array.isArray(values) || values.length > 8 || values.some(value => typeof value !== "number" || !Number.isInteger(value) || value < 0 || value > 1000) || !budget.trim() || !Number.isInteger(limit) || limit <= 0 || limit > 1000) throw Error("Use up to eight whole-millisecond durations from 0 to 1000ms and a whole-millisecond budget from 1 to 1000ms");
  return { attempts: values, budget: limit };
}
/** Restore question inputs only, never a foreign prediction/rationale or actual. */
export function savedBrowserPredictionInputs(): BrowserInputs {
  try {
    const value = JSON.parse(localStorage.getItem("hickory.browserPredictionInputs.v1") ?? "null");
    if (!value || typeof value !== "object" || Object.keys(value).some(key => !["attempts", "budget"].includes(key)) || typeof value.budget !== "number") throw Error("Invalid cached inputs");
    return parseBrowserInputs(JSON.stringify(value.attempts), String(value.budget));
  } catch { return { attempts: [80, 80], budget: 100 }; }
}
export function browserPredictionSource(corrected: boolean, inputs: BrowserInputs) {
  const example = scenarioSource(corrected);
  const implementation = example.slice(example.indexOf("function elapsed"), example.indexOf("var actual"));
  return `# Browser-local retry prediction\n\nOne total budget including retries. The implementation is a recorded example; execution is live.\n\n<hick:file path="deadline.js">\n${implementation}console.log(JSON.stringify({target: elapsed(${JSON.stringify(inputs.attempts)}, ${inputs.budget}), short: elapsed([20, 30], ${inputs.budget}), empty: elapsed([], ${inputs.budget})}));\n</hick:file>\n`;
}
async function transition<T>(command: Record<string, unknown>, generation = false, current?: () => boolean): Promise<T> {
  await loadHickLang();
  if (!navigator.locks) throw Error("This browser lacks the storage lease required for prediction practice; use a supported browser or native private practice");
  try { return await navigator.locks.request("hickory-browser-prediction-journal", async () => {
    if (current && !current()) throw Error("Practice changed");
    const previous = localStorage.getItem(BROWSER_PREDICTION_KEY) ?? "";
    const operation = generation ? { ...command, generation: previous ? JSON.parse(previous).entries.length : 0 } : command;
    const result = browserPredictionTransition(previous, operation);
    if (result.error || typeof result.snapshot !== "string") throw Error("Browser prediction operation refused; retained history, identity or ordering is unavailable");
    // Synchronous durable admission before dispatch. Quota/permission failure
    // prevents execution; corrupt/missing history is never silently reset.
    localStorage.setItem(BROWSER_PREDICTION_KEY, result.snapshot);
    return result.value as T;
  }); } catch { throw Error("Browser prediction history could not be validated or persisted; execution is refused"); }
}
export async function sealBrowserPrediction(corrected: boolean, inputs: BrowserInputs, prediction: string, rationale: string, actor: UiActor, current: () => boolean): Promise<BrowserSeal> {
  const expected = Number(prediction);
  if (!prediction.trim() || !Number.isFinite(expected) || expected < 0 || expected > 8000 || !rationale.trim() || new TextEncoder().encode(rationale).length > 8192) throw Error("Enter a finite prediction from 0 to 8000ms and a rationale before sealing");
  const source = browserPredictionSource(corrected, inputs);
  const [sourceHash, inputsHash, worker, mechanism] = await Promise.all([hash(source), hash(JSON.stringify(inputs)), browserRuntimeAssetHash(), hickLangArtifactHash()]);
  if (!current()) throw Error("Practice changed before sealing; inspect it again");
  const identity = { source: sourceHash, inputs: inputsHash, dependencies: { mechanism }, environment: { worker_entry: worker } };
  const value = await transition<{ receipt: BrowserSeal["receipt"]; scenario: string }>({ operation: "seal", actor, seal: {
    scenario: { family: "retry-total-budget-browser-v1", inputs, requirement_reference: "R1: one total budget including retries", requirement: { kind: "number_at_most", value: inputs.budget }, identity_known: false },
    identity, target: { kind: "output", execution: "deadline-output" }, predicted: { kind: "equals", value: expected }, rationale,
  } }, true, current);
  return { ...value, actor, identity, source };
}
export async function browserPredictionHistory(seal: BrowserSeal, actor: UiActor): Promise<BrowserSummary[]> {
  const history = await transition<{ attempts: BrowserSummary[] }>({ operation: "history", current: seal.identity, actor, scenarios: [seal.scenario] });
  return history.attempts.map(value => ({ attempt: value.attempt, classification: value.classification, prediction_matched: value.prediction_matched, original_requirement_satisfied: value.original_requirement_satisfied, current_certification: false, original_expectation_unchanged: value.original_expectation_unchanged === true }));
}
async function checkRuntimeAssets(seal: BrowserSeal) {
  const [worker, mechanism] = await Promise.all([browserRuntimeAssetHash(), hickLangArtifactHash()]);
  if (worker !== seal.identity.environment.worker_entry || mechanism !== seal.identity.dependencies.mechanism) throw Error("Browser runtime assets changed; reseal before execution. Imported dependency/browser closure remains unverified");
}
export async function runBrowserPrediction(seal: BrowserSeal, inputs: BrowserInputs, actor: UiActor, signal: AbortSignal, current: () => boolean): Promise<BrowserSummary> {
  if (seal.actor.kind !== actor.kind || seal.actor.id !== actor.id || !current() || signal.aborted) throw Error("The current predictor must run its own current seal");
  await checkRuntimeAssets(seal);
  // One same-origin runtime lease. Another page cannot recover or race a live
  // practice run. Journal transitions use their separate short storage lease.
  return navigator.locks.request("hickory-browser-prediction-runtime", { ifAvailable: true }, async lease => {
    if (!lease) throw Error("Another browser prediction runtime lease is active; inspect it before retrying");
    if (!current() || signal.aborted) throw Error("Practice changed before admission");
    const admission = await transition<{ receipt: { id: string }; dispatch: boolean }>({ operation: "admit", seal: seal.receipt.id, request: crypto.randomUUID(), identity: seal.identity, actor });
    if (!admission.dispatch) throw Error("This admission already dispatched; inspect retained history");
    let backend: ReturnType<typeof createBrowserDebugger> | undefined;
    try {
      if (!current() || signal.aborted) throw Error("Practice changed before dispatch");
      const { run, output } = await new Promise<{ run: RuntimeRun; output: string }>((resolve, reject) => {
        let output = "", settled = false;
        const finish = (error?: boolean, run?: RuntimeRun) => { if (settled) return; settled = true; clearTimeout(timer); signal.removeEventListener("abort", abort); if (error || !run) reject(Error("Browser execution unavailable; diagnostics remain withheld")); else resolve({ run, output }); };
        const abort = () => finish(true);
        const timer = setTimeout(() => finish(true), 10000);
        signal.addEventListener("abort", abort, { once: true });
        backend = createBrowserDebugger({ readDocument: () => ({ source: seal.source, revision: seal.identity.source }), onOutput: text => { if (output.length + text.length > 65536) finish(true); else output += text; }, onRuntime: run => { if (run.status !== "running") finish(run.status !== "completed", run); } });
        backend.client.on(event => { if (event.event === "failed" || event.event === "stopped") finish(true); });
        backend.client.start("browser-prediction.md", [], "deadline.js");
      });
      await checkRuntimeAssets(seal);
      if (!current() || signal.aborted || run.sources.find(source => source.id === "document")?.revision !== seal.identity.source || run.truncated || run.missing_artifacts.length) throw Error("Browser result identity unavailable");
      const actual: unknown = JSON.parse(output.trim());
      if (!actual || typeof actual !== "object") throw Error("Browser result unavailable");
      const fields = actual as Record<string, unknown>;
      if (Object.keys(fields).sort().join(",") !== "empty,short,target" || [fields.target, fields.short, fields.empty].some(value => typeof value !== "number" || !Number.isFinite(value))) throw Error("Browser result unavailable");
      const accepted = fields.short === Math.min(50, inputs.budget) && fields.empty === 0 && Number(fields.target) >= 0;
      const summary = await transition<BrowserSummary>({ operation: "finish", attempt: admission.receipt.id, execution_accepted: accepted, result: { identity: seal.identity, target: { kind: "output", execution: "deadline-output" }, actual: fields.target, evidence: { runtime: run.id, source: seal.identity.source, short: fields.short, empty: fields.empty, scope: "Live interpreter; browser-local history, incomplete environment/prior-exposure knowledge" } } });
      return { ...summary, current_certification: false, independent_checks_accepted: accepted };
    } catch {
      // The worker is terminated before an interrupted result is recorded. An
      // uncertain/crashed page remains non-passing rather than auto-reexecuting.
      backend?.dispose();
      await transition({ operation: "interrupt", attempt: admission.receipt.id }).catch(() => {});
      throw Error("Browser execution could not be retained; values/diagnostics remain withheld. Inspect history before retrying");
    } finally { backend?.dispose(); }
  });
}
export async function promoteBrowserPrediction(seal: BrowserSeal, attempt: string, actor: UiActor): Promise<BrowserSummary> {
  try { await transition<BrowserSummary>({ operation: "promote", attempt, current: seal.identity, actor }); } catch { throw Error("Promotion blocked: browser-local prior exposure and full environment identity are Unknown; passing comparisons cannot grant host certification. Use a fresh native private practice for an authoritative check"); }
  throw Error("User-controlled browser history cannot grant host certification");
}
export async function inspectBrowserPrediction(attempt: string, actor: UiActor) {
  if (actor.kind !== "human") throw Error("A human inspects private browser results in a separate window");
  return transition<{ actual: unknown; evidence: unknown }>({ operation: "reveal", attempt, actor });
}

/** Caller holds no live worker. A cross-tab runtime lease is the independent witness. */
export async function recoverBrowserPrediction(seal: BrowserSeal, actor: UiActor): Promise<{ interrupted: number; execution_replayed: false; current_certification: false }> {
  if (!navigator.locks) throw Error("This browser cannot establish a runtime recovery lease");
  return navigator.locks.request("hickory-browser-prediction-runtime", { ifAvailable: true }, async lease => {
    if (!lease) throw Error("Another browser prediction runtime lease is active; worker absence is unconfirmed, so recovery is refused");
    return transition({ operation: "recover", actor, scenarios: [seal.scenario] });
  });
}
</book:entry>

</book:doc>
