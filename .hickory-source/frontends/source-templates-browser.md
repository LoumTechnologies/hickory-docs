<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Preview a reusable template in the retained source editor

For maintainers connecting the shared template catalog to an existing browser
editor. The catalog and plan are read-only provider observations. Applying a
selected plan is a separate, explicit draft action, guarded by the original
source owner, namespace, editor mount, source and selection. No template contains a
command or authorizes saving a file.

## Capture an editor lease independently of provider reads

Retain the actual owner and editor, rather than looking up a pathname after an
await. Selection, scroll interaction, composition, source edits and resource
routing retire the choice. The pure read validator is separate from Apply
eligibility. In the current app the shared command requires writable source,
and the engine refuses generated input; the verified read-only path is a
disabled command with a provider refusal.

Main captures an authored tab together with its frozen compiler contributors.
The panel can read a held catalog while only the original writer can apply it.
Both the single and grouped plans send complete native editor transactions to
the guarded private draft path after the explicit Apply and fresh provider read.

A template snapshot captures the exact authored source and converts the visible selection into authored UTF-16 offsets. Preview and Apply share that lease; normalized display lines never replace the bytes used for provider identity or retained history. The common completion authority performs the raw edit and maps its final linked selection back to the display.

A multiple-selection reading captures the complete ordered authored roster and primary selection. If a live template group has coincident logical carets, read its immutable peer selections instead of the widget's merged display selection. Retain that exact group and editor intent through every asynchronous read. The shared engine prepares all rows against one compiler graph; the adapter never splits one choice into independent provider requests.

Capture the immutable supported program list beside the source lease, then require the same echo and reject each unnegotiated field before reader conversion. Whole-batch raw metadata is metered before a v3 reader can adopt another VM; replay support alone never widens a proposal. The generated focused checks and actual negotiated program paths below have passed.

<ui:copy id="source-templates-browser-capture">
import {EditorView} from '@codemirror/view';
import {EditorState} from '@codemirror/state';
import type {Buffer,Buffers} from './buffers';
import {request} from '../api/client';
import {documentUri} from '../lsp/documentUri';
import {utf16ToPosition} from '../lsp/positions';
import type {CompletionItem} from '../lsp/client';
import {readCompletionAcceptance} from '../lsp/completionAcceptance';
import {planCompletion,type CompletionDraft} from './CompletionPlan';
import {completionExpansion} from './SnippetExpansion';
import {parallelSelections,planParallelSnippet,type ParallelSnippet,type ParallelExpansion,type ParallelSnippetChange} from './ParallelSnippet';
import {supportedSnippetTransformPrograms,type SnippetTransformBudget} from './SnippetTransform';
import {captureSnippetVariables,captureSnippetClipboard,type SnippetVariableSnapshot} from './SnippetContext';
import {readFrozenEntrySelection} from './WorkspaceEntrySelection';
import {acceptSourceCompletion,type SourceCompletionAdapter} from './SourceCompletion';
import {readSourceSnippet,continueSourceSnippetCompletion,readSourceEditorIntent,readParallelSourceSnippet,applyParallelSourceSnippet} from './SourceSnippets';
import {authoredSource,authoredOffset} from './SourceText';
export interface SimpleTemplateEditor {buffer:Buffer;buffers:Buffers;view:EditorView;mounted:()=>boolean;interaction:()=>number;entrySelection?:()=>Promise<TemplateEntrySelection>}
export interface HostedTemplateEditor {
 id:string;view:EditorView;mounted:()=>boolean;
 capture:()=>{path:string;workspaceRoot:string;contributors:TemplateReadingContext;current:(writable?:boolean)=>boolean;entrySelection:()=>Promise<TemplateEntrySelection>;commit:NonNullable<SourceCompletionAdapter['commit']>};
}
export type TemplateEditor=SimpleTemplateEditor|HostedTemplateEditor;
export interface TemplateContext {version:1|2;path:string;source:string;selection:{from:number;to:number};variable_context?:SnippetVariableSnapshot;reading_context?:TemplateReadingContext;transform_program_versions?:readonly number[];carets?:Readonly<TemplateCaretRoster>}
export interface TemplateSnapshot {editor:TemplateEditor & {entrySelection?:()=>Promise<TemplateEntrySelection>};view:EditorView;context:Readonly<TemplateContext>;selection:EditorView['state']['selection'];parallel?:ParallelSnippet|null;intent?:symbol;workspaceRoot?:string;commit?:NonNullable<SourceCompletionAdapter['commit']>;current:(writable?:boolean)=>boolean}
import {captureResourceRoster} from './ResourceEditors';
export interface TemplateEntrySelection {entryPath:string|null;proof:unknown;current:()=>boolean}
export interface TemplateReadingContext {version:1;project_entry:string|null;project_endpoint?:string;sources:Readonly<Record<string,string>>;unaddressed_sources:Readonly<Record<string,string>>;missing_sources:readonly string[];entry_selection?:unknown}
export interface TemplateSourceContext {version:1;scope:string;language:string|null;languages:readonly string[];targets:readonly string[];reason:string|null;project_entry:string|null;source_hashes:Readonly<Record<string,string>>;graph_fingerprint:string;fingerprint:string}
export interface TemplateCatalogReading {snapshot:TemplateSnapshot;catalog:Readonly<TemplateCatalog>}
export function captureTemplateContributors(editor:SimpleTemplateEditor):{context:TemplateReadingContext;current:()=>boolean}{
 const roster=captureResourceRoster(editor.buffers),sources:Record<string,string>=Object.create(null),missing=new Set<string>();let bytes=0;
 for(const row of roster.captures){if(row.deleted){if(row.path)missing.add(row.path);continue}if(row.transient||row.untitled||editor.buffers.values.get(row.path)!==row.buffer)continue;if(row.namespace&&row.namespace!==editor.buffers.workspace)throw Error('Template source context belongs to another workspace.');if(Object.hasOwn(sources,row.path))throw Error('Template contributor addresses are ambiguous.');const size=new TextEncoder().encode(row.text).length;if(size>8*1024*1024||(bytes+=size)>32*1024*1024)throw Error('Template contributor capture exceeds its interactive budget.');sources[row.path]=row.text;}
 for(const path of Object.keys(sources))missing.delete(path);if(Object.keys(sources).length>4096||missing.size>4096)throw Error('Too many template contributors.');
 return {context:Object.freeze({version:1,project_entry:null,sources:Object.freeze(sources),unaddressed_sources:Object.freeze({...roster.unaddressed_sources}),missing_sources:Object.freeze([...missing].sort())}),current:()=>roster.current()};
}
export interface TemplateCaretRoster {version:1;selections:readonly {from:number;to:number}[];primary:number;continuing:boolean}
export type TemplateLanguageFacts=Omit<TemplateSourceContext,'source_hashes'>;
export interface TemplateCaretReading {selection:{from:number;to:number};context:Readonly<TemplateLanguageFacts>}
function capturedTemplateCarets(view:EditorView,source:string,parallel:ParallelSnippet|null):Readonly<TemplateCaretRoster>|undefined {
 const selected=view.state.selection,logical=parallel?parallelSelections(parallel):{ranges:selected.ranges.map(range=>({from:authoredOffset(view.state,range.from),to:authoredOffset(view.state,range.to)})),primary:selected.mainIndex};
 if(parallel&&parallel.source!==source)throw Error('The original multiple template fields no longer match this source.');
 if(logical.ranges.length<2){if(parallel)throw Error('The original template group has no complete selection roster.');return undefined;}
 if(logical.ranges.length>256||logical.primary<0||logical.primary>=logical.ranges.length)throw Error('Choose at most 256 source selections before reading templates.');
 const current=logical.ranges[logical.primary],primary=selected.main;if(current.from!==authoredOffset(view.state,primary.from)||current.to!==authoredOffset(view.state,primary.to))throw Error('The primary template selection no longer matches the native editor.');
 return freezeTemplateData({version:1 as const,selections:logical.ranges.map(range=>({from:range.from,to:range.to})),primary:logical.primary,continuing:parallel!==null});
}
function languageFacts(value:unknown,snapshot:TemplateSnapshot,includeHashes:boolean):{row:Record<string,unknown>;facts:TemplateLanguageFacts;bytes:number} {
 if(!value||typeof value!=='object'||Array.isArray(value))throw Error('Source-language context is unavailable. Update the engine before applying embedded templates.');const row=value as Record<string,unknown>;
 const keys=['version','scope','language','languages','targets','reason','project_entry','graph_fingerprint','fingerprint',...(includeHashes?['source_hashes']:[])];if(Object.keys(row).length!==keys.length||keys.some(key=>!Object.hasOwn(row,key)))throw Error('Malformed source-language context fields.');
 if(row.version!==1||typeof row.scope!=='string'||!(typeof row.language==='string'||row.language===null)||!strings(row.languages)||!strings(row.targets)||!(typeof row.reason==='string'||row.reason===null)||!(typeof row.project_entry==='string'||row.project_entry===null)||typeof row.fingerprint!=='string'||!(/^[0-9a-f]{64}$/).test(row.fingerprint)||typeof row.graph_fingerprint!=='string'||!(/^[0-9a-f]{64}$/).test(row.graph_fingerprint))throw Error('Malformed source-language context.');
 if(row.project_entry!==snapshot.context.reading_context?.project_entry&&!(snapshot.context.reading_context?.project_entry===null&&row.project_entry===snapshot.context.path))throw Error('The construction entry changed while reading templates.');
 if(row.language!==null&&(row.languages.length!==1||row.languages[0]!==row.language))throw Error('Ambiguous source-language identity.');
 let bytes=128;const encoder=new TextEncoder(),bound=(text:string)=>{bytes+=encoder.encode(text).length;if(bytes>1048576)throw Error('Source-language information is too large to preview safely.');};bound(row.scope);if(typeof row.reason==='string')bound(row.reason);if(typeof row.project_entry==='string')bound(row.project_entry);for(const text of[...row.languages,...row.targets])bound(text);
 const facts:TemplateLanguageFacts={version:1,scope:row.scope,language:row.language as string|null,languages:row.languages,targets:row.targets,reason:row.reason as string|null,project_entry:row.project_entry as string|null,graph_fingerprint:row.graph_fingerprint,fingerprint:row.fingerprint};return {row,facts,bytes};
}
function copyLanguageFacts(facts:TemplateLanguageFacts):TemplateLanguageFacts{return {...facts,languages:[...facts.languages],targets:[...facts.targets]};}
function sourceContext(value:unknown,snapshot:TemplateSnapshot,hash:string):Readonly<TemplateSourceContext>{
 const {row,facts,bytes:initial}=languageFacts(value,snapshot,true);if(!row.source_hashes||typeof row.source_hashes!=='object'||Array.isArray(row.source_hashes))throw Error('Missing source-language hashes.');const hashes=row.source_hashes as Record<string,unknown>;
 if(!Object.hasOwn(hashes,snapshot.context.path)||Object.keys(hashes).length>10000||Object.values(hashes).some(value=>typeof value!=='string'||!(/^[0-9a-f]{64}$/).test(value))||hashes[snapshot.context.path]!==hash)throw Error('The language context does not retain the original source.');let bytes=initial;const encoder=new TextEncoder();for(const[path,value]of Object.entries(hashes)){bytes+=encoder.encode(path).length+(value as string).length;if(bytes>1048576)throw Error('Source-language information is too large to preview safely.');}
 return freezeTemplateData({...copyLanguageFacts(facts),source_hashes:Object.fromEntries(Object.entries(hashes)) as Record<string,string>});
}
function caretReadings(value:unknown,snapshot:TemplateSnapshot,binding:Readonly<TemplateSourceContext>):readonly TemplateCaretReading[]{
 const carets=snapshot.context.carets;if(!carets||!Array.isArray(value)||value.length!==carets.selections.length)throw Error('The complete template caret reading is unavailable.');let bytes=0;
 const observed=value.map((item,index)=>{if(!item||typeof item!=='object'||Array.isArray(item)||Object.keys(item).length!==2||!Object.hasOwn(item,'selection')||!Object.hasOwn(item,'context'))throw Error('Malformed template caret reading.');const row=item as {selection:unknown;context:unknown};if(!exactSelection(row.selection,carets.selections[index]))throw Error('A template caret reading belongs to another selection.');const {facts,bytes:cost}=languageFacts(row.context,snapshot,false);bytes+=cost;if(bytes>1048576)throw Error('The complete caret language information is too large to preview safely.');if(facts.graph_fingerprint!==binding.graph_fingerprint||facts.project_entry!==binding.project_entry||facts.targets.some(target=>!binding.targets.includes(target))||binding.language!==null&&facts.language!==binding.language)throw Error('A caret belongs to another captured compiler context.');return facts;});
 return Object.freeze(observed.map((facts,index)=>freezeTemplateData({selection:{...carets.selections[index]},context:copyLanguageFacts(facts)})));
}
function templateCaretIdentity(row:Record<string,unknown>,snapshot:TemplateSnapshot):void {
 const carets=snapshot.context.carets;if(carets){if(row.template_batch_protocol!==1||!sameTemplateData(row.carets,carets))throw Error('The engine did not preserve every captured template selection.');}
 else if(Object.hasOwn(row,'template_batch_protocol')||Object.hasOwn(row,'carets')||Object.hasOwn(row,'expansions'))throw Error('Unexpected multiple-selection template proposal.');
}

