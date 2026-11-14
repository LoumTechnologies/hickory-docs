<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retain source test observations on browser

For maintainers replacing the test UI without replacing source and execution admission.

Keep source snapshots and provider observations as distinct values. Unknown source mappings stay absent; target runtime locations remain available without guessing an authored owner.

<ui:copy id="test-sources-browser-values">
import type { SourceTestRequest, SourceTestScan, SourceTestCandidate, SourceTestRunRequest } from "../api/testSources";
/** Retain supplied drafts, separately from saved bytes that admission checks later. */
export interface TestSourceSnapshot { workspace: string; request: SourceTestRequest }
</ui:copy>

A view becomes stale when its primary draft, selected construction entry, workspace identity or any contributing supplied draft changes. An unrelated unused draft must not retire it. Newly retained contributor text requires a fresh scan. Saved files are checked independently by the engine at execution admission.

<ui:copy id="test-sources-browser-current">
export function testSourceCurrent(before: TestSourceSnapshot, facts: SourceTestScan, now: TestSourceSnapshot): boolean {
  const a=before.request,b=now.request;
  if(facts.version!==1||facts.input_path!==a.path||before.workspace!==now.workspace||a.path!==b.path||a.source!==b.source||(a.project_entry??null)!==(b.project_entry??null))return false;
  return Object.keys(facts.source_hashes).every(path=>path===a.path || a.sources?.[path]===b.sources?.[path]);
}
</ui:copy>

Build a versioned saved-run request only for a candidate in the current observation. Dynamic declarations cannot become name filters; a user must explicitly select the containing runner scope or discover concrete cases.

<ui:copy id="test-sources-browser-selection">
export function sourceTestRun(before:TestSourceSnapshot,facts:SourceTestScan,now:TestSourceSnapshot,id:string,selection:"name_filter"|"containing_scope"):SourceTestRunRequest {
  if(!testSourceCurrent(before,facts,now))throw new Error("Test source changed. Refresh candidates before running.");
  const candidate=facts.candidates.find(item=>item.id===id);
  if(!candidate)throw new Error("Candidate unavailable. Refresh test candidates.");
  if(selection==="name_filter"&&candidate.name_filter===null)throw new Error("This declaration needs runner discovery or an explicit containing-scope run.");
  return {version:1,input:{kind:"saved"},analysis:before.request,fingerprint:facts.fingerprint,candidate_id:id,selection:{kind:selection}};
}
/** Resolve only the source owner the compiler reported; never guess an authored line. */
export function candidatePoint(candidate:SourceTestCandidate){return candidate.source??candidate.target;}
</ui:copy>

The focused witnesses change primary/contributor/workspace observations, preserve an unrelated draft, reject a future protocol version, and distinguish dynamic declaration filters from explicit containing-scope execution.

<ui:copy id="test-sources-browser-witness">
import {describe,it,expect} from "vitest";
import {testSourceCurrent,sourceTestRun,type TestSourceSnapshot} from "./TestSources";
import type {SourceTestScan} from "../api/testSources";
const before:TestSourceSnapshot={workspace:"w1",request:{path:"cases.md",source:"draft",project_entry:"project.md",sources:{"helper.md":"helper","unused.md":"old"}}};
const point={path:"test.py",line:1,character:4,hash:"target"};
const facts:SourceTestScan={version:1,workspace:"root",workspace_id:"w1",input_hash:"draft-hash",scope:"project",project_entry:"project.md",syntax_complete:true,notices:[],unsupported_languages:[],unclassified_outputs:[],scanned_files:1,unused_sources:["unused.md"],project_context:null,input_path:"cases.md",fingerprint:"scan",source_hashes:{"cases.md":"draft-hash","helper.md":"helper-hash"},candidates:[{id:"case",name:"test_case",language:"python",run_scope:"file",name_filter:"test_case",confidence:"syntax",source:null,target:point}]};
describe("source test observations",()=>{
  it("retires changed contributors and workspace identity while ignoring unrelated drafts",()=>{
    expect(testSourceCurrent(before,facts,before)).toBe(true);
    expect(testSourceCurrent(before,facts,{...before,workspace:"w2"})).toBe(false);
    expect(testSourceCurrent(before,facts,{...before,request:{...before.request,source:"changed"}})).toBe(false);
    expect(testSourceCurrent(before,facts,{...before,request:{...before.request,sources:{"helper.md":"changed"}}})).toBe(false);
    expect(testSourceCurrent(before,facts,{...before,request:{...before.request,sources:{"helper.md":"helper","unused.md":"new"}}})).toBe(true);
    expect(testSourceCurrent(before,{...facts,version:2} as unknown as SourceTestScan,before)).toBe(false);
  });
  it("requires explicit supported selection and keeps saved execution bound to its facts",()=>{
    const request=sourceTestRun(before,facts,before,"case","name_filter");
    expect(request).toMatchObject({input:{kind:"saved"},fingerprint:"scan",candidate_id:"case",analysis:before.request});
    const dynamic={...facts,candidates:[{...facts.candidates[0],name_filter:null}]};
    expect(()=>sourceTestRun(before,dynamic,before,"case","name_filter")).toThrow("discovery");
    expect(sourceTestRun(before,dynamic,before,"case","containing_scope").selection.kind).toBe("containing_scope");
    expect(()=>sourceTestRun(before,facts,{...before,workspace:"changed"},"case","containing_scope")).toThrow("changed");
    expect(()=>sourceTestRun(before,facts,before,"missing","name_filter")).toThrow("unavailable");
  });
});
</ui:copy>

<ui:file path="apps/web/src/simple/TestSources.ts"><ui:paste select="#test-sources-browser-values"/><ui:paste select="#test-sources-browser-current"/><ui:paste select="#test-sources-browser-selection"/></ui:file>
<ui:file path="apps/web/src/simple/TestSources.test.ts"><ui:paste select="#test-sources-browser-witness"/></ui:file>
</ui:doc>
