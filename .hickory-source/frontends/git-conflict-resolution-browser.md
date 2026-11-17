<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Review complete Git conflict groups in the browser

For maintainers connecting composition to shared source and Git index authority.

## Separate proposal construction, draft application, saving and staging

A conflict group is an explicit set of members and destinations. A rename retains
an explicitly chosen identity; it is never inferred from filenames. Preview uses
the shared compiler/resource service, and only server-held immutable reviews can
save or stage. A request identity is retained before any mutation await. Unknown
outcomes remain owned across closing the floating view and cannot be retried.

Retained status distinguishes completed refusal from pending and unknown. A
completed refusal can retire only after the original complete review preimage is
revalidated. Source receipt adoption and coordinated undo remain host adapters;
policy does not belong to the React renderer.

<ui:copy id="git-resolution-browser-model">
import {resourceWirePatches,type ResourceSaved,type ResourceWirePlan,type ResourceEditorPatch,type ResourceConstructionFacts,type ResourceReceipt} from './ResourceEditors';
export interface ConflictStage {stage:number;mode:string;oid:string;text:string|null;unavailable:string|null}
export interface ConflictPath {path:string;working:ResourceSaved|null;stages:ConflictStage[];unavailable:string|null}
export interface ConflictReading {version:number;resolution_protocol:number;members:string[];paths:ConflictPath[];observation:unknown;source_files_modified:boolean;index_modified:boolean}
export interface ConflictOutput {path:string;identity_from:string|null;mode:'100644'|'100755';content:{kind:'text';text:string}|{kind:'stage';path:string;stage:number;oid:string}}
export interface ConflictPreview {version:number;resolution_protocol:number;token:string;fingerprint:string;members:string[];remaining_conflicts:string[];plan:ResourceWirePlan;consequences:ResourceConstructionFacts;saved_sources:Record<string,ResourceSaved>;needs_save:boolean;stage_available:boolean;stage_unavailable:string|null;source_files_modified:boolean;index_modified:boolean}
export interface HeldConflict {value:ConflictPreview;patches:ResourceEditorPatch[];current:()=>boolean}
export interface ConflictCapture {inputs:unknown;capture:unknown;current:()=>boolean}
export interface ConflictPorts {
 load:<T>(method:'GET'|'POST',path:string,body?:unknown)=>Promise<T>;
 capture:(paths:string[],edit:unknown)=>Promise<ConflictCapture>;
 adopt:(review:HeldConflict,receipt:ResourceReceipt,request:string)=>void;
 draft:(review:HeldConflict)=>void;
 lease:()=>()=>boolean;
}
function validMembers(value:unknown):value is string[]{return Array.isArray(value)&&value.length<=1000&&value.every(path=>typeof path==='string'&&!!path)&&new Set(value).size===value.length;}
function sameMembers(value:unknown,expected:string[]){return validMembers(value)&&value.length===expected.length&&value.every(path=>expected.includes(path));}
export function decodeConflictPreview(raw:ConflictPreview):ConflictPreview {
 const value=structuredClone(raw);
 if(value.version!==1||value.resolution_protocol!==1||!/^[a-f0-9]{32}$/.test(value.token)||!/^[a-f0-9]{64}$/.test(value.fingerprint)||value.source_files_modified!==false||value.index_modified!==false||typeof value.needs_save!=='boolean'||typeof value.stage_available!=='boolean'||value.plan?.version!==1||!value.plan.context||!Array.isArray(value.plan.patches)||!value.saved_sources||!value.consequences?.workspace||value.consequences.context!==value.plan.context)throw Error('Unsupported or ambiguous conflict review.');
 if(!validMembers(value.members)||!value.members.length||!validMembers(value.remaining_conflicts)||value.remaining_conflicts.some(path=>value.members.includes(path)))throw Error('Conflict review does not identify its complete unique group.');
 if(value.plan.patches.length)resourceWirePatches(value.plan);
 return value;
}
export function conflictDraftable(review:HeldConflict){return review.patches.some(patch=>patch.before?.text!==patch.after?.text)&&review.patches.every(patch=>patch.before&&patch.after&&patch.before.identity===patch.after.identity);}
interface PendingConflict {id:string;review:HeldConflict;stage:boolean;receipt?:ResourceReceipt}
export class ConflictResolutionModel {
 reading:ConflictReading|null=null;members:string[]=[];outputs:ConflictOutput[]=[];review:HeldConflict|null=null;notice='';busy=false;saved=false;pending:PendingConflict|null=null;private ticket=0;
 constructor(private ports:ConflictPorts,private changed:()=>void=()=>{}){}
 close(){if(!this.pending){this.ticket++;this.review=null;this.busy=false;}}
 change(members:string[],outputs:ConflictOutput[]){if(this.busy||this.pending)return;this.ticket++;this.review=null;this.saved=false;this.members=[...members];this.outputs=structuredClone(outputs);this.changed();}
 async read(members:string[]){if(this.busy||this.pending)return;const ticket=++this.ticket;this.busy=true;this.review=null;this.changed();
  try{const lease=this.ports.lease();const reading=await this.ports.load<ConflictReading>('POST','/api/git/conflict-resolution/read',{version:1,members:[...members].sort()});if(ticket!==this.ticket)return;if(!lease())throw Error('Workspace or source context changed while reading conflict stages.');
   if(reading.version!==1||reading.resolution_protocol!==1||reading.source_files_modified!==false||reading.index_modified!==false||!Array.isArray(reading.paths)||reading.paths.length>1000||new Set(reading.paths.map(row=>row.path)).size!==reading.paths.length||JSON.stringify(reading.members)!==JSON.stringify([...members].sort()))throw Error('Conflict group identity changed or protocol is unavailable.');this.reading=structuredClone(reading);this.members=[...members];this.notice='Select the complete group and explicitly retain every destination. Omitted members will be deleted.';
  }catch(error){if(ticket===this.ticket)this.notice=String(error);}finally{if(ticket===this.ticket){this.busy=false;this.changed();}}
 }
 async prepare(){if(this.busy||this.pending||!this.reading)return;const ticket=++this.ticket,reading=structuredClone(this.reading),members=[...this.members].sort(),outputs=structuredClone(this.outputs);this.busy=true;this.review=null;this.saved=false;this.changed();
  try{if(!members.length||new Set(members).size!==members.length||outputs.some(row=>!row.path))throw Error('Choose complete conflict members and every destination explicitly.');const paths=[...new Set([...members,...outputs.map(row=>row.path)])].sort(),edit={changes:Object.fromEntries(paths.map(path=>['hick:///'+path.split('/').map(encodeURIComponent).join('/'),[]]))};const captured=await this.ports.capture(paths,edit);if(ticket!==this.ticket||!captured.current())throw Error('Source editor changed during acquisition.');const raw=await this.ports.load<ConflictPreview>('POST','/api/git/conflict-resolution/prepare',{version:1,reading,members,outputs,inputs:captured.inputs,capture:captured.capture});if(ticket!==this.ticket||!captured.current())throw Error('Source editor changed during conflict review.');const value=decodeConflictPreview(raw);if(JSON.stringify(value.members)!==JSON.stringify(members))throw Error('Reviewed conflict members changed.');this.review={value,patches:value.plan.patches.length?resourceWirePatches(value.plan):[],current:()=>ticket===this.ticket&&captured.current()};this.notice='Review all source and generated consequences. Draft, Save and Stage are separate actions.';
  }catch(error){if(ticket===this.ticket)this.notice=String(error);}finally{if(ticket===this.ticket){this.busy=false;this.changed();}}
 }
 private body(review:HeldConflict){return {version:1,token:review.value.token,fingerprint:review.value.fingerprint};}
 private async validate(review:HeldConflict,stage=false){if(!review.current())throw Error('Source editor changed after review.');const result=await this.ports.load<{version:number;token:string;fingerprint:string;stage:boolean;current:boolean;source_files_modified:boolean;index_modified:boolean;plan:ResourceWirePlan;saved_sources:Record<string,ResourceSaved>}>('POST','/api/git/conflict-resolution/validate',{...this.body(review),stage});if(result.version!==1||result.token!==review.value.token||result.fingerprint!==review.value.fingerprint||result.source_files_modified!==false||result.index_modified!==false||result.current!==true||result.stage!==stage||JSON.stringify(result.plan)!==JSON.stringify(review.value.plan)||JSON.stringify(result.saved_sources)!==JSON.stringify(review.value.saved_sources)||!review.current())throw Error('Conflict stages, reviewed plan or source authority changed.');}
 async draft(){const review=this.review;if(!review||this.busy||this.pending)return;this.busy=true;this.changed();try{if(!conflictDraftable(review))throw Error('Address changes require guarded resource Save; drafts were retained.');await this.validate(review);this.ports.draft(review);this.review=null;this.notice='Applied coordinated source drafts. Nothing saved or staged. Prepare again before saving.';}catch(error){this.notice=String(error);}finally{this.busy=false;this.changed();}}
 async act(stage:boolean){const review=this.review;if(!review||this.busy||this.pending)return;this.busy=true;this.changed();try{if(!review.current())throw Error('Source editor or workspace changed after review. Prepare the complete group again.');if(stage? !review.value.stage_available||review.value.needs_save&&!this.saved : !review.value.needs_save||this.saved)throw Error('Save the complete current review before staging; a consumed review cannot repeat.');if(!stage)await this.validate(review);const pending={id:crypto.randomUUID().replaceAll('-',''),review,stage};this.pending=pending;const result=await this.ports.load<unknown>('POST','/api/git/conflict-resolution/'+(stage?'stage':'save'),{...this.body(review),request_id:pending.id});this.accept(result);}catch(error){this.notice=String(error)+(this.pending?' Outcome unconfirmed. Inspect the original request; it will not be retried.':'');}finally{this.busy=false;this.changed();}}
 async status(){const pending=this.pending;if(!pending||this.busy)return;this.busy=true;this.changed();try{const value=await this.ports.load<{version:number;request_id:string;state:string;outcome?:unknown;error?:string}>('GET','/api/git/conflict-resolution/status?request_id='+encodeURIComponent(pending.id));if(value.version!==1||value.request_id!==pending.id)throw Error('Retained conflict request identity changed.');if(value.state==='completed'&&value.outcome)this.accept(value.outcome);else if(value.state==='completed'&&value.error){await this.validate(pending.review,pending.stage);this.pending=null;this.review=null;this.notice='The retained request was refused and the original complete preimage is current. Read and prepare a new review.';}else this.notice='The original request is pending or unknown. Preserve drafts and inspect Source recovery; no new request is authorized.';}catch(error){this.notice=String(error);}finally{this.busy=false;this.changed();}}
 private accept(raw:unknown){const pending=this.pending;if(!pending)throw Error('No owned conflict publication.');const review=pending.review,value=raw as Record<string,unknown>;if(pending.stage){if(value.version!==1||value.operation!=='git_conflict_stage'||value.complete!==true||value.index_modified!==true||value.index_uncertain!==false||value.source_files_modified!==false||value.request_id!==pending.id||value.token!==review.value.token||value.review_fingerprint!==review.value.fingerprint||!sameMembers(value.resolved_members,review.value.members)||!sameMembers(value.remaining_conflicts,review.value.remaining_conflicts)||typeof value.fingerprint!=='string'||!/^[a-f0-9]{64}$/.test(value.fingerprint))throw Error('Complete staged group was not confirmed.');this.pending=null;this.review=null;this.notice='Complete saved conflict group staged. Unsaved drafts remain separate. No tests were run.';}else{const receipt=structuredClone(raw) as ResourceReceipt;pending.receipt=receipt;this.ports.adopt(review,receipt,pending.id);this.pending=null;this.saved=true;this.review={...review,current:this.ports.lease()};this.notice='Complete saved group confirmed. Choose Stage separately.';}}
}
</ui:copy>
## Reuse authoritative resource capture and retained editor adoption