async function resolveTemplateCatalog(snapshot:TemplateSnapshot,provider:TemplateProvider,alive:()=>boolean=()=>true):Promise<TemplateCatalogReading>{
 // The template protocol hashes the canonical root spelling. The separate
 // short Files workspace ID remains part of the construction-entry proof.
 const [hash,workspace]=await Promise.all([templateSourceHash(snapshot.context.source),snapshot.workspaceRoot===undefined?undefined:templateSourceHash(snapshot.workspaceRoot)]);for(let attempt=0;attempt<2;attempt++){
  if(!alive()||!snapshot.current())throw Error('Source or contributor context changed. Refresh templates.');const raw=await provider.catalog(snapshot.context);const row=inputIdentity(raw,snapshot,hash,workspace);if(!alive()||!snapshot.current())throw Error('Source or contributor context changed. Refresh templates.');
  if(!Object.prototype.hasOwnProperty.call(row,'reading_context_request')){const catalog=validateCatalogRow(row,snapshot,hash);return Object.freeze({snapshot,catalog})}
  const requirement=row.reading_context_request as {version?:unknown;kind?:unknown}|null;if(attempt!==0||!requirement||typeof requirement!=='object'||Array.isArray(requirement)||Object.keys(requirement).length!==2||requirement.version!==1||requirement.kind!=='construction_entry'||row.available!==false||!Array.isArray(row.entries)||row.entries.length!==0||snapshot.context.reading_context?.entry_selection!==undefined)throw Error('Unsupported or repeated construction context request.');
  const read=snapshot.editor.entrySelection;if(!read)throw Error("The construction entry reader is unavailable. Source remains editable.");const entry=await read();if(!alive()||!snapshot.current()||!entry.current())throw Error('Original construction entry or contributors changed while reading templates.');const original=snapshot;
  snapshot=Object.freeze({...original,context:Object.freeze({...original.context,reading_context:Object.freeze({...original.context.reading_context!,project_entry:entry.entryPath,entry_selection:freezeTemplateData(structuredClone(entry.proof))})}),current:(writable=false)=>original.current(writable)&&entry.current()});
 }
 throw Error('A current construction context is unavailable. Source remains editable.');
}

export function captureTemplateEditor(editor:TemplateEditor):TemplateSnapshot {
 if('capture' in editor){
  const captured=editor.capture(),{view}=editor,{doc,selection}=view.state,source=authoredSource(view.state),intent=readSourceEditorIntent(view.state),snippet=readSourceSnippet(view),parallel=readParallelSourceSnippet(view.state);
  const carets=capturedTemplateCarets(view,source,parallel);
  const current=(writable=false)=>editor.mounted()&&view.dom.isConnected&&view.state.doc===doc&&authoredSource(view.state)===source&&view.state.selection.eq(selection)&&readSourceEditorIntent(view.state)===intent&&readSourceSnippet(view)===snippet&&readParallelSourceSnippet(view.state)===parallel&&captured.current(writable)&&(!writable||!view.composing&&!view.state.readOnly&&view.state.facet(EditorView.editable)&&(carets?!!intent&&view.state.facet(EditorState.allowMultipleSelections):selection.ranges.length===1));
  if(!current())throw Error('The original source editor is unavailable. Reopen templates from its current editor.');
  const context=Object.freeze({version:2 as const,path:captured.path,source,selection:Object.freeze({from:authoredOffset(view.state,selection.main.from),to:authoredOffset(view.state,selection.main.to)}),variable_context:captureSnippetVariables(),reading_context:captured.contributors,transform_program_versions:Object.freeze([...supportedSnippetTransformPrograms]),...(carets?{carets}:{})});
  return Object.freeze({editor:Object.freeze({...editor,entrySelection:captured.entrySelection}),view,context,selection,parallel,intent,current,workspaceRoot:captured.workspaceRoot,commit:captured.commit});
 }
 const {buffer,buffers,view}=editor,file=buffer.file,path=file.path,namespace=buffers.workspace,revision=buffer.sourceRevision,doc=view.state.doc,selection=view.state.selection,interactionNow=editor.interaction,mounted=editor.mounted,interaction=interactionNow(),snippet=readSourceSnippet(view),source=authoredSource(view.state),intent=readSourceEditorIntent(view.state),parallel=readParallelSourceSnippet(view.state),carets=capturedTemplateCarets(view,source,parallel),contributors=captureTemplateContributors(editor);
 const context=Object.freeze({version:2 as const,path,source,selection:Object.freeze({from:authoredOffset(view.state,selection.main.from),to:authoredOffset(view.state,selection.main.to)}),variable_context:captureSnippetVariables(),reading_context:contributors.context,transform_program_versions:Object.freeze([...supportedSnippetTransformPrograms]),...(carets?{carets}:{})});
 const current=(writable=false)=>contributors.current()&&!!namespace&&buffers.workspace===namespace&&buffer.namespace===namespace&&buffers.values.get(path)===buffer&&buffer.file===file&&file.path===path&&buffer.sourceRevision===revision&&buffer.text===source&&mounted()&&view.dom.isConnected&&view.state.doc===doc&&view.state.selection.eq(selection)&&interactionNow()===interaction&&readSourceSnippet(view)===snippet&&readParallelSourceSnippet(view.state)===parallel&&readSourceEditorIntent(view.state)===intent&&(!writable||!buffer.readonly&&!buffer.saving&&!buffer.resourceDeleted&&!view.composing&&!view.state.readOnly&&view.state.facet(EditorView.editable)&&(carets?!!intent&&view.state.facet(EditorState.allowMultipleSelections):selection.ranges.length===1));
 if(!current())throw Error('The original source editor is unavailable. Reopen templates from its current editor.');
 const entrySelection=editor.entrySelection??(()=>readFrozenEntrySelection(()=>request('GET','/api/workspace/entry'),{workspace:buffers.workspaceId,namespace:namespace!,current:()=>current()}));
 return Object.freeze({editor:Object.freeze({...editor,entrySelection}),view,context,selection,parallel,intent,current});
}
export async function templateSourceHash(source:string):Promise<string>{return [...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(source)))].map(value=>value.toString(16).padStart(2,'0')).join('');}
export interface TemplateEntry {id:string;label:string;description:string;language:string;origin:'builtin'|'workspace';available:boolean;reason?:string}
export interface TemplateCatalog {version:1;template_protocol:1|2;language:string;workspace_id:string;path:string;source_hash:string;selection:{from:number;to:number};available:boolean;reason?:string;notices:string[];catalog_fingerprint:string;entries:readonly TemplateEntry[];source_context?:Readonly<TemplateSourceContext>;readings?:readonly TemplateCaretReading[]}
export interface TemplateProvider {catalog:(context:TemplateContext)=>Promise<unknown>;plan:(context:TemplateContext&{catalog_fingerprint:string;template_id:string})=>Promise<unknown>}
export const engineTemplateProvider:TemplateProvider={catalog:context=>request('POST','/api/templates/catalog',context),plan:context=>request('POST','/api/templates/plan',context)};
const exactSelection=(value:unknown,selection:TemplateContext['selection'])=>!!value&&typeof value==='object'&&!Array.isArray(value)&&Object.keys(value).length===2&&(value as {from?:unknown}).from===selection.from&&(value as {to?:unknown}).to===selection.to;
const strings=(value:unknown):value is string[]=>Array.isArray(value)&&value.length<=1024&&value.every(item=>typeof item==='string'&&item.length<=65536);
function inputIdentity(value:unknown,snapshot:TemplateSnapshot,hash:string,workspace?:string):Record<string,unknown>{
 if(!value||typeof value!=='object'||Array.isArray(value))throw Error('Malformed template provider response.');const row=value as Record<string,unknown>;templateCaretIdentity(row,snapshot);if(['command','additionalTextEdits','workspaceEdit','edit'].some(key=>Object.prototype.hasOwnProperty.call(row,key)))throw Error('Template response includes unsupported additional effects. No edits were applied.');
 if(row.version!==1||row.template_protocol!==snapshot.context.version)throw Error('Template provider protocol is unavailable. Update the engine before applying templates.');if(snapshot.context.version===2&&(!snapshot.context.variable_context||row.source_files_modified!==false||!sameTemplateData(row.variable_context,snapshot.context.variable_context)))throw Error('Template variable context changed. Refresh before applying.');
 if(snapshot.context.transform_program_versions&&!sameTemplateData(row.transform_program_versions,snapshot.context.transform_program_versions))throw Error('Template transform consumer versions changed.');if(snapshot.context.reading_context){const binding=sourceContext(row.source_context,snapshot,hash);if(row.language!==(binding.language??'unknown'))throw Error('Template catalog and source language disagree.');}
 if(typeof row.workspace_id!=='string'||!row.workspace_id||workspace!==undefined&&row.workspace_id!==workspace||row.path!==snapshot.context.path||row.source_hash!==hash||!exactSelection(row.selection,snapshot.context.selection))throw Error('Template source or workspace identity changed. Refresh templates.');return row;
}
function validateCatalogRow(row:Record<string,unknown>,snapshot:TemplateSnapshot,hash:string):Readonly<TemplateCatalog>{
 if(typeof row.language!=='string'||!row.language||typeof row.available!=='boolean'||!strings(row.notices)||typeof row.catalog_fingerprint!=='string'||!row.catalog_fingerprint||!Array.isArray(row.entries)||row.entries.length>1024)throw Error('Malformed template catalog. No draft changes were made.');
 const ids=new Set<string>();let size=0;const entries:TemplateEntry[]=row.entries.map(raw=>{if(!raw||typeof raw!=='object')throw Error('Malformed template entry.');const item=raw as TemplateEntry;if(!['id','label','description','language'].every(key=>typeof (item as unknown as Record<string,unknown>)[key]==='string')||!item.id||!item.label||!item.language||item.language!==row.language||typeof item.available!=='boolean'||!['builtin','workspace'].includes(item.origin)||ids.has(item.id))throw Error('Malformed or duplicate template entry.');ids.add(item.id);size+=item.id.length+item.label.length+item.description.length+item.language.length;if(size>1048576)throw Error('Template catalog is too large.');return Object.freeze({id:item.id,label:item.label,description:item.description,language:item.language,origin:item.origin,available:item.available,reason:typeof item.reason==='string'?item.reason:undefined});});
 const binding=snapshot.context.reading_context||snapshot.context.carets?sourceContext(row.source_context,snapshot,hash):undefined;const readings=snapshot.context.carets?caretReadings(row.readings,snapshot,binding!):undefined;return Object.freeze({version:1,template_protocol:snapshot.context.version,language:row.language,workspace_id:row.workspace_id as string,path:snapshot.context.path,source_hash:hash,selection:snapshot.context.selection,available:row.available,reason:typeof row.reason==='string'?row.reason:undefined,notices:Object.freeze([...row.notices]) as unknown as string[],catalog_fingerprint:row.catalog_fingerprint,entries:Object.freeze(entries),...(binding?{source_context:binding}:{}),...(readings?{readings}:{})});
}
export async function readTemplateCatalog(snapshot:TemplateSnapshot,provider:TemplateProvider,alive:()=>boolean=()=>true):Promise<TemplateCatalogReading>{return resolveTemplateCatalog(snapshot,provider,alive);}

