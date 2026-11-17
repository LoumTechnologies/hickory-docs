import {authoredSource,authoredChanges,authoredTransactionSource} from './SourceText';
import {Buffer,type Buffers} from './buffers';
import type {FileNode} from '../api/types';
import {editorForBuffer} from './Document';
import {captureRetainedEditorViewport} from './EditorMounts';
import {EditorView} from '@codemirror/view';
import {isolateHistory} from '@codemirror/commands';
import type {Transaction} from '@codemirror/state';
import {isGroup,groups,type Layout} from './layout';
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

export type ResourceSaved = {kind:'missing'}|{kind:'file';text:string;sha256:string;readonly:boolean;executable:boolean;mode:number|null};
export interface ResourceReceipt {version:number;adoption_current:boolean;adoption_reason?:string;request_id:string;workspace:string;context:string;outcome:{complete:boolean;recovery:string;applied:string[]};saved_sources:Record<string,ResourceSaved>|null;source_files:{file:FileNode;saved:ResourceSaved;hash:string;save_guard?:string}[]}
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
export function captureResourceEditors(buffers:Buffers,patches:readonly ResourceEditorPatch[],restoring:ReadonlyMap<string,Buffer>=new Map()):()=>boolean {
 const paths=patches.map(patch=>({path:patch.path,buffer:buffers.values.get(patch.path)}));
 const captures=[...new Set([...paths.flatMap(row=>row.buffer?[row.buffer]:[]),...restoring.values()])].map(buffer=>({buffer,text:buffer.text,base:buffer.base,hash:buffer.hash,version:buffer.version,file:buffer.file}));
 return ()=>paths.every(row=>buffers.values.get(row.path)===row.buffer)&&captures.every(row=>row.buffer.text===row.text&&row.buffer.base===row.base&&row.buffer.hash===row.hash&&row.buffer.version===row.version&&row.buffer.file===row.file&&!row.buffer.saving);
}
export function adoptResourceReceipt(buffers:Buffers,patches:readonly ResourceEditorPatch[],receipt:ResourceReceipt,expected:ResourceExpectation,current:()=>boolean,restoring:ReadonlyMap<string,Buffer>=new Map()):ResourceEditorRoute<Buffer>[] {
 if(!current())throw new Error('Editor drafts or baselines changed after review. Preserve them and inspect Source recovery.');
 const retained=buffers.retainedResources;
 const files=confirmedResourceFiles(patches,receipt,expected),live=new Map<string,ResourceEditorCapture<Buffer>>();
 for(const patch of patches)if(patch.before){const buffer=buffers.values.get(patch.path);if(!buffer||buffer.saving||buffer.readonly||buffer.transient)throw new Error('Source editor is unavailable or saving. Preserve drafts and inspect recovery.');live.set(patch.before.identity,{editor:buffer,source:buffer.text});}
 for(const buffer of restoring.values())if(!buffer.resourceDeleted||![...retained.values()].includes(buffer)||[...buffers.values.values()].includes(buffer))throw new Error('Restoration must select a retained deleted editor. Inspect Source recovery.');
 const held=new Map([...restoring].map(([id,editor])=>[id,{editor,source:editor.text}]));
 const routes=routeResourceEditors(patches,live,held),prepared:{route:ResourceEditorRoute<Buffer>;buffer:Buffer;view?:EditorView;transaction?:Transaction}[]=[];
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
 proposal:unknown;current:()=>boolean;restoring:Map<string,Buffer>;
}
export type ResourceTransport=<T>(route:string,body:unknown)=>Promise<T>;
export function resourceWirePatches(plan:ResourceWirePlan):ResourceEditorPatch[]{
 if(plan.version!==1||!plan.context||!plan.patches.length||plan.patches.length>1000)throw new Error('Resource plan is unavailable or exceeds the review bound.');
 const side=(value:ResourceWireState):ResourceSource|null=>{if(value.kind==='missing')return null;if(value.kind!=='file'||!value.identity||typeof value.text!=='string')throw new Error('Resource source identity is unavailable.');return {identity:value.identity,text:value.text};};
 return plan.patches.map(patch=>({path:patch.path,before:side(patch.before),after:side(patch.after)}));
}
/** Preparation observes current saved sources and drafts; it never publishes. */
export async function prepareResourceRestore(buffers:Buffers,record:{id:string;workspace:string;fingerprint:string;paths:{path:string;saved_after:{kind:string}}[]},acquire:(path:string)=>Promise<Buffer>,load:ResourceTransport,alive:()=>boolean,hash:(text:string)=>Promise<string>=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),byte=>byte.toString(16).padStart(2,'0')).join('')):Promise<ResourceRestoreReview>{
 if(!/^[0-9a-f]{32}$/.test(record.id)||!record.paths.length||record.paths.length>1000)throw new Error('Choose a bounded recovery record.');
 for(const item of record.paths)if(item.saved_after.kind==='file')await acquire(item.path);
 if(!alive())throw new Error('Workspace or recovery selection changed. Prepare restoration again.');
 const paths=new Set(record.paths.map(item=>item.path));if(paths.size!==record.paths.length)throw new Error('Recovery paths are not unique.');
 const context=crypto.randomUUID().replaceAll('-',''),edit={changes:Object.fromEntries(record.paths.map(item=>['hick:///'+item.path.split('/').map(encodeURIComponent).join('/'),[]]))};
 const {inputs,capture,current}=await captureResourceInputs(buffers,[...paths],context,edit,load,alive,hash);
 if(inputs.workspace!==record.workspace)throw new Error('Current source acquisition refused or changed. Prepare restoration again.');
 const proposal={recovery_id:record.id,inputs,capture};
 const prepared=await load<{version:number;restoration:{recovery_fingerprint:string;workspace:string;context:string;plan:ResourceWirePlan;saved_before:Record<string,ResourceSaved>;saved_after:Record<string,ResourceSaved>};consequences:ResourceConstructionFacts}>('/api/changes/prepare-restoration',proposal);
 if(!current()||prepared.version!==1||prepared.restoration.workspace!==record.workspace||prepared.restoration.context!==context||prepared.restoration.recovery_fingerprint!==record.fingerprint||!currentResourceFacts(prepared.consequences,record.workspace,context))throw new Error('Recovery evidence or source ownership changed. Read and prepare again.');
 const patches=resourceWirePatches(prepared.restoration.plan),restoring=new Map<string,Buffer>();
 for(const patch of patches)if(patch.after&&!patches.some(item=>item.before?.identity===patch.after!.identity)){const held=buffers.retainedResources.get(patch.after.identity);if(held)restoring.set(patch.after.identity,held);}
 const live=new Map<string,ResourceEditorCapture<Buffer>>();for(const patch of patches)if(patch.before){const buffer=buffers.values.get(patch.path);if(!buffer||buffer.text!==patch.before.text)throw new Error("Current resource source does not match the reviewed restoration. Preserve drafts and prepare again.");live.set(patch.before.identity,{editor:buffer,source:buffer.text});}
 routeResourceEditors(patches,live,new Map([...restoring].map(([identity,editor])=>[identity,{editor,source:editor.text}])));
 const editorCurrent=captureResourceEditors(buffers,patches,restoring);
 return {operation:'restore',facts:prepared.consequences,record:record.id,fingerprint:record.fingerprint,consequence:prepared.consequences.fingerprint,workspace:record.workspace,context,plan:prepared.restoration.plan,patches,saved:prepared.restoration.saved_after,savedBefore:prepared.restoration.saved_before,proposal,current:()=>current()&&editorCurrent(),restoring};
}
const resourceAttempts=new WeakMap<ResourceRestoreReview,string>();
/** Never retry a failed/uncertain write here. The caller requires a new review. */
export async function publishResourceRestore(buffers:Buffers,review:ResourceRestoreReview,load:ResourceTransport):Promise<{routes:ResourceEditorRoute<Buffer>[];recovery:string}>{
 const earlier=resourceAttempts.get(review);if(earlier)throw new Error('This review already has publication request '+earlier+'. Inspect its retained recovery; it will not be retried.');
 if(!review.current())throw new Error('Source changed after review. Prepare restoration again.');
 for(const patch of review.patches)if(patch.before){const buffer=buffers.values.get(patch.path),view=buffer&&editorForBuffer(buffer)?.view;if(!buffer||view&&(view.composing||authoredSource(view.state)!==buffer.text||view.state.readOnly||!view.state.facet(EditorView.editable)))throw new Error('Finish composition and restore current source before publication.');}
 const request=crypto.randomUUID().replaceAll('-','');resourceAttempts.set(review,request);
 let receipt:ResourceReceipt;try{receipt=await load<ResourceReceipt>(review.operation==='save'?'/api/changes/resource-save':'/api/changes/restore-resources',{request_id:request,reviewed_fingerprint:review.consequence,...(review.operation==='restore'?{reviewed_recovery_fingerprint:review.fingerprint}:{}),proposal:review.proposal});}catch(error){throw new Error(String(error)+' Publication request '+request+' is not confirmed. Inspect Source recovery using this identity before another review.');}
 let routes:ResourceEditorRoute<Buffer>[];try{routes=adoptResourceReceipt(buffers,review.patches,receipt,{request,workspace:review.workspace,context:review.context,saved:new Map(Object.entries(review.saved))},review.current,review.restoring);}catch(error){throw new Error(String(error)+" Publication request "+request+"; retained recovery "+(receipt.outcome?.recovery??"unknown")+". Preserve local drafts while inspecting the result.");}
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
/** A resource proposal is acquired as a whole; no text-only fallback may escape review. */
export async function prepareResourceSave(buffers:Buffers,title:string,edit:unknown,acquire:(path:string)=>Promise<Buffer>,load:ResourceTransport,alive:()=>boolean,hash:(text:string)=>Promise<string>=resourceHash):Promise<ResourceRestoreReview>{
 const context=crypto.randomUUID().replaceAll('-',''),targets=await load<{paths:string[]}>('/api/changes/resource-targets',{edit});
 if(!alive()||!targets.paths.length||targets.paths.length>1000||new Set(targets.paths).size!==targets.paths.length)throw new Error('Resource targets changed or exceed the review bound.');
 const initial=await captureResourceInputs(buffers,targets.paths,context,edit,load,alive,hash);
 for(const path of targets.paths)if(initial.inputs.sources[path]?.kind==='file'){const buffer=await acquire(path);if(buffer.text!==initial.inputs.sources[path].text)throw new Error('Referenced source changed during acquisition. Prepare the provider action again.');}
 const captured=await captureResourceInputs(buffers,targets.paths,context,edit,load,alive,hash);
 for(const path of targets.paths){const before=initial.inputs.sources[path],after=captured.inputs.sources[path];if(before?.kind!==after?.kind||before?.text!==after?.text||JSON.stringify(initial.inputs.saved_sources[path])!==JSON.stringify(captured.inputs.saved_sources[path])||initial.inputs.saved_guards[path]!==captured.inputs.saved_guards[path])throw new Error('Saved or draft resource input changed during acquisition. Prepare the whole action again.');}
 if(JSON.stringify(initial.inputs.saved_guards)!==JSON.stringify(captured.inputs.saved_guards))throw Error('Physical construction dependencies changed during acquisition. Prepare the whole action again.');
 const plan=await load<{plan:ResourceWirePlan}>('/api/changes/plan-resources',{context,title,edit,sources:captured.inputs.sources});
 if(!captured.current())throw new Error('Source changed while planning resource operations.');
 const proposal={inputs:captured.inputs,capture:captured.capture,plan:plan.plan},facts=await load<ResourceConstructionFacts&{saved_sources:Record<string,ResourceSaved>}>('/api/changes/resource-consequences',proposal);
 if(!captured.current()||!currentResourceFacts(facts,captured.inputs.workspace,context)||!facts.saved_sources)throw new Error('Resource ownership changed or saved consequences are unavailable. Prepare again.');
 const patches=resourceWirePatches(plan.plan),editorCurrent=captureResourceEditors(buffers,patches);
 if(patches.some(patch=>patch.before&&buffers.values.get(patch.path)?.text!==patch.before.text))throw new Error('Acquire every reviewed resource source before publication.');
 return {operation:'save',facts,record:'',fingerprint:'',consequence:facts.fingerprint,workspace:captured.inputs.workspace,context,plan:plan.plan,patches,saved:facts.saved_sources,savedBefore:captured.inputs.saved_sources,proposal,current:()=>captured.current()&&editorCurrent(),restoring:new Map()};
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
export async function captureResourceInputs(buffers:Buffers,targetPaths:string[],context:string,edit:unknown,load:ResourceTransport,alive:()=>boolean,hash:(text:string)=>Promise<string>){
 const paths=new Set(targetPaths),roster=captureResourceRoster(buffers),current=()=>alive()&&roster.current();
 const drafts:Record<string,unknown>=Object.create(null);for(const row of roster.captures){if(!paths.has(row.path)||buffers.values.get(row.path)!==row.buffer||row.deleted)continue;if(row.readonly||row.transient||row.buffer.saving)throw new Error('Choose writable current resource source.');if(!row.guard)throw Error('The loaded source has no physical identity guard. Preserve its draft and reopen or compare the exact saved source before preparing resource changes.');drafts[row.path]={save_guard:row.guard,text:row.text,base_sha256:await hash(row.base),version:row.version,writable:true};}
 if(!current())throw new Error('Source changed while capturing resource inputs.');
 const saved_guards:Record<string,string>=Object.create(null);for(const row of roster.captures)if(Object.hasOwn(roster.ownership_sources,row.path)&&row.guard)saved_guards[row.path]=row.guard;
 const capture={context,edit,drafts,ownership_sources:roster.ownership_sources,unaddressed_sources:roster.unaddressed_sources,saved_guards};
 const inputs=await load<ResourceInputs>('/api/changes/resource-inputs',capture);
 if(!current()||!inputs||inputs.context!==context||typeof inputs.workspace!=='string'||!inputs.workspace||inputs.source_files_modified!==false||!inputs.sources||typeof inputs.sources!=='object'||Array.isArray(inputs.sources)||!inputs.saved_sources||typeof inputs.saved_sources!=='object'||Array.isArray(inputs.saved_sources)||!inputs.refusals||typeof inputs.refusals!=='object'||Array.isArray(inputs.refusals)||!inputs.saved_guards||typeof inputs.saved_guards!=='object'||Array.isArray(inputs.saved_guards)||Object.entries(inputs.saved_guards).some(([path,guard])=>!path||typeof guard!=='string'||!guard||guard.length>16384)||Object.entries(inputs.sources).some(([path,source])=>source.kind==='file'&&!Object.hasOwn(inputs.saved_guards,path))||Object.keys(inputs.refusals).length)throw new Error('Current source acquisition refused or changed. Prepare again.');
 return {inputs,capture,current};
}
