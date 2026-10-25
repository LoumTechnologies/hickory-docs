<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/AssumptionAudit.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Assumptions remain auditable after the work](../../../../../../guarantees/agent/assumptions-remain-auditable-after-the-work/contract.md)
- [Historical evidence stays historical](../../../../../../guarantees/agent/historical-evidence-stays-historical/contract.md)

Captured SHA-256: `c02680030807b88c352bacd9f950a8fa6cf7ddadbeed771e3566f6370578e45c`.

<book:entry path="apps/web/src/components/AssumptionAudit.tsx">
import { useId, useMemo, useState } from "react";
import { EvidenceDisclosure } from "./EvidenceDisclosure";
import { AuditNoteForm } from "./AuditNoteForm";
import { AuditLocationButton, AuditSourceExport } from "./AuditNavigation";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import "./AssumptionAudit.css";

import type { AuditRecord } from "../api/sessionTypes";
export type { AuditRecord } from "../api/sessionTypes";

/** All branches stay available here; declared rationale never becomes proof. */
export function AssumptionAudit({ records, path, source }: { records: AuditRecord[]; path: string; source?: string }) {
  const declared = records.filter(record => record.record_kind === "audit").length;
  return <EvidenceDisclosure className="assumption-audit" label={`Assumption audit · ${declared} declared records · all branches`}
    value={{ path, declared_records: declared, completeness_established: false }}>
    <AuditBody key={path} records={records} path={path} source={source} />
  </EvidenceDisclosure>;
}

function AuditBody({ records, path, source }: { records: AuditRecord[]; path: string; source?: string }) {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("audit");
  const gaps = records.filter(record => record.record_kind === "audit-gap");
  const filtered = useMemo(() => records.filter(record => (category === "all" || record.record_kind === category) &&
    JSON.stringify(record).toLowerCase().includes(query.toLowerCase())), [records, query, category]);
  const parent = `audit-filter:${useId()}`;
  const revision = useUiRevision(() => [path, records, query, category]);
  useUiCommands([
    { key: `${parent}:search`, command: { parent, subject: parent, label: "Search assumption audit", parameters: { text: { type: "string", maxLength: 8192 } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true, value: { query } }), execute: async args => { setQuery(args.text as string); return { status: "completed", value: { filter_updated: true } }; } } },
    { key: `${parent}:category`, command: { parent, subject: parent, label: "Filter assumption audit records", parameters: { category: { type: "string", maxLength: 128,
      choices: ["audit", "observed-work", "edit-decision", "acp-edit-mode", "acp-permission-result", "acp-activity", "audit-action", "verification", "read-snapshot", "model-request", "citation-check", "audit-gap", "all"] } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true, value: { category } }), execute: async args => { setCategory(args.category as string); return { status: "completed", value: { filter_updated: true } }; } } },
  ]);
  useUiReading("visible assumption audit records", { path, query, category, gaps: gaps.length,
    records: filtered.map(record => ({ id: record.id, record_kind: record.record_kind, actor: record.actor, turn: record.turn,
      parent: record.parent, recorded_at: record.recorded_at, session_line: record.session_line,
      summary: record.entry?.statement ?? record.tool ?? record.path ?? record.state ?? record.kind ?? record.record_kind,
      entry: record.entry ? { statement: record.entry.statement, reason: record.entry.reason, requirement: record.entry.requirement,
        supersedes: record.entry.supersedes, evidence: record.entry.evidence, affects: record.entry.affects, alternatives: record.entry.alternatives,
        exclusions: record.entry.exclusions, witnesses: record.entry.witnesses ?? [], challenges: record.entry.challenges ?? [], falsifier: record.entry.falsifier } : null,
      dependency_snapshots: record.dependency_snapshots,
      historical_read: record.record_kind === "read-snapshot" ? { path: record.path, first_line: record.first_line, last_line: record.last_line,
        content: record.content, sha256: record.sha256, current_state: record.current_state ?? "unchecked" } : null,
      verification: record.record_kind === "verification" ? { state: record.state, current_state: record.current_state ?? "unchecked", revision: record.revision, scope: record.scope } : null,
    })), completeness_established: false });
  return <>
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
    {source && <AuditSourceExport path={path} source={source} />}
    <EvidenceDisclosure label="Add an audit note or correction" value={{ path, authority: "User declaration; earlier records remain" }}>
      <AuditNoteForm path={path} />
    </EvidenceDisclosure>
    {filtered.length === 0 && <p>No matching records. This does not establish that the work had no consequential assumptions.</p>}
    {filtered.map((record, index) => <section key={record.id ?? `${record.session_line}:${index}`}>
      <p><strong>{record.entry?.statement ?? record.tool ?? record.path ?? record.state ?? record.kind ?? record.record_kind}</strong>
        {" · "}{record.actor ?? "recorded work"}{record.turn ? ` · turn ${record.turn}` : ""}
        {record.parent ? ` · parent ${record.parent}` : ""}
        <AuditLocationButton path={path} line={record.session_line} label="Open record" /></p>
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
        {record.path && <AuditLocationButton path={record.path!} line={record.first_line ?? 1} label="Compare with current source" />}
      </>}
      {record.record_kind === "verification" && <p>{record.state} · current sources {record.current_state ?? "unchecked"} · revision {record.revision ?? "not recorded"} · {record.scope}</p>}
      {!record.entry && record.record_kind !== "read-snapshot" && <EvidenceDisclosure label={`Recorded details · ${record.id ?? record.session_line}`} value={record}><pre>{JSON.stringify(record, null, 2)}</pre></EvidenceDisclosure>}
    </section>)}
  </>;
}

/** The harness result is visible beside the answer, independently of its prose. */
export function VerificationStanding({ records, turn, running }: { records: AuditRecord[]; turn?: string; running: boolean }) {
  const record = [...records].reverse().find(record => record.record_kind === "verification" && record.turn === turn);
  const state = running ? "pending" : record?.state === "passed" && record.current_state !== "unchanged" ? `historical pass · current sources ${record.current_state ?? "unavailable"}` : record?.state ?? "not recorded";
  const scope = record?.scope ?? "Document execution and expectations only; semantic support is not assessed";
  useUiReading("verification standing", { state, revision: record?.revision ?? null, turn: turn ?? null,
    scope });
  return <p className="verification-standing" role="status">Hickory checks: <strong>{state}</strong>
    {record?.revision && <> · revision <code>{record.revision}</code></>}. {scope}.</p>;
}
</book:entry>

</book:doc>
