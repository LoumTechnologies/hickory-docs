<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Route resources by source identity, retaining editor handles

For maintainers connecting reviewed resource publication to the existing editors.

A path is an address, not an editor identity. Resolve the complete before/after
namespace before changing any buffer or tab. A swap must route both editors from
the original namespace; a deletion must return its existing handle for retention,
and restoration must explicitly supply that retained handle. Never infer restoration
identity from a reused path. This pure routing boundary does no IO and grants no
write permission. Engine publication receipts and current local editor checks are
still required at adoption. Saved baselines are not inferred from draft text.

The result describes both source sides even when text changes accompany a move.
Hosts must prepare the complete editor transactions before applying any route.
A null editor requests a new editor; an existing handle must keep its renderer,
selection, viewport and undo history. Routing itself never touches those objects.
The three implementations intentionally share this small value model; rendering,
language-service retirement and platform transaction ordering remain host adapters.

## Verification boundary

Five focused browser cases and three focused cases per native implementation pass
(2026-11-15). The browser adoption witness uses live buffers and a CodeMirror view:
move, deletion and explicit restoration retain the same state object, selection,
scroll position and undo depth, while the restored saved baseline differs from its
dirty draft. Refusals cover partial publication and changed local baselines. Native
receipt witnesses check exact scope, complete coverage and separate saved/draft
text; existing routing witnesses retain opaque editor references.

TypeScript checking, the Xcode-backed macOS build and selected XCTest cases,
Android compilation and selected JUnit cases, and the iOS simulator build pass.
The default Command Line Tools Swift build cannot load the SwiftUI macro plugin;
use the installed Xcode developer directory for these Apple checks. The iOS project
was regenerated from `project.yml` so it includes the resource routing source.

The live adoption seams update buffer namespaces and native language lifetimes,
retire obsolete opens and keep deleted dirty buffers in unsaved-change guards.
They are not yet called by resource publication menus. Tab-address adoption,
menu-driven resource Save/restoration, coordinated saved/draft workspace history
and real host/device acceptance remain required. These adapter checks do not
establish completed live apply/undo parity.

## Browser namespace routing

Reject malformed namespaces and missing or stale captures before returning any routes. Retained handles are admitted only for identities absent from the before namespace and only against their exact restoration draft; they cannot replace a live editor.

Browser resource adoption validates exact authored text and prepares raw-source transactions before changing identity routes. A full replacement carries its original inserted line endings and native undo metadata; normalized display equivalence grants no publication authority.

<ui:copy id="resource-editor-routes-browser">
import {authoredSource,authoredChanges,authoredTransactionSource} from './SourceText';
import {Buffer,type Buffers} from './buffers';
import type {FileNode} from '../api/types';
import {editorForBuffer} from './Document';
import {captureRetainedEditorViewport} from './EditorMounts';
import {EditorView} from '@codemirror/view';
import {isolateHistory} from '@codemirror/commands';
import type {Transaction} from '@codemirror/state';
import {isGroup,groups,type Layout} from './layout';
import {confirmEntryOutcome,entryPublicationHadNoEffect,prepareEntryIntent,readEntryTransition,readFrozenEntrySelection,requireEntryRestorationScope,type EntryIntent,type InterruptedEntry} from './WorkspaceEntrySelection';
export interface ResourceSource {identity:string;text:string}
export interface ResourceEditorPatch {path:string;before:ResourceSource|null;after:ResourceSource|null}
export interface ResourceEditorCapture<T> {editor:T;source:string}
export interface ResourceEditorRoute<T> {identity:string;from:string|null;to:string|null;before:string|null;after:string|null;editor:T|null}
export function routeResourceEditors<T>(patches:readonly ResourceEditorPatch[],live:ReadonlyMap<string,ResourceEditorCapture<T>>,retained:ReadonlyMap<string,ResourceEditorCapture<T>>=new Map()):ResourceEditorRoute<T>[] {
 const before=new Map<string,{path:string;source:string}>(),after=new Map<string,{path:string;source:string}>(),paths=new Set<string>();
 if(!patches.length||patches.length>1000)throw new Error('Choose a bounded complete resource namespace.');
 for(const patch of patches){
  if(!patch.path||paths.has(patch.path)||(!patch.before&&!patch.after))throw new Error('Resource paths must be unique and describe a source transition.');paths.add(patch.path);
  for(const [side,target] of [[patch.before,before],[patch.after,after]] as const)if(side){if(!side.identity||target.has(side.identity))throw new Error('A source identity must have exactly one address per namespace.');target.set(side.identity,{path:patch.path,source:side.text});}
 }
 if([...live.keys()].some(id=>!before.has(id))||[...retained.keys()].some(id=>before.has(id)||!after.has(id)))throw new Error('Editor captures must belong to this complete transition.');
 const handles=new Set<T>();
 for(const [id,capture] of [...live,...retained]){
  const expected=(before.get(id)??after.get(id))!;
  if(capture.source!==expected.source||handles.has(capture.editor))throw new Error('Source changed or one editor was assigned to multiple identities.');handles.add(capture.editor);
 }
 for(const id of before.keys())if(!live.has(id))throw new Error('Acquire every existing source editor before namespace adoption.');
 return [...new Set([...before.keys(),...after.keys()])].map(identity=>({identity,from:before.get(identity)?.path??null,to:after.get(identity)?.path??null,before:before.get(identity)?.source??null,after:after.get(identity)?.source??null,editor:(live.get(identity)??retained.get(identity))?.editor??null}));
}

</ui:copy>

Validate the whole publication before acquiring mutation authority. Match saved
baselines independently of draft text; restoration intentionally keeps them different.

<ui:copy id="resource-editor-confirmation-browser">
export type ResourceSaved = {kind:'missing'}|{kind:'file';text:string;sha256:string;readonly:boolean;executable:boolean;mode:number|null};
export interface ResourceReceipt {version:number;adoption_current:boolean;adoption_reason?:string;request_id:string;workspace:string;context:string;outcome:{complete:boolean;recovery:string;applied:string[];source_files_modified?:boolean;source_files_uncertain?:boolean};saved_sources:Record<string,ResourceSaved>|null;source_files:{file:FileNode;saved:ResourceSaved;hash:string;save_guard?:string}[];entry_outcome?:unknown}
export interface ResourceExpectation {request:string;workspace:string;context:string;saved:ReadonlyMap<string,ResourceSaved>}
function sameSaved(a:ResourceSaved|undefined,b:ResourceSaved|undefined):boolean {
 if(!a||!b||a.kind!==b.kind)return false;
 return a.kind==='missing'||b.kind==='file'&&a.text===b.text&&a.sha256===b.sha256&&a.readonly===b.readonly&&a.executable===b.executable&&a.mode===b.mode;
}
export function confirmedResourceFiles(patches:readonly ResourceEditorPatch[],receipt:ResourceReceipt,expected:ResourceExpectation):Map<string,{file:FileNode;saved:Extract<ResourceSaved,{kind:'file'}>;hash:string;save_guard?:string}> {
 const refuse=()=>{throw new Error('Resource publication is not completely confirmed for current editor adoption. '+(receipt.adoption_reason??'Inspect Source recovery before changing editors; historical completion remains separate.'));};
 if(receipt.version!==2||receipt.adoption_current!==true||receipt.request_id!==expected.request||receipt.workspace!==expected.workspace||receipt.context!==expected.context||!receipt.outcome?.complete||!receipt.saved_sources||!receipt.outcome.recovery)refuse();
 const paths=new Set(patches.map(p=>p.path));
 if(paths.size!==patches.length||paths.size!==expected.saved.size||Object.keys(receipt.saved_sources!).length!==paths.size||receipt.outcome.applied.length!==paths.size||new Set(receipt.outcome.applied).size!==paths.size||receipt.outcome.applied.some(path=>!paths.has(path)))refuse();
 const files=new Map<string,{file:FileNode;saved:Extract<ResourceSaved,{kind:'file'}>;hash:string;save_guard?:string}>();
 for(const patch of patches){const saved=receipt.saved_sources![patch.path];if(!sameSaved(saved,expected.saved.get(patch.path))||(saved?.kind==='file')!==(patch.after!==null))refuse();}
 for(const entry of receipt.source_files){const saved=receipt.saved_sources![entry.file.path];if(!paths.has(entry.file.path)||files.has(entry.file.path)||entry.file.dir||entry.file.generated_by||saved?.kind!=='file'||!sameSaved(saved,entry.saved)||typeof entry.hash!=='string'||entry.save_guard!==undefined&&(typeof entry.save_guard!=='string'||!entry.save_guard))refuse();else files.set(entry.file.path,{file:entry.file,saved,hash:entry.hash,save_guard:entry.save_guard});}
 if(files.size!==patches.filter(p=>p.after!==null).length)refuse();return files;
}
/** Adopt only an already confirmed publication; this function never writes files. */
</ui:copy>

The browser recovery adapter acquires every current source before reviewing an inverse. It hashes saved baselines independently from drafts, binds the selected record and compiler consequence fingerprints, and refuses changed buffers or ownership drafts before publication. A failed or uncertain publication is never retried automatically. Confirmed receipts alone authorize editor adoption. Route tabs in one transformation so swaps cannot collide; retain deleted pane placement by explicit buffer handle for a later inverse. Reconfigure the cached editor address without replacing its text history.

Capture current buffer authority before network publication. Adoption preflights
every editor before removing addresses, retires delayed reads and retains deleted
objects outside the writable namespace. Ordinary Save cannot write those objects.

An interrupted Move can confirm a previously removed original address is still
absent. Keep that row in the complete saved receipt. Omit it from the editor
routes only when it is the interrupted original and no live editor owns it.

A confirmed source deletion may leave a live debugger whose code still needs a readable control surface. Retain that exact buffer under a transient, NUL-prefixed tab alias; NUL cannot be a saved path, and the alias never enters engine requests or source ownership. Restoration replaces the alias with the newly confirmed real address. Save As retires only the original object after the copied text still matches; a different buffer reusing its old spelling survives.

Resource acquisition captures both current and retained editor maps, deduplicating
handles while preserving distinct owners at a reused pathname. Unmaterialized
owners travel as typed opaque IDs and text; they never become guessed filesystem
paths. The engine decides whether those drafts have an attributable construction
context. Every await rechecks the complete roster, text, baseline, saved token,
file metadata and workspace, including changes to retained deleted editors. A
caller that explicitly opens an already reviewed target may name that one new
handle when rechecking acquisition; every original owner and every other map
entry must still match. This exception never admits unrelated new drafts.
A successful explicit acquisition can extend the existing lease with exactly that pristine handle. Subsequent source revisions, guards and metadata belong to the same original observations; no second whole-roster capture can bless intervening edits. Provider draft preparation uses this extension, while resource-input captures retain their existing complete ownership payloads.

Opaque buffer IDs are shared with agent snapshots, so an unnamed editor keeps
its identity without acquiring a guessed filesystem address.

The browser resource planner can be used before a host has an editor
adoption path. It checks two observations of saved files, drafts, and
physical guards around acquisition and requests construction consequences
for the complete ordered edit. Main uses this read-only review. Simple
also captures its live buffers for confirmed publication and restoration.