Acquire every existing source before capturing the complete group again. The
second capture must retain the first saved/draft namespace. Draft application
uses coordinated workspace undo. Saved adoption requires the exact request,
saved metadata and recovery identity before routing retained source editors.

<ui:copy id="git-resolution-browser-adapter">
import {request} from '../api/client';
import type {Buffer,Buffers} from './buffers';
import {WorkspaceChangeHistory,invertChangePlan,type ChangePlan} from './ChangePlan';
import {captureDraftHistory,applyWorkspacePlan} from './WorkspaceChanges';
import {captureResourceInputs,captureResourceRoster,adoptResourceReceipt,type ResourceEditorRoute} from './ResourceEditors';
import {historySourceHash} from './SourceHistory';
import {ConflictResolutionModel,type ConflictPorts} from './GitConflictResolution';
export function browserConflictModel(buffers:Buffers,history:WorkspaceChangeHistory,acquire:(path:string)=>Promise<Buffer>,adopt:(routes:ResourceEditorRoute<Buffer>[])=>void,alive:()=>boolean,changed:()=>void){
 const namespace=buffers.workspace,currentNamespace=()=>alive()&&!!namespace&&buffers.workspace===namespace;
 const lease=()=>{const roster=captureResourceRoster(buffers);return ()=>currentNamespace()&&roster.current();};
 const load:ConflictPorts['load']=(method,path,body)=>request(method,path,body);
 return new ConflictResolutionModel({load,lease,
  capture:async(paths,edit)=>{const context=crypto.randomUUID().replaceAll('-',''),post=<T>(path:string,body:unknown)=>load<T>('POST',path,body);const initial=await captureResourceInputs(buffers,paths,context,edit,post,currentNamespace,historySourceHash);
   for(const path of paths)if(initial.inputs.sources[path]?.kind==='file'){const buffer=await acquire(path);if(!currentNamespace()||buffer.text!==initial.inputs.sources[path].text)throw Error('Referenced conflict source changed during acquisition.');}
   const captured=await captureResourceInputs(buffers,paths,context,edit,post,currentNamespace,historySourceHash);
   for(const path of paths){const before=initial.inputs.sources[path],after=captured.inputs.sources[path];if(before?.kind!==after?.kind||before?.text!==after?.text||JSON.stringify(initial.inputs.saved_sources[path])!==JSON.stringify(captured.inputs.saved_sources[path])||initial.inputs.saved_guards[path]!==captured.inputs.saved_guards[path])throw Error('Conflict source or saved baseline changed during acquisition.');}
   if(JSON.stringify(initial.inputs.saved_guards)!==JSON.stringify(captured.inputs.saved_guards))throw Error('Physical conflict dependencies changed during acquisition.');
   return captured;
  },draft:review=>{if(!review.current())throw Error('Conflict source draft changed.');const plan:ChangePlan={title:'Compose Git conflict group',patches:review.patches.map(patch=>({path:patch.path,before:patch.before!.text,after:patch.after!.text,edits:[{from:0,to:patch.before!.text.length,text:patch.after!.text}]}))};applyWorkspacePlan(buffers,plan);history.record(captureDraftHistory(buffers,invertChangePlan(plan)));},
  adopt:(review,receipt,id)=>{const recovery=receipt.outcome?.recovery?.match(/(?:^|\/)resource-changes\/([0-9a-f]{32})\/[^/]+$/)?.[1];if(!recovery)throw Error('Resource recovery identity is unavailable; source editors were retained.');const routes=adoptResourceReceipt(buffers,review.patches,receipt,{request:id,workspace:review.value.consequences.workspace,context:review.value.plan.context,saved:new Map(Object.entries(review.value.saved_sources))},review.current);history.record({kind:'resource',recovery});adopt(routes);}
 },changed);
}
</ui:copy>

