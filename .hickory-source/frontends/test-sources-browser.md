<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retain source test observations on browser

For maintainers replacing the test UI without replacing source and execution admission.

Keep source snapshots and provider observations as distinct values. Unknown source mappings stay absent; target runtime locations remain available without guessing an authored owner.

<ui:copy id="test-sources-browser-values">
import type { SourceTestRequest, SourceTestScan, SourceTestCandidate, SourceTestRunRequest } from "../api/testSources";
/** Retain supplied drafts, separately from saved bytes that admission checks later. */
export interface TestSourceSnapshot { workspace: string; request: SourceTestRequest;owner?:object;namespace?:string;revision?:number;executionCurrent?:boolean }
</ui:copy>

A view becomes stale when its primary draft, selected construction entry, workspace identity or any contributing supplied draft changes. An unrelated unused draft must not retire it. Newly retained contributor text requires a fresh scan. Saved files are checked independently by the engine at execution admission.

<ui:copy id="test-sources-browser-current">
export function testSourceCurrent(before: TestSourceSnapshot, facts: SourceTestScan, now: TestSourceSnapshot): boolean {
  const a=before.request,b=now.request;
  if(facts.version!==1||facts.input_path!==a.path||before.workspace!==now.workspace||a.path!==b.path||a.source!==b.source||before.owner!==now.owner||before.namespace!==now.namespace||before.revision!==now.revision||(a.project_entry??null)!==(b.project_entry??null))return false;
  return Object.keys(facts.source_hashes).every(path=>path===a.path || a.sources?.[path]===b.sources?.[path]);
}
</ui:copy>

Build a versioned saved-run request only for a candidate in the current observation. Dynamic declarations cannot become name filters; a user must explicitly select the containing runner scope or discover concrete cases.

