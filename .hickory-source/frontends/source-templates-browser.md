<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Preview a reusable template in the retained source editor

For maintainers connecting the shared template catalog to an existing browser
editor. The catalog and plan are read-only provider observations. Applying a
selected plan is a separate, explicit draft action, guarded by the original
Buffer, namespace, editor mount, source and selection. No template contains a
command or authorizes saving a file.

## Capture an editor lease independently of provider reads

Retain the actual owner and editor, rather than looking up a pathname after an
await. Selection, scroll interaction, composition, source edits and resource
routing retire the choice. The pure read validator is separate from Apply
eligibility. In the current app the shared command requires writable source,
and the engine refuses generated input; the verified read-only path is a
disabled command with a provider refusal.

<ui:copy id="source-templates-browser-capture">
import {EditorView} from '@codemirror/view';
import type {Buffer,Buffers} from './buffers';
import {request} from '../api/client';
import {documentUri} from '../lsp/documentUri';
import {utf16ToPosition} from '../lsp/positions';
import type {CompletionItem} from '../lsp/client';
import {readCompletionAcceptance} from '../lsp/completionAcceptance';
import {planCompletion,type CompletionDraft} from './CompletionPlan';
import {completionExpansion} from './SnippetExpansion';
import {acceptSourceCompletion} from './SourceCompletion';
import {readSourceSnippet,continueSourceSnippetCompletion} from './SourceSnippets';
export interface TemplateEditor {buffer:Buffer;buffers:Buffers;view:EditorView;mounted:()=>boolean;interaction:()=>number}
export interface TemplateContext {version:1;path:string;source:string;selection:{from:number;to:number}}
export interface TemplateSnapshot {editor:TemplateEditor;view:EditorView;context:Readonly<TemplateContext>;selection:EditorView['state']['selection'];current:(writable?:boolean)=>boolean}
export function captureTemplateEditor(editor:TemplateEditor):TemplateSnapshot {
 const {buffer,buffers,view}=editor,file=buffer.file,path=file.path,namespace=buffers.workspace,revision=buffer.sourceRevision,doc=view.state.doc,selection=view.state.selection,interactionNow=editor.interaction,mounted=editor.mounted,interaction=interactionNow(),snippet=readSourceSnippet(view),source=doc.toString();
 const context=Object.freeze({version:1 as const,path,source,selection:Object.freeze({from:selection.main.from,to:selection.main.to})});
 const current=(writable=false)=>!!namespace&&buffers.workspace===namespace&&buffer.namespace===namespace&&buffers.values.get(path)===buffer&&buffer.file===file&&file.path===path&&buffer.sourceRevision===revision&&buffer.text===source&&mounted()&&view.dom.isConnected&&view.state.doc===doc&&view.state.selection.eq(selection)&&interactionNow()===interaction&&readSourceSnippet(view)===snippet&&(!writable||!buffer.readonly&&!buffer.saving&&!buffer.resourceDeleted&&!view.composing&&!view.state.readOnly&&view.state.facet(EditorView.editable)&&selection.ranges.length===1);
 if(!current())throw Error('The original source editor is unavailable. Reopen templates from its current editor.');
 return Object.freeze({editor:Object.freeze({...editor}),view,context,selection,current});
}
export async function templateSourceHash(source:string):Promise<string>{return [...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(source)))].map(value=>value.toString(16).padStart(2,'0')).join('');}
export interface TemplateEntry {id:string;label:string;description:string;language:string;origin:'builtin'|'workspace';available:boolean;reason?:string}
export interface TemplateCatalog {version:1;template_protocol:1;language:string;workspace_id:string;path:string;source_hash:string;selection:{from:number;to:number};available:boolean;reason?:string;notices:string[];catalog_fingerprint:string;entries:readonly TemplateEntry[]}
export interface TemplateProvider {catalog:(context:TemplateContext)=>Promise<unknown>;plan:(context:TemplateContext&{catalog_fingerprint:string;template_id:string})=>Promise<unknown>}
export const engineTemplateProvider:TemplateProvider={catalog:context=>request('POST','/api/templates/catalog',context),plan:context=>request('POST','/api/templates/plan',context)};
const exactSelection=(value:unknown,selection:TemplateContext['selection'])=>!!value&&typeof value==='object'&&!Array.isArray(value)&&Object.keys(value).length===2&&(value as {from?:unknown}).from===selection.from&&(value as {to?:unknown}).to===selection.to;
const strings=(value:unknown):value is string[]=>Array.isArray(value)&&value.length<=1024&&value.every(item=>typeof item==='string'&&item.length<=65536);
function inputIdentity(value:unknown,snapshot:TemplateSnapshot,hash:string,workspace?:string):Record<string,unknown>{
 if(!value||typeof value!=='object'||Array.isArray(value))throw Error('Malformed template provider response.');const row=value as Record<string,unknown>;
 if(row.version!==1||row.template_protocol!==1)throw Error('Template provider protocol 1 is unavailable. Update the engine before applying templates.');
 if(typeof row.workspace_id!=='string'||!row.workspace_id||workspace!==undefined&&row.workspace_id!==workspace||row.path!==snapshot.context.path||row.source_hash!==hash||!exactSelection(row.selection,snapshot.context.selection))throw Error('Template source or workspace identity changed. Refresh templates.');return row;
}
export async function readTemplateCatalog(snapshot:TemplateSnapshot,provider:TemplateProvider):Promise<Readonly<TemplateCatalog>> {
 const hash=await templateSourceHash(snapshot.context.source);if(!snapshot.current())throw Error('Source or editor interaction changed. Refresh templates.');
 const row=inputIdentity(await provider.catalog(snapshot.context),snapshot,hash);if(!snapshot.current())throw Error('Source or editor interaction changed. Refresh templates.');
 if(typeof row.language!=='string'||!row.language||typeof row.available!=='boolean'||!strings(row.notices)||typeof row.catalog_fingerprint!=='string'||!row.catalog_fingerprint||!Array.isArray(row.entries)||row.entries.length>1024)throw Error('Malformed template catalog. No draft changes were made.');
 const ids=new Set<string>();let size=0;const entries:TemplateEntry[]=row.entries.map(raw=>{if(!raw||typeof raw!=='object')throw Error('Malformed template entry.');const item=raw as TemplateEntry;if(!['id','label','description','language'].every(key=>typeof (item as unknown as Record<string,unknown>)[key]==='string')||!item.id||!item.label||!item.language||typeof item.available!=='boolean'||!['builtin','workspace'].includes(item.origin)||ids.has(item.id))throw Error('Malformed or duplicate template entry.');ids.add(item.id);size+=item.id.length+item.label.length+item.description.length+item.language.length;if(size>1048576)throw Error('Template catalog is too large.');return Object.freeze({id:item.id,label:item.label,description:item.description,language:item.language,origin:item.origin,available:item.available,reason:typeof item.reason==='string'?item.reason:undefined});});
 return Object.freeze({version:1,template_protocol:1,language:row.language,workspace_id:row.workspace_id as string,path:snapshot.context.path,source_hash:hash,selection:snapshot.context.selection,available:row.available,reason:typeof row.reason==='string'?row.reason:undefined,notices:Object.freeze([...row.notices]) as unknown as string[],catalog_fingerprint:row.catalog_fingerprint,entries:Object.freeze(entries)});
}
</ui:copy>

