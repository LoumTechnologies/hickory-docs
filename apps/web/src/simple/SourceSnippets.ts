import {authoredSource,authoredOffset,editorOffset,editorOffsetIn,authoredOffsetIn,authoredTransactionSource,authoredChanges,normalizedSource} from './SourceText';
import {EditorSelection,EditorState,Prec,StateEffect,StateField,type Transaction,type Extension} from '@codemirror/state';
import {Decoration,EditorView,keymap} from '@codemirror/view';
import {invertedEffects,isolateHistory} from '@codemirror/commands';
import {acceptCompletion,closeCompletion,completionStatus,pickedCompletion,startCompletion,type CompletionSource} from '@codemirror/autocomplete';
import {exclusiveCompletionSources,withCompletionOrigin} from '../lsp/completion';
import {cancelCompletionAcceptance,readCompletionAcceptance} from '../lsp/completionAcceptance';
import {prepareEditorChanges} from '../editor/textInput';
import {beginSnippet,advanceSnippet,editSnippet,completeSnippet,nestSnippet,type SnippetSession} from './SnippetSession';
import type {CompletionDraft} from './CompletionPlan';
import {snippetTransformBytes} from './SnippetTransform';
function computedSnippetInput(session:SnippetSession,from:number,to:number):boolean{return session.fields.some(field=>!!field.transform&&field.from<field.to&&field.from<=from&&to<=field.to&&(from<to||from>field.from&&from<field.to));}
export const setSourceSnippet=StateEffect.define<SnippetSession|null>();
interface SnippetSnapshot {source:string;session:SnippetSession|null;selection:EditorSelection}
interface SnippetReplay {target:symbol;inverse:symbol}
export function createSourceSnippetHistory(limit=32*1024*1024,maxEntries=4096){
 if(!Number.isSafeInteger(limit)||limit<=0||!Number.isSafeInteger(maxEntries)||maxEntries<1||maxEntries>4096)throw Error('Invalid snippet history budget');
 const snapshots=new Map<symbol,{value:SnippetSnapshot;cost:number}>();let bytes=0;
 return {hold(value:SnippetSnapshot):symbol {
  // Count the retained UTF-16 strings and every alternative, including empty
  // choices, before constructing or retaining a snapshot. Oversized context
  // never evicts unrelated valid history; CodeMirror still owns text undo.
  const token=Symbol('snippet history'),cost=128+value.source.length*2+(value.session?.fields.reduce((sum,field)=>sum+64+(field.transform?snippetTransformBytes(field.transform):0)+field.choices.reduce((total,choice)=>total+32+choice.length*2,0),0)??0);
  if(cost>limit)return token;
  const normalized={...value,session:value.session?{...value.session,source:value.source}:null};snapshots.set(token,{value:normalized,cost});bytes+=cost;
  while((bytes>limit||snapshots.size>maxEntries)&&snapshots.size){const oldest=snapshots.keys().next().value!;bytes-=snapshots.get(oldest)!.cost;snapshots.delete(oldest);}return token;
 },read(token:symbol):SnippetSnapshot|undefined{return snapshots.get(token)?.value;}};
}
const snippetHistory=createSourceSnippetHistory();
function holdSnippetSnapshot(value:SnippetSnapshot):symbol{return snippetHistory.hold(value);}
function snippetSnapshot(token:symbol):SnippetSnapshot|undefined{return snippetHistory.read(token);}
const replaySourceSnippet=StateEffect.define<SnippetReplay>();
export function dispatchSourceSnippetTransactions(transactions:readonly Transaction[],view:EditorView){
 let state=view.state;const updates:Transaction[]=[];
 for(const original of transactions){
  const replay=original.effects.find(effect=>effect.is(replaySourceSnippet))?.value as SnippetReplay|undefined;
  const target=replay?snippetSnapshot(replay.target):undefined,selected=target&&authoredTransactionSource(original)===target.source?target.selection:undefined;
  const transaction=state===original.startState&&!selected?original:state.update(original,{...(selected?{selection:selected}:{}),sequential:true,filter:false});
  updates.push(transaction);state=transaction.state;
 }
 view.update(updates);
}
const sourceEditorIntent=StateField.define<symbol>({create:()=>Symbol('source editor intent'),update:(value,tr)=>tr.docChanged||authoredSource(tr.startState)!==authoredTransactionSource(tr)||tr.selection||tr.effects.some(effect=>effect.is(setSourceSnippet)||effect.is(replaySourceSnippet))?Symbol('source editor intent'):value});
export function readSourceEditorIntent(state:EditorState):symbol|undefined{return state.field(sourceEditorIntent,false);}
const snippetState=StateField.define<SnippetSession|null>({
 create:()=>null,
 update(session,tr){
  let explicit=false;
  for(const effect of tr.effects){if(effect.is(setSourceSnippet)){session=effect.value;explicit=true;}else if(effect.is(replaySourceSnippet)){session=snippetSnapshot(effect.value.target)?.session??null;explicit=true;}}
  if(!session)return null;
  if(authoredTransactionSource(tr)!==session.source||tr.newSelection.ranges.length!==1)return null;
  if(tr.docChanged&&!explicit)return null;
  if(tr.selection){
   const current=session,raw=tr.newSelection.main,source=authoredTransactionSource(tr),selected={from:authoredOffsetIn(source,raw.from),to:authoredOffsetIn(source,raw.to)};if(computedSnippetInput(current,selected.from,selected.to))return null;const candidates=current.fields.filter(field=>!field.transform&&field.index>0&&field.from<=selected.from&&field.to>=selected.to);
   const same=candidates.find(field=>field.id===current.current),next=same??candidates.sort((a,b)=>(a.to-a.from)-(b.to-b.from))[0];
   if(!next)return null;if(next.index!==current.active||next.id!==current.current)session={...current,active:next.index,current:next.id!};
  }
  return session;
 },
 provide:field=>EditorView.decorations.compute([field],state=>{const session=state.field(field);return Decoration.set((session?.fields??[]).filter(field=>field.index>0&&field.from<field.to).map(field=>Decoration.mark({class:!field.transform&&field.index===session!.active?'simple-snippet-field active':'simple-snippet-field'}).range(editorOffset(state,field.from),editorOffset(state,field.to))),true);}),
});
export function sourceSnippetEffect(draft:CompletionDraft,after:string){return setSourceSnippet.of(draft.continuing??beginSnippet(after,draft.fields));}
export function continueSourceSnippetCompletion(view:EditorView,draft:CompletionDraft):CompletionDraft {
 const current=readSourceSnippet(view);if(!current)return draft;
 const edit=draft.primary;if(!edit)throw new Error('Completion primary edit is unavailable. No edits were applied.');
 const input=draft.fields.length?nestSnippet(current,draft.before,edit,draft.additional??[],draft.fields.map(field=>({...field,from:field.from-draft.insertionFrom,to:field.to-draft.insertionFrom}))):completeSnippet(current,draft.before,edit,draft.additional??[]);if(!input)throw new Error('Completion overlaps template fields or is unavailable. Finish the template and retry.');
 return {...draft,plan:{title:draft.plan.title,patches:input.session.source===draft.before?[]:[{path:draft.path,before:draft.before,after:input.session.source,edits:input.edits}]},selection:input.selection,insertionFrom:input.selection.from-edit.text.length,continuing:input.session};
}
export function readSourceSnippet(view:EditorView):SnippetSession|null{return view.state.field(snippetState,false)??null;}
export function finishSourceSnippet(view:EditorView):boolean {if(!readSourceSnippet(view))return false;view.dispatch({effects:setSourceSnippet.of(null)});return true;}
export function moveSourceSnippet(view:EditorView,direction:1|-1):boolean {
 const session=readSourceSnippet(view);if(!session||view.composing)return false;
 const selected=view.state.selection.main,result=advanceSnippet(session,authoredSource(view.state),authoredOffset(view.state,selected.from),authoredOffset(view.state,selected.to),direction);
 if(!result){if(direction===-1&&session.active===(session.scopes?.at(-1)?.order[0]??Math.min(...session.fields.filter(field=>!field.transform&&field.index>0).map(field=>field.index))))return true;finishSourceSnippet(view);return false;}
 closeCompletion(view);view.dispatch({selection:{anchor:editorOffset(view.state,result.selection.from),head:editorOffset(view.state,result.selection.to)},effects:setSourceSnippet.of(result.session),scrollIntoView:true,userEvent:'select.snippet'});return true;
}
export function sourceSnippets(adapter:{writable:()=>boolean;notice:(message:string)=>void}):Extension {
 const choiceSource:CompletionSource=context=>{
  const session=context.state.field(snippetState,false),field=session?.fields.find(field=>field.id===session.current),document=context.state.doc,intent=readSourceEditorIntent(context.state);
  if(!session||!field?.choices.length||authoredSource(context.state)!==session.source)return null;
  return {from:editorOffset(context.state,field.from),to:editorOffset(context.state,field.to),filter:false,options:field.choices.map(value=>withCompletionOrigin({label:value||'Empty value',section:'Template values',apply:(view,completion)=>{
   if(readSourceSnippet(view)!==session||view.state.doc!==document||readSourceEditorIntent(view.state)!==intent||!adapter.writable()||view.composing||view.state.readOnly||!view.state.facet(EditorView.editable)){adapter.notice('Template choice changed. Reopen the values.');return;}
   const plan=editSnippet(session,authoredSource(view.state),field.from,field.to,value);if(!plan){adapter.notice('Template choice unavailable. Current draft is retained.');return;}
   const changes=plan.edits.map(edit=>({from:edit.from,to:edit.to,insert:edit.text})),prepared=prepareEditorChanges(session.source,changes);if('reason' in prepared||prepared.expected!==plan.session.source){adapter.notice('Template choice unavailable. Current draft is retained.');return;}
   const authored=authoredChanges(view.state,changes);closeCompletion(view);view.dispatch({...authored,selection:{anchor:editorOffsetIn(plan.session.source,plan.selection.from),head:editorOffsetIn(plan.session.source,plan.selection.to)},effects:[...(authored.effects as readonly StateEffect<unknown>[]??[]),setSourceSnippet.of(plan.session)],annotations:[isolateHistory.of('full'),pickedCompletion.of(completion)],userEvent:'input.snippet.choice'});
  }},'template'))};
 };
 return [sourceEditorIntent,snippetState,
  invertedEffects.of(tr=>{
   if(!tr.docChanged&&authoredSource(tr.startState)===authoredTransactionSource(tr))return [];
   const replay=tr.effects.find(effect=>effect.is(replaySourceSnippet))?.value as SnippetReplay|undefined;
   if(replay)return [replaySourceSnippet.of({target:replay.inverse,inverse:replay.target})];
   const before=tr.startState.field(snippetState),effect=tr.effects.find(effect=>effect.is(setSourceSnippet));
   if(!before&&!effect)return [];
   return [replaySourceSnippet.of({target:holdSnippetSnapshot({source:authoredSource(tr.startState),session:before,selection:tr.startState.selection}),inverse:holdSnippetSnapshot({source:authoredTransactionSource(tr),session:effect?.value??null,selection:tr.newSelection})})];
  }),
  exclusiveCompletionSources.of(choiceSource),
  EditorState.transactionFilter.of(tr=>{
   const session=tr.startState.field(snippetState);if(!session||!tr.docChanged||tr.effects.some(effect=>effect.is(setSourceSnippet))||!['input.type','input.paste','input.drop','input.complete','delete'].some(event=>tr.isUserEvent(event))||tr.startState.selection.ranges.length!==1||!adapter.writable()||tr.startState.readOnly||!tr.startState.facet(EditorView.editable))return tr;
   const changes:{from:number;to:number;text:string}[]=[],after=authoredTransactionSource(tr);tr.changes.iterChanges((from,to,a,b)=>changes.push({from:authoredOffset(tr.startState,from),to:authoredOffset(tr.startState,to),text:after.slice(authoredOffsetIn(after,a),authoredOffsetIn(after,b))}));if(changes.length!==1)return tr;
   const changed=changes[0];if(computedSnippetInput(session,changed.from,changed.to))return tr;const plan=editSnippet(session,authoredSource(tr.startState),changed.from,changed.to,changed.text);if(!plan)return tr;
   const edits=plan.edits.map(edit=>({from:edit.from,to:edit.to,insert:edit.text})),prepared=prepareEditorChanges(session.source,edits);if('reason' in prepared||prepared.expected!==plan.session.source)return tr;
   const authored=authoredChanges(tr.startState,edits),complete=tr.startState.changes(authored.changes);if(complete.apply(tr.startState.doc).toString()!==normalizedSource(plan.session.source))return tr;
   const additional=tr.changes.invert(tr.startState.doc).compose(complete),rawSelected=tr.newSelection.main,selected={from:authoredOffsetIn(after,rawSelected.from),to:authoredOffsetIn(after,rawSelected.to),anchor:authoredOffsetIn(after,rawSelected.anchor),head:authoredOffsetIn(after,rawSelected.head)},end=changed.from+changed.text.length;
   const anchor=plan.selection.from+selected.anchor-end,head=plan.selection.to+selected.head-end;
   if(selected.from<changed.from||selected.to>end)return tr;
   return [tr,{changes:additional,selection:EditorSelection.single(editorOffsetIn(plan.session.source,anchor),editorOffsetIn(plan.session.source,head)),effects:[...(authored.effects as readonly StateEffect<unknown>[]??[]),setSourceSnippet.of(plan.session)],sequential:true}];
  }),
  Prec.high(keymap.of([
   {key:'Tab',run:view=>view.composing?false:completionStatus(view.state)==='active'?acceptCompletion(view):moveSourceSnippet(view,1),shift:view=>moveSourceSnippet(view,-1)},
   {key:'Escape',run:view=>{if(view.composing)return false;if(readCompletionAcceptance(view)?.phase==='resolving')return cancelCompletionAcceptance(view);return closeCompletion(view)||finishSourceSnippet(view);}},
  ])),
  EditorView.updateListener.of(update=>{const before=update.startState.field(snippetState),after=update.state.field(snippetState);if(before===after)return;if(after)adapter.notice(`Template field ${after.active} · Tab next · Shift-Tab previous · Esc finish`);else if(before)adapter.notice('Template editing finished · draft');}),
  EditorView.baseTheme({'.simple-snippet-field':{outline:'1px dotted color-mix(in srgb,currentColor 30%,transparent)',outlineOffset:'1px'},'.simple-snippet-field.active':{backgroundColor:'color-mix(in srgb,currentColor 8%,transparent)'}}),
 ];
}
export function openSourceSnippetChoices(view:EditorView):boolean {const session=readSourceSnippet(view);if(!session?.fields.some(field=>!field.transform&&field.index===session.active&&field.choices.length))return false;return startCompletion(view);}