<ui:copy id="test-sources-browser-selection">
export function sourceTestRun(before:TestSourceSnapshot,facts:SourceTestScan,now:TestSourceSnapshot,id:string,selection:"name_filter"|"containing_scope"):SourceTestRunRequest {
  if(before.executionCurrent===false||now.executionCurrent===false)throw new Error("Save the current owned source before Run or Debug; deleted or saving editors cannot authorize execution.");
  if(!testSourceCurrent(before,facts,now))throw new Error("Test source changed. Refresh candidates before running.");
  const candidate=facts.candidates.find(item=>item.id===id);
  if(!candidate)throw new Error("Candidate unavailable. Refresh test candidates.");
  if(selection!=="name_filter"&&selection!=="containing_scope")throw new Error("Unsupported test selection.");
  if(selection==="name_filter"&&(typeof candidate.name_filter!=="string"||!candidate.name_filter.trim()||candidate.confidence==="dynamic"))throw new Error("This declaration needs runner discovery or an explicit containing-scope run.");
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


## Bind a Debug choice before asking a provider to execute

A test candidate is not a program or argv. Copy the saved-run request and every
source hash into an immutable choice before the asynchronous plan. Accept only a
protocol-1 plan that echoes that choice and the canonical authored source URI.
The provider owns the runtime target. A matching Started receipt must echo the
complete contributor map; a process session is never a collected test verdict.

<ui:copy id="test-sources-browser-debug-choice">
import {documentUri} from '../lsp/documentUri';
export interface SourceTestDebugPlan {version:1;test_debug_protocol:1;available:boolean;reason?:string;workspace_id:string;input_path:string;input_hash:string;analysis_fingerprint:string;candidate_id:string;candidate_name:string;selection:SourceTestRunRequest['selection'];source_uri:string;source_hash:string;language:string;framework:string;name:string;notices:string[]}
export interface SourceTestDebugBinding {version:1;workspace_id:string;input_path:string;input_hash:string;analysis_fingerprint:string;candidate_id:string;candidate_name:string;selection:SourceTestRunRequest['selection'];checked_sources:Record<string,string>;language:string;framework:string}
export interface SourceTestDebugChoice {request:SourceTestRunRequest;workspace:string;inputPath:string;inputHash:string;fingerprint:string;candidateId:string;candidateName:string;selection:SourceTestRunRequest['selection'];sources:Readonly<Record<string,string>>;sourceUri:string;language:string}
export interface PreparedSourceTestDebug {choice:SourceTestDebugChoice;plan:Readonly<SourceTestDebugPlan>}
export type SourceTestDebugSelection=Omit<SourceTestDebugChoice,'request'>;
export interface SourceTestDebugProcess {session:string;choice:SourceTestDebugSelection;binding:SourceTestDebugBinding;phase:string;exitCode:number|null;message:string|null}
const sourceHash=(value:unknown):value is string=>typeof value==='string'&&/^[a-f0-9]{64}$/.test(value);
function frozenRun(request:SourceTestRunRequest):SourceTestRunRequest {return Object.freeze({...request,input:Object.freeze({...request.input}),selection:Object.freeze({...request.selection}),analysis:Object.freeze({...request.analysis,...(request.analysis.sources?{sources:Object.freeze({...request.analysis.sources})}:{})})});}
export function sourceTestDebugChoice(before:TestSourceSnapshot,facts:SourceTestScan,now:TestSourceSnapshot,id:string,selection:'name_filter'|'containing_scope'):SourceTestDebugChoice {
 const request=frozenRun(sourceTestRun(before,facts,now,id,selection)),candidate=facts.candidates.find(item=>item.id===id)!;
 if(!facts.workspace_id||!sourceHash(facts.input_hash)||facts.source_hashes[facts.input_path]!==facts.input_hash||Object.values(facts.source_hashes).some(value=>!sourceHash(value)))throw Error('Test source hashes are unavailable. Refresh candidates before Debug.');
 return Object.freeze({request,workspace:facts.workspace_id,inputPath:facts.input_path,inputHash:facts.input_hash,fingerprint:facts.fingerprint,candidateId:candidate.id,candidateName:candidate.name,selection:request.selection,sources:Object.freeze({...facts.source_hashes}),sourceUri:documentUri(facts.input_path),language:candidate.language});
}
const sameSelection=(a:unknown,b:SourceTestRunRequest['selection'])=>!!a&&typeof a==='object'&&Object.keys(a).length===1&&(a as {kind?:unknown}).kind===b.kind;
const sameSources=(a:unknown,b:Readonly<Record<string,string>>)=>!!a&&typeof a==='object'&&!Array.isArray(a)&&Object.keys(a).length===Object.keys(b).length&&Object.entries(b).every(([path,hash])=>Object.prototype.hasOwnProperty.call(a,path)&&(a as Record<string,unknown>)[path]===hash);
export function prepareSourceTestDebug(choice:SourceTestDebugChoice,value:unknown):PreparedSourceTestDebug {
 const plan=value as SourceTestDebugPlan;if(!plan||plan.version!==1||plan.test_debug_protocol!==1)throw Error('Test Debug protocol 1 is unavailable. Update the engine; no Debug launch was sent.');
 if(plan.workspace_id!==choice.workspace||plan.input_path!==choice.inputPath||plan.input_hash!==choice.inputHash||plan.analysis_fingerprint!==choice.fingerprint||plan.candidate_id!==choice.candidateId||plan.candidate_name!==choice.candidateName||!sameSelection(plan.selection,choice.selection)||plan.source_uri!==choice.sourceUri||plan.source_hash!==choice.inputHash||plan.language!==choice.language)throw Error('The Test Debug plan does not identify this exact source observation and selection. Refresh candidates.');
 if(plan.available!==true)throw Error(plan.reason||'This provider cannot Debug the selected test scope. No launch was sent.');
 if(typeof plan.framework!=='string'||!plan.framework||typeof plan.name!=='string'||!plan.name||!Array.isArray(plan.notices)||plan.notices.some(value=>typeof value!=='string'))throw Error('The Test Debug provider plan is malformed. No launch was sent.');
 return Object.freeze({choice,plan:Object.freeze({...plan,selection:Object.freeze({...plan.selection}),notices:Object.freeze([...plan.notices]) as unknown as string[]})});
}
/** Retained process history keeps identity hashes, not another copy of source bodies. */
export function retainedSourceTestDebugChoice(choice:SourceTestDebugChoice):SourceTestDebugSelection {const {workspace,inputPath,inputHash,fingerprint,candidateId,candidateName,selection,sources,sourceUri,language}=choice;return Object.freeze({workspace,inputPath,inputHash,fingerprint,candidateId,candidateName,selection,sources,sourceUri,language});}
export function sourceTestDebugBinding(prepared:PreparedSourceTestDebug):SourceTestDebugBinding {const choice=prepared.choice;return Object.freeze({version:1,workspace_id:choice.workspace,input_path:choice.inputPath,input_hash:choice.inputHash,analysis_fingerprint:choice.fingerprint,candidate_id:choice.candidateId,candidate_name:choice.candidateName,selection:choice.selection,checked_sources:choice.sources,language:choice.language,framework:prepared.plan.framework});}
export function sourceTestDebugStarted(prepared:PreparedSourceTestDebug,binding:unknown):binding is SourceTestDebugBinding {
 const choice=prepared.choice,value=binding as SourceTestDebugBinding;return !!value&&value.version===1&&value.workspace_id===choice.workspace&&value.input_path===choice.inputPath&&value.input_hash===choice.inputHash&&value.analysis_fingerprint===choice.fingerprint&&value.candidate_id===choice.candidateId&&value.candidate_name===choice.candidateName&&sameSelection(value.selection,choice.selection)&&sameSources(value.checked_sources,choice.sources)&&value.language===choice.language&&value.framework===prepared.plan.framework;
}
</ui:copy>


Raw protocol identity must survive decoding. JSON.parse silently merges duplicate
keys and erases the distinction between integer `1` and `1.0`. Scan only this
small plan/Started boundary before admission; normal editor reads keep their
existing codec. Keys remain exact JavaScript strings, preserving byte-distinct
Unicode paths instead of normalizing their namespace.

<ui:copy id="test-sources-browser-debug-json">
export function parseSourceTestDebugJson(raw:string):unknown {
 const value:unknown=JSON.parse(raw),stack:(Set<string>|null)[]=[];let i=0;
 while(i<raw.length){const char=raw[i];if(char==='{'){stack.push(new Set());i++;continue;}if(char==='['){stack.push(null);i++;continue;}if(char==='}'||char===']'){stack.pop();i++;continue;}if(char!=='"'){i++;continue;}
  const start=i++;let escaped=false;for(;i<raw.length;i++){const next=raw[i];if(escaped){escaped=false;continue;}if(next==='\\'){escaped=true;continue;}if(next==='"')break;}const end=++i;let at=i;while(/\s/.test(raw[at]??'')&&at<raw.length)at++;
  if(raw[at]!==':')continue;const key=JSON.parse(raw.slice(start,end)) as string,keys=stack[stack.length-1];if(!keys)throw Error('Malformed Test Debug object key.');if(keys.has(key))throw Error('Duplicate Test Debug JSON key refused.');keys.add(key);
  if(key==='version'||key==='test_debug_protocol'){at++;while(/\s/.test(raw[at]??'')&&at<raw.length)at++;if(raw[at]==='-'||/[0-9]/.test(raw[at]??'')){let last=at;while(last<raw.length&&!/[\s,}\]]/.test(raw[last]))last++;if(!/^(0|[1-9][0-9]*)$/.test(raw.slice(at,last)))throw Error('Test Debug protocol requires a canonical integer version.');}}
 }
 return value;
}
</ui:copy>