<ui:copy id="resource-editor-adoption-browser">
export function captureResourceEditors(buffers:Buffers,patches:readonly ResourceEditorPatch[],restoring:ReadonlyMap<string,Buffer>=new Map()):()=>boolean {
 const paths=patches.map(patch=>({path:patch.path,buffer:buffers.values.get(patch.path)}));
 const captures=[...new Set([...paths.flatMap(row=>row.buffer?[row.buffer]:[]),...restoring.values()])].map(buffer=>({buffer,text:buffer.text,base:buffer.base,hash:buffer.hash,version:buffer.version,file:buffer.file}));
 return ()=>paths.every(row=>buffers.values.get(row.path)===row.buffer)&&captures.every(row=>row.buffer.text===row.text&&row.buffer.base===row.base&&row.buffer.hash===row.hash&&row.buffer.version===row.version&&row.buffer.file===row.file&&!row.buffer.saving);
}
export function adoptResourceReceipt(buffers:Buffers,patches:readonly ResourceEditorPatch[],receipt:ResourceReceipt,expected:ResourceExpectation,current:()=>boolean,restoring:ReadonlyMap<string,Buffer>=new Map(),absentOriginal?:string):ResourceEditorRoute<Buffer>[] {
 if(!current())throw new Error('Editor drafts or baselines changed after review. Preserve them and inspect Source recovery.');
 const retained=buffers.retainedResources;
 const files=confirmedResourceFiles(patches,receipt,expected),live=new Map<string,ResourceEditorCapture<Buffer>>();
 for(const patch of patches)if(patch.before){const buffer=buffers.values.get(patch.path);if(!buffer||buffer.saving||buffer.readonly||buffer.transient)throw new Error('Source editor is unavailable or saving. Preserve drafts and inspect recovery.');live.set(patch.before.identity,{editor:buffer,source:buffer.text});}
 for(const buffer of restoring.values())if(!buffer.resourceDeleted||![...retained.values()].includes(buffer)||[...buffers.values.values()].includes(buffer))throw new Error('Restoration must select a retained deleted editor. Inspect Source recovery.');
 const held=new Map([...restoring].map(([id,editor])=>[id,{editor,source:editor.text}]));
 const editorPatches=patches.filter(patch=>patch.path!==absentOriginal||patch.before||patch.after||buffers.values.has(patch.path));
 const routes=routeResourceEditors(editorPatches,live,held),prepared:{route:ResourceEditorRoute<Buffer>;buffer:Buffer;view?:EditorView;transaction?:Transaction}[]=[];
 for(const route of routes){
  const entry=route.to===null?undefined:files.get(route.to)!;
  const buffer=route.editor??new Buffer(entry!.file,route.after!,entry!.hash,()=>buffers.changed(),false,buffers.workspace);
  const view=editorForBuffer(buffer)?.view;
  if(view&&(view.composing||authoredSource(view.state)!==buffer.text||view.state.readOnly||!view.state.facet(EditorView.editable)))throw new Error('Finish composition and inspect recovery before adopting source editors.');
  if(route.to!==null&&!patches.some(p=>p.path===route.to&&p.before)&&buffers.values.has(route.to))throw new Error('A destination editor appeared after review. Preserve it and inspect recovery.');
  const transaction=view&&route.after!==null&&buffer.text!==route.after?view.state.update({...authoredChanges(view.state,[{from:0,to:buffer.text.length,insert:route.after}]),annotations:isolateHistory.of('full'),userEvent:'input.workspace',filter:false}):undefined;
  if(transaction&&authoredTransactionSource(transaction)!==route.after)throw new Error('Editor refused the complete resource change. Inspect recovery.');
  prepared.push({route,buffer,view,transaction});
 }
 // Capture attached geometry before the host loses its old styled parent.
 // Publication/source guards above remain the only mutation authority.
 for(const {buffer} of prepared){const editor=editorForBuffer(buffer);if(editor)captureRetainedEditorViewport(editor);}
 // All failures above precede mutation. Stable buffer objects keep cached renderers.
 buffers.retirePendingReads();
 for(const {route} of prepared)if(route.from!==null)buffers.values.delete(route.from);
 for(const {route,buffer,view,transaction} of prepared){
  if(route.to===null){buffer.sourceRevision++;buffer.resourceDeleted=true;retained.set(route.identity,buffer);continue;}
  const entry=files.get(route.to)!;
  if(view&&transaction)view.update([transaction]);
  buffer.sourceRevision++;buffer.file=entry.file;buffer.text=route.after!;buffer.base=entry.saved.text;buffer.hash=entry.hash;buffer.saveGuard=entry.save_guard;buffer.saveGuardReason=entry.save_guard?undefined:"This resource receipt did not include a source identity guard.";buffer.resourceDeleted=false;buffer.error='';
  buffers.values.set(route.to,buffer);for(const [id,held] of retained)if(held===buffer)retained.delete(id);
 }
 for(const {buffer} of prepared)buffer.publish();buffers.changed();return routes;
}

export interface ResourceWireState {kind:'file'|'missing';identity?:string;text?:string}
export interface ResourceWirePlan {version:number;context:string;title:string;patches:{path:string;before:ResourceWireState;after:ResourceWireState}[]}
export interface ResourceConstructionLayer {entries:string[];removed_entries:string[];output_paths:string[]}
export interface ResourceConstructionFacts {version:number;workspace:string;context:string;fingerprint:string;protected_targets:string[];saved:ResourceConstructionLayer;draft:ResourceConstructionLayer}
function currentResourceFacts(facts:ResourceConstructionFacts,workspace:string,context:string){return facts.version===1&&facts.workspace===workspace&&facts.context===context&&/^[0-9a-f]{64}$/.test(facts.fingerprint)&&facts.protected_targets.length===0;}
export interface ResourceRestoreReview {
 operation:'restore'|'save';record:string;fingerprint:string;consequence:string;workspace:string;context:string;
 facts:ResourceConstructionFacts;plan:ResourceWirePlan;patches:ResourceEditorPatch[];saved:Record<string,ResourceSaved>;savedBefore:Record<string,ResourceSaved>;
 proposal:unknown;current:()=>boolean;restoring:Map<string,Buffer>;entryIntent?:EntryIntent;
}
export type ResourceTransport=<T>(route:string,body:unknown)=>Promise<T>;
export function resourceWirePatches(plan:ResourceWirePlan):ResourceEditorPatch[]{
 if(plan.version!==1||!plan.context||!plan.patches.length||plan.patches.length>1000)throw new Error('Resource plan is unavailable or exceeds the review bound.');
 const side=(value:ResourceWireState):ResourceSource|null=>{if(value.kind==='missing')return null;if(value.kind!=='file'||!value.identity||typeof value.text!=='string')throw new Error('Resource source identity is unavailable.');return {identity:value.identity,text:value.text};};
 return plan.patches.map(patch=>({path:patch.path,before:side(patch.before),after:side(patch.after)}));
}
/** Preparation observes current saved sources and drafts; it never publishes. */
export async function prepareResourceRestore(buffers:Buffers,record:{id:string;workspace:string;fingerprint:string;paths:{path:string;saved_after:{kind:string}}[];entry_transition?:unknown},acquire:(path:string)=>Promise<Buffer>,load:ResourceTransport,alive:()=>boolean,hash:(text:string)=>Promise<string>=resourceHash,readEntry?:()=>Promise<unknown>):Promise<ResourceRestoreReview>{
 if(!/^[0-9a-f]{32}$/.test(record.id)||!record.paths.length||record.paths.length>1000)throw new Error('Choose a bounded recovery record.');
 const transition=readEntryTransition(record.entry_transition,record.id);
 const roster=transition?captureResourceRoster(buffers):null;
 const original=()=>alive()&&(!roster||roster.current());
 let entryIntent:EntryIntent|undefined;
 if(transition){
  if(!readEntry||!buffers.workspace||!buffers.workspaceId)throw new Error('Connect to the original workspace before reviewing its main construction restoration.');
  const selection=await readFrozenEntrySelection(readEntry,{workspace:buffers.workspaceId,namespace:buffers.workspace,current:original});
  requireEntryRestorationScope(selection,transition.after);
  entryIntent=await prepareEntryIntent(selection,{kind:'restoration',recovery_id:record.id,recovery_fingerprint:record.fingerprint},
   body=>load('/api/workspace/entry/intent',body),transition.before.path,original);
 }
 for(const item of record.paths)if(item.saved_after.kind==='file'){
  const buffer=await acquire(item.path);
  if(roster)roster.retain(item.path,buffer);
 }
 if(!original())throw new Error('Workspace or recovery selection changed. Prepare restoration again.');
 const paths=new Set(record.paths.map(item=>item.path));if(paths.size!==record.paths.length)throw new Error('Recovery paths are not unique.');
 const context=entryIntent?.context??crypto.randomUUID().replaceAll('-','');
 const edit=entryIntent?.edit??{changes:Object.fromEntries(record.paths.map(item=>['hick:///'+item.path.split('/').map(encodeURIComponent).join('/'),[]]))};
 const {inputs,capture,current}=await captureResourceInputs(buffers,[...paths],context,edit,load,original,hash,entryIntent);
 if(inputs.workspace!==record.workspace)throw new Error('Current source acquisition refused or changed. Prepare restoration again.');
 const proposal={recovery_id:record.id,inputs,capture};
 const prepared=await load<{version:number;restoration:{recovery_fingerprint:string;workspace:string;context:string;plan:ResourceWirePlan;saved_before:Record<string,ResourceSaved>;saved_after:Record<string,ResourceSaved>};consequences:ResourceConstructionFacts}>('/api/changes/prepare-restoration',proposal);
 if(!current()||prepared.version!==1||prepared.restoration.workspace!==record.workspace||prepared.restoration.context!==context||prepared.restoration.recovery_fingerprint!==record.fingerprint||!currentResourceFacts(prepared.consequences,record.workspace,context))throw new Error('Recovery evidence or source ownership changed. Read and prepare again.');
 const patches=resourceWirePatches(prepared.restoration.plan),restoring=new Map<string,Buffer>();
 for(const patch of patches)if(patch.after&&!patches.some(item=>item.before?.identity===patch.after!.identity)){const held=buffers.retainedResources.get(patch.after.identity);if(held)restoring.set(patch.after.identity,held);}
 const live=new Map<string,ResourceEditorCapture<Buffer>>();for(const patch of patches)if(patch.before){const buffer=buffers.values.get(patch.path);if(!buffer||buffer.text!==patch.before.text)throw new Error("Current resource source does not match the reviewed restoration. Preserve drafts and prepare again.");live.set(patch.before.identity,{editor:buffer,source:buffer.text});}
 routeResourceEditors(patches,live,new Map([...restoring].map(([identity,editor])=>[identity,{editor,source:editor.text}])));
 const editorCurrent=captureResourceEditors(buffers,patches,restoring);
 return {operation:'restore',facts:prepared.consequences,record:record.id,fingerprint:record.fingerprint,consequence:prepared.consequences.fingerprint,workspace:record.workspace,context,plan:prepared.restoration.plan,patches,saved:prepared.restoration.saved_after,savedBefore:prepared.restoration.saved_before,proposal,current:()=>current()&&editorCurrent(),restoring,entryIntent};
}
const resourceAttempts=new WeakMap<ResourceRestoreReview,string>();
export class EntryPublicationNoEffect extends Error {
 constructor(){super('Publication ended before any source write. Your draft and history are retained. Prepare a fresh review.');}
}
/** Never retry a failed/uncertain write here. The caller requires a new review. */
export async function publishResourceRestore(buffers:Buffers,review:ResourceRestoreReview,load:ResourceTransport):Promise<{routes:ResourceEditorRoute<Buffer>[];recovery:string}>{
 const earlier=resourceAttempts.get(review);if(earlier)throw new Error('This review already has publication request '+earlier+'. Inspect its retained recovery; it will not be retried.');
 if(!review.current())throw new Error('Source changed after review. Prepare restoration again.');
 for(const patch of review.patches)if(patch.before){const buffer=buffers.values.get(patch.path),view=buffer&&editorForBuffer(buffer)?.view;if(!buffer||view&&(view.composing||authoredSource(view.state)!==buffer.text||view.state.readOnly||!view.state.facet(EditorView.editable)))throw new Error('Finish composition and restore current source before publication.');}
 const request=crypto.randomUUID().replaceAll('-','');resourceAttempts.set(review,request);
 let receipt:ResourceReceipt;try{receipt=await load<ResourceReceipt>(review.operation==='save'?'/api/changes/resource-save':'/api/changes/restore-resources',{request_id:request,reviewed_fingerprint:review.consequence,...(review.operation==='restore'?{reviewed_recovery_fingerprint:review.fingerprint}:{}),proposal:review.proposal});}catch(error){throw new Error(String(error)+' Publication request '+request+' is not confirmed. Inspect Source recovery using this identity before another review.');}
 let routes:ResourceEditorRoute<Buffer>[];
 try {
  if (entryPublicationHadNoEffect(receipt.entry_outcome,request,review.entryIntent)) {
   if (receipt.version!==2 || receipt.request_id!==request || receipt.workspace!==review.workspace || receipt.context!==review.context ||
       receipt.adoption_current!==false || receipt.saved_sources!==null || receipt.outcome?.complete!==false ||
       receipt.outcome.source_files_modified!==false || receipt.outcome.source_files_uncertain!==false ||
       !Array.isArray(receipt.outcome.applied) || receipt.outcome.applied.length || !Array.isArray(receipt.source_files) || receipt.source_files.length) {
    throw new Error('The no-effect receipt does not confirm the original source state.');
   }
   throw new EntryPublicationNoEffect();
  }
  confirmEntryOutcome(receipt.entry_outcome,request,review.entryIntent);
  const interrupted=review.entryIntent?.interrupted;
  routes=adoptResourceReceipt(buffers,review.patches,receipt,{request,workspace:review.workspace,context:review.context,saved:new Map(Object.entries(review.saved))},review.current,review.restoring,interrupted?.missingOriginal?interrupted.source:undefined);
 }catch(error){if(error instanceof EntryPublicationNoEffect)throw error;throw new Error(String(error)+" Publication request "+request+"; retained recovery "+(receipt.outcome?.recovery??"unknown")+". Preserve local drafts while inspecting the result.");}
 const match=receipt.outcome.recovery.match(/(?:^|\/)resource-changes\/([0-9a-f]{32})\/[^/]+$/);if(!match)throw new Error('Source restored but inverse recovery identity is unavailable. Inspect Source recovery.');
 return {routes,recovery:match[1]};
}


