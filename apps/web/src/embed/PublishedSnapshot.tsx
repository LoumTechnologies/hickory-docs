import { useEffect, useMemo, useRef, useState } from "react";
import type { ExecBlock } from "../api/types";
import { DocumentEmbed } from "./DocumentEmbed";
import { CellPanel } from "../components/CellPanel";
import { sha256 } from "../lib/reviews";

export interface Verification {
  status: "passed" | "failed" | "not_run" | "unavailable" | "stale";
  revision: string; source_hash: string; dependencies: Record<string, string>; assets: Record<string, string>;
  environment: Record<string, unknown>; commands: { command: string[]; exit_code?: number; stdout?: string; stderr?: string; error?: string }[];
  execution_evidence?: { from_document: string[]; recording_keys: unknown[]; coverage: string } | null;
  requirements: { id: string; text: string; witnesses: number[] }[]; origin: string; scope: string;
}
export interface ViewerSnapshot {
  demonstration?: { capture_kind: "native-provider"; captured_revision: string; recording_asset: string; session_asset?: string; cuts?: string[] };
  format: "hickory-viewer-snapshot"; version: 1; path: string; source: string;
  verification?: Verification;
  source_hash: string; captured_at_ms: number; execution: string; cells: ExecBlock[];
}
/** Validate external snapshots before supplying them to product components. */
export async function decodeSnapshot(value: unknown): Promise<ViewerSnapshot> {
  if (!value || typeof value !== "object") throw new Error("Expected a Hickory viewer snapshot");
  const v = value as ViewerSnapshot;
  if (v.format !== "hickory-viewer-snapshot" || v.version !== 1 || typeof v.source !== "string" || typeof v.path !== "string" || typeof v.execution !== "string" || !Number.isFinite(v.captured_at_ms) || !Array.isArray(v.cells)) throw new Error("Unsupported viewer snapshot");
  if (await sha256(v.source) !== v.source_hash) throw new Error("Snapshot source hash does not match");
  if (v.demonstration) {
    const demo = v.demonstration;
    if (demo.capture_kind !== "native-provider" || !/^[a-f0-9]{40}$/.test(demo.captured_revision) || typeof demo.recording_asset !== "string" || !/\.(mp4|webm)$/.test(demo.recording_asset) || !v.verification?.assets?.[demo.recording_asset] || demo.session_asset !== undefined && !v.verification?.assets?.[demo.session_asset] || demo.cuts !== undefined && (!Array.isArray(demo.cuts) || demo.cuts.some(note => typeof note !== "string"))) throw Error("Invalid reviewed demonstration assets");
  }
  if (v.verification) {
    const check = v.verification;
    if (!["passed", "failed", "not_run", "unavailable", "stale"].includes(check.status) || !/^[a-f0-9]{40}$/.test(check.revision) || check.source_hash !== v.source_hash || !Array.isArray(check.commands) || !Array.isArray(check.requirements) || typeof check.origin !== "string" || typeof check.scope !== "string" || !check.dependencies || !check.environment) throw new Error("Invalid revision-bound verification");
    for (const requirement of check.requirements) if (typeof requirement.id !== "string" || typeof requirement.text !== "string" || !Array.isArray(requirement.witnesses) || requirement.witnesses.some(i => !Number.isInteger(i) || i < 0)) throw new Error("Invalid requirement witness");
    for (const command of check.commands) if (!Array.isArray(command.command) || command.command.some(arg => typeof arg !== "string") || command.stdout !== undefined && typeof command.stdout !== "string" || command.stderr !== undefined && typeof command.stderr !== "string" || command.error !== undefined && typeof command.error !== "string") throw new Error("Invalid verification command");
  }
  for (const cell of v.cells) {
    if (cell.kind !== "exec" || typeof cell.id !== "string" || typeof cell.container !== "string" || typeof cell.command !== "string" || !Array.isArray(cell.span) || cell.span.length !== 2 || !cell.span.every(Number.isFinite) || cell.status !== undefined && !["ok", "failed", "stale", "unrecorded"].includes(cell.status)) throw new Error("Invalid snapshot cell");
    if (cell.expect && (!["exact", "regex-lines"].includes(cell.expect.match) || typeof cell.expect.body !== "string")) throw new Error("Invalid expectation");
    if (cell.transcript && (!Array.isArray(cell.transcript) || cell.transcript.some(event => !Number.isFinite(event.t) || (event.kind === "exit" ? !Number.isFinite(event.code) : !["cmd", "out", "err"].includes(event.kind) || typeof event.data !== "string")))) throw new Error("Invalid transcript");
  }
  return v;
}
/** Shared by the snapshot renderer and homepage; publication selection owns assets. */
export async function verifiedPublicationAsset(snapshot: ViewerSnapshot, artifactUrl: string, key: string, signal?: AbortSignal): Promise<string> {
  const expected = snapshot.verification?.assets?.[key];
  const unsafe = !key || /^[a-z][a-z\d+.-]*:/i.test(key) || /[\\?#]/.test(key) || key.startsWith("/") || key.split("/").some(part => {
    try { const decoded = decodeURIComponent(part); return decoded === "." || decoded === ".." || /[\\/]/.test(decoded); } catch { return true; }
  });
  if (!expected || !/^[a-f0-9]{64}$/.test(expected) || unsafe) throw Error("Asset not selected for publication");
  const base = new URL("./", artifactUrl);
  const url = new URL(key, base);
  if (url.origin !== location.origin || !url.pathname.startsWith(base.pathname)) throw Error("External asset origin refused");
  const response = await fetch(url, { signal });
  if (!response.ok) throw Error("Published asset unavailable");
  const bytes = await response.arrayBuffer();
  const hash = Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256", bytes)), b => b.toString(16).padStart(2, "0")).join("");
  if (hash !== expected) throw Error("Published asset hash mismatch");
  return URL.createObjectURL(new Blob([bytes], { type: response.headers.get("Content-Type") ?? "application/octet-stream" }));
}
export function PublishedSnapshot({ snapshot, artifactUrl }: { snapshot: ViewerSnapshot; artifactUrl?: string }) {
  const assetUrls = useRef(new Set<string>());
  useEffect(() => () => { for (const url of assetUrls.current) URL.revokeObjectURL(url); assetUrls.current.clear(); }, [snapshot, artifactUrl]);
  const host = useMemo(() => ({ resolveAsset: async (path: string, signal: AbortSignal) => {
    if (!artifactUrl || !snapshot.verification) throw Error("No published asset manifest available");
    const sourcePath = new URL(snapshot.path, "https://publication.invalid/");
    const relative = new URL(path, sourcePath);
    if (relative.origin !== sourcePath.origin) throw Error("External assets are not approved");
    const key = decodeURIComponent(relative.pathname.slice(1));
    const immutable = await verifiedPublicationAsset(snapshot, artifactUrl, key, signal);
    assetUrls.current.add(immutable);
    return immutable;
  } }), [snapshot, artifactUrl]);
  const [replay, setReplay] = useState(true);
  return <main className="published-snapshot">
    <h1>{snapshot.path}</h1>
    <p>Captured {new Date(snapshot.captured_at_ms).toLocaleString()} · historical snapshot</p>
    <p>Source SHA-256: <code>{snapshot.source_hash}</code></p>
    <p>This read-only snapshot displays recorded results. Opening or exporting it does not execute checks. It does not certify the current repository or authenticate the supplied results.</p>
    {snapshot.verification && <section aria-label="Revision-bound verification">
      <h2>Verification: {snapshot.verification.status}</h2>
      <a href={`https://github.com/LoumTechnologies/hickory-docs/commit/${snapshot.verification.revision}`}>Tested revision {snapshot.verification.revision}</a>
      <p>{snapshot.verification.origin}</p>
      <details><summary>Recording/replay evidence</summary><pre>{JSON.stringify(snapshot.verification.execution_evidence ?? "Recording origin unavailable", null, 2)}</pre></details><p>{snapshot.verification.scope}</p>
      <p>Historical success certifies only these source/dependency hashes; changed inputs need a new run.</p>
      <details><summary>Environment and dependency identities</summary><pre>{JSON.stringify({ environment: snapshot.verification.environment, dependencies: snapshot.verification.dependencies, assets: snapshot.verification.assets }, null, 2)}</pre></details>
      {snapshot.verification.requirements.map((r, n) => <section key={`${r.id}:${n}`} id={`requirement-${n}`}><h3>{r.id}</h3><p>{r.text}</p>{r.witnesses.map(i => snapshot.cells[i] ? <a key={i} href={`#witness-${i}`}>Witness {snapshot.cells[i].id} </a> : <p key={i}>Witness {i} unavailable · not verified</p>)}</section>)}
      {snapshot.verification.commands.map((c, i) => <details key={i} open={snapshot.verification!.status !== "passed"}><summary>{c.command.join(" ")} · {c.exit_code === undefined ? "unavailable" : `exit ${c.exit_code}`}</summary><pre>{c.stdout}</pre><pre>{c.stderr ?? c.error}</pre></details>)}
    </section>}
    <DocumentEmbed source={snapshot.source} revision={snapshot.source_hash} path={snapshot.path} host={host} readOnly />
    <h2>Recorded cell results</h2>
    <label><input type="checkbox" checked={replay} onChange={event => setReplay(event.target.checked)} /> Show execution transcripts</label>
    {!snapshot.cells.length && <p>No execution results recorded.</p>}
    {snapshot.cells.map((cell, index) => <section key={`${cell.id}:${index}`} id={`witness-${index}`}><h3>{cell.id} · {cell.status ?? "unrecorded"}</h3><CellPanel block={cell} command={cell.command} replay={replay} /></section>)}
  </main>;
}