<ui:file path="apps/web/src/simple/TestSources.ts"><ui:paste select="#test-sources-browser-values"/><ui:paste select="#test-sources-browser-current"/><ui:paste select="#test-sources-browser-selection"/><ui:paste select="#test-sources-browser-debug-choice"/><ui:paste select="#test-sources-browser-debug-json"/></ui:file>

<ui:copy id="test-sources-browser-debug-witness">
import {sourceTestDebugChoice,prepareSourceTestDebug,sourceTestDebugStarted,sourceTestDebugBinding,parseSourceTestDebugJson,retainedSourceTestDebugChoice} from './TestSources';
const H='a'.repeat(64),J='b'.repeat(64);
function debugFixture(){const snapshot={...before,request:{...before.request,sources:{...before.request.sources}}},scan={...facts,input_hash:H,source_hashes:{'cases.md':H,'helper.md':J},candidates:facts.candidates.map(candidate=>({...candidate}))};const choice=sourceTestDebugChoice(snapshot,scan,snapshot,'case','name_filter'),plan={version:1,test_debug_protocol:1,available:true,workspace_id:'w1',input_path:'cases.md',input_hash:H,analysis_fingerprint:'scan',candidate_id:'case',candidate_name:'test_case',selection:{kind:'name_filter'},source_uri:'hick:///cases.md',source_hash:H,language:'python',framework:'pytest',name:'Pytest selected family',notices:[]};return {snapshot,scan,choice,plan};}
it('freezes an exact saved request and never substitutes a runtime target or mutable selector',()=>{const {snapshot,scan,choice,plan}=debugFixture();snapshot.request.source='changed';snapshot.request.sources['helper.md']='different';scan.candidates[0].name='renamed';scan.source_hashes['helper.md']=H;expect(choice.request.analysis.source).toBe('draft');expect(choice.request.analysis.sources?.['helper.md']).toBe('helper');expect(choice.candidateName).toBe('test_case');expect(choice.sources['helper.md']).toBe(J);expect(choice.sourceUri).toBe('hick:///cases.md');expect(Object.isFrozen(choice.request.analysis)).toBe(true);expect(prepareSourceTestDebug(choice,plan).plan.framework).toBe('pytest');expect(retainedSourceTestDebugChoice(choice)).not.toHaveProperty('request');});
it('refuses unavailable, legacy, malformed and mismatched plans without widening the selected scope',()=>{const {choice,plan}=debugFixture();for(const patch of [{version:2},{test_debug_protocol:0},{test_debug_protocol:true},{available:false,reason:'Provider unavailable'},{workspace_id:'other'},{input_path:'runtime.py'},{input_hash:J},{analysis_fingerprint:'old'},{candidate_id:'other'},{candidate_name:'other'},{selection:{kind:'containing_scope'}},{source_uri:'hick:///runtime.py'},{source_hash:J},{language:'rust'},{framework:''},{notices:[1]}])expect(()=>prepareSourceTestDebug(choice,{...plan,...patch})).toThrow();});
it('requires a complete Started selection/contributor receipt and preserves process-only identity',()=>{const {choice,plan}=debugFixture(),prepared=prepareSourceTestDebug(choice,plan),binding=sourceTestDebugBinding(prepared);expect(sourceTestDebugStarted(prepared,binding)).toBe(true);for(const patch of [{version:true},{workspace_id:'other'},{input_hash:J},{candidate_name:'sibling'},{selection:{kind:'containing_scope'}},{checked_sources:{'cases.md':H}},{checked_sources:{...choice.sources,'extra.md':H}},{framework:'other'},{language:'rust'}])expect(sourceTestDebugStarted(prepared,{...binding,...patch})).toBe(false);expect(sourceTestDebugStarted(prepared,null)).toBe(false);});
it('rejects merged JSON keys and noncanonical numeric protocol versions at the raw boundary',()=>{for(const text of ['{"version":1,"version":1}',String.raw`{"version":1,"\u0076ersion":1}`,'{"source_test":{"checked_sources":{"a.py":"one","a.py":"two"}}}','{"version":1.0}','{"test_debug_protocol":1e0}'])expect(()=>parseSourceTestDebugJson(text)).toThrow();expect(parseSourceTestDebugJson('{"version":1,"test_debug_protocol":1,"available":false}')).toMatchObject({version:1});});
it('refuses deleted/replaced/saving source ownership and dynamic or empty filtered choices',()=>{expect(()=>sourceTestRun({...before,executionCurrent:false},facts,before,'case','name_filter')).toThrow('owned source');expect(testSourceCurrent({...before,owner:{}},facts,{...before,owner:{}})).toBe(false);const {snapshot,scan}=debugFixture();for(const candidate of [{...scan.candidates[0],name_filter:''},{...scan.candidates[0],confidence:'dynamic' as const}])expect(()=>sourceTestDebugChoice(snapshot,{...scan,candidates:[candidate]},snapshot,'case','name_filter')).toThrow();expect(sourceTestDebugChoice(snapshot,{...scan,candidates:[{...scan.candidates[0],confidence:'dynamic',name_filter:null}]},snapshot,'case','containing_scope').selection.kind).toBe('containing_scope');});
</ui:copy>

<ui:file path="apps/web/src/simple/TestSources.test.ts"><ui:paste select="#test-sources-browser-witness"/><ui:paste select="#test-sources-browser-debug-witness"/></ui:file>
</ui:doc>
