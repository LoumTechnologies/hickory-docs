import { useEffect, useRef, useState } from "react";
import type { EditorView } from "@codemirror/view";
import type { RuntimeRun } from "./runtimeObservations";
import { profileRows } from "./profileRows";
import { runtimeReading } from "./runtimeHistory";
import { selectObservations } from "./observations";
import { byteToChar } from "../lib/offsets";
import { revealRuntimeEvent } from "./runtimeNavigation";
export function ProfileEvidence({ run, view, stale }: { run: RuntimeRun; view?: EditorView | null; stale: boolean }) {
  const [expanded, setExpanded] = useState(false), [pinned, setPinned] = useState(false);
  const owner = useRef(Symbol("profile-context"));
  const rows = profileRows(run.events);
  useEffect(() => {
    const reading = runtimeReading({ ...run, events: run.events.filter(e => e.observation.kind === "profile") });
    selectObservations(reading, expanded || pinned, stale || run.status !== "running", owner.current);
    return () => selectObservations(reading, false, false, owner.current);
  }, [run, expanded, pinned, stale]);
  if (!rows.length) return null;
  return <section aria-label="Function profile evidence"><label><input type="checkbox" checked={pinned} onChange={e => setPinned(e.target.checked)} />Pin profile evidence in next context</label>
    <details onToggle={e => setExpanded(e.currentTarget.open)}><summary>Function hotspots · {rows.length} aggregates · {stale ? "source changed" : "captured revision"}</summary>
    <p>Self seconds exclude subcalls; cumulative seconds include subcalls and overlap. Total calls include recursive calls; primitive calls exclude recursive reentry. Main-thread interval only. These are function aggregates, not per-line costs. Timing varies and includes profiler overhead; source mapping precision is separate.</p>
    <table><thead><tr><th>Function</th><th>Calls / primitive</th><th>Self seconds</th><th>Cumulative seconds</th><th>Source and evidence</th></tr></thead><tbody>{rows.map(row => <tr key={row.events[0].id} data-profile-evidence={row.events.map(e => e.id).join(" ")}>
      <td>{row.function.name}</td><td>{row.function.total_calls} / {row.function.primitive_calls}</td><td>{row.self ?? "unavailable"}</td><td>{row.cumulative ?? "unavailable"}</td>
      <td>{row.events[0].emission ? <button disabled={stale} onClick={() => {
        const event = row.events[0]; const at = event.emission?.bytes?.start;
        if (view && at !== undefined) { view.dispatch({ selection: { anchor: byteToChar(view.state.doc.toString(), at) }, scrollIntoView: true }); view.focus(); }
        revealRuntimeEvent(run.id, event.id);
      }}>Source · {row.events[0].id}</button> : `Unresolved/native: ${row.function.file}`}</td>
    </tr>)}</tbody></table></details></section>;
}
