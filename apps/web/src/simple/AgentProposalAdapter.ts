import {request} from '../api/client';
import type {AgentEditorContext} from '../api/agentTypes';
import {captureResourceRoster,resourceBufferId,resourceWirePatches,publishResourceRestore,type ResourceRestoreReview,type ResourceEditorRoute,type ResourceReceipt} from './ResourceEditors';
import {editorForBuffer} from './Document';
import type {Buffer,Buffers} from './buffers';
import {captureDraftHistory,applyWorkspacePlan,workspaceSnapshots} from './WorkspaceChanges';
import {planWorkspaceEdit,invertChangePlan,type ChangePlan,WorkspaceChangeHistory} from './ChangePlan';
import {AgentProposalModel,proposalHash,type ProposalPorts,type PreparedProposal,type VerifiedProposal} from './AgentProposals';
interface BrowserProposal extends PreparedProposal {draft?:ChangePlan;resource?:ResourceRestoreReview}
const equalMap=(left:Record<string,string>,right:Record<string,string>)=>Object.keys(left).length===Object.keys(right).length&&Object.entries(left).every(([path,text])=>Object.hasOwn(right,path)&&right[path]===text);
export function proposalEditorContext(buffers:Buffers,focused?:Buffer):AgentEditorContext {
 const roster=captureResourceRoster(buffers);if(roster.captures.some(row=>!Number.isSafeInteger(row.revision)||row.revision<0))throw Error('Original editor revision is unavailable. Keep the source readable and reconcile its editor before proposing changes.');
 return {live:false,buffers:roster.captures.filter(row=>!row.readonly&&!row.transient).map(row=>({id:resourceBufferId(row.buffer),kind:'authored',observations:[{kind:'editor_revision' as const,version:1 as const,source_revision:String(row.revision)}],name:row.file.name,path:Object.hasOwn(roster.ownership_sources,row.path)?row.path:null,content:row.text,focused:row.buffer===focused,save_guard:Object.hasOwn(roster.ownership_sources,row.path)?row.guard??null:null,...row.file.doc_id?{document:row.file.doc_id}:{}}))};
}
/** Optional richer capability never replaces an editor; its lease retires on user interaction. */
function editorLease(buffers:Buffers){const roster=captureResourceRoster(buffers),views=roster.captures.map(row=>{const view=editorForBuffer(row.buffer)?.view;return {buffer:row.buffer,view,doc:view?.state.doc,selection:view?.state.selection,readonly:view?.state.readOnly,connected:view?.dom.isConnected,parent:view?.dom.parentNode,top:view?.scrollDOM.scrollTop,left:view?.scrollDOM.scrollLeft};});return (acquired:ReadonlyMap<string,Buffer>=new Map())=>roster.current(acquired)&&views.every(row=>{const view=editorForBuffer(row.buffer)?.view;return view===row.view&&(!view||view.dom.isConnected===row.connected&&view.dom.parentNode===row.parent&&!view.composing&&view.state.readOnly===row.readonly&&view.state.doc===row.doc&&!!row.selection&&view.state.selection.eq(row.selection)&&view.scrollDOM.scrollTop===row.top&&view.scrollDOM.scrollLeft===row.left);});}
/** Managed proposals bind the original handles and edit epochs, not just equal current text. */
async function originalEditorBinding(buffers:Buffers,proposal:VerifiedProposal,current:()=>boolean){
 const snapshot=proposal.value.intention.editor_snapshot,authority=proposal.value.origin_authority;
 if(!authority.startsWith('engine-managed agent tool')&&!authority.startsWith('caller-provided intention'))throw Error('This proposal origin has no supported host application authority. Its material remains readable.');
 if(!snapshot){if(authority.startsWith('engine-managed agent tool'))throw Error('This managed proposal has no original editor snapshot. Its review remains readable; request a new captured intention before Apply.');return;}
 const lease=editorLease(buffers),context=proposalEditorContext(buffers),ids=new Set<string>();
 if(snapshot.buffers.length!==context.buffers.length)throw Error('The complete retained editor roster differs from the original agent snapshot. Request a fresh intention; a new lease cannot absorb unknown owners.');
 for(const original of snapshot.buffers){if(!original.id||ids.has(original.id)||original.source_revision===null)throw Error('Original editor identity/revision is unavailable. Retain the readable proposal and request a new complete snapshot.');ids.add(original.id);const row=context.buffers.find(row=>row.id===original.id);const revisions=row?.observations?.filter(value=>'kind' in value&&value.kind==='editor_revision');const revision=revisions?.length===1&&'source_revision' in revisions[0]?revisions[0].source_revision:null;if(!row||row.path!==original.path||(row.save_guard??null)!==original.save_guard||revision!==original.source_revision||await proposalHash(row.content)!==original.source_hash)throw Error('Original editor identity, revision, text or physical guard changed before review. Edit/Undo and independently reopened handles cannot rebind this intention.');if(!current()||!lease())throw Error('Editor interaction or workspace changed while checking the original snapshot.');}
}
export function browserProposalPorts(buffers:Buffers,history:WorkspaceChangeHistory,acquire:(path:string)=>Promise<Buffer>,adopt:(routes:ResourceEditorRoute<Buffer>[])=>void,alive:()=>boolean):ProposalPorts {
 const scope=()=>{if(!buffers.workspace||!buffers.workspaceId)throw Error('Workspace source authority is not available yet. Retain the review and wait for the workspace.');return {namespace:buffers.workspace,workspace:buffers.workspaceId};};
 return {scope,
  prepare:async(proposal:VerifiedProposal,valid)=>{
   const value=proposal.value,capture=value.intention.capture,patches=resourceWirePatches(value.plan),paths=new Set(patches.map(patch=>patch.path));if(paths.size!==patches.length)throw Error('Proposal repeats source targets. The complete namespace cannot be reviewed.');
   const root=scope(),currentRoot=()=>alive()&&valid()&&buffers.workspace===root.namespace&&buffers.workspaceId===root.workspace;
   await originalEditorBinding(buffers,proposal,currentRoot);if(!currentRoot())throw Error('Original proposal context changed.');
   const required=new Set([...Object.keys(capture.ownership_sources),...patches.filter(patch=>patch.before).map(patch=>patch.path)]);
   for(const path of required){if(!currentRoot())throw Error('Proposal workspace or review changed.');if(buffers.values.has(path))continue;const before=editorLease(buffers);const target=await acquire(path);if(!currentRoot()||buffers.values.get(path)!==target||!before(new Map([[path,target]])))throw Error('Editor roster changed while acquiring a reviewed source. No partial proposal was admitted.');}
   const roster=captureResourceRoster(buffers),originalGuards=capture.saved_guards??{},anonymous=capture.unaddressed_sources??{};
   if(!equalMap(roster.unaddressed_sources,anonymous))throw Error('Unaddressed retained editors differ from the sealed complete intention. Request a fresh agent proposal with the current drafts.');
   for(const [path,text] of Object.entries(capture.ownership_sources))if(!Object.hasOwn(roster.ownership_sources,path)||roster.ownership_sources[path]!==text)throw Error('A sealed authored editor is missing or changed: '+path);
   for(const [path,text] of Object.entries(roster.ownership_sources)){
    const row=roster.captures.find(row=>row.path===path&&buffers.values.get(path)===row.buffer)!;
    const patch=patches.find(patch=>patch.path===path),sealed=value.inputs.sources[path];
    if(!Object.hasOwn(capture.ownership_sources,path)&&(!patch?.before||sealed?.kind!=='file'||sealed.text!==text))throw Error('A current authored editor was not captured by this complete proposal: '+path);
    const expected=originalGuards[path]??(patch?.before?value.saved_guards[path]:undefined);
    if(!expected||row.guard!==expected||Object.hasOwn(value.saved_guards,path)&&value.saved_guards[path]!==row.guard)throw Error('Originally loaded physical source authority differs from the sealed proposal: '+path);
   }
   for(const patch of patches){if(!patch.before){if(buffers.values.has(patch.path))throw Error('A new proposal target is already occupied by another editor.');continue;}const buffer=buffers.values.get(patch.path),saved=value.inputs.saved_sources[patch.path];if(!buffer||buffer.readonly||buffer.transient||buffer.resourceDeleted||buffer.saving||buffer.text!==patch.before.text||saved?.kind!=='file'||typeof saved.text!=='string'||buffer.base!==saved.text||buffer.saveGuard!==value.saved_guards[patch.path])throw Error('Saved baseline, authored ownership or complete target changed: '+patch.path);}
   const editors=editorLease(buffers),current=()=>currentRoot()&&editors();
   const after_hashes:Record<string,string>=Object.create(null);for(const patch of patches)after_hashes[patch.path]=patch.after?await proposalHash(patch.after.text):'missing';if(!current())throw Error('Source or editor interaction changed while preparing the complete review.');
   const textOnly=patches.every(patch=>patch.before&&patch.after&&patch.before.identity===patch.after.identity);
   if(textOnly){const draft=planWorkspaceEdit(value.intention.title,capture.edit,workspaceSnapshots(buffers));if(draft.patches.length!==patches.length||patches.some(patch=>!draft.patches.some(item=>item.path===patch.path&&item.before===patch.before!.text&&item.after===patch.after!.text)))throw Error('The exact captured edit does not describe every sealed source transition. No partial draft Apply is available.');return {proposal,kind:'draft',draft,current,after_hashes} as BrowserProposal;}
   const resource:ResourceRestoreReview={operation:'save',record:'',fingerprint:'',consequence:value.consequences.fingerprint,workspace:proposal.workspace,context:value.plan.context,facts:value.consequences,plan:value.plan,patches,saved:value.consequences.saved_sources,savedBefore:value.inputs.saved_sources,proposal:{capture,inputs:value.inputs,plan:value.plan},current,restoring:new Map()};return {proposal,kind:'namespace',resource,current,after_hashes} as BrowserProposal;
  },
  draft:prepared=>{const selected=prepared as BrowserProposal;if(!selected.current()||!selected.draft)throw Error('The complete editor review is no longer current.');applyWorkspacePlan(buffers,selected.draft);history.record(captureDraftHistory(buffers,invertChangePlan(selected.draft)));},
  publish:async prepared=>{const selected=prepared as BrowserProposal;if(!selected.current()||!selected.resource)throw Error('The complete resource review is no longer current.');const outcome=await publishResourceRestore(buffers,selected.resource,async<T>(path:string,body:unknown)=>{const receipt=await request<T>('POST',path,body);selected.publication=receipt as ResourceReceipt;return receipt;});history.record({kind:'resource',recovery:outcome.recovery});adopt(outcome.routes);}
 };
}
export function browserProposalModel(buffers:Buffers,history:WorkspaceChangeHistory,acquire:(path:string)=>Promise<Buffer>,adopt:(routes:ResourceEditorRoute<Buffer>[])=>void,alive:()=>boolean){return new AgentProposalModel((method,path,body)=>request(method,path,body),browserProposalPorts(buffers,history,acquire,adopt,alive));}
