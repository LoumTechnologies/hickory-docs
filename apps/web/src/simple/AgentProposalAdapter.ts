import {request} from '../api/client';
import type {AgentEditorContext} from '../api/agentTypes';
import {captureResourceRoster,retainedResourceInputs,resourceBufferId,resourceWirePatches,publishResourceRestore,type ResourceRestoreReview,type ResourceEditorRoute,type ResourceReceipt} from './ResourceEditors';
import {editorForBuffer} from './Document';
import type {Buffer,Buffers} from './buffers';
import {captureDraftHistory,applyWorkspacePlan,workspaceSnapshots} from './WorkspaceChanges';
import {planWorkspaceEdit,invertChangePlan,type ChangePlan,WorkspaceChangeHistory} from './ChangePlan';
import {AgentProposalModel,proposalHash,verifyProposal,type ProposalPorts,type PreparedProposal,type VerifiedProposal,type ProposalProvider,type ProposalScope,type ProposalEditorSnapshot} from './AgentProposals';
import {readFrozenEntrySelection} from './WorkspaceEntrySelection';
interface BrowserProposal extends PreparedProposal {draft?:ChangePlan;resource?:ResourceRestoreReview}
const equalMap=(left:Record<string,string>,right:Record<string,string>)=>Object.keys(left).length===Object.keys(right).length&&Object.entries(left).every(([path,text])=>Object.hasOwn(right,path)&&right[path]===text);
export function proposalEditorContext(buffers:Buffers,focused?:Buffer):AgentEditorContext {
 const roster=captureResourceRoster(buffers);if(roster.captures.some(row=>!Number.isSafeInteger(row.revision)||row.revision<0))throw Error('Original editor revision is unavailable. Keep the source readable and reconcile its editor before proposing changes.');
 return {live:false,buffers:roster.captures.filter(row=>!row.readonly&&!row.transient).map(row=>({id:resourceBufferId(row.buffer),kind:'authored',observations:[{kind:'editor_revision' as const,version:1 as const,source_revision:String(row.revision)}],name:row.file.name,path:Object.hasOwn(roster.ownership_sources,row.path)?row.path:null,content:row.text,focused:row.buffer===focused,save_guard:Object.hasOwn(roster.ownership_sources,row.path)?row.guard??null:null,...row.file.doc_id?{document:row.file.doc_id}:{}}))};
}
export interface RecordedPatchAddress {
 side: 'user' | 'assistant';
 source_hash: string;
 opening_line: number;
}
export type ReviewRecordedPatch = (turn: string, fence: RecordedPatchAddress, current: () => boolean, anchor: HTMLElement) => Promise<void>;
export type ProposalSelection = Pick<VerifiedProposal, 'token' | 'fingerprint'>;
export interface WorkspaceDraftStage {
 current(): boolean;
 commit(): void;
 rollback(): void;
}
export type ResourceAuthority = {hash: string; guard: string; docId?: string};
export interface ResourceAdoptionStage {
 current: () => boolean;
 begin?: () => void;
 waitForSavedSource?: (authority: ResourceAuthority | null) => Promise<void>;
 release?: () => void;
 adopt: (authority: ResourceAuthority | null) => void;
}
export type RecordedPatchSource = {
 path: string;
 text: string;
 base: string;
 guard: string;
 revision: number;
 scope: ProposalScope;
 current: () => boolean;
 prepareDraft?: (after: string) => WorkspaceDraftStage | null;
 prepareResource?: (after: string, saved: string, destination?: string | null) => ResourceAdoptionStage | null;
};
type RecordedPatchInputs = Pick<VerifiedProposal['value']['intention']['capture'], 'ownership_sources' | 'unaddressed_sources' | 'saved_guards'> & {drafts: Record<string, unknown>};
type RecordedPatchEditor = Omit<ProposalEditorSnapshot['buffers'][number], 'source_hash'> & {text: string};
const recordedFenceAuthority = /^recorded (?:user|assistant) fence, source SHA-256 [0-9a-f]{64}; claims remain unverified$/;

