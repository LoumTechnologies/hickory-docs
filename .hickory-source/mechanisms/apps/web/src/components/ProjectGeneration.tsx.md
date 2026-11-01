<gencode:doc xmlns:gencode="http://www.hickorydocs.com/1.0">
# Choose, preview and generate saved project files

For: maintainers changing safe document opening and explicit generation.

Opening the dialog makes no request. Editing either target field invalidates the captured plan; source files are read only after Preview.

<gencode:copy id="generation-dialog-state">import { useRef, useState } from "react";
import { createPortal } from "react-dom";
import { request } from "../api/client";

type Plan = { root: string; entry: string; revision: string; outputs: string[] };

/** Opening this dialog reads nothing. Only Preview captures a generation context. */
export function ProjectGenerationDialog({ onClose }: { onClose: () => void }) {
  const [root, setRoot] = useState(".");
  const [entry, setEntry] = useState("project.md");
  const [plan, setPlan] = useState<{ target: { root: string; entry: string }; captured: Plan } | null>(null);
  const [busy, setBusy] = useState(false);
  const active = useRef(false);
  const [error, setError] = useState("");
  const [result, setResult] = useState("");
  const clear = () => { setPlan(null); setError(""); setResult(""); };
</gencode:copy>

Keep the captured root/entry request with its revision. Generate uses that exact plan, never a newer input value or an automatic retry after refusal.

<gencode:copy id="generation-dialog-actions">  async function act(generate: boolean) {
    if (active.current || (generate && !plan)) return;
    active.current = true; setBusy(true); setError(""); setResult("");
    try {
      if (generate && plan) {
        const outcome = await request<{ written: string[] }>("POST", "/api/generation/generate", { ...plan.target, revision: plan.captured.revision });
        setResult(`Generated ${outcome.written.length} ${outcome.written.length === 1 ? "file" : "files"}.`); setPlan(null);
      } else {
        const target = { root, entry };
        const captured = await request<Plan>("POST", "/api/generation/preview", target);
        setPlan({ target, captured });
      }
    } catch (cause) {
      setPlan(null); setError(cause instanceof Error ? cause.message : String(cause));
    } finally { active.current = false; setBusy(false); }
  }
</gencode:copy>

Disclose the canonical root, entry and output paths before offering the separate Generate action. Explain that generation reads saved files.

<gencode:copy id="generation-dialog-review">  return createPortal(<dialog open aria-label="Generate project files" style={{ position: "fixed", top: "15%", maxWidth: "36rem", zIndex: 1000 }}>
    <h2>Generate project files</h2>
    <p>Uses saved files. Save your edits first. Opening documents never generates files.</p>
    <label>Project root (relative to the opened location)<input value={root} disabled={busy} onChange={e => { clear(); setRoot(e.target.value); }} /></label>
    <label>Entry document (relative to the project root)<input value={entry} disabled={busy} onChange={e => { clear(); setEntry(e.target.value); }} /></label>
    <p><button disabled={busy || !root || !entry} onClick={() => void act(false)}>Preview generation</button></p>
    {plan && <section aria-label="Generation preview">
      <p>Root: <code>{plan.captured.root}</code></p><p>Entry: <code>{plan.captured.entry}</code></p>
      <details><summary>{plan.captured.outputs.length} output files</summary><ul style={{ maxHeight: "16rem", overflow: "auto" }}>{plan.captured.outputs.map(path => <li key={path}>{path}</li>)}</ul></details>
      <button disabled={busy} onClick={() => void act(true)}>Generate {plan.captured.outputs.length} {plan.captured.outputs.length === 1 ? "file" : "files"}</button>
    </section>}
    {error && <p role="alert">{error}</p>}{result && <p role="status">{result}</p>}
    <p><button disabled={busy} onClick={onClose}>Close</button></p>
  </dialog>, document.body);
}
</gencode:copy>

<gencode:file path="apps/web/src/components/ProjectGeneration.tsx"><gencode:paste select="#generation-dialog-state"/><gencode:paste select="#generation-dialog-actions"/><gencode:paste select="#generation-dialog-review"/></gencode:file>
</gencode:doc>