</ui:copy>

## Validate the selected plan before reusing snippet acceptance

A plan echoes the catalog and exact source selection. Construct a fresh data-only
completion from its canonical expansion; provider commands and additional edits
are never copied. The existing completion planner checks UTF-16 ranges and field
ancestry. Applying then reuses the editor's snippet transactions, mirrors, nesting
and undo history. An obsolete read produces no partial insertion.

The preview model distinguishes one insertion from a complete caret group. Preflight aggregate text, field and metadata capacity before defensively copying any row; share transform validation work across all rows. Use the pure nested parallel planner before showing the preview, retain its complete source and typed edits, and require fresh reviewed equality before the existing one-transaction source reflection. A partial or foreign roster, unsupported continuation or retired intent refuses the whole action. The focused cases below passed on the generated snapshot; the separate actual flows verify the adapters rather than treating these model checks as UI evidence.

<ui:copy id="source-templates-browser-apply">
function freezeTemplateData<T>(value:T):T{const pending:object[]=[];if(value&&typeof value==='object')pending.push(value);const seen=new WeakSet<object>();while(pending.length){const next=pending.pop()!;if(seen.has(next))continue;seen.add(next);for(const child of Object.values(next))if(child&&typeof child==='object')pending.push(child);Object.freeze(next);}return value;}
interface TemplatePreviewBase {provider:TemplateProvider;snapshot:TemplateSnapshot;catalog:Readonly<TemplateCatalog>;entry:TemplateEntry;text:string;fields:number;notices:readonly string[]}
export interface SingleTemplatePreview extends TemplatePreviewBase {kind:'single';item:CompletionItem;draft:CompletionDraft}
export interface BatchTemplatePreview extends TemplatePreviewBase {kind:'batch';items:readonly CompletionItem[];expansions:readonly ParallelExpansion[];plan:Readonly<ParallelSnippetChange>}
export type TemplatePreview=SingleTemplatePreview|BatchTemplatePreview;
interface TemplateMetadataMeter {bytes:number;nodes:number}
function templateMetadataCharge(meter:TemplateMetadataMeter,bytes:number):void {
 if(!Number.isSafeInteger(bytes)||bytes<0||bytes>1048576-meter.bytes)throw Error('The complete template metadata is too large to preview safely.');meter.bytes+=bytes;
}
function templateProgramJSONCost(value:unknown,meter:TemplateMetadataMeter,depth=0):void {
 if(++meter.nodes>1048576||depth>256)throw Error('The complete template program exceeds its interactive metadata budget.');
 const string=(text:string)=>{templateMetadataCharge(meter,2);for(let at=0;at<text.length;at++){const unit=text.charCodeAt(at);if(unit<32)templateMetadataCharge(meter,[8,9,10,12,13].includes(unit)?2:6);else if(unit===34||unit===92)templateMetadataCharge(meter,2);else if(unit<128)templateMetadataCharge(meter,1);else if(unit<2048)templateMetadataCharge(meter,2);else if(unit>=0xd800&&unit<=0xdbff){const low=text.charCodeAt(++at);if(!(low>=0xdc00&&low<=0xdfff))throw Error('Malformed template program text.');templateMetadataCharge(meter,4);}else{if(unit>=0xdc00&&unit<=0xdfff)throw Error('Malformed template program text.');templateMetadataCharge(meter,3);}}};
 if(value===null){templateMetadataCharge(meter,4);return;}
 if(typeof value==='string'){string(value);return;}
 if(typeof value==='boolean'){templateMetadataCharge(meter,value?4:5);return;}
 if(typeof value==='number'&&Number.isFinite(value)){templateMetadataCharge(meter,String(value).length);return;}
 if(Array.isArray(value)){if(value.length>1048576-meter.nodes)throw Error('The complete template program contains too many metadata values.');templateMetadataCharge(meter,2+Math.max(0,value.length-1));for(const item of value)templateProgramJSONCost(item,meter,depth+1);return;}
 if(value&&typeof value==='object'){const keys=Object.keys(value);if(keys.length>16)throw Error('Malformed template program fields.');templateMetadataCharge(meter,2+Math.max(0,keys.length-1));for(const key of keys){string(key);templateMetadataCharge(meter,1);templateProgramJSONCost((value as Record<string,unknown>)[key],meter,depth+1);}return;}
 throw Error('Malformed template program metadata.');
}
function templateBatchMetadataCost(fields:unknown[],meter:TemplateMetadataMeter):void {
 for(const raw of fields){if(!raw||typeof raw!=='object'||Array.isArray(raw))throw Error('Malformed template field metadata.');const field=raw as {choices?:unknown;transform?:unknown};if(!Array.isArray(field.choices)||field.choices.length>4096)throw Error('Malformed or excessive template choice metadata.');templateMetadataCharge(meter,64+32*field.choices.length);for(const choice of field.choices){if(typeof choice!=='string'||choice.length>1048576-meter.bytes)throw Error('The complete template choice metadata is too large to preview safely.');templateMetadataCharge(meter,choice.length?new TextEncoder().encode(choice).length:0);}if(field.transform!=null)templateProgramJSONCost(field.transform,meter);}
}

function templateTransformConsumers(fields:unknown,programs:readonly number[]):void {
 if(!Array.isArray(fields))throw Error('Malformed template fields.');for(const field of fields){if(!field||typeof field!=='object')throw Error('Malformed template field.');const transform=(field as {transform?:unknown}).transform;if(transform==null)continue;if(typeof transform!=='object'||Array.isArray(transform)||!programs.includes((transform as {version?:unknown}).version as number))throw Error('A template transform was not accepted by this source snapshot.');}
}
function batchTemplateExpansion(row:Record<string,unknown>,snapshot:TemplateSnapshot,entry:TemplateEntry):{items:readonly CompletionItem[];expansions:readonly ParallelExpansion[];plan:Readonly<ParallelSnippetChange>}{
 const carets=snapshot.context.carets;if(!carets||Object.hasOwn(row,'snippet')||Object.hasOwn(row,'replacement')||!Array.isArray(row.expansions)||row.expansions.length!==carets.selections.length)throw Error('The provider did not return one complete template expansion roster.');
 let textBytes=0,syntaxBytes=0,fieldCount=0;const metadata:TemplateMetadataMeter={bytes:0,nodes:0};const encoder=new TextEncoder(),programs=snapshot.context.transform_program_versions??[2];
 const observed=row.expansions.map((raw,index)=>{if(!raw||typeof raw!=='object'||Array.isArray(raw)||Object.keys(raw).length!==3||!Object.hasOwn(raw,'selection')||!Object.hasOwn(raw,'replacement')||!Object.hasOwn(raw,'snippet'))throw Error('Malformed template caret expansion.');const value=raw as Record<string,unknown>,selection=carets.selections[index];if(!exactSelection(value.selection,selection)||!exactSelection(value.replacement,selection))throw Error('A template expansion targets another original caret.');
  const snippet=value.snippet as {version?:unknown;syntax?:unknown;text?:unknown;fields?:unknown}|null;if(!snippet||![1,2].includes(snippet.version as number)||typeof snippet.syntax!=='string'||typeof snippet.text!=='string'||!Array.isArray(snippet.fields)||!snippet.fields.length||snippet.fields.length>4096)throw Error('Template expansion is unavailable.');
  if(snippet.syntax.length>262144||snippet.text.length>1048576-textBytes)throw Error('The complete template expansion is too large to preview.');syntaxBytes+=encoder.encode(snippet.syntax).length;textBytes+=encoder.encode(snippet.text).length;fieldCount+=snippet.fields.length;if(fieldCount>4096)throw Error('The complete template has too many fields to preview.');templateTransformConsumers(snippet.fields,programs);templateBatchMetadataCost(snippet.fields,metadata);if(textBytes>1048576||syntaxBytes>8*1048576||fieldCount>4096||metadata.bytes>1048576)throw Error('The complete template expansion is too large to preview.');return {selection,snippet:{version:snippet.version as 1|2,syntax:snippet.syntax,text:snippet.text,fields:snippet.fields}};});
 const budget:SnippetTransformBudget={work:0};const items:CompletionItem[]=[],expansions:ParallelExpansion[]=[];
 for(const {selection,snippet}of observed){const item:CompletionItem={label:entry.label,insertTextFormat:2,insertText:snippet.syntax,data:{hickory_snippet:snippet}};const decoded=completionExpansion(item,budget);if(!decoded?.fields)throw Error('A template caret has no validated field expansion.');const fields=decoded.fields.map(field=>({...field,choices:[...field.choices]}));item.data={hickory_snippet:{version:snippet.version,syntax:snippet.syntax,text:decoded.text,fields}};items.push(freezeTemplateData(item));expansions.push(freezeTemplateData({from:selection.from,to:selection.to,text:decoded.text,fields}));}
 const plan=planParallelSnippet(snapshot.context.source,expansions,carets.primary,snapshot.parallel??null);if(!plan)throw Error('The complete template group cannot continue these original selections. No code changed.');return freezeTemplateData({items,expansions,plan});
}