export async function prepareRecordedPatch(
 buffers: Buffers, turn: string, fence: RecordedPatchAddress, alive: () => boolean, focused?: Buffer,
 call: ProposalProvider = (method, path, body) => request(method, path, body),
): Promise<ProposalSelection> {
 const scope = {namespace: buffers.workspace, workspace: buffers.workspaceId};
 if (!scope.namespace || !/^[0-9a-f]{32}$/.test(scope.workspace)) {
  throw Error('Workspace edit authority is unavailable. Refresh Files and try the recorded patch again.');
 }
 const roster = captureResourceRoster(buffers), lease = editorLease(buffers);
 const current = () => alive() && lease();
 const context = proposalEditorContext(buffers, focused);
 const paths = roster.captures.filter(row => row.guard && Object.hasOwn(roster.ownership_sources, row.path)).map(row => row.path);
 const editors = context.buffers.map(row => {
  const revision = row.observations?.find(value => 'kind' in value && value.kind === 'editor_revision');
  if (!row.id || !revision || !('source_revision' in revision)) {
   throw Error('Original editor identity or revision is unavailable. Refresh the captured patch review.');
  }
  return {id: row.id, path: row.path ?? null, save_guard: row.save_guard ?? null,
   source_revision: revision.source_revision, text: row.content};
 });
 const focus = focused && Object.hasOwn(roster.ownership_sources, focused.file.path) ? focused.file.path : null;
 const review = await requestRecordedPatch(scope, turn, fence, current, editors,
  () => retainedResourceInputs(buffers, roster, paths, current), focus, call);
 return {token: review.token, fingerprint: review.fingerprint};
}