## Compose inside the existing floating review surface

The renderer edits proposal data only. It offers explicit stage bytes, composed
text, retained identity, destination and mode. All immutable consequences are
paged inside the existing popover; no additional permanent chrome appears.
Save and Stage remain separate, and uncertainty exposes only original-request
inspection rather than an automatic second write.

<ui:copy id="git-resolution-browser-view">
import {useEffect,useState} from 'react';
import {conflictDraftable,type ConflictOutput,type ConflictReading,type ConflictResolutionModel} from './GitConflictResolution';
function outputText(output:ConflictOutput,reading:ConflictReading){const content=output.content;return content.kind==='text'?content.text:reading.paths.find(row=>row.path===content.path)?.stages.find(stage=>stage.stage===content.stage&&stage.oid===content.oid)?.text??'';}
export function GitResolutionView({path,model}:{path:string;model:ConflictResolutionModel}){
 const [page,setPage]=useState(0);
 useEffect(()=>()=>model.close(),[model]);
 const update=(index:number,value:ConflictOutput|null)=>model.change(model.members,model.outputs.flatMap((row,at)=>at===index?value?[value]:[]:[row]));
 const blocked=model.busy||!!model.pending,reading=model.reading,review=model.review,raw=review?JSON.stringify(review.value,null,2):'',last=Math.max(0,Math.ceil(raw.length/8000)-1),shown=Math.min(page,last);
 return <section aria-label="Complete conflict resolution"><button disabled={blocked} onClick={()=>void model.read([path])}>Compose conflict group…</button><p role="status">{model.notice}</p>{model.pending?.receipt&&<p>Retained publication {model.pending.receipt.request_id} · historical completion {String(model.pending.receipt.outcome?.complete)} · current adoption {String(model.pending.receipt.adoption_current)}. {model.pending.receipt.adoption_reason}</p>}{model.pending&&<button disabled={model.busy} onClick={()=>void model.status()}>Inspect original resolution request</button>}
 {reading&&<><p>Select every member in this resolution. Rename relationships and retained identities are explicit choices.</p>{reading.paths.map(row=><div key={row.path}><label><input type="checkbox" disabled={blocked} checked={model.members.includes(row.path)} onChange={()=>model.change(model.members.includes(row.path)?model.members.filter(path=>path!==row.path):[...model.members,row.path],model.outputs)}/>{JSON.stringify(row.path)}</label>{row.unavailable&&<p>{row.unavailable}</p>}</div>)}<button disabled={blocked||!model.members.length} onClick={()=>void model.read(model.members)}>Read selected group stages</button>
 {model.outputs.map((output,index)=><article key={index}><label>Destination<input aria-label={`Conflict destination ${index+1}`} disabled={blocked} value={output.path} onChange={event=>update(index,{...output,path:event.target.value})}/></label><label>Retained source identity<select aria-label={`Retained identity ${index+1}`} disabled={blocked} value={output.identity_from??''} onChange={event=>update(index,{...output,identity_from:event.target.value||null})}><option value="">New source identity</option>{model.members.map(member=><option key={member} value={member}>{JSON.stringify(member)}</option>)}</select></label>
 <label>Composed resolution<textarea aria-label={`Composed resolution ${index+1}`} rows={5} disabled={blocked} value={outputText(output,reading)} onChange={event=>update(index,{...output,content:{kind:'text',text:event.target.value}})}/></label>
 <label>Saved mode<select aria-label={`Saved mode ${index+1}`} disabled={blocked} value={output.mode} onChange={event=>update(index,{...output,mode:event.target.value as '100644'|'100755'})}><option value="100644">Regular file</option><option value="100755">Executable file</option></select></label><button disabled={blocked} onClick={()=>update(index,null)}>Remove destination</button>
 {reading.paths.filter(row=>model.members.includes(row.path)).map(row=><div key={row.path}>{row.stages.map(stage=><button key={stage.stage} disabled={blocked||stage.text===null} onClick={()=>update(index,{...output,mode:stage.mode as '100644'|'100755',content:{kind:'stage',path:row.path,stage:stage.stage,oid:stage.oid}})}>Use {JSON.stringify(row.path)} stage {stage.stage}</button>)}</div>)}</article>)}
 <button disabled={blocked} onClick={()=>model.change(model.members,[...model.outputs,{path:'',identity_from:null,mode:'100644',content:{kind:'text',text:''}}])}>Add explicit destination</button><p>Members without a retained destination will be deleted: {model.members.filter(member=>!model.outputs.some(output=>output.identity_from===member)).map(member=>JSON.stringify(member)).join(', ')||'none'}</p><button disabled={blocked} onClick={()=>void model.prepare()}>Preview complete resolution consequences</button></>}
 {review&&<><p>Complete immutable source/resource and generated-consequence review · {review.value.fingerprint}</p><pre style={{whiteSpace:'pre-wrap'}}>{raw.slice(shown*8000,(shown+1)*8000)}</pre><button disabled={!shown} onClick={()=>setPage(shown-1)}>Previous resolution consequences</button><span>{shown+1}/{last+1}</span><button disabled={shown===last} onClick={()=>setPage(shown+1)}>Next resolution consequences</button>{!review.value.stage_available&&<p>{review.value.stage_unavailable??'Staging unavailable'}</p>}<button disabled={blocked||!conflictDraftable(review)} onClick={()=>void model.draft()}>Apply text drafts only</button><button disabled={blocked||!review.value.needs_save||model.saved} onClick={()=>void model.act(false)}>Save complete resolution group</button><button disabled={blocked||!review.value.stage_available||review.value.needs_save&&!model.saved} onClick={()=>void model.act(true)}>Stage complete saved group</button></>}
 </section>;
}
</ui:copy>
<ui:file path="apps/web/src/simple/GitConflictResolutionAdapter.ts"><ui:paste select="#git-resolution-browser-adapter"/></ui:file>
<ui:file path="apps/web/src/simple/GitConflictResolutionView.tsx"><ui:paste select="#git-resolution-browser-view"/></ui:file>

