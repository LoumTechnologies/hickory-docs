<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Accept a complete source completion through the existing popup

For maintainers changing source completion behavior independently of popup and editor rendering.

## Observe asynchronous acceptance independently of a popup

A popup acceptance can begin a source-bound resolver job. Keep its latest identity, cancellation signal and observed outcome separate from whether the native popup is open. Read returns a copy; a superseded job cannot update the current observation. Escape or the invoking agent signal cancels pending acceptance. A waiter resolves only when the acceptance is applied or unavailable, so an agent does not treat popup acceptance as a committed edit. Saved and verified remain false for this draft-only operation.

<ui:copy id="completion-acceptance-observation">
import type {EditorView} from '@codemirror/view';
export interface CompletionAcceptance {job:string;phase:'resolving'|'applying'|'applied'|'unavailable';message:string;applied:boolean;source_changed:boolean;saved:false;verified:false}
interface PendingAcceptance {value:CompletionAcceptance;cancelled:boolean;promise:Promise<CompletionAcceptance>;update:(change:Partial<CompletionAcceptance>)=>void;cancel:(message:string)=>boolean}
const latest=new WeakMap<EditorView,PendingAcceptance>(),acceptingSignals=new WeakMap<EditorView,AbortSignal>();
export function withCompletionAcceptanceSignal<T>(view:EditorView,signal:AbortSignal,accept:()=>T):T {acceptingSignals.set(view,signal);try{return accept();}finally{acceptingSignals.delete(view);}}
export function readCompletionAcceptance(view:EditorView):CompletionAcceptance|null {const value=latest.get(view)?.value;return value?{...value}:null;}
export function waitCompletionAcceptance(view:EditorView,job:string):Promise<CompletionAcceptance> {const current=latest.get(view);return current?.value.job===job?current.promise:Promise.reject(new Error('Completion acceptance was superseded.'));}
export function cancelCompletionAcceptance(view:EditorView):boolean {return latest.get(view)?.cancel('Completion cancelled.')??false;}
export function beginCompletionAcceptance(view:EditorView,notice:(message:string)=>void=()=>{}) {
 const previous=latest.get(view);if(previous&&previous.value.phase==='resolving'){previous.cancelled=true;previous.update({phase:'unavailable',message:'Superseded by a newer completion.'});}
 const signal=acceptingSignals.get(view),value:CompletionAcceptance={job:crypto.randomUUID(),phase:'resolving',message:'Resolving completion…',applied:false,source_changed:false,saved:false,verified:false};
 let settle!:(value:CompletionAcceptance)=>void;
 const promise=new Promise<CompletionAcceptance>(resolve=>{settle=resolve;});
 const onAbort=()=>{entry.cancel('Completion cancelled.');};
 const entry:PendingAcceptance={value,cancelled:false,promise,update:change=>{Object.assign(value,change);if(value.phase==='applied'||value.phase==='unavailable'){signal?.removeEventListener('abort',onAbort);settle({...value});}},cancel:message=>{if(value.phase!=='resolving')return false;entry.cancelled=true;entry.update({phase:'unavailable',message});if(latest.get(view)===entry)notice(message);return true;}};
 latest.set(view,entry);signal?.addEventListener('abort',onAbort,{once:true});if(signal?.aborted)onAbort();
 return {job:value.job,cancelled:()=>entry.cancelled,current:()=>latest.get(view)===entry,update:(change:Partial<CompletionAcceptance>)=>{if(latest.get(view)===entry||change.phase==='applied'||change.phase==='unavailable')entry.update(change);}};
}
</ui:copy>

<ui:file path="apps/web/src/lsp/completionAcceptance.ts"><ui:paste select="#completion-acceptance-observation"/></ui:file>

## Prepare and commit one complete browser transaction

An optional source-editor adapter retains the actual native choice and query selection, resolves through its producing client and rechecks attachment, source, selection, provider, permissions, saving and composition before planning. An offered query and its pending acceptance retain the original document object and the shared editor-input intent token. Source edits, undo and cursor round trips advance that token; harmless popup/capability effects preserve it. Equal bytes or a restored cursor cannot revive an old query. The whole source/import plan passes the shared input guard and transaction preparation before one synchronous update with isolated undo. A following selection-only transaction in that same update records the final selection for CodeMirror redo; it adds no second document edit or separate undo step. Guarded no-op selection is supported; a late or replaced job cannot write source or overwrite a newer status.

For a completion at up to 16 carets, retain the main popup choice
and independently query and resolve the unique matching offer at every peer.
Check the exact source hash, revision and distinct opaque provider tokens.
The drafts planner requires disjoint primary edits and a single agreed set
of additional edits. Submit the resulting text and all carets through the
same editor transaction. Snippet offers share one clock and random capture.
Each selection receives its authored ordinal. Validate the first replies
before sharing one clipboard read with the peers whose tokens requested
it. The transaction installs one parallel field session and the editor's
history retains the full original and resulting selections.

An active template group retains its own selections and linked fields. A
separate ordinary completion waits until that template has finished.

Provider edits, snippet fields and retained source hashes use authored UTF-16 coordinates. The browser converts popup replacement offsets to authored positions once, prepares the entire raw edit with its line-ending proof, then maps only the final caret back into the display. Applied status and Undo/Redo checks compare exact authored bytes, including mixed line endings.