/** Route tabs by reviewed source identity, retaining deleted placement for inverse restoration. */
/** UI-only aliases never enter a resource request or the saved namespace. */
export class RetainedResourceTabs {
 readonly buffers=new Map<string,Buffer>();private aliases=new WeakMap<Buffer,string>();private copies=new WeakMap<Buffer,string>();
 routes(routes:readonly ResourceEditorRoute<Buffer>[],keep:(buffer:Buffer)=>boolean):ResourceEditorRoute<Buffer>[] {
  return routes.map(route=>{const buffer=route.editor;if(!buffer)return route;const held=this.aliases.get(buffer);
   if(route.to===null&&keep(buffer)){const alias=held??`\0retained-resource:${crypto.randomUUID()}`;this.aliases.set(buffer,alias);this.buffers.set(alias,buffer);return {...route,to:alias};}
   if(route.to!==null&&held&&this.buffers.get(held)===buffer){this.buffers.delete(held);this.aliases.delete(buffer);this.copies.delete(buffer);return {...route,from:held};}
   return route;
  });
 }
 copied(buffer:Buffer){return this.copies.get(buffer)===buffer.text;}
 noteCopy(buffer:Buffer,text:string){if(buffer.text===text)this.copies.set(buffer,text);}
 retire(buffer:Buffer){const alias=this.aliases.get(buffer);if(alias&&this.buffers.get(alias)===buffer)this.buffers.delete(alias);this.aliases.delete(buffer);this.copies.delete(buffer);}
}
/** Copy publication does not authorize deleting a different editor now at the old spelling. */
export function retireCopiedBuffer(buffers:Buffers,source:Buffer,text:string):boolean {
 if(source.text!==text)return false;
 for(const [path,buffer] of buffers.values)if(buffer===source)buffers.values.delete(path);
 for(const [id,buffer] of buffers.retainedResources)if(buffer===source)buffers.retainedResources.delete(id);
 return true;
}
export function resourceLayout(layout:Layout,routes:readonly ResourceEditorRoute<Buffer>[],placements:Map<Buffer,{group:string;index:number;active:boolean}>):Layout {
 const byPath=new Map(routes.filter(route=>route.from!==null).map(route=>[route.from!,route]));
 for(const group of groups(layout))group.tabs.forEach((path,index)=>{const route=byPath.get(path);if(route?.to===null&&route.editor)placements.set(route.editor,{group:group.id,index,active:group.active===path});});
 const routeTree=(node:Layout):Layout=>{
  if(!isGroup(node))return {...node,first:routeTree(node.first),second:routeTree(node.second)};
  const tabs=node.tabs.flatMap(path=>{const route=byPath.get(path);return route?(route.to===null?[]:[route.to]):[path];});
  const moved=node.active?byPath.get(node.active):undefined;let active=moved?moved.to:node.active;
  for(const route of [...routes].sort((a,b)=>(a.editor?placements.get(a.editor)?.index??0:0)-(b.editor?placements.get(b.editor)?.index??0:0)))if(route.from===null&&route.to!==null&&route.editor){const held=placements.get(route.editor);if(held?.group===node.id){tabs.splice(Math.min(held.index,tabs.length),0,route.to);if(held.active)active=route.to;placements.delete(route.editor);}}
  return {...node,tabs,active:active&&tabs.includes(active)?active:tabs.at(-1)??null};
 };return routeTree(layout);
}
export function isResourceEdit(edit:unknown):boolean{return !!edit&&typeof edit==='object'&&Array.isArray((edit as {documentChanges?:unknown}).documentChanges)&&((edit as {documentChanges:unknown[]}).documentChanges).some(item=>!!item&&typeof item==='object'&&'kind' in item);}
export type ResourceSaveReview = Pick<ResourceRestoreReview, "facts" | "consequence" | "workspace" | "context" | "plan" | "patches" | "saved" | "savedBefore" | "proposal" | "current">;

type ObserveResourceInputs = (paths: string[], context: string) => Promise<{
  inputs: ResourceInputs;
  capture: ResourceInputCapture;
  current: () => boolean;
}>;

/** Acquire and review a whole resource proposal before choosing a host's adoption policy. */
export async function prepareResourceReview(title: string, edit: unknown, context: string,
  acquire: (path: string) => Promise<string>, observe: ObserveResourceInputs, load: ResourceTransport,
  alive: () => boolean): Promise<ResourceSaveReview> {
  const targets = await load<{paths: string[]}>("/api/changes/resource-targets", {edit});
  if (!alive() || !targets.paths.length || targets.paths.length > 1000 || new Set(targets.paths).size !== targets.paths.length) {
    throw Error("Resource targets changed or exceed the review bound.");
  }
  const initial = await observe(targets.paths, context);
  for (const path of targets.paths) {
    const source = initial.inputs.sources[path];
    if (source?.kind === "file" && await acquire(path) !== source.text) {
      throw Error("Referenced source changed during acquisition. Prepare the provider action again.");
    }
  }
  const captured = await observe(targets.paths, context);
  if (initial.inputs.workspace !== captured.inputs.workspace) throw Error("Resource workspace changed. Prepare again.");
  for (const path of targets.paths) {
    const before = initial.inputs.sources[path], after = captured.inputs.sources[path];
    if (before?.kind !== after?.kind || before?.text !== after?.text ||
        JSON.stringify(initial.inputs.saved_sources[path]) !== JSON.stringify(captured.inputs.saved_sources[path]) ||
        initial.inputs.saved_guards[path] !== captured.inputs.saved_guards[path]) {
      throw Error("Saved or draft resource input changed during acquisition. Prepare the whole action again.");
    }
  }
  if (JSON.stringify(initial.inputs.saved_guards) !== JSON.stringify(captured.inputs.saved_guards)) {
    throw Error("Physical construction dependencies changed during acquisition. Prepare the whole action again.");
  }
  const {plan} = await load<{plan: ResourceWirePlan}>("/api/changes/plan-resources", {context, title, edit, sources: captured.inputs.sources});
  if (!captured.current()) throw Error("Source changed while planning resource operations.");
  const proposal = {inputs: captured.inputs, capture: captured.capture, plan};
  const facts = await load<ResourceConstructionFacts & {saved_sources: Record<string, ResourceSaved>}>("/api/changes/resource-consequences", proposal);
  if (!captured.current() || !currentResourceFacts(facts, captured.inputs.workspace, context) || !facts.saved_sources) {
    throw Error("Resource ownership changed or saved consequences are unavailable. Prepare again.");
  }
  return {
    facts, consequence: facts.fingerprint, workspace: captured.inputs.workspace, context, plan,
    patches: resourceWirePatches(plan), saved: facts.saved_sources, savedBefore: captured.inputs.saved_sources,
    proposal, current: captured.current,
  };
}

export async function prepareResourceSave(buffers:Buffers,title:string,edit:unknown,acquire:(path:string)=>Promise<Buffer>,load:ResourceTransport,alive:()=>boolean,hash:(text:string)=>Promise<string>=resourceHash,entryIntent?:EntryIntent):Promise<ResourceRestoreReview>{
 const context=entryIntent?.context??crypto.randomUUID().replaceAll('-','');
 const review=await prepareResourceReview(title,edit,context,async path=>(await acquire(path)).text,
  (paths,id)=>captureResourceInputs(buffers,paths,id,edit,load,alive,hash,entryIntent),load,alive);
 const editorCurrent=captureResourceEditors(buffers,review.patches);
 if(review.patches.some(patch=>patch.before&&buffers.values.get(patch.path)?.text!==patch.before.text))throw new Error('Acquire every reviewed resource source before publication.');
 return {...review,operation:'save',record:'',fingerprint:'',current:()=>review.current()&&editorCurrent(),restoring:new Map(),entryIntent};
}
const resourceHash=async(text:string)=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),byte=>byte.toString(16).padStart(2,'0')).join('');
export interface ResourceInputs {source_files_modified:false;workspace:string;context:string;refusals:Record<string,string>;sources:Record<string,ResourceWireState>;saved_sources:Record<string,ResourceSaved>;saved_guards:Record<string,string>}
export function resourceBufferId(buffer:Buffer){return buffer.sourceId;}
/** Explicit buffer kind, never a filename prefix, determines anonymous ownership.
 * Retained deleted editors and reused spellings remain distinct authored owners. */