export async function prepareTemplate(snapshot:TemplateSnapshot,catalog:Readonly<TemplateCatalog>,id:string,provider:TemplateProvider,requestCurrent:()=>boolean=()=>true):Promise<TemplatePreview>{
 if(!catalog.available)throw Error(catalog.reason||'Templates are unavailable for this source.');if(!requestCurrent()||!snapshot.current(true))throw Error('Template source is read-only, saving, composing or changed. No draft changes were made.');
 const entry=catalog.entries.find(item=>item.id===id);if(!entry)throw Error('Template entry changed. Refresh the catalog.');if(!entry.available)throw Error(entry.reason||'This template is unavailable for the selected source.');
 let row:Record<string,unknown>|undefined;
 for(let attempt=0;attempt<2;attempt++){
  const value=inputIdentity(await provider.plan({...snapshot.context,catalog_fingerprint:catalog.catalog_fingerprint,template_id:id}),snapshot,catalog.source_hash,catalog.workspace_id);if((snapshot.context.reading_context||snapshot.context.carets)&&!sameTemplateData(value.source_context,catalog.source_context))throw Error('Captured compiler context changed. Refresh the catalog.');if(snapshot.context.carets&&!sameTemplateData(value.readings,catalog.readings))throw Error('The complete caret language reading changed. Refresh the catalog.');if(['command','additionalTextEdits','workspaceEdit','edit'].some(key=>Object.prototype.hasOwnProperty.call(value,key)))throw Error('Template contains unsupported additional effects. No edits were applied.');if(!requestCurrent()||!snapshot.current(true))throw Error('Source or editor interaction changed while reading the template. No draft changes were made.');
  if(!Object.prototype.hasOwnProperty.call(value,'context_request')){row=value;break;}
  const request=value.context_request as {version?:unknown;variables?:unknown}|null;if(attempt!==0||snapshot.context.version!==2||snapshot.context.variable_context?.clipboard!==null||!request||typeof request!=='object'||Array.isArray(request)||Object.keys(request).length!==2||request.version!==1||value.available!==false||!Array.isArray(request.variables)||request.variables.length!==1||request.variables[0]!=='CLIPBOARD'||value.catalog_fingerprint!==catalog.catalog_fingerprint||value.template_id!==id||value.language!==entry.language||value.origin!==entry.origin||(snapshot.context.carets?(Object.hasOwn(value,'replacement')||Object.hasOwn(value,'snippet')||Object.hasOwn(value,'expansions')):!exactSelection(value.replacement,snapshot.context.selection)))throw Error('Unsupported or foreign template context request. No edits were applied.');
  const clipboard=await captureSnippetClipboard();if(!requestCurrent()||!snapshot.current(true))throw Error('Source or selection changed while reading clipboard context. No edits were applied.');snapshot=Object.freeze({...snapshot,context:Object.freeze({...snapshot.context,variable_context:Object.freeze({...snapshot.context.variable_context!,clipboard})})});
 }
 if(!row)throw Error('Template context is unavailable. No draft changes were made.');
 if(!strings(row.notices))throw Error('Malformed template notices.');
 if(row.available!==true)throw Error(typeof row.reason==='string'?row.reason:'This template cannot be applied to the selected source.');
 if(row.catalog_fingerprint!==catalog.catalog_fingerprint||row.template_id!==id||row.language!==entry.language||row.origin!==entry.origin||(!snapshot.context.carets&&!exactSelection(row.replacement,snapshot.context.selection)))throw Error('Template catalog or replacement changed. Refresh its preview.');
 if(snapshot.context.carets){const batch=batchTemplateExpansion(row,snapshot,entry),fields=batch.expansions.reduce((sum,item)=>sum+new Set(item.fields.filter(field=>field.index>0&&!field.transform).map(field=>field.index)).size,0);return Object.freeze({kind:'batch' as const,provider,snapshot,catalog,entry,...batch,text:batch.expansions.map((item,index)=>`Selection ${index+1}\n${item.text}`).join('\n\n'),fields,notices:Object.freeze([...row.notices])});}
 const snippet=row.snippet as {version?:unknown;syntax?:unknown;text?:unknown;fields?:unknown}|null;
 if(!snippet||typeof snippet.syntax!=='string')throw Error('Template expansion unavailable. No draft changes were made.');templateTransformConsumers(snippet.fields,snapshot.context.transform_program_versions??[2]);
 const from=snapshot.context.selection.from,to=snapshot.context.selection.to,item:CompletionItem={label:entry.label,insertTextFormat:2,textEdit:{range:{start:utf16ToPosition(snapshot.context.source,from),end:utf16ToPosition(snapshot.context.source,to)},newText:snippet.syntax},data:{hickory_snippet:snippet}};
 const decoded=completionExpansion(item)!,expansion={...decoded,fields:decoded.fields?.map(field=>({...field,choices:[...field.choices]}))};item.data={hickory_snippet:{version:snippet.version,syntax:snippet.syntax,text:expansion.text,fields:expansion.fields}};freezeTemplateData(item);const draft=freezeTemplateData(continueSourceSnippetCompletion(snapshot.view,planCompletion(documentUri(snapshot.context.path),item,{text:snapshot.context.source,writable:true},from,to,expansion)));
 return Object.freeze({kind:'single' as const,provider,snapshot,catalog,entry,item,draft,text:expansion.text,notices:Object.freeze([...row.notices]),fields:new Set(expansion.fields?.filter(field=>field.index>0).map(field=>field.index)).size});
}
function sameTemplateData(a:unknown,b:unknown):boolean{const pairs:[unknown,unknown][]=[[a,b]];while(pairs.length){const [left,right]=pairs.pop()!;if(left===right)continue;if(!left||!right||typeof left!=='object'||typeof right!=='object'||Array.isArray(left)!==Array.isArray(right))return false;const keys=Object.keys(left);if(keys.length!==Object.keys(right).length)return false;for(const key of keys){if(!Object.prototype.hasOwnProperty.call(right,key))return false;pairs.push([(left as Record<string,unknown>)[key],(right as Record<string,unknown>)[key]]);}}return true;}
export async function applyTemplate(preview:TemplatePreview,current:()=>boolean,notice:(message:string)=>void):Promise<boolean>{
 const {snapshot}=preview,{view}=snapshot;if(!current()||!snapshot.current(true))throw Error('Template preview is obsolete or source is unavailable. Refresh before applying.');
 const refreshed=await prepareTemplate(snapshot,preview.catalog,preview.entry.id,preview.provider,current);if(!current()||!snapshot.current(true))throw Error('Template preview is obsolete. Refresh before applying.');if(refreshed.kind!==preview.kind||!sameTemplateData(refreshed.notices,preview.notices)||(refreshed.kind==='single'&&preview.kind==='single'&&!sameTemplateData(refreshed.item,preview.item))||(refreshed.kind==='batch'&&preview.kind==='batch'&&(!sameTemplateData(refreshed.items,preview.items)||!sameTemplateData(refreshed.plan,preview.plan))))throw Error('The template provider changed the reviewed expansion. Refresh and review it again.');
 if(preview.kind==='batch'){
  if(!snapshot.context.carets||!snapshot.intent||readParallelSourceSnippet(view.state)!==(snapshot.parallel??null))throw Error('The original template group changed. Refresh before applying.');
  const applied=applyParallelSourceSnippet(view,snapshot.context.source,preview.expansions,snapshot.context.carets.primary,snapshot.intent,()=>current()&&snapshot.current(true),snapshot.commit);if(!applied||authoredSource(view.state)!==preview.plan.state.source)throw Error('The editor refused the complete reviewed template group. No replacement request was sent.');notice('Template applied to all captured selections · unsaved draft');if(current()&&snapshot.editor.mounted())view.focus();return true;
 }
 const before=readCompletionAcceptance(view),after=preview.draft.plan.patches[0]?.after??snapshot.context.source;let message='Template was not applied.';
 await acceptSourceCompletion(view,structuredClone(preview.item),snapshot.context.source,snapshot.selection,snapshot.context.selection.from,snapshot.context.selection.to,{uri:documentUri(snapshot.context.path),resolve:async item=>item,current:()=>current()&&snapshot.current(true),writable:()=>snapshot.current(true),commit:snapshot.commit,notice:value=>{message=value;notice(value.replace(/^Completion/,'Template'));}});
 const acceptance=readCompletionAcceptance(view),applied=acceptance?.job!==before?.job&&acceptance?.phase==='applied'&&acceptance.applied&&authoredSource(view.state)===after;
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
export function SourceTemplatesView({visible,close,editor,provider=engineTemplateProvider,className='simple-workspace-problems simple-source-templates'}:{visible:boolean;close:()=>void;editor:()=>TemplateEditor|null;provider?:TemplateProvider;className?:string}){
 const [snapshot,setSnapshot]=useState<TemplateSnapshot|null>(null),[catalog,setCatalog]=useState<Readonly<TemplateCatalog>|null>(null),[preview,setPreview]=useState<TemplatePreview|null>(null),[query,setQuery]=useState(''),[notice,setNotice]=useState(''),[loading,setLoading]=useState(false);
 const ticket=useRef(0),shown=useRef(visible),latest=useRef(editor),providers=useRef(provider),catalogProvider=useRef<TemplateProvider|null>(null),panel=useRef<HTMLElement>(null);shown.current=visible;latest.current=editor;providers.current=provider;
 const [,paint]=useState(0);
 const owner=snapshot&&'buffer' in snapshot.editor?snapshot.editor.buffer:null;
 useSyncExternalStore(listener=>owner?.subscribe(listener)??(()=>{}),()=>owner?.snapshot()??0);
 function sameEditor(held:TemplateSnapshot){const current=latest.current();if(!current||current.view!==held.view)return false;const original=held.editor;return 'capture' in current?'capture' in original&&current.id===original.id:'buffer' in original&&current.buffer===original.buffer&&current.buffers===original.buffers;}
 function dismiss(){ticket.current++;setLoading(false);close();}
 useEffect(()=>{if(!visible){ticket.current++;setLoading(false);setSnapshot(null);setCatalog(null);setPreview(null);return;}panel.current?.focus();void read();},[visible,provider]);
 useEffect(()=>()=>{ticket.current++;},[]);
 useEffect(()=>{if(!snapshot||!visible)return;const view=snapshot.view,update=()=>paint(value=>value+1);view.dom.addEventListener('keyup',update);view.dom.addEventListener('pointerup',update);view.scrollDOM.addEventListener('scroll',update);return()=>{view.dom.removeEventListener('keyup',update);view.dom.removeEventListener('pointerup',update);view.scrollDOM.removeEventListener('scroll',update);};},[snapshot,visible]);
 async function read(){const id=++ticket.current;setLoading(false);setCatalog(null);setPreview(null);setNotice('');
  try{const current=latest.current();if(!current)throw Error('The source editor is still attaching. Retry after it becomes available.');const held=captureTemplateEditor(current),origin=providers.current;setSnapshot(held);setLoading(true);const reading=await readTemplateCatalog(held,origin,()=>id===ticket.current&&shown.current&&sameEditor(held));const {catalog,snapshot:resolved}=reading;if(id!==ticket.current||!shown.current||!sameEditor(held)||providers.current!==origin)return;catalogProvider.current=origin;setSnapshot(resolved);setCatalog(catalog);if(!catalog.available)setNotice(catalog.reason||'Templates are unavailable for this source.');}
  catch(error){if(id===ticket.current)setNotice(error instanceof Error?error.message:String(error));}finally{if(id===ticket.current)setLoading(false);}
 }
 async function choose(id:string){if(!snapshot||!catalog)return;const held=snapshot,read=++ticket.current;setLoading(true);setPreview(null);setNotice('');
  try{if(!shown.current||!sameEditor(held)||catalogProvider.current!==providers.current)throw Error('The selected editor or provider changed. Refresh templates.');const plan=await prepareTemplate(held,catalog,id,providers.current,()=>read===ticket.current&&shown.current&&sameEditor(held)&&catalogProvider.current===providers.current);if(read!==ticket.current||!shown.current||!sameEditor(held))return;setPreview(plan);}
  catch(error){if(read===ticket.current)setNotice(error instanceof Error?error.message:String(error));}finally{if(read===ticket.current)setLoading(false);}
 }
 async function apply(){if(!preview)return;const held=preview,id=++ticket.current;setLoading(true);setNotice('');
  try{await applyTemplate(held,()=>id===ticket.current&&shown.current&&sameEditor(held.snapshot)&&held.provider===providers.current,message=>{if(id===ticket.current)setNotice(message);});if(id===ticket.current&&shown.current)dismiss();}
  catch(error){if(id===ticket.current)setNotice(error instanceof Error?error.message:String(error));}finally{if(id===ticket.current)setLoading(false);}
 }
 if(!visible)return null;const current=!!snapshot&&sameEditor(snapshot)&&snapshot.current()&&(!catalog||catalogProvider.current===providers.current),writable=current&&snapshot!.current(true),entries=(catalog?.entries??[]).filter(entry=>`${entry.label} ${entry.description} ${entry.language}`.toLowerCase().includes(query.toLowerCase()));
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Source templates" className={className} onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();dismiss();}}}>
  <button aria-label="Close source templates" onClick={dismiss}>×</button><strong>Source templates</strong><p>{snapshot?.context.path??'No source editor selected'}</p>
  <p>Choose a reusable template, inspect its insertion, then apply it to this draft. Tab moves through linked fields. Files are saved separately.</p>
  <button disabled={loading} title={loading ? "Wait for the current reading to load." : undefined} onClick={()=>{void read();}}>Read templates</button><label>Find template<input aria-label="Find template" value={query} onChange={event=>{setQuery(event.target.value);}}/></label>
  {!current&&snapshot&&<p role="status">The original source, selection or provider changed. Refresh before applying.</p>}{current&&!writable&&<p role="status">This source is read-only, saving, composing or its selection controls are still attaching. Its templates remain readable.</p>}
  {notice&&<p role="status">{notice}</p>}{loading&&<p role="status">Reading template provider…</p>}{catalog?.notices.map((message,index)=><p key={index}>{message}</p>)}
  {entries.slice(0,100).map(entry=><div key={entry.id}><button disabled={loading||!current||!writable||!catalog?.available||!entry.available} title={loading ? "Wait for the current reading to load." : !current ? "The template catalog is stale. Read it again for the current source." : !writable ? "Open writable source to insert a template." : !catalog?.available ? "No template provider is available for this source." : !entry.available ? "This template is unavailable for the current source." : undefined} onClick={()=>{void choose(entry.id);}}>Preview {entry.label}</button><span> {entry.language} · {entry.origin}</span><p>{entry.description}{!entry.available?` · ${entry.reason??'Unavailable for this source'}`:''}</p></div>)}{entries.length>100&&<p>Narrow the search to read the remaining templates.</p>}
  {catalog&&entries.length===0&&<p>No templates matched. This does not establish that the language has no generation capabilities.</p>}
  {preview&&<section aria-label="Template preview"><strong>{preview.entry.label}</strong><p>{preview.kind==='batch'?`${preview.expansions.length} captured selections · `:''}{preview.fields} linked fields · existing mirrors update together</p><pre>{preview.text}</pre>{preview.notices.map((message,index)=><p key={index}>{message}</p>)}<button disabled={loading||!writable} title={loading ? "Wait for the current reading to load." : !writable ? "Open writable source to insert a template." : undefined} onClick={()=>{void apply();}}>Apply template to draft</button></section>}
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
import formats3 from '../../../simple-contract/snippet-regex-format-cases.json';
import * as regex3 from './SnippetTransformV3';
import {afterEach,beforeAll,expect,it,vi} from 'vitest';
import {waitFor} from '@testing-library/react';
import {EditorState,EditorSelection,StateEffect} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {history,undo,redo} from '@codemirror/commands';
import {Buffer,Buffers} from './buffers';
import {sourceSnippets,readSourceSnippet,setSourceSnippet,dispatchSourceSnippetTransactions,readParallelSourceSnippet} from './SourceSnippets';
import {sourceText,authoredSource,editorOffsetIn} from './SourceText';
import {beginSnippet} from './SnippetSession';
import derived from '../../../simple-contract/snippet-derived-cases.json';
import {captureTemplateEditor,readTemplateCatalog,prepareTemplate,applyTemplate,templateSourceHash,engineTemplateProvider,type TemplateProvider,type TemplateEntrySelection,type TemplateContext} from './SourceTemplates';
beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});});
const cleanups:(()=>void)[]=[];afterEach(()=>{for(const close of cleanups.splice(0))close();vi.unstubAllGlobals();});
async function fixture(source='old',from=0,to=source.length){const buffers=new Buffers(()=>{});buffers.workspace='/root';const buffer=new Buffer({path:'a.py',name:'a.py',dir:false},source,'hash',()=>{},false,'/root');buffers.values.set('a.py',buffer);let mounted=true,interaction=0;
 const parent=document.body.appendChild(document.createElement('div')),view=new EditorView({parent,dispatchTransactions:dispatchSourceSnippetTransactions,state:EditorState.create({doc:buffer.text,selection:{anchor:editorOffsetIn(source,from),head:editorOffsetIn(source,to)},extensions:[history(),sourceText(source),sourceSnippets({writable:()=>!buffer.readonly&&!buffer.saving,notice:()=>{}}),EditorView.updateListener.of(update=>{if(authoredSource(update.startState)!==authoredSource(update.state))buffer.edit(authoredSource(update.state));})]})});
 cleanups.push(()=>{view.destroy();parent.remove();});const snapshot=captureTemplateEditor({buffer,buffers,view,mounted:()=>mounted,interaction:()=>interaction}),hash=await templateSourceHash(source),identity={version:1,template_protocol:snapshot.context.version,variable_context:snapshot.context.variable_context,source_files_modified:false,transform_program_versions:snapshot.context.transform_program_versions,source_context:{version:1,scope:'plain',language:'python',languages:['python'],targets:[],reason:null,project_entry:null,source_hashes:{'a.py':hash},graph_fingerprint:'a'.repeat(64),fingerprint:'b'.repeat(64)},language:'python',workspace_id:'workspace',path:'a.py',source_hash:hash,selection:{from,to}},entry={id:'assign',label:'Assignment',description:'Linked variable',language:'python',origin:'builtin',available:true},catalog={...identity,available:true,notices:[],catalog_fingerprint:'catalog',entries:[entry]},plan={...identity,available:true,notices:[],catalog_fingerprint:'catalog',template_id:'assign',origin:'builtin',replacement:{from,to},snippet:{version:1,syntax:'${1:name} = $1;$0',text:'name = name;',fields:[{id:0,parent:null,index:1,from:0,to:4,choices:[]},{id:1,parent:null,index:1,from:7,to:11,choices:[]},{id:2,parent:null,index:0,from:12,to:12,choices:[]}]}};
 const provider:TemplateProvider={catalog:vi.fn(async()=>catalog),plan:vi.fn(async()=>plan)};return {buffers,buffer,view,snapshot,provider,catalog,plan,dispose:()=>{mounted=false;},interact:()=>{interaction++;}};
}
it('previews without writes, then reuses linked fields and exact editor undo/redo',async()=>{const f=await fixture(),before=f.view.state.selection,catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog,preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);expect(f.view.state.doc.toString()).toBe('old');expect(f.buffer.dirty).toBe(false);expect(preview.text).toBe('name = name;');expect(preview.fields).toBe(1);
 await expect(applyTemplate(preview,()=>true,()=>{})).resolves.toBe(true);expect(f.view.state.selection.main.from).toBe(0);expect(f.view.state.selection.main.to).toBe(4);expect(readSourceSnippet(f.view)?.active).toBe(1);expect(f.buffer.base).toBe('old');
 f.view.dispatch({changes:{from:0,to:4,insert:'updated'},selection:{anchor:7},userEvent:'input.type'});expect(f.view.state.doc.toString()).toBe('updated = updated;');expect(undo(f.view)).toBe(true);expect(f.view.state.doc.toString()).toBe('name = name;');expect(undo(f.view)).toBe(true);expect(f.view.state.doc.toString()).toBe('old');expect(f.view.state.selection.eq(before)).toBe(true);expect(redo(f.view)).toBe(true);expect(readSourceSnippet(f.view)?.active).toBe(1);expect(redo(f.view)).toBe(true);expect(f.view.state.doc.toString()).toBe('updated = updated;');
 await expect(applyTemplate(preview,()=>true,()=>{})).rejects.toThrow('obsolete');
});
it('keeps the reviewed insertion immutable and restores UTF16 replacement and selection',async()=>{const f=await fixture('😀old',2,5),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog,preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);if(preview.kind!=='single')throw Error('Expected a single-selection preview');expect(()=>{(preview.item.textEdit as {newText:string}).newText='unreviewed';}).toThrow();expect(()=>{preview.text='unreviewed';}).toThrow();await applyTemplate(preview,()=>true,()=>{});expect(f.view.state.doc.toString()).toBe('😀name = name;');expect(f.view.state.selection.main.from).toBe(2);expect(f.view.state.selection.main.to).toBe(6);expect(undo(f.view)).toBe(true);expect(f.view.state.doc.toString()).toBe('😀old');expect(f.view.state.selection.main.from).toBe(2);expect(f.view.state.selection.main.to).toBe(5);});
it('rejects close, edit, selection or saving while the explicit Apply awaits acceptance',async()=>{for(const mode of ['close','edit','selection','saving'] as const){const f=await fixture(),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog,preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);let current=true;const pending=applyTemplate(preview,()=>current,()=>{});if(mode==='close')current=false;if(mode==='edit')f.view.dispatch({changes:{from:0,to:3,insert:'new'}});if(mode==='selection')f.view.dispatch({selection:{anchor:1}});if(mode==='saving')f.buffer.saving=true;await expect(pending).rejects.toThrow();expect(f.view.state.doc.toString()).toBe(mode==='edit'?'new':'old');}});
it('revalidates the reviewed manifest and expansion immediately before local application',async()=>{for(const mode of ['revoked','changed'] as const){const f=await fixture(),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog,preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);if(mode==='revoked')vi.mocked(f.provider.plan).mockRejectedValueOnce(Error('Template catalog changed'));else vi.mocked(f.provider.plan).mockResolvedValueOnce({...f.plan,snippet:{version:1,syntax:'${1:other} = $1;$0',text:'other = other;',fields:[{id:0,parent:null,index:1,from:0,to:5,choices:[]},{id:1,parent:null,index:1,from:8,to:13,choices:[]},{id:2,parent:null,index:0,from:14,to:14,choices:[]}]}});await expect(applyTemplate(preview,()=>true,()=>{})).rejects.toThrow(/changed/);expect(f.provider.plan).toHaveBeenCalledTimes(2);expect(f.view.state.doc.toString()).toBe('old');expect(f.buffer.dirty).toBe(false);}});
it('previews and applies inside an existing linked scope with exact nested undo/redo',async()=>{const f=await fixture('old + old',0,3),fields=[{id:0,parent:null,index:1,from:0,to:3,choices:[]},{id:1,parent:null,index:1,from:6,to:9,choices:[]},{id:2,parent:null,index:0,from:9,to:9,choices:[]}];f.view.dispatch({effects:setSourceSnippet.of(beginSnippet('old + old',fields))});const snapshot=captureTemplateEditor({...f.snapshot.editor});f.catalog.variable_context=snapshot.context.variable_context;f.plan.variable_context=snapshot.context.variable_context;const catalog=(await readTemplateCatalog(snapshot,f.provider)).catalog,preview=await prepareTemplate(snapshot,catalog,'assign',f.provider);if(preview.kind!=='single')throw Error('Expected a single-selection preview');expect(preview.draft.plan.patches[0].after).toBe('name = name; + name = name;');await applyTemplate(preview,()=>true,()=>{});expect(f.view.state.doc.toString()).toBe('name = name; + name = name;');expect(readSourceSnippet(f.view)?.scopes?.length).toBe(1);expect(undo(f.view)).toBe(true);expect(f.view.state.doc.toString()).toBe('old + old');expect(readSourceSnippet(f.view)?.scopes?.length??0).toBe(0);expect(redo(f.view)).toBe(true);expect(readSourceSnippet(f.view)?.scopes?.length).toBe(1);});
it('refuses duplicate keys and noncanonical numeric template protocols before provider admission',async()=>{const f=await fixture();for(const kind of ['catalog','plan'] as const){const catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog,value=JSON.stringify(kind==='catalog'?f.catalog:f.plan);for(const raw of [value.replace('"version":1','"version":1,"version":1'),value.replace('"template_protocol":2','"template_protocol":2.0'),value.replace('"template_protocol":2','"template_protocol":2e0'),value.replace('"template_protocol":2',String.raw`"\u0074emplate_protocol":2.0`),value.replace('"from":0','"from":0,"from":0')]){vi.stubGlobal('fetch',vi.fn(async()=>new Response(raw,{status:200,headers:{'Content-Type':'application/json'}})));if(kind==='catalog')await expect(readTemplateCatalog(f.snapshot,engineTemplateProvider)).rejects.toThrow();else await expect(prepareTemplate(f.snapshot,catalog,'assign',engineTemplateProvider)).rejects.toThrow();expect(f.buffer.text).toBe('old');expect(f.buffer.dirty).toBe(false);}}});
it('checks excessive choice metadata before defensively copying a provider expansion',async()=>{const f=await fixture(),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog;vi.mocked(f.provider.plan).mockResolvedValueOnce({...f.plan,snippet:{...f.plan.snippet,fields:[{...f.plan.snippet.fields[0],choices:Array<string>(4097).fill('')},f.plan.snippet.fields[2]]}});const clone=vi.spyOn(globalThis,'structuredClone');try{await expect(prepareTemplate(f.snapshot,catalog,'assign',f.provider)).rejects.toThrow(/metadata/);expect(clone).not.toHaveBeenCalled();expect(f.buffer.text).toBe('old');}finally{clone.mockRestore();}});
it('rejects legacy, mismatched and duplicate catalog observations without requesting a plan',async()=>{for(const patch of [{version:true},{template_protocol:1},{template_protocol:0},{template_protocol:true},{source_hash:'wrong'},{selection:{from:1,to:3}},{entries:[{id:'x',label:'x',description:'',language:'python',origin:'workspace',available:true},{id:'x',label:'x',description:'',language:'python',origin:'workspace',available:true}]}]){const f=await fixture();vi.mocked(f.provider.catalog).mockResolvedValue({...f.catalog,...patch});await expect(readTemplateCatalog(f.snapshot,f.provider)).rejects.toThrow();expect(f.provider.plan).not.toHaveBeenCalled();expect(f.buffer.text).toBe('old');}});
it('keeps provider unavailability readable and rejects unsupported or mismatched plans as a whole',async()=>{const f=await fixture();vi.mocked(f.provider.catalog).mockResolvedValue({...f.catalog,available:false,reason:'Provider unavailable'});const unavailable=(await readTemplateCatalog(f.snapshot,f.provider)).catalog;await expect(prepareTemplate(f.snapshot,unavailable,'assign',f.provider)).rejects.toThrow('Provider unavailable');expect(f.provider.plan).not.toHaveBeenCalled();
 vi.mocked(f.provider.catalog).mockResolvedValue(f.catalog);const catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog;for(const patch of [{replacement:{from:0,to:2}},{catalog_fingerprint:'changed'},{workspace_id:'foreign'},{snippet:{version:1,syntax:'${1/name/x/}',text:'unsafe',fields:[]}}]){vi.mocked(f.provider.plan).mockResolvedValue({...f.plan,...patch});await expect(prepareTemplate(f.snapshot,catalog,'assign',f.provider)).rejects.toThrow();expect(f.buffer.text).toBe('old');}
});
it('retires held plans after source, selection, scroll, owner, namespace and mount changes',async()=>{for(const mode of ['text','selection','scroll','owner','path','namespace','mount'] as const){const f=await fixture(),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog;let release!:(value:unknown)=>void;vi.mocked(f.provider.plan).mockImplementationOnce(()=>new Promise(resolve=>{release=resolve;}));const pending=prepareTemplate(f.snapshot,catalog,'assign',f.provider);if(mode==='text')f.view.dispatch({changes:{from:0,to:3,insert:'new'}});if(mode==='selection')f.view.dispatch({selection:{anchor:1}});if(mode==='scroll')f.interact();if(mode==='owner')f.buffers.values.set('a.py',new Buffer({...f.buffer.file},'old','hash',()=>{},false,'/root'));if(mode==='path'){f.buffer.file.path='moved.py';f.buffers.values.delete('a.py');f.buffers.values.set('moved.py',f.buffer);}if(mode==='namespace')f.buffers.workspace='/other';if(mode==='mount')f.dispose();release(f.plan);await expect(pending).rejects.toThrow('changed');expect(f.view.state.doc.toString()).toBe(mode==='text'?'new':'old');}});
it('reads readonly catalogs but refuses generated, saving, deleted, composition and closed previews',async()=>{for(const mode of ['readonly','saving','deleted','composing','closed'] as const){const f=await fixture(),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog,preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);if(mode==='readonly')f.buffer.file={...f.buffer.file,generated_by:'entry.md'};if(mode==='saving')f.buffer.saving=true;if(mode==='deleted')f.buffer.resourceDeleted=true;if(mode==='composing')Object.defineProperty(f.view,'composing',{value:true,configurable:true});await expect(applyTemplate(preview,()=>mode!=='closed',()=>{})).rejects.toThrow();expect(f.buffer.text).toBe('old');expect(f.buffer.dirty).toBe(false);}});

