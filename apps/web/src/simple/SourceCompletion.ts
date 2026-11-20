import {isolateHistory} from '@codemirror/commands';
import {EditorSelection,EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {closeCompletion,pickedCompletion,CompletionContext,type Completion} from '@codemirror/autocomplete';
import type {CompletionItem} from '../lsp/client';
import {lspCompletion,wordBefore} from '../lsp/completion';
import {beginCompletionAcceptance} from '../lsp/completionAcceptance';
import {prepareEditorChanges} from '../editor/textInput';
import {planCompletion,planCompletionGroup,type CapturedCompletion,type CompletionDraft} from './CompletionPlan';
import {sourceDigest} from '../lsp/diagnosticRevision';
import {completionExpansion} from './SnippetExpansion';
import {snippetContextNotice} from './SnippetContext';
import {sourceSnippetEffect,continueSourceSnippetCompletion,readSourceSnippet,readSourceEditorIntent} from './SourceSnippets';
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

function ordinaryProof(item:CompletionItem,hash:string,revision?:string) {
 const data = item.data as {hickory_completion?:unknown;hickory_context?:{source_hash?:unknown;source_revision?:unknown}} | undefined;
 const token = data?.hickory_completion, context = data?.hickory_context;
 const currentRevision = context?.source_revision;
 if ((item.insertTextFormat ?? 1) !== 1 || item.kind === 15 || item.command != null) throw new Error("Completion groups require ordinary provider edits.");
 if (typeof token !== "string" || !token || context?.source_hash !== hash || typeof currentRevision !== "string"
   || !/^[a-f0-9]{64}$/.test(currentRevision) || (revision !== undefined && currentRevision !== revision)) {
  throw new Error("Completion proof changed across carets. Request the completions again.");
 }
 return {token,revision:currentRevision};
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
 const hash = await sourceDigest(source);
 const requireCurrent = () => { if (!valid()) throw new Error("Source, carets or provider changed. Request completions again."); };
 requireCurrent();
 const proof = ordinaryProof(item,hash);
 const seen = new Set<string>();
 const captures: CapturedCompletion[] = [];
 const main = state.selection.mainIndex;
 const identity = offerIdentity(item);
 for (const index of [main,...selections.keys()].filter((index,offset) => offset === 0 || index !== main)) {
  const selection = selections[index];
  const matching = index === main ? [item] : (await session.query(selection.head,source)).filter(candidate => offerIdentity(candidate) === identity);
  requireCurrent();
  if (matching.length !== 1) throw new Error("A caret has no unique matching completion. Choose a different completion.");
  const offer = ordinaryProof(matching[0],hash,proof.revision);
  if (seen.has(offer.token)) throw new Error("Each caret requires its own provider completion offer.");
  seen.add(offer.token);
  const resolved = await session.resolve(structuredClone(matching[0]));
  requireCurrent();
  if (ordinaryProof(resolved,hash,proof.revision).token !== offer.token || resolved.label !== item.label) {
   throw new Error("The provider returned a different completion. Request the completions again.");
  }
  captures[index] = {item:resolved,...selection};
 }
 return planCompletionGroup(adapter.uri,{text:source,writable:adapter.writable()},captures);
}

export async function acceptSourceCompletion(view:EditorView,item:CompletionItem,source:string,selection:EditorState['selection'],from:number,to:number,adapter:SourceCompletionAdapter,completion?:Completion):Promise<void> {
 const request=beginCompletionAcceptance(view,adapter.notice),snippetContext=readSourceSnippet(view),document=view.state.doc,intent=readSourceEditorIntent(view.state);
 let expectedAfter:string|undefined;
 const valid=()=>!request.cancelled()&&request.current()&&readSourceSnippet(view)===snippetContext&&view.state.doc===document&&readSourceEditorIntent(view.state)===intent&&adapter.current()&&view.dom.isConnected&&authoredSource(view.state)===source&&view.state.selection.eq(selection)&&adapter.writable()&&!view.composing&&!view.state.readOnly&&view.state.facet(EditorView.editable);
 const report=(text:string)=>{if(request.current())adapter.notice(text);};
 try {
  if(request.cancelled())return;
  if(view.composing)throw new Error('Finish composition before accepting this completion.');
  if(!valid())throw new Error('Completion is out of date or source is unavailable. Request completions again.');
  report('Resolving completion…');
  let draft:CompletionDraft|undefined,group:Awaited<ReturnType<typeof resolveCompletionGroup>>|undefined,contextNotice='';
  if (selection.ranges.length > 1) {
   if (snippetContext) throw new Error('Finish editing the template before accepting a completion group.');
   group = await resolveCompletionGroup(view.state,source,item,adapter,valid);
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
  const resultSelection = group
   ? EditorSelection.create(group.carets.map(caret=>EditorSelection.cursor(editorOffsetIn(after,caret))),selection.mainIndex)
   : EditorSelection.single(editorOffsetIn(after,draft!.selection.from),editorOffsetIn(after,draft!.selection.to));
  const transaction=view.state.update({...raw,selection:resultSelection,effects:draft?[...rawEffects,sourceSnippetEffect(draft,after)]:rawEffects,annotations:[isolateHistory.of('full'),...(completion?[pickedCompletion.of(completion)]:[])],userEvent:'input.complete',filter:false});
  if(authoredSource(transaction.state)!==after)throw new Error('Editor could not prepare the complete completion. Request again.');
  // CodeMirror records the redo selection from a following selection event;
  // otherwise it maps the old caret through the replacement and loses fields.
  const retainedSelection=after!==source?transaction.state.update({selection:transaction.newSelection,userEvent:'select.complete',filter:false}):null;
  expectedAfter=after;request.update({phase:'applying'});view.update(retainedSelection?[transaction,retainedSelection]:[transaction]);
  const applied=authoredSource(view.state)===after;
  request.update({phase:applied?'applied':'unavailable',applied,source_changed:applied&&source!==after,message:applied?'Completion applied to draft.':'Editor could not reflect the complete completion. Inspect the current draft.'});
  report(applied?((source!==after?'Completion applied · draft':'Completion selected')+(contextNotice?' · '+contextNotice:'')):'Editor could not reflect the complete completion. Inspect the current draft.');
 }catch(error){if((request.current()||expectedAfter!==undefined)&&!request.cancelled()){const applied=expectedAfter!==undefined&&authoredSource(view.state)===expectedAfter,message=error instanceof Error?error.message:String(error);request.update({phase:'unavailable',message,applied,source_changed:applied&&source!==expectedAfter});report(message);}}
}
