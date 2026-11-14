<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Check bounded analysis and stale evidence

For maintainers changing workspace services independently of the editor renderer.

Hold the first pair of outline reads at an explicit gate. Stopping must admit no third source, and another run must be refused until that pair drains. A separate run edits one loaded draft while its outline is pending and fails another provider read. The model must retire the stale locations, retain readable source, finish independent work and expose each failure without claiming verification.

<ui:copy id="simple-workspace-analysis-browser-tests">
import { describe, it, expect } from "vitest";
import { WorkspaceAnalysis } from "./WorkspaceAnalysis";
const files = Array.from({length:5},(_,i)=>({name:`${i}.py`,path:`${i}.py`,dir:false,language:"python"}));
function gate() { let release!:()=>void; const promise = new Promise<void>(resolve=>release=resolve); return {promise,release}; }
describe("workspace analysis admission",()=>{
  it("drains two reads before accepting a restart",async()=>{
    const model=new WorkspaceAnalysis(), entered=gate(), held=gate();let loads=0,active=0,peak=0;
    const run=model.run(files,async()=>{loads++;return {text:"source"}},async()=>{active++;peak=Math.max(peak,active);if(active===2) entered.release();await held.promise;active--;return []});
    await entered.promise;model.stop();expect(model.running).toBe(true);
    expect(await model.run(files,async()=>{throw Error("must not load")},async()=>[])).toBe(false);
    held.release();await run;expect(loads).toBe(2);expect(peak).toBe(2);expect(model.completed).toBe(2);expect(model.running).toBe(false);
  });
  it("rejects stale source and retains per-file failures",async()=>{
    const model=new WorkspaceAnalysis(),buffer={text:"old"};let reads=0;
    await model.run(files,async file=>file.path==="0.py"?buffer:{text:"source"},async file=>{reads++;if(file.path==="0.py")buffer.text="draft";if(file.path==="1.py")throw Error("provider unavailable");return []});
    expect(reads).toBe(5);expect(model.completed).toBe(5);expect(buffer.text).toBe("draft");expect(model.knowledge.get("0.py")?.state).toBe("stale");expect(model.current("0.py","old")).toBe(null);
    expect(model.knowledge.get("1.py")?.notice).toBe("provider unavailable");expect(model.current("2.py","source")).not.toBe(null);expect(model.current("2.py","changed")).toBe(null);
  });
});
</ui:copy>

<ui:file path="apps/web/src/simple/WorkspaceAnalysis.test.ts"><ui:paste select="#simple-workspace-analysis-browser-tests"/></ui:file>
</ui:doc>