<ui:copy id="completion-acceptance-browser">
import {isolateHistory} from '@codemirror/commands';
import {EditorSelection,EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {closeCompletion,pickedCompletion,CompletionContext,type Completion} from '@codemirror/autocomplete';
import type {CompletionItem} from '../lsp/client';
import {lspCompletion,wordBefore} from '../lsp/completion';
import {beginCompletionAcceptance} from '../lsp/completionAcceptance';
import {prepareEditorChanges} from '../editor/textInput';
import {planCompletion,planCompletionGroup,preflightSnippetGroup,type CapturedCompletion,type CompletionDraft} from './CompletionPlan';
import {sourceDigest} from '../lsp/diagnosticRevision';
import {completionExpansion} from './SnippetExpansion';
import {bindSnippetVariables,captureSnippetClipboard,captureSnippetVariables,snippetContextNotice,snippetRequestsClipboard,type SnippetVariableSnapshot} from './SnippetContext';
import {sourceSnippetEffect,continueSourceSnippetCompletion,readSourceSnippet,readParallelSourceSnippet,readSourceEditorIntent,parallelCompletionSelection} from './SourceSnippets';
import {authoredSource,authoredOffset,editorOffsetIn,authoredChanges} from './SourceText';
export interface SourceCompletionAdapter {
 uri:string;resolve:(item:CompletionItem)=>Promise<CompletionItem>;current:()=>boolean;writable:()=>boolean;notice:(text:string)=>void;
 group?:{query:(offset:number,source:string)=>Promise<CompletionItem[]>;resolve:(item:CompletionItem)=>Promise<CompletionItem>};
}
const querySnapshots=new WeakMap<EditorState,{source:string;selection:EditorState['selection'];document:EditorState['doc'];intent:symbol|undefined}>();
export function sourceCompletion(item:CompletionItem,context:CompletionContext,adapter:SourceCompletionAdapter):Completion {
 let snapshot=querySnapshots.get(context.state);
 if(!snapshot){snapshot={source:authoredSource(context.state),selection:context.state.selection,document:context.state.doc,intent:readSourceEditorIntent(context.state)};querySnapshots.set(context.state,snapshot);}
 const {source,selection,document,intent}=snapshot,original=structuredClone(item);
 const label=typeof item.filterText==='string'?item.filterText:item.label;
 return {...lspCompletion(item),label,...(label!==item.label?{displayLabel:item.label}:{}),apply:(view,completion,from,to)=>{closeCompletion(view);try{void acceptSourceCompletion(view,original,source,selection,authoredOffset(view.state,from),authoredOffset(view.state,to),{...adapter,current:()=>adapter.current()&&view.state.doc===document&&readSourceEditorIntent(view.state)===intent},completion);}catch(error){adapter.notice(error instanceof Error?error.message:String(error));}}};
}

function offerIdentity(item:CompletionItem) {
 const details = item.labelDetails as {detail?:unknown;description?:unknown} | undefined;
 return JSON.stringify([item.label,item.kind,item.detail,details?.detail,details?.description]);
}

function groupProof(item:CompletionItem,hash:string,revision?:string) {
 const data = item.data as {hickory_completion?:unknown;hickory_context?:{source_hash?:unknown;source_revision?:unknown}} | undefined;
 const token = data?.hickory_completion, context = data?.hickory_context;
 const currentRevision = context?.source_revision;
 if (item.command != null) throw new Error("Completion groups require provider edits without commands.");
 if (typeof token !== "string" || !token || context?.source_hash !== hash || typeof currentRevision !== "string"
   || !/^[a-f0-9]{64}$/.test(currentRevision) || (revision !== undefined && currentRevision !== revision)) {
  throw new Error("Completion proof changed across carets. Request the completions again.");
 }
 return {token,revision:currentRevision,context};
}

async function resolveCompletionGroup(state:EditorState,source:string,item:CompletionItem,adapter:SourceCompletionAdapter,valid:()=>boolean) {
 const session = adapter.group;
 if (!session || state.selection.ranges.length > 16) throw new Error("Choose 2 to 16 supported carets for a completion group.");
 const selections = state.selection.ranges.map(range => {
  const prefix = range.empty ? wordBefore(new CompletionContext(state,range.head,true)) : null;
  return {
   from:authoredOffset(state,range.from),to:authoredOffset(state,range.to),head:authoredOffset(state,range.head),
   ...(prefix ? {prefixFrom:authoredOffset(state,prefix.from)} : {}),
  };
 });
 const snapshot = captureSnippetVariables();
 const hash = await sourceDigest(source);
 const requireCurrent = () => { if (!valid()) throw new Error("Source, carets or provider changed. Request completions again."); };
 requireCurrent();
 const proof = groupProof(item,hash);
 const seen = new Set<string>();
 const captures: CapturedCompletion[] = [];
 const demands = new Map<number,CompletionItem>();
 const main = state.selection.mainIndex;
 const identity = offerIdentity(item);
 async function resolvePeer(offered:CompletionItem,index:number,values:SnippetVariableSnapshot) {
  const {from,to} = selections[index];
  const offeredProof = groupProof(offered,hash,proof.revision);
  const context = await bindSnippetVariables(source,from,to,offeredProof.context,values,{index,count:selections.length});
  requireCurrent();
  const data = offered.data as Record<string,unknown>;
  const resolved = await session!.resolve({...structuredClone(offered),data:{...data,hickory_context:context}});
  requireCurrent();
  const returned = groupProof(resolved,hash,proof.revision);
  const demand = snippetRequestsClipboard(resolved);
  if (resolved.label !== item.label || (demand
    ? resolved.insertTextFormat !== 2 || values.clipboard !== null || seen.has(returned.token)
      || Object.hasOwn(resolved.data as object,'hickory_snippet')
    : returned.token !== offeredProof.token)) {
   throw new Error("The provider returned a different completion. Request the completions again.");
  }
  if (demand) {
   seen.add(returned.token);
   demands.set(index,resolved);
  }
  return resolved;
 }
 for (const index of [main,...selections.keys()].filter((index,offset) => offset === 0 || index !== main)) {
  const selection = selections[index];
  const matching = index === main ? [item] : (await session.query(selection.head,source)).filter(candidate => offerIdentity(candidate) === identity);
  requireCurrent();
  if (matching.length !== 1) throw new Error("A caret has no unique matching completion. Choose a different completion.");
  const offer = groupProof(matching[0],hash,proof.revision);
  if (seen.has(offer.token)) throw new Error("Each caret requires its own provider completion offer.");
  seen.add(offer.token);
  const resolved = await resolvePeer(matching[0],index,snapshot);
  captures[index] = {item:resolved,...selection};
 }
 const format = captures[0].item.insertTextFormat ?? 1;
 if ((format !== 1 && format !== 2) || captures.some(({item}) => (item.insertTextFormat ?? 1) !== format)) {
  throw new Error("The carets must agree on the completion format.");
 }
 const budget = {work:0};
 for (const [index,capture] of captures.entries()) {
  if (!demands.has(index)) capture.expansion = completionExpansion(capture.item,budget);
 }
 if (demands.size) {
  preflightSnippetGroup(adapter.uri,{text:source,writable:adapter.writable()},captures);
  const clipboard = await captureSnippetClipboard();
  requireCurrent();
  const edit = (item:CompletionItem) => JSON.stringify([item.textEdit,item.insertText,item.additionalTextEdits]);
  for (const [index,pending] of demands) {
   const resolved = await resolvePeer(pending,index,{...snapshot,clipboard});
   if (resolved.insertTextFormat !== format || edit(resolved) !== edit(pending)) {
    throw new Error("The clipboard reply changed the completion edit.");
   }
   captures[index].item = resolved;
   captures[index].expansion = completionExpansion(resolved,budget);
  }
 }
 const notice = [...new Set(captures.map(({item}) => snippetContextNotice(item)).filter(Boolean))].join(' · ');
 return {...planCompletionGroup(adapter.uri,{text:source,writable:adapter.writable()},captures,main),notice};
}

export async function acceptSourceCompletion(view:EditorView,item:CompletionItem,source:string,selection:EditorState['selection'],from:number,to:number,adapter:SourceCompletionAdapter,completion?:Completion):Promise<void> {
 const request=beginCompletionAcceptance(view,adapter.notice),snippetContext=readSourceSnippet(view),document=view.state.doc,intent=readSourceEditorIntent(view.state);
 const parallelContext=readParallelSourceSnippet(view.state);
 let expectedAfter:string|undefined;
 const valid=()=>!request.cancelled()&&request.current()&&readSourceSnippet(view)===snippetContext&&readParallelSourceSnippet(view.state)===parallelContext&&view.state.doc===document&&readSourceEditorIntent(view.state)===intent&&adapter.current()&&view.dom.isConnected&&authoredSource(view.state)===source&&view.state.selection.eq(selection)&&adapter.writable()&&!view.composing&&!view.state.readOnly&&view.state.facet(EditorView.editable);
 const report=(text:string)=>{if(request.current())adapter.notice(text);};
 try {
  if(request.cancelled())return;
  if(view.composing)throw new Error('Finish composition before accepting this completion.');
  if(!valid())throw new Error('Completion is out of date or source is unavailable. Request completions again.');
  report('Resolving completion…');
  let draft:CompletionDraft|undefined,group:Awaited<ReturnType<typeof resolveCompletionGroup>>|undefined,contextNotice='';
  if (selection.ranges.length > 1) {
   if (snippetContext || parallelContext) throw new Error('Finish editing the template before accepting a completion group.');
   group = await resolveCompletionGroup(view.state,source,item,adapter,valid);
   contextNotice = group.notice;
  } else {
   const resolved=await adapter.resolve(structuredClone(item));
   if(!request.current()||request.cancelled())return;
   if(view.composing)throw new Error('Finish composition, then request completions again.');
   if(!valid())throw new Error('Source, cursor or provider changed. Request completions again.');
   contextNotice=snippetContextNotice(resolved);
   draft=continueSourceSnippetCompletion(view,planCompletion(adapter.uri,resolved,{text:source,writable:adapter.writable()},from,to,completionExpansion(resolved)));
  }
  const patch=(group?.plan ?? draft!.plan).patches[0],after=patch?.after??source,changes=(patch?.edits??[]).map(edit=>({from:edit.from,to:edit.to,insert:edit.text}));
  const input=prepareEditorChanges(source,changes);if('reason' in input)throw new Error(input.reason);
  if(!valid())throw new Error('Source, cursor or provider changed. Request completions again.');
  const raw=authoredChanges(view.state,changes),rawEffects=raw.effects?[raw.effects].flat():[];
  const linked = group?.parallel ? parallelCompletionSelection(group.parallel) : null;
  if (group?.parallel && !linked) throw new Error('The editor cannot retain every snippet selection.');
  const resultSelection = linked?.selection ?? (group
   ? EditorSelection.create(group.carets.map(caret=>EditorSelection.cursor(editorOffsetIn(after,caret))),selection.mainIndex)
   : EditorSelection.single(editorOffsetIn(after,draft!.selection.from),editorOffsetIn(after,draft!.selection.to)));
  const effect = linked?.effect ?? (draft ? sourceSnippetEffect(draft,after) : undefined);
  const transaction=view.state.update({...raw,selection:resultSelection,effects:effect?[...rawEffects,effect]:rawEffects,annotations:[isolateHistory.of('full'),...(completion?[pickedCompletion.of(completion)]:[])],userEvent:'input.complete',filter:false});
  if(authoredSource(transaction.state)!==after || (group?.parallel && readParallelSourceSnippet(transaction.state)!==group.parallel))throw new Error('Editor could not prepare the complete completion. Request again.');
  // CodeMirror records the redo selection from a following selection event;
  // otherwise it maps the old caret through the replacement and loses fields.
  const retainedSelection=after!==source?transaction.state.update({selection:transaction.newSelection,userEvent:'select.complete',filter:false}):null;
  expectedAfter=after;request.update({phase:'applying'});view.update(retainedSelection?[transaction,retainedSelection]:[transaction]);
  const applied=authoredSource(view.state)===after;
  request.update({phase:applied?'applied':'unavailable',applied,source_changed:applied&&source!==after,message:applied?'Completion applied to draft.':'Editor could not reflect the complete completion. Inspect the current draft.'});
  report(applied?((source!==after?'Completion applied · draft':'Completion selected')+(contextNotice?' · '+contextNotice:'')):'Editor could not reflect the complete completion. Inspect the current draft.');
 }catch(error){if((request.current()||expectedAfter!==undefined)&&!request.cancelled()){const applied=expectedAfter!==undefined&&authoredSource(view.state)===expectedAfter,message=error instanceof Error?error.message:String(error);request.update({phase:'unavailable',message,applied,source_changed:applied&&source!==expectedAfter});report(message);}}
}
</ui:copy>

<ui:file path="apps/web/src/simple/SourceCompletion.ts"><ui:paste select="#completion-acceptance-browser"/></ui:file>

## Check browser intent and atomic draft application

Held resolvers keep the source readable and unchanged until complete. Apply primary/import edits with one undo, map the caret, reject source changes and composition, retire older choices and settle cancellation before a late reply.

<ui:copy id="source-completion-browser-witness">
import {expect,it} from 'vitest';
import {EditorSelection,EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {history,undo,redo} from '@codemirror/commands';
import {acceptSourceCompletion,sourceCompletion,type SourceCompletionAdapter} from './SourceCompletion';
import {readCompletionAcceptance,waitCompletionAcceptance,cancelCompletionAcceptance,withCompletionAcceptanceSignal} from '../lsp/completionAcceptance';
import type {CompletionItem} from '../lsp/client';
import {sourceDigest} from '../lsp/diagnosticRevision';
import {CompletionContext} from '@codemirror/autocomplete';
import {sourceSnippets,dispatchSourceSnippetTransactions,applyParallelSourceSnippet,readParallelSourceSnippet,readSourceEditorIntent} from './SourceSnippets';
import {sourceText,authoredSource,editorOffsetIn} from './SourceText';
import snippetVectors from '../../../simple-contract/snippet-expansion-cases.json';
function held(){let resolve!:(value:CompletionItem)=>void;const promise=new Promise<CompletionItem>(done=>{resolve=done;});return {promise,resolve};}
function fixture(source='va\n',caret=2){const parent=document.body.appendChild(document.createElement('div'));const view=new EditorView({dispatchTransactions:dispatchSourceSnippetTransactions,state:EditorState.create({doc:source,selection:{anchor:editorOffsetIn(source,caret)},extensions:[EditorState.allowMultipleSelections.of(true),history(),sourceText(source),sourceSnippets({writable:()=>true,notice:()=>{}})]}),parent});return {view,close:()=>{view.destroy();parent.remove();}};}
const raw={label:'value',data:{hickory_completion:'held'}};
function resolved(text='value'):CompletionItem {return {...raw,textEdit:{range:{start:{line:0,character:0},end:{line:0,character:2}},newText:text},additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:'# import\n'}]};}
it('waits for the whole resolved import edit, commits one undo and maps the caret',async()=>{
 const {view,close}=fixture(),pending=held(),notices:string[]=[];
 try{const work=acceptSourceCompletion(view,raw,'va\n',view.state.selection,0,2,{uri:'hick:///a.py',resolve:()=>pending.promise,current:()=>true,writable:()=>true,notice:value=>notices.push(value)});
  const before=readCompletionAcceptance(view)!;const finished=waitCompletionAcceptance(view,before.job);expect(before.phase).toBe('resolving');expect(view.state.doc.toString()).toBe('va\n');pending.resolve(resolved());await work;
  expect((await finished).phase).toBe('applied');expect(view.state.doc.toString()).toBe('# import\nvalue\n');expect(view.state.selection.main.head).toBe(14);expect(notices.at(-1)).toContain('draft');expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe('va\n');expect(redo(view)).toBe(true);expect(view.state.doc.toString()).toBe('# import\nvalue\n');
 }finally{close();}
});
it('restores snippet field selection with complete imports on redo',async()=>{
 const {view,close}=fixture();try{
  const resolved=snippetVectors.cases[0].item as CompletionItem;
  await acceptSourceCompletion(view,raw,'va\n',view.state.selection,0,2,{uri:'hick:///a.py',resolve:async()=>resolved,current:()=>true,writable:()=>true,notice:()=>{}});
  expect(view.state.doc.toString()).toBe('#\n😀 + 😀\n');expect(view.state.selection.main.anchor).toBe(2);expect(view.state.selection.main.head).toBe(4);
  expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe('va\n');expect(redo(view)).toBe(true);
  expect(view.state.doc.toString()).toBe('#\n😀 + 😀\n');expect(view.state.selection.main.anchor).toBe(2);expect(view.state.selection.main.head).toBe(4);
 }finally{close();}
});
it('accepts a source-equal completion without a fake source edit',async()=>{
 const {view,close}=fixture();try{await acceptSourceCompletion(view,raw,'va\n',view.state.selection,0,2,{uri:'hick:///a.py',resolve:async()=>({...resolved('va'),additionalTextEdits:[]}),current:()=>true,writable:()=>true,notice:()=>{}});expect(view.state.doc.toString()).toBe('va\n');expect(readCompletionAcceptance(view)).toMatchObject({phase:'applied',applied:true,source_changed:false});}finally{close();}
});
it('a newer choice owns the result and late older replies cannot overwrite it',async()=>{
 const {view,close}=fixture(),first=held(),second=held(),notices:string[]=[];let calls=0;
 const adapter:SourceCompletionAdapter={uri:'hick:///a.py',resolve:()=>++calls===1?first.promise:second.promise,current:()=>true,writable:()=>true,notice:value=>notices.push(value)};
 try{const selection=view.state.selection,a=acceptSourceCompletion(view,raw,'va\n',selection,0,2,adapter),old=waitCompletionAcceptance(view,readCompletionAcceptance(view)!.job),b=acceptSourceCompletion(view,raw,'va\n',selection,0,2,adapter);
  expect((await old).phase).toBe('unavailable');second.resolve(resolved('newer'));await b;const notice=notices.at(-1);first.resolve(resolved('older'));await a;expect(view.state.doc.toString()).toBe('# import\nnewer\n');expect(readCompletionAcceptance(view)?.phase).toBe('applied');expect(notices.at(-1)).toBe(notice);
 }finally{close();}
});
it('source edits and composition refuse both the primary and import portions',async()=>{
 const {view,close}=fixture(),pending=held();let calls=0;
 const adapter:SourceCompletionAdapter={uri:'hick:///a.py',resolve:()=>{calls++;return pending.promise;},current:()=>true,writable:()=>true,notice:()=>{}};
 try{const work=acceptSourceCompletion(view,raw,'va\n',view.state.selection,0,2,adapter);view.dispatch({changes:{from:0,insert:'# user\n'}});pending.resolve(resolved());await work;expect(view.state.doc.toString()).toBe('# user\nva\n');expect(readCompletionAcceptance(view)?.phase).toBe('unavailable');
  Object.defineProperty(view,'composing',{value:true,configurable:true});const before=view.state.doc.toString();await acceptSourceCompletion(view,raw,before,view.state.selection,0,2,adapter);expect(calls).toBe(1);expect(view.state.doc.toString()).toBe(before);
 }finally{close();}
});
it('retires a held peer completion after changing and restoring the secondary caret',async()=>{
 const source='va\nva\n',{view,close}=fixture(source),pending=held();
 try {
  const hash=await sourceDigest(source),context={source_hash:hash,source_revision:'a'.repeat(64)};
  const offer=(token:string):CompletionItem=>({label:'value',data:{hickory_completion:token,hickory_context:context}});
  const first=offer('first'),peer=offer('peer');
  const selections=(end:number)=>EditorSelection.create([EditorSelection.cursor(2),EditorSelection.cursor(end)]);
  const original=selections(5);
  view.dispatch({selection:original});
  let beginPeer!:()=>void;
  const resolvingPeer=new Promise<void>(resolve=>{beginPeer=resolve;});
  const work=acceptSourceCompletion(view,first,source,original,0,2,{
   uri:'hick:///a.py',current:()=>true,writable:()=>true,notice:()=>{},
   resolve:async()=>{throw new Error('Single-caret resolution was unexpected');},
   group:{query:async offset=>{expect(offset).toBe(5);return [peer];},resolve:async item=>{
    if((item.data as {hickory_completion:string}).hickory_completion==='first') return first;
    beginPeer();
    return pending.promise;
   }},
  });
  await resolvingPeer;
  view.dispatch({selection:selections(4)});
  view.dispatch({selection:original});
  pending.resolve(peer);
  await work;
  expect(authoredSource(view.state)).toBe(source);
  expect(view.state.selection.eq(original)).toBe(true);
  expect(readCompletionAcceptance(view)?.phase).toBe('unavailable');
  expect(undo(view)).toBe(false);
 } finally {close();}
});
it('reads the clipboard once after validating both grouped demands and shared edits',async()=>{
 const source='va\nva\n';
 const descriptor=Object.getOwnPropertyDescriptor(navigator,'clipboard');
 const phases:string[]=[];
 Object.defineProperty(navigator,'clipboard',{configurable:true,value:{readText:async()=>{phases.push('clipboard');return 'shared';}}});
 try {
  const hash=await sourceDigest(source),context={source_hash:hash,source_revision:'a'.repeat(64)};
  const offer=(token:string):CompletionItem=>({label:'value',data:{hickory_completion:token,hickory_context:context}});
  const syntax='${1:${CLIPBOARD}}$0';
  for(const conflict of [true,false]){
   const {view,close}=fixture(source);
   try {
    phases.length=0;
    const selection=EditorSelection.create([EditorSelection.cursor(2),EditorSelection.cursor(5)],1);
    view.dispatch({selection});
    await acceptSourceCompletion(view,offer('second'),source,selection,3,5,{
     uri:'hick:///a.py',current:()=>true,writable:()=>true,notice:()=>{},
     resolve:async()=>{throw new Error('Unexpected single resolution');},
     group:{query:async()=>[offer('first')],resolve:async item=>{
      const data=item.data as {hickory_completion:string;hickory_context:{selection:{from:number;to:number};carets:{index:number;count:number};clipboard:unknown}};
      const {index,count}=data.hickory_context.carets;
      expect(count).toBe(2);
      expect(data.hickory_context.selection).toEqual(index===0?{from:2,to:2}:{from:5,to:5});
      const retry=data.hickory_completion.endsWith('-retry');
      phases.push(`${retry?'retry':'first'}-${index}`);
      const text=index===0?'one':'two';
      const resultData={hickory_completion:retry?data.hickory_completion:data.hickory_completion+'-retry',hickory_context:context};
      if(retry) expect(data.hickory_context.clipboard).toEqual({available:true,text:'shared'});
      else expect(data.hickory_context.clipboard).toBeNull();
      return {
       label:item.label,insertTextFormat:2,
       textEdit:{range:{start:{line:index,character:0},end:{line:index,character:2}},newText:syntax},
       additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:conflict&&index?'# other\n':'# import\n'}],
       data:retry?{...resultData,hickory_snippet:{version:2,syntax,text,fields:[
        {index:1,from:0,to:3,choices:[],id:0,parent:null},{index:0,from:3,to:3,choices:[],id:1,parent:null},
       ]}}:{...resultData,hickory_context_request:{version:1,variables:['CLIPBOARD']}},
      };
     }},
    });
    expect(phases).toEqual(conflict?['first-1','first-0']:['first-1','first-0','clipboard','retry-1','retry-0']);
    expect(authoredSource(view.state)).toBe(conflict?source:'# import\none\ntwo\n');
    expect(readParallelSourceSnippet(view.state)?.peers.length ?? 0).toBe(conflict?0:2);
   } finally {close();}
  }
 } finally {
  if(descriptor)Object.defineProperty(navigator,'clipboard',descriptor);
  else Reflect.deleteProperty(navigator,'clipboard');
 }
});
it('retains the active template group when an ordinary completion is chosen',async()=>{
 const source='va\nva\n',{view,close}=fixture(source);
 try {
  const ranges=[{from:0,to:2},{from:3,to:5}];
  view.dispatch({selection:EditorSelection.create(ranges.map(range=>EditorSelection.range(range.from,range.to)))});
  const fields=[
   {index:1,from:0,to:5,choices:[],id:0,parent:null},
   {index:0,from:5,to:5,choices:[],id:1,parent:null},
  ];
  expect(applyParallelSourceSnippet(view,source,ranges.map(range=>({...range,text:'value',fields})),0,readSourceEditorIntent(view.state)!,()=>true)).toBe(true);
  const parallel=readParallelSourceSnippet(view.state),draft=authoredSource(view.state),selection=view.state.selection;
  let calls=0;
  const resolve=async()=>{calls++;return resolved();};
  await acceptSourceCompletion(view,raw,draft,selection,0,5,{
   uri:'hick:///a.py',current:()=>true,writable:()=>true,notice:()=>{},resolve,
   group:{query:async()=>{calls++;return [raw];},resolve},
  });
  expect(calls).toBe(0);
  expect(authoredSource(view.state)).toBe(draft);
  expect(readParallelSourceSnippet(view.state)).toBe(parallel);
  expect(view.state.selection.eq(selection)).toBe(true);
  expect(readCompletionAcceptance(view)?.message).toContain('Finish editing the template');
 } finally {close();}
});
it('Escape cancellation and an agent abort settle observers before the late provider reply',async()=>{
 for(const agent of[false,true]){const {view,close}=fixture(),pending=held(),signal=new AbortController();try{
  const work=withCompletionAcceptanceSignal(view,signal.signal,()=>acceptSourceCompletion(view,raw,'va\n',view.state.selection,0,2,{uri:'hick:///a.py',resolve:()=>pending.promise,current:()=>true,writable:()=>true,notice:()=>{}}));
  const finished=waitCompletionAcceptance(view,readCompletionAcceptance(view)!.job);if(agent)signal.abort();else expect(cancelCompletionAcceptance(view)).toBe(true);expect((await finished).phase).toBe('unavailable');pending.resolve(resolved());await work;expect(view.state.doc.toString()).toBe('va\n');
 }finally{close();}}
});