export async function requestRecordedPatch(
 scope: ProposalScope, turn: string, fence: RecordedPatchAddress, current: () => boolean,
 sources: RecordedPatchEditor[], retain: () => Promise<RecordedPatchInputs>, focus: string | null,
 call: ProposalProvider = (method, path, body) => request(method, path, body),
): Promise<VerifiedProposal> {
 const entry = await readFrozenEntrySelection(() => call('GET', '/api/workspace/entry'), {...scope, current});
 const retained = await retain();
 const editors = await Promise.all(sources.map(async ({text, ...row}) => ({...row, source_hash: await proposalHash(text)})));
 const snapshot = {live: false, buffers: editors};
 if (!current()) throw Error('Message or editor source changed before patch review. Open the fence again.');
 const requestId = crypto.randomUUID().replaceAll('-', '');
 const authority = `recorded ${fence.side} fence, source SHA-256 ${fence.source_hash}; claims remain unverified`;
 const origin = `${fence.side} message in turn ${turn}, line ${fence.opening_line + 1}`;
 const raw = await call('POST', `/api/docs/workspace/agent/turns/${encodeURIComponent(turn)}/fence-patch`, {
  fence,
  intention: {
   version: 1, request_id: requestId, origin: 'Recorded message', title: 'Review recorded patch',
   explanation: 'Review the exact fenced changes and their generated consequences before applying them.',
   claims: [], check_ids: [], editor_snapshot: snapshot,
   capture: {context: requestId, edit: {changes: {}}, project_entry: entry.entryPath, ...retained},
   reading: {version: 1, entry: entry.entryPath ?? 'project.md', sources: {}, missing_sources: [],
    focus,
    ...(entry.entryPath ? {entry_selection: entry.proof} : {})},
  },
 });
 const review = await verifyProposal(raw, scope);
 const sealed = review.value.intention.editor_snapshot;
 const exactEditors = sealed?.live === false && sealed.buffers.length === editors.length
  && sealed.buffers.every((row, i) => row.id === editors[i].id && row.path === editors[i].path
   && row.save_guard === editors[i].save_guard && row.source_revision === editors[i].source_revision
   && row.source_hash === editors[i].source_hash);
 if (!current() || !exactEditors || review.historical || review.host_application != null
   || review.value.intention.request_id !== requestId || review.value.origin_authority !== authority
   || review.value.intention.origin !== origin) {
  throw Error('The patch review does not match the current message and workspace. Open the fence again.');
 }
 return review;
}
/** Optional richer capability never replaces an editor; its lease retires on user interaction. */
function editorLease(buffers:Buffers){const roster=captureResourceRoster(buffers),views=roster.captures.map(row=>{const view=editorForBuffer(row.buffer)?.view;return {buffer:row.buffer,view,doc:view?.state.doc,selection:view?.state.selection,readonly:view?.state.readOnly,connected:view?.dom.isConnected,parent:view?.dom.parentNode,top:view?.scrollDOM.scrollTop,left:view?.scrollDOM.scrollLeft};});return (acquired:ReadonlyMap<string,Buffer>=new Map())=>roster.current(acquired)&&views.every(row=>{const view=editorForBuffer(row.buffer)?.view;return view===row.view&&(!view||view.dom.isConnected===row.connected&&view.dom.parentNode===row.parent&&!view.composing&&view.state.readOnly===row.readonly&&view.state.doc===row.doc&&!!row.selection&&view.state.selection.eq(row.selection)&&view.scrollDOM.scrollTop===row.top&&view.scrollDOM.scrollLeft===row.left);});}
/** Managed proposals bind the original handles and edit epochs, not just equal current text. */
async function originalEditorBinding(buffers:Buffers,proposal:VerifiedProposal,current:()=>boolean){
 const snapshot=proposal.value.intention.editor_snapshot,authority=proposal.value.origin_authority;
 const requiresSnapshot=authority.startsWith('engine-managed agent tool')||recordedFenceAuthority.test(authority);
 if(!requiresSnapshot&&!authority.startsWith('caller-provided intention'))throw Error('This proposal origin has no supported host application authority. Its material remains readable.');
 if(!snapshot){if(requiresSnapshot)throw Error('This proposal has no original editor snapshot. Its review remains readable; request a new captured intention before Apply.');return;}
 const lease=editorLease(buffers),context=proposalEditorContext(buffers),ids=new Set<string>();
 if(snapshot.buffers.length!==context.buffers.length)throw Error('The complete retained editor roster differs from the original agent snapshot. Request a fresh intention; a new lease cannot absorb unknown owners.');
 for(const original of snapshot.buffers){if(!original.id||ids.has(original.id)||original.source_revision===null)throw Error('Original editor identity/revision is unavailable. Retain the readable proposal and request a new complete snapshot.');ids.add(original.id);const row=context.buffers.find(row=>row.id===original.id);const revisions=row?.observations?.filter(value=>'kind' in value&&value.kind==='editor_revision');const revision=revisions?.length===1&&'source_revision' in revisions[0]?revisions[0].source_revision:null;if(!row||row.path!==original.path||(row.save_guard??null)!==original.save_guard||revision!==original.source_revision||await proposalHash(row.content)!==original.source_hash)throw Error('Original editor identity, revision, text or physical guard changed before review. Edit/Undo and independently reopened handles cannot rebind this intention.');if(!current()||!lease())throw Error('Editor interaction or workspace changed while checking the original snapshot.');}
}
export function browserProposalPorts(buffers:Buffers,history:WorkspaceChangeHistory,acquire:(path:string)=>Promise<Buffer>,adopt:(routes:ResourceEditorRoute<Buffer>[])=>void,alive:()=>boolean):ProposalPorts {
 const scope=()=>{if(!buffers.workspace||!buffers.workspaceId)throw Error('Workspace source authority is not available yet. Retain the review and wait for the workspace.');return {namespace:buffers.workspace,workspace:buffers.workspaceId};};
 return {scope,
  prepare:async(proposal:VerifiedProposal,valid)=>{
   if(proposal.historical)throw Error('Historical proposal readings have no current editor authority. Request a fresh proposal.');
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
