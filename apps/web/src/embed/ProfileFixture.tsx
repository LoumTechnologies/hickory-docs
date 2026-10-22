// Explicit recorded-profile host. Importing a record never launches its program.
import { useEffect, useRef, useState } from "react";
import type { EditorView } from "@codemirror/view";
import { DocumentEmbed } from "./DocumentEmbed";
import { RuntimeHistory } from "../debug/RuntimeRuns";
import { observationContext } from "../debug/observations";
import type { RuntimeRun } from "../debug/runtimeObservations";
import { decodeRuntimeRun } from "../debug/decodeRuntimeRun";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
export function ProfileFixture() {
  const [run, setRun] = useState<RuntimeRun | null>(null), [source, setSource] = useState("");
  const [view, setView] = useState<EditorView | null>(null), [context, setContext] = useState("");
  const [error, setError] = useState(""), [busy, setBusy] = useState(false);
  const owner = useRef(0);
  const editorOwner = useRef({ view, generation: 0 });
  if (editorOwner.current.view !== view) editorOwner.current = { view, generation: editorOwner.current.generation + 1 };
  useEffect(() => () => { owner.current++; }, []);
  const revision = useUiRevision(() => [run, source, editorOwner.current.generation, context, busy, owner.current]);
  useUiReading("retained profile import status", { run: run?.id ?? null, source, context, busy, error, execution: "recorded import only", current_certification: false });
  const visible = (commands: { key: string; command: import("../lib/uiActions").UiCommand }[]) => run ? commands : commands.filter(command => command.key === "profile-fixture:import");
  const invoke = useUiCommands(visible([
    { key: "profile-fixture:import", command: { label: "Import retained profile JSON", parent: "profile-fixture", subject: "profile-fixture:record", parameters: { json: { type: "string", maxLength: 2 * 1024 * 1024 } }, prerequisites: ["Explicit captured runtime record; source bytes and hash must match. Producer/execution are not authenticated"], read: () => ({ revision: revision(), enabled: !busy }), execute: async args => {
      const captured = ++owner.current; setBusy(true); setError("");
      try { const record = await decodeRuntimeRun(String(args.json)); if (captured !== owner.current) return { status: "unavailable", reason: "A newer profile import superseded this read" }; setRun(record); setSource(new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(new Uint8Array(record.sources[0].bytes))); setContext(""); return { status: "completed", value: { loaded: true, historical: true, executed: false, current_certification: false } }; }
      catch (reason) { if (captured === owner.current) setError(String(reason)); return { status: "unavailable", reason: String(reason) }; }
      finally { if (captured === owner.current) setBusy(false); }
    } } },
    { key: "profile-fixture:context", command: { label: "Inspect selected profile context", parent: "profile-fixture", subject: "profile-fixture:context", parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !!run && !busy }), execute: async () => { const reading = JSON.stringify(observationContext(run!.sources[0].uri, source)); setContext(reading); return { status: "completed", value: { context: reading, recorded: true } }; } } },
    { key: "profile-fixture:edit", command: { label: "Edit after captured profile", parent: "profile-fixture", subject: "profile-fixture:record", parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !!run && !busy }), execute: async () => { setSource(source + "\n# edited after capture"); return { status: "completed", value: { edited: true, capture_retained: true, executed: false } }; } } },
  ]));
  const act = (key: string, args = {}) => { const captured = owner.current; void invoke(`profile-fixture:${key}`, args).catch(reason => { if (owner.current === captured) setError(String(reason)); }); };
  return <main><h1>Retained profile replay</h1><p>This browser does not execute Python. Load an actual saved native capture; source and metrics remain captured evidence. Pin/remove affect this page only.</p>
    {error && <p role="alert">{error}</p>}
    <label>Retained profile record<input type="file" accept=".json" disabled={busy} onChange={event => {
      const file = event.target.files?.[0]; if (!file) return;
      const captured = ++owner.current;
      if (file.size > 2 * 1024 * 1024) { setError("Recorded runtime import exceeds 2 MiB"); return; }
      void file.text().then(json => { if (captured === owner.current) act("import", { json }); }).catch(reason => { if (captured === owner.current) setError(String(reason)); });
    }} /></label>
    {run && <><DocumentEmbed path={run.sources[0].uri} revision={source} source={source} onChange={event => { owner.current++; setBusy(false); setSource(event.source); }} onViewReady={setView} />
      <RuntimeHistory runs={[run]} source={source} view={view} onPin={async selected => { setRun(current => current === selected ? { ...current, pinned: !current.pinned } : current); }} onDelete={async selected => { if (run === selected) { owner.current++; setBusy(false); setRun(null); setSource(""); setContext(""); } }} />
      <button disabled={busy} onClick={() => act("context")}>Inspect selected profile context</button>
      <output aria-label="Selected profile context">{context}</output>
      <button disabled={busy} onClick={() => act("edit")}>Edit after captured profile</button></>}
  </main>;
}