it('an offered query cannot revive after equal-byte undo or a cursor round trip',async()=>{
 for(const sourceABA of[true,false]){const {view,close}=fixture();let calls=0;try{const item=sourceCompletion(raw,new CompletionContext(view.state,2,true),{uri:'hick:///a.py',resolve:async()=>{calls++;return resolved()},current:()=>true,writable:()=>true,notice:()=>{}});if(sourceABA){view.dispatch({changes:{from:0,to:2,insert:'xy'},selection:{anchor:2},userEvent:'input.type'});expect(undo(view)).toBe(true)}else{view.dispatch({selection:{anchor:0}});view.dispatch({selection:{anchor:2}})}expect(view.state.doc.toString()).toBe('va\n');expect(view.state.selection.main.head).toBe(2);expect(typeof item.apply).toBe('function');(item.apply as Exclude<typeof item.apply,string|undefined>)(view,item,0,2);await waitCompletionAcceptance(view,readCompletionAcceptance(view)!.job);expect(calls).toBe(0);expect(view.state.doc.toString()).toBe('va\n');expect(readCompletionAcceptance(view)?.phase).toBe('unavailable')}finally{close()}}
});
it('a pending provider cannot revive after equal-byte undo or a cursor round trip',async()=>{
 for(const sourceABA of[true,false]){const {view,close}=fixture(),pending=held();try{const work=acceptSourceCompletion(view,raw,'va\n',view.state.selection,0,2,{uri:'hick:///a.py',resolve:()=>pending.promise,current:()=>true,writable:()=>true,notice:()=>{}});if(sourceABA){view.dispatch({changes:{from:0,to:2,insert:'xy'},selection:{anchor:2},userEvent:'input.type'});expect(undo(view)).toBe(true)}else{view.dispatch({selection:{anchor:0}});view.dispatch({selection:{anchor:2}})}pending.resolve(resolved());await work;expect(view.state.doc.toString()).toBe('va\n');expect(view.state.selection.main.head).toBe(2);expect(readCompletionAcceptance(view)?.phase).toBe('unavailable')}finally{close()}}
});

it('applies complete raw CRLF edits through a normalized editor and restores exact bytes with Undo and Redo',async()=>{
 const source='head\r\nva\r\nend\n',after='# imported\r\nhead\r\nλ😀\nnext\r\nend\n',{view,close}=fixture(source,8);
 try{const item=sourceCompletion(raw,new CompletionContext(view.state,7,true),{uri:'hick:///a.py',resolve:async()=>({...raw,textEdit:{range:{start:{line:1,character:0},end:{line:1,character:2}},newText:'λ😀\nnext'},additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:'# imported\r\n'}]}),current:()=>true,writable:()=>true,notice:()=>{}});
  (item.apply as Exclude<typeof item.apply,string|undefined>)(view,item,5,7);await waitCompletionAcceptance(view,readCompletionAcceptance(view)!.job);
  expect(readCompletionAcceptance(view)?.phase).toBe('applied');expect(authoredSource(view.state)).toBe(after);expect(view.state.doc.toString()).toBe(after.replace(/\r\n/g,'\n'));const caret=view.state.selection.main.head;
  expect(undo(view)).toBe(true);expect(authoredSource(view.state)).toBe(source);expect(view.state.selection.main.head).toBe(7);expect(redo(view)).toBe(true);expect(authoredSource(view.state)).toBe(after);expect(view.state.selection.main.head).toBe(caret);
 }finally{close()}
});
</ui:copy>

<ui:file path="apps/web/src/simple/SourceCompletion.test.ts"><ui:paste select="#source-completion-browser-witness"/></ui:file>

## Drive the real popup through the agent adapter

The agent opens and selects an actual CodeMirror popup choice. Its acceptance must remain pending while resolution is held and report completion only after the whole source/import draft is committed. No fabricated apply range or second completion query substitutes for the popup.

<ui:copy id="source-completion-agent-witness">
import {act,cleanup,render,waitFor} from '@testing-library/react';
import {expect,it} from 'vitest';
import {EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {completionStatus} from '@codemirror/autocomplete';
import {completions} from '../lsp/completion';
import {readCompletionAcceptance} from '../lsp/completionAcceptance';
import {sourceCompletion} from './SourceCompletion';
import type {CompletionItem} from '../lsp/client';
import {UiActions} from '../lib/uiActions';
import {UiActionContext} from '../lib/uiActionBridge';
import {useEditorActions} from '../editor/uiEditorActions';
it('agent native-popup acceptance completes only after resolution and the whole draft commit',async()=>{
 const registry=new UiActions({workspace:'p',window:'w'},()=>true,async()=>{}),parent=document.body.appendChild(document.createElement('div'));
 let resolve!:(item:CompletionItem)=>void;const held=new Promise<CompletionItem>(done=>{resolve=done;});
 const item={label:'value',data:{hickory_completion:'held'}};
 const view=new EditorView({state:EditorState.create({doc:'va\n',selection:{anchor:2},extensions:completions({lsp:{client:{completion:async()=>[item]} as never,uri:'hick:///a.py',positionAt:offset=>({line:0,character:offset})},lspItem:(item,context)=>sourceCompletion(item,context,{uri:'hick:///a.py',resolve:()=>held,current:()=>true,writable:()=>true,notice:()=>{}})})}),parent});
 function Harness(){useEditorActions(view,'source');return null;}
 try{
  render(<UiActionContext.Provider value={registry}><Harness/></UiActionContext.Provider>);
  const actions=()=>registry.groups().groups.flatMap(group=>registry.observe(group).actions);let id=0;
  const run=async(label:string,args={})=>{let reply:unknown;await act(async()=>{reply=await registry.act(registry.scope,{id:'python:turn',kind:'agent'},{handle:actions().find(item=>item.label===label)!.handle,arguments:args,request_id:String(++id)});});return reply as {outcome:{value:{snapshot:string}}};};
  await waitFor(()=>expect(actions().some(item=>item.label==='Open editor completions')).toBe(true));await run('Open editor completions');await waitFor(()=>expect(completionStatus(view.state)).toBe('active'));
  const observed=await run('Observe editor completion popup'),selected=await run('Select editor completion',{snapshot:observed.outcome.value.snapshot,index:0});await new Promise(done=>setTimeout(done,90));
  let completed=false;
  const accepting=registry.act(registry.scope,{id:'python:turn',kind:'agent'},{handle:actions().find(item=>item.label==='Accept selected editor completion')!.handle,arguments:{snapshot:selected.outcome.value.snapshot},request_id:String(++id)}).then(value=>{completed=true;return value;});
  await waitFor(()=>expect(readCompletionAcceptance(view)?.phase).toBe('resolving'));expect(completed).toBe(false);expect(view.state.doc.toString()).toBe('va\n');
  let reply:unknown;await act(async()=>{resolve({...item,textEdit:{range:{start:{line:0,character:0},end:{line:0,character:2}},newText:'value'},additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:'# import\n'}]});reply=await accepting;});
  expect(reply).toMatchObject({status:'result',outcome:{status:'completed',value:{accepted:true,source_changed:true,saved:false,verified:false,acceptance:{phase:'applied'}}}});expect(view.state.doc.toString()).toBe('# import\nvalue\n');expect(view.state.selection.main.head).toBe(14);
 }finally{cleanup();view.destroy();parent.remove();}
});
</ui:copy>

