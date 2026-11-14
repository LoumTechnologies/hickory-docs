import type { SourceTestRequest, SourceTestScan, SourceTestCandidate, SourceTestRunRequest } from "../api/testSources";
/** Retain supplied drafts, separately from saved bytes that admission checks later. */
export interface TestSourceSnapshot { workspace: string; request: SourceTestRequest }
export function testSourceCurrent(before: TestSourceSnapshot, facts: SourceTestScan, now: TestSourceSnapshot): boolean {
  const a=before.request,b=now.request;
  if(facts.version!==1||facts.input_path!==a.path||before.workspace!==now.workspace||a.path!==b.path||a.source!==b.source||(a.project_entry??null)!==(b.project_entry??null))return false;
  return Object.keys(facts.source_hashes).every(path=>path===a.path || a.sources?.[path]===b.sources?.[path]);
}
export function sourceTestRun(before:TestSourceSnapshot,facts:SourceTestScan,now:TestSourceSnapshot,id:string,selection:"name_filter"|"containing_scope"):SourceTestRunRequest {
  if(!testSourceCurrent(before,facts,now))throw new Error("Test source changed. Refresh candidates before running.");
  const candidate=facts.candidates.find(item=>item.id===id);
  if(!candidate)throw new Error("Candidate unavailable. Refresh test candidates.");
  if(selection==="name_filter"&&candidate.name_filter===null)throw new Error("This declaration needs runner discovery or an explicit containing-scope run.");
  return {version:1,input:{kind:"saved"},analysis:before.request,fingerprint:facts.fingerprint,candidate_id:id,selection:{kind:selection}};
}
/** Resolve only the source owner the compiler reported; never guess an authored line. */
export function candidatePoint(candidate:SourceTestCandidate){return candidate.source??candidate.target;}
