import { useEffect, useState } from "react";
import { api } from "../api/client";
import type { ChecksSnapshot } from "../api/checks";

export function ChecksPane() {
  const [snapshot, setSnapshot] = useState<ChecksSnapshot | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [stale, setStale] = useState(false);
  useEffect(() => {
    let live = true;
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      try {
        const next = await api.checks();
        if (!live) return;
        if (next.availability === "unavailable") {
          setMessage(next.message); setStale(true);
        } else {
          setSnapshot(next); setMessage(next.message); setStale(false);
        }
      } catch (error) {
        if (!live) return;
        setStale(true);
        setMessage(`${error instanceof Error ? error.message : String(error)}. Check the configured checks provider and retry.`);
      }
      if (live) timer = setTimeout(poll, 3000);
    };
    void poll();
    return () => { live = false; clearTimeout(timer); };
  }, []);
  return <section aria-label="Checks" style={{ padding: "0.75rem", overflow: "auto", height: "100%", fontSize: "0.84rem" }}>
    <h2 style={{ fontSize: "1rem", margin: "0 0 0.5rem" }}>Checks</h2>
    <p className="muted">Recent runs for this folder · updates every 3 seconds</p>
    {message && <p role={stale ? "alert" : "status"}>{message}</p>}
    {stale && <p role="status">Status unavailable. Last results below may be stale; retrying automatically.</p>}
    {!snapshot && !stale && <p role="status">Reading checks…</p>}
    {snapshot?.availability === "ready" && snapshot.runs.length === 0 && <p>No recent checks for this folder.</p>}
    {!!snapshot?.runs.length && <table style={{ width: "100%", textAlign: "left" }}>
      <thead><tr><th>Check / run</th><th>Status</th><th>Source</th><th>Started</th></tr></thead>
      <tbody>{snapshot.runs.map(run => <tr key={run.id}>
        <td><div>{run.name || run.id}</div><small className="muted">{run.id}{run.parent ? ` · child of ${run.parent}` : ""}</small></td>
        <td style={{ whiteSpace: "nowrap" }}>{run.status.replaceAll("_", " ")}{run.code !== null ? ` (${run.code})` : ""}</td>
        <td>{run.revision ? <code title={run.revision}>{run.revision.slice(0, 12)}</code> : "Workspace (no immutable revision)"}</td>
        <td>{run.created_ms ? new Date(run.created_ms).toLocaleString() : "Unknown"}</td>
      </tr>)}</tbody>
    </table>}
  </section>;
}