<ui:file path="apps/web/src/simple/SourceCompletion.agent.test.tsx"><ui:paste select="#source-completion-agent-witness"/></ui:file>

## Check native batching, undo and obsolete intents

The actual AppKit editor applies a compound completion with the intended caret and one undo/redo group. Independent controller witnesses hold competing replies and dismiss a pending choice; older work must not write source or replace the newer notice. These checks do not establish native popover pointer or IME acceptance.

The grouped snippet witness retains two different source selections through
resolution and supplies one clipboard refusal after both peers ask. Its two
Unicode field sets stay independently editable under linked input. A single
Undo restores the prepared selections, and Redo restores the group.
The source-equal case selects the fields and starts the same linked session.
The editor creates a text Undo entry only after actual linked typing changes
the draft.

For background reads, a null reply leaves the last rows readable. A held
error after source authority is lost leaves the newer status alone, while
a current provider failure still appears.

<ui:copy id="source-completion-apple-witness">
import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class CompletionAcceptanceTests:XCTestCase {
    func settle(_ condition:@escaping ()->Bool)async{let end=Date().addingTimeInterval(2);while !condition()&&Date()<end{try? await Task.sleep(for:.milliseconds(1))}}
    func edit(_ line:Int,_ from:Int,_ to:Int,_ text:String)->[String:Any]{["range":["start":["line":line,"character":from],"end":["line":line,"character":to]],"newText":text]}
    func testBackgroundCompletionKeepsRetainedRowsAndIgnoresObsoleteErrors() async throws {
        let file = FileNode(name: "a.py", path: "a.py", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: "va", hash: "base"))
        let language = NativeLanguage(engine: try Engine(address: "http://fixture.invalid"), buffer: buffer)
        buffer.language = language
        buffer.selection = NSRange(location: 2, length: 0)
        defer { language.close() }

        await language.complete(fetch: { _ in [["label": "value"]] })
        XCTAssertEqual(language.suggestions.map(\.label), ["value"])
        await language.complete(fetch: { _ in NSNull() })
        XCTAssertEqual(language.suggestions.map(\.label), ["value"])
        XCTAssertEqual(language.status, "Showing last received completions. Request completions again.")

        var held: CheckedContinuation<Any, Error>?
        let obsolete = Task {
            await language.complete(fetch: { _ in
                try await withCheckedThrowingContinuation { held = $0 }
            })
        }
        await settle { held != nil }
        let reply = try XCTUnwrap(held)
        buffer.resourceDeleted = true
        language.status = "Source is now read-only."
        reply.resume(throwing: EngineError(message: "obsolete completion transport"))
        await obsolete.value
        XCTAssertEqual(language.status, "Source is now read-only.")

        buffer.resourceDeleted = false
        await language.complete(fetch: { _ in throw EngineError(message: "fixture provider failure") })
        XCTAssertEqual(language.status, "Completion unavailable: fixture provider failure")
        XCTAssertEqual(buffer.text, "va")
        XCTAssertEqual(buffer.selection, NSRange(location: 2, length: 0))
    }

    func testActualMacEditorAcceptsImportsAndCaretInOneUndo()async throws {
        #if os(macOS)
        let source="header\nva\n",file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"base")),service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll
        defer{service.close();window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:9,length:0));buffer.selection=editor.text.selectedRange();let original:[String:Any]=["label":"value","data":["hickory_completion":"held"]]
        let item=NativeCompletion(id:1,label:"value",detail:"",range:NSRange(location:7,length:2),insert:"value",source:source,selection:buffer.selection,original:original)
        let accepted=await service.accept(item,resolution:{_ in ["label":"value","textEdit":self.edit(1,0,2,"value"),"additionalTextEdits":[self.edit(0,0,6,"longheader")]]})
        XCTAssertTrue(accepted);XCTAssertEqual(buffer.text,"longheader\nvalue\n");XCTAssertEqual(editor.text.selectedRange().location,16);XCTAssertEqual(buffer.base,source);XCTAssertEqual(buffer.hash,"base")
        buffer.saving=true;editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);buffer.saving=false
        editor.text.undoManager?.redo();XCTAssertEqual(buffer.text,"longheader\nvalue\n");XCTAssertEqual(editor.text.selectedRange().location,16)
        buffer.text="external";editor.text.string="external";editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,"external");XCTAssertEqual(editor.text.string,"external");XCTAssertTrue(service.completionNotice.contains("history retired"));XCTAssertFalse(editor.text.undoManager?.canUndo ?? false)
        #endif
    }
    private func caretOffer(_ source:String,token:String)->[String:Any] {
        ["label":"value","kind":6,"detail":"./dep",
         "data":["hickory_completion":token,"hickory_context":[
            "source_hash":"sha256:"+nativeSourceHash(source),"source_revision":String(repeating:"a",count:64)
         ]]]
    }

    private func resolvedCaretProof(_ offer:[String:Any]) throws -> [String:Any] {
        var resolved=offer
        var data=try XCTUnwrap(offer["data"] as? [String:Any])
        let context=try XCTUnwrap(data["hickory_context"] as? [String:Any])
        data["hickory_context"]=try XCTUnwrap(context["expected"] as? [String:String])
        data.removeValue(forKey:"hickory_context_request")
        resolved["data"]=data
        return resolved
    }

    func testActualMacMultiCaretCompletionSharesImportAndRestoresEveryCaret() async throws {
        #if os(macOS)
        let source="😀\nva\nva\n",file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer)
        buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=editor.scroll
        defer{service.close();window.contentView=nil}
        let before=[5,8].map{NSRange(location:$0,length:0)}
        XCTAssertTrue(buffer.selectMultiple(before,source:source))
        await service.complete(fetch:{_ in [self.caretOffer(source,token:"primary")]})
        let choice=try XCTUnwrap(service.suggestions.first)
        var queries:[Int]=[],tokens:[String]=[]
        let accepted=await service.accept(choice,fetch:{params in
            let position=try XCTUnwrap(params["position"] as? [String:Int])
            queries.append(try XCTUnwrap(position["line"]))
            XCTAssertEqual(position["character"],2)
            return [self.caretOffer(source,token:"peer")]
        },resolution:{offer in
            let data=try XCTUnwrap(offer["data"] as? [String:Any])
            tokens.append(try XCTUnwrap(data["hickory_completion"] as? String))
            var resolved=try self.resolvedCaretProof(offer)
            resolved["additionalTextEdits"]=[self.edit(0,0,0,"import x\n")]
            return resolved
        })
        let after="import x\n😀\nvalue\nvalue\n"
        let carets=[17,23].map{NSRange(location:$0,length:0)}
        XCTAssertTrue(accepted,service.completionNotice)
        XCTAssertEqual(queries,[2]);XCTAssertEqual(tokens,["primary","peer"])
        XCTAssertEqual(buffer.text,after);XCTAssertEqual(editor.text.string,after)
        XCTAssertEqual(buffer.multiRanges,carets);XCTAssertEqual(editor.text.selectedRange(),carets[0])
        XCTAssertEqual(buffer.base,source);XCTAssertEqual(buffer.hash,"base")
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.multiRanges,before)
        XCTAssertEqual(editor.text.selectedRange(),before[0])
        editor.text.undoManager?.redo()
        XCTAssertEqual(buffer.text,after);XCTAssertEqual(buffer.multiRanges,carets)
        XCTAssertEqual(editor.text.selectedRange(),carets[0])
        #endif
    }

    func testActualMacGroupedSnippetKeepsIndependentFieldsAndOneClipboardOutcome() async throws {
        #if os(macOS)
        let source="header\nva\nvo\n",file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer)
        buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=editor.scroll
        defer{service.close();window.contentView=nil}
        let before=[NSRange(location:7,length:2),NSRange(location:10,length:2)]
        XCTAssertTrue(buffer.selectMultiple(before,source:source))
        let syntax="${1:${TM_SELECTED_TEXT}}+$1!$0${CLIPBOARD:}"
        var reads=0,requests:[String]=[],randoms:[String]=[],clocks:[[String:Int64]]=[]
        service.snippetClipboardReader={reads+=1;return nil}
        await service.complete(fetch:{_ in [self.caretOffer(source,token:"primary")]})
        let choice=try XCTUnwrap(service.suggestions.first)
        let accepted=await service.accept(choice,fetch:{params in
            XCTAssertEqual(params["position"] as? [String:Int],["line":2,"character":0])
            return [self.caretOffer(source,token:"peer")]
        },resolution:{offer in
            let data=try XCTUnwrap(offer["data"] as? [String:Any])
            let token=try XCTUnwrap(data["hickory_completion"] as? String)
            let context=try XCTUnwrap(data["hickory_context"] as? [String:Any])
            let row=token.hasPrefix("primary") ? 0 : 1
            let original=before[row]
            XCTAssertEqual(context["selection"] as? [String:Int],["from":original.location,"to":NSMaxRange(original)])
            XCTAssertEqual(context["carets"] as? [String:Int],["index":row,"count":2])
            randoms.append(try XCTUnwrap(context["random_hex"] as? String))
            clocks.append(try XCTUnwrap(context["clock"] as? [String:Int64]))
            requests.append(token)
            var result=try self.resolvedCaretProof(offer)
            var reply=try XCTUnwrap(result["data"] as? [String:Any])
            result["insertTextFormat"]=2
            result["textEdit"]=self.edit(row+1,0,2,syntax)
            result["additionalTextEdits"]=[self.edit(0,0,0,"import x\n")]
            if !token.hasSuffix("-retry") {
                XCTAssertEqual(reads,0)
                XCTAssertTrue(context["clipboard"] is NSNull)
                reply["hickory_completion"]=token+"-retry"
                reply["hickory_context_request"]=["version":1,"variables":["CLIPBOARD"]]
            }else{
                let clipboard=try XCTUnwrap(context["clipboard"] as? [String:Any])
                XCTAssertEqual(clipboard["available"] as? Bool,false)
                XCTAssertTrue(clipboard["text"] is NSNull)
                let value=row==0 ? "😀" : "β",width=value.utf16.count,exit=2*width+2
                let fields:[[String:Any]]=[
                    ["index":1,"from":0,"to":width,"choices":[],"id":0,"parent":NSNull()],
                    ["index":1,"from":width+1,"to":2*width+1,"choices":[],"id":1,"parent":NSNull()],
                    ["index":0,"from":exit,"to":exit,"choices":[],"id":2,"parent":NSNull()],
                ]
                reply["hickory_context_unavailable"]=["CLIPBOARD"]
                reply["hickory_snippet"]=["version":1,"syntax":syntax,"text":"\(value)+\(value)!","fields":fields]
            }
            result["data"]=reply
            return result
        })
        let expanded="import x\nheader\n😀+😀!\nβ+β!\n"
        let initialFields=[NSRange(location:16,length:2),NSRange(location:23,length:1)]
        guard accepted else{XCTFail(service.completionNotice);return}
        XCTAssertTrue(service.completionNotice.contains("Clipboard unavailable"))
        XCTAssertEqual(reads,1)
        XCTAssertEqual(requests,["primary","peer","primary-retry","peer-retry"])
        XCTAssertEqual(Set(randoms).count,1);XCTAssertTrue(clocks.allSatisfy{$0==clocks.first})
        XCTAssertEqual(buffer.text,expanded);XCTAssertEqual(buffer.multiRanges,initialFields)
        XCTAssertEqual(buffer.snippets.parallel?.selections,initialFields)
        editor.text.insertText("Z",replacementRange:editor.text.selectedRange())
        let typed="import x\nheader\nZ+Z!\nZ+Z!\n"
        XCTAssertEqual(buffer.text,typed)
        XCTAssertTrue(buffer.snippets.move(1))
        XCTAssertEqual(buffer.multiRanges,[20,25].map{NSRange(location:$0,length:0)})
        let manager=try XCTUnwrap(editor.text.undoManager)
        manager.undo();XCTAssertEqual(buffer.text,expanded);XCTAssertEqual(buffer.multiRanges,initialFields)
        manager.undo();XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.multiRanges,before);XCTAssertNil(buffer.snippets.parallel)
        manager.redo();XCTAssertEqual(buffer.text,expanded);XCTAssertEqual(buffer.multiRanges,initialFields);XCTAssertNotNil(buffer.snippets.parallel)
        manager.redo();XCTAssertEqual(buffer.text,typed);XCTAssertNotNil(buffer.snippets.parallel)
        XCTAssertEqual(buffer.base,source);XCTAssertEqual(buffer.hash,"base")
        #endif
    }

    func testActualMacSourceEqualGroupedSnippetStartsFieldsWithoutTextUndo() async throws {
        #if os(macOS)
        let source="va\nva\n",file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer)
        buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=editor.scroll
        defer{service.close();window.contentView=nil}
        let before=[2,5].map{NSRange(location:$0,length:0)},fields=[0,3].map{NSRange(location:$0,length:2)}
        XCTAssertTrue(buffer.selectMultiple(before,source:source))
        let manager=try XCTUnwrap(editor.text.undoManager)
        manager.removeAllActions()
        await service.complete(fetch:{_ in [self.caretOffer(source,token:"primary")]})
        let choice=try XCTUnwrap(service.suggestions.first)
        let accepted=await service.accept(choice,fetch:{_ in [self.caretOffer(source,token:"peer")]},resolution:{offer in
            var reply=try self.resolvedCaretProof(offer)
            var data=try XCTUnwrap(reply["data"] as? [String:Any])
            let row=data["hickory_completion"] as? String=="primary" ? 0 : 1
            let syntax="${1:va}$0"
            let fields:[[String:Any]]=[
                ["index":1,"from":0,"to":2,"choices":[],"id":0,"parent":NSNull()],
                ["index":0,"from":2,"to":2,"choices":[],"id":1,"parent":NSNull()],
            ]
            data["hickory_snippet"]=["version":1,"syntax":syntax,"text":"va","fields":fields]
            reply["insertTextFormat"]=2;reply["textEdit"]=self.edit(row,0,2,syntax);reply["data"]=data
            return reply
        })
        guard accepted else{XCTFail(service.completionNotice);return}
        XCTAssertEqual(buffer.text,source);XCTAssertFalse(buffer.dirty);XCTAssertFalse(manager.canUndo)
        XCTAssertEqual(buffer.multiRanges,fields);XCTAssertEqual(buffer.snippets.parallel?.selections,fields)
        editor.text.insertText("edited",replacementRange:editor.text.selectedRange())
        XCTAssertEqual(buffer.text,"edited\nedited\n");XCTAssertTrue(manager.canUndo)
        manager.undo()
        XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.multiRanges,fields)
        XCTAssertEqual(buffer.snippets.parallel?.selections,fields);XCTAssertFalse(manager.canUndo)
        #endif
    }

    func testInvalidGroupedPeerPreventsClipboardAndCommit() async throws {
        let source="//\nva\nva\n",file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer)
        buffer.language=service
        defer{service.close()}
        let before=[5,8].map{NSRange(location:$0,length:0)},editorID=UUID()
        buffer.selection=before[0];buffer.multiSource=source;buffer.multiRanges=before
        buffer.sourceEditorState={ [weak buffer] in buffer.map {
            NativeSourceEditorState(identity:editorID,source:$0.text,selection:$0.selection,composing:false,editable:true,attached:true,selections:$0.multiRanges.count)
        }}
        var reads=0,commits=0
        service.snippetClipboardReader={reads+=1;return "private clipboard"}
        buffer.applyCompletionDraft={_,_ in commits+=1;return true}
        for mixed in [true,false] {
            await service.complete(fetch:{_ in [self.caretOffer(source,token:"primary")]})
            let choice=try XCTUnwrap(service.suggestions.first)
            var requests:[String]=[]
            let accepted=await service.accept(choice,fetch:{_ in [self.caretOffer(source,token:"peer")]},resolution:{offer in
                var reply=try self.resolvedCaretProof(offer)
                var data=try XCTUnwrap(reply["data"] as? [String:Any])
                let token=try XCTUnwrap(data["hickory_completion"] as? String)
                requests.append(token)
                if token=="peer",mixed{return reply}
                data["hickory_completion"]=token+"-retry"
                data["hickory_context_request"]=["version":1,"variables":["CLIPBOARD"]]
                reply["data"]=data
                reply["insertTextFormat"]=2
                reply["textEdit"]=self.edit(token=="primary" ? 1 : 2,0,2,"$1$0${CLIPBOARD:}")
                reply["additionalTextEdits"]=[self.edit(0,0,0,token=="primary" ? "import x\n" : "import y\n")]
                return reply
            })
            XCTAssertFalse(accepted)
            XCTAssertEqual(requests,["primary","peer"])
            XCTAssertEqual(reads,0);XCTAssertEqual(commits,0)
            XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.multiRanges,before)
        }
    }

    func testSecondaryCaretRoundTripAndAmbiguousPeerRefuseTheWholeCompletion() async throws {
        let source="va\nva\n",file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer)
        buffer.language=service
        defer{service.close()}
        let origin=[2,5].map{NSRange(location:$0,length:0)},editorID=UUID()
        buffer.selection=origin[0];buffer.multiSource=source;buffer.multiRanges=origin
        buffer.sourceEditorState={ [weak buffer] in buffer.map {
            NativeSourceEditorState(identity:editorID,source:$0.text,selection:$0.selection,composing:false,editable:true,attached:true,selections:$0.multiRanges.count)
        }}
        var commits=0
        buffer.applyCompletionDraft={_,_ in commits+=1;return true}
        for ambiguous in [true,false] {
            await service.complete(fetch:{_ in [self.caretOffer(source,token:"primary")]})
            let choice=try XCTUnwrap(service.suggestions.first)
            let accepted=await service.accept(choice,fetch:{_ in
                var offers=[self.caretOffer(source,token:"peer")]
                if ambiguous {offers.append(self.caretOffer(source,token:"competitor"))}
                return offers
            },resolution:{offer in
                if (offer["data"] as? [String:Any])?["hickory_completion"] as? String=="peer" {
                    buffer.multiRanges=[origin[0],NSRange(location:4,length:0)]
                    buffer.multiRanges=origin
                }
                return try self.resolvedCaretProof(offer)
            })
            XCTAssertFalse(accepted)
            XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.multiRanges,origin)
            XCTAssertEqual(commits,0)
        }
    }

    func testActualMacSnippetSelectionAndImportsUndoTogether()async throws {
        #if os(macOS)
        let apps=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let vectors=try JSONSerialization.jsonObject(with:Data(contentsOf:apps.appendingPathComponent("simple-contract/snippet-expansion-cases.json"))) as! [String:Any]
        let resolved=(vectors["cases"] as! [[String:Any]])[0]["item"] as! [String:Any]
        let source="va",file=FileNode(name:"a.py",path:"a.py",dir:false),buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"va",hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll
        defer{service.close();window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:2,length:0));buffer.selection=editor.text.selectedRange()
        let item=NativeCompletion(id:1,label:file.name,detail:"",range:NSRange(location:0,length:2),insert:"",source:source,selection:buffer.selection,original:resolved)
        let accepted=await service.accept(item,resolution:{_ in resolved});XCTAssertTrue(accepted)
        XCTAssertEqual(buffer.text,"#\n😀 + 😀");XCTAssertEqual(editor.text.selectedRange(),NSRange(location:2,length:2));XCTAssertEqual(buffer.base,source)
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);editor.text.undoManager?.redo();XCTAssertEqual(editor.text.selectedRange(),NSRange(location:2,length:2))
        #endif
    }
    func testNewerChoiceAndDismissalRetireHeldResolutionWithoutOverwritingSource()async throws {
        let source="va\n",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"va\n",hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service;buffer.selection=NSRange(location:2,length:0);defer{service.close()}
        var commits=0
        buffer.applyCompletionDraft={draft,initial in
            let edits=(draft.plan.patches.first?.edits ?? []).map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},before=draft.before,selected=NSRange(location:draft.selection.from,length:draft.selection.to-draft.selection.from)
            guard exactUTF8(buffer.text,before),buffer.selection==initial else{return false};commits+=1
            let text=NSMutableString(string:before);for edit in edits.sorted(by:{$0.range.location>$1.range.location}){text.replaceCharacters(in:edit.range,with:edit.text)};buffer.text=text as String;buffer.selection=selected;return true
        }
        var held:[String:CheckedContinuation<[String:Any],Error>]=[:]
        let resolve:([String:Any]) async throws->[String:Any]={item in try await withCheckedThrowingContinuation{held[item["label"] as! String]=$0}}
        func item(_ label:String)->NativeCompletion{NativeCompletion(id:1,label:label,detail:"",range:NSRange(location:0,length:2),insert:label,source:source,selection:NSRange(location:2,length:0),original:["label":label,"data":["hickory_completion":label]])}
        func result(_ label:String)->[String:Any]{["label":label,"textEdit":edit(0,0,2,label),"additionalTextEdits":[edit(0,0,0,"# import\n")]]}
        let first=Task{await service.accept(item("old"),resolution:resolve)};await settle{held["old"] != nil}
        let second=Task{await service.accept(item("new"),resolution:resolve)};await settle{held["new"] != nil}
        try XCTUnwrap(held["new"]).resume(returning:result("new"));let secondAccepted=await second.value;XCTAssertTrue(secondAccepted);let notice=service.completionNotice
        try XCTUnwrap(held["old"]).resume(returning:result("old"));let firstAccepted=await first.value;XCTAssertFalse(firstAccepted);XCTAssertEqual(commits,1);XCTAssertEqual(buffer.text,"# import\nnew\n");XCTAssertEqual(service.completionNotice,notice)
        buffer.text=source;buffer.selection=NSRange(location:2,length:0)
        let pending=Task{await service.accept(item("dismissed"),resolution:resolve)};await settle{held["dismissed"] != nil};service.cancelCompletion()
        try XCTUnwrap(held["dismissed"]).resume(returning:result("dismissed"));let pendingAccepted=await pending.value;XCTAssertFalse(pendingAccepted);XCTAssertEqual(buffer.text,source);XCTAssertEqual(commits,1)
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/CompletionAcceptanceTests.swift"><ui:paste select="#source-completion-apple-witness"/></ui:file>