## Check one-shot mutation ownership and no-op staging

These focused controller witnesses separate draft/save/stage, retain uncertain
requests through close, refuse stale namespace/source leases and require an exact
stage receipt. An already-saved complete group stages without a redundant Save.
The shared real-Git and host integration witnesses remain separate acceptance.

<ui:copy id="git-resolution-browser-tests">
import {beforeAll,expect,it,vi} from 'vitest';
import {ConflictResolutionModel,decodeConflictPreview,conflictDraftable,type ConflictPorts,type ConflictPreview,type ConflictReading,type ConflictOutput} from './GitConflictResolution';
beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});});
function reading():ConflictReading{return {version:1,resolution_protocol:1,members:['a.py'],paths:[{path:'a.py',working:null,stages:[],unavailable:null}],observation:{fingerprint:'index'},source_files_modified:false,index_modified:false};}
function preview(noop=false):ConflictPreview{return {version:1,resolution_protocol:1,token:'a'.repeat(32),fingerprint:'b'.repeat(64),members:['a.py'],remaining_conflicts:[],plan:{version:1,context:'context',title:'Resolve',patches:noop?[]:[{path:'a.py',before:{kind:'file',identity:'a',text:'before'},after:{kind:'file',identity:'a',text:'after'}}]},consequences:{version:1,workspace:'workspace',context:'context',fingerprint:'c'.repeat(64),protected_targets:[],saved:{entries:[],removed_entries:[],output_paths:[]},draft:{entries:[],removed_entries:[],output_paths:[]}},saved_sources:{'a.py':{kind:'file',text:'after',sha256:'d'.repeat(64),readonly:false,executable:false,mode:420}},needs_save:!noop,stage_available:true,stage_unavailable:null,source_files_modified:false,index_modified:false};}
const output:ConflictOutput={path:'a.py',identity_from:'a.py',mode:'100644',content:{kind:'text',text:'after'}};
function fixture(noop=false){const value=preview(noop),calls:{path:string;body:unknown}[]=[],draft=vi.fn(),adopt=vi.fn();let current=true,failStage=false,unknown=false;const ports:ConflictPorts={lease:()=>()=>current,capture:async()=>({inputs:{workspace:'workspace'},capture:{},current:()=>current}),draft,adopt,load:async<T>(_method:string,path:string,body?:unknown)=>{calls.push({path,body});const input=body as Record<string,unknown>;let result:unknown;if(path.endsWith('/read'))result=reading();else if(path.endsWith('/prepare'))result=value;else if(path.endsWith('/validate'))result={version:1,current:true,stage:input.stage??false,token:value.token,fingerprint:value.fingerprint,plan:value.plan,saved_sources:value.saved_sources,source_files_modified:false,index_modified:false};else if(path.endsWith('/save'))result={version:2,adoption_current:true,request_id:input.request_id};else if(path.endsWith('/stage'))result={version:1,operation:'git_conflict_stage',complete:true,index_modified:true,index_uncertain:false,source_files_modified:false,request_id:failStage?'foreign':input.request_id,token:value.token,review_fingerprint:value.fingerprint,resolved_members:value.members,remaining_conflicts:value.remaining_conflicts,fingerprint:'e'.repeat(64)};else result={version:1,request_id:path.split('request_id=')[1],state:unknown?'unknown':'completed',outcome:null,error:'refused'};return result as T;}};const model=new ConflictResolutionModel(ports);return {model,value,calls,draft,adopt,stale:()=>{current=false;},foreignStage:()=>{failStage=true;},unknown:()=>{unknown=true;}};}
async function prepared(f:ReturnType<typeof fixture>){await f.model.read(['a.py']);f.model.change(['a.py'],[output]);await f.model.prepare();expect(f.model.review).not.toBeNull();}
it('keeps composition, draft application, source Save and explicit Stage separate',async()=>{const f=fixture();await prepared(f);await f.model.act(true);expect(f.calls.some(call=>call.path.endsWith('/stage'))).toBe(false);await f.model.draft();expect(f.draft).toHaveBeenCalledOnce();expect(f.adopt).not.toHaveBeenCalled();expect(f.calls.some(call=>call.path.endsWith('/save'))).toBe(false);await prepared(f);await f.model.act(false);expect(f.adopt).toHaveBeenCalledOnce();expect(f.model.saved).toBe(true);await f.model.act(true);expect(f.model.pending).toBeNull();expect(f.model.notice).toContain('group staged');});
it('stages an already-saved complete group without a no-op Save',async()=>{const f=fixture(true);await prepared(f);expect(conflictDraftable(f.model.review!)).toBe(false);const same=preview();same.plan.patches[0].after.text='before';expect(conflictDraftable({value:same,patches:[{path:'a.py',before:{identity:'a',text:'before'},after:{identity:'a',text:'before'}}],current:()=>true})).toBe(false);await f.model.act(true);expect(f.calls.some(call=>call.path.endsWith('/save'))).toBe(false);expect(f.model.notice).toContain('group staged');});
it('retains uncertain request identity across close and refuses a second mutation',async()=>{const f=fixture(true);await prepared(f);f.foreignStage();await f.model.act(true);const id=f.model.pending?.id;expect(id).toBeTruthy();f.model.close();await f.model.act(true);expect(f.model.pending?.id).toBe(id);expect(f.calls.filter(call=>call.path.endsWith('/stage'))).toHaveLength(1);f.unknown();await f.model.status();expect(f.model.pending?.id).toBe(id);expect(f.adopt).not.toHaveBeenCalled();});
it('retires only a confirmed refused request whose complete original preimage is current',async()=>{for(const stale of [false,true]){const f=fixture(true);await prepared(f);f.foreignStage();await f.model.act(true);if(stale)f.stale();await f.model.status();expect(!!f.model.pending).toBe(stale);if(!stale)expect(f.model.notice).toContain('original complete preimage');}});
it('refuses stale source before dispatch and keeps immutable decoded proposals independent of raw mutation',async()=>{const f=fixture();await prepared(f);const held=f.model.review!;f.value.plan.patches[0].after.text='tampered';expect(held.patches[0].after?.text).toBe('after');f.stale();await f.model.act(false);expect(f.calls.some(call=>call.path.endsWith('/save'))).toBe(false);for(const [key,bad] of [['token','foreign'],['fingerprint','old'],['version',true],['resolution_protocol',1.5],['index_modified',true]] as const){expect(()=>decodeConflictPreview({...preview(),[key]:bad} as ConflictPreview)).toThrow();}});
it('retires a delayed reading on close or lost source/namespace lease',async()=>{for(const closed of [false,true]){let release!:(value:unknown)=>void,current=true;const ports:ConflictPorts={load:<T>()=>new Promise<T>(done=>{release=done as (value:unknown)=>void;}),lease:()=>()=>current,capture:async()=>{throw Error('unused');},draft:()=>{},adopt:()=>{}};const model=new ConflictResolutionModel(ports),pending=model.read(['a.py']);if(closed)model.close();else current=false;release(reading());await pending;expect(model.reading).toBeNull();}});
it('reports refused source lease acquisition without rejecting the optional reading promise',async()=>{let reads=0;const ports:ConflictPorts={lease:()=>{throw Error('Foreign retained source namespace');},load:async<T>()=>{reads++;return reading() as T;},capture:async()=>{throw Error('unused');},draft:()=>{},adopt:()=>{}};const model=new ConflictResolutionModel(ports);await expect(model.read(['a.py'])).resolves.toBeUndefined();expect(model.reading).toBeNull();expect(model.busy).toBe(false);expect(model.notice).toContain('Foreign retained source');expect(reads).toBe(0);});

</ui:copy>
<ui:file path="apps/web/src/simple/GitConflictResolution.test.ts"><ui:paste select="#git-resolution-browser-tests"/></ui:file>

<ui:file path="apps/web/src/simple/GitConflictResolution.ts"><ui:paste select="#git-resolution-browser-model"/></ui:file>
</ui:doc>
