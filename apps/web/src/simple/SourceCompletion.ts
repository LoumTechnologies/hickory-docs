import {isolateHistory} from '@codemirror/commands';
import {EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {closeCompletion,pickedCompletion,type Completion,type CompletionContext} from '@codemirror/autocomplete';
import type {CompletionItem} from '../lsp/client';
import {lspCompletion} from '../lsp/completion';
import {beginCompletionAcceptance} from '../lsp/completionAcceptance';
import {prepareEditorChanges} from '../editor/textInput';
import {planCompletion} from './CompletionPlan';
import {completionExpansion} from './SnippetExpansion';
import {sourceSnippetEffect,continueSourceSnippetCompletion,readSourceSnippet,readSourceEditorIntent} from './SourceSnippets';
import {authoredSource,authoredOffset,editorOffsetIn,authoredChanges} from './SourceText';
export interface SourceCompletionAdapter {
 uri:string;resolve:(item:CompletionItem)=>Promise<CompletionItem>;current:()=>boolean;writable:()=>boolean;notice:(text:string)=>void;
}
const querySnapshots=new WeakMap<EditorState,{source:string;selection:EditorState['selection'];document:EditorState['doc'];intent:symbol|undefined}>();
export function sourceCompletion(item:CompletionItem,context:CompletionContext,adapter:SourceCompletionAdapter):Completion {
 let snapshot=querySnapshots.get(context.state);
 if(!snapshot){snapshot={source:authoredSource(context.state),selection:context.state.selection,document:context.state.doc,intent:readSourceEditorIntent(context.state)};querySnapshots.set(context.state,snapshot);}
 const {source,selection,document,intent}=snapshot,original=structuredClone(item);
 const label=typeof item.filterText==='string'?item.filterText:item.label;
 return {...lspCompletion(item),label,...(label!==item.label?{displayLabel:item.label}:{}),apply:(view,completion,from,to)=>{closeCompletion(view);try{void acceptSourceCompletion(view,original,source,selection,authoredOffset(view.state,from),authoredOffset(view.state,to),{...adapter,current:()=>adapter.current()&&view.state.doc===document&&readSourceEditorIntent(view.state)===intent},completion);}catch(error){adapter.notice(error instanceof Error?error.message:String(error));}}};
}
export async function acceptSourceCompletion(view:EditorView,item:CompletionItem,source:string,selection:EditorState['selection'],from:number,to:number,adapter:SourceCompletionAdapter,completion?:Completion):Promise<void> {
 const request=beginCompletionAcceptance(view,adapter.notice),snippetContext=readSourceSnippet(view),document=view.state.doc,intent=readSourceEditorIntent(view.state);
 let expectedAfter:string|undefined;
 const valid=()=>!request.cancelled()&&request.current()&&readSourceSnippet(view)===snippetContext&&view.state.doc===document&&readSourceEditorIntent(view.state)===intent&&adapter.current()&&view.dom.isConnected&&authoredSource(view.state)===source&&view.state.selection.eq(selection)&&adapter.writable()&&!view.composing&&!view.state.readOnly&&view.state.facet(EditorView.editable);
 const report=(text:string)=>{if(request.current())adapter.notice(text);};
 try {
  if(request.cancelled())return;
  if(selection.ranges.length!==1)throw new Error('Choose one caret before accepting this source completion.');
  if(view.composing)throw new Error('Finish composition before accepting this completion.');
  if(!valid())throw new Error('Completion is out of date or source is unavailable. Request completions again.');
  report('Resolving completion…');const resolved=await adapter.resolve(structuredClone(item));
  if(!request.current()||request.cancelled())return;
  if(view.composing)throw new Error('Finish composition, then request completions again.');
  if(!valid())throw new Error('Source, cursor or provider changed. Request completions again.');
  const draft=continueSourceSnippetCompletion(view,planCompletion(adapter.uri,resolved,{text:source,writable:adapter.writable()},from,to,completionExpansion(resolved)));
  const patch=draft.plan.patches[0],after=patch?.after??source,changes=(patch?.edits??[]).map(edit=>({from:edit.from,to:edit.to,insert:edit.text}));
  const input=prepareEditorChanges(source,changes);if('reason' in input)throw new Error(input.reason);
  if(!valid())throw new Error('Source, cursor or provider changed. Request completions again.');
  const raw=authoredChanges(view.state,changes),rawEffects=raw.effects?[raw.effects].flat():[];
  const transaction=view.state.update({...raw,selection:{anchor:editorOffsetIn(after,draft.selection.from),head:editorOffsetIn(after,draft.selection.to)},effects:[...rawEffects,sourceSnippetEffect(draft,after)],annotations:[isolateHistory.of('full'),...(completion?[pickedCompletion.of(completion)]:[])],userEvent:'input.complete',filter:false});
  if(authoredSource(transaction.state)!==after)throw new Error('Editor could not prepare the complete completion. Request again.');
  // CodeMirror records the redo selection from a following selection event;
  // otherwise it maps the old caret through the replacement and loses fields.
  const retainedSelection=after!==source?transaction.state.update({selection:transaction.newSelection,userEvent:'select.complete',filter:false}):null;
  expectedAfter=after;request.update({phase:'applying'});view.update(retainedSelection?[transaction,retainedSelection]:[transaction]);
  const applied=authoredSource(view.state)===after;
  request.update({phase:applied?'applied':'unavailable',applied,source_changed:applied&&source!==after,message:applied?'Completion applied to draft.':'Editor could not reflect the complete completion. Inspect the current draft.'});
  report(applied?(source!==after?'Completion applied · draft':'Completion selected'):'Editor could not reflect the complete completion. Inspect the current draft.');
 }catch(error){if((request.current()||expectedAfter!==undefined)&&!request.cancelled()){const applied=expectedAfter!==undefined&&authoredSource(view.state)===expectedAfter,message=error instanceof Error?error.message:String(error);request.update({phase:'unavailable',message,applied,source_changed:applied&&source!==expectedAfter});report(message);}}
}
