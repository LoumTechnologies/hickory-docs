import { useState } from "react";
import { ComparisonEditor } from "./ComparisonEditor";
import type { ComparisonModel } from "../lib/projectComparison";

export function WorktreeRegionEditor({path,values,writable,shared,onApply}: {
  path:string;values:Record<string,string>;writable:ReadonlySet<string>;shared:boolean;
  onApply:(text:string,target:string,route:"shared"|"just-here")=>Promise<void>;
}) {
  const [draft,setDraft]=useState<ComparisonModel|null>(null);
  const [route,setRoute]=useState<"shared"|"just-here">(shared?"shared":"just-here");
  const [busy,setBusy]=useState(false),[error,setError]=useState("");
  const sources=Object.entries(values).map(([name,text])=>({id:name,label:name,kind:"directory" as const,text,target:writable.has(name)?name:undefined}));
  if(sources.length===0)return null;
  sources.unshift({...sources[0],id:`before:${sources[0].id}`,label:"Captured input",target:undefined});
  const model:ComparisonModel=draft??{sources,old:sources[0].id,new:sources[1].id,policy:writable.has(sources[1].id)?"new":"neither",evidence:"unverified"};
  return <section aria-label="Worktree region edit">
    <label>Write route <select value={route} onChange={e=>setRoute(e.target.value as typeof route)}>
      {shared && <option value="shared">Every target opened for writing</option>}<option value="just-here">Selected New target only</option>
    </select></label>
    <ComparisonEditor path={path} model={model} onChange={next=>setDraft({...next,policy:writable.has(next.new)?"new":"neither"})} />
    <button className="btn" disabled={busy || !draft || !writable.has(model.new)} onClick={()=> {
      setBusy(true);setError("");void onApply(model.sources.find(s=>s.id===model.new)!.text,model.new,route).catch(e=>setError(String(e))).finally(()=>setBusy(false));
    }}>Apply region to selected targets</button>
    {error && <p role="alert">{error}</p>}
  </section>;
}
