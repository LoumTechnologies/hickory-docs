import { useId, useState } from "react";
import { UiActionsVisibility, useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";
export type PredictionRevisionNavigation = { available: string[]; open: (hash: string) => Promise<UiOutcome> };

type RecordValue = Record<string, unknown>;
export interface PredictionHistory {
  format: "hickory-selected-prediction-history"; version: 1;
  selected_scenarios: string[]; omitted_events: number; omitted_attempts: number;
  omitted_context_observations: number; scope: string;
  history: { generation: number; attempts: RecordValue[]; ordering: { sequence: number; previous: string; digest: string; event: RecordValue }[] };
  debugger_contexts?: RecordValue[];
}
const classifications = ["not_run", "running", "mismatch", "requirement_failed", "execution_rejected", "requirement_changed", "first_attempt_verified", "informed_prediction_verified", "verified_after_correction", "verified_after_prior_attempt", "adjusted_after_observation", "prediction_changed", "stale", "unknown", "replayed"];
function record(value: unknown): value is RecordValue { return !!value && typeof value === "object" && !Array.isArray(value); }
function natural(value: unknown): value is number { return Number.isSafeInteger(value) && Number(value) >= 0; }
const hash = (value: unknown) => typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
const text = (value: unknown): value is string => typeof value === "string" && value.length > 0;
function keys(value: RecordValue, allowed: string[]) { return Object.keys(value).every(key => allowed.includes(key)); }

/** Structural validation only. A supplied host verdict is never re-certified
 * by a browser without its private execution/identity authority. */
export function validatePredictionHistory(value: unknown): asserts value is PredictionHistory {
  const fail = () => { throw Error("Invalid selected prediction history"); };
  if (!record(value) || value.format !== "hickory-selected-prediction-history" || value.version !== 1 ||
      !keys(value, ["format", "version", "selected_scenarios", "omitted_events", "omitted_attempts", "omitted_context_observations", "history", "scope", "debugger_contexts"]) ||
      !Array.isArray(value.selected_scenarios) || !value.selected_scenarios.length || value.selected_scenarios.length > 64 || !value.selected_scenarios.every(hash) ||
      ![value.omitted_events, value.omitted_attempts, value.omitted_context_observations].every(natural) || !text(value.scope) || !record(value.history)) return fail();
  const history = value.history;
  if (!keys(history, ["generation", "attempts", "ordering"]) || !natural(history.generation) || !Array.isArray(history.attempts) || !Array.isArray(history.ordering) || history.ordering.length > 100000 || JSON.stringify(value).length > 8 * 1024 * 1024) return fail();
  const seals = new Map<string, RecordValue>();
  const starts = new Map<string, string>();
  const finished = new Set<string>();
  let previous = 0;
  for (const entry of history.ordering) {
    if (!record(entry) || !keys(entry, ["sequence", "previous", "digest", "event"]) || !natural(entry.sequence) || entry.sequence <= previous || entry.sequence > history.generation || typeof entry.previous !== "string" || !hash(entry.digest) || !record(entry.event)) return fail();
    previous = entry.sequence;
    const event = entry.event;
    const eventKeys: Record<string, string[]> = {
      sealed: ["kind", "id", "actor", "seal", "context", "future_stop"], started: ["kind", "id", "seal", "replay", "actor"], finished: ["kind", "attempt", "actual"],
      promoted: ["kind", "attempt", "actor", "identity"], interrupted: ["kind", "attempt", "actor", "reason"], evidence_unavailable: ["kind", "attempt", "actor"], observed: ["kind", "attempt", "scenario", "actor", "surface", "identity"],
    };
    if (!eventKeys[String(event.kind)] || !keys(event, eventKeys[String(event.kind)])) return fail();
    if (event.kind === "sealed") {
      if (!text(event.id) || seals.has(event.id) || !record(event.actor) || !record(event.seal) || !record(event.seal.identity) || !hash(event.seal.identity.source) || !hash(event.seal.identity.inputs) || !record(event.seal.scenario) || !text(event.seal.scenario.requirement_reference) || !text(event.seal.rationale) || !record(event.seal.target) || !record(event.seal.predicted) || !Array.isArray(event.context)) return fail();
      if (event.future_stop !== undefined) {
        const stop = event.future_stop;
        if (!record(stop) || !keys(stop, ["run", "thread", "frame", "hit", "prior_visible_digest"]) || !text(stop.run) || !natural(stop.thread) || Number(stop.thread) <= 0 || !natural(stop.frame) || Number(stop.frame) <= 0 || !natural(stop.hit) || Number(stop.hit) < 1 || Number(stop.hit) >= 100 || !hash(stop.prior_visible_digest)) return fail();
      }
      seals.set(event.id, event.seal);
      if (!keys(event.seal, ["scenario", "identity", "target", "predicted", "rationale"]) || !keys(event.seal.identity as RecordValue, ["source", "inputs", "dependencies", "environment"])) return fail();
    } else if (event.kind === "started") {
      if (!text(event.id) || starts.has(event.id) || !text(event.seal) || !seals.has(event.seal) || typeof event.replay !== "boolean" || !record(event.actor)) return fail();
      starts.set(event.id, event.seal);
    } else if (event.kind === "finished") {
      // The selected exporter emits a fixed sentinel, never actual/evidence.
      if (!keys(event, ["kind", "attempt", "actual"]) || event.actual !== "withheld from agents" || !text(event.attempt) || !starts.has(event.attempt)) return fail();
      if (finished.has(event.attempt)) return fail();
      finished.add(event.attempt);
    } else if (["promoted", "interrupted", "evidence_unavailable"].includes(String(event.kind))) {
      if (!text(event.attempt) || !starts.has(event.attempt) || !record(event.actor)) return fail();
    } else if (event.kind === "observed") {
      if (!text(event.scenario) || !value.selected_scenarios.includes(event.scenario) || !record(event.actor) || !record(event.identity) || !text(event.surface)) return fail();
    } else return fail();
  }
  for (const attempt of history.attempts) {
    if (!record(attempt) || !keys(attempt, ["attempt", "scenario", "seal", "classification", "prediction_matched", "original_requirement_satisfied", "observed", "promoted", "current_certification"]) ||
        !text(attempt.attempt) || !text(attempt.seal) || starts.get(attempt.attempt) !== attempt.seal || !text(attempt.scenario) || !value.selected_scenarios.includes(attempt.scenario) ||
        !classifications.includes(String(attempt.classification)) || ![null, true, false].includes(attempt.prediction_matched as null | boolean) || ![null, true, false].includes(attempt.original_requirement_satisfied as null | boolean) ||
        typeof attempt.observed !== "boolean" || typeof attempt.promoted !== "boolean" || attempt.current_certification !== false) return fail();
    if ((attempt.prediction_matched !== null || attempt.original_requirement_satisfied !== null) && !finished.has(attempt.attempt)) return fail();
  }
  if (history.attempts.length !== starts.size) return fail();
  if (value.debugger_contexts !== undefined) {
    if (!Array.isArray(value.debugger_contexts)) return fail();
    for (const capture of value.debugger_contexts) {
      if (!record(capture) || !keys(capture, ["attempt", "adapter", "file", "line", "problem", "moved", "truncated", "hits"]) || !text(capture.attempt) || !starts.has(capture.attempt) || !["private_capture_v1", "combined_live_v1"].includes(String(capture.adapter)) || !text(capture.file) || !natural(capture.line) || !Array.isArray(capture.hits) || typeof capture.problem !== "boolean" || typeof capture.moved !== "boolean" || typeof capture.truncated !== "boolean") return fail();
      for (const hit of capture.hits) {
        if (!record(hit) || !keys(hit, ["index", "context"]) || !natural(hit.index) || hit.context !== null && (!record(hit.context) || !keys(hit.context, ["run", "thread", "frame", "function", "document_line", "source", "source_line", "breakpoint_bound"]))) return fail();
        if (record(hit.context)) {
          const context = hit.context;
          if (["run", "function", "source"].some(key => context[key] !== undefined && context[key] !== null && typeof context[key] !== "string") ||
              ["thread", "frame", "document_line", "source_line"].some(key => context[key] !== undefined && context[key] !== null && !natural(context[key])) || context.breakpoint_bound !== undefined && typeof context.breakpoint_bound !== "boolean") return fail();
        }
      }
    }
  }
}

export function predictionWitnesses(value: PredictionHistory, requirement: string): string[] {
  const seals = new Map(value.history.ordering.filter(entry => entry.event.kind === "sealed").map(entry => [entry.event.id, entry.event.seal as RecordValue]));
  return value.history.attempts.filter(attempt => String((seals.get(attempt.seal)?.scenario as RecordValue | undefined)?.requirement_reference ?? "").split(";").map(part => part.trim()).includes(requirement)).map(attempt => String(attempt.attempt));
}

export function PredictionEvidence({ value, sourceHash, navigation }: { value: PredictionHistory; sourceHash: string; navigation?: PredictionRevisionNavigation }) {
  const [open, setOpen] = useState(false);
  const parent = `prediction-evidence:${useId()}`;
  const revision = useUiRevision(() => [value, sourceHash, open]);
  useUiReading("selected prediction history status", { historical: true, attempts: value.history.attempts.length, open, actual_values: "withheld", current_certification: false });
  useUiCommands([{ key: parent, command: { label: "Set selected prediction history visibility", parent, subject: parent, parameters: { open: { type: "boolean" } }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true }), execute: async args => { setOpen(Boolean(args.open)); return { status: "completed", value: { open: Boolean(args.open), current_certification: false } }; },
  } }]);
  return <section aria-label="Selected prediction evidence"><h2>Recorded prediction chain</h2>
    <p>Historical host-reported comparisons. Actual target values and private diagnostics are withheld. This viewer does not execute checks or certify current source.</p>
    <p>Omitted: {value.omitted_events} events, {value.omitted_attempts} attempts, {value.omitted_context_observations} prior context observations. {value.scope}</p>
    <details open={open} onToggle={event => setOpen(event.currentTarget.open)}><summary>Inspect sealed predictions and retained attempts</summary>
      <UiActionsVisibility visible={open}><PredictionRows value={value} sourceHash={sourceHash} navigation={navigation} /></UiActionsVisibility>
    </details></section>;
}
function PredictionRows({ value, sourceHash, navigation }: { value: PredictionHistory; sourceHash: string; navigation?: PredictionRevisionNavigation }) {
  useUiReading("disclosed selected prediction chain", value);
  const seals = new Map(value.history.ordering.filter(entry => entry.event.kind === "sealed").map(entry => [entry.event.id, entry.event.seal as RecordValue]));
  const parent = `prediction-revisions:${useId()}`;
  const revision = useUiRevision(() => [value, sourceHash, navigation?.available]);
  const revisions = [...new Set([...seals.values()].map(seal => String((seal.identity as RecordValue).source)))];
  const invoke = useUiCommands(revisions.map(hash => ({ key: `${parent}:${hash}`, command: {
    label: `Inspect captured source revision ${hash}`, parent, subject: `${parent}:${hash}`, parameters: {}, prerequisites: ["Exact source/path/tested-revision match in the selected publication index"],
    read: () => ({ revision: revision(), enabled: !!navigation?.available.includes(hash), reason: navigation?.available.includes(hash) ? undefined : "Load a publication index with one matching source revision; missing/ambiguous evidence is unavailable" }),
    execute: async () => navigation ? navigation.open(hash) : { status: "unavailable" as const, reason: "No selected publication index is available" },
  } })));
  return <>{value.history.attempts.map(attempt => {
    const seal = seals.get(attempt.seal)!, identity = seal.identity as RecordValue, scenario = seal.scenario as RecordValue;
    return <section key={String(attempt.attempt)} id={`prediction-${attempt.attempt}`}><h3>{String(attempt.classification)} · attempt {String(attempt.attempt)}</h3>
      <p>Scenario {String(attempt.scenario)} · requirement {String(scenario.requirement_reference)}</p>
      <p>Prediction matched: {String(attempt.prediction_matched ?? "unavailable")}; original requirement satisfied: {String(attempt.original_requirement_satisfied ?? "unavailable")}.</p>
      <p>Source SHA-256: {String(identity.source)} · {identity.source === sourceHash ? "matches this captured document" : "different revision; source bytes not included in this snapshot"}. Current certification: unavailable.</p>
      <button disabled={!navigation?.available.includes(String(identity.source))} onClick={() => { void invoke(`${parent}:${identity.source}`, {}); }}>Inspect captured source revision {String(identity.source)}</button>
      <p>{String(seal.rationale)}</p><pre>{JSON.stringify({ requirement: scenario.requirement, predicted: seal.predicted, target: seal.target, identity, prior_exposure: seal.context ?? "See ordered exposure events" }, null, 2)}</pre>
      <h4>Recorded debugger context</h4>
      {value.debugger_contexts?.some(capture => capture.attempt === attempt.attempt) ? value.debugger_contexts.filter(capture => capture.attempt === attempt.attempt).map((capture, index) => <div key={index}>
        <p>{String(capture.file)}:{String(capture.line)} · {capture.problem || capture.moved ? "unavailable or moved capture; inspect the non-passing host verdict" : "recorded capture; browsing does not execute a debugger"}</p>
        <pre>{JSON.stringify(capture, null, 2)}</pre>
      </div>) : <p>Observed frame/hit context was not selected or is unavailable. The sealed target above is a prediction, not an observed stop.</p>}
    </section>;
  })}<h3>Retained event ordering</h3><pre>{JSON.stringify(value.history.ordering, null, 2)}</pre></>;
}