## Check Compose source state and obsolete intents

The real TextFieldState batch applies primary/import edits and selection in one undo/redo operation. Held replies prove latest-choice ownership and dismissal refusal independently of rendering. Device pointer/IME acceptance remains a separate check.

The Compose witness offers independent proof tokens at two native ranges.
It checks one import, the full caret roster in both history directions, and
an unchanged draft when proofs, ranges, imports or composition disagree.

The grouped snippet case starts with lazy ordinary-looking offers. It
checks each ordinal and selection, the shared clock and random values, one
validated clipboard read, linked input, and restoration of the original
and expanded field rosters through native Undo and Redo. A mismatched
first reply leaves the clipboard unread.

<ui:copy id="source-completion-android-witness">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
import org.json.JSONArray
import androidx.compose.ui.text.TextRange
class CompletionAcceptanceTest {
 private fun edit(line:Int,from:Int,to:Int,text:String)=JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",line).put("character",from)).put("end",JSONObject().put("line",line).put("character",to))).put("newText",text)
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun completeImportAndCaretAreOneUndo()=runBlocking {
  val source="header\nva\n";val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(9)}
  try{val item=NativeLanguageCompletion("value","",NativeSourceEdit(7,9,"value"),source,buffer.state.selection,JSONObject().put("label","value").put("data",JSONObject().put("hickory_completion","held")))
   assertTrue(service.accept(item){JSONObject().put("label","value").put("textEdit",edit(1,0,2,"value")).put("additionalTextEdits",JSONArray().put(edit(0,0,6,"longheader")))})
   assertEquals("longheader\nvalue\n",buffer.text);assertEquals(16,buffer.state.selection.start);assertEquals(source,buffer.base);assertEquals("base",buffer.hash)
   buffer.state.undoState.undo();assertEquals(source,buffer.text);buffer.state.undoState.redo();assertEquals("longheader\nvalue\n",buffer.text);assertEquals(16,buffer.state.selection.start)
  }finally{service.close()}
 }
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun snippetFieldsAndImportsAreOneUndo()=runBlocking {
  val source="va";val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(2)}
  val resolved=JSONObject(javaClass.classLoader!!.getResource("snippet-expansion-cases.json")!!.readText()).getJSONArray("cases").getJSONObject(0).getJSONObject("item")
  try{val item=NativeLanguageCompletion("snippet","",NativeSourceEdit(0,2,""),source,buffer.state.selection,resolved)
   assertTrue(service.accept(item){resolved});assertEquals("#\n😀 + 😀",buffer.text);assertEquals(TextRange(2,4),buffer.state.selection);assertEquals(source,buffer.base)
   buffer.state.undoState.undo();assertEquals(source,buffer.text);buffer.state.undoState.redo();assertEquals(TextRange(2,4),buffer.state.selection)
  }finally{service.close()}
 }
 @Test fun newerChoiceAndDismissalRetireHeldResolution()=runBlocking {
  val source="va\n";val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(2)}
  val held=mutableMapOf<String,CompletableDeferred<JSONObject>>()
  val resolve:suspend (JSONObject)->JSONObject={item->val promise=CompletableDeferred<JSONObject>();held[item.getString("label")]=promise;promise.await()}
  fun item(label:String)=NativeLanguageCompletion(label,"",NativeSourceEdit(0,2,label),source,TextRange(2),JSONObject().put("label",label).put("data",JSONObject().put("hickory_completion",label)))
  fun result(label:String)=JSONObject().put("label",label).put("textEdit",edit(0,0,2,label)).put("additionalTextEdits",JSONArray().put(edit(0,0,0,"# import\n")))
  suspend fun settle(condition:()->Boolean){withTimeout(2000){while(!condition())yield()}}
  try{val first=async{service.accept(item("old"),resolve)};settle{held["old"]!=null};val second=async{service.accept(item("new"),resolve)};settle{held["new"]!=null}
   held.getValue("new").complete(result("new"));assertTrue(second.await());val notice=service.completionNotice;held.getValue("old").complete(result("old"));assertFalse(first.await());assertEquals("# import\nnew\n",buffer.text);assertEquals(notice,service.completionNotice)
   buffer.state.edit{replace(0,length,source);selection=TextRange(2)};val pending=async{service.accept(item("dismissed"),resolve)};settle{held["dismissed"]!=null};service.cancelCompletion();held.getValue("dismissed").complete(result("dismissed"));assertFalse(pending.await());assertEquals(source,buffer.text)
  }finally{service.close()}
 }
 private fun groupOffer(source:String,token:String)=JSONObject().put("label","value").put("kind",6).put("detail","fixture value")
  .put("data",JSONObject().put("hickory_completion",token).put("hickory_context",JSONObject()
   .put("source_hash","sha256:"+debugHash(source)).put("source_revision","a".repeat(64))))
 private fun groupItem(buffer:DocumentBuffer)=NativeLanguageCompletion("value","fixture value",NativeSourceEdit(0,2,"value"),
  buffer.text,buffer.state.selection,groupOffer(buffer.text,"primary"),buffer.sourceRevision,
  NativeCompletionGroupCapture(buffer.multiRanges,buffer.multiSelectionRevision,buffer.file,0,buffer.snippets.intent))
 private fun resolveGroupOffer(offer:JSONObject,importText:String="# import\n"):JSONObject {
  val data=offer.getJSONObject("data")
  val expected=data.getJSONObject("hickory_context").optJSONObject("expected")
  if(expected!=null)data.put("hickory_context",expected)
  return offer.put("textEdit",edit(if(offer.getJSONObject("data").getString("hickory_completion")=="primary")0 else 1,0,2,"value"))
   .put("additionalTextEdits",JSONArray().put(edit(0,0,0,importText)))
 }
 private fun roster(vararg pairs:Pair<Int,Int>)=pairs.map { (from,to)->NativeSourceEdit(from,to,"") }

 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun groupAdoptsTwoDistinctProofsAndRestoresBothCaretsWithOneHistoryStep()=runBlocking {
  val source="va\nva\n";val before=roster(0 to 2,3 to 5)
  val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"))
  val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service
  val originalEditor=buffer.state;val tokens=mutableListOf<String>()
  try {
   assertTrue(buffer.selectMultiple(before,source))
   assertTrue(service.accept(groupItem(buffer),resolution={offer->
    tokens+=offer.getJSONObject("data").getString("hickory_completion")
    assertEquals(source,buffer.text)
    resolveGroupOffer(offer)
   },query={position->
    assertEquals(5,position)
    listOf(groupOffer(source,"unrelated").put("detail","another match"),groupOffer(source,"peer"))
   }))
   assertEquals(listOf("primary","peer"),tokens)
   assertEquals("# import\nvalue\nvalue\n",buffer.text)
   val after=roster(14 to 14,20 to 20)
   assertEquals(after,buffer.multiRanges);assertEquals(TextRange(14),buffer.state.selection)
   assertTrue(buffer.snippets.undoSource())
   assertEquals(source,buffer.text);assertEquals(before,buffer.multiRanges);assertEquals(TextRange(0,2),buffer.state.selection)
   assertFalse(buffer.snippets.canUndo)
   assertTrue(buffer.snippets.redoSource())
   assertEquals("# import\nvalue\nvalue\n",buffer.text);assertEquals(after,buffer.multiRanges)
   assertEquals(TextRange(14),buffer.state.selection);assertSame(originalEditor,buffer.state)
   assertEquals(source,buffer.base);assertEquals("base",buffer.hash)
  } finally { service.close() }
 }

 @Test fun groupRefusesAmbiguousReusedAndRetiredProofsAndConflictingEdits()=runBlocking {
  val source="va\nva\n";val initial=roster(0 to 2,3 to 5)
  for (failure in listOf("ambiguous","reused","roster round trip","imports","wrong range","snippet","composition","revision")) {
   val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"))
   val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service
   try {
    assertTrue(buffer.selectMultiple(initial,source))
    val accepted=service.accept(groupItem(buffer),resolution={offer->
     val token=offer.getJSONObject("data").getString("hickory_completion")
     val output=resolveGroupOffer(offer,if(token=="peer"&&failure=="imports")"# different\n" else "# import\n")
     if(token=="peer"&&failure=="wrong range")output.put("textEdit",edit(1,1,2,"value"))
     if(token=="peer"&&failure=="snippet")output.put("insertTextFormat",2)
     if(token=="peer"&&failure=="composition")buffer.snippets.enteringMarkedEvent()
     output
    },query={
     val peer=groupOffer(source,if(failure=="reused")"primary" else "peer")
     if(failure=="roster round trip"){
      assertTrue(buffer.selectMultiple(roster(0 to 2,4 to 5),source))
      assertTrue(buffer.selectMultiple(initial,source))
     }
     if(failure=="revision")peer.getJSONObject("data").getJSONObject("hickory_context").put("source_revision","b".repeat(64))
     if(failure=="ambiguous")listOf(peer,groupOffer(source,"ambiguous")) else listOf(peer)
    })
    assertFalse(failure,accepted);assertEquals(failure,source,buffer.text)
    assertEquals(failure,initial,buffer.multiRanges);assertEquals(failure,TextRange(0,2),buffer.state.selection)
    assertFalse(failure,buffer.snippets.canUndo)
   } finally {
    if(buffer.snippets.inMarkedInput)buffer.snippets.leavingMarkedEvent()
    service.close()
   }
  }
 }

 private val sharedSnippetSyntax="${'$'}{1:x} + ${'$'}1 / ${'$'}CURSOR_NUMBER / ${'$'}CLIPBOARD${'$'}0"

 private fun groupSnippetReply(source:String,token:String,index:Int,clip:String?):JSONObject {
  val indent=" ".repeat(index)
  val syntax=indent+sharedSnippetSyntax
  val item=groupOffer(source,token).put("insertTextFormat",2)
   .put("textEdit",edit(index,0,2,syntax))
   .put("additionalTextEdits",JSONArray().put(edit(0,0,0,"# import\n")))
  val data=item.getJSONObject("data")
  if(clip==null){
   data.put("hickory_context_request",JSONObject().put("version",1).put("variables",JSONArray().put("CLIPBOARD")))
   return item
  }
  val expanded=indent+"x + x / ${index+1} / $clip"
  fun field(index:Int,from:Int,to:Int,id:Int)=JSONObject().put("index",index).put("from",from)
   .put("to",to).put("id",id).put("parent",JSONObject.NULL).put("choices",JSONArray())
  data.put("hickory_snippet",JSONObject().put("version",1).put("syntax",syntax)
   .put("text",expanded).put("fields",JSONArray().put(field(1,index,index+1,0)).put(field(1,index+4,index+5,1))
    .put(field(0,expanded.length,expanded.length,2))))
  return item
 }

 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun groupedLazySnippetsShareOneContextAndClipboardWithExactNativeHistory()=runBlocking {
  val source="va\nva\n";val initial=roster(0 to 2,3 to 5)
  val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"))
  val engine=Engine("http://127.0.0.1:1")
  var clipboardReads=0
  engine.snippetClipboardReader={clipboardReads++;"clipboard"}
  val service=NativeLanguage(engine,buffer,this);buffer.language=service
  val captured=mutableListOf<JSONObject>()
  try{
   assertTrue(buffer.selectMultiple(initial,source))
   assertTrue(service.accept(groupItem(buffer),query={ offset ->
    assertEquals(5,offset)
    assertEquals("Resolve the current offer before querying the next caret",1,captured.size)
    listOf(groupOffer(source,"peer"))
   },resolution={item->
    assertEquals(source,buffer.text)
    val data=item.getJSONObject("data");val ctx=data.getJSONObject("hickory_context")
    captured.add(ctx)
    val ordinal=ctx.getJSONObject("carets")
    val index=ordinal.getInt("index")
    val selection=ctx.getJSONObject("selection")
    assertEquals(2,ordinal.getInt("count"))
    assertEquals(initial[index].from,selection.getInt("from"));assertEquals(initial[index].to,selection.getInt("to"))
    val token=data.getString("hickory_completion")
    val expected=if(index==0)"primary" else "peer"
    if(ctx.isNull("clipboard")){
     assertEquals(0,clipboardReads);assertEquals(expected,token)
     groupSnippetReply(source,"$expected-negotiation",index,null)
    }else{
     assertEquals(1,clipboardReads);assertEquals("$expected-negotiation",token)
     assertTrue(ctx.getJSONObject("clipboard").getBoolean("available"))
     groupSnippetReply(source,token,index,ctx.getJSONObject("clipboard").getString("text"))
    }
   }))
   assertEquals(1,clipboardReads)
   assertEquals(listOf(0,1,0,1),captured.map {it.getJSONObject("carets").getInt("index")})
   assertEquals(1,captured.map {it.getJSONObject("clock").toString()}.distinct().size)
   assertEquals(1,captured.map {it.getString("random_hex")}.distinct().size)
   val draft="# import\nx + x / 1 / clipboard\n x + x / 2 / clipboard\n"
   assertEquals(draft,buffer.text)
   val next=draft.indexOf('\n',9)+1
   val active=roster(9 to 10,next+1 to next+2)
   assertEquals(active,buffer.multiRanges);assertNotNull(buffer.snippets.parallel)
   assertTrue(buffer.snippets.inputParallel("done"))
   assertEquals("# import\ndone + done / 1 / clipboard\n done + done / 2 / clipboard\n",buffer.text)
   assertTrue(buffer.snippets.undoSource());assertEquals(draft,buffer.text);assertEquals(active,buffer.multiRanges)
   assertTrue(buffer.snippets.undoSource());assertEquals(source,buffer.text);assertEquals(initial,buffer.multiRanges)
   assertNull(buffer.snippets.parallel);assertFalse(buffer.snippets.canUndo)
   assertTrue(buffer.snippets.redoSource());assertEquals(draft,buffer.text);assertEquals(active,buffer.multiRanges)
   assertNotNull(buffer.snippets.parallel);assertEquals(source,buffer.base)
  }finally{service.close()}
 }

 @Test fun groupedClipboardStaysUnreadIfAnyFirstReplyCannotEnterTheGroup()=runBlocking {
  val source="va\nva\n";val initial=roster(0 to 2,3 to 5)
  for(failure in listOf("mixed","reused","malformed","conflicting imports","overlap")){
   val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"))
   val engine=Engine("http://127.0.0.1:1")
   var clipboardReads=0
   engine.snippetClipboardReader={clipboardReads++;"private"}
   val service=NativeLanguage(engine,buffer,this);buffer.language=service
   try{
    assertTrue(buffer.selectMultiple(initial,source))
    assertFalse(service.accept(groupItem(buffer),query={ listOf(groupOffer(source,"peer")) },resolution={item->
     val data=item.getJSONObject("data")
     val token=data.getString("hickory_completion")
     if(token=="primary")groupSnippetReply(source,"rotated",0,null).apply {
      if(failure=="overlap")put("additionalTextEdits",JSONArray().put(edit(1,0,2,"replacement")))
     }
     else when(failure){
      "mixed" -> resolveGroupOffer(groupOffer(source,token))
      "reused" -> groupSnippetReply(source,"rotated",1,null)
      "conflicting imports" -> groupSnippetReply(source,"different",1,null).put("additionalTextEdits",JSONArray().put(edit(0,0,0,"# other\n")))
      "overlap" -> groupSnippetReply(source,"different",1,null).put("additionalTextEdits",JSONArray().put(edit(1,0,2,"replacement")))
      else -> groupSnippetReply(source,"different",1,null).apply {
       getJSONObject("data").getJSONObject("hickory_context_request").put("variables",JSONArray().put("wrong"))
      }
     }
    }))
    assertEquals(failure,0,clipboardReads);assertEquals(failure,source,buffer.text)
    assertEquals(failure,initial,buffer.multiRanges);assertNull(buffer.snippets.parallel)
    assertFalse(failure,buffer.snippets.canUndo)
   } finally {service.close()}
  }
 }

 @Test fun groupedClipboardRetryMustPreserveEachProjectedEditRole()=runBlocking {
  val source="va!\nva!\n";val initial=roster(0 to 2,4 to 6)
  for(failure in listOf("syntax","range","imports")){
   val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"))
   val engine=Engine("http://127.0.0.1:1")
   var clipboardReads=0
   engine.snippetClipboardReader={clipboardReads++;"clipboard"}
   val service=NativeLanguage(engine,buffer,this);buffer.language=service
   try{
    assertTrue(buffer.selectMultiple(initial,source))
    assertFalse(service.accept(groupItem(buffer),query={listOf(groupOffer(source,"peer"))},resolution={item->
     val data=item.getJSONObject("data");val ctx=data.getJSONObject("hickory_context")
     val index=ctx.getJSONObject("carets").getInt("index")
     val token=data.getString("hickory_completion")
     if(ctx.isNull("clipboard"))groupSnippetReply(source,"$token-rotation",index,null)
     else groupSnippetReply(source,token,index,"clipboard").apply {
      val textEdit=getJSONObject("textEdit")
      when(failure){
       "syntax" -> if(index==0){
        val syntax=textEdit.getString("newText")+"!"
        textEdit.put("newText",syntax)
        getJSONObject("data").getJSONObject("hickory_snippet").put("syntax",syntax)
       }
       "range" -> if(index==0)textEdit.getJSONObject("range").getJSONObject("end").put("character",3)
       "imports" -> put("additionalTextEdits",JSONArray().put(edit(0,0,0,"# replaced\n")))
      }
     }
    }))
    assertTrue(service.completionNotice,service.completionNotice.contains("edits changed"))
    assertEquals(failure,1,clipboardReads);assertEquals(failure,source,buffer.text)
    assertEquals(failure,initial,buffer.multiRanges);assertNull(buffer.snippets.parallel)
    assertFalse(failure,buffer.snippets.canUndo)
   }finally{service.close()}
  }
 }

 @Test fun sourceEqualGroupSelectsCaretsOrFieldsWithoutTextHistory()=runBlocking {
  val source="va\nva\n";val initial=roster(0 to 2,3 to 5)
  for(snippet in listOf(false,true)){
   val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"))
   val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service
   try{
    assertTrue(buffer.selectMultiple(initial,source))
    assertTrue(service.accept(groupItem(buffer),query={listOf(groupOffer(source,"peer"))},resolution={item->
     val token=item.getJSONObject("data").getString("hickory_completion")
     val row=if(token=="primary")0 else 1
     val syntax=if(snippet)"${'$'}{1:va}${'$'}0" else "va"
     val result=groupOffer(source,token).put("textEdit",edit(row,0,2,syntax))
     if(snippet){
      fun field(index:Int,from:Int,to:Int,id:Int)=JSONObject().put("index",index).put("from",from)
       .put("to",to).put("id",id).put("parent",JSONObject.NULL).put("choices",JSONArray())
      result.put("insertTextFormat",2).getJSONObject("data").put("hickory_snippet",
       JSONObject().put("version",1).put("syntax",syntax).put("text","va")
        .put("fields",JSONArray().put(field(1,0,2,0)).put(field(0,2,2,1))))
     }
     result
    }))
    assertEquals(source,buffer.text);assertEquals("Completion selected",service.completionNotice)
    assertEquals(if(snippet)initial else roster(2 to 2,5 to 5),buffer.multiRanges)
    assertEquals(snippet,buffer.snippets.parallel!=null);assertFalse(buffer.snippets.canUndo)
    assertFalse(buffer.snippets.canRedo)
   }finally{service.close()}
  }
 }

}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/CompletionAcceptanceTest.kt"><ui:paste select="#source-completion-android-witness"/></ui:file>

