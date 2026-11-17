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
import {authoredSource,authoredOffset} from './SourceText';
export interface TemplateEditor {buffer:Buffer;buffers:Buffers;view:EditorView;mounted:()=>boolean;interaction:()=>number}
export interface TemplateContext {version:1;path:string;source:string;selection:{from:number;to:number}}
export interface TemplateSnapshot {editor:TemplateEditor;view:EditorView;context:Readonly<TemplateContext>;selection:EditorView['state']['selection'];current:(writable?:boolean)=>boolean}
export function captureTemplateEditor(editor:TemplateEditor):TemplateSnapshot {
 const {buffer,buffers,view}=editor,file=buffer.file,path=file.path,namespace=buffers.workspace,revision=buffer.sourceRevision,doc=view.state.doc,selection=view.state.selection,interactionNow=editor.interaction,mounted=editor.mounted,interaction=interactionNow(),snippet=readSourceSnippet(view),source=authoredSource(view.state);
 const context=Object.freeze({version:1 as const,path,source,selection:Object.freeze({from:authoredOffset(view.state,selection.main.from),to:authoredOffset(view.state,selection.main.to)})});
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
 const acceptance=readCompletionAcceptance(view),applied=acceptance?.job!==before?.job&&acceptance?.phase==='applied'&&acceptance.applied&&authoredSource(view.state)===after;
 if(!applied)throw Error(message);if(current()&&snapshot.editor.mounted())view.focus();return true;
}
