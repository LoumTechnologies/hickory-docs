import { useState } from "react";
import type { ExecBlock } from "../api/types";
import { DocumentEmbed } from "./DocumentEmbed";
import { CellPanel } from "../components/CellPanel";
import { sha256 } from "../lib/reviews";

export interface ViewerSnapshot {
  format: "hickory-viewer-snapshot"; version: 1; path: string; source: string;
  source_hash: string; captured_at_ms: number; execution: string; cells: ExecBlock[];
}
/** Validate external snapshots before supplying them to product components. */
export async function decodeSnapshot(value: unknown): Promise<ViewerSnapshot> {
  if (!value || typeof value !== "object") throw new Error("Expected a Hickory viewer snapshot");
  const v = value as ViewerSnapshot;
  if (v.format !== "hickory-viewer-snapshot" || v.version !== 1 || typeof v.source !== "string" || typeof v.path !== "string" || typeof v.execution !== "string" || !Number.isFinite(v.captured_at_ms) || !Array.isArray(v.cells)) throw new Error("Unsupported viewer snapshot");
  if (await sha256(v.source) !== v.source_hash) throw new Error("Snapshot source hash does not match");
  for (const cell of v.cells) {
    if (cell.kind !== "exec" || typeof cell.id !== "string" || typeof cell.container !== "string" || typeof cell.command !== "string" || !Array.isArray(cell.span) || cell.span.length !== 2 || !cell.span.every(Number.isFinite) || cell.status !== undefined && !["ok", "failed", "stale", "unrecorded"].includes(cell.status)) throw new Error("Invalid snapshot cell");
    if (cell.expect && (!["exact", "regex-lines"].includes(cell.expect.match) || typeof cell.expect.body !== "string")) throw new Error("Invalid expectation");
    if (cell.transcript && (!Array.isArray(cell.transcript) || cell.transcript.some(event => !Number.isFinite(event.t) || (event.kind === "exit" ? !Number.isFinite(event.code) : !["cmd", "out", "err"].includes(event.kind) || typeof event.data !== "string")))) throw new Error("Invalid transcript");
  }
  return v;
}
export function PublishedSnapshot({ snapshot }: { snapshot: ViewerSnapshot }) {
  const [replay, setReplay] = useState(true);
  return <main className="published-snapshot">
    <h1>{snapshot.path}</h1>
    <p>Captured {new Date(snapshot.captured_at_ms).toLocaleString()} · historical snapshot</p>
    <p>Source SHA-256: <code>{snapshot.source_hash}</code></p>
    <p>This read-only snapshot displays recorded results. Opening or exporting it does not execute checks. It does not certify the current repository or authenticate the supplied results.</p>
    <DocumentEmbed source={snapshot.source} revision={snapshot.source_hash} path={snapshot.path} readOnly />
    <h2>Recorded cell results</h2>
    <label><input type="checkbox" checked={replay} onChange={event => setReplay(event.target.checked)} /> Show execution transcripts</label>
    {!snapshot.cells.length && <p>No execution results recorded.</p>}
    {snapshot.cells.map((cell, index) => <section key={`${cell.id}:${index}`}><h3>{cell.id} · {cell.status ?? "unrecorded"}</h3><CellPanel block={cell} command={cell.command} replay={replay} /></section>)}
  </main>;
}