## Resolve through the actual engine and existing popup

A controlled lazy child supplies standards-expanded completions and an import edit. The browser opens the real native popup, chooses its rendered item, checks the complete draft and exact caret, then uses one ordinary undo/redo. Retained editor identity, geometry and unchanged disk distinguish the interaction from a reconstructed editor or a hidden save. This is a protocol/UI witness, not a claim about the child's language analysis.

The installed TypeScript language server mode starts with an unresolved
identifier and an unopened module that exports the matching function. Select
the module's visible completion to check that its resolved import and primary
replacement enter the draft together between one Undo and Redo. Set
`HICKORY_SIMPLE_TEST_TYPESCRIPT_PROVIDER` to the installed server entry point
and run `python3 scripts/test-source-completion-browser.py --typescript`.
Use `--typescript-group` to offer the same import independently at two
carets and check their one-step source and selection history.

Build the current engine with
`cargo build --locked -p hickory-cli --example engine_client`.
Then run `python3 scripts/test-source-completion-browser.py --snippet-group`
to check two ordinal expansions, one shared import, linked typing and
restored fields through the existing popup and the controlled child.

<ui:copy id="source-completion-browser-engine-witness">
"""Exercise the completion popup through the actual engine and child providers."""
import importlib.util
import json
import os
from pathlib import Path
import re
import socket
import sys
import tempfile
import time
ROOT=Path(__file__).resolve().parents[1]