it('captures clipboard only for the selected consumer and retains one snapshot through preview and requery',async()=>{
 for(const available of[true,false]){const f=await fixture(),read=vi.fn(async()=>{if(!available)throw Error('denied');return 'λ😀'});vi.stubGlobal('navigator',{clipboard:{readText:read}});const catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog;expect(read).not.toHaveBeenCalled();const contexts:unknown[]=[];vi.mocked(f.provider.plan).mockImplementation(async context=>{contexts.push(structuredClone(context.variable_context));if(context.variable_context?.clipboard===null)return {...f.plan,variable_context:context.variable_context,available:false,context_request:{version:1,variables:['CLIPBOARD']},reason:'Needs a clipboard reading'};const text=context.variable_context?.clipboard?.available?context.variable_context.clipboard.text!:'fallback';return {...f.plan,variable_context:context.variable_context,notices:available?[]:['Clipboard unavailable; its fallback or empty text was used.'],snippet:{version:1,syntax:'${CLIPBOARD:fallback}$0',text,fields:[{id:0,parent:null,index:0,from:text.length,to:text.length,choices:[]}]}};});const preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);expect(read).toHaveBeenCalledTimes(1);expect(f.buffer.text).toBe('old');expect(preview.text).toBe(available?'λ😀':'fallback');await applyTemplate(preview,()=>true,()=>{});expect(read).toHaveBeenCalledTimes(1);expect(f.buffer.text).toBe(available?'λ😀':'fallback');expect((contexts[0] as {clock:unknown}).clock).toEqual((contexts[1] as {clock:unknown}).clock);expect((contexts[0] as {random_hex:unknown}).random_hex).toEqual((contexts[1] as {random_hex:unknown}).random_hex);expect(contexts[1]).toEqual(contexts[2]);expect(undo(f.view)).toBe(true);expect(f.buffer.text).toBe('old');}
});
it('does not send a clipboard reading after the original source or cursor intent retires',async()=>{
 for(const sourceABA of[true,false]){const f=await fixture(),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog;let release!:(value:string)=>void;const read=vi.fn(()=>new Promise<string>(resolve=>{release=resolve}));vi.stubGlobal('navigator',{clipboard:{readText:read}});vi.mocked(f.provider.plan).mockResolvedValue({...f.plan,available:false,context_request:{version:1,variables:['CLIPBOARD']}});const pending=prepareTemplate(f.snapshot,catalog,'assign',f.provider);await waitFor(()=>expect(read).toHaveBeenCalledTimes(1));if(sourceABA){f.view.dispatch({changes:{from:0,to:3,insert:'new'},selection:{anchor:3},userEvent:'input.type'});expect(undo(f.view)).toBe(true)}else{f.view.dispatch({selection:{anchor:1}});f.view.dispatch({selection:{anchor:0,head:3}})}release('private clipboard');await expect(pending).rejects.toThrow('changed');expect(f.provider.plan).toHaveBeenCalledTimes(1);expect(f.buffer.text).toBe('old');}
});
it('refuses foreign, repeated or altered variable snapshots before any source action',async()=>{
 const f=await fixture(),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog,read=vi.fn(async()=>'clipboard');vi.stubGlobal('navigator',{clipboard:{readText:read}});for(const patch of [{variable_context:{...f.snapshot.context.variable_context,random_hex:'0'.repeat(32)}},{context_request:{version:1,variables:['CLIPBOARD']},catalog_fingerprint:'foreign'},{context_request:{version:1,variables:['OTHER']}}]){vi.mocked(f.provider.plan).mockResolvedValue({...f.plan,...patch});await expect(prepareTemplate(f.snapshot,catalog,'assign',f.provider)).rejects.toThrow();expect(read).not.toHaveBeenCalled();expect(f.buffer.text).toBe('old');}vi.mocked(f.provider.plan).mockImplementation(async context=>({...f.plan,available:false,variable_context:context.variable_context,context_request:{version:1,variables:['CLIPBOARD']}}));await expect(prepareTemplate(f.snapshot,catalog,'assign',f.provider)).rejects.toThrow(/repeated|Unsupported/);expect(read).toHaveBeenCalledTimes(1);expect(f.buffer.text).toBe('old');
});