## Validate the selected plan before reusing snippet acceptance

A plan echoes the catalog and exact source selection. Construct a fresh data-only
completion from its canonical expansion; provider commands and additional edits
are never copied. The existing completion planner checks UTF-16 ranges and field
ancestry. Applying then reuses the editor's snippet transactions, mirrors, nesting
and undo history. An obsolete read produces no partial insertion.

<ui:copy id="source-templates-browser-apply">
function freezeTemplateData<T>(value:T):T{const pending:object[]=[];if(value&&typeof value==='object')pending.push(value);const seen=new WeakSet<object>();while(pending.length){const next=pending.pop()!;if(seen.has(next))continue;seen.add(next);for(const child of Object.values(next))if(child&&typeof child==='object')pending.push(child);Object.freeze(next);}return value;}
export interface TemplatePreview {provider:TemplateProvider;snapshot:TemplateSnapshot;catalog:Readonly<TemplateCatalog>;entry:TemplateEntry;item:CompletionItem;draft:CompletionDraft;text:string;fields:number;notices:readonly string[]}
export async function prepareTemplate(snapshot:TemplateSnapshot,catalog:Readonly<TemplateCatalog>,id:string,provider:TemplateProvider):Promise<TemplatePreview>{
 if(!catalog.available)throw Error(catalog.reason||'Templates are unavailable for this source.');if(!snapshot.current(true))throw Error('Template source is read-only, saving, composing or changed. No draft changes were made.');
 const entry=catalog.entries.find(item=>item.id===id);if(!entry)throw Error('Template entry changed. Refresh the catalog.');if(!entry.available)throw Error(entry.reason||'This template is unavailable for the selected source.');
 const row=inputIdentity(await provider.plan({...snapshot.context,catalog_fingerprint:catalog.catalog_fingerprint,template_id:id}),snapshot,catalog.source_hash,catalog.workspace_id);
 if(!snapshot.current(true))throw Error('Source or editor interaction changed while reading the template. No draft changes were made.');
 if(!strings(row.notices))throw Error('Malformed template notices.');
 if(row.available!==true)throw Error(typeof row.reason==='string'?row.reason:'This template cannot be applied to the selected source.');
 if(row.catalog_fingerprint!==catalog.catalog_fingerprint||row.template_id!==id||row.language!==entry.language||row.origin!==entry.origin||!exactSelection(row.replacement,snapshot.context.selection))throw Error('Template catalog or replacement changed. Refresh its preview.');
 const snippet=row.snippet as {version?:unknown;syntax?:unknown;text?:unknown;fields?:unknown}|null;
 if(!snippet||typeof snippet.syntax!=='string')throw Error('Template expansion unavailable. No draft changes were made.');
 const from=snapshot.context.selection.from,to=snapshot.context.selection.to,item:CompletionItem={label:entry.label,insertTextFormat:2,textEdit:{range:{start:utf16ToPosition(snapshot.context.source,from),end:utf16ToPosition(snapshot.context.source,to)},newText:snippet.syntax},data:{hickory_snippet:snippet}};
 const decoded=completionExpansion(item)!,expansion={...decoded,fields:decoded.fields?.map(field=>({...field,choices:[...field.choices]}))};item.data={hickory_snippet:{version:1,syntax:snippet.syntax,text:expansion.text,fields:expansion.fields}};freezeTemplateData(item);const draft=freezeTemplateData(continueSourceSnippetCompletion(snapshot.view,planCompletion(documentUri(snapshot.context.path),item,{text:snapshot.context.source,writable:true},from,to,expansion)));
 return Object.freeze({provider,snapshot,catalog,entry,item,draft,text:expansion.text,notices:Object.freeze([...row.notices]),fields:new Set(expansion.fields?.filter(field=>field.index>0).map(field=>field.index)).size});
}
function sameTemplateData(a:unknown,b:unknown):boolean{const pairs:[unknown,unknown][]=[[a,b]];while(pairs.length){const [left,right]=pairs.pop()!;if(left===right)continue;if(!left||!right||typeof left!=='object'||typeof right!=='object'||Array.isArray(left)!==Array.isArray(right))return false;const keys=Object.keys(left);if(keys.length!==Object.keys(right).length)return false;for(const key of keys){if(!Object.prototype.hasOwnProperty.call(right,key))return false;pairs.push([(left as Record<string,unknown>)[key],(right as Record<string,unknown>)[key]]);}}return true;}
export async function applyTemplate(preview:TemplatePreview,current:()=>boolean,notice:(message:string)=>void):Promise<boolean>{
 const {snapshot}=preview,{view}=snapshot;if(!current()||!snapshot.current(true))throw Error('Template preview is obsolete or source is unavailable. Refresh before applying.');
 const refreshed=await prepareTemplate(snapshot,preview.catalog,preview.entry.id,preview.provider);if(!current()||!snapshot.current(true))throw Error('Template preview is obsolete. Refresh before applying.');if(!sameTemplateData(refreshed.item,preview.item)||!sameTemplateData(refreshed.notices,preview.notices))throw Error('The template provider changed the reviewed expansion. Refresh and review it again.');
 const before=readCompletionAcceptance(view),after=preview.draft.plan.patches[0]?.after??snapshot.context.source;let message='Template was not applied.';
 await acceptSourceCompletion(view,structuredClone(preview.item),snapshot.context.source,snapshot.selection,snapshot.context.selection.from,snapshot.context.selection.to,{uri:documentUri(snapshot.context.path),resolve:async item=>item,current:()=>current()&&snapshot.current(true),writable:()=>snapshot.current(true),notice:value=>{message=value;notice(value.replace(/^Completion/,'Template'));}});
 const acceptance=readCompletionAcceptance(view),applied=acceptance?.job!==before?.job&&acceptance?.phase==='applied'&&acceptance.applied&&view.state.doc.toString()===after;
 if(!applied)throw Error(message);if(current()&&snapshot.editor.mounted())view.focus();return true;
}
</ui:copy>

