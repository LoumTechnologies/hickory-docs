import { useState } from "react";
import { request } from "../api/client";
import { parseHickDoc } from "../editor/hickDoc";
import { materializeProject, type EvaluatedProject } from "../editor/hickLang";

export function ProjectContextSelector({ doc, source, value, onChange, disabled }: {
  doc: string; source: string; value: string; onChange: (selection: string) => void; disabled: boolean;
}) {
  const [project,setProject] = useState<EvaluatedProject | null>(null);
  const [error,setError] = useState("");
  if (!parseHickDoc(source).blocks.some(block => block.name === "project")) return null;
  const inspect = async () => {
    try {
      const context = await request<{entry:string;sources:Record<string,string>}>("POST",`/api/docs/${encodeURIComponent(doc)}/project`,{source});
      setProject(materializeProject(context.entry,context.sources)); setError("");
    } catch (e) {setError(e instanceof Error ? e.message : String(e));}
  };
  return <div className="project-context-selector">
    <label>Project tools <select aria-label="Project tools context" value={value} disabled={disabled}
      onFocus={() => void inspect()} onChange={e => onChange(e.target.value)}>
      <option value="new">New · completed project</option><option value="old">Old · scope input</option>
      {project && <optgroup label="Explicit atomic scopes">{Object.keys(project.scopes).flatMap(scope => [
        <option key={`new:${scope}`} value={`scope-new:${scope}`}>{scope} · completed scope</option>,
        <option key={`old:${scope}`} value={`scope-old:${scope}`}>{scope} · scope input</option>,
      ])}</optgroup>}
      {project && <optgroup label="Advanced · intermediate states">{Object.keys(project.local_states).flatMap(id => [
        <option key={`before:${id}`} value={`local-before:${id}`}>Before {id} · intermediate</option>,
        <option key={`after:${id}`} value={`local-after:${id}`}>After {id} · intermediate</option>,
      ])}</optgroup>}
    </select></label>
    {value === "old" && (!project || project.old.length === 0) && <span className="muted">Old is empty; there is no program to launch.</span>}
    {error && <p role="alert">{error}</p>}
  </div>;
}