it('retains computed metadata through the reusable template preview, editor input and one-action history',async()=>{
 const f=await fixture(),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog,syntax='${1:foo} ${1/(.*)/${1:/upcase}/}$0',fields=[{id:0,parent:null,index:1,from:0,to:3,choices:[]},{id:1,parent:null,index:1,from:4,to:7,choices:[],transform:derived.programs.upper},{id:2,parent:null,index:0,from:7,to:7,choices:[]}];vi.mocked(f.provider.plan).mockResolvedValue({...f.plan,snippet:{version:2,syntax,text:'foo FOO',fields}});const preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);if(preview.kind!=='single')throw Error('Expected a single-selection preview');expect((preview.item.data as {hickory_snippet:{version:number}}).hickory_snippet.version).toBe(2);await applyTemplate(preview,()=>true,()=>{});expect(f.buffer.text).toBe('foo FOO');f.view.dispatch({changes:{from:0,to:3,insert:'ßa'},selection:{anchor:2},userEvent:'input.type'});expect(f.buffer.text).toBe('ßa SSA');expect(undo(f.view)).toBe(true);expect(f.buffer.text).toBe('foo FOO');expect(readSourceSnippet(f.view)?.fields.some(field=>!!field.transform)).toBe(true);expect(undo(f.view)).toBe(true);expect(f.buffer.text).toBe('old');expect(redo(f.view)).toBe(true);expect(readSourceSnippet(f.view)?.fields.some(field=>!!field.transform)).toBe(true);
});
it('refuses extra template effects and computed metadata under a legacy expansion version',async()=>{
 const f=await fixture(),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog;for(const key of['command','additionalTextEdits','workspaceEdit','edit']){vi.mocked(f.provider.plan).mockResolvedValue({...f.plan,[key]:null});await expect(prepareTemplate(f.snapshot,catalog,'assign',f.provider)).rejects.toThrow('unsupported additional effects');expect(f.buffer.text).toBe('old');}const syntax='${1:FOO} ${1/(.*)/${1:/upcase}/}$0';vi.mocked(f.provider.plan).mockResolvedValue({...f.plan,snippet:{version:1,syntax,text:'FOO FOO',fields:[{id:0,parent:null,index:1,from:0,to:3,choices:[]},{id:1,parent:null,index:1,from:4,to:7,choices:[],transform:derived.programs.upper},{id:2,parent:null,index:0,from:7,to:7,choices:[]}]}});await expect(prepareTemplate(f.snapshot,catalog,'assign',f.provider)).rejects.toThrow();expect(f.buffer.text).toBe('old');
});