export function captureResourceRoster(buffers:Buffers){
 const capture=(buffer:Buffer)=>({buffer,path:buffer.file.path,text:buffer.text,base:buffer.base,hash:buffer.hash,guard:buffer.saveGuard,version:buffer.version,revision:buffer.sourceRevision,file:buffer.file,metadata:JSON.stringify(buffer.file),readonly:buffer.readonly,transient:buffer.transient,deleted:buffer.resourceDeleted,untitled:buffer.isUntitledDraft,namespace:buffer.namespace});
 const namespace=buffers.workspace,workspace=buffers.workspaceId,values=[...buffers.values],retained=[...buffers.retainedResources];
 const captures=[...new Set([...values,...retained].map(([,buffer])=>buffer))].map(capture);
 const same=(map:Map<string,Buffer>,rows:[string,Buffer][],additions:ReadonlyMap<string,Buffer>=new Map())=>map.size===rows.length+additions.size&&rows.every(([key,buffer])=>map.get(key)===buffer)&&[...additions].every(([key,buffer])=>!rows.some(([original])=>original===key)&&map.get(key)===buffer);
 const current=(acquired:ReadonlyMap<string,Buffer>=new Map())=>buffers.workspace===namespace&&buffers.workspaceId===workspace&&same(buffers.values,values,acquired)&&same(buffers.retainedResources,retained)&&captures.every(row=>row.buffer.file===row.file&&row.buffer.file.path===row.path&&row.buffer.text===row.text&&row.buffer.base===row.base&&row.buffer.hash===row.hash&&row.buffer.saveGuard===row.guard&&row.buffer.sourceRevision===row.revision&&JSON.stringify(row.buffer.file)===row.metadata&&row.buffer.readonly===row.readonly&&row.buffer.transient===row.transient&&row.buffer.resourceDeleted===row.deleted&&row.buffer.isUntitledDraft===row.untitled&&row.buffer.namespace===row.namespace&&!row.buffer.saving);
 const ownership_sources:Record<string,string>=Object.create(null),unaddressed_sources:Record<string,string>=Object.create(null);
 for(const row of captures){if(row.readonly||row.transient)continue;if(row.namespace&&row.namespace!==namespace)throw Error('A retained authored source belongs to another workspace. Preserve it before preparing resource changes.');const relative=!!row.path&&!row.path.startsWith('/')&&!/^[A-Za-z]:[\\/]/.test(row.path)&&!row.path.split('/').some(part=>part==='..'||part==='.'||part==='');const named=!row.deleted&&relative&&!row.untitled&&buffers.values.get(row.path)===row.buffer;if(named){if(Object.hasOwn(ownership_sources,row.path))throw Error('Authored resource identities are ambiguous.');ownership_sources[row.path]=row.text;}else unaddressed_sources[resourceBufferId(row.buffer)]=row.text;}
 const retain=(path:string,buffer:Buffer)=>{const existing=values.find(([key])=>key===path);if(existing){if(existing[1]!==buffer||!current())throw Error("Original source owners changed during acquisition.");return;}if(!current(new Map([[path,buffer]])))throw Error("Original source owners changed during acquisition.");values.push([path,buffer]);captures.push(capture(buffer));};
 return {captures,ownership_sources,unaddressed_sources,current,retain};
}
export async function retainedResourceInputs(buffers:Buffers,roster:ReturnType<typeof captureResourceRoster>,targetPaths:readonly string[],
 current:()=>boolean,hash:(text:string)=>Promise<string>=resourceHash,interrupted?:InterruptedEntry){
 const paths=new Set(targetPaths);
 const drafts:Record<string,unknown>=Object.create(null);for(const row of roster.captures){if(!paths.has(row.path)||buffers.values.get(row.path)!==row.buffer||row.deleted)continue;if(row.readonly||row.transient||row.buffer.saving)throw new Error('Choose writable current resource source.');if(!row.guard)throw Error('The loaded source has no physical identity guard. Preserve its draft and reopen or compare the exact saved source before preparing resource changes.');drafts[row.path]={save_guard:row.guard,text:row.text,base_sha256:await hash(row.base),version:row.version,writable:true};}
 if(!current())throw new Error('Source changed while capturing resource inputs.');
 const saved_guards:Record<string,string>=Object.create(null);
 for (const row of roster.captures) {
  if (Object.hasOwn(roster.ownership_sources,row.path) && row.guard &&
      !(interrupted?.missingOriginal && row.path === interrupted.source)) saved_guards[row.path]=row.guard;
 }
 return {drafts,ownership_sources:roster.ownership_sources,unaddressed_sources:roster.unaddressed_sources,saved_guards};
}
export async function captureResourceInputs(buffers:Buffers,targetPaths:string[],context:string,edit:unknown,load:ResourceTransport,alive:()=>boolean,hash:(text:string)=>Promise<string>,entryIntent?:EntryIntent){
 const roster=captureResourceRoster(buffers),current=()=>alive()&&roster.current();
 const retained=await retainedResourceInputs(buffers,roster,targetPaths,current,hash,entryIntent?.interrupted);
 return observeResourceInputs({context,edit,...retained},load,current,entryIntent);
}
export interface ResourceInputCapture {
 context: string;
 edit: unknown;
 drafts: Record<string, unknown>;
 ownership_sources: Record<string, string>;
 unaddressed_sources: Record<string, string>;
 saved_guards: Record<string, string>;
}
export async function observeResourceInputs(capture:ResourceInputCapture,load:ResourceTransport,current:()=>boolean,entryIntent?:EntryIntent) {
 const {context}=capture;
 const inputs=entryIntent
    ? await load<ResourceInputs>('/api/workspace/entry/inputs',{version:1,token:entryIntent.token,capture})
    : await load<ResourceInputs>('/api/changes/resource-inputs',capture);
 if(!current()||!inputs||inputs.context!==context||typeof inputs.workspace!=='string'||!inputs.workspace||inputs.source_files_modified!==false||!inputs.sources||typeof inputs.sources!=='object'||Array.isArray(inputs.sources)||!inputs.saved_sources||typeof inputs.saved_sources!=='object'||Array.isArray(inputs.saved_sources)||!inputs.refusals||typeof inputs.refusals!=='object'||Array.isArray(inputs.refusals)||!inputs.saved_guards||typeof inputs.saved_guards!=='object'||Array.isArray(inputs.saved_guards)||Object.entries(inputs.saved_guards).some(([path,guard])=>!path||typeof guard!=='string'||!guard||guard.length>16384)||Object.keys(inputs.refusals).length)throw new Error('Current source acquisition refused or changed. Prepare again.');
 const interrupted=entryIntent?.interrupted;
 for (const [path, source] of Object.entries(inputs.sources)) {
  if (source.kind !== 'file' || Object.hasOwn(inputs.saved_guards,path)) continue;
  if (interrupted?.missingOriginal && path === interrupted.source && inputs.saved_sources[path]?.kind === 'missing' &&
     Object.hasOwn(capture.drafts,path)) continue;
  throw Error('Current source acquisition lacks its physical observation. Prepare again.');
 }
 return {inputs,capture,current};
}
</ui:copy>

<ui:file path="apps/web/src/simple/ResourceEditors.ts"><ui:paste select="#resource-editor-routes-browser"/><ui:paste select="#resource-editor-confirmation-browser"/><ui:paste select="#resource-editor-adoption-browser"/></ui:file>

## Apple namespace routing

Use UTF-8 keys for both paths and identities: Swift canonical string equality must not merge byte-distinct source addresses. Keep the generic editor handle opaque so the same model serves AppKit and UIKit.

<ui:copy id="resource-editor-routes-apple">
import Foundation
struct ResourceSource {var identity:String;var text:String}
struct ResourceEditorPatch {var path:String;var before:ResourceSource?;var after:ResourceSource?}
struct ResourceEditorCapture<Editor:AnyObject> {var editor:Editor;var source:String}
struct ResourceEditorRoute<Editor:AnyObject> {var identity:String;var from:String?;var to:String?;var before:String?;var after:String?;var editor:Editor?}
private struct ResourceEditorAddress {var identity:String;var path:String;var source:String}
func routeResourceEditors<Editor:AnyObject>(_ patches:[ResourceEditorPatch],live:[String:ResourceEditorCapture<Editor>],retained:[String:ResourceEditorCapture<Editor>]=[:]) throws->[ResourceEditorRoute<Editor>] {
 func key(_ text:String)->Data {Data(text.utf8)}
 func refuse(_ message:String)->NSError {NSError(domain:"ResourceEditors",code:1,userInfo:[NSLocalizedDescriptionKey:message])}
 guard !patches.isEmpty,patches.count<=1000 else{throw refuse("Choose a bounded complete resource namespace.")}
 var before:[Data:ResourceEditorAddress]=[:],after:[Data:ResourceEditorAddress]=[:],paths=Set<Data>(),order:[Data]=[]
 for patch in patches {
  guard !patch.path.isEmpty,paths.insert(key(patch.path)).inserted,patch.before != nil || patch.after != nil else{throw refuse("Resource paths must be unique and describe a source transition.")}
  if let side=patch.before {let id=key(side.identity);guard !id.isEmpty,before[id]==nil else{throw refuse("A source identity must have exactly one address per namespace.")};before[id]=ResourceEditorAddress(identity:side.identity,path:patch.path,source:side.text);order.append(id)}
  if let side=patch.after {let id=key(side.identity);guard !id.isEmpty,after[id]==nil else{throw refuse("A source identity must have exactly one address per namespace.")};after[id]=ResourceEditorAddress(identity:side.identity,path:patch.path,source:side.text)}
 }
 let captured=Dictionary(uniqueKeysWithValues:live.map{(key($0.key),$0.value)}),held=Dictionary(uniqueKeysWithValues:retained.map{(key($0.key),$0.value)})
 guard captured.keys.allSatisfy({before[$0] != nil}),held.keys.allSatisfy({before[$0]==nil && after[$0] != nil}) else{throw refuse("Editor captures must belong to this complete transition.")}
 var handles=Set<ObjectIdentifier>()
 for (id,capture) in Array(captured)+Array(held) {guard let expected=before[id] ?? after[id],key(capture.source)==key(expected.source),handles.insert(ObjectIdentifier(capture.editor)).inserted else{throw refuse("Source changed or one editor was assigned to multiple identities.")}}
 guard before.keys.allSatisfy({captured[$0] != nil}) else{throw refuse("Acquire every existing source editor before namespace adoption.")}
 for patch in patches {if let side=patch.after {let id=key(side.identity);if before[id]==nil{order.append(id)}}}
 return order.map{id in let a=before[id],b=after[id];return ResourceEditorRoute(identity:(a ?? b)!.identity,from:a?.path,to:b?.path,before:a?.source,after:b?.source,editor:(captured[id] ?? held[id])?.editor)}
}

</ui:copy>

Decode complete receipts separately from renderer state. Compare source text,
addresses and scope as UTF-8 bytes; saved restoration text need not equal its draft.

