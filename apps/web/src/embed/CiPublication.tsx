import { useEffect, useState } from "react";
import { DocumentEmbed } from "./DocumentEmbed";
import { sha256 } from "../lib/reviews";

export interface CiCheck {
  id: string; owner: string; source: string; source_hash: string;
  status: "passed" | "failed" | "blocked" | "timed_out" | "unavailable";
  argv: string[]; needs: string[]; assertion: string; exit_code: number | null;
  elapsed_seconds: number | null; logs: { stdout?: string; stderr?: string };
}
export interface CiPublication {
  format: "hickory-ci-publication"; version: 1; title: string; captured_at: string;
  base_revision: string; tested_revision: string | null; scope: string; checks: CiCheck[];
}
export async function decodeCiPublication(value: unknown): Promise<CiPublication> {
  const v = value as CiPublication;
  if (!v || v.format !== "hickory-ci-publication" || v.version !== 1 || typeof v.title !== "string" || typeof v.scope !== "string" || typeof v.captured_at !== "string" || !/^[a-f0-9]{40}$/.test(v.base_revision) || v.tested_revision !== null && !/^[a-f0-9]{40}$/.test(v.tested_revision) || !Array.isArray(v.checks) || !v.checks.length) throw Error("Unsupported CI publication");
  const ids = new Set<string>();
  for (const c of v.checks) {
    if (!c || typeof c.id !== "string" || !/^[a-z][a-z0-9-]*$/.test(c.id) || ids.has(c.id) || typeof c.owner !== "string" || typeof c.source !== "string" || !/^[a-f0-9]{64}$/.test(c.source_hash) || !["passed", "failed", "blocked", "timed_out", "unavailable"].includes(c.status) || !Array.isArray(c.argv) || c.argv.some(a => typeof a !== "string") || !Array.isArray(c.needs) || c.needs.some(n => typeof n !== "string") || c.assertion !== "exit-zero" || c.exit_code !== null && !Number.isInteger(c.exit_code) || c.elapsed_seconds !== null && (typeof c.elapsed_seconds !== "number" || !Number.isFinite(c.elapsed_seconds) || c.elapsed_seconds < 0) || !c.logs || [c.logs.stdout, c.logs.stderr].some(log => log !== undefined && typeof log !== "string")) throw Error("Invalid CI check");
    if (await sha256(c.source) !== c.source_hash) throw Error("Captured source hash differs");
    ids.add(c.id);
  }
  return v;
}

export function CiPublicationView({ publication }: { publication: CiPublication }) {
  const [selected, setSelected] = useState(publication.checks[0].id);
  const check = publication.checks.find(c => c.id === selected)!;
  const [draft, setDraft] = useState<string | null>(null);
  const [editing, setEditing] = useState(false);
  const passed = publication.checks.filter(c => c.status === "passed").length;
  function choose(id: string) { setSelected(id); setDraft(null); setEditing(false); }
  function download() {
    const url = URL.createObjectURL(new Blob([draft ?? check.source], { type: "text/markdown;charset=utf-8" }));
    const link = document.createElement("a"); link.href = url; link.download = check.owner.split("/").pop() ?? "document.md"; link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  return <div className="ci-publication">
    <header className="ci-masthead"><a href="https://hickorydocs.com">Hickory Docs</a><span>Local execution · static evidence</span></header>
    <section className="ci-intro"><p className="ci-eyebrow">A working experiment</p><h1>{publication.title}</h1><p>Read the requirement, inspect its implementation, and see the checks that ran against it. This page is an artifact of a local run.</p>
      <div className="ci-summary"><strong>{passed} / {publication.checks.length} checks passed</strong><span>Captured {new Date(publication.captured_at).toLocaleString()}</span></div>
      <p className="ci-muted">{publication.tested_revision ? `Tested commit ${publication.tested_revision}` : `Working-tree run based on ${publication.base_revision.slice(0, 12)} · no tested commit associated`}</p>
    </section>
    <div className="ci-workspace"><aside aria-label="Checks"><h2>Checks and their documents</h2>{publication.checks.map(c => <button key={c.id} aria-pressed={selected === c.id} onClick={() => choose(c.id)}><span className={`ci-status ci-status-${c.status}`}>{c.status.replaceAll("_", " ")}</span><strong>{c.id}</strong><small>{c.owner.split("/").slice(-2).join("/")}</small></button>)}<p className="ci-muted">{publication.scope}</p></aside>
    <main><section className="ci-result" aria-label="Check result"><div><span className={`ci-status ci-status-${check.status}`}>{check.status.replaceAll("_", " ")}</span><h2>{check.id}</h2></div><code>{check.argv.join(" ")}</code><p>Assertion: command exits with zero · observed exit {check.exit_code ?? "unavailable"} · {check.elapsed_seconds ?? "—"} seconds</p>
      {check.needs.length > 0 && <p>Depends on: {check.needs.join(", ")}</p>}
      <details><summary>Captured command logs</summary><h3>Standard output</h3><pre>{check.logs.stdout || "No output captured."}</pre><h3>Standard error</h3><pre>{check.logs.stderr || "No error output captured."}</pre></details>
    </section><section aria-label="Owning document"><div className="ci-document-heading"><div><p className="ci-eyebrow">The document behind this check</p><h2>{check.owner.split("/").slice(-2).join("/")}</h2></div><div className="ci-actions"><button onClick={() => { setEditing(!editing); if (editing) setDraft(null); }}>{editing ? "Return to captured source" : "Try editing a local draft"}</button><button onClick={download}>Download {editing ? "draft" : "source"}</button></div></div>
      {editing && <p className="ci-draft" role="status">Local draft · these edits have not been tested or saved. Switching checks discards this draft; download it to keep it.</p>}
      <DocumentEmbed key={`${check.id}:${editing}`} path={check.owner} revision={check.source_hash} source={draft ?? check.source} readOnly={!editing} onChange={change => setDraft(change.source)} />
      <p className="ci-muted ci-hash">Captured source SHA-256: {check.source_hash}</p>
    </section></main></div><footer>Built with Hickory’s document editor. Reading and editing here run no CI commands.</footer>
  </div>;
}

export function CiPublicationPage() {
  const [publication, setPublication] = useState<CiPublication | null>(null);
  const [error, setError] = useState("");
  useEffect(() => {
    const controller = new AbortController();
    void fetch(new URL("ci-preview/bundle.json", location.href), { signal: controller.signal })
      .then(response => { if (!response.ok) throw Error(`Publication unavailable (${response.status})`); return response.json(); })
      .then(decodeCiPublication).then(value => { if (!controller.signal.aborted) setPublication(value); })
      .catch(failure => { if (!controller.signal.aborted) setError(String(failure)); });
    return () => controller.abort();
  }, []);
  if (error) return <main className="ci-loading" role="alert"><h1>Could not open this CI publication</h1><p>{error}</p></main>;
  return publication ? <CiPublicationView publication={publication} /> : <p className="ci-loading" role="status">Loading captured checks and documents…</p>;
}
