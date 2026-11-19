<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Review one complete agent intention

For maintainers connecting engine-managed proposals to the simple editor without creating another mutation authority. The engine retains immutable material; existing editor and resource mechanisms remain responsible for local effects.

## Review the sealed intention before touching an editor

An agent claim is text, not a test receipt. Verify the SHA-256 of the complete retained material before parsing or displaying it, retain its exact token/fingerprint for admission, and consume the local attempt before crossing the editor or publisher boundary. A lost acknowledgment never authorizes Apply again. Closing or a newer choice retires held validation; independent retained outcomes remain readable.

Historical readings preserve the original sealed material and match a separate current reading scope. They never enter editor preparation or gain Apply. Failed durable retention stays visible while an otherwise valid live review remains usable.

<ui:copy id="agent-proposals-browser-model">
import {parseProtocolJson} from '../api/client';
import type {ResourceWirePlan,ResourceConstructionFacts,ResourceInputs,ResourceSaved,ResourceReceipt} from './ResourceEditors';
export interface ProposalScope {workspace:string;namespace:string}
export interface ProposalSummary {historical?:boolean;token:string;fingerprint:string;title:string;origin:string;origin_authority:string;request_id:string;host_application:unknown}
export interface ProposalEditorSnapshot {live:boolean;buffers:{id:string|null;path:string|null;source_hash:string;save_guard:string|null;source_revision:string|null}[]}
export interface ProposalMaterial extends ProposalScope {version:1;origin_authority:string;intention:{editor_snapshot?:ProposalEditorSnapshot|null;version:1;request_id:string;origin:string;title:string;explanation:string;claims:string[];check_ids:string[];capture:{context:string;edit:unknown;ownership_sources:Record<string,string>;unaddressed_sources:Record<string,string>;saved_guards:Record<string,string>;[key:string]:unknown};reading:unknown};inputs:ResourceInputs;plan:ResourceWirePlan;consequences:ResourceConstructionFacts&{saved_sources:Record<string,ResourceSaved>};saved_guards:Record<string,string>;before:Record<string,unknown>;after:Record<string,unknown>}
export interface VerifiedProposal extends ProposalScope {historical?:boolean;durability?:{retained:boolean;notice?:string};version:1;proposal_protocol:1;token:string;fingerprint:string;material:string;value:Readonly<ProposalMaterial>;host_application:unknown}
export interface PreparedProposal {proposal:VerifiedProposal;kind:'draft'|'namespace';current:()=>boolean;after_hashes:Record<string,string>;publication?:ResourceReceipt;[key:string]:unknown}
export type ProposalProvider=(method:'GET'|'POST',path:string,body?:unknown)=>Promise<unknown>;
export interface ProposalPorts {scope:()=>ProposalScope;prepare:(proposal:VerifiedProposal,alive:()=>boolean)=>Promise<PreparedProposal>;draft:(prepared:PreparedProposal)=>void;publish:(prepared:PreparedProposal)=>Promise<void>}
const id=(value:unknown):value is string=>typeof value==='string'&&/^[a-f0-9]{32}$/.test(value);
const digest=(value:unknown):value is string=>typeof value==='string'&&/^[a-f0-9]{64}$/.test(value);
const record=(value:unknown):value is Record<string,unknown>=>!!value&&typeof value==='object'&&!Array.isArray(value);
const textMap=(value:unknown):value is Record<string,string>=>record(value)&&Object.entries(value).every(([key,text])=>!!key&&typeof text==='string');
const sameScope=(a:ProposalScope,b:ProposalScope)=>a.workspace===b.workspace&&a.namespace===b.namespace;
export async function proposalHash(text:string):Promise<string>{return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),byte=>byte.toString(16).padStart(2,'0')).join('');}
function freezeMaterial<T>(value:T):T {const queue:unknown[]=[value];let count=0;while(queue.length){const item=queue.pop();if(!item||typeof item!=='object')continue;if(++count>100000)throw Error('Proposal structure exceeds its bounded inspection capacity.');Object.freeze(item);for(const child of Object.values(item)){if(typeof child==='number'&&(!Number.isFinite(child)||Number.isInteger(child)&&!Number.isSafeInteger(child)))throw Error('Proposal has an unsupported numeric authority value. No source was changed.');if(child&&typeof child==='object')queue.push(child);}}return value;}
function layer(value:unknown):boolean{return record(value)&&['entries','removed_entries','output_paths'].every(key=>Array.isArray(value[key])&&value[key].length<=32768&&value[key].every((path:unknown)=>typeof path==='string'&&path.length<=4096))&&Object.values(value).filter(Array.isArray).flat().reduce((size:number,path:unknown)=>size+(typeof path==='string'?path.length:0),0)<=4*1024*1024;}
function reading(value:unknown):boolean {if(!record(value)||value.version!==1||value.behavior_protocol!==1||!record(value.graph)||!record(value.graph.nodes)||Object.keys(value.graph.nodes).length>4096||!Array.isArray(value.graph.edges)||value.graph.edges.length>16384||!Array.isArray(value.graph.notices)||value.graph.notices.some(item=>typeof item!=='string')||!Array.isArray(value.graph.checks)||value.graph.checks.length>4096)return false;return Object.values(value.graph.nodes).every(node=>record(node)&&['id','kind','path','label','status'].every(key=>typeof node[key]==='string')&&(node.hash===null||digest(node.hash))&&(node.preview===null||typeof node.preview==='string')&&(node.source_path==null||typeof node.source_path==='string'))&&value.graph.checks.every(check=>record(check)&&typeof check.id==='string'&&typeof check.label==='string'&&typeof check.reason==='string')&&(value.impact==null||record(value.impact)&&Array.isArray(value.impact.via)&&value.impact.via.length<=16384&&value.impact.via.every((edge:unknown)=>record(edge)&&['from','to','relationship','basis','reason','authority'].every(key=>typeof edge[key]==='string')));}
function editorSnapshot(value:unknown):boolean{return value==null||record(value)&&typeof value.live==='boolean'&&Array.isArray(value.buffers)&&value.buffers.length<=4096&&value.buffers.every(row=>record(row)&&(row.id===null||typeof row.id==='string'&&!!row.id&&row.id.length<=1024)&&(row.path===null||typeof row.path==='string'&&row.path.length<=16384)&&digest(row.source_hash)&&(row.save_guard===null||typeof row.save_guard==='string'&&row.save_guard.length<=16384)&&(row.source_revision===null||typeof row.source_revision==='string'&&/^(0|[1-9][0-9]{0,19})$/.test(row.source_revision)));}
function state(value:unknown):boolean{return record(value)&&(value.kind==='missing'||value.kind==='file'&&typeof value.identity==='string'&&!!value.identity&&typeof value.text==='string');}
export async function verifyProposal(raw:unknown,expected:ProposalScope):Promise<VerifiedProposal>{
 if(!record(raw))throw Error('Unsupported proposal envelope.');
 const historical=raw.historical===true;
 if(raw.historical!==undefined&&typeof raw.historical!=='boolean')throw Error('Unsupported proposal retention state.');
 const boundScope=historical?raw.reading_scope:raw;
 if(!record(boundScope)||!sameScope(boundScope as unknown as ProposalScope,expected))throw Error('Unsupported or foreign proposal reading scope.');
 const materialScope={workspace:raw.workspace,namespace:raw.namespace};
 if(typeof materialScope.workspace!=='string'||!materialScope.workspace||typeof materialScope.namespace!=='string'||!materialScope.namespace||materialScope.namespace.length>16384)throw Error('Proposal original scope is malformed.');
 if(raw.version!==1||raw.proposal_protocol!==1||!id(raw.token)||!digest(raw.fingerprint)||raw.source_effects!==false||raw.checks_executed!==false||typeof raw.material!=='string'||new TextEncoder().encode(raw.material).length>32*1024*1024)throw Error('Unsupported or foreign proposal envelope. Source remains unchanged.');
 // The sealed bytes establish what was reviewed. Never normalize or reserialize this intention for admission.
 if(await proposalHash(raw.material)!==raw.fingerprint)throw Error('Proposal material does not match its sealed fingerprint. Nothing was displayed or applied.');
 const value=parseProtocolJson(raw.material) as ProposalMaterial;
 if(!record(value)||value.version!==1||!sameScope(value,materialScope as ProposalScope)||!record(value.intention)||!editorSnapshot(value.intention.editor_snapshot)||value.intention.version!==1||!id(value.intention.request_id)||typeof value.intention.title!=='string'||!value.intention.title.trim()||value.intention.title.length>512||typeof value.intention.explanation!=='string'||!value.intention.explanation.trim()||value.intention.explanation.length>65536||typeof value.origin_authority!=='string'||!Array.isArray(value.intention.claims)||value.intention.claims.length>128||value.intention.claims.some(claim=>typeof claim!=='string'||claim.length>8192)||!Array.isArray(value.intention.check_ids)||value.intention.check_ids.length>256||value.intention.check_ids.some(check=>typeof check!=='string')||!record(value.intention.capture)||!textMap(value.intention.capture.ownership_sources)||!textMap(value.intention.capture.unaddressed_sources??{})||!textMap(value.intention.capture.saved_guards??{})||!record(value.inputs)||!record(value.inputs.sources)||!record(value.inputs.saved_sources)||value.inputs.workspace!==materialScope.workspace||value.inputs.source_files_modified!==false||!textMap(value.inputs.saved_guards)||!record(value.plan)||value.plan.version!==1||value.plan.context!==value.inputs.context||value.plan.context!==value.intention.capture.context||!Array.isArray(value.plan.patches)||!value.plan.patches.length||value.plan.patches.length>1000||value.plan.patches.some(patch=>!record(patch)||typeof patch.path!=='string'||!patch.path||!state(patch.before)||!state(patch.after))||!record(value.consequences)||value.consequences.version!==1||value.consequences.workspace!==materialScope.workspace||value.consequences.context!==value.plan.context||!digest(value.consequences.fingerprint)||!Array.isArray(value.consequences.protected_targets)||value.consequences.protected_targets.length||!record(value.consequences.saved_sources)||!layer(value.consequences.saved)||!layer(value.consequences.draft)||!textMap(value.saved_guards)||!reading(value.before)||!reading(value.after))throw Error('The complete proposal material is unavailable or malformed. No partial review may Apply.');
 freezeMaterial(value);return Object.freeze({version:1,proposal_protocol:1,token:raw.token,fingerprint:raw.fingerprint,workspace:materialScope.workspace as string,namespace:materialScope.namespace as string,material:raw.material,value,host_application:raw.host_application,historical,...(record(raw.durability)&&typeof raw.durability.retained==='boolean'?{durability:{retained:raw.durability.retained,...(typeof raw.durability.notice==='string'?{notice:raw.durability.notice.slice(0,4096)}:{})}}:{})});
}
export class AgentProposalModel {
 rows:ProposalSummary[]=[];selection='';value:VerifiedProposal|null=null;prepared:PreparedProposal|null=null;notice='';loading=false;busy=false;status='';reportNotices=new Map<string,string>();private pendingReports=new Set<string>();lastAttempt:PreparedProposal|null=null;changed:()=>void=()=>{};private ticket=0;private consumed=new Set<string>();private reports=new Map<string,Readonly<Record<string,unknown>>>();
 constructor(private call:ProposalProvider,private ports:ProposalPorts,private host = 'browser simple editor'){}
 close(){this.ticket++;this.loading=false;this.selection='';this.value=null;this.prepared=null;this.status='';this.notice='Review closed. Source application and acknowledgment are never retried automatically.';this.changed();}
 async refresh(){const ticket=++this.ticket;this.selection='';this.prepared=null;if(this.value)this.status='retained material — choose again';this.loading=true;this.notice='Reading retained agent proposals…';this.changed();try{const scope={...this.ports.scope()};const raw=await this.call('GET','/api/agent-proposals');if(ticket!==this.ticket)return;if(!record(raw)||raw.version!==1||raw.proposal_protocol!==1||!sameScope(raw as unknown as ProposalScope,scope)||!sameScope(this.ports.scope(),scope)||!Array.isArray(raw.reviews)||raw.reviews.length>32)throw Error('Proposal catalogue changed or is unsupported.');this.rows=raw.reviews.map(row=>{if(!record(row)||!id(row.token)||!digest(row.fingerprint)||typeof row.title!=='string'||typeof row.origin!=='string'||typeof row.origin_authority!=='string'||!id(row.request_id)||row.applied!==false||row.saved!==false||(row.historical!==undefined&&typeof row.historical!=='boolean'))throw Error('Malformed retained proposal catalogue.');return Object.freeze({token:row.token,fingerprint:row.fingerprint,title:row.title.slice(0,512),origin:row.origin.slice(0,1024),origin_authority:row.origin_authority,request_id:row.request_id,host_application:row.host_application,historical:row.historical===true});});this.notice=this.rows.length?'Choose one complete intention. Claims are not verification evidence.':'No proposal reading is retained for this workspace.';if(typeof raw.retention_notice==='string')this.notice+=' '+raw.retention_notice.slice(0,4096);}catch(error){if(ticket===this.ticket)this.notice=String(error);}finally{if(ticket===this.ticket){this.loading=false;this.changed();}}}
 async open(token: string, verified?: VerifiedProposal) {
  const row = this.rows.find(row => row.token === token);
  if (!row && verified?.token !== token) return;
  this.selection = token;
  const ticket = ++this.ticket;
  let captured: ProposalScope | undefined;
  const alive = () => {
   if (ticket !== this.ticket || !captured) return false;
   try { return sameScope(this.ports.scope(), captured); } catch { return false; }
  };
  const retired = () => {
   if (ticket === this.ticket) this.notice = 'Workspace changed. Retained material remains readable; no source action was admitted.';
  };
  this.loading = true;
  this.prepared = null;
  this.status = 'reading';
  this.notice = 'Verifying the sealed complete intention…';
  this.changed();
  try {
   const scope = {...this.ports.scope()};
   captured = scope;
   let proposal = verified;
   if (proposal) {
    // The recorded-fence response already passed the same material verifier.
    if (!sameScope(proposal, scope)) throw Error('The recorded proposal belongs to a different workspace.');
   } else {
    const raw = await this.call('GET', '/api/agent-proposals/' + token);
    if (!alive()) { retired(); return; }
    proposal = await verifyProposal(raw, scope);
   }
   if (!alive()) { retired(); return; }
   if (proposal.token !== token || row && proposal.fingerprint !== row.fingerprint) throw Error('Selected proposal identity changed.');
   this.value = proposal;
   this.status = 'read-only';
   if (proposal.historical) {
    this.status = 'historical reading';
    this.notice = 'Retained sealed intention without current live admission. Its source and host reports are historical readings; request a fresh proposal to change current drafts. Apply is unavailable.';
    return;
   }
   if (proposal.host_application != null || this.consumed.has(token)) {
    this.status = proposal.host_application != null ? 'host report retained' : 'prior local attempt consumed';
    this.notice = 'This intention already has an application report or consumed local attempt. Inspect its retained material; Apply is unavailable.';
    return;
   }
   const prepared = await this.ports.prepare(proposal, alive);
   if (!alive()) { retired(); return; }
   if (prepared.proposal !== proposal || !prepared.current() || !textMap(prepared.after_hashes)) throw Error('Editor/source ownership changed during complete review.');
   this.prepared = prepared;
   this.status = 'reviewed';
   this.notice = prepared.kind === 'draft' ? 'Review all source changes, generated consequences, explanation and evidence. Apply changes drafts only.' : 'Review the complete namespace and saved/draft consequences. The separate explicit Save publishes resources; it does not stage or run checks.';
   if (proposal.durability?.retained === false) this.notice += ' ' + (proposal.durability.notice ?? 'This live proposal could not be retained for a later engine.');
  } catch (error) {
   if (ticket === this.ticket) this.notice = String(error);
  } finally {
   if (ticket === this.ticket) { this.loading = false; this.changed(); }
  }
 }
 canApply(){return !!this.prepared&&!this.prepared.proposal.historical&&!this.loading&&!this.busy&&!this.consumed.has(this.prepared.proposal.token)&&this.prepared.current();}
 async apply(){const prepared=this.prepared;if(!prepared||prepared.proposal.historical||this.busy)return;if(this.consumed.has(prepared.proposal.token)){this.notice='This exact local attempt is consumed. Inspect its retained outcome; no replacement application or publication was attempted.';this.changed();return;}if(!this.canApply()){this.notice='Source, editor interaction or workspace changed. Review the complete intention again; no Apply was attempted.';this.changed();return;}const proposal=prepared.proposal,ticket=this.ticket;this.busy=true;this.notice='Revalidating this exact sealed intention…';this.changed();try{const raw=await this.call('POST','/api/agent-proposals/validate',{version:1,token:proposal.token,fingerprint:proposal.fingerprint});if(ticket!==this.ticket||this.prepared!==prepared||!prepared.current())throw Error('Editor, selection, source or workspace changed after admission. No local Apply was attempted.');if(!record(raw)||raw.version!==1||raw.proposal_protocol!==1||raw.token!==proposal.token||raw.fingerprint!==proposal.fingerprint||!sameScope(raw as unknown as ProposalScope,proposal)||raw.admitted!==true||raw.source_effects!==false||raw.checks_executed!==false)throw Error('Exact proposal admission is unavailable. No local Apply was attempted.');if(this.consumed.size>=64)throw Error('This host retains 64 consumed intentions. Inspect their outcomes before preparing another host.');this.consumed.add(proposal.token);this.lastAttempt=prepared;this.status='application-attempted';if(prepared.kind==='draft')this.ports.draft(prepared);else await this.ports.publish(prepared);this.status=prepared.kind==='draft'?'draft-applied':'resources-published';this.prepared=null;this.notice=prepared.kind==='draft'?'Complete proposal applied to drafts. Workspace Undo/Redo is available. Nothing was saved or verified.':'Complete resource publication confirmed. It was not staged or verified.';const body=Object.freeze({version:1,token:proposal.token,fingerprint:proposal.fingerprint,workspace:proposal.workspace,namespace:proposal.namespace,request_id:crypto.randomUUID().replaceAll('-',''),host:this.host,after_hashes:Object.freeze({...prepared.after_hashes})});this.reports.set(proposal.token,body);this.pendingReports.add(proposal.token);await this.report(proposal.token);}catch(error){if(ticket===this.ticket)this.notice=String(error)+(this.consumed.has(proposal.token)?' This local attempt is consumed. Preserve drafts and inspect recovery; never reapply to recover an acknowledgment.':'');}finally{this.busy=false;this.changed();}}
 canReport(token:string){return !(this.value?.token===token&&this.value.historical)&&this.reports.has(token)&&this.pendingReports.has(token);}
 async report(token:string){const body=this.reports.get(token);if(!body)return;try{const raw=await this.call('POST','/api/agent-proposals/report',body);if(!record(raw)||raw.version!==1||raw.proposal_protocol!==1||raw.token!==body.token||raw.fingerprint!==body.fingerprint||raw.request_id!==body.request_id||raw.workspace!==body.workspace||raw.namespace!==body.namespace||raw.saved!==false||raw.checks_executed!==false||raw.status!=='host-reported draft application')throw Error('Host application acknowledgment was not confirmed.');this.pendingReports.delete(token);this.reportNotices.set(token,'Host-reported application acknowledged; this is not an engine-observed save or test verdict.');}catch(error){this.reportNotices.set(token,String(error)+' The actual local transition is retained; only this exact acknowledgment may be retried.');}if(this.value?.token===token||!this.value&&this.lastAttempt?.proposal.token===token)this.notice+=' '+this.reportNotices.get(token);this.changed();}
}
</ui:copy>
<ui:file path="apps/web/src/simple/AgentProposals.ts"><ui:paste select="#agent-proposals-browser-model"/></ui:file>

