import { useEffect, useId, useMemo, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { EditorView } from "@codemirror/view";
import type { RuntimeRun } from "./runtimeObservations";
import { profileRows } from "./profileRows";
import { runtimeReading } from "./runtimeHistory";
import { selectObservations } from "./observations";
import { byteToChar } from "../lib/offsets";
import { revealRuntimeEvent } from "./runtimeNavigation";
type Props = { run: RuntimeRun; view?: EditorView | null; stale: boolean };
export function ProfileEvidence(props: Props) {
  return <CapturedProfileEvidence key={JSON.stringify([props.run.id, props.run.sources.map(source => [source.id, source.revision])])} {...props} />;
}
function CapturedProfileEvidence({ run, view, stale }: Props) {
  const [expanded, setExpanded] = useState(false), [pinned, setPinned] = useState(false);
  const owner = useRef(Symbol("profile-context"));
  const rows = profileRows(run.events);
  useEffect(() => {
    const reading = runtimeReading({ ...run, events: run.events.filter(e => e.observation.kind === "profile") });
    selectObservations(reading, expanded || pinned, stale || run.status !== "running", owner.current);
    return () => selectObservations(reading, false, false, owner.current);
  }, [run, expanded, pinned, stale]);
  const parent = `profile-evidence:${useId()}`;
  const editor = useMemo(() => crypto.randomUUID(), [view]);
  const revision = useUiRevision(() => [editor, run, stale, expanded, pinned, view?.state.doc.toString()]);
  const navigate = (event: RuntimeRun["events"][number]) => {
    if (stale) throw Error("The displayed source changed; inspect the captured runtime evidence instead");
    const at = event.emission?.bytes?.start;
    const source = run.sources.find(source => source.id === event.emission?.source);
    const matches = !!view && !!source && view.state.doc.toString() === new TextDecoder().decode(new Uint8Array(source.bytes));
    if (view && matches && at !== undefined) { view.dispatch({ selection: { anchor: byteToChar(view.state.doc.toString(), at) }, scrollIntoView: true }); view.focus(); }
    revealRuntimeEvent(run.id, event.id);
    return { recorded_event_requested: true, source_selection_changed: matches && at !== undefined, current_source_verified: false };
  };
  useUiReading("profile evidence status", { run: run.id, aggregates: rows.length, expanded, pinned, stale,
    scope: "Function aggregates from a recorded profile, not per-line costs; timing includes overhead. Pinning selects existing evidence for the next context, not a delivery receipt." });
  useUiCommands(rows.length ? [
    { key: `${parent}:expand`, command: { label: "Set profile evidence visibility", parent, subject: parent, parameters: { open: { type: "boolean" } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true }), execute: async args => { setExpanded(args.open as boolean); return { status: "completed", value: { expanded_requested: args.open } }; } } },
    { key: `${parent}:pin`, command: { label: "Set profile evidence context pin", parent, subject: parent, parameters: { pinned: { type: "boolean" } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true }), execute: async args => { setPinned(args.pinned as boolean); return { status: "completed", value: { pin_requested: args.pinned, context_delivered: false } }; } } },
    ...(expanded ? rows.filter(row => row.events[0].emission).map(row => ({ key: `${parent}:${row.events[0].id}`, command: {
      label: `Open profile source ${row.events[0].id}`, parent, subject: parent, parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !stale, ...(stale ? { reason: "The displayed source changed" } : {}) }),
      execute: async () => ({ status: "completed" as const, value: navigate(row.events[0]) }),
    } })) : []),
  ] : []);
  if (!rows.length) return null;
  return <section aria-label="Function profile evidence"><label><input type="checkbox" checked={pinned} onChange={e => setPinned(e.target.checked)} />Pin profile evidence in next context</label>
    <details open={expanded} onToggle={e => setExpanded(e.currentTarget.open)}><summary>Function hotspots · {rows.length} aggregates · {stale ? "source changed" : "captured revision"}</summary>
    <p>Self seconds exclude subcalls; cumulative seconds include subcalls and overlap. Total calls include recursive calls; primitive calls exclude recursive reentry. Main-thread interval only. These are function aggregates, not per-line costs. Timing varies and includes profiler overhead; source mapping precision is separate.</p>
    {expanded && <ProfileTableReading rows={rows} />}
    <table><thead><tr><th>Function</th><th>Calls / primitive</th><th>Self seconds</th><th>Cumulative seconds</th><th>Source and evidence</th></tr></thead><tbody>{rows.map(row => <tr key={row.events[0].id} data-profile-evidence={row.events.map(e => e.id).join(" ")}>
      <td>{row.function.name}</td><td>{row.function.total_calls} / {row.function.primitive_calls}</td><td>{row.self ?? "unavailable"}</td><td>{row.cumulative ?? "unavailable"}</td>
      <td>{row.events[0].emission ? <button disabled={stale} onClick={() => navigate(row.events[0])}>Source · {row.events[0].id}</button> : `Unresolved/native: ${row.function.file}`}</td>
    </tr>)}</tbody></table></details></section>;
}

function ProfileTableReading({ rows }: { rows: ReturnType<typeof profileRows> }) {
  useUiReading("disclosed profile aggregates", rows.map(row => ({ function: row.function.name,
    total_calls: row.function.total_calls, primitive_calls: row.function.primitive_calls,
    self_seconds: row.self, cumulative_seconds: row.cumulative, event: row.events[0].id,
    source: row.events[0].emission ? "recorded source navigation" : `Unresolved/native: ${row.function.file}` })));
  return null;
}