it('captures authored CRLF ranges and applies linked template fields without normalizing untouched bytes',async()=>{
 const original='head\r\nold\r\nend\n',f=await fixture(original,6,9),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog,preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);
 expect(f.snapshot.context.source).toBe(original);expect(f.snapshot.context.selection).toEqual({from:6,to:9});await expect(applyTemplate(preview,()=>true,()=>{})).resolves.toBe(true);
 expect(authoredSource(f.view.state)).toBe('head\r\nname = name;\r\nend\n');expect(f.view.state.selection.main.from).toBe(5);expect(f.view.state.selection.main.to).toBe(9);
 f.view.dispatch({changes:{from:5,to:9,insert:'λ😀'},selection:{anchor:8},userEvent:'input.type'});const changed='head\r\nλ😀 = λ😀;\r\nend\n';expect(authoredSource(f.view.state)).toBe(changed);
 expect(undo(f.view)).toBe(true);expect(authoredSource(f.view.state)).toBe('head\r\nname = name;\r\nend\n');expect(undo(f.view)).toBe(true);expect(authoredSource(f.view.state)).toBe(original);expect(redo(f.view)).toBe(true);expect(redo(f.view)).toBe(true);expect(authoredSource(f.view.state)).toBe(changed);
});
it('retains contributing editors and rejects a source-equal contributor edit undo while catalog is held',async()=>{
 const f=await fixture(),part=new Buffer({path:'part.md',name:'part.md',dir:false},'part','saved',()=>{},false,'/root');f.buffers.values.set('part.md',part);const snapshot=captureTemplateEditor({...f.snapshot.editor});f.catalog.variable_context=snapshot.context.variable_context;f.plan.variable_context=snapshot.context.variable_context;
 let release!:(value:unknown)=>void;vi.mocked(f.provider.catalog).mockImplementation(()=>new Promise(resolve=>{release=resolve}));const pending=readTemplateCatalog(snapshot,f.provider);await waitFor(()=>expect(f.provider.catalog).toHaveBeenCalledTimes(1));part.edit('changed');part.edit('part');release(f.catalog);await expect(pending).rejects.toThrow('changed');expect(f.buffer.text).toBe('old');expect(f.provider.plan).not.toHaveBeenCalled();
});
it('includes readonly contributor text without granting it template write authority',async()=>{const f=await fixture(),part=new Buffer({path:'part.md',name:'part.md',dir:false,generated_by:'owner.md'},'part','saved',()=>{},false,'/root');f.buffers.values.set('part.md',part);const snapshot=captureTemplateEditor({...f.snapshot.editor});expect(snapshot.context.reading_context?.sources['part.md']).toBe('part');expect(snapshot.current(true)).toBe(true);f.catalog.variable_context=snapshot.context.variable_context;f.plan.variable_context=snapshot.context.variable_context;const reading=await readTemplateCatalog(snapshot,f.provider),preview=await prepareTemplate(reading.snapshot,reading.catalog,'assign',f.provider);await applyTemplate(preview,()=>true,()=>{});expect(part.text).toBe('part');expect(f.buffer.text).toBe('name = name;');});
it('captures one original entry lease before review and never retargets its source roster',async()=>{
 const f=await fixture();let alive=true;const proof={fixture:'entry-proof'},entry=vi.fn(async()=>({entryPath:'project.md',proof,current:()=>alive}));const snapshot=captureTemplateEditor({...f.snapshot.editor,entrySelection:entry});f.catalog.variable_context=snapshot.context.variable_context;
 vi.mocked(f.provider.catalog).mockImplementation(async context=>context.reading_context?.entry_selection===undefined?{...f.catalog,available:false,entries:[],language:'unknown',source_context:{...f.catalog.source_context,language:null,scope:'entry-required'},reading_context_request:{version:1,kind:'construction_entry'}}:{...f.catalog,source_context:{...f.catalog.source_context,project_entry:'project.md'}});
 const result=await readTemplateCatalog(snapshot,f.provider);expect(entry).toHaveBeenCalledTimes(1);expect(f.provider.catalog).toHaveBeenCalledTimes(2);expect(snapshot.context.reading_context?.project_entry).toBe(null);expect(result.snapshot.context.reading_context?.project_entry).toBe('project.md');expect(result.snapshot.context.reading_context?.entry_selection).toEqual(proof);expect(result.snapshot.current()).toBe(true);alive=false;expect(result.snapshot.current()).toBe(false);expect(f.buffer.text).toBe('old');
});
it('rejects held entry acquisition after contributor topology changes and rejects substituted language metadata',async()=>{
 const f=await fixture();let release!:(value:TemplateEntrySelection)=>void;const snapshot=captureTemplateEditor({...f.snapshot.editor,entrySelection:()=>new Promise(resolve=>{release=resolve})});f.catalog.variable_context=snapshot.context.variable_context;
 vi.mocked(f.provider.catalog).mockResolvedValue({...f.catalog,available:false,entries:[],language:'unknown',source_context:{...f.catalog.source_context,language:null,scope:'entry-required'},reading_context_request:{version:1,kind:'construction_entry'}});const pending=readTemplateCatalog(snapshot,f.provider);await waitFor(()=>expect(release).toBeTypeOf('function'));f.buffers.values.set('new.md',new Buffer({path:'new.md',name:'new.md',dir:false},'new','base',()=>{},false,'/root'));release({entryPath:'project.md',proof:{fixture:true},current:()=>true});await expect(pending).rejects.toThrow('changed');expect(f.buffer.text).toBe('old');
 const other=await fixture();for(const patch of[{language:'javascript'},{entries:[{...other.catalog.entries[0],language:'javascript'}]},{source_context:{...other.catalog.source_context,source_hashes:{'a.py':'0'.repeat(64)}}}]){vi.mocked(other.provider.catalog).mockResolvedValue({...other.catalog,...patch});await expect(readTemplateCatalog(other.snapshot,other.provider)).rejects.toThrow();}expect(other.provider.plan).not.toHaveBeenCalled();
});
async function batchFixture(source='left right',ranges=[{from:0,to:4},{from:5,to:10}],primary=1,workspace='workspace'){
 const f=await fixture(source);f.view.dispatch({effects:StateEffect.appendConfig.of(EditorState.allowMultipleSelections.of(true))});f.view.dispatch({selection:EditorSelection.create(ranges.map(range=>EditorSelection.range(editorOffsetIn(source,range.from),editorOffsetIn(source,range.to))),primary)});
 const snapshot=captureTemplateEditor(f.snapshot.editor),entry={id:'assign',label:'Grouped template',description:'One complete captured source action',language:'python',origin:'workspace',available:true};
 async function observation(context:TemplateContext){const hash=await templateSourceHash(context.source),facts={version:1,scope:'caret-batch',language:'python',languages:['python'],targets:[],reason:null,project_entry:context.reading_context?.project_entry??null,graph_fingerprint:'b'.repeat(64),fingerprint:'c'.repeat(64)};return {version:1,template_protocol:2,template_batch_protocol:1,path:context.path,selection:context.selection,carets:context.carets,source_hash:hash,workspace_id:workspace,language:'python',catalog_fingerprint:'batch-catalog',variable_context:context.variable_context,transform_program_versions:context.transform_program_versions,source_files_modified:false,source_context:{...facts,source_hashes:{[context.path]:hash}},readings:context.carets!.selections.map((selection,index)=>({selection,context:{...facts,scope:'plain',fingerprint:(index%2?'d':'e').repeat(64)}})),notices:[]};}
 const snippet=(text:string,syntax='${1:${TM_SELECTED_TEXT}}-$1$0')=>({version:2,syntax,text:text+'-'+text,fields:[{id:0,parent:null,index:1,from:0,to:text.length,choices:[]},{id:1,parent:null,index:1,from:text.length+1,to:2*text.length+1,choices:[]},{id:2,parent:null,index:0,from:2*text.length+1,to:2*text.length+1,choices:[]}]});
 const provider:TemplateProvider={catalog:vi.fn(async(context:TemplateContext)=>({...await observation(context),available:true,entries:[entry]})),plan:vi.fn(async(context:TemplateContext)=>({...await observation(context),available:true,template_id:entry.id,origin:entry.origin,expansions:context.carets!.selections.map(selection=>({selection,replacement:selection,snippet:snippet(context.source.slice(selection.from,selection.to))}))}))};
 return {...f,snapshot,provider,entry,observation,snippet};
}
it('admits a hosted template batch only through the captured workspace and draft commit',async()=>{
 const workspaceRoot='/root',f=await batchFixture(undefined,undefined,undefined,await templateSourceHash(workspaceRoot)),selection=f.view.state.selection;let live=true;
 const commit:NonNullable<import('./SourceCompletion').SourceCompletionAdapter['commit']>=vi.fn((view,batch,privateDraft)=>{
  expect(view).toBe(f.view);expect(privateDraft).toBe(true);expect(batch).toHaveLength(2);expect(batch[0].startState).toBe(view.state);
  view.update(batch);
 });
 const snapshot=captureTemplateEditor({id:'main-tab',view:f.view,mounted:()=>true,capture:()=>({path:'a.py',workspaceRoot,
  contributors:f.snapshot.context.reading_context!,current:()=>live,entrySelection:async()=>{throw Error('Unexpected entry demand');},commit})});
 vi.mocked(f.provider.catalog).mockImplementationOnce(async context=>({...await f.observation(context),workspace_id:'foreign',available:true,entries:[f.entry]}));
 await expect(readTemplateCatalog(snapshot,f.provider)).rejects.toThrow('workspace identity');
 const catalog=(await readTemplateCatalog(snapshot,f.provider)).catalog,preview=await prepareTemplate(snapshot,catalog,'assign',f.provider);
 expect(f.buffer.text).toBe('left right');expect(commit).not.toHaveBeenCalled();
 live=false;await expect(applyTemplate(preview,()=>true,()=>{})).rejects.toThrow('obsolete');expect(commit).not.toHaveBeenCalled();
 live=true;await applyTemplate(preview,()=>true,()=>{});expect(commit).toHaveBeenCalledOnce();
 expect(f.buffer.text).toBe('left-left right-right');expect(readParallelSourceSnippet(f.view.state)?.peers).toHaveLength(2);
 expect(undo(f.view)).toBe(true);expect(f.buffer.text).toBe('left right');expect(f.view.state.selection.eq(selection)).toBe(true);
 expect(redo(f.view)).toBe(true);expect(readParallelSourceSnippet(f.view.state)?.peers).toHaveLength(2);
});
it('previews all carets and applies independent field defaults in one source undo action',async()=>{
 const f=await batchFixture(),original=f.view.state.selection,catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog,preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);expect(preview.kind).toBe('batch');expect(f.buffer.text).toBe('left right');expect(preview.text).toContain('left-left');expect(preview.text).toContain('right-right');await applyTemplate(preview,()=>true,()=>{});expect(f.buffer.text).toBe('left-left right-right');expect(readParallelSourceSnippet(f.view.state)?.peers).toHaveLength(2);f.view.dispatch({...f.view.state.replaceSelection('x'),userEvent:'input.type'});expect(f.buffer.text).toBe('x-x x-x');expect(undo(f.view)).toBe(true);expect(f.buffer.text).toBe('left-left right-right');expect(undo(f.view)).toBe(true);expect(f.buffer.text).toBe('left right');expect(f.view.state.selection.eq(original)).toBe(true);expect(redo(f.view)).toBe(true);expect(redo(f.view)).toBe(true);expect(f.buffer.text).toBe('x-x x-x');
});
it('continues the actual logical peer group through nested preview, Apply and history',async()=>{
 const f=await batchFixture(),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog,outer=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);await applyTemplate(outer,()=>true,()=>{});const before=f.buffer.text,snapshot=captureTemplateEditor(f.snapshot.editor);expect(snapshot.context.carets?.continuing).toBe(true);
 vi.mocked(f.provider.plan).mockImplementation(async context=>({...await f.observation(context),available:true,template_id:'assign',origin:'workspace',expansions:context.carets!.selections.map(selection=>({selection,replacement:selection,snippet:f.snippet('kid','${1:kid}-$1$0')}))}));const nestedCatalog=(await readTemplateCatalog(snapshot,f.provider)).catalog,nested=await prepareTemplate(snapshot,nestedCatalog,'assign',f.provider);await applyTemplate(nested,()=>true,()=>{});expect(f.buffer.text).toBe('kid-kid-kid-kid kid-kid-kid-kid');f.view.dispatch({...f.view.state.replaceSelection('x'),userEvent:'input.type'});expect(f.buffer.text).toBe('x-x-x-x x-x-x-x');expect(undo(f.view)).toBe(true);expect(undo(f.view)).toBe(true);expect(f.buffer.text).toBe(before);expect(readParallelSourceSnippet(f.view.state)?.peers).toHaveLength(2);expect(redo(f.view)).toBe(true);expect(redo(f.view)).toBe(true);expect(f.buffer.text).toBe('x-x-x-x x-x-x-x');
});
it('keeps one clipboard reading and original clock seed for the whole caret request',async()=>{
 const f=await batchFixture('old ',[{from:0,to:3},{from:4,to:4}],0),read=vi.fn(async()=>{throw Error('denied')});vi.stubGlobal('navigator',{clipboard:{readText:read}});const contexts:unknown[]=[];
 vi.mocked(f.provider.plan).mockImplementation(async context=>{contexts.push(structuredClone(context.variable_context));const base={...await f.observation(context),template_id:'assign',origin:'workspace'};if(context.variable_context?.clipboard===null)return {...base,available:false,context_request:{version:1,variables:['CLIPBOARD']}};return {...base,available:true,expansions:context.carets!.selections.map(selection=>({selection,replacement:selection,snippet:{version:2,syntax:'${CLIPBOARD:fallback}$0',text:'fallback',fields:[{id:0,parent:null,index:0,from:8,to:8,choices:[]}]}}))};});
 const catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog;expect(read).not.toHaveBeenCalled();const preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider);expect(read).toHaveBeenCalledTimes(1);await applyTemplate(preview,()=>true,()=>{});expect(read).toHaveBeenCalledTimes(1);expect(f.buffer.text).toBe('fallback fallback');expect((contexts[0] as {clock:unknown}).clock).toEqual((contexts[1] as {clock:unknown}).clock);expect((contexts[0] as {random_hex:unknown}).random_hex).toEqual((contexts[1] as {random_hex:unknown}).random_hex);expect(contexts[1]).toEqual(contexts[2]);expect(undo(f.view)).toBe(true);expect(f.buffer.text).toBe('old ');
});
it('refuses foreign or partial caret arrays and same-selection stale intents without source edits',async()=>{
 for(const mode of['partial','primary','reading','program','intent'] as const){const f=await batchFixture(),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog;if(mode==='intent'){const preview=await prepareTemplate(f.snapshot,catalog,'assign',f.provider),selected=f.view.state.selection;f.view.dispatch({selection:selected});await expect(applyTemplate(preview,()=>true,()=>{})).rejects.toThrow('obsolete');}else{const original=await f.provider.plan({...f.snapshot.context,catalog_fingerprint:catalog.catalog_fingerprint,template_id:'assign'}) as Record<string,any>;if(mode==='partial')original.expansions.pop();if(mode==='primary')original.carets={...original.carets,primary:0};if(mode==='reading')original.readings[1].context.fingerprint='f'.repeat(64);if(mode==='program')original.expansions[1].snippet.fields[0].transform={version:99};vi.mocked(f.provider.plan).mockResolvedValue(original);await expect(prepareTemplate(f.snapshot,catalog,'assign',f.provider)).rejects.toThrow();}expect(f.buffer.text).toBe('left right');}
});
it('pins template program policy and refuses unnegotiated v3 before reader conversion',async()=>{
 const f=await batchFixture();expect(f.snapshot.context.transform_program_versions).toEqual([2,3]);
 const legacy={...f.snapshot,context:{...f.snapshot.context,transform_program_versions:Object.freeze([2])}};
 const reading=await readTemplateCatalog(legacy,f.provider),raw=await f.provider.plan({...legacy.context,catalog_fingerprint:reading.catalog.catalog_fingerprint,template_id:'assign'}) as Record<string,any>;
 raw.expansions[1].snippet.fields[0].transform={version:3};vi.mocked(f.provider.plan).mockResolvedValue(raw);const clone=vi.spyOn(globalThis,'structuredClone');try{await expect(prepareTemplate(legacy,reading.catalog,'assign',f.provider)).rejects.toThrow('not accepted');expect(clone).not.toHaveBeenCalled();}finally{clone.mockRestore();}
 expect(f.buffer.text).toBe('left right');
 const normal=await batchFixture();vi.mocked(normal.provider.catalog).mockImplementationOnce(async context=>({...await f.provider.catalog(context) as Record<string,unknown>,transform_program_versions:[2]}));await expect(readTemplateCatalog(normal.snapshot,normal.provider)).rejects.toThrow('versions changed');
});