## Lease the complete retained editor namespace

The browser compares every authored owner, including anonymous and deleted retained buffers, against the sealed capture. Engine-managed material also binds the original opaque handle IDs and edit revisions before any acquisition: equal text after Edit/Undo or a separately reopened handle cannot restore that original identity. Missing original revisions keep the review readable but unavailable for Apply. A later lease protects subsequent interaction; it cannot repair an obsolete original snapshot. Originally loaded physical guards cannot be replaced by later reads. Opening an existing reviewed target admits only that exact new handle, then takes a fresh complete lease. A user selection, scroll, composition, mount, source or workspace change retires application. Reuse coordinated text transactions and workspace inverses; namespace publication keeps the existing resource publisher and receipt/adoption guards.

For a recorded patch, the clicked fence names its precise archived revision.
Read the workspace entry and send the original guarded editor capture alongside
that address. The engine parses and replays the fenced diff itself. The browser
checks the returned sealed identity before selecting its existing proposal
reader. Only the explicit Apply or Save action can alter a source.

<ui:copy id="agent-proposals-browser-adapter">
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
export type RecordedPatchSource = {
 path: string;
 text: string;
 base: string;
 guard: string;
 revision: number;
 scope: ProposalScope;
 current: () => boolean;
 prepareDraft?: (after: string) => WorkspaceDraftStage | null;
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
</ui:copy>
<ui:file path="apps/web/src/simple/AgentProposalAdapter.ts"><ui:paste select="#agent-proposals-browser-adapter"/></ui:file>

## Read explanation, implementation, consequences and evidence together

Use a compact floating reader reached from existing commands. Source transitions are complete and paged; generated consequences and connected source readings retain compiler provenance and explicit unknowns. The reader keeps agent claims separate from collected evidence and never starts a check implicitly. The primary editor stays mounted beneath the surface.

<ui:copy id="agent-proposals-browser-view">
import {useEffect,useMemo,useReducer,useRef,useState} from 'react';
import {createPortal} from 'react-dom';
import {request} from '../api/client';
import type {Buffer,Buffers} from './buffers';
import type {WorkspaceChangeHistory} from './ChangePlan';
import {resourceWirePatches,type ResourceEditorRoute} from './ResourceEditors';
import {browserProposalModel,type ProposalSelection} from './AgentProposalAdapter';
import type {VerifiedProposal} from './AgentProposals';
import {BehaviorSourceReading} from './BehaviorSourceReading';
import type {BehaviorReading} from './BehaviorReading';
const object=(value:unknown):value is Record<string,unknown>=>!!value&&typeof value==='object'&&!Array.isArray(value);
const text=(value:unknown,limit=4000)=>typeof value==='string'?value.slice(0,limit):'Unavailable';
function SourceTransition({proposal}:{proposal:VerifiedProposal}){
 const patches=resourceWirePatches(proposal.value.plan),[path,setPath]=useState(patches[0].path),[side,setSide]=useState<'draft_before'|'draft_after'|'saved_before'|'saved_after'>('draft_after'),[offset,setOffset]=useState(0),[previous,setPrevious]=useState<number[]>([]);
 const patch=patches.find(patch=>patch.path===path)??patches[0],saved=side==='saved_before'?proposal.value.inputs.saved_sources[patch.path]:proposal.value.consequences.saved_sources[patch.path],source=side==='draft_before'?patch.before?.text:side==='draft_after'?patch.after?.text:saved?.kind==='file'?saved.text??undefined:undefined;let end=Math.min((source?.length??0),offset+4000);if(source&&end<source.length&&/[\uD800-\uDBFF]/.test(source[end-1])&&/[\uDC00-\uDFFF]/.test(source[end]))end++;
 const reset=()=>{setOffset(0);setPrevious([]);};
 return <section aria-label="Complete proposal source transitions"><label>Source target <select aria-label="Proposal source target" value={patch.path} onChange={event=>{setPath(event.target.value);reset();}}>{patches.map(item=><option key={item.path}>{item.path}</option>)}</select></label><label>Source snapshot <select aria-label="Proposal source snapshot" value={side} onChange={event=>{setSide(event.target.value as typeof side);reset();}}><option value="draft_before">Before draft</option><option value="draft_after">Proposed draft</option><option value="saved_before">Reviewed saved baseline before</option><option value="saved_after">Projected saved baseline after publication</option></select></label><p>Saved before is an observed baseline; saved after is a reviewed projection. Apply alone never claims publication.</p>{side.startsWith('saved_')&&saved?.kind==='file'&&<p>Saved metadata: SHA {text(saved.sha256,128)} · mode {typeof saved.mode==='number'?saved.mode:'unavailable'} · read-only {String(saved.readonly)} · executable {String(saved.executable)}</p>}<p>{patches.length} complete source transitions. {patch.before&&patch.after&&patch.before.identity!==patch.after.identity?'A different retained source identity occupies this address.':''}</p>{source===undefined?<p>{side.startsWith('saved_')&&saved?.kind==='file'?'The saved source is not decoded as text. Inspect its hash and metadata in the retained engine review.':'This address is missing in the selected namespace.'}</p>:<><pre style={{whiteSpace:'pre-wrap'}}>{source.slice(offset,end)||'(empty source)'}</pre><p>Characters {offset}–{end} of {source.length}; source text is paged, never rewritten by this reader.</p><button disabled={!previous.length} onClick={()=>{setOffset(previous.at(-1)!);setPrevious(previous.slice(0,-1));}}>Previous source</button><button disabled={end===source.length} onClick={()=>{setPrevious([...previous,offset]);setOffset(end);}}>Next source</button></>}</section>;
}
function ConnectedReading({raw,current}:{raw:unknown;current:()=>boolean}){
 const [,paint]=useReducer((value:number)=>value+1,0),reader=useMemo(()=>new BehaviorSourceReading((path,body)=>request(body===undefined?'GET':'POST',path,body),paint),[]),[selected,setSelected]=useState(''),[edgePage,setEdgePage]=useState(0);
 useEffect(()=>()=>reader.close(),[reader]);
 if(!object(raw)||!object(raw.graph)||!object(raw.graph.nodes))return <p>Connected reading unavailable in this retained material; impact remains unknown.</p>;
 const reading=raw as unknown as BehaviorReading,nodes=Object.values(reading.graph.nodes).filter(node=>object(node)&&typeof node.id==='string'&&typeof node.label==='string'),node=nodes.find(node=>node.id===selected)??nodes[0],edges=Array.isArray(reading.impact?.via)?reading.impact!.via:[],evidence=object(raw.evidence)?raw.evidence:null,records=Array.isArray(evidence?.records)?evidence!.records.filter(object):[];
 return <section aria-label="Proposal connected reading"><p>Captured relationships; source hashes identify captured bytes, not frozen runtime inputs. Current source authority must be rechecked before any verification.</p><p>Impact: {text(reading.impact?.basis)} · {text(reading.impact?.reason)} {reading.impact?.truncated?'Impact is truncated; unknown relationships remain.':''}</p>{node&&<><label>Connected source <select aria-label="Proposal connected source" value={node.id} onChange={event=>{setSelected(event.target.value);reader.close();}}>{nodes.map(item=><option key={item.id} value={item.id}>{text(item.label,512)} · {item.kind} · {item.status}</option>)}</select></label><p>{node.source_path??node.path} · {node.status} · {node.text_origin??'Source origin unknown'}</p><pre style={{whiteSpace:'pre-wrap'}}>{text(node.preview)}</pre><button disabled={!node.hash||reader.loading} onClick={()=>void reader.read(reading,node,undefined,current)}>Read complete connected source</button></>}
 {reader.notice&&<p role="status">{reader.notice}</p>}{reader.page&&<section aria-label="Complete connected source"><p>{reader.page.state} · {reader.page.target_status} · {reader.page.notice}</p><pre style={{whiteSpace:'pre-wrap'}}>{reader.page.text}</pre><p>{reader.page.offset}–{reader.page.offset+[...reader.page.text].length} of {reader.page.total} Unicode scalars</p><button disabled={reader.page.offset===0||reader.loading} onClick={()=>node&&void reader.read(reading,node,Math.max(0,reader.page!.offset-4000),current)}>Previous connected source</button><button disabled={reader.page.next===null||reader.loading} onClick={()=>node&&void reader.read(reading,node,reader.page!.next!,current)}>Next connected source</button></section>}
 <details><summary>Why these mechanisms and checks are related</summary>{edges.slice(edgePage*50,(edgePage+1)*50).map((edge,index)=><p key={index}>{edge.from} → {edge.to}: {edge.relationship} · {edge.basis} · {edge.authority} · {edge.reason}</p>)}{edges.length>50&&<><button disabled={!edgePage} onClick={()=>setEdgePage(edgePage-1)}>Previous relationships</button><button disabled={(edgePage+1)*50>=edges.length} onClick={()=>setEdgePage(edgePage+1)}>Next relationships</button></>}{!edges.length&&<p>No impact edges were supplied. This is not proof of zero impact.</p>}</details>
 <details><summary>Captured evidence and related checks</summary><p>{text(evidence?.status)} · {text(evidence?.notice)} {evidence?.truncated?'Evidence is truncated.':''}</p>{records.slice(0,100).map((record,index)=><p key={index}>{text(record.id,128)} · {text(record.evidence_kind)} · {text(record.freshness)} · {text(record.origin)}</p>)}{records.length>100&&<p>First 100 evidence records shown; inspect the connected reading for remaining retained observations.</p>}{reading.graph.checks?.slice(0,100).map(check=><p key={check.id}>{check.label} · {check.reason}</p>)}<p>These are declarations and retained evidence, not checks executed by this proposal.</p></details>
 {reading.graph.notices?.map((notice,index)=><p key={index}>{notice}</p>)}{reading.graph.truncated&&<p>Graph truncated; omitted relationships are unknown.</p>}</section>;
}
function ProposalConsequences({proposal}:{proposal:VerifiedProposal}){const [layer,setLayer]=useState<'saved'|'draft'>('saved'),[page,setPage]=useState(0),value=proposal.value.consequences[layer],rows=[...value.entries.map(path=>'Construction entry: '+path),...value.removed_entries.map(path=>'Removed entry: '+path),...value.output_paths.map(path=>'Generated output: '+path)],last=Math.max(0,Math.ceil(rows.length/100)-1),selected=Math.min(page,last);return <section aria-label="Saved and draft generated consequences"><label>Construction layer <select aria-label="Proposal construction layer" value={layer} onChange={event=>{setLayer(event.target.value as typeof layer);setPage(0);}}><option value="saved">Saved construction</option><option value="draft">Draft construction</option></select></label><p>{value.output_paths.length} generated output paths. Compiler-reported relationships; no outputs were materialized and no checks were run.</p><pre style={{whiteSpace:'pre-wrap'}}>{rows.slice(selected*100,(selected+1)*100).join('\n')||'No construction entries or outputs reported.'}</pre>{last>0&&<><button disabled={!selected} onClick={()=>setPage(selected-1)}>Previous consequences</button><span>{selected+1}/{last+1}</span><button disabled={selected===last} onClick={()=>setPage(selected+1)}>Next consequences</button></>}</section>;}
function RetainedMaterial({proposal,current}:{proposal:VerifiedProposal;current:()=>boolean}){
 const value=proposal.value,[side,setSide]=useState<'before'|'after'>('after');
 return <><h3>{value.intention.title}</h3><p>{value.origin_authority} · {value.intention.origin}</p><p>{value.intention.explanation}</p><details open><summary>Agent claims — unverified text</summary>{value.intention.claims.map((claim,index)=><p key={index}>{claim}</p>)}{!value.intention.claims.length&&<p>No agent claims were supplied.</p>}</details><details><summary>Requested check identities — not run</summary>{value.intention.check_ids.map(check=><code key={check}>{check}</code>)}{!value.intention.check_ids.length&&<p>No checks requested.</p>}</details>
 <SourceTransition key={proposal.fingerprint} proposal={proposal}/>
 <ProposalConsequences proposal={proposal}/>
 <label>Connected reading snapshot <select aria-label="Proposal connected snapshot" value={side} onChange={event=>setSide(event.target.value as typeof side)}><option value="before">Before intention</option><option value="after">Proposed intention</option></select></label><ConnectedReading key={proposal.fingerprint+side} raw={value[side]} current={current}/>
 <p>Sealed fingerprint: <code>{proposal.fingerprint}</code>. Apply, Save, Stage and verification are separate outcomes.</p></>;
}
export function RecordedPatchReview({proposal, anchor, close, model, undo, redo}: {
 proposal: VerifiedProposal; anchor: HTMLElement; close: () => void;
 model?: import('./AgentProposals').AgentProposalModel; undo?: () => void; redo?: () => void;
}) {
 const panel=useRef<HTMLElement>(null);
 useEffect(()=>{
  const dialog = panel.current;
  dialog?.focus({preventScroll:true});
  return () => {
   const currentFocus = document.activeElement;
   if (anchor.isConnected && (currentFocus === document.body || dialog?.contains(currentFocus))) {
    anchor.focus({preventScroll:true});
   }
  };
 },[anchor,proposal]);
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Recorded patch review"
  className="refs-panel" style={{width:"min(38rem, calc(100vw - 2.5rem))",maxHeight:"75dvh"}}
  onKeyDown={event=>{if(event.key==="Escape"){event.preventDefault();close();}}}>
  <header className="refs-head"><strong>Recorded patch review</strong><button onClick={close}>Close</button></header>
  <div style={{padding:"0.7rem"}}><p role="status">{model?.notice || "Read-only review of the captured workspace."}</p>
   {model?.prepared?.proposal === proposal && <button disabled={!model.canApply()} onClick={() => void model.apply()}>Apply complete proposal to drafts</button>}
   {undo && <button disabled={model?.busy} onClick={undo}>Undo workspace draft Apply</button>}
   {redo && <button disabled={model?.busy} onClick={redo}>Redo workspace draft Apply</button>}
   {model?.canReport(proposal.token) && <button onClick={() => void model.report(proposal.token)}>Retry acknowledgment</button>}
   <RetainedMaterial key={proposal.fingerprint} proposal={proposal} current={()=>!!panel.current?.isConnected}/>
  </div>
 </section>,document.body);
}
export function AgentProposalReview({open,close,buffers,history,acquire,adopt,initialSelection}:{open:boolean;close:()=>void;buffers:Buffers;history:WorkspaceChangeHistory;acquire:(path:string)=>Promise<Buffer>;adopt:(routes:ResourceEditorRoute<Buffer>[])=>void;initialSelection?:ProposalSelection|null}){
 const [,paint]=useReducer((value:number)=>value+1,0),lifetime=useRef(open),panel=useRef<HTMLElement>(null);lifetime.current=open;
 const model=useMemo(()=>browserProposalModel(buffers,history,acquire,adopt,()=>lifetime.current),[buffers,history,buffers.workspaceId]);model.changed=paint;
 useEffect(() => {
  if (!open) { model.close(); return; }
  let current = true;
  panel.current?.focus({preventScroll:true});
  void model.refresh().then(() => {
   if (!current || !initialSelection || model.selection) return;
   const row = model.rows.find(item => item.token === initialSelection.token && item.fingerprint === initialSelection.fingerprint);
   if (row) void model.open(row.token);
   else {
    model.notice = 'The selected sealed patch reading is unavailable. Reopen the message and review the fence again.';
    model.changed();
   }
  });
  return () => { current = false; };
 }, [open, model, initialSelection]);
 useEffect(()=>()=>{model.close();model.changed=()=>{};},[model]);
 if(!open)return null;const proposal=model.value,attempt=model.lastAttempt?.proposal.token===proposal?.token?model.lastAttempt:null;
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Agent proposal review" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}><header><strong>Agent proposal review</strong><button onClick={close}>Close</button></header><p role="status">{model.notice}</p><button disabled={model.loading||model.busy} onClick={()=>void model.refresh()}>Refresh pending proposals</button><label>Complete intention <select aria-label="Retained agent proposal" value={model.selection} onChange={event=>void model.open(event.target.value)}><option value="">Choose an intention</option>{model.rows.map(row=><option key={row.token} value={row.token}>{row.title} · {row.historical?'historical reading':row.host_application?'host report retained':'pending'}</option>)}</select></label>
 {proposal&&<RetainedMaterial proposal={proposal} current={()=>model.value===proposal}/>}<button disabled={!model.canApply()} onClick={()=>void model.apply()}>{model.prepared?.kind==='namespace'?'Save reviewed resource namespace':'Apply complete proposal to drafts'}</button><p>{model.status}</p>{proposal&&model.reportNotices.has(proposal.token)&&<p>{model.reportNotices.get(proposal.token)}</p>}{proposal&&model.canReport(proposal.token)&&<button onClick={()=>void model.report(proposal.token)}>Retry acknowledgment of recorded transition</button>}{attempt?.publication&&<section aria-label="Retained resource publication response"><p>Received publication response; current editor adoption remains separate. {attempt.publication.adoption_current===false?attempt.publication.adoption_reason:''}</p><code>{attempt.publication.outcome?.recovery??'Recovery identity unavailable'}</code></section>}</section>,document.body);
}
</ui:copy>
<ui:file path="apps/web/src/simple/AgentProposalView.tsx"><ui:paste select="#agent-proposals-browser-view"/></ui:file>