<ui:file path="apps/web/src/simple/SourceTemplates.ts"><ui:paste select="#source-templates-browser-capture"/><ui:paste select="#source-templates-browser-apply"/></ui:file>

## Keep richer interaction in a temporary floating view

Reading the catalog and choosing a preview never edits the source. Closing,
refreshing or switching the originating editor retires held requests. Apply is
explicit and rechecks the same lease and re-reads the exact reviewed provider plan before changing the draft; controls remain outside the code layout.
Availability and provider notices stay visible even when no template can apply.

<ui:copy id="source-templates-browser-view">
import {useEffect,useRef,useState,useSyncExternalStore} from 'react';
import {createPortal} from 'react-dom';
import {captureTemplateEditor,readTemplateCatalog,prepareTemplate,applyTemplate,engineTemplateProvider,type TemplateEditor,type TemplateSnapshot,type TemplateCatalog,type TemplatePreview,type TemplateProvider} from './SourceTemplates';
export function SourceTemplatesView({visible,close,editor,provider=engineTemplateProvider}:{visible:boolean;close:()=>void;editor:()=>TemplateEditor|null;provider?:TemplateProvider}){
 const [snapshot,setSnapshot]=useState<TemplateSnapshot|null>(null),[catalog,setCatalog]=useState<Readonly<TemplateCatalog>|null>(null),[preview,setPreview]=useState<TemplatePreview|null>(null),[query,setQuery]=useState(''),[notice,setNotice]=useState(''),[loading,setLoading]=useState(false);
 const ticket=useRef(0),shown=useRef(visible),latest=useRef(editor),providers=useRef(provider),catalogProvider=useRef<TemplateProvider|null>(null),panel=useRef<HTMLElement>(null);shown.current=visible;latest.current=editor;providers.current=provider;
 const [,paint]=useState(0);
 useSyncExternalStore(listener=>snapshot?snapshot.editor.buffer.subscribe(listener):()=>{},()=>snapshot?.editor.buffer.snapshot()??0);
 function sameEditor(held:TemplateSnapshot){const current=latest.current();return !!current&&current.buffer===held.editor.buffer&&current.buffers===held.editor.buffers&&current.view===held.view;}
 function dismiss(){ticket.current++;setLoading(false);close();}
 useEffect(()=>{if(!visible){ticket.current++;setLoading(false);setSnapshot(null);setCatalog(null);setPreview(null);return;}panel.current?.focus();void read();},[visible,provider]);
 useEffect(()=>()=>{ticket.current++;},[]);
 useEffect(()=>{if(!snapshot||!visible)return;const view=snapshot.view,update=()=>paint(value=>value+1);view.dom.addEventListener('keyup',update);view.dom.addEventListener('pointerup',update);view.scrollDOM.addEventListener('scroll',update);return()=>{view.dom.removeEventListener('keyup',update);view.dom.removeEventListener('pointerup',update);view.scrollDOM.removeEventListener('scroll',update);};},[snapshot,visible]);
 async function read(){const id=++ticket.current;setLoading(false);setCatalog(null);setPreview(null);setNotice('');
  try{const current=latest.current();if(!current)throw Error('The source editor is still attaching. Retry after it becomes available.');const held=captureTemplateEditor(current),origin=providers.current;setSnapshot(held);setLoading(true);const catalog=await readTemplateCatalog(held,origin);if(id!==ticket.current||!shown.current||!sameEditor(held)||providers.current!==origin)return;catalogProvider.current=origin;setCatalog(catalog);if(!catalog.available)setNotice(catalog.reason||'Templates are unavailable for this source.');}
  catch(error){if(id===ticket.current)setNotice(error instanceof Error?error.message:String(error));}finally{if(id===ticket.current)setLoading(false);}
 }
 async function choose(id:string){if(!snapshot||!catalog)return;const held=snapshot,read=++ticket.current;setLoading(true);setPreview(null);setNotice('');
  try{if(!shown.current||!sameEditor(held)||catalogProvider.current!==providers.current)throw Error('The selected editor or provider changed. Refresh templates.');const plan=await prepareTemplate(held,catalog,id,providers.current);if(read!==ticket.current||!shown.current||!sameEditor(held))return;setPreview(plan);}
  catch(error){if(read===ticket.current)setNotice(error instanceof Error?error.message:String(error));}finally{if(read===ticket.current)setLoading(false);}
 }
 async function apply(){if(!preview)return;const held=preview,id=++ticket.current;setLoading(true);setNotice('');
  try{await applyTemplate(held,()=>id===ticket.current&&shown.current&&sameEditor(held.snapshot)&&held.provider===providers.current,message=>{if(id===ticket.current)setNotice(message);});if(id===ticket.current&&shown.current)dismiss();}
  catch(error){if(id===ticket.current)setNotice(error instanceof Error?error.message:String(error));}finally{if(id===ticket.current)setLoading(false);}
 }
 if(!visible)return null;const current=!!snapshot&&sameEditor(snapshot)&&snapshot.current()&&(!catalog||catalogProvider.current===providers.current),writable=current&&snapshot!.current(true),entries=(catalog?.entries??[]).filter(entry=>`${entry.label} ${entry.description} ${entry.language}`.toLowerCase().includes(query.toLowerCase()));
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Source templates" className="simple-workspace-problems simple-source-templates" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();dismiss();}}}>
  <button aria-label="Close source templates" onClick={dismiss}>×</button><strong>Source templates</strong><p>{snapshot?.context.path??'No source editor selected'}</p>
  <p>Choose a reusable template, inspect its insertion, then apply it to this draft. Tab moves through linked fields. Files are saved separately.</p>
  <button disabled={loading} onClick={()=>{void read();}}>Read templates</button><label>Find template<input aria-label="Find template" value={query} onChange={event=>{setQuery(event.target.value);}}/></label>
  {!current&&snapshot&&<p role="status">The original source, selection or provider changed. Refresh before applying.</p>}{current&&!writable&&<p role="status">This source is read-only, saving, composing or has several carets. Its templates remain readable.</p>}
  {notice&&<p role="status">{notice}</p>}{loading&&<p role="status">Reading template provider…</p>}{catalog?.notices.map((message,index)=><p key={index}>{message}</p>)}
  {entries.slice(0,100).map(entry=><div key={entry.id}><button disabled={loading||!current||!writable||!catalog?.available||!entry.available} onClick={()=>{void choose(entry.id);}}>Preview {entry.label}</button><span> {entry.language} · {entry.origin}</span><p>{entry.description}{!entry.available?` · ${entry.reason??'Unavailable for this source'}`:''}</p></div>)}{entries.length>100&&<p>Narrow the search to read the remaining templates.</p>}
  {catalog&&entries.length===0&&<p>No templates matched. This does not establish that the language has no generation capabilities.</p>}
  {preview&&<section aria-label="Template preview"><strong>{preview.entry.label}</strong><p>{preview.fields} linked fields · existing mirrors update together</p><pre>{preview.text}</pre>{preview.notices.map((message,index)=><p key={index}>{message}</p>)}<button disabled={loading||!writable} onClick={()=>{void apply();}}>Apply template to draft</button></section>}
 </section>,document.body);
}
</ui:copy>
<ui:file path="apps/web/src/simple/SourceTemplatesView.tsx"><ui:paste select="#source-templates-browser-view"/></ui:file>


