import {Buffer,type Buffers} from './buffers';
import type {FileNode} from '../api/types';
import {editorForBuffer} from './Document';
import {EditorView} from '@codemirror/view';
import {isolateHistory} from '@codemirror/commands';
import type {Transaction} from '@codemirror/state';
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
export interface ResourceReceipt {version:number;request_id:string;workspace:string;context:string;outcome:{complete:boolean;recovery:string;applied:string[]};saved_sources:Record<string,ResourceSaved>|null;source_files:{file:FileNode;saved:ResourceSaved;hash:string}[]}
export interface ResourceExpectation {request:string;workspace:string;context:string;saved:ReadonlyMap<string,ResourceSaved>}
function sameSaved(a:ResourceSaved|undefined,b:ResourceSaved|undefined):boolean {
 if(!a||!b||a.kind!==b.kind)return false;
 return a.kind==='missing'||b.kind==='file'&&a.text===b.text&&a.sha256===b.sha256&&a.readonly===b.readonly&&a.executable===b.executable&&a.mode===b.mode;
}
export function confirmedResourceFiles(patches:readonly ResourceEditorPatch[],receipt:ResourceReceipt,expected:ResourceExpectation):Map<string,{file:FileNode;saved:Extract<ResourceSaved,{kind:'file'}>;hash:string}> {
 const refuse=()=>{throw new Error('Resource publication is not completely confirmed for this review. Inspect Source recovery before changing editors.');};
 if(receipt.version!==1||receipt.request_id!==expected.request||receipt.workspace!==expected.workspace||receipt.context!==expected.context||!receipt.outcome?.complete||!receipt.saved_sources||!receipt.outcome.recovery)refuse();
 const paths=new Set(patches.map(p=>p.path));
 if(paths.size!==patches.length||paths.size!==expected.saved.size||Object.keys(receipt.saved_sources!).length!==paths.size||receipt.outcome.applied.length!==paths.size||new Set(receipt.outcome.applied).size!==paths.size||receipt.outcome.applied.some(path=>!paths.has(path)))refuse();
 const files=new Map<string,{file:FileNode;saved:Extract<ResourceSaved,{kind:'file'}>;hash:string}>();
 for(const patch of patches){const saved=receipt.saved_sources![patch.path];if(!sameSaved(saved,expected.saved.get(patch.path))||(saved?.kind==='file')!==(patch.after!==null))refuse();}
 for(const entry of receipt.source_files){const saved=receipt.saved_sources![entry.file.path];if(!paths.has(entry.file.path)||files.has(entry.file.path)||entry.file.dir||entry.file.generated_by||saved?.kind!=='file'||!sameSaved(saved,entry.saved)||typeof entry.hash!=='string')refuse();else files.set(entry.file.path,{file:entry.file,saved,hash:entry.hash});}
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
  const buffer=route.editor??new Buffer(entry!.file,route.after!,entry!.hash,()=>buffers.changed());
  const view=editorForBuffer(buffer)?.view;
  if(view&&(view.composing||view.state.doc.toString()!==buffer.text||view.state.readOnly||!view.state.facet(EditorView.editable)))throw new Error('Finish composition and inspect recovery before adopting source editors.');
  if(route.to!==null&&!patches.some(p=>p.path===route.to&&p.before)&&buffers.values.has(route.to))throw new Error('A destination editor appeared after review. Preserve it and inspect recovery.');
  const transaction=view&&route.after!==null&&buffer.text!==route.after?view.state.update({changes:{from:0,to:view.state.doc.length,insert:route.after},annotations:isolateHistory.of('full'),userEvent:'input.workspace',filter:false}):undefined;
  if(transaction&&transaction.newDoc.toString()!==route.after)throw new Error('Editor refused the complete resource change. Inspect recovery.');
  prepared.push({route,buffer,view,transaction});
 }
 // All failures above precede mutation. Stable buffer objects keep cached renderers.
 buffers.retirePendingReads();
 for(const {route} of prepared)if(route.from!==null)buffers.values.delete(route.from);
 for(const {route,buffer,view,transaction} of prepared){
  if(route.to===null){buffer.resourceDeleted=true;retained.set(route.identity,buffer);continue;}
  const entry=files.get(route.to)!;
  if(view&&transaction)view.update([transaction]);
  buffer.file=entry.file;buffer.text=route.after!;buffer.base=entry.saved.text;buffer.hash=entry.hash;buffer.resourceDeleted=false;buffer.error='';
  buffers.values.set(route.to,buffer);for(const [id,held] of retained)if(held===buffer)retained.delete(id);
 }
 for(const {buffer} of prepared)buffer.publish();buffers.changed();return routes;
}