## Reject tampering, obsolete admission and repeated effects

Focused model witnesses distinguish the sealed bytes from parsed projections, exercise held replies and source retirement, and prove that draft application, resource publication and host acknowledgment are separate operations. These controlled tests are not a real provider/tool or physical-input acceptance receipt.

<ui:copy id="agent-proposals-browser-tests">
import {it,expect,vi} from 'vitest';
import {AgentProposalModel,verifyProposal,proposalHash,type ProposalMaterial,type ProposalPorts,type ProposalProvider,type VerifiedProposal} from './AgentProposals';
const scope={workspace:'a'.repeat(32),namespace:'/workspace'},token='b'.repeat(32),request_id='c'.repeat(32),hash='d'.repeat(64);
function reading(){return {version:1,behavior_protocol:1,workspace_id:'e'.repeat(64),namespace:scope.namespace,engine_workspace_id:scope.workspace,reading_fingerprint:hash,context:{version:1,entry:'project.md',sources:{},missing_sources:[],focus:'note.py',saved_guards:{},unaddressed_sources:{}},graph:{nodes:{source:{id:'source',kind:'implementation',path:'note.py',label:'note.py',status:'canonical',hash,preview:'value=1\n'}},edges:[],notices:[],checks:[],truncated:false},impact:null,evidence:{status:'missing',records:[]}};}
export function material():ProposalMaterial {const saved={kind:'file',text:'value=1\n',sha256:hash,mode:420,readonly:false};return {version:1,...scope,origin_authority:'engine-managed agent tool; claims remain unverified',intention:{version:1,request_id,origin:'controlled provider',title:'Change source coherently',explanation:'Propose the source and its explanation together.',claims:['Agent claims passed; not evidence.'],check_ids:[],capture:{context:'capture',edit:{changes:{'hick:///note.py':[{range:{start:{line:0,character:6},end:{line:0,character:7}},newText:'41'}]}},ownership_sources:{'note.py':'value=1\n'},unaddressed_sources:{},saved_guards:{'note.py':'original-guard'}},reading:{}},inputs:{source_files_modified:false,workspace:scope.workspace,context:'capture',refusals:{},sources:{'note.py':{kind:'file',identity:'source',text:'value=1\n'}},saved_sources:{'note.py':saved},saved_guards:{'note.py':'original-guard'}},plan:{version:1,context:'capture',title:'Change source coherently',patches:[{path:'note.py',before:{kind:'file',identity:'source',text:'value=1\n'},after:{kind:'file',identity:'source',text:'value=41\n'}}]},consequences:{version:1,workspace:scope.workspace,context:'capture',fingerprint:hash,protected_targets:[],saved:{entries:[],removed_entries:[],output_paths:[]},draft:{entries:[],removed_entries:[],output_paths:[]},saved_sources:{'note.py':{...saved,text:'value=41\n'}}},saved_guards:{'note.py':'original-guard'},before:reading(),after:reading()} as ProposalMaterial;}
async function envelope(value=material()){const sealed=JSON.stringify(value);return {version:1,proposal_protocol:1,...scope,token,fingerprint:await proposalHash(sealed),material:sealed,source_effects:false,checks_executed:false,host_application:null};}
const catalogue=(fingerprint:string)=>({version:1,proposal_protocol:1,...scope,reviews:[{token,fingerprint,title:'Change source coherently',origin:'provider',origin_authority:'engine-managed tool',request_id,applied:false,saved:false,host_application:null}]});
async function setup(overrides:Partial<ProposalPorts>={},provider?:ProposalProvider){const raw=await envelope(),draft=vi.fn(),publish=vi.fn(async()=>{});let current=true;const ports:ProposalPorts={scope:()=>scope,prepare:async(proposal)=>({proposal,kind:'draft',current:()=>current,after_hashes:{'note.py':hash}}),draft,publish,...overrides};const model=new AgentProposalModel(provider??(async(_method,path,body)=>path==='/api/agent-proposals'?catalogue(raw.fingerprint):path.endsWith(token)?raw:path.endsWith('validate')?{version:1,proposal_protocol:1,...scope,...body as object,admitted:true,source_effects:false,checks_executed:false}:{...body as object,version:1,proposal_protocol:1,status:'host-reported draft application',saved:false,checks_executed:false}),ports);await model.refresh();await model.open(token);return {model,raw,draft,publish,retire:()=>{current=false;}};}
async function historicalEnvelope(){const value=material(),old='f'.repeat(32);value.workspace=old;value.inputs.workspace=old;value.consequences.workspace=old;value.before.engine_workspace_id=old;value.after.engine_workspace_id=old;return {...await envelope(value),workspace:old,historical:true,reading_scope:{...scope}};}
it('reads an old sealed intention through current scope without preparing or applying it',async()=>{const raw=await historicalEnvelope(),prepare=vi.fn(async()=>{throw Error('Historical review entered preparation');}),call=vi.fn(async(_method:string,path:string)=>path==='/api/agent-proposals'?{...catalogue(raw.fingerprint),reviews:catalogue(raw.fingerprint).reviews.map(row=>({...row,historical:true}))}:raw);const result=await setup({prepare},call as ProposalProvider);expect(result.model.value?.historical).toBe(true);expect(result.model.value?.workspace).toBe('f'.repeat(32));expect(result.model.status).toBe('historical reading');expect(prepare).not.toHaveBeenCalled();expect(result.model.canApply()).toBe(false);await result.model.apply();expect(result.draft).not.toHaveBeenCalled();expect(call.mock.calls.every(([method])=>method==='GET')).toBe(true);});
it('refuses a foreign historical reading scope, stripped history flag and changed seal',async()=>{const raw=await historicalEnvelope();await expect(verifyProposal({...raw,reading_scope:{...scope,workspace:'0'.repeat(32)}},scope)).rejects.toThrow('foreign');await expect(verifyProposal({...raw,historical:false},scope)).rejects.toThrow('foreign');await expect(verifyProposal({...raw,material:raw.material+' '},scope)).rejects.toThrow('sealed fingerprint');});
it('keeps live review usable and reports unavailable durable retention',async()=>{const raw={...await envelope(),durability:{retained:false,notice:'Private historical retention failed.'}};const result=await setup({},async(_method,path)=>path==='/api/agent-proposals'?catalogue(raw.fingerprint):raw);expect(result.model.canApply()).toBe(true);expect(result.model.notice).toContain('Private historical retention failed');});
it('verifies exact sealed bytes before parsing or exposing complete immutable material',async()=>{const raw=await envelope(),value=await verifyProposal(raw,scope);expect(value.value.intention.claims[0]).toContain('not evidence');expect(Object.isFrozen(value.value.plan.patches)).toBe(true);await expect(verifyProposal({...raw,material:raw.material+' '},scope)).rejects.toThrow('sealed fingerprint');await expect(verifyProposal({...raw,namespace:'/foreign'},scope)).rejects.toThrow('foreign');});
it('refuses old/malformed protocol, duplicate version fields and unsafe opaque integers',async()=>{const raw=await envelope();for(const changed of [{proposal_protocol:undefined},{proposal_protocol:true},{version:2},{source_effects:'false'},{checks_executed:true}])await expect(verifyProposal({...raw,...changed},scope)).rejects.toThrow();for(const altered of [raw.material.replace('"version":1','"version":1,"version":1'),raw.material.replace('"version":1','"version":1.0'),raw.material.replace('"version":1','"version":1,"opaque":18446744073709551615')])await expect(verifyProposal({...raw,material:altered,fingerprint:await proposalHash(altered)},scope)).rejects.toThrow();});
it('keeps unavailable early scope and a retired held scope inside the optional capability',async()=>{const raw=await envelope();let ready=false,finish!:(value:unknown)=>void;const prepare=vi.fn(),model=new AgentProposalModel(async(_method,path)=>path==='/api/agent-proposals'?catalogue(raw.fingerprint):new Promise(resolve=>{finish=resolve;}),{scope:()=>{if(!ready)throw Error('Workspace authority is not ready');return scope;},prepare,draft:vi.fn(),publish:vi.fn()});await expect(model.refresh()).resolves.toBeUndefined();expect(model.loading).toBe(false);expect(model.notice).toContain('not ready');ready=true;await model.refresh();const pending=model.open(token);ready=false;finish(raw);await pending;expect(model.loading).toBe(false);expect(model.value).toBeNull();expect(model.notice).toContain('Workspace changed');expect(prepare).not.toHaveBeenCalled();});
it('does not adopt a held proposal after close or a newer selection',async()=>{const raw=await envelope();let finish!:(value:unknown)=>void;const prepare=vi.fn();const model=new AgentProposalModel(async(_method,path)=>path==='/api/agent-proposals'?catalogue(raw.fingerprint):new Promise(resolve=>{finish=resolve;}),{scope:()=>scope,prepare,draft:vi.fn(),publish:vi.fn()});await model.refresh();const pending=model.open(token);model.close();finish(raw);await pending;expect(model.value).toBeNull();expect(prepare).not.toHaveBeenCalled();});
it('validates only the sealed token/fingerprint then applies all drafts once without saving or running',async()=>{const calls:{path:string;body:unknown}[]=[],raw=await envelope();const {model,draft,publish}=await setup({},async(_method,path,body)=>{calls.push({path,body});return path==='/api/agent-proposals'?catalogue(raw.fingerprint):path.endsWith(token)?raw:path.endsWith('validate')?{version:1,proposal_protocol:1,...scope,...body as object,admitted:true,source_effects:false,checks_executed:false}:{...body as object,version:1,proposal_protocol:1,status:'host-reported draft application',saved:false,checks_executed:false};});await model.apply();expect(draft).toHaveBeenCalledTimes(1);expect(publish).not.toHaveBeenCalled();expect(calls.find(call=>call.path.endsWith('validate'))?.body).toEqual({version:1,token,fingerprint:raw.fingerprint});expect(model.status).toBe('draft-applied');await model.open(token);await model.apply();expect(draft).toHaveBeenCalledTimes(1);expect(model.notice).toContain('consumed');});
it('retires admission when source/selection ownership changes while validation is held',async()=>{let finish!:(value:unknown)=>void;const raw=await envelope();const result=await setup({},async(_method,path)=>path==='/api/agent-proposals'?catalogue(raw.fingerprint):path.endsWith(token)?raw:new Promise(resolve=>{finish=resolve;}));const pending=result.model.apply();result.retire();finish({version:1,proposal_protocol:1,...scope,token,fingerprint:raw.fingerprint,admitted:true,source_effects:false,checks_executed:false});await pending;expect(result.draft).not.toHaveBeenCalled();expect(result.publish).not.toHaveBeenCalled();expect(result.model.notice).toContain('No local Apply');});
it('retains an actual local transition after lost acknowledgment and retries only the exact report',async()=>{const raw=await envelope(),reports:unknown[]=[];let refusal=true;const result=await setup({},async(_method,path,body)=>{if(path==='/api/agent-proposals')return catalogue(raw.fingerprint);if(path.endsWith(token))return raw;if(path.endsWith('validate'))return {version:1,proposal_protocol:1,...scope,...body as object,admitted:true,source_effects:false,checks_executed:false};reports.push(body);if(refusal)throw Error('transport lost');return {...body as object,version:1,proposal_protocol:1,status:'host-reported draft application',saved:false,checks_executed:false};});await result.model.apply();expect(result.draft).toHaveBeenCalledTimes(1);expect(result.model.notice).toContain('only this exact acknowledgment');refusal=false;await result.model.report(token);expect(reports[1]).toEqual(reports[0]);await result.model.apply();expect(result.draft).toHaveBeenCalledTimes(1);});
it('requires explicit namespace publication and consumes an uncertain publication without draft fallback',async()=>{const publish=vi.fn(async()=>{throw Error('publication unknown');});const result=await setup({prepare:async(proposal)=>({proposal,kind:'namespace',current:()=>true,after_hashes:{'note.py':hash}}),publish});await result.model.apply();await result.model.apply();expect(publish).toHaveBeenCalledTimes(1);expect(result.draft).not.toHaveBeenCalled();expect(result.model.lastAttempt?.kind).toBe('namespace');expect(result.model.notice).toContain('consumed');});
</ui:copy>