## Exercise source lifetimes and real field history

The policy tests use a real retained editor with the production snippet and
history extensions. Provider fixtures supply data only. They check a complete
insertion and mirrors through undo/redo, plus refusal after ownership, source,
selection, namespace, disposal, composition or permission changes. These are
focused editor tests; a catalog menu and actual shared engine still need their
own connected acceptance before this feature is complete.

<ui:copy id="source-templates-browser-witness">
import {afterEach,beforeAll,expect,it,vi} from 'vitest';
import {EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {history,undo,redo} from '@codemirror/commands';
import {Buffer,Buffers} from './buffers';
import {sourceSnippets,readSourceSnippet,setSourceSnippet,dispatchSourceSnippetTransactions} from './SourceSnippets';
import {beginSnippet} from './SnippetSession';
import {captureTemplateEditor,readTemplateCatalog,prepareTemplate,applyTemplate,templateSourceHash,engineTemplateProvider,type TemplateProvider} from './SourceTemplates';
beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});});
const cleanups:(()=>void)[]=[];afterEach(()=>{for(const close of cleanups.splice(0))close();vi.unstubAllGlobals();});
async function fixture(source='old',from=0,to=source.length){const buffers=new Buffers(()=>{});buffers.workspace='/root';const buffer=new Buffer({path:'a.py',name:'a.py',dir:false},source,'hash',()=>{},false,'/root');buffers.values.set('a.py',buffer);let mounted=true,interaction=0;
 const parent=document.body.appendChild(document.createElement('div')),view=new EditorView({parent,dispatchTransactions:dispatchSourceSnippetTransactions,state:EditorState.create({doc:buffer.text,selection:{anchor:from,head:to},extensions:[history(),sourceSnippets({writable:()=>!buffer.readonly&&!buffer.saving,notice:()=>{}}),EditorView.updateListener.of(update=>{if(update.docChanged)buffer.edit(update.state.doc.toString());})]})});
 cleanups.push(()=>{view.destroy();parent.remove();});const snapshot=captureTemplateEditor({buffer,buffers,view,mounted:()=>mounted,interaction:()=>interaction}),hash=await templateSourceHash(source),identity={version:1,template_protocol:1,language:'python',workspace_id:'workspace',path:'a.py',source_hash:hash,selection:{from,to}},entry={id:'assign',label:'Assignment',description:'Linked variable',language:'python',origin:'builtin',available:true},catalog={...identity,available:true,notices:[],catalog_fingerprint:'catalog',entries:[entry]},plan={...identity,available:true,notices:[],catalog_fingerprint:'catalog',template_id:'assign',origin:'builtin',replacement:{from,to},snippet:{version:1,syntax:'${1:name} = $1;$0',text:'name = name;',fields:[{id:0,parent:null,index:1,from:0,to:4,choices:[]},{id:1,parent:null,index:1,from:7,to:11,choices:[]},{id:2,parent:null,index:0,from:12,to:12,choices:[]}]}};
 const provider:TemplateProvider={catalog:vi.fn(async()=>catalog),plan:vi.fn(async()=>plan)};return {buffers,buffer,view,snapshot,provider,catalog,plan,dispose:()=>{mounted=false;},interact:()=>{interaction++;}};
}
it('previews without writes, then reuses linked fields and exact editor undo/redo',async()=>{const f=await fixture(),before=f.view.state.selection,catalog=await readTemplateCatalog(f.snapshot,f.provider),preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);expect(f.view.state.doc.toString()).toBe('old');expect(f.buffer.dirty).toBe(false);expect(preview.text).toBe('name = name;');expect(preview.fields).toBe(1);
 await expect(applyTemplate(preview,()=>true,()=>{})).resolves.toBe(true);expect(f.view.state.selection.main.from).toBe(0);expect(f.view.state.selection.main.to).toBe(4);expect(readSourceSnippet(f.view)?.active).toBe(1);expect(f.buffer.base).toBe('old');
 f.view.dispatch({changes:{from:0,to:4,insert:'updated'},selection:{anchor:7},userEvent:'input.type'});expect(f.view.state.doc.toString()).toBe('updated = updated;');expect(undo(f.view)).toBe(true);expect(f.view.state.doc.toString()).toBe('name = name;');expect(undo(f.view)).toBe(true);expect(f.view.state.doc.toString()).toBe('old');expect(f.view.state.selection.eq(before)).toBe(true);expect(redo(f.view)).toBe(true);expect(readSourceSnippet(f.view)?.active).toBe(1);expect(redo(f.view)).toBe(true);expect(f.view.state.doc.toString()).toBe('updated = updated;');
 await expect(applyTemplate(preview,()=>true,()=>{})).rejects.toThrow('obsolete');
});
it('keeps the reviewed insertion immutable and restores UTF16 replacement and selection',async()=>{const f=await fixture('😀old',2,5),catalog=await readTemplateCatalog(f.snapshot,f.provider),preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);expect(()=>{(preview.item.textEdit as {newText:string}).newText='unreviewed';}).toThrow();expect(()=>{preview.text='unreviewed';}).toThrow();await applyTemplate(preview,()=>true,()=>{});expect(f.view.state.doc.toString()).toBe('😀name = name;');expect(f.view.state.selection.main.from).toBe(2);expect(f.view.state.selection.main.to).toBe(6);expect(undo(f.view)).toBe(true);expect(f.view.state.doc.toString()).toBe('😀old');expect(f.view.state.selection.main.from).toBe(2);expect(f.view.state.selection.main.to).toBe(5);});
it('rejects close, edit, selection or saving while the explicit Apply awaits acceptance',async()=>{for(const mode of ['close','edit','selection','saving'] as const){const f=await fixture(),catalog=await readTemplateCatalog(f.snapshot,f.provider),preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);let current=true;const pending=applyTemplate(preview,()=>current,()=>{});if(mode==='close')current=false;if(mode==='edit')f.view.dispatch({changes:{from:0,to:3,insert:'new'}});if(mode==='selection')f.view.dispatch({selection:{anchor:1}});if(mode==='saving')f.buffer.saving=true;await expect(pending).rejects.toThrow();expect(f.view.state.doc.toString()).toBe(mode==='edit'?'new':'old');}});
it('revalidates the reviewed manifest and expansion immediately before local application',async()=>{for(const mode of ['revoked','changed'] as const){const f=await fixture(),catalog=await readTemplateCatalog(f.snapshot,f.provider),preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);if(mode==='revoked')vi.mocked(f.provider.plan).mockRejectedValueOnce(Error('Template catalog changed'));else vi.mocked(f.provider.plan).mockResolvedValueOnce({...f.plan,snippet:{version:1,syntax:'${1:other} = $1;$0',text:'other = other;',fields:[{id:0,parent:null,index:1,from:0,to:5,choices:[]},{id:1,parent:null,index:1,from:8,to:13,choices:[]},{id:2,parent:null,index:0,from:14,to:14,choices:[]}]}});await expect(applyTemplate(preview,()=>true,()=>{})).rejects.toThrow(/changed/);expect(f.provider.plan).toHaveBeenCalledTimes(2);expect(f.view.state.doc.toString()).toBe('old');expect(f.buffer.dirty).toBe(false);}});
it('previews and applies inside an existing linked scope with exact nested undo/redo',async()=>{const f=await fixture('old + old',0,3),fields=[{id:0,parent:null,index:1,from:0,to:3,choices:[]},{id:1,parent:null,index:1,from:6,to:9,choices:[]},{id:2,parent:null,index:0,from:9,to:9,choices:[]}];f.view.dispatch({effects:setSourceSnippet.of(beginSnippet('old + old',fields))});const snapshot=captureTemplateEditor({...f.snapshot.editor}),catalog=await readTemplateCatalog(snapshot,f.provider),preview=await prepareTemplate(snapshot,catalog,'assign',f.provider);expect(preview.draft.plan.patches[0].after).toBe('name = name; + name = name;');await applyTemplate(preview,()=>true,()=>{});expect(f.view.state.doc.toString()).toBe('name = name; + name = name;');expect(readSourceSnippet(f.view)?.scopes?.length).toBe(1);expect(undo(f.view)).toBe(true);expect(f.view.state.doc.toString()).toBe('old + old');expect(readSourceSnippet(f.view)?.scopes?.length??0).toBe(0);expect(redo(f.view)).toBe(true);expect(readSourceSnippet(f.view)?.scopes?.length).toBe(1);});
it('refuses duplicate keys and noncanonical numeric template protocols before provider admission',async()=>{const f=await fixture();for(const kind of ['catalog','plan'] as const){const catalog=await readTemplateCatalog(f.snapshot,f.provider),value=JSON.stringify(kind==='catalog'?f.catalog:f.plan);for(const raw of [value.replace('"version":1','"version":1,"version":1'),value.replace('"template_protocol":1','"template_protocol":1.0'),value.replace('"template_protocol":1','"template_protocol":1e0'),value.replace('"template_protocol":1',String.raw`"\u0074emplate_protocol":1.0`),value.replace('"from":0','"from":0,"from":0')]){vi.stubGlobal('fetch',vi.fn(async()=>new Response(raw,{status:200,headers:{'Content-Type':'application/json'}})));if(kind==='catalog')await expect(readTemplateCatalog(f.snapshot,engineTemplateProvider)).rejects.toThrow();else await expect(prepareTemplate(f.snapshot,catalog,'assign',engineTemplateProvider)).rejects.toThrow();expect(f.buffer.text).toBe('old');expect(f.buffer.dirty).toBe(false);}}});
it('checks excessive choice metadata before defensively copying a provider expansion',async()=>{const f=await fixture(),catalog=await readTemplateCatalog(f.snapshot,f.provider);vi.mocked(f.provider.plan).mockResolvedValueOnce({...f.plan,snippet:{...f.plan.snippet,fields:[{...f.plan.snippet.fields[0],choices:Array<string>(4097).fill('')},f.plan.snippet.fields[2]]}});const clone=vi.spyOn(globalThis,'structuredClone');try{await expect(prepareTemplate(f.snapshot,catalog,'assign',f.provider)).rejects.toThrow(/metadata/);expect(clone).not.toHaveBeenCalled();expect(f.buffer.text).toBe('old');}finally{clone.mockRestore();}});
it('rejects legacy, mismatched and duplicate catalog observations without requesting a plan',async()=>{for(const patch of [{version:true},{template_protocol:0},{template_protocol:true},{source_hash:'wrong'},{selection:{from:1,to:3}},{entries:[{id:'x',label:'x',description:'',language:'python',origin:'workspace',available:true},{id:'x',label:'x',description:'',language:'python',origin:'workspace',available:true}]}]){const f=await fixture();vi.mocked(f.provider.catalog).mockResolvedValue({...f.catalog,...patch});await expect(readTemplateCatalog(f.snapshot,f.provider)).rejects.toThrow();expect(f.provider.plan).not.toHaveBeenCalled();expect(f.buffer.text).toBe('old');}});
it('keeps provider unavailability readable and rejects unsupported or mismatched plans as a whole',async()=>{const f=await fixture();vi.mocked(f.provider.catalog).mockResolvedValue({...f.catalog,available:false,reason:'Provider unavailable'});const unavailable=await readTemplateCatalog(f.snapshot,f.provider);await expect(prepareTemplate(f.snapshot,unavailable,'assign',f.provider)).rejects.toThrow('Provider unavailable');expect(f.provider.plan).not.toHaveBeenCalled();
 vi.mocked(f.provider.catalog).mockResolvedValue(f.catalog);const catalog=await readTemplateCatalog(f.snapshot,f.provider);for(const patch of [{replacement:{from:0,to:2}},{catalog_fingerprint:'changed'},{workspace_id:'foreign'},{snippet:{version:1,syntax:'${1/name/x/}',text:'unsafe',fields:[]}}]){vi.mocked(f.provider.plan).mockResolvedValue({...f.plan,...patch});await expect(prepareTemplate(f.snapshot,catalog,'assign',f.provider)).rejects.toThrow();expect(f.buffer.text).toBe('old');}
});
it('retires held plans after source, selection, scroll, owner, namespace and mount changes',async()=>{for(const mode of ['text','selection','scroll','owner','path','namespace','mount'] as const){const f=await fixture(),catalog=await readTemplateCatalog(f.snapshot,f.provider);let release!:(value:unknown)=>void;vi.mocked(f.provider.plan).mockImplementationOnce(()=>new Promise(resolve=>{release=resolve;}));const pending=prepareTemplate(f.snapshot,catalog,'assign',f.provider);if(mode==='text')f.view.dispatch({changes:{from:0,to:3,insert:'new'}});if(mode==='selection')f.view.dispatch({selection:{anchor:1}});if(mode==='scroll')f.interact();if(mode==='owner')f.buffers.values.set('a.py',new Buffer({...f.buffer.file},'old','hash',()=>{},false,'/root'));if(mode==='path'){f.buffer.file.path='moved.py';f.buffers.values.delete('a.py');f.buffers.values.set('moved.py',f.buffer);}if(mode==='namespace')f.buffers.workspace='/other';if(mode==='mount')f.dispose();release(f.plan);await expect(pending).rejects.toThrow('changed');expect(f.view.state.doc.toString()).toBe(mode==='text'?'new':'old');}});
it('reads readonly catalogs but refuses generated, saving, deleted, composition and closed previews',async()=>{for(const mode of ['readonly','saving','deleted','composing','closed'] as const){const f=await fixture(),catalog=await readTemplateCatalog(f.snapshot,f.provider),preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);if(mode==='readonly')f.buffer.file={...f.buffer.file,generated_by:'entry.md'};if(mode==='saving')f.buffer.saving=true;if(mode==='deleted')f.buffer.resourceDeleted=true;if(mode==='composing')Object.defineProperty(f.view,'composing',{value:true,configurable:true});await expect(applyTemplate(preview,()=>mode!=='closed',()=>{})).rejects.toThrow();expect(f.buffer.text).toBe('old');expect(f.buffer.dirty).toBe(false);}});
</ui:copy>
<ui:file path="apps/web/src/simple/SourceTemplates.test.ts"><ui:paste select="#source-templates-browser-witness"/></ui:file>