TYPESCRIPT_PATH = "main.ts"
TYPESCRIPT_FRAGMENT = "export const result = hickUniqueAn\n"
TYPESCRIPT_GROUP_FRAGMENT = TYPESCRIPT_FRAGMENT + "export const second = hickUniqueAn\n"
TYPESCRIPT_MODULE = "export function hickUniqueAnswer() { return 42; }\n"
TYPESCRIPT_COMPLETION = (
    'import { hickUniqueAnswer } from "./answers";\n\n'
    "export const result = hickUniqueAnswer\n"
)


def typescript_fixture(root, grouped):
    provider = Path(os.environ["HICKORY_SIMPLE_TEST_TYPESCRIPT_PROVIDER"]).resolve(strict=True)
    if not provider.is_file():
        raise RuntimeError("The installed TypeScript language server entry point must be a file.")
    fragment = TYPESCRIPT_GROUP_FRAGMENT if grouped else TYPESCRIPT_FRAGMENT
    saved = {TYPESCRIPT_PATH: fragment, "answers.ts": TYPESCRIPT_MODULE}
    for path, source in saved.items():
        (root / path).write_bytes(source.encode())
    (root / "tsconfig.json").write_text(json.dumps({
        "compilerOptions": {"target": "ES2022"}, "include": ["*.ts"],
    }))
    return saved, {"typescript": {"command": ["node", str(provider), "--stdio"]}}


def typescript_acceptance(driver, root, saved, size, grouped):
    def carets(source, identifier):
        return [{"from": match.end(), "to": match.end()} for match in re.finditer(rf"\b{identifier}$", source, re.MULTILINE)]

    path = TYPESCRIPT_PATH
    before = saved[path]
    after = TYPESCRIPT_COMPLETION
    if grouped:
        after = after.replace(";", "", 1) + "export const second = hickUniqueAnswer\n"
    before_carets = carets(before, "hickUniqueAn")
    after_carets = carets(after, "hickUniqueAnswer")
    primary = 1 if grouped else 0
    if grouped:
        assert driver.act("editor-selections", path=path, ranges=before_carets, primary=primary) == {
            "ranges": before_carets, "primary": primary,
        }
    assert not any("answers.ts" in label for label in driver.act("tabs"))
    deadline = time.monotonic() + 20
    while True:
        choices = driver.act("completion-open", path=path, offset=before_carets[primary]["to"], keepSelection=grouped)
        if any("hickUniqueAnswer" in choice and "./answers" in choice for choice in choices):
            break
        assert time.monotonic() < deadline, choices
    assert driver.act("search-source", path=path) == before
    notice = driver.act("completion-choose", title="hickUniqueAnswer")
    assert "Completion applied" in notice, notice
    assert driver.act("search-source", path=path) == after
    assert driver.act("editor-selections", path=path) == {"ranges": after_carets, "primary": primary}
    driver.act("undo", path=path)
    assert driver.act("search-source", path=path) == before
    assert driver.act("editor-selections", path=path) == {"ranges": before_carets, "primary": primary}
    driver.act("redo", path=path)
    assert driver.act("search-source", path=path) == after
    assert driver.act("editor-selections", path=path) == {"ranges": after_carets, "primary": primary}
    assert driver.act("editor-is-retained", path=path)
    assert driver.act("editor-size", path=path) == size
    assert not any("answers.ts" in label for label in driver.act("tabs"))
    for name, source in saved.items():
        assert (root / name).read_bytes() == source.encode()
    print("PASS: installed TypeScript provider resolved one shared import, with one Undo/Redo and unchanged saved bytes")


def grouped_snippet_acceptance(driver, root, path, source, size, events):
    original = {"ranges": [{"from": 3, "to": 3}, {"from": 13, "to": 13}], "primary": 1}
    driver.act("editor-selections", path=path, **original)
    deadline = time.monotonic() + 20
    while True:
        choices = driver.act("completion-open", path=path, keepSelection=True)
        if any("python completion" in choice for choice in choices):
            break
        assert time.monotonic() < deadline, choices
    notice = driver.act("completion-choose", title="python completion")
    assert "Completion applied" in notice, notice
    after = "1:😀 + 1:😀 = 1\n2:😀 + 2:😀 = 2\n# imported\n"
    fields = {"ranges": [{"from": 0, "to": 4}, {"from": 16, "to": 20}], "primary": 1}
    assert driver.act("source-text-state", path=path)["source"] == after
    assert driver.act("editor-selections", path=path) == fields
    driver.act("undo", path=path)
    assert driver.act("source-text-state", path=path)["source"] == source
    assert driver.act("editor-selections", path=path) == original
    driver.act("redo", path=path)
    assert driver.act("source-text-state", path=path)["source"] == after
    assert driver.act("editor-selections", path=path) == fields
    driver.act("type-source-selection", path=path, text="linked")
    assert driver.act("source-text-state", path=path)["source"] == "linked + linked = 1\nlinked + linked = 2\n# imported\n"
    assert driver.act("editor-is-retained", path=path)
    assert driver.act("editor-size", path=path) == size
    assert (root / path).read_bytes() == source.encode()
    observed = [json.loads(line) for line in events.read_text().splitlines()]
    assert sum("completion_resolved" in row for row in observed) == 2, observed
    print("PASS: grouped snippets retained both ordinal fields and mirrors through one Undo/Redo with one shared import")