<ui:copy id="resource-editor-confirmation-apple">
struct ResourceSaved:Codable {
 var kind:String;var text:String?;var sha256:String?;var readonly:Bool?;var executable:Bool?;var mode:Int?
 func matches(_ other:ResourceSaved)->Bool {
  func exact(_ a:String?,_ b:String?)->Bool {a.map{Data($0.utf8)}==b.map{Data($0.utf8)}}
  return exact(kind,other.kind) && (kind=="missing" || kind=="file" && exact(text,other.text) && exact(sha256,other.sha256) && readonly==other.readonly && executable==other.executable && mode==other.mode)
 }
}
struct ResourcePublication<File:Decodable>:Decodable {
 struct Outcome:Decodable {var complete:Bool;var recovery:String;var applied:[String];var source_files_modified:Bool?=nil;var source_files_uncertain:Bool?=nil}
 struct Entry:Decodable {var file:File;var saved:ResourceSaved;var hash:String;var save_guard:String? = nil}
 var version:Int;var request_id:String;var workspace:String;var context:String;var outcome:Outcome;var saved_sources:[String:ResourceSaved]?;var source_files:[Entry];var adoption_current:Bool=false;var adoption_reason:String?=nil;var entry_outcome:WorkspaceEntryPublication?=nil
}
func confirmedResourceFiles<File>(_ patches:[ResourceEditorPatch],receipt:ResourcePublication<File>,request:String,workspace:String,context:String,expected:[String:ResourceSaved],address:(File)->(path:String,authored:Bool)) throws->[Data:ResourcePublication<File>.Entry] {
 func key(_ value:String)->Data {Data(value.utf8)}
 func refuse()->NSError {NSError(domain:"ResourceEditors",code:2,userInfo:[NSLocalizedDescriptionKey:receipt.adoption_reason ?? "Resource publication is not completely confirmed for this review. Inspect Source recovery before changing editors."])}
 guard receipt.version==2,receipt.adoption_current,key(receipt.request_id)==key(request),key(receipt.workspace)==key(workspace),key(receipt.context)==key(context),receipt.outcome.complete,!receipt.outcome.recovery.isEmpty,let saved=receipt.saved_sources else{throw refuse()}
 let paths=Set(patches.map{key($0.path)}),savedByKey=Dictionary(uniqueKeysWithValues:saved.map{(key($0.key),$0.value)}),expectedByKey=Dictionary(uniqueKeysWithValues:expected.map{(key($0.key),$0.value)})
 guard paths.count==patches.count,Set(savedByKey.keys)==paths,Set(expectedByKey.keys)==paths,Set(receipt.outcome.applied.map{key($0)})==paths,receipt.outcome.applied.count==paths.count else{throw refuse()}
 for patch in patches {guard let state=savedByKey[key(patch.path)],let reviewed=expectedByKey[key(patch.path)],state.matches(reviewed),(state.kind=="file")==((patch.after) != nil) else{throw refuse()}}
 var files:[Data:ResourcePublication<File>.Entry]=[:]
 for entry in receipt.source_files {let target=address(entry.file),path=key(target.path);guard paths.contains(path),files[path]==nil,target.authored,let state=savedByKey[path],state.kind=="file",state.matches(entry.saved),state.text != nil else{throw refuse()};files[path]=entry}
 guard files.count==patches.filter({$0.after != nil}).count else{throw refuse()};return files
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/ResourceEditors.swift"><ui:paste select="#resource-editor-routes-apple"/><ui:paste select="#resource-editor-confirmation-apple"/></ui:file>

## Android namespace routing

Compute the whole transition without mutating Compose state or TextFieldState. Returning the original editor references lets an adoption adapter preserve the input connection, selection and undo manager instead of replacing the text-field state.

<ui:copy id="resource-editor-routes-android">
package com.hickorydocs.simple
import java.util.Collections
import java.util.IdentityHashMap
class SourceAcquisitionLease(private val probe:(String?)->Boolean,private val adopt:(String,DocumentBuffer)->Unit) {
 fun current(allowing:String?=null)=probe(allowing)
 fun retain(path:String,buffer:DocumentBuffer)=adopt(path,buffer)
}
data class ResourceSource(val identity:String,val text:String)
data class ResourceEditorPatch(val path:String,val before:ResourceSource?,val after:ResourceSource?)
data class ResourceEditorCapture<T:Any>(val editor:T,val source:String)
data class ResourceEditorRoute<T:Any>(val identity:String,val from:String?,val to:String?,val before:String?,val after:String?,val editor:T?)
fun <T:Any> routeResourceEditors(patches:List<ResourceEditorPatch>,live:Map<String,ResourceEditorCapture<T>>,retained:Map<String,ResourceEditorCapture<T>> = emptyMap()):List<ResourceEditorRoute<T>> {
 data class Address(val path:String,val source:String)
 val before=linkedMapOf<String,Address>();val after=linkedMapOf<String,Address>();val paths=mutableSetOf<String>()
 require(patches.isNotEmpty()&&patches.size<=1000){"Choose a bounded complete resource namespace."}
 for(patch in patches){
  require(patch.path.isNotEmpty()&&paths.add(patch.path)&&(patch.before!=null||patch.after!=null)){"Resource paths must be unique and describe a source transition."}
  for((side,target) in listOf(patch.before to before,patch.after to after))if(side!=null){require(side.identity.isNotEmpty()&&!target.containsKey(side.identity)){"A source identity must have exactly one address per namespace."};target[side.identity]=Address(patch.path,side.text)}
 }
 require(live.keys.all{it in before}&&retained.keys.all{it !in before&&it in after}){"Editor captures must belong to this complete transition."}
 val handles=Collections.newSetFromMap(IdentityHashMap<T,Boolean>())
 for((id,capture) in live.toList()+retained.toList()){
  val expected=before[id] ?: after.getValue(id)
  require(capture.source==expected.source&&handles.add(capture.editor)){"Source changed or one editor was assigned to multiple identities."}
 }
 require(before.keys.all{it in live}){"Acquire every existing source editor before namespace adoption."}
 return (before.keys+after.keys).map{id->ResourceEditorRoute(id,before[id]?.path,after[id]?.path,before[id]?.source,after[id]?.source,(live[id] ?: retained[id])?.editor)}
}

</ui:copy>

Require exact scope, path coverage and reviewed saved states before returning
file metadata. The caller receives every surviving file together or no authority.

<ui:copy id="resource-editor-confirmation-android">
data class ResourceSaved(val kind:String,val text:String?=null,val sha256:String?=null,val readonly:Boolean?=null,val executable:Boolean?=null,val mode:Int?=null)
data class ResourcePublishedFile<File>(val file:File,val saved:ResourceSaved,val hash:String,val saveGuard:String?=null)
data class ResourcePublication<File>(val version:Int,val request:String,val workspace:String,val context:String,val complete:Boolean,val recovery:String,val applied:List<String>,val saved:Map<String,ResourceSaved>?,val files:List<ResourcePublishedFile<File>>,val adoptionCurrent:Boolean=false,val adoptionReason:String?=null)
fun <File> confirmedResourceFiles(patches:List<ResourceEditorPatch>,receipt:ResourcePublication<File>,request:String,workspace:String,context:String,expected:Map<String,ResourceSaved>,address:(File)->Pair<String,Boolean>):Map<String,ResourcePublishedFile<File>> {
 fun admit(ok:Boolean){require(ok){"Resource publication is not completely confirmed for this review. Inspect Source recovery before changing editors."}}
 admit(receipt.version==2&&receipt.request==request&&receipt.workspace==workspace&&receipt.context==context&&receipt.complete&&receipt.adoptionCurrent&&receipt.recovery.isNotEmpty())
 val paths=patches.map{it.path}.toSet();val saved=receipt.saved
 admit(paths.size==patches.size&&saved!=null&&saved.keys==paths&&expected.keys==paths&&receipt.applied.size==paths.size&&receipt.applied.toSet()==paths)
 for(patch in patches){val state=saved!!.getValue(patch.path);admit(state==expected[patch.path]&&(state.kind=="file")== (patch.after!=null)&&state.kind in listOf("file","missing"))}
 val files=linkedMapOf<String,ResourcePublishedFile<File>>()
 for(entry in receipt.files){val (path,authored)=address(entry.file);val state=saved!![path];admit(path in paths&&path !in files&&authored&&state?.kind=="file"&&state==entry.saved&&state.text!=null&&(entry.saveGuard==null||entry.saveGuard.isNotEmpty()));files[path]=entry}
 admit(files.size==patches.count{it.after!=null});return files
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/ResourceEditors.kt"><ui:paste select="#resource-editor-routes-android"/><ui:paste select="#resource-editor-confirmation-android"/></ui:file>

## Adopt a confirmed browser publication into live buffers

The receipt must bind the exact request, workspace, context and reviewed saved
states, cover every applied path, and provide one fresh file descriptor for every
surviving source. Partial publication never supplies editor authority. Prepare all
CodeMirror transactions before removing any address, then publish the namespace
synchronously. A move only replaces address metadata; it leaves the cached view
and buffer intact. Deletion keeps the buffer in a separate identity map and blocks
ordinary Save. Restoration supplies that handle explicitly. Saved text comes from
the receipt independently of the restored draft, so dirty state survives undo.

This adapter does not itself provide resource menu admission or restoration IO.
Those callers must retain the request and review and use current local captures.

## Browser preservation and refusal witnesses

These model witnesses use mutable editor handles with selection, scroll and undo state. Routing a swap, deletion, explicit restoration and creation must return those exact objects without touching their state. Refusals produce no partial mutations. This is model evidence, not renderer adoption acceptance.

<ui:copy id="resource-editor-routes-browser-tests">
import {describe,it,expect,vi} from 'vitest';
import {Buffer,Buffers} from './buffers';
import {api} from '../api/client';
import {EditorView} from '@codemirror/view';
const views=vi.hoisted(()=>new WeakMap());
vi.mock('./Document',()=>({editorForBuffer:(buffer:Buffer)=>views.get(buffer)}));
import {EditorState} from '@codemirror/state';
import {history,undoDepth} from '@codemirror/commands';
import {WorkspaceChangeHistory} from './ChangePlan';
import {captureResourceRoster,captureResourceInputs,prepareResourceSave,prepareResourceRestore,publishResourceRestore,RetainedResourceTabs,retireCopiedBuffer,resourceLayout,routeResourceEditors,adoptResourceReceipt,captureResourceEditors,type ResourceEditorPatch,type ResourceReceipt,type ResourceSaved} from './ResourceEditors';
// Contract: .hickory-source/frontends/contract.md (resource namespace preservation).
const source=(identity:string,text=identity)=>({identity,text});
describe('resource editor identity routing',()=>{
 it('captures distinct named and unaddressed retained owners without filename or prototype aliasing',()=>{
  const buffers=new Buffers(()=>{});buffers.workspace='/owned';buffers.workspaceId='a'.repeat(32);
  const live=new Buffer({path:'__proto__',name:'live',dir:false},'independent named text','',()=>{},false,buffers.workspace),deleted=new Buffer({path:'__proto__',name:'deleted',dir:false},'retained deleted text','',()=>{},false,buffers.workspace),loose=new Buffer({path:'Untitled-other',name:'loose',dir:false},'unnamed notes','',()=>{});deleted.resourceDeleted=true;loose.isUntitledDraft=true;buffers.values.set('__proto__',live);buffers.values.set('Untitled-other',loose);buffers.retainedResources.set('deleted',deleted);buffers.retainedResources.set('duplicate-handle',deleted);
  const capture=captureResourceRoster(buffers),again=captureResourceRoster(buffers);expect(capture.captures.length).toBe(3);expect(Object.entries(capture.ownership_sources)).toEqual([['__proto__','independent named text']]);expect(Object.values(capture.unaddressed_sources).sort()).toEqual(['retained deleted text','unnamed notes']);expect(again.unaddressed_sources).toEqual(capture.unaddressed_sources);expect(Object.keys(capture.unaddressed_sources).every(id=>/^[a-f0-9]{32}$/.test(id))).toBe(true);expect(capture.current()).toBe(true);deleted.edit('new deleted text');expect(capture.current()).toBe(false);
 });
 it('retires held resource acquisition when a new retained author, token or file identity appears',async()=>{
  for(const change of [(buffers:Buffers)=>buffers.retainedResources.set('new',new Buffer({path:'deleted.md',name:'deleted',dir:false},'retained','',()=>{})),(_buffers:Buffers,buffer:Buffer)=>{buffer.saveGuard='new-token';},(_buffers:Buffers,buffer:Buffer)=>{buffer.file.doc_id='different-document';}]){
   const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'a.py',name:'a.py',dir:false},'before','base',()=>{});buffer.saveGuard='loaded';buffers.values.set('a.py',buffer);let resolve!:(value:unknown)=>void,body:any;const load=<T>(_path:string,value:unknown)=>{body=value;return new Promise<T>(done=>{resolve=done as (value:unknown)=>void;});};const pending=captureResourceInputs(buffers,['a.py'],'capture',{changes:{}},load,()=>true,async()=> 'b'.repeat(64));await Promise.resolve();await Promise.resolve();change(buffers,buffer);resolve({context:'capture',workspace:'scope',source_files_modified:false,refusals:{},saved_guards:{},sources:{},saved_sources:{}});await expect(pending).rejects.toThrow('changed');expect(body.unaddressed_sources).toEqual({});expect(buffer.text).toBe('before');
  }
 });

 it('admits only the explicitly acquired target handle while preserving every original owner',()=>{const buffers=new Buffers(()=>{}),source=new Buffer({path:'a.py',name:'a.py',dir:false},'before','base',()=>{});buffers.values.set('a.py',source);const roster=captureResourceRoster(buffers),target=new Buffer({path:'b.py',name:'b.py',dir:false},'target','base',()=>{});buffers.values.set('b.py',target);expect(roster.current()).toBe(false);expect(roster.current(new Map([['b.py',target]]))).toBe(true);buffers.retainedResources.set('late',new Buffer({path:'Untitled-late',name:'notes',dir:false},'new construction','',()=>{}));expect(roster.current(new Map([['b.py',target]]))).toBe(false);});
 it('requires the mandatory physical guard map and forwards only originally loaded identities',async()=>{for(const guards of [undefined,null,[],{}, {'a.py':''}]){const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'a.py',name:'a.py',dir:false},'before','hash',()=>{});buffer.saveGuard='original-loaded';buffers.values.set('a.py',buffer);let body:any;const load=async<T>(_path:string,value:unknown)=>{body=value;return {context:'capture',workspace:'workspace',source_files_modified:false,refusals:{},sources:{'a.py':{kind:'file',identity:'source',text:'before'}},saved_sources:{},saved_guards:guards} as T;};await expect(captureResourceInputs(buffers,['a.py'],'capture',{changes:{}},load,()=>true,async()=> 'a'.repeat(64))).rejects.toThrow('acquisition');expect(body.drafts['a.py'].save_guard).toBe('original-loaded');expect(body.saved_guards['a.py']).toBe('original-loaded');expect(buffer.saveGuard).toBe('original-loaded');}});
 it('keeps historical complete outcomes readable but refuses v1, missing or stale adoption authority',()=>{const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'a.py',name:'a.py',dir:false},'a','hash',()=>{});buffers.values.set('a.py',buffer);const patches=[{path:'a.py',before:source('a'),after:null}],expected={request:'request',workspace:'workspace',context:'context',saved:new Map<string,ResourceSaved>([['a.py',{kind:'missing'}]])};const base={version:2,adoption_current:true,request_id:'request',workspace:'workspace',context:'context',outcome:{complete:true,recovery:'retained-history',applied:['a.py']},saved_sources:{'a.py':{kind:'missing'}},source_files:[]};for(const changed of [{version:1},{adoption_current:false,adoption_reason:'Same-byte independent source now owns the path.'},{adoption_current:undefined}]){const receipt={...base,...changed} as ResourceReceipt;expect(()=>adoptResourceReceipt(buffers,patches,receipt,expected,captureResourceEditors(buffers,patches))).toThrow();expect(receipt.outcome.complete).toBe(true);expect(buffers.values.get('a.py')).toBe(buffer);expect(buffers.retainedResources.size).toBe(0);}});
 it('captures prototype-looking source names literally and refuses mismatching restored source before publication',async()=>{
  const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'__proto__',name:'__proto__',dir:false},'base','',()=>{});buffer.saveGuard='loaded';buffer.edit('dirty');buffers.values.set('__proto__',buffer);let context='',reads=0;
  const record={id:'a'.repeat(32),workspace:'scope',fingerprint:'b'.repeat(64),paths:[{path:'__proto__',saved_after:{kind:'file'}}]};
  const load=async<T>(route:string,body:unknown):Promise<T>=>{reads++;const request=body as Record<string,any>;
   if(route.endsWith('resource-inputs')){context=request.context;expect(Object.hasOwn(request.drafts,'__proto__')).toBe(true);expect(Object.getPrototypeOf(request.drafts)).toBeNull();expect(request.drafts['__proto__'].text).toBe('dirty');return {workspace:'scope',context,source_files_modified:false,refusals:{},saved_guards:Object.fromEntries([['__proto__','loaded']]),sources:Object.fromEntries([['__proto__',{kind:'file',identity:'source',text:'dirty'}]]),saved_sources:{}} as T;}
   expect(route).toBe('/api/changes/prepare-restoration');return {version:1,restoration:{workspace:'scope',context,recovery_fingerprint:record.fingerprint,saved_before:{},saved_after:{},plan:{version:1,context,title:'Refuse stale restoration',patches:[{path:'__proto__',before:{kind:'file',identity:'source',text:'base'},after:{kind:'file',identity:'source',text:'older'}}]}},consequences:{version:1,workspace:'scope',context,fingerprint:'c'.repeat(64),protected_targets:[],saved:{entries:[],removed_entries:[],output_paths:[]},draft:{entries:[],removed_entries:[],output_paths:[]}}} as T;
  };
  await expect(prepareResourceRestore(buffers,record,async()=>buffer,load,()=>true,async()=> 'd'.repeat(64))).rejects.toThrow('does not match');expect(reads).toBe(2);expect(buffer.text).toBe('dirty');expect(buffer.base).toBe('base');
 });

 it('previews whole resource publication, rejects expanded namespaces and consumes uncertain requests without retry',async()=>{
  const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'old.txt',name:'old.txt',dir:false},'source','old',()=>{});buffer.saveGuard='loaded';buffers.values.set('old.txt',buffer);
  const saved:ResourceSaved={kind:'file',text:'source',sha256:'a'.repeat(64),readonly:false,executable:false,mode:0o600},edit={documentChanges:[{kind:'rename',oldUri:'hick:///old.txt',newUri:'hick:///new.txt'}]};
  let context='',writes=0;const workspace='scope';
  const load=async<T>(route:string,body:unknown):Promise<T>=>{const request=body as Record<string,any>;context=request.context??context;
   if(route.endsWith('resource-targets'))return {paths:['old.txt','new.txt']} as T;
   if(route.endsWith('resource-inputs'))return {workspace,context,source_files_modified:false,refusals:{},saved_guards:{'old.txt':'loaded'},sources:{'old.txt':{kind:'file',identity:'source',text:'source'},'new.txt':{kind:'missing'}},saved_sources:{'old.txt':saved,'new.txt':{kind:'missing'}}} as T;
   if(route.endsWith('plan-resources'))return {plan:{version:1,context,title:'Move source',patches:[{path:'old.txt',before:{kind:'file',identity:'source',text:'source'},after:{kind:'missing'}},{path:'new.txt',before:{kind:'missing'},after:{kind:'file',identity:'source',text:'source'}}]}} as T;
   if(route.endsWith('resource-consequences'))return {version:1,workspace,context,saved:{entries:[],removed_entries:[],output_paths:[]},draft:{entries:[],removed_entries:[],output_paths:[]},fingerprint:'c'.repeat(64),protected_targets:[],saved_sources:{'old.txt':{kind:'missing'},'new.txt':saved}} as T;
   writes++;throw new Error('Transport result unavailable');
  };
  const prepare=()=>prepareResourceSave(buffers,'Move source',edit,async()=>buffer,load,()=>true,async()=> 'a'.repeat(64));
  const changed=await prepare();const extra=new Buffer({path:'extra.md',name:'extra.md',dir:false},'new ownership','',()=>{});buffers.values.set('extra.md',extra);expect(changed.current()).toBe(false);buffers.values.delete('extra.md');buffer.file.generated_by='owner';expect(changed.current()).toBe(false);delete buffer.file.generated_by;
  const review=await prepare();expect(review.operation).toBe('save');expect(review.saved['new.txt']).toEqual(saved);expect(review.savedBefore['old.txt']).toEqual(saved);
  await expect(publishResourceRestore(buffers,review,load)).rejects.toThrow('not confirmed');await expect(publishResourceRestore(buffers,review,load)).rejects.toThrow('already has publication request');expect(writes).toBe(1);expect(buffers.values.get('old.txt')).toBe(buffer);expect(buffers.values.has('new.txt')).toBe(false);
 });
 it('coordinates draft and resource undo with a bounded history and retires interleaved review tokens',()=>{
  const history=new WorkspaceChangeHistory(),draft={title:'Source',patches:[]};history.record({kind:'draft',plan:draft,read:()=>draft});history.record({kind:'resource',recovery:'record'});
  const old=history.preview('undo')!;history.record({kind:'draft',plan:draft,read:()=>draft});expect(history.current(old)).toBe(false);expect(()=>history.commit(old,{kind:'resource',recovery:'wrong'})).toThrow('history changed');
  const latest=history.preview('undo')!;history.commit(latest,{kind:'draft',plan:draft,read:()=>draft});expect(history.preview('redo')?.entry.kind).toBe('draft');const resource=history.preview('undo')!;expect(resource.entry).toEqual({kind:'resource',recovery:'record'});history.commit(resource,{kind:'resource',recovery:'inverse'});expect(history.preview('redo')?.entry).toEqual({kind:'resource',recovery:'inverse'});
  for(let i=0;i<40;i++)history.record({kind:'resource',recovery:String(i)});let count=0;for(let next=history.preview('undo');next;next=history.preview('undo')){history.commit(next,{kind:'resource',recovery:'redo'});count++;}expect(count).toBe(32);
 });

 it('prepares and publishes explicit restoration with separate baselines, identity-bound receipts and no stale retry',async()=>{
  const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'b.txt',name:'b.txt',dir:false},'dirty','old',()=>{});buffer.saveGuard='loaded';buffers.values.set('b.txt',buffer);
  const saved:ResourceSaved={kind:'file',text:'saved',sha256:'a'.repeat(64),readonly:false,executable:false,mode:null};
  const calls:string[]=[];let context='',workspace='workspace';
  const load=async<T>(route:string,body:unknown):Promise<T>=>{calls.push(route);const request=body as Record<string,any>;
   if(route.endsWith('resource-inputs')){context=request.context;expect(request.drafts['b.txt'].text).toBe('dirty');return {workspace,context,source_files_modified:false,refusals:{},saved_guards:{'b.txt':'loaded'},sources:{'b.txt':{kind:'file',identity:'source',text:'dirty'},'a.txt':{kind:'missing'}},saved_sources:{}} as T;}
   if(route.endsWith('prepare-restoration'))return {version:1,restoration:{workspace,context,recovery_fingerprint:'c'.repeat(64),saved_before:{'b.txt':{...saved,text:'dirty'},'a.txt':{kind:'missing'}},saved_after:{'a.txt':saved,'b.txt':{kind:'missing'}},plan:{version:1,context,title:'Restore',patches:[{path:'b.txt',before:{kind:'file',identity:'source',text:'dirty'},after:{kind:'missing'}},{path:'a.txt',before:{kind:'missing'},after:{kind:'file',identity:'source',text:'dirty'}}]}},consequences:{version:1,workspace,context,saved:{entries:[],removed_entries:[],output_paths:[]},draft:{entries:[],removed_entries:[],output_paths:[]},fingerprint:'d'.repeat(64),protected_targets:[]}} as T;
   expect(route).toBe('/api/changes/restore-resources');return {version:2,adoption_current:true,request_id:request.request_id,workspace,context,outcome:{complete:true,recovery:'.hick-cache/resource-changes/'+ 'e'.repeat(32)+'/manifest.json',applied:['a.txt','b.txt']},saved_sources:{'a.txt':saved,'b.txt':{kind:'missing'}},source_files:[{file:{path:'a.txt',name:'a.txt',dir:false},saved,hash:'new'}]} as T;
  };
  const record={id:'b'.repeat(32),workspace,fingerprint:'c'.repeat(64),paths:[{path:'a.txt',saved_after:{kind:'missing'}},{path:'b.txt',saved_after:{kind:'file'}}]};
  const prepare=()=>prepareResourceRestore(buffers,record,async()=>buffer,load,()=>true,async()=> 'a'.repeat(64));
  const stale=await prepare();buffer.edit('independent');await expect(publishResourceRestore(buffers,stale,load)).rejects.toThrow('Source changed');expect(calls.filter(route=>route.endsWith('restore-resources'))).toHaveLength(0);
  buffer.edit('dirty');const review=await prepare(),result=await publishResourceRestore(buffers,review,load);expect(result.recovery).toBe('e'.repeat(32));expect(buffers.values.get('a.txt')).toBe(buffer);expect(buffers.values.has('b.txt')).toBe(false);expect(buffer.text).toBe('dirty');expect(buffer.base).toBe('saved');expect(buffer.dirty).toBe(true);
  await expect(publishResourceRestore(buffers,review,load)).rejects.toThrow('already has publication request');expect(calls.filter(route=>route.endsWith('restore-resources'))).toHaveLength(1);
 });
 it('routes live tab swaps without losing pane identity and explicitly restores deleted placement',()=>{
  const a=new Buffer({path:'a',name:'a',dir:false},'a','',()=>{}),b=new Buffer({path:'b',name:'b',dir:false},'b','',()=>{}),placements=new Map<Buffer,{group:string;index:number;active:boolean}>();
  const layout={id:'pane',tabs:['a','b'],active:'a'},route=(editor:Buffer,from:string|null,to:string|null)=>({identity:editor.text,from,to,before:editor.text,after:editor.text,editor});
  const swapped=resourceLayout(layout,[route(a,'a','b'),route(b,'b','a')],placements);expect(swapped).toEqual({id:'pane',tabs:['b','a'],active:'b'});expect(layout.tabs).toEqual(['a','b']);
  const deleted=resourceLayout(swapped,[route(a,'b',null)],placements);expect(deleted).toEqual({id:'pane',tabs:['a'],active:'a'});expect(placements.get(a)).toEqual({group:'pane',index:0,active:true});
  const restored=resourceLayout(deleted,[route(a,null,'restored')],placements);expect(restored).toEqual({id:'pane',tabs:['restored','a'],active:'restored'});expect(placements.size).toBe(0);
 });

 it('adopts a confirmed move, deletion and restoration into live buffers with separate saved baselines',async()=>{
  const buffers=new Buffers(()=>{}),a=new Buffer({path:'old.py',name:'old.py',dir:false},'saved','old-hash',()=>{});a.edit('dirty');buffers.values.set('old.py',a);
  const state=EditorState.create({doc:'saved',extensions:[history()]}).update({changes:{from:0,to:5,insert:'dirty'},selection:{anchor:2}}).state;
  const view=new EditorView({state});views.set(a,{view});view.scrollDOM.scrollTop=73;
  const retained=buffers.retainedResources,saved:ResourceSaved={kind:'file',text:'dirty',sha256:'a'.repeat(64),readonly:false,executable:false,mode:null};
  function adopt(patches:ResourceEditorPatch[],baseline:ResourceSaved,restoring=new Map<string,Buffer>()){
   const expected={request:'request',workspace:'workspace',context:'context',saved:new Map(patches.map(p=>[p.path,p.after?baseline:{kind:'missing'} as ResourceSaved]))};
   const receipt:ResourceReceipt={version:2,adoption_current:true,request_id:'request',workspace:'workspace',context:'context',outcome:{complete:true,recovery:'recovery',applied:patches.map(p=>p.path)},saved_sources:Object.fromEntries(expected.saved),source_files:patches.filter(p=>p.after).map(p=>({file:{path:p.path,name:p.path,dir:false},saved:baseline,hash:'fresh'}))};
   return adoptResourceReceipt(buffers,patches,receipt,expected,captureResourceEditors(buffers,patches,restoring),restoring);
  }
  const original=view.state;
  adopt([{path:'old.py',before:source('a','dirty'),after:null},{path:'new.py',before:null,after:source('a','dirty')}],saved);
  expect(buffers.values.get('new.py')).toBe(a);expect(view.state).toBe(original);expect(a.base).toBe('dirty');expect(a.dirty).toBe(false);
  a.base='saved';
  adopt([{path:'new.py',before:source('a','dirty'),after:null}],saved);expect(retained.get('a')).toBe(a);expect(a.resourceDeleted).toBe(true);expect(buffers.hasUnsavedChanges).toBe(true);
  const write=vi.spyOn(api,'saveFile').mockRejectedValue(new Error('Deleted editors cannot write.'));await a.save();expect(write).not.toHaveBeenCalled();write.mockRestore();
  adopt([{path:'old.py',before:null,after:source('restored','dirty')}],{...saved,text:'saved'},new Map([['restored',a]]));
  expect(buffers.values.get('old.py')).toBe(a);expect(a.base).toBe('saved');expect(a.text).toBe('dirty');expect(a.dirty).toBe(true);expect(a.resourceDeleted).toBe(false);
  expect(view.state).toBe(original);expect(view.state.selection.main.anchor).toBe(2);expect(undoDepth(view.state)).toBe(1);expect(view.scrollDOM.scrollTop).toBe(73);view.destroy();
 });
 it('refuses partial receipts and changed local baselines without changing any live address',()=>{
  const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'a.py',name:'a.py',dir:false},'a','hash',()=>{});buffers.values.set('a.py',buffer);
  const patches=[{path:'a.py',before:source('a'),after:null}],expected={request:'r',workspace:'w',context:'c',saved:new Map<string,ResourceSaved>([['a.py',{kind:'missing'}]])};
  const receipt:ResourceReceipt={version:2,adoption_current:true,request_id:'r',workspace:'w',context:'c',outcome:{complete:false,recovery:'recovery',applied:[]},saved_sources:null,source_files:[]},retained=buffers.retainedResources;
  expect(()=>adoptResourceReceipt(buffers,patches,receipt,expected,captureResourceEditors(buffers,patches))).toThrow();
  const current=captureResourceEditors(buffers,patches);buffer.base='independent';
  expect(()=>adoptResourceReceipt(buffers,patches,{...receipt,outcome:{complete:true,recovery:'recovery',applied:['a.py']},saved_sources:{'a.py':{kind:'missing'}}},expected,current)).toThrow();
  expect(buffers.values.get('a.py')).toBe(buffer);expect(buffer.base).toBe('independent');expect(retained.size).toBe(0);
 });

 it('retains editor state through swaps, deletion and explicit restoration',()=>{
  const a={selection:[2,4],scroll:108,undo:['old'],draft:'dirty'},b={selection:[0,0],scroll:0,undo:[],draft:'B'};
  const live=new Map([['a',{editor:a,source:'dirty'}],['b',{editor:b,source:'B'}]]);
  const patches:ResourceEditorPatch[]=[{path:'a.py',before:source('a','dirty'),after:source('b','B')},{path:'b.py',before:source('b','B'),after:source('a','dirty')}];
  const swap=routeResourceEditors(patches,live);expect(swap.find(x=>x.to==='b.py')?.editor).toBe(a);expect(swap.find(x=>x.to==='a.py')?.editor).toBe(b);
  const deleted=routeResourceEditors([{path:'b.py',before:source('a','dirty'),after:null}],new Map([['a',live.get('a')!]]));expect(deleted[0].editor).toBe(a);expect(deleted[0].to).toBeNull();
  const restored=routeResourceEditors([{path:'b.py',before:null,after:source('restored','dirty')}],new Map(),new Map([['restored',live.get('a')!]]));expect(restored[0].editor).toBe(a);
  const created=routeResourceEditors([{path:'new.py',before:null,after:source('new','')}],new Map());expect(created[0].editor).toBeNull();expect(created[0].after).toBe('');
  expect(a).toEqual({selection:[2,4],scroll:108,undo:['old'],draft:'dirty'});
 });
 it('retains actual CodeMirror selection and undo history through a move and inverse',()=>{
  let state=EditorState.create({doc:'saved',extensions:[history()]});state=state.update({changes:{from:5,insert:' dirty'},selection:{anchor:3}}).state;
  const editor={state,scrollTop:201},capture={editor,source:state.doc.toString()},a=source('a',capture.source);
  const moved=routeResourceEditors([{path:'old.py',before:a,after:null},{path:'new.py',before:null,after:a}],new Map([['a',capture]]));
  const inverse=routeResourceEditors([{path:'new.py',before:a,after:null},{path:'old.py',before:null,after:a}],new Map([['a',capture]]));
  expect(moved[0].editor).toBe(editor);expect(inverse[0].editor?.state).toBe(state);expect(editor.state.selection.main.anchor).toBe(3);expect(undoDepth(editor.state)).toBe(1);expect(editor.scrollTop).toBe(201);
 });
 it('refuses duplicate, stale, missing and aliased captures before mutation',()=>{
  const editor={undo:['retained']},capture={editor,source:'a'},live=new Map([['a',capture]]),patch={path:'a.py',before:source('a'),after:source('a','changed')};
  expect(()=>routeResourceEditors([patch,patch],live)).toThrow();expect(()=>routeResourceEditors([patch],new Map())).toThrow();expect(()=>routeResourceEditors([patch],new Map([['a',{editor,source:'later'}]]))).toThrow();
  expect(()=>routeResourceEditors([patch],live,new Map([['a',capture]]))).toThrow();expect(()=>routeResourceEditors([patch,{path:'b.py',before:source('b'),after:null}],new Map([...live,['b',{editor,source:'b'}]]))).toThrow();
  expect(editor.undo).toEqual(['retained']);
  const exact=routeResourceEditors([{path:'é.py',before:null,after:source('é')},{path:'e\u0301.py',before:null,after:source('e\u0301')}],new Map());expect(exact).toHaveLength(2);
 });
 it('retains active deleted editor tabs, restores the same route and retires only an exact copied handle',()=>{
  const buffers=new Buffers(()=>{}),original=new Buffer({path:'old.py',name:'old.py',dir:false},'retained','old',()=>{}),independent=new Buffer({path:'old.py',name:'old.py',dir:false},'independent','new',()=>{}),tabs=new RetainedResourceTabs();
  const layout={id:'one',tabs:['old.py'],active:'old.py'};const deletion={identity:'source',from:'old.py',to:null,editor:original,before:'retained',after:null};original.resourceDeleted=true;buffers.retainedResources.set('source',original);buffers.values.set('old.py',independent);
  const held=resourceLayout(layout,tabs.routes([deletion],buffer=>buffer===original),new Map());const alias=(held as typeof layout).active!;expect(alias.startsWith('\0')).toBe(true);expect(tabs.buffers.get(alias)).toBe(original);expect(buffers.values.get('old.py')).toBe(independent);
  const restored=resourceLayout(held,tabs.routes([{...deletion,from:null,to:'restored.py',after:'retained'}],()=>false),new Map());expect((restored as typeof layout).tabs).toEqual(['restored.py']);expect(tabs.buffers.size).toBe(0);
  expect(retireCopiedBuffer(buffers,original,'stale')).toBe(false);expect(buffers.retainedResources.get('source')).toBe(original);tabs.noteCopy(original,original.text);expect(tabs.copied(original)).toBe(true);original.edit('new unsaved');expect(tabs.copied(original)).toBe(false);
  expect(retireCopiedBuffer(buffers,original,original.text)).toBe(true);expect(buffers.retainedResources.size).toBe(0);expect(buffers.values.get('old.py')).toBe(independent);
 });
});
</ui:copy>

