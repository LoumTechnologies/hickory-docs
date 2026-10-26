<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/ProfileEvidence.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9b75394c3f2e"></a>
## `apps/web/src/debug/ProfileEvidence.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Captured reading surfaces share owned native actions](../../../../../contract.md).

<a id="f-9b75394c3f2e-Props"></a>
### `Props`

<storycode:copy id="f-9b75394c3f2e-Props">
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
</storycode:copy>
<a id="f-9b75394c3f2e-ProfileEvidence"></a>
### `ProfileEvidence`

<storycode:copy id="f-9b75394c3f2e-ProfileEvidence">
export function ProfileEvidence(props: Props) {
  return <CapturedProfileEvidence key={JSON.stringify([props.run.id, props.run.sources.map(source => [source.id, source.revision])])} {...props} />;
}
</storycode:copy>
<a id="f-9b75394c3f2e-CapturedProfileEvidence"></a>
### `CapturedProfileEvidence`

<storycode:copy id="f-9b75394c3f2e-CapturedProfileEvidence">
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

</storycode:copy>
<a id="f-9b75394c3f2e-ProfileTableReading"></a>
### `ProfileTableReading`

<storycode:copy id="f-9b75394c3f2e-ProfileTableReading">
function ProfileTableReading({ rows }: { rows: ReturnType<typeof profileRows> }) {
  useUiReading("disclosed profile aggregates", rows.map(row => ({ function: row.function.name,
    total_calls: row.function.total_calls, primitive_calls: row.function.primitive_calls,
    self_seconds: row.self, cumulative_seconds: row.cumulative, event: row.events[0].id,
    source: row.events[0].emission ? "recorded source navigation" : `Unresolved/native: ${row.function.file}` })));
  return null;
}
</storycode:copy>

### Assemble `apps/web/src/debug/ProfileEvidence.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/ProfileEvidence.tsx"><storycode:paste select="#f-9b75394c3f2e-Props"/><storycode:paste select="#f-9b75394c3f2e-ProfileEvidence"/><storycode:paste select="#f-9b75394c3f2e-CapturedProfileEvidence"/><storycode:paste select="#f-9b75394c3f2e-ProfileTableReading"/></storycode:file>

</storycode:doc>