it('admits aggregate raw program metadata before constructing any v3 VM',async()=>{
 const f=await batchFixture(),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog,raw=await f.provider.plan({...f.snapshot.context,catalog_fingerprint:catalog.catalog_fingerprint,template_id:'assign'}) as Record<string,any>;
 for(const row of raw.expansions){row.snippet.fields[0].choices=[row.snippet.text.slice(row.snippet.fields[0].from,row.snippet.fields[0].to),...Array(3999).fill('x'.repeat(128))];row.snippet.fields[1].transform=structuredClone(formats3[0].program);}
 vi.mocked(f.provider.plan).mockResolvedValue(raw);const reader=vi.spyOn(regex3,'readRegexV3'),clone=vi.spyOn(globalThis,'structuredClone');try{await expect(prepareTemplate(f.snapshot,catalog,'assign',f.provider)).rejects.toThrow('metadata');expect(reader).not.toHaveBeenCalled();expect(clone).not.toHaveBeenCalled();expect(f.buffer.text).toBe('left right');}finally{reader.mockRestore();clone.mockRestore();}
});

it('preflights the complete expansion metadata before copying any row',async()=>{
 const f=await batchFixture(),catalog=(await readTemplateCatalog(f.snapshot,f.provider)).catalog,original=await f.provider.plan({...f.snapshot.context,catalog_fingerprint:catalog.catalog_fingerprint,template_id:'assign'}) as Record<string,any>;original.expansions[1].snippet.fields[0].choices=Array(4097).fill('x');vi.mocked(f.provider.plan).mockResolvedValue(original);const clone=vi.spyOn(globalThis,'structuredClone');await expect(prepareTemplate(f.snapshot,catalog,'assign',f.provider)).rejects.toThrow();expect(clone).not.toHaveBeenCalled();clone.mockRestore();expect(f.buffer.text).toBe('left right');
});
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
import {captureTemplateEditor,readTemplateCatalog,prepareTemplate,applyTemplate,engineTemplateProvider,type TemplateEditor,type SimpleTemplateEditor,type TemplateSnapshot,type TemplateCatalog,type TemplatePreview} from './SourceTemplates';
import {SourceTemplatesView} from './SourceTemplatesView';
vi.mock('./SourceTemplates',()=>({captureTemplateEditor:vi.fn(),readTemplateCatalog:vi.fn(),prepareTemplate:vi.fn(),applyTemplate:vi.fn(),engineTemplateProvider:{catalog:vi.fn(),plan:vi.fn()}}));
afterEach(cleanup);beforeEach(()=>vi.resetAllMocks());
function fixture(options:{readonly?:boolean;available?:boolean}={}){const buffers=new Buffers(()=>{});buffers.workspace='/root';const buffer=new Buffer({path:'a.py',name:'a.py',dir:false},'old','hash',()=>{},false,'/root');buffers.values.set('a.py',buffer);const view={dom:document.createElement('div'),scrollDOM:document.createElement('div')} as unknown as TemplateEditor['view'];const editor:SimpleTemplateEditor={buffer,buffers,view,mounted:()=>true,interaction:()=>0};let active=editor;
 const snapshot={editor,view,context:{version:1,path:'a.py',source:'old',selection:{from:0,to:3}},selection:{},current:(writable=false)=>!writable||!options.readonly} as TemplateSnapshot;
 const catalog={version:1,template_protocol:1,language:'python',workspace_id:'workspace',path:'a.py',source_hash:'hash',selection:{from:0,to:3},available:options.available!==false,reason:'Provider unavailable',notices:[],catalog_fingerprint:'catalog',entries:[{id:'assign',label:'Assignment',description:'Linked variable',language:'python',origin:'builtin',available:true}]} as TemplateCatalog;
 const preview={snapshot,catalog,entry:catalog.entries[0],provider:engineTemplateProvider,kind:'single',item:{label:'Assignment'},draft:{path:'a.py',before:'old',plan:{title:'Template',patches:[]},insertionFrom:0,selection:{from:0,to:4},fields:[]},text:'name = name;',fields:1,notices:[]} as TemplatePreview;
 vi.mocked(captureTemplateEditor).mockReturnValue(snapshot);vi.mocked(readTemplateCatalog).mockResolvedValue({snapshot,catalog});vi.mocked(prepareTemplate).mockResolvedValue(preview);vi.mocked(applyTemplate).mockImplementation(async(_plan,current)=>{if(!current())throw Error('Editor changed');return true;});
 const close=vi.fn();function Harness(){const [visible,setVisible]=useState(true),[,paint]=useState(0);return <><button onClick={()=>{active={...editor,buffer:new Buffer({...buffer.file},'old','hash',()=>{},false,'/root')};paint(value=>value+1);}}>Replace source owner</button><SourceTemplatesView visible={visible} editor={()=>active} close={()=>{close();setVisible(false);}}/></>;}render(<Harness/>);return {snapshot,catalog,preview,close};
}
it('reads and previews without writes until explicit Apply, then closes the same floating view',async()=>{const f=fixture();await screen.findByRole('button',{name:'Preview Assignment'});expect(applyTemplate).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('button',{name:'Preview Assignment'}));await screen.findByRole('button',{name:'Apply template to draft'});expect(screen.getByRole('region',{name:'Template preview'}).textContent).toContain('name = name;');expect(applyTemplate).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('button',{name:'Apply template to draft'}));await waitFor(()=>expect(f.close).toHaveBeenCalledOnce());expect(applyTemplate).toHaveBeenCalledOnce();});
it('dismisses a held preview without applying or displaying an obsolete reply',async()=>{fixture();await screen.findByRole('button',{name:'Preview Assignment'});let release!:(value:TemplatePreview)=>void;const held=vi.mocked(prepareTemplate).mock.results.length;let preview!:TemplatePreview;vi.mocked(prepareTemplate).mockImplementationOnce(async(...args)=>new Promise(resolve=>{release=resolve;preview={snapshot:args[0],catalog:args[1],entry:args[1].entries[0],provider:engineTemplateProvider,kind:'single',item:{label:'Assignment'},draft:{path:'a.py',before:'old',plan:{title:'Template',patches:[]},insertionFrom:0,selection:{from:0,to:4},fields:[]},text:'late',fields:1,notices:[]} as TemplatePreview;}));fireEvent.click(screen.getByRole('button',{name:'Preview Assignment'}));expect(vi.mocked(prepareTemplate).mock.calls.length).toBe(held+1);fireEvent.click(screen.getByRole('button',{name:'Close source templates'}));await act(async()=>release(preview));expect(screen.queryByRole('dialog',{name:'Source templates'})).toBeNull();expect(applyTemplate).not.toHaveBeenCalled();});
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
        manifest={"version":1,"templates":[{"id":"workspace.linked","label":"Workspace linked","description":"Mirrors, Unicode and selected text","language":"python","syntax":"${1|first,second|} + $1\n${2:😀 value} ${TM_FILENAME} ${TM_SELECTED_TEXT}$0"},{"id":"workspace.unsupported","label":"Unsupported transform","description":"Explicit unsupported capability","language":"python","syntax":"${1/(?=x)/y/}"}]}
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


## Verified negotiated program paths

On 2026-11-18 the generated focused selection passed 38 checks: 28 template policy checks, four floating-view checks and six current shared entry-reader checks. Full generated web typechecking passed with zero diagnostics. The captured-policy, intentional legacy `[2]` refusal and aggregate raw metadata-before-VM checks executed; logs are `target/template-program3-browser-focused.log` and `target/template-program3-browser-tsc.log`.

The fresh-engine rendered `scripts/test-embedded-templates-browser.py --program3` passed for a code fence and a dirty reused fragment, including lookbehind/Unicode letter properties, nested computed mirrors, distinct cursor variables under one graph/clock/seed, source Undo/Redo and held secondary-selection refusal. The ordinary `--batch` uppercase/variable path also passed under the new default policy. Final logs are `target/template-program3-browser-actual.log` and `target/template-program3-browser-compatibility.log`. Saved bytes and editor geometry/identity remained unchanged; no generated output was materialized. Physical input and accessibility acceptance remain unverified.

</ui:doc>
