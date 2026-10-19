// Acceptance host for the product's retained-profile view, explicitly replay only.
import { useState } from "react";
import type { EditorView } from "@codemirror/view";
import { DocumentEmbed } from "./DocumentEmbed";
import { RuntimeHistory } from "../debug/RuntimeRuns";
import { observationContext } from "../debug/observations";
import type { RuntimeRun } from "../debug/runtimeObservations";
export function ProfileFixture() {
  const [run, setRun] = useState<RuntimeRun | null>(null), [source, setSource] = useState("");
  const [view, setView] = useState<EditorView | null>(null), [context, setContext] = useState("");
  return <main><h1>Retained profile replay</h1><p>This browser does not execute Python. Load an actual saved native capture; source and metrics remain captured evidence.</p>
    <label>Retained profile record<input type="file" accept=".json" onChange={async e => { const file = e.target.files?.[0]; if (file) { const record = JSON.parse(await file.text()) as RuntimeRun; setRun(record); setSource(new TextDecoder().decode(new Uint8Array(record.sources[0].bytes))); } }} /></label>
    {run && <><DocumentEmbed path={run.sources[0].uri} revision={source} source={source} onChange={e => setSource(e.source)} onViewReady={setView} />
      <RuntimeHistory runs={[run]} source={source} view={view} onPin={async () => undefined} onDelete={async () => undefined} />
      <button onClick={() => setContext(JSON.stringify(observationContext(run.sources[0].uri, source)))}>Inspect selected profile context</button>
      <output aria-label="Selected profile context">{context}</output>
      <button onClick={() => setSource(source + "\n# edited after capture")}>Edit after captured profile</button></>}
  </main>;
}