## Check the floating view separately from insertion policy

These view witnesses hold provider reads and change the originating editor. The
policy witnesses above own real snippet behavior; these checks establish that a
preview is never an implicit Apply, closing a held request cannot apply it, and
unsupported/read-only choices retain a readable explanation.

<ui:copy id="source-templates-browser-view-witness">
import {useState} from 'react';
import {act,cleanup,fireEvent,render,screen,waitFor} from '@testing-library/react';
import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {Buffer,Buffers} from './buffers';
import {captureTemplateEditor,readTemplateCatalog,prepareTemplate,applyTemplate,engineTemplateProvider,type TemplateEditor,type TemplateSnapshot,type TemplateCatalog,type TemplatePreview} from './SourceTemplates';
import {SourceTemplatesView} from './SourceTemplatesView';
vi.mock('./SourceTemplates',()=>({captureTemplateEditor:vi.fn(),readTemplateCatalog:vi.fn(),prepareTemplate:vi.fn(),applyTemplate:vi.fn(),engineTemplateProvider:{catalog:vi.fn(),plan:vi.fn()}}));
afterEach(cleanup);beforeEach(()=>vi.resetAllMocks());
function fixture(options:{readonly?:boolean;available?:boolean}={}){const buffers=new Buffers(()=>{});buffers.workspace='/root';const buffer=new Buffer({path:'a.py',name:'a.py',dir:false},'old','hash',()=>{},false,'/root');buffers.values.set('a.py',buffer);const view={dom:document.createElement('div'),scrollDOM:document.createElement('div')} as unknown as TemplateEditor['view'];const editor:TemplateEditor={buffer,buffers,view,mounted:()=>true,interaction:()=>0};let active=editor;
 const snapshot={editor,view,context:{version:1,path:'a.py',source:'old',selection:{from:0,to:3}},selection:{},current:(writable=false)=>!writable||!options.readonly} as TemplateSnapshot;
 const catalog={version:1,template_protocol:1,language:'python',workspace_id:'workspace',path:'a.py',source_hash:'hash',selection:{from:0,to:3},available:options.available!==false,reason:'Provider unavailable',notices:[],catalog_fingerprint:'catalog',entries:[{id:'assign',label:'Assignment',description:'Linked variable',language:'python',origin:'builtin',available:true}]} as TemplateCatalog;
 const preview={snapshot,catalog,entry:catalog.entries[0],provider:engineTemplateProvider,item:{label:'Assignment'},draft:{path:'a.py',before:'old',plan:{title:'Template',patches:[]},insertionFrom:0,selection:{from:0,to:4},fields:[]},text:'name = name;',fields:1,notices:[]} as TemplatePreview;
 vi.mocked(captureTemplateEditor).mockReturnValue(snapshot);vi.mocked(readTemplateCatalog).mockResolvedValue(catalog);vi.mocked(prepareTemplate).mockResolvedValue(preview);vi.mocked(applyTemplate).mockImplementation(async(_plan,current)=>{if(!current())throw Error('Editor changed');return true;});
 const close=vi.fn();function Harness(){const [visible,setVisible]=useState(true),[,paint]=useState(0);return <><button onClick={()=>{active={...editor,buffer:new Buffer({...buffer.file},'old','hash',()=>{},false,'/root')};paint(value=>value+1);}}>Replace source owner</button><SourceTemplatesView visible={visible} editor={()=>active} close={()=>{close();setVisible(false);}}/></>;}render(<Harness/>);return {snapshot,catalog,preview,close};
}
it('reads and previews without writes until explicit Apply, then closes the same floating view',async()=>{const f=fixture();await screen.findByRole('button',{name:'Preview Assignment'});expect(applyTemplate).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('button',{name:'Preview Assignment'}));await screen.findByRole('button',{name:'Apply template to draft'});expect(screen.getByRole('region',{name:'Template preview'}).textContent).toContain('name = name;');expect(applyTemplate).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('button',{name:'Apply template to draft'}));await waitFor(()=>expect(f.close).toHaveBeenCalledOnce());expect(applyTemplate).toHaveBeenCalledOnce();});
it('dismisses a held preview without applying or displaying an obsolete reply',async()=>{fixture();await screen.findByRole('button',{name:'Preview Assignment'});let release!:(value:TemplatePreview)=>void;const held=vi.mocked(prepareTemplate).mock.results.length;let preview!:TemplatePreview;vi.mocked(prepareTemplate).mockImplementationOnce(async(...args)=>new Promise(resolve=>{release=resolve;preview={snapshot:args[0],catalog:args[1],entry:args[1].entries[0],provider:engineTemplateProvider,item:{label:'Assignment'},draft:{path:'a.py',before:'old',plan:{title:'Template',patches:[]},insertionFrom:0,selection:{from:0,to:4},fields:[]},text:'late',fields:1,notices:[]} as TemplatePreview;}));fireEvent.click(screen.getByRole('button',{name:'Preview Assignment'}));expect(vi.mocked(prepareTemplate).mock.calls.length).toBe(held+1);fireEvent.click(screen.getByRole('button',{name:'Close source templates'}));await act(async()=>release(preview));expect(screen.queryByRole('dialog',{name:'Source templates'})).toBeNull();expect(applyTemplate).not.toHaveBeenCalled();});
it('refuses an identical-text replacement owner before Apply',async()=>{fixture();await screen.findByRole('button',{name:'Preview Assignment'});fireEvent.click(screen.getByRole('button',{name:'Preview Assignment'}));await screen.findByRole('button',{name:'Apply template to draft'});fireEvent.click(screen.getByRole('button',{name:'Replace source owner'}));expect((screen.getByRole('button',{name:'Apply template to draft'}) as HTMLButtonElement).disabled).toBe(true);expect(screen.getByText(/original source, selection or provider changed/)).toBeTruthy();expect(applyTemplate).not.toHaveBeenCalled();});
it('keeps unavailable and readonly catalogs readable without exposing an Apply path',async()=>{for(const options of [{available:false},{readonly:true}]){fixture(options);await screen.findByText('Linked variable');expect((screen.getByRole('button',{name:'Preview Assignment'}) as HTMLButtonElement).disabled).toBe(true);expect(screen.queryByRole('button',{name:'Apply template to draft'})).toBeNull();expect(prepareTemplate).not.toHaveBeenCalled();cleanup();}});
</ui:copy>
<ui:file path="apps/web/src/simple/SourceTemplatesView.test.tsx"><ui:paste select="#source-templates-browser-view-witness"/></ui:file>