## Verify the real editor and original agent snapshot boundary

Real CodeMirror transactions retain their mounted editor, selection, scrolling,
local history and complete workspace inverse. Selection, scroll, mount,
physical baseline, generated ownership and unrelated roster changes retire a
prepared review. The agent send witness captures original source guards and
anonymous identities once; later edits never refresh that snapshot silently.
Raw HTTP protocol aliases and duplicates fail before they reach a host model.
A recorded review submits the immutable message address and host preimages.
The sealed response opens the existing host guard, which still rejects an
obsolete editor revision after intervening edits return to the same text.

<ui:copy id="agent-proposals-browser-host-tests">
import {afterEach} from 'vitest';
import {EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {history as editorHistory,undo as nativeUndo,redo as nativeRedo} from '@codemirror/commands';
import {Buffer,Buffers} from './buffers';
import {resourceBufferId} from './ResourceEditors';
import {browserProposalPorts,prepareRecordedPatch,proposalEditorContext,type RecordedPatchAddress} from './AgentProposalAdapter';
import {WorkspaceChangeHistory,invertChangePlan} from './ChangePlan';
import {captureDraftHistory,applyWorkspacePlan} from './WorkspaceChanges';
const proposalViews=vi.hoisted(()=>new Map<unknown,{view:EditorView}>());
vi.mock('./Document',()=>({editorForBuffer:(buffer:unknown)=>proposalViews.get(buffer)}));
afterEach(()=>{for(const {view} of proposalViews.values())view.destroy();proposalViews.clear();document.body.replaceChildren();});
async function callerProposal(){const value=material();value.origin_authority='caller-provided intention; not an agent attestation';return verifyProposal(await envelope(value),scope);}
async function managedProposal(buffer:Buffer){const value=material();value.intention.editor_snapshot={live:false,buffers:[{id:resourceBufferId(buffer),path:buffer.file.path,source_hash:await proposalHash(buffer.text),save_guard:buffer.saveGuard??null,source_revision:String(buffer.sourceRevision)}]};return verifyProposal(await envelope(value),scope);}
function host(){const buffers=new Buffers(()=>{});buffers.workspace=scope.namespace;buffers.workspaceId=scope.workspace;const buffer=new Buffer({path:'note.py',name:'note.py',dir:false},'value=1\n','loaded-fnv',()=>{},false,scope.namespace);buffer.saveGuard='original-guard';buffers.values.set('note.py',buffer);const view=new EditorView({parent:document.body,state:EditorState.create({doc:buffer.text,selection:{anchor:2},extensions:[editorHistory(),EditorView.updateListener.of(update=>{if(update.docChanged)buffer.edit(update.state.doc.toString());})]})});view.scrollDOM.scrollTop=73;proposalViews.set(buffer,{view});const history=new WorkspaceChangeHistory(),acquire=vi.fn(async(_path:string)=>{throw Error('Unexpected acquisition');}),adopt=vi.fn(),ports=browserProposalPorts(buffers,history,acquire,adopt,()=>true);return {buffers,buffer,view,history,ports,acquire,adopt};}
const absentEntry = {version:1, entry_protocol:1, ...scope, guard:'2'.repeat(64), movable:false,
 source_effects:false, entry:{version:1,identity:'3'.repeat(32),generation:0,path:null}};
it('binds a recorded patch to its message and guarded editor before opening review', async () => {
 const {buffers,buffer,ports} = host();
 const fence: RecordedPatchAddress = {side:'assistant',source_hash:'4'.repeat(64),opening_line:5};
 let sealed: VerifiedProposal | undefined;
 const call = vi.fn(async (method: string, path: string, raw?: unknown) => {
  if (method === 'GET') { expect(path).toBe('/api/workspace/entry'); return absentEntry; }
  expect(path).toBe('/api/docs/workspace/agent/turns/turn%20one/fence-patch');
  const body = raw as { fence: RecordedPatchAddress; intention: ProposalMaterial['intention'] };
  expect(body.fence).toEqual(fence);
  expect(body.intention.capture.edit).toEqual({changes:{}});
  expect(body.intention.capture.drafts).toMatchObject({'note.py':{text:'value=1\n',save_guard:'original-guard'}});
  const value = material();
  value.origin_authority = `recorded assistant fence, source SHA-256 ${fence.source_hash}; claims remain unverified`;
  value.intention.origin = 'assistant message in turn turn one, line 6';
  value.intention.request_id = body.intention.request_id;
  value.intention.editor_snapshot = body.intention.editor_snapshot;
  const reply = await envelope(value);
  sealed = await verifyProposal(reply,scope);
  return reply;
 });
 const selection = await prepareRecordedPatch(buffers,'turn one',fence,()=>true,buffer,call);
 expect(selection).toEqual({token: sealed!.token,fingerprint: sealed!.fingerprint});
 expect(call).toHaveBeenCalledTimes(2);
 const review = await ports.prepare(sealed!,()=>true);
 expect(review.current()).toBe(true);
 expect(buffer.text).toBe('value=1\n');
 buffer.edit('intervening text'); buffer.edit('value=1\n');
 expect(review.current()).toBe(false);
 await expect(ports.prepare(sealed!,()=>true)).rejects.toThrow('Original editor identity');
 expect(buffer.text).toBe('value=1\n');
});
it('retires a recorded patch when its source card closes during entry acquisition', async () => {
 const {buffers} = host();
 let alive = true, finish!: (value:unknown)=>void;
 const call = vi.fn(() => new Promise(resolve => {finish = resolve;}));
 const pending = prepareRecordedPatch(buffers,'turn',{side:'user',source_hash:'4'.repeat(64),opening_line:0},()=>alive,undefined,call);
 alive = false;
 finish(absentEntry);
 await expect(pending).rejects.toThrow('obsolete source context');
 expect(call).toHaveBeenCalledTimes(1);
 expect(call).toHaveBeenCalledWith('GET','/api/workspace/entry');
});
it('applies precise sealed edits to the same editor and preserves caret/scroll, coordinated and native undo',async()=>{const {buffers,buffer,view,history,ports}=host(),proposal=await callerProposal(),prepared=await ports.prepare(proposal,()=>true),original=view.dom;expect(prepared.current()).toBe(true);ports.draft(prepared);expect(buffer.text).toBe('value=41\n');expect(buffer.base).toBe('value=1\n');expect(view.dom).toBe(original);expect(view.state.selection.main.anchor).toBe(2);expect(view.scrollDOM.scrollTop).toBe(73);const token=history.preview('undo')!;expect(token.entry.kind).toBe('draft');if(token.entry.kind!=='draft')throw Error('Expected one coordinated draft inverse');const inverse=history.readDraft(token);applyWorkspacePlan(buffers,inverse);history.commit(token,captureDraftHistory(buffers,invertChangePlan(inverse)));expect(buffer.text).toBe('value=1\n');const redo=history.preview('redo')!;if(redo.entry.kind!=='draft')throw Error('Expected one draft redo');const forward=history.readDraft(redo);applyWorkspacePlan(buffers,forward);history.commit(redo,captureDraftHistory(buffers,invertChangePlan(forward)));expect(buffer.text).toBe('value=41\n');nativeUndo(view);expect(buffer.text).toBe('value=1\n');nativeRedo(view);expect(buffer.text).toBe('value=41\n');expect(view.dom).toBe(original);});
it('preserves a detached cached editor and unrelated unsaved notes while applying another source',async()=>{const value=host(),notes=new Buffer({path:'Untitled-notes',name:'notes',dir:false},'','',()=>{},false,scope.namespace);notes.isUntitledDraft=true;notes.edit('retained unsaved notes');const other=new EditorView({state:EditorState.create({doc:notes.text,extensions:[editorHistory()]})});value.buffers.values.set(notes.file.path,notes);proposalViews.set(notes,{view:other});const materialValue=material();materialValue.origin_authority='caller-provided intention; not an agent attestation';materialValue.intention.capture.unaddressed_sources={[resourceBufferId(notes)]:notes.text};const proposal=await verifyProposal(await envelope(materialValue),scope),prepared=await value.ports.prepare(proposal,()=>true);expect(other.dom.isConnected).toBe(false);expect(prepared.current()).toBe(true);value.ports.draft(prepared);expect(value.buffer.text).toBe('value=41\n');expect(notes.text).toBe('retained unsaved notes');expect(notes.dirty).toBe(true);expect(other.state.doc.toString()).toBe(notes.text);expect(other.dom.isConnected).toBe(false);});
it('retires actual editor leases on selection, scrolling and replaced mounts',async()=>{for(const change of ['selection','scroll','mount']){const value=host(),proposal=await callerProposal(),prepared=await value.ports.prepare(proposal,()=>true);if(change==='selection')value.view.dispatch({selection:{anchor:3}});else if(change==='scroll')value.view.scrollDOM.scrollTop=74;else value.view.dom.remove();expect(prepared.current()).toBe(false);expect(()=>value.ports.draft(prepared)).toThrow('no longer current');expect(value.buffer.text).toBe('value=1\n');value.view.destroy();proposalViews.delete(value.buffer);}});
it('binds original managed IDs/revisions before a fresh lease, including edit then Undo and independent same-byte handles',async()=>{for(const change of ['undo','independent']){const value=host(),proposal=await managedProposal(value.buffer);if(change==='undo'){value.buffer.edit('intervening draft');value.buffer.edit('value=1\n');}else{const replacement=new Buffer({...value.buffer.file},value.buffer.text,value.buffer.hash,()=>{},false,scope.namespace);replacement.saveGuard=value.buffer.saveGuard;value.buffers.values.set('note.py',replacement);}await expect(value.ports.prepare(proposal,()=>true)).rejects.toThrow(/Original editor/);expect(value.buffer.text).toBe('value=1\n');value.view.destroy();proposalViews.delete(value.buffer);}});
it('admits an unchanged managed snapshot and keeps missing original revision readable but unavailable',async()=>{const value=host(),proposal=await managedProposal(value.buffer);expect((await value.ports.prepare(proposal,()=>true)).current()).toBe(true);const missing=material();missing.intention.editor_snapshot={live:false,buffers:[{id:resourceBufferId(value.buffer),path:'note.py',source_hash:await proposalHash(value.buffer.text),save_guard:value.buffer.saveGuard??null,source_revision:null}]};const readable=await verifyProposal(await envelope(missing),scope);expect(readable.value.intention.explanation).toContain('Propose');await expect(value.ports.prepare(readable,()=>true)).rejects.toThrow('revision is unavailable');});
it('refuses same-text different loaded source guard and generated target ownership',async()=>{for(const change of ['guard','generated']){const value=host();if(change==='guard')value.buffer.saveGuard='different-origin';else value.buffer.file.generated_by='construction.md';await expect(value.ports.prepare(await callerProposal(),()=>true)).rejects.toThrow();expect(value.buffer.text).toBe('value=1\n');value.view.destroy();proposalViews.delete(value.buffer);}});
it('includes every retained authored buffer with stable opaque anonymous IDs and original guards',()=>{const {buffers,buffer}=host(),unnamed=new Buffer({path:'Untitled-notes',name:'notes',dir:false},'ordinary notes','',()=>{}),deleted=new Buffer({path:'old.md',name:'old',dir:false},'<h:file path="other.py">code</h:file>','',()=>{});deleted.resourceDeleted=true;unnamed.isUntitledDraft=true;buffers.values.set(unnamed.file.path,unnamed);buffers.retainedResources.set('deleted',deleted);const first=proposalEditorContext(buffers,buffer),second=proposalEditorContext(buffers,buffer);expect(first.live).toBe(false);expect(first.buffers).toHaveLength(3);expect(first.buffers.find(row=>row.path==='note.py')?.save_guard).toBe('original-guard');expect(first.buffers.filter(row=>row.path===null).map(row=>row.content).sort()).toEqual(['<h:file path="other.py">code</h:file>','ordinary notes']);expect(first.buffers.map(row=>row.id)).toEqual(second.buffers.map(row=>row.id));});
it('does not bless newly added named constructors or plain owners before opening the review',async()=>{for(const text of ['ordinary new source','<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:include path="other.md"/></h:doc>']){const value=host(),late=new Buffer({path:'other.unusual',name:'other',dir:false},text,'base',()=>{},false,scope.namespace);late.saveGuard='newly-loaded-valid-guard';value.buffers.values.set(late.file.path,late);await expect(value.ports.prepare(await callerProposal(),()=>true)).rejects.toThrow('not captured');expect(value.buffer.text).toBe('value=1\n');expect(late.text).toBe(text);value.view.destroy();proposalViews.delete(value.buffer);}});
it('refuses complete review when an unrelated retained anonymous owner was not in the sealed intention',async()=>{const {buffers,ports}=host();buffers.retainedResources.set('late',new Buffer({path:'Untitled-other',name:'other',dir:false},'new retained construction','',()=>{}));await expect(ports.prepare(await callerProposal(),()=>true)).rejects.toThrow('Unaddressed retained editors differ');});
it('permits only an explicitly acquired reviewed target and rejects a new unrelated buffer during acquisition',async()=>{const buffers=new Buffers(()=>{});buffers.workspace=scope.namespace;buffers.workspaceId=scope.workspace;const history=new WorkspaceChangeHistory();let finish!:(buffer:Buffer)=>void;const ports=browserProposalPorts(buffers,history,()=>new Promise(resolve=>{finish=resolve;}),()=>{},()=>true),pending=ports.prepare(await callerProposal(),()=>true);await Promise.resolve();const target=new Buffer({path:'note.py',name:'note.py',dir:false},'value=1\n','loaded',()=>{},false,scope.namespace);target.saveGuard='original-guard';buffers.values.set('note.py',target);buffers.values.set('Untitled-late',new Buffer({path:'Untitled-late',name:'late',dir:false},'extra text','',()=>{}));finish(target);await expect(pending).rejects.toThrow('roster changed');expect(target.text).toBe('value=1\n');expect(history.preview('undo')).toBeNull();});
import {api,request as requestProtocol,parseProtocolJson} from '../api/client';
import {AgentSession} from './Agent';
it('sends one non-live complete agent snapshot with original guards and stable anonymous identity',async()=>{const {buffers,buffer}=host(),notes=new Buffer({path:'Untitled-note',name:'notes',dir:false},'retained note','',()=>{});notes.isUntitledDraft=true;buffers.values.set(notes.file.path,notes);const started=vi.spyOn(api,'agent').mockResolvedValue({session_id:'owned-turn'});const session=new AgentSession();session.captureContext=()=>proposalEditorContext(buffers,buffer);session.prompt='Prepare one coherent proposal';const pending=session.send();buffer.edit('changed after send');await pending;expect(started).toHaveBeenCalledTimes(1);const context=started.mock.calls[0][6]!;expect(context.live).toBe(false);expect(context.buffers.find(row=>row.path==='note.py')?.content).toBe('value=1\n');expect(context.buffers.find(row=>row.path==='note.py')?.save_guard).toBe('original-guard');expect(context.buffers.find(row=>row.path===null)?.content).toBe('retained note');started.mockRestore();});
it('rejects raw proposal protocol numeric aliases and duplicates at the HTTP boundary',async()=>{for(const raw of ['{"version":1,"proposal_protocol":1.0}','{"version":1,"proposal_protocol":1,"proposal_protocol":1}']){expect(()=>parseProtocolJson(raw)).toThrow();const previous=globalThis.fetch;try{globalThis.fetch=vi.fn(async()=>new Response(raw,{status:200,headers:{'content-type':'application/json'}})) as typeof fetch;await expect(requestProtocol('GET','/api/agent-proposals')).rejects.toThrow();}finally{globalThis.fetch=previous;}}});
</ui:copy>
## Review both sides of a recorded turn in the main workspace

Mount the real main conversation, plain-file editor, and shared sealed reader.
The controlled engine reply carries the exact received editor snapshot and
distinct source transitions for the user and assistant fences. The witness
checks the outgoing authority, keeps all three editors mounted, and checks
that closing the review restores focus through the same gutter action without
offering a source write.

<ui:copy id="agent-proposals-main-mounted-review">
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { installMockHandler } from '../api/client';
import type { AgentTurn, FilesResponse } from '../api/types';
import { PlainFilePane } from '../components/PlainFilePane';
import { freeform, open, tab } from '../shell/layout';
import { SessionRegistry } from '../views/documentSession';
import { WorkspaceChat } from '../views/WorkspaceChat';
import type { RecordedPatchSource } from './AgentProposalAdapter';
// @ts-expect-error Node runtime crypto is available to Vitest without browser Node typings.
import { webcrypto } from 'node:crypto';

it('opens a sealed read-only Main Workspace review from either recorded patch fence', async () => {
 const original = 'value=1\n', guard = 'original-guard';
 const originalHash = await proposalHash(original);
 const marker = String.fromCharCode(96).repeat(3);
 const message = (value: number) => ['Recorded edit', '', marker + 'patch', '--- a/note.py', '+++ b/note.py',
  '@@ -1 +1 @@', '-value=1', '+value=' + value, marker].join('\n');
 const turn: AgentTurn = {id:'mounted-review', parent_id:null, prompt:message(41), answer:message(52),
  status:'ok', error:null, provider:'anthropic', model:'fixture', created_at:'', usage:null};
 const file = tab('file', 'note.py');
 const root: FilesResponse = {root:'/workspace', root_path:scope.namespace, workspace_id:scope.workspace,
  tree:[{path:'note.py', name:'note.py', dir:false}]};
 const named = {current:new Map<string, () => RecordedPatchSource | null>()};
 const transitions: string[] = [], proposed = {user:41, assistant:52};
 const writes: string[] = [];
 const heldCrypto = globalThis.crypto;
 Object.defineProperty(globalThis, 'crypto', {configurable:true, value:webcrypto});
 installMockHandler(async (method, path, raw) => {
  if (method !== 'GET') writes.push(method + ' ' + path);
  if (path === '/api/files') return root;
  if (path === '/api/file?path=note.py') return {path:'note.py', content:original, hash:'original-fnv', save_guard:guard, language:'python'};
  if (path.endsWith('/agent/turns')) return {turns:[turn], provider:'anthropic', model:'fixture',
    totals:{usd:0, input:0, output:0, cache_read:0, cache_write:0}};
  if (path.startsWith('/api/workspace/responses')) return {scope:'fixture', state:null};
  if (path.endsWith('/agent/python/context')) return {items:[], updates:[], coverage:'Empty fixture context'};
  if (path === '/api/workspace/entry') return absentEntry;
  if (path.endsWith('/fence-patch')) {
   const {fence, intention} = raw as {fence:RecordedPatchAddress; intention:ProposalMaterial['intention']};
   expect(method).toBe('POST');
   const side = fence.side, source = side === 'user' ? turn.prompt : turn.answer!;
   expect(fence).toEqual({side, source_hash:await proposalHash(source), opening_line:2});
   expect(intention.editor_snapshot).toEqual({live:false, buffers:[{id:file.id, path:'note.py', source_hash:originalHash,
    save_guard:guard, source_revision:String(named.current.get(file.id)?.()?.revision)}]});
   expect(intention.capture.drafts).toMatchObject({'note.py':{text:original, base_sha256:originalHash, save_guard:guard}});
   const value = material(), next = 'value=' + proposed[side] + '\n';
   const context = intention.capture.context;
   value.origin_authority = `recorded ${side} fence, source SHA-256 ${fence.source_hash}; claims remain unverified`;
   value.intention = {...intention, origin:side + ' message in turn ' + turn.id + ', line ' + (fence.opening_line + 1),
    capture:{...intention.capture, edit:{changes:{'hick:///note.py':[{
    range:{start:{line:0,character:0}, end:{line:1,character:0}}, newText:next,
   }]}}}};
   value.inputs.context = context;
   value.plan.context = context;
   value.consequences.context = context;
   const after = value.plan.patches[0].after, projected = value.consequences.saved_sources['note.py'];
   if (after.kind !== 'file' || projected.kind !== 'file') throw Error('Fixture needs a file transition.');
   after.text = next;
   projected.text = next;
   value.consequences.draft.output_paths.push('generated.py');
   transitions.push(side);
   return envelope(value);
  }
  return {backend:'python', ready:true, agents:[], edits:{mode:'review', changes:[]}};
 });
 try {
  const mounted = render(<><PlainFilePane path={file.target} proposalTabId={file.id}
    proposalOwner={(id, read) => {named.current.set(id, read); return () => {named.current.delete(id);};}} />
   <WorkspaceChat layout={open(freeform(), file)} registry={new SessionRegistry()}
    untitledSources={{current:{}}} plainSources={{current:new Map()}} plainPatchOwners={named}
    patchRoot={root} folder="/workspace" focusedEditor={file.id} onOpenSession={() => {}} /></>);
  const buttons = await screen.findAllByRole('button', {name:'Review patch fence on line 3'});
  expect(buttons).toHaveLength(2);
  await waitFor(() => expect(named.current.get(file.id)?.()?.guard).toBe(guard));
  const fileView = EditorView.findFromDOM(mounted.container.querySelector('.plain-file-pane .cm-editor') as HTMLElement)!;
  const transcript = buttons.map(button => EditorView.findFromDOM(button.closest('.conversation-card')!.querySelector('.cm-editor') as HTMLElement)!);
  const composer = EditorView.findFromDOM(await screen.findByRole('textbox', {name:'Agent conversation and response'}))!;
  const initialText = [...transcript, composer, fileView].map(view => view.state.doc.toString());
  act(() => fileView.dispatch({selection:{anchor:2}}));
  const selection = fileView.state.selection;
  for (const [index, side] of ['user','assistant'].entries()) {
   act(() => buttons[index].focus());
   fireEvent.click(buttons[index]);
   await waitFor(() => {
    const card = within(buttons[index].closest('.conversation-card') as HTMLElement);
    expect(card.queryAllByRole('alert').map(alert => alert.textContent)).toEqual([]);
    expect(screen.queryByRole('dialog', {name:'Recorded patch review'})).not.toBeNull();
   });
   const dialog = within(screen.getByRole('dialog', {name:'Recorded patch review'}));
   expect(dialog.getByText(new RegExp('^recorded ' + side + ' fence'))).toBeTruthy();
   expect(dialog.getByText(index === 0 ? 'value=41' : 'value=52', {exact:false})).toBeTruthy();
   fireEvent.change(dialog.getByRole('combobox', {name:'Proposal construction layer'}), {target:{value:'draft'}});
   expect(dialog.getByText(/Generated output: generated.py/)).toBeTruthy();
   expect(dialog.queryByRole('button', {name:/apply|save|stage/i})).toBeNull();
   fireEvent.click(dialog.getByRole('button', {name:'Close'}));
   expect(document.activeElement).toBe(buttons[index]);
  }
  expect(transitions).toEqual(['user','assistant']);
  expect(writes.filter(path => /fence-patch|PUT|agent-proposals\/(validate|report)|resource.*(save|publish)/.test(path))).toEqual([
   'POST /api/docs/workspace/agent/turns/mounted-review/fence-patch',
   'POST /api/docs/workspace/agent/turns/mounted-review/fence-patch',
  ]);
  expect(writes.some(path => path.includes('/api/docs/conversation%3A') && path.endsWith('/project'))).toBe(false);
  const current = [...transcript, composer, fileView];
  expect(current.every(view => view.dom.isConnected && EditorView.findFromDOM(view.dom) === view)).toBe(true);
  expect(current.map(view => view.state.doc.toString())).toEqual(initialText);
  expect(fileView.state.selection.eq(selection)).toBe(true);
 } finally {
  cleanup();
  installMockHandler(null as never);
  Object.defineProperty(globalThis, 'crypto', {configurable:true, value:heldCrypto});
 }
});
</ui:copy>
<ui:file path="apps/web/src/simple/AgentProposals.test.tsx"><ui:paste select="#agent-proposals-browser-tests"/><ui:paste select="#agent-proposals-browser-host-tests"/><ui:paste select="#agent-proposals-main-mounted-review"/></ui:file>

The canonical focused checks passed 20 proposal model/host cases, 16 resource-editor cases and 12 API cases; TypeScript checking passed. The [owned controlled-tool witness](agent-proposal-browser-witness.md) passed the real engine/ACP/browser path: original editor snapshot capture, sealed complete review, one coordinated draft Apply, workspace and editor-local Undo/Redo, retained unrelated unsaved notes and cached editors, exact lost-report acknowledgment replay, stale editor and before-open unknown-owner refusal, then separate v2 resource publication/adoption and guarded inverse restoration. Saved bytes remain unchanged during draft Apply; generated outputs are not materialized. Acknowledgments remain host reports, not independently observed saves or test verdicts. Physical input, screen-reader acceptance and real model-provider semantic judgment remain unverified.
</ui:doc>
