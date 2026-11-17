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