## Drive the catalog through the existing workspace menu

The connected witness owns a disposable workspace and workspace catalog. It
requires the actual engine and browser adapter: a menu label or successful model
test cannot satisfy this path. It verifies a selected-text variable, Unicode
fields, mirrors, traversal and exact undo/redo without saving. Later preview
staleness and a generated read-only source must refuse application.

<ui:copy id="source-templates-browser-flow">
"""Focused live-engine reusable source-template browser acceptance."""
import importlib.util
import json
import os
from pathlib import Path
import socket
import tempfile
import urllib.error
import urllib.request
ROOT=Path(__file__).resolve().parents[1]
def main():
    spec=importlib.util.spec_from_file_location("simple_fixture",ROOT/"scripts/test-simple-browser.py")
    fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-source-template-ui-") as temporary:
        root=Path(temporary)/"workspace";root.mkdir();(root/".hickory").mkdir()
        source="old";path="a.py";(root/path).write_text(source)
        (root/"generated.py").write_text("generated = True\n")
        (root/"generator.md").write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="generated.py">\ngenerated = True\n</h:file>\n</h:doc>\n')
        manifest={"version":1,"templates":[{"id":"workspace.linked","label":"Workspace linked","description":"Mirrors, Unicode and selected text","language":"python","syntax":"${1|first,second|} + $1\n${2:😀 value} ${TM_FILENAME} ${TM_SELECTED_TEXT}$0"},{"id":"workspace.unsupported","label":"Unsupported transform","description":"Explicit unsupported capability","language":"python","syntax":"${1/x/y/}"}]}
        (root/".hickory/templates.json").write_text(json.dumps(manifest,ensure_ascii=False))
        environment={**os.environ,"HICKORY_STATE_DIR":str(Path(temporary)/"state")};processes=[];driver=None
        try:
            with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
            ui=f"http://127.0.0.1:{port}"
            engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine)
            backend=fixture.origin(engine)
            vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
            driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html")
            driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
            before=driver.act("editor-position",path=path,anchor=0,head=3)
            driver.act("templates-open");preview=driver.act("templates-preview",label="Workspace linked")
            assert "first + first\n😀 value a.py old" in preview,preview
            assert driver.act("construction-source",path=path)==source
            assert driver.act("editor-size",path=path)==size and driver.act("editor-is-retained",path=path)
            driver.act("templates-hold-apply")
            altered=json.loads(json.dumps(manifest));altered["templates"][0]["syntax"]="revoked$0"
            (root/".hickory/templates.json").write_text(json.dumps(altered,ensure_ascii=False))
            refused=driver.act("templates-release-apply");assert refused["status"]==422 and "changed" in refused["body"].lower(),refused
            assert driver.act("construction-source",path=path)==source
            (root/".hickory/templates.json").write_text(json.dumps(manifest,ensure_ascii=False))
            driver.act("templates-read");driver.act("templates-preview",label="Workspace linked")
            state=driver.act("templates-apply",path=path);base="first + first\n😀 value a.py old"
            assert state=={"source":base,"anchor":0,"head":5},state
            driver.act("type-source-selection",path=path,text="longer");expanded="longer + longer\n😀 value a.py old"
            fixture.eventually(lambda:driver.act("construction-source",path=path),expanded)
            driver.act("template-next");selected=driver.act("construction-source",path=path,selection=True)
            at=len("longer + longer\n".encode("utf-16-le"))//2
            assert selected["anchor"]==at and selected["head"]==at+len("😀 value".encode("utf-16-le"))//2,selected
            driver.act("construction-history");fixture.eventually(lambda:driver.act("construction-source",path=path),base)
            driver.act("construction-history");restored=driver.act("construction-source",path=path,selection=True)
            assert restored=={"source":source,"anchor":before["anchor"],"head":before["head"]},restored
            driver.act("construction-history",redo=True);driver.act("construction-history",redo=True)
            fixture.eventually(lambda:driver.act("construction-source",path=path),expanded)
            driver.act("template-choices");driver.act("completion-pick",title="second")
            fixture.eventually(lambda:driver.act("construction-source",path=path),"second + second\n😀 value a.py old")
            driver.act("template-finish")
            driver.act("templates-open",via="search");driver.act("templates-preview",label="Workspace linked")
            driver.act("source-tests-edit",path=path,text="\n# intervening draft\n")
            assert not driver.act("templates-apply-enabled"),"Obsolete preview remained enabled"
            changed=driver.act("construction-source",path=path);driver.act("templates-close")
            assert driver.act("construction-source",path=path)==changed
            assert driver.act("editor-size",path=path)==size and driver.act("editor-is-retained",path=path)
            assert (root/path).read_text()==source,"Templates unexpectedly saved the draft"
            driver.act("open",path="generated.py");fixture.eventually(lambda:driver.act("editor-write-state",path="generated.py"),{"readOnly":True,"editable":False})
            refused=driver.act("templates-command-state");assert refused["disabled"] and "editable" in refused["reason"],refused
            request=urllib.request.Request(backend+"/api/templates/catalog",data=json.dumps({"version":1,"path":"generated.py","source":"generated = True\n","selection":{"from":0,"to":0}}).encode(),headers={"Content-Type":"application/json"})
            try:urllib.request.urlopen(request,timeout=15);raise AssertionError("Generated template source was admitted")
            except urllib.error.HTTPError as error:assert error.code==422 and "generated" in error.read().decode().lower(),error
            assert not driver.act("templates-apply-enabled")
            assert (root/"generated.py").read_text()=="generated = True\n"
            assert not driver.act("debug-diagnostics")["errors"]
            print("PASS source templates UI: workspace catalog, explicit preview/Apply, selected-text/Unicode fields, mirrors/choices/traversal, exact undo/redo, revoked-manifest/stale/readonly refusal, retained editor/geometry/draft and unchanged disk")
        finally:
            if driver:driver.close()
            for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
