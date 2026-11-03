import { useEffect, useState } from "react";
import { request } from "../api/client";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";

type Arm = { arm: string; status: string; model?: string; answer?: string; error?: string; wall_seconds: number | null; input_tokens: number | null; output_tokens: number | null; total_tokens: number | null; usage_scope?: string; correct: boolean | null; workspace?: string; transcript?: string; review?: { notes: string } | null };
type Pair = { feature: string; arms: Arm[]; savings: Record<string, number | null> | null };
type TrialRecord = { id: string; document: string; prompt: string; status: string; created_at: string; pairs: Pair[]; active_pair?: Pair };
type Dataset = { records: TrialRecord[]; features: { id: string; title: string; pairs: number; wall_seconds_saved?: { n: number; median: number | null }; tokens_saved?: { n: number; median: number | null } }[]; coverage: { contract: string; status: string }[]; storage: string; limitations: string };
const choices = [{ id: "literate-indexes", title: "Literate source indexes" }, { id: "lineage-tools", title: "Lineage tools for generated code" }];
const number = (value: number | null | undefined) => value == null ? "unknown" : value.toLocaleString(undefined, { maximumFractionDigits: 2 });

/** The checkbox authorizes extra ACP requests, so only a human can turn it on. */
export function FeatureExperiments({ doc, backend, running, features, activeFeatures = [], onFeatures, onUpdate }: { doc: string; backend: string; running: boolean; features: string[]; activeFeatures?: string[]; onFeatures(value: string[]): void; onUpdate?: () => Promise<void> }) {
  const [open, setOpen] = useState(false);
  const [data, setData] = useState<Dataset | null>(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [notes, setNotes] = useState("");
  const [contract, setContract] = useState("");
  const [busy, setBusy] = useState(false);
  const parent = `feature-experiments:${doc}`;
  const revision = useUiRevision(() => [backend, running, features, open, data, notes, contract, busy]);
  useUiReading("Feature comparison dataset", { document: doc, opted_in: backend === "codex" && features.length > 0, features, current_request_features: activeFeatures, open, data, error });
  useEffect(() => {
    if (!open) return;
    let active = true;
    const read = async () => { try { const next = await request<Dataset>("GET", "/api/feature-experiments"); if (active) { setData(next); setError(""); } } catch (reason) { if (active) setError(String(reason)); } };
    void read();
    const timer = running ? setInterval(() => void read(), 5000) : undefined;
    return () => { active = false; if (timer) clearInterval(timer); };
  }, [open, running, doc]);
  const unavailable = (reason: string): UiOutcome => ({ status: "unavailable", reason });
  const invoke = useUiCommands([
    { key: `${parent}:measure`, command: { label: "Measure this request with and without selected features", parent, subject: doc, parameters: { enabled: { type: "boolean" } }, prerequisites: ["Human opt-in authorizes two extra Codex ACP runs per selected feature"], read: () => ({ revision: revision(), enabled: !busy && !running && backend === "codex", value: features.length > 0 }), execute: async (args, actor) => {
      if (actor.kind !== "human") return { status: "requires_input", request: parent, reason: "A human must authorize additional agent runs" };
      onFeatures(args.enabled ? ["literate-indexes"] : []); return { status: "completed", value: { enabled: args.enabled } };
    } } },
    { key: `${parent}:feature`, command: { label: "Choose a feature to measure", parent, subject: doc, parameters: { id: { type: "string", maxLength: 128, choices: choices.map(c => c.id) }, enabled: { type: "boolean" } }, prerequisites: ["Human opt-in; one feature is changed per pair"], read: () => ({ revision: revision(), enabled: !busy && !running && backend === "codex" && features.length > 0, value: features }), execute: async (args, actor) => {
      if (actor.kind !== "human") return { status: "requires_input", request: parent, reason: "A human must choose extra comparison runs" };
      const id = args.id as string;
      onFeatures(args.enabled ? [...new Set([...features, id])] : features.filter(f => f !== id)); return { status: "completed", value: { selected: id, enabled: args.enabled } };
    } } },
    { key: `${parent}:open`, command: { label: "Show feature measurement dataset", parent, subject: doc, parameters: { open: { type: "boolean" } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true, value: open }), execute: async args => { setOpen(args.open as boolean); return { status: "completed", value: { open: args.open } }; } } },
    { key: `${parent}:export`, command: { label:"Export feature measurement dataset", parent, subject:doc, parameters:{}, prerequisites:[], read:() => ({ revision:revision(), enabled:open && !busy }), execute:async (_args, _actor, signal): Promise<UiOutcome> => {
      setBusy(true); setError("");
      try {
        const latest = await request<Dataset>("GET", "/api/feature-experiments");
        setData(latest);
        const text = JSON.stringify(latest, null, 2);
        if (signal.aborted) return { status:"completed", value:{ browser_download_requested:false } };
        const url = URL.createObjectURL(new Blob([text], { type:"application/json" }));
        const revoke = URL.revokeObjectURL.bind(URL);
        try { const link = document.createElement("a"); link.href = url; link.download = "hickory-feature-measurements.json"; link.click(); }
        finally { setTimeout(() => revoke(url), 1000); }
        setNotice("Dataset download requested. Source snapshots and working copies remain in the listed local storage directory.");
        return { status:"completed", value:{ browser_download_requested:true, file_saved:"Browser-owned; not observed", bytes:new TextEncoder().encode(text).length } };
      } catch (reason) { setError(String(reason)); return unavailable(String(reason)); }
      finally { setBusy(false); }
    } } },
    { key: `${parent}:update`, command: { label: "Update Codex adapter for measured token totals", parent, subject: doc, parameters: {}, prerequisites: ["Downloads the audited adapter through Hickory's existing installer"], read: () => ({ revision: revision(), enabled: !!onUpdate && backend === "codex" && !running && !busy }), execute: async (_args, actor) => {
      if (actor.kind !== "human") return { status:"requires_input", request:parent, reason:"A human chooses whether to update the installed adapter" };
      if (!onUpdate) return unavailable("Adapter installation is unavailable in this view");
      setBusy(true); setError(""); setNotice("");
      try { await onUpdate(); setNotice("Codex adapter updated. New comparison sessions will use it."); return { status:"completed", value:{ adapter_updated:true } }; }
      catch (reason) { setError(String(reason)); return unavailable(String(reason)); } finally { setBusy(false); }
    } } },
    { key: `${parent}:notes`, command: { label: "Draft feature correctness evidence", parent, subject: doc, parameters: { notes: { type: "string", maxLength: 8192 }, contract: { type: "string", maxLength: 4096 } }, prerequisites: [], read: () => ({ revision: revision(), enabled: !busy, value: notes }), execute: async args => { setNotes(args.notes as string); setContract(args.contract as string); return { status: "completed", value: { drafted: true, reviewed: false } }; } } },
    { key: `${parent}:review`, command: { label: "Record correctness of a comparison result", parent, subject: doc, parameters: { id: { type: "string", maxLength: 16 }, feature: { type: "string", maxLength: 128, choices: choices.map(c => c.id) }, arm: { type: "string", maxLength: 128, choices: ["enabled", "disabled"] }, correct: { type: "boolean" } }, prerequisites: ["Inspect the retained working copy and tests; human judgment is separate from completion"], read: () => ({ revision: revision(), enabled: !busy && !!notes.trim() }), execute: async (args, actor) => {
      if (actor.kind !== "human") return { status: "requires_input", request: parent, reason: "An agent cannot certify its own comparison outcome" };
      if (!notes.trim()) return unavailable("Describe the correctness evidence first");
      setBusy(true);
      try { setData(await request<Dataset>("POST", `/api/feature-experiments/${args.id}/review`, { feature: args.feature, arm: args.arm, correct: args.correct, notes, ...(contract ? { contract } : {}) })); setNotes(""); return { status: "completed", value: { review_recorded: true, kind: "human-declared" } }; }
      catch (reason) { setError(String(reason)); return unavailable(String(reason)); } finally { setBusy(false); }
    } } },
  ]);
  const act = (key: string, args: Parameters<typeof invoke>[1]) => { void invoke(`${parent}:${key}`, args).catch(reason => setError(String(reason))); };
  const records = data?.records.filter(record => record.document === doc) ?? [];
  return <section className="feature-experiments" aria-label="Feature performance measurements">
    <label><input type="checkbox" checked={backend === "codex" && features.length > 0} disabled={running || busy || backend !== "codex"} onChange={event => act("measure", { enabled: event.target.checked })} /> Measure this request with and without Hickory features</label>
    {running && activeFeatures.length > 0 && <p role="status">This request is measuring {activeFeatures.join(", ")}. The next request starts with measurement off.</p>}
    {backend !== "codex" && <small>Choose Codex ACP to collect comparisons.</small>}
    {backend === "codex" && features.length > 0 && <div>
      {choices.map(choice => <label key={choice.id}><input type="checkbox" checked={features.includes(choice.id)} disabled={running || busy} onChange={event => act("feature", { id: choice.id, enabled: event.target.checked })} />{choice.title}</label>)}
      <p>{2 * features.length} extra Codex runs after your normal request, up to five minutes each. Fresh working copies keep trial edits separate; they use the adapter’s normal host permissions. Comparisons start fresh, without conversation history.</p>
    </div>}
    <button type="button" className="btn-link" aria-expanded={open} disabled={busy} onClick={() => act("open", { open: !open })}>{open ? "Hide measurements" : "Measurements and dataset"}</button>
    {error && <p role="alert">{error}</p>}
    {notice && <p role="status">{notice}</p>}
    {open && <div>
      <p>Results accumulate locally. Completion is separate from correctness; savings appear only after both results are reviewed as correct. Negative savings mean the feature cost more.</p>
      {onUpdate && backend === "codex" && <button disabled={running || busy} onClick={() => act("update", {})}>Update Codex adapter for token totals</button>}
      <button disabled={busy} onClick={() => act("export", {})}>Export dataset</button>
      <p>{data?.coverage.length ?? 0} contracts inventoried; feature-specific correctness and performance evidence still needs to be collected. Storage: {data?.storage ?? "loading"}</p>
      <table><thead><tr><th>Feature</th><th>Pairs</th><th>Median prompt seconds saved</th><th>Median tokens saved</th></tr></thead><tbody>{data?.features.map(feature => <tr key={feature.id}><td>{feature.title}</td><td>{feature.pairs}</td><td>{number(feature.wall_seconds_saved?.median)} (n={feature.wall_seconds_saved?.n ?? 0})</td><td>{number(feature.tokens_saved?.median)} (n={feature.tokens_saved?.n ?? 0})</td></tr>)}</tbody></table>
      <p>Medians describe reviewed pairs in this project across recorded tasks; they are not population estimates.</p>
      {!records.length && <p>No comparisons recorded for this conversation yet.</p>}
      {records.map(record => <article key={record.id}>
        <h4>{record.prompt.slice(0, 160)}</h4><p>Last recorded status: {record.status} · {record.created_at}</p>
        {[...record.pairs, ...(record.active_pair ? [record.active_pair] : [])].map((pair, i) => <div key={`${pair.feature}-${i}`}>
          <strong>{pair.feature}</strong>
          {pair.arms.map(arm => <details key={arm.arm}><summary>{arm.arm} · {arm.status} · {number(arm.wall_seconds)} seconds · {number(arm.total_tokens)} tokens · correctness {arm.correct === null ? "unreviewed" : arm.correct ? "accepted" : "rejected"}</summary>
            <pre>{arm.answer ?? arm.error ?? "No answer recorded"}</pre><p>{arm.usage_scope}</p><p>Working copy: {arm.workspace} · Record: {arm.transcript}</p>
            {arm.review?.notes && <p>Evidence: {arm.review.notes}</p>}
            {arm.status === "completed" && <><button disabled={busy || !notes.trim()} onClick={() => act("review", { id: record.id, feature: pair.feature, arm: arm.arm, correct: true })}>Correct</button><button disabled={busy || !notes.trim()} onClick={() => act("review", { id: record.id, feature: pair.feature, arm: arm.arm, correct: false })}>Incorrect</button></>}
          </details>)}
          {pair.savings && <p>Saved {number(pair.savings.wall_seconds)} seconds · {number(pair.savings.total_tokens)} tokens (disabled minus enabled).</p>}
        </div>)}
      </article>)}
      <label>Correctness evidence<textarea disabled={busy} value={notes} maxLength={8192} onChange={event => act("notes", { notes: event.target.value, contract })} placeholder="Tests or observations supporting the result you are about to review" /></label>
      <label>Feature contract checked (optional)<select disabled={busy} value={contract} onChange={event => act("notes", { notes, contract: event.target.value })}><option value="">No contract attribution</option>{data?.coverage.map(item => <option key={item.contract} value={item.contract}>{item.contract} · {item.status}</option>)}</select></label>
      <p>{data?.limitations}</p>
    </div>}
  </section>;
}