def main():
    spec=importlib.util.spec_from_file_location("simple_fixture",ROOT/"scripts/test-simple-browser.py")
    fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-source-completion-") as temporary:
        folder = Path(temporary)
        root = folder / "workspace"
        root.mkdir()
        grouped = "--typescript-group" in sys.argv
        snippet_group = "--snippet-group" in sys.argv
        typescript = grouped or "--typescript" in sys.argv
        if typescript:
            path = TYPESCRIPT_PATH
            saved, servers = typescript_fixture(root, grouped)
        else:
            nested = "--nested" in sys.argv
            snippet = "--snippet" in sys.argv or nested
            path = "grouped_snippet.py" if snippet_group else "nested.py" if nested else "snippet.py" if snippet else "a.py"
            source = "value = 1\nvalue = 2\n" if snippet_group else "value = 1\n"
            (root / path).write_text(source)
            events = folder / "events.jsonl"
            gate = folder / "release"
            command = [sys.executable, str(ROOT / "scripts/controlled-action-provider.py"), "python", str(events), str(gate)]
            servers = {"python": {"command": command}}
        (root / ".hick-lsp.json").write_text(json.dumps({"servers": servers}))
        environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
        try:
            with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
            ui=f"http://127.0.0.1:{port}"
            engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
            vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
            driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path);deadline=time.monotonic()+20
            if typescript:
                typescript_acceptance(driver, root, saved, size, grouped)
                return
            if snippet_group:
                grouped_snippet_acceptance(driver, root, path, source, size, events)
                return
            while True:
                options=driver.act("completion-open",path=path,offset=3)
                if any("python completion" in value for value in options):break
                assert time.monotonic()<deadline,options;time.sleep(.1)
            assert driver.act("editor-size",path=path)==size;assert driver.act("search-source",path=path)==source
            notice=driver.act("completion-choose",title="python completion");assert "Completion applied" in notice,notice
            after=("value + value = 1\n# imported\n" if nested else "😀 value + 😀 value = 1\n# imported\n" if snippet else "python_completed = 1\n# imported\n");caret=0 if snippet else 16;assert driver.act("search-source",path=path)==after,{"expected":after,"actual":driver.act("search-source",path=path)};assert driver.act("caret-offset",path=path)==caret,{"caret":driver.act("caret-offset",path=path)}
            if snippet:assert driver.act("selected-source-text",path=path)==("value" if nested else "😀 value")
            assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size;assert (root/path).read_text()==source
            driver.act("undo",path=path);assert driver.act("search-source",path=path)==source
            driver.act("redo",path=path);assert driver.act("search-source",path=path)==after,{"redo_source":driver.act("search-source",path=path)};assert driver.act("caret-offset",path=path)==caret,{"redo_caret":driver.act("caret-offset",path=path),"selected":driver.act("selected-source-text",path=path)};assert (root/path).read_text()==source
            initial_after=after
            if nested:
                options=driver.act("completion-open",path=path,offset=3);assert any("python completion" in value for value in options),options
                notice=driver.act("completion-choose",title="python completion");assert "Completion applied" in notice,notice
                after="value + value + value + value = 1\n# imported\n# imported\n";assert driver.act("search-source",path=path)==after;assert driver.act("selected-source-text",path=path)=="value"
                driver.act("undo",path=path);assert driver.act("search-source",path=path)==initial_after
                driver.act("redo",path=path);assert driver.act("search-source",path=path)==after;assert driver.act("selected-source-text",path=path)=="value"
            if snippet:
                assert driver.act("selected-source-text",path=path)==("value" if nested else "😀 value")
                driver.act("type-source-selection",path=path,text="renamed")
                typed="renamed + renamed + renamed + renamed = 1\n# imported\n# imported\n" if nested else "renamed + renamed = 1\n# imported\n";assert driver.act("search-source",path=path)==typed;assert driver.act("caret-offset",path=path)==7
                driver.act("undo",path=path);assert driver.act("search-source",path=path)==after
                driver.act("redo",path=path);assert driver.act("search-source",path=path)==typed;assert driver.act("caret-offset",path=path)==7
                driver.act("template-next",path=path);assert driver.act("caret-offset",path=path)==17
                if nested:driver.act("template-next",path=path);assert driver.act("caret-offset",path=path)==37
                assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size;assert (root/path).read_text()==source
                driver.act("undo",path=path);assert driver.act("search-source",path=path)==after
            if nested:driver.act("undo",path=path);assert driver.act("search-source",path=path)==initial_after
            driver.act("undo",path=path);assert driver.act("search-source",path=path)==source
            options=driver.act("completion-open",path=path,offset=2);assert any("python completion" in value for value in options),options
            driver.act("completion-pick",title="python completion");deadline=time.monotonic()+3
            while True:
                observed=[json.loads(line) for line in events.read_text().splitlines()]
                if any(item.get("slow_completion") for item in observed):break
                assert time.monotonic()<deadline,observed;time.sleep(.01)
            driver.act("completion-cancel",path=path);gate.write_text("release");deadline=time.monotonic()+3
            while True:
                observed=[json.loads(line) for line in events.read_text().splitlines()]
                if any("completion_released" in item for item in observed):break
                assert time.monotonic()<deadline,observed;time.sleep(.01)
            driver.act("completion-drain");assert driver.act("search-source",path=path)==source;assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size;assert (root/path).read_text()==source
            observed=[json.loads(line) for line in events.read_text().splitlines()];assert [item["completion_resolved"] for item in observed if "completion_resolved" in item]==(["python"]*3 if nested else ["python","python"]),observed
            print("PASS: native completion popup, actual lazy resolution, primary and import edits, "+("nested snippet mirrors, outer-scope return, imports and retained field history, " if nested else "expanded snippet, linked typing, menu traversal and retained field history, " if snippet else "precise caret, ")+"one undo/redo, Escape refusal of a held late reply, retained editor geometry and unchanged disk")
        finally:
            if driver:driver.close()
            for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
</ui:copy>

<ui:file path="scripts/test-source-completion-browser.py"><ui:paste select="#source-completion-browser-engine-witness"/></ui:file>

## Replay a native source draft and its selection as one operation

Cocoa's default undo records each replacement's caret independently. An import applied after the primary completion therefore placed redo's caret at the import. This platform bridge derives forward/inverse edits from the shared change plan, suppresses nested native registrations and records one source-and-selection step. Consecutive explicit actions close the previous implicit event group and replace that event frame before registering the next step. Keeping the replacement frame open preserves Cocoa’s pending end-of-event grouping; nested caller groups and undo/redo replay retain their existing ownership. A typed context hook retains optional feature state with each direction; the history mechanism has no dependency on template rendering or field types. Sparse edit records use exact UTF-8 SHA-256 revision identities rather than retaining a complete file per keystroke. Optional context is held under bounded tokens, so an evicted field context cannot prevent source undo. The native editor still checks source and write permissions before application. Replay checks exact source and current editor permissions before applying; a mismatch retires incompatible native history while retaining the current draft. Marked input can also register a transition already reflected by the native editor. Observe its exact final bytes and independently reconstruct the forward patch from the original source before accepting its inverse. This preserves earlier native history while keeping provisional marked updates out of the source undo stack. Other editing paths keep their existing history behavior. Source snapshots are retained by native undo entries rather than discarded on parser or provider attachment.

<ui:copy id="native-source-draft-history">
import Foundation
import CryptoKit
@MainActor final class SourceDraftHistory {
    private struct ContextEntry {var cost:Int;var restore:()->Void}
    private static var contexts:[UUID:ContextEntry]=[:]
    private static var contextOrder:[UUID]=[]
    private static var contextBytes=0
    private static func hold<Context>(_ value:Context,cost:Int,restore:@escaping (Context)->Void)->UUID {
        let id=UUID(),weight=max(0,cost)
        guard weight<=32*1024*1024 else{return id}
        contexts[id]=ContextEntry(cost:weight,restore:{restore(value)});contextOrder.append(id);contextBytes+=weight
        while contextBytes>32*1024*1024 || contextOrder.count>4096 {
            let old=contextOrder.removeFirst();if let value=contexts.removeValue(forKey:old){contextBytes-=value.cost}
        }
        return id
    }
    private var retirementObservers:[ObjectIdentifier:NSObjectProtocol]=[:]
    private func retire(_ manager:UndoManager) {
        guard manager.isUndoing || manager.isRedoing else{manager.removeAllActions();return}
        let key=ObjectIdentifier(manager);guard retirementObservers[key]==nil else{return}
        let event=manager.isUndoing ? NSNotification.Name.NSUndoManagerDidUndoChange : NSNotification.Name.NSUndoManagerDidRedoChange
        retirementObservers[key]=NotificationCenter.default.addObserver(forName:event,object:manager,queue:nil){[weak self,weak manager] _ in
            MainActor.assumeIsolated {
                guard let self else{return}
                if let token=self.retirementObservers.removeValue(forKey:key){NotificationCenter.default.removeObserver(token)}
                manager?.removeAllActions()
            }
        }
    }
    private struct Step {
        var before:SHA256.Digest;var after:SHA256.Digest;var edits:[NativeSourceEdit];var inverse:[NativeSourceEdit]
        var selectionBefore:NSRange;var selectionAfter:NSRange;var title:String;var contextBefore:UUID;var contextAfter:UUID
        var reversed:Step {Step(before:after,after:before,edits:inverse,inverse:edits,selectionBefore:selectionAfter,selectionAfter:selectionBefore,title:title,contextBefore:contextAfter,contextAfter:contextBefore)}
    }
    typealias Apply=([NativeSourceEdit],String,NSRange)->Bool
    private func identity(_ source:String)->SHA256.Digest{SHA256.hash(data:Data(source.utf8))}
    func commit<Context>(_ plan:ChangePlan,path:String,before:String,selectionBefore:NSRange,selectionAfter:NSRange,contextBefore:Context,contextAfter:Context,manager:UndoManager?,read:@escaping ()->String,apply:@escaping Apply,notice:@escaping (String)->Void,restoreContext:@escaping (Context)->Void,contextCost:(Context)->Int,discardContext:@escaping ()->Void,alreadyObserved:Bool=false)->Bool {
        guard plan.patches.count<=1 else{return false}
        let patch=plan.patches.first
        if let patch{guard exactUTF8(patch.path,path),exactUTF8(patch.before,before) else{return false}}
        if alreadyObserved {
            guard exactUTF8(read(),patch?.after ?? before) else{return false}
            if let patch {
                guard patch.edits.allSatisfy({$0.from>=0 && $0.to>=$0.from && $0.to<=before.utf16.count}) else{return false}
                let edits=patch.edits.map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)}
                guard validSourceEdits(edits,length:before.utf16.count) else{return false}
                let observed=NSMutableString(string:before)
                for edit in edits.reversed(){observed.replaceCharacters(in:edit.range,with:edit.text)}
                guard exactUTF8(observed as String,patch.after) else{return false}
            }
        } else if !exactUTF8(read(),before) {return false}
        let first=Self.hold(contextBefore,cost:contextCost(contextBefore),restore:restoreContext),last=Self.hold(contextAfter,cost:contextCost(contextAfter),restore:restoreContext)
        let step=Step(before:identity(before),after:identity(patch?.after ?? before),edits:(patch?.edits ?? []).map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},inverse:(invertChangePlan(plan).patches.first?.edits ?? []).map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},selectionBefore:selectionBefore,selectionAfter:selectionAfter,title:plan.title,contextBefore:first,contextAfter:last)
        if alreadyObserved {
            restoreContext(contextAfter)
        } else {
            guard perform(step,manager:manager,read:read,apply:apply,discardContext:discardContext,forwardContext:{restoreContext(contextAfter)}) else{return false}
        }
        if step.before != step.after,let manager{register(step.reversed,manager:manager,read:read,apply:apply,notice:notice,discardContext:discardContext)}
        return true
    }
    private func perform(_ step:Step,manager:UndoManager?,read:()->String,apply:Apply,discardContext:()->Void,forwardContext:(()->Void)?=nil)->Bool {
        let source=read();guard identity(source)==step.before else{return false}
        manager?.disableUndoRegistration();defer{manager?.enableUndoRegistration()}
        guard apply(step.edits,source,step.selectionAfter),identity(read())==step.after else{return false}
        if let forwardContext{forwardContext()}else if let context=Self.contexts[step.contextAfter]{context.restore()}else{discardContext()}
        return true
    }
    private func register(_ step:Step,manager:UndoManager,read:@escaping ()->String,apply:@escaping Apply,notice:@escaping (String)->Void,discardContext:@escaping ()->Void) {
        let group = !manager.isUndoing && !manager.isRedoing
        // Separate consecutive explicit source actions even in one host event.
        // Leave caller-owned nested groups and undo/redo replay intact.
        if group,manager.groupsByEvent,manager.groupingLevel==1{manager.endUndoGrouping();manager.beginUndoGrouping()}
        if group{manager.beginUndoGrouping()}
        manager.registerUndo(withTarget:self){[weak manager] history in
            guard let manager else{return}
            guard history.perform(step,manager:manager,read:read,apply:apply,discardContext:discardContext) else{
                history.retire(manager);notice("Undo history retired because source or permissions changed. Current draft is retained.");return
            }
            history.register(step.reversed,manager:manager,read:read,apply:apply,notice:notice,discardContext:discardContext)
        }
        manager.setActionName(step.title)
        if group{manager.endUndoGrouping()}
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/SourceDraftHistory.swift"><ui:paste select="#native-source-draft-history"/></ui:file>

An oversized optional context is refused before its value enters the history pool. Its absent token still permits ordinary source undo and discards only unavailable feature state; it must not evict valid contexts belonging to earlier edits. The focused witness supplies an over-budget cost without allocating a large payload, then replays both directions through the existing native undo manager.

<ui:copy id="native-source-draft-history-tests">
import XCTest
@testable import HickorySimple
@MainActor final class SourceDraftHistoryTests:XCTestCase {
    func testOversizedContextCannotEvictEarlierUndoState() {
        let history=SourceDraftHistory(),manager=UndoManager();manager.groupsByEvent=false
        var source="a",restored=""
        func commit(_ before:String,_ after:String,_ first:String,_ last:String)->Bool {
            let plan=ChangePlan(title:"Replace source",patches:[SourcePatch(path:"a.py",before:before,after:after,edits:[SourceReplacement(from:0,to:before.utf16.count,text:after)])])
            return history.commit(plan,path:"a.py",before:before,selectionBefore:NSRange(location:0,length:0),selectionAfter:NSRange(location:1,length:0),contextBefore:first,contextAfter:last,manager:manager,
                read:{source},apply:{edits,before,_ in let text=NSMutableString(string:before);for edit in edits.sorted(by:{$0.range.location>$1.range.location}){text.replaceCharacters(in:edit.range,with:edit.text)};source=text as String;return true},notice:{_ in XCTFail("Source undo must remain available")},restoreContext:{restored=$0},contextCost:{$0=="oversized" ? 32*1024*1024+1:64},discardContext:{restored="discarded"})
        }
        XCTAssertTrue(commit("a","b","old-a","old-b"));XCTAssertTrue(commit("b","c","oversized","oversized"))
        manager.undo();XCTAssertEqual(source,"b");XCTAssertEqual(restored,"discarded")
        manager.undo();XCTAssertEqual(source,"a");XCTAssertEqual(restored,"old-a");XCTAssertTrue(manager.canRedo)
        manager.redo();XCTAssertEqual(source,"b");XCTAssertEqual(restored,"old-b")
        manager.redo();XCTAssertEqual(source,"c");XCTAssertEqual(restored,"discarded")
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/SourceDraftHistoryTests.swift"><ui:paste select="#native-source-draft-history-tests"/></ui:file>

</ui:doc>