<ui:file path="apps/web/src/simple/ResourceEditors.test.ts"><ui:paste select="#resource-editor-routes-browser-tests"/></ui:file>

## Apple preservation and refusal witnesses

The same lifecycle retains reference identity on both Apple hosts. Byte-distinct paths remain separate; duplicate addresses, missing captures, stale source and aliasing one editor across identities refuse.

<ui:copy id="resource-editor-routes-apple-tests">
import XCTest
@testable import HickorySimple
// Contract: .hickory-source/frontends/contract.md (resource namespace preservation).
final class ResourceEditorsTests:XCTestCase {
 final class Editor {var selection=2;var scroll=108;var undo=["old"]}
 func testTabAddressesMoveSimultaneouslyWithoutChangingPaneGeometry() throws {
  var layout=PaneLayout();layout.activate("a.py",in:"document-group");layout.activate("b.py",in:"document-group")
  let before=layout
  let swapped=try layout.remappingTabs([Data("a.py".utf8):"b.py",Data("b.py".utf8):"a.py"])
  XCTAssertEqual(swapped.nodes["document-group"]?.tabs,["b.py","a.py"]);XCTAssertEqual(swapped.nodes["document-group"]?.active,"a.py")
  XCTAssertEqual(swapped.nodes["root"],before.nodes["root"]);XCTAssertEqual(swapped.rootID,before.rootID)
  XCTAssertThrowsError(try layout.remappingTabs([Data("a.py".utf8):"b.py"]))
  XCTAssertEqual(layout.nodes,before.nodes)
 }
 @MainActor func testLiveReceiptAdoptsMoveDeletionAndExplicitRetainedRestoration() throws {
  let workspace=Workspace();workspace.engine=try Engine(address:"http://127.0.0.1:1")
  let source=ResourceSource(identity:"source",text:"dirty"),missing=ResourceSaved(kind:"missing")
  let saved=ResourceSaved(kind:"file",text:"saved",sha256:String(repeating:"a",count:64),readonly:false,executable:false,mode:nil)
  let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"saved",hash:"old"));buffer.text="dirty";buffer.selection=NSRange(location:2,length:1)
  workspace.buffers["a.py"]=buffer;workspace.layout.activate("a.py",in:"document-group")
  let geometry=workspace.layout.nodes["root"]
  func receipt(_ paths:[String],_ values:[String:ResourceSaved],_ path:String?)->ResourcePublication<FileNode> {
   ResourcePublication(version:2,request_id:"request",workspace:"workspace",context:"context",outcome:.init(complete:true,recovery:"record",applied:paths),saved_sources:values,source_files:path.map{[.init(file:FileNode(name:$0,path:$0,dir:false),saved:saved,hash:"fresh")]} ?? [],adoption_current:true)
  }
  let move=[ResourceEditorPatch(path:"a.py",before:source,after:nil),ResourceEditorPatch(path:"b.py",before:nil,after:source)]
  let moving=workspace.captureResourceEditors(move)
  _ = try workspace.adoptResourceReceipt(move,receipt:receipt(["a.py","b.py"],["a.py":missing,"b.py":saved],"b.py"),request:"request",workspace:"workspace",context:"context",expected:["a.py":missing,"b.py":saved],current:moving)
  XCTAssertTrue(workspace.buffers["b.py"] === buffer);XCTAssertNil(workspace.buffers["a.py"]);XCTAssertEqual(workspace.active,"b.py");XCTAssertTrue(workspace.currentBuffer === buffer)
  XCTAssertEqual(buffer.selection,NSRange(location:2,length:1));XCTAssertEqual(buffer.text,"dirty");XCTAssertEqual(buffer.base,"saved");XCTAssertTrue(buffer.dirty)
  let deletion=[ResourceEditorPatch(path:"b.py",before:source,after:nil)]
  _ = try workspace.adoptResourceReceipt(deletion,receipt:receipt(["b.py"],["b.py":missing],nil),request:"request",workspace:"workspace",context:"context",expected:["b.py":missing],current:workspace.captureResourceEditors(deletion))
  let retained=workspace.active!
  XCTAssertTrue(buffer.readOnly);XCTAssertTrue(workspace.currentBuffer === buffer);XCTAssertNil(workspace.buffers["b.py"]);XCTAssertTrue(workspace.hasUnsavedChanges)
  workspace.close(retained);XCTAssertEqual(workspace.active,retained)
  let restore=[ResourceEditorPatch(path:"b.py",before:nil,after:source)]
  let restoring=["source":buffer]
  _ = try workspace.adoptResourceReceipt(restore,receipt:receipt(["b.py"],["b.py":saved],"b.py"),request:"request",workspace:"workspace",context:"context",expected:["b.py":saved],current:workspace.captureResourceEditors(restore,restoring:restoring),restoring:restoring)
  XCTAssertTrue(workspace.currentBuffer === buffer);XCTAssertEqual(workspace.active,"b.py");XCTAssertFalse(buffer.readOnly);XCTAssertTrue(workspace.retainedResourceEditors.isEmpty)
  XCTAssertEqual(workspace.layout.nodes["root"],geometry);XCTAssertEqual(buffer.selection,NSRange(location:2,length:1))
  buffer.language?.close()
 }

 func testMoveDeleteRestoreAndCreateKeepOpaqueEditorState() throws {
  let editor=Editor(),a=ResourceSource(identity:"a",text:"dirty"),capture=ResourceEditorCapture(editor:editor,source:"dirty")
  let move=try routeResourceEditors([ResourceEditorPatch(path:"a.py",before:a,after:nil),ResourceEditorPatch(path:"b.py",before:nil,after:a)],live:["a":capture]);XCTAssertTrue(move[0].editor===editor);XCTAssertEqual(move[0].to,"b.py")
  let other=Editor(),b=ResourceSource(identity:"b",text:"B"),otherCapture=ResourceEditorCapture(editor:other,source:"B")
  let swapped=try routeResourceEditors([ResourceEditorPatch(path:"a.py",before:a,after:b),ResourceEditorPatch(path:"b.py",before:b,after:a)],live:["a":capture,"b":otherCapture]);XCTAssertTrue(swapped.first(where:{$0.to=="b.py"})?.editor===editor);XCTAssertTrue(swapped.first(where:{$0.to=="a.py"})?.editor===other)
  let deleted=try routeResourceEditors([ResourceEditorPatch(path:"b.py",before:a,after:nil)],live:["a":capture]);XCTAssertNil(deleted[0].to);XCTAssertTrue(deleted[0].editor===editor)
  let restored=try routeResourceEditors([ResourceEditorPatch(path:"b.py",before:nil,after:ResourceSource(identity:"restored",text:"dirty"))],live:[:],retained:["restored":capture]);XCTAssertTrue(restored[0].editor===editor)
  let created:[ResourceEditorRoute<Editor>]=try routeResourceEditors([ResourceEditorPatch(path:"new.py",before:nil,after:ResourceSource(identity:"new",text:""))],live:[:]);XCTAssertNil(created[0].editor);XCTAssertEqual(created[0].after,"")
  XCTAssertEqual(editor.selection,2);XCTAssertEqual(editor.scroll,108);XCTAssertEqual(editor.undo,["old"])
 }
 func testConfirmedReceiptRequiresWholeNamespaceAndSeparateSavedBaseline() throws {
  let saved=ResourceSaved(kind:"file",text:"saved",sha256:String(repeating:"a",count:64),readonly:false,executable:false,mode:nil)
  let absent=ResourceSaved(kind:"missing"),patches=[ResourceEditorPatch(path:"old.py",before:ResourceSource(identity:"a",text:"dirty"),after:nil),ResourceEditorPatch(path:"new.py",before:nil,after:ResourceSource(identity:"a",text:"dirty"))]
  var receipt=ResourcePublication<String>(version:2,request_id:"request",workspace:"workspace",context:"context",outcome:.init(complete:true,recovery:"record",applied:["old.py","new.py"]),saved_sources:["old.py":absent,"new.py":saved],source_files:[.init(file:"new.py",saved:saved,hash:"fresh")],adoption_current:true)
  func confirm() throws->[Data:ResourcePublication<String>.Entry] {try confirmedResourceFiles(patches,receipt:receipt,request:"request",workspace:"workspace",context:"context",expected:["old.py":absent,"new.py":saved]){($0,true)}}
  XCTAssertEqual(try confirm()[Data("new.py".utf8)]?.saved.text,"saved")
  receipt.version=1;XCTAssertThrowsError(try confirm());receipt.version=2
  receipt.adoption_current=false;receipt.adoption_reason="Historical publication; source was replaced.";XCTAssertThrowsError(try confirm());receipt.adoption_current=true
  receipt.outcome.complete=false;XCTAssertThrowsError(try confirm());receipt.outcome.complete=true
  receipt.request_id="other";XCTAssertThrowsError(try confirm());receipt.request_id="request"
  receipt.source_files.append(receipt.source_files[0]);XCTAssertThrowsError(try confirm())
 }
 func testCompletePreflightAndByteExactNamespaces() throws {
  let editor=Editor(),capture=ResourceEditorCapture(editor:editor,source:"a"),a=ResourceSource(identity:"a",text:"a"),patch=ResourceEditorPatch(path:"a.py",before:a,after:nil)
  XCTAssertThrowsError(try routeResourceEditors([patch,patch],live:["a":capture]));XCTAssertThrowsError(try routeResourceEditors([patch],live:[String:ResourceEditorCapture<Editor>]()))
  XCTAssertThrowsError(try routeResourceEditors([patch],live:["a":ResourceEditorCapture(editor:editor,source:"later")]))
  XCTAssertThrowsError(try routeResourceEditors([patch],live:["a":capture],retained:["a":capture]))
  let routes:[ResourceEditorRoute<Editor>]=try routeResourceEditors([ResourceEditorPatch(path:"é.py",before:nil,after:ResourceSource(identity:"first",text:"")),ResourceEditorPatch(path:"e\u{301}.py",before:nil,after:ResourceSource(identity:"second",text:""))],live:[:]);XCTAssertEqual(routes.count,2)
  XCTAssertEqual(editor.undo,["old"])
 }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/ResourceEditorsTests.swift"><ui:paste select="#resource-editor-routes-apple-tests"/></ui:file>

