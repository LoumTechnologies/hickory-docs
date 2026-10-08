import { useMemo, useState } from "react";
import { request } from "../api/client";
import "./AssumptionAudit.css";
import { openLocation } from "../lib/revealLine";

import type { AuditRecord } from "../api/sessionTypes";
export type { AuditRecord } from "../api/sessionTypes";

/** All branches stay available here; declared rationale never becomes proof. */
export function AssumptionAudit({ records, path, source }: { records: AuditRecord[]; path: string; source?: string }) {
  const [statement, setStatement] = useState("");
  const [reason, setReason] = useState("");
  const [supersedes, setSupersedes] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("audit");
  const gaps = records.filter(record => record.record_kind === "audit-gap");
  const declared = records.filter(record => record.record_kind === "audit");
  const filtered = useMemo(() => records.filter(record => (category === "all" || record.record_kind === category) &&
    JSON.stringify(record).toLowerCase().includes(query.toLowerCase())), [records, query, category]);
  return <details className="assumption-audit">
    <summary>Assumption audit · {declared.length} declared records · all branches</summary>
    <p>Keep this session to revisit why work was done. Records are declarations of assumptions and decisions;
      completeness is not established. Observed work and executed checks remain separate.</p>
    {gaps.length > 0 && <p role="alert">{gaps.length} unreadable audit records. Coverage is incomplete; inspect the full session record.</p>}
    <label>Search audit <input aria-label="Search assumption audit" value={query} onChange={event => setQuery(event.target.value)} /></label>
    <label>Show <select aria-label="Audit records" value={category} onChange={event => setCategory(event.target.value)}>
      <option value="audit">Assumptions, decisions and guarantee reviews</option>
      <option value="observed-work">Work to inspect for missing assumptions</option>
      <option value="edit-decision">Review decisions</option>
      <option value="acp-edit-mode">Review policy changes</option>
      <option value="acp-permission-result">Adapter permission decisions</option>
      <option value="acp-activity">Adapter reported work</option>
      <option value="audit-action">Requested actions and outcomes</option>
      <option value="verification">Executed verification</option>
      <option value="read-snapshot">Historical source passages</option>
      <option value="model-request">Supplied requests (adapter memory may be opaque)</option>
      <option value="citation-check">Citation target checks</option>
      <option value="audit-gap">Unreadable records</option>
      <option value="all">All records</option>
    </select></label>
    {source && <button className="btn-link" onClick={() => {
      const url = URL.createObjectURL(new Blob([source], { type: "text/markdown" }));
      const anchor = document.createElement("a"); anchor.href = url; anchor.download = path.split(/[\\/]/).pop() || "session.md";
      anchor.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
    }}>Download full session record</button>}
    <details><summary>Add an audit note or correction</summary><form onSubmit={async event => {
      event.preventDefault(); setSaving(true); setMessage(null);
      try {
        await request("POST", "/api/sessions/audit", { path, entry: { statement, reason, ...(supersedes.trim() ? { supersedes: supersedes.trim() } : {}) } });
        setStatement(""); setReason(""); setSupersedes(""); setMessage("Recorded as a user declaration. Earlier records remain; reopen or wait for the session to refresh.");
      } catch (error) { setMessage(error instanceof Error ? error.message : String(error)); }
      finally { setSaving(false); }
    }}>
      <label>Statement<textarea required value={statement} onChange={event => setStatement(event.target.value)} /></label>
      <label>Reason<textarea required value={reason} onChange={event => setReason(event.target.value)} /></label>
      <label>Supersedes entry id (optional)<input value={supersedes} onChange={event => setSupersedes(event.target.value)} /></label>
      <button disabled={saving} className="btn" type="submit">Record audit note</button>
      {message && <p role="status">{message}</p>}
    </form></details>
    {filtered.length === 0 && <p>No matching records. This does not establish that the work had no consequential assumptions.</p>}
    {filtered.map((record, index) => <section key={record.id ?? `${record.session_line}:${index}`}>
      <p><strong>{record.entry?.statement ?? record.tool ?? record.path ?? record.state ?? record.kind ?? record.record_kind}</strong>
        {" · "}{record.actor ?? "recorded work"}{record.turn ? ` · turn ${record.turn}` : ""}
        {record.parent ? ` · parent ${record.parent}` : ""}
        <button className="btn-link" onClick={() => openLocation(path, record.session_line)}>Open record</button></p>
      {record.id && <p><code>{record.id}</code>{record.recorded_at ? ` · ${record.recorded_at}` : ""}</p>}
      {record.entry && <>
        <p>Reason: {record.entry.reason}</p>
        {record.entry.requirement && <p>Original requirement: {record.entry.requirement}</p>}
        {record.entry.supersedes && <p>Supersedes <code>{record.entry.supersedes}</code>; the original remains recorded.</p>}
        {([['Evidence claimed', record.entry.evidence], ['Affects', record.entry.affects], ['Alternatives', record.entry.alternatives], ['Exclusions', record.entry.exclusions], ['Witnesses claimed', record.entry.witnesses ?? []], ['Challenges', record.entry.challenges ?? []]] as const).map(([label, values]) =>
          values?.length ? <p key={label}>{label}: {values.join("; ")}</p> : null)}
        <p>Would disprove it: {record.entry.falsifier || "Not recorded"}</p>
      </>}
      {record.dependency_snapshots?.map(dependency => <p key={dependency.path}>Dependency {dependency.path}: {dependency.current_state ?? "unavailable"} · <code>{dependency.sha256 ?? "snapshot unavailable"}</code></p>)}
      {record.record_kind === "read-snapshot" && <>
        <p>Historical returned bytes · current source {record.current_state ?? "unchecked"} · lines {record.first_line}–{record.last_line} · snapshot <code>{record.sha256}</code></p>
        <pre>{record.content}</pre>
        {record.path && <button className="btn-link" onClick={() => openLocation(record.path!, record.first_line ?? 1)}>Compare with current source</button>}
      </>}
      {record.record_kind === "verification" && <p>{record.state} · current sources {record.current_state ?? "unchecked"} · revision {record.revision ?? "not recorded"} · {record.scope}</p>}
      {!record.entry && record.record_kind !== "read-snapshot" && <details><summary>Recorded details</summary><pre>{JSON.stringify(record, null, 2)}</pre></details>}
    </section>)}
  </details>;
}

/** The harness result is visible beside the answer, independently of its prose. */
export function VerificationStanding({ records, turn, running }: { records: AuditRecord[]; turn?: string; running: boolean }) {
  const record = [...records].reverse().find(record => record.record_kind === "verification" && record.turn === turn);
  const state = running ? "pending" : record?.state === "passed" && record.current_state !== "unchanged" ? `historical pass · current sources ${record.current_state ?? "unavailable"}` : record?.state ?? "not recorded";
  return <p className="verification-standing" role="status">Hickory checks: <strong>{state}</strong>
    {record?.revision && <> · revision <code>{record.revision}</code></>}. Document execution and expectations only; semantic support is not assessed.</p>;
}
