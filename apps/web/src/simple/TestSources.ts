import type { SourceTestRequest, SourceTestScan, SourceTestCandidate, SourceTestRunRequest } from "../api/testSources";
/** Retain supplied drafts, separately from saved bytes that admission checks later. */
export interface TestSourceSnapshot { workspace: string; request: SourceTestRequest;owner?:object;namespace?:string;revision?:number;executionCurrent?:boolean }
export function testSourceCurrent(before: TestSourceSnapshot, facts: SourceTestScan, now: TestSourceSnapshot): boolean {
  const a=before.request,b=now.request;
  if(facts.version!==1||facts.input_path!==a.path||before.workspace!==now.workspace||a.path!==b.path||a.source!==b.source||before.owner!==now.owner||before.namespace!==now.namespace||before.revision!==now.revision||(a.project_entry??null)!==(b.project_entry??null))return false;
  return Object.keys(facts.source_hashes).every(path=>path===a.path || a.sources?.[path]===b.sources?.[path]);
}
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
export function parseSourceTestDebugJson(raw:string):unknown {
 const value:unknown=JSON.parse(raw),stack:(Set<string>|null)[]=[];let i=0;
 while(i<raw.length){const char=raw[i];if(char==='{'){stack.push(new Set());i++;continue;}if(char==='['){stack.push(null);i++;continue;}if(char==='}'||char===']'){stack.pop();i++;continue;}if(char!=='"'){i++;continue;}
  const start=i++;let escaped=false;for(;i<raw.length;i++){const next=raw[i];if(escaped){escaped=false;continue;}if(next==='\\'){escaped=true;continue;}if(next==='"')break;}const end=++i;let at=i;while(/\s/.test(raw[at]??'')&&at<raw.length)at++;
  if(raw[at]!==':')continue;const key=JSON.parse(raw.slice(start,end)) as string,keys=stack[stack.length-1];if(!keys)throw Error('Malformed Test Debug object key.');if(keys.has(key))throw Error('Duplicate Test Debug JSON key refused.');keys.add(key);
  if(key==='version'||key==='test_debug_protocol'){at++;while(/\s/.test(raw[at]??'')&&at<raw.length)at++;if(raw[at]==='-'||/[0-9]/.test(raw[at]??'')){let last=at;while(last<raw.length&&!/[\s,}\]]/.test(raw[last]))last++;if(!/^(0|[1-9][0-9]*)$/.test(raw.slice(at,last)))throw Error('Test Debug protocol requires a canonical integer version.');}}
 }
 return value;
}