## Android preservation and refusal witnesses

Reference equality matters even for editor wrappers that define value equality. The identity set uses JVM reference identity, preserving distinct editor instances while rejecting a single instance assigned to two sources.

<ui:copy id="resource-editor-routes-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
// Contract: .hickory-source/frontends/contract.md (resource namespace preservation).
class ResourceEditorsTest {
 class Editor {var selection=2;var scroll=108;val undo=mutableListOf("old")}
 @Test fun moveDeleteRestoreAndCreateKeepOpaqueEditorState(){
  val editor=Editor();val a=ResourceSource("a","dirty");val capture=ResourceEditorCapture(editor,"dirty")
  val move=routeResourceEditors(listOf(ResourceEditorPatch("a.py",a,null),ResourceEditorPatch("b.py",null,a)),mapOf("a" to capture));assertSame(editor,move.single().editor);assertEquals("b.py",move.single().to)
  val other=Editor();val b=ResourceSource("b","B");val swapped=routeResourceEditors(listOf(ResourceEditorPatch("a.py",a,b),ResourceEditorPatch("b.py",b,a)),mapOf("a" to capture,"b" to ResourceEditorCapture(other,"B")));assertSame(editor,swapped.first{it.to=="b.py"}.editor);assertSame(other,swapped.first{it.to=="a.py"}.editor)
  val deleted=routeResourceEditors(listOf(ResourceEditorPatch("b.py",a,null)),mapOf("a" to capture));assertNull(deleted.single().to);assertSame(editor,deleted.single().editor)
  val restored=routeResourceEditors(listOf(ResourceEditorPatch("b.py",null,ResourceSource("restored","dirty"))),emptyMap(),mapOf("restored" to capture));assertSame(editor,restored.single().editor)
  val created=routeResourceEditors<Editor>(listOf(ResourceEditorPatch("new.py",null,ResourceSource("new",""))),emptyMap());assertNull(created.single().editor);assertEquals("",created.single().after)
  assertEquals(2,editor.selection);assertEquals(108,editor.scroll);assertEquals(listOf("old"),editor.undo)
 }
 @Test fun confirmedReceiptRequiresWholeNamespaceAndSeparateSavedBaseline(){
  val saved=ResourceSaved("file","saved","a".repeat(64),false,false,null);val absent=ResourceSaved("missing")
  val patches=listOf(ResourceEditorPatch("old.py",ResourceSource("a","dirty"),null),ResourceEditorPatch("new.py",null,ResourceSource("a","dirty")))
  val receipt=ResourcePublication(2,"request","workspace","context",true,"record",listOf("old.py","new.py"),mapOf("old.py" to absent,"new.py" to saved),listOf(ResourcePublishedFile("new.py",saved,"fresh")),adoptionCurrent=true)
  fun confirm(value:ResourcePublication<String>)=confirmedResourceFiles(patches,value,"request","workspace","context",mapOf("old.py" to absent,"new.py" to saved)){it to true}
  assertEquals("saved",confirm(receipt).getValue("new.py").saved.text)
  assertTrue(runCatching{confirm(receipt.copy(files=listOf(ResourcePublishedFile("new.py",saved,"fresh",""))))}.isFailure)
  for(value in listOf(receipt.copy(version=1),receipt.copy(complete=false),receipt.copy(adoptionCurrent=false,adoptionReason="Historical completed publication; source was replaced."),receipt.copy(request="other"),receipt.copy(files=receipt.files+receipt.files),receipt.copy(applied=listOf("new.py")))){try{confirm(value);fail("Expected refusal")}catch(_:IllegalArgumentException){}}
 }
 @Test fun completePreflightAndByteExactNamespaces(){
  val editor=Editor();val capture=ResourceEditorCapture(editor,"a");val a=ResourceSource("a","a");val patch=ResourceEditorPatch("a.py",a,null)
  fun refuses(block:()->Unit){try{block();fail("Expected refusal")}catch(_:IllegalArgumentException){}}
  refuses{routeResourceEditors(listOf(patch,patch),mapOf("a" to capture))};refuses{routeResourceEditors<Editor>(listOf(patch),emptyMap())}
  refuses{routeResourceEditors(listOf(patch),mapOf("a" to ResourceEditorCapture(editor,"later")))};refuses{routeResourceEditors(listOf(patch),mapOf("a" to capture),mapOf("a" to capture))}
  refuses{routeResourceEditors(listOf(patch,ResourceEditorPatch("b.py",ResourceSource("b","b"),null)),mapOf("a" to capture,"b" to ResourceEditorCapture(editor,"b")))}
  assertEquals(2,routeResourceEditors<Editor>(listOf(ResourceEditorPatch("é.py",null,ResourceSource("é","")),ResourceEditorPatch("e\u0301.py",null,ResourceSource("e\u0301",""))),emptyMap()).size)
  assertEquals(listOf("old"),editor.undo)
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/ResourceEditorsTest.kt"><ui:paste select="#resource-editor-routes-android-tests"/></ui:file>

</ui:doc>