</ui:copy>
<ui:file path="scripts/test-source-templates-browser.py"><ui:paste select="#source-templates-browser-flow"/></ui:file>

## Focused verification receipt

The final browser snapshot passes TypeScript checking and 145 focused tests in
12 affected files, including the shared raw protocol codec, existing saved-test
Debug paths, source completion/session/history, template policy and floating
view. The template checks use actual CodeMirror fields and undo/redo, not only
provider mocks. Empty alternatives are bounded before choice copying; nested
proposals reserve their complete mirrored metadata; the history cache charges
fixed snapshot and alternative overhead and caps both bytes and token count.
Small injected budgets exercise eviction without large allocations.

The maintained `scripts/test-source-templates-browser.py` passed against the
fresh bounded engine in Chromium. It opens the existing Edit command and action
search, reads a workspace catalog, previews without edits, and holds the fresh
Apply-plan request while revoking the manifest. The engine refuses that old
fingerprint with no draft change. After an explicit new read and Apply, selected
text and filename variables, Unicode fields, mirrors, choice selection, numeric
traversal and exact source/selection undo/redo pass. A later source edit disables
the old preview. A genuinely generated source has a read-only rendered editor,
disabled command and independent provider refusal. Editor identity, dimensions,
unsaved text and saved disk bytes remain intact, with no page errors.

The saved-test Debug physical flow also passed after sharing the raw codec,
including a user breakpoint, locals/watch, retained session link, Stop and a
second filtered process completion without its sibling. These receipts do not
establish human hardware input, assistive-technology acceptance, other browser
engines, full transforms, multiple-caret templates or mirrored native IME.
Templates remain data-only draft edits; they do not establish semantic generation
correctness or authorize saving or executing code.

</ui:doc>
