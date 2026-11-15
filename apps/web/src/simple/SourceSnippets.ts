import {EditorSelection,EditorState,Prec,StateEffect,StateField,type Transaction,type Extension} from '@codemirror/state';
import {Decoration,EditorView,keymap} from '@codemirror/view';
import {invertedEffects,isolateHistory} from '@codemirror/commands';
import {acceptCompletion,closeCompletion,completionStatus,pickedCompletion,startCompletion,type CompletionSource} from '@codemirror/autocomplete';
import {exclusiveCompletionSources,withCompletionOrigin} from '../lsp/completion';
import {cancelCompletionAcceptance,readCompletionAcceptance} from '../lsp/completionAcceptance';
import {prepareEditorChanges} from '../editor/textInput';
import {beginSnippet,advanceSnippet,editSnippet,completeSnippet,type SnippetSession} from './SnippetSession';
import type {CompletionDraft} from './CompletionPlan';
export const setSourceSnippet=StateEffect.define<SnippetSession|null>();
interface SnippetSnapshot {source:string;session:SnippetSession|null;selection:EditorSelection}
interface SnippetReplay {target:symbol;inverse:symbol}
const snippetSnapshots=new Map<symbol,{value:SnippetSnapshot;cost:number}>();let snippetSnapshotBytes=0;
function holdSnippetSnapshot(value:SnippetSnapshot):symbol {
 const token=Symbol('snippet history'),cost=value.source.length*2+(value.session?.fields.reduce((sum,field)=>sum+64+field.choices.reduce((bytes,value)=>bytes+value.length*2,0),0)??0);
 const normalized={...value,session:value.session?{...value.session,source:value.source}:null};
 snippetSnapshots.set(token,{value:normalized,cost});snippetSnapshotBytes+=cost;
 while(snippetSnapshotBytes>32*1024*1024&&snippetSnapshots.size){const oldest=snippetSnapshots.keys().next().value!;snippetSnapshotBytes-=snippetSnapshots.get(oldest)!.cost;snippetSnapshots.delete(oldest);}
 return token;
}
function snippetSnapshot(token:symbol):SnippetSnapshot|undefined{return snippetSnapshots.get(token)?.value;}
const replaySourceSnippet=StateEffect.define<SnippetReplay>();
export function dispatchSourceSnippetTransactions(transactions:readonly Transaction[],view:EditorView){
 let state=view.state;const updates:Transaction[]=[];
 for(const original of transactions){
  const replay=original.effects.find(effect=>effect.is(replaySourceSnippet))?.value as SnippetReplay|undefined;
  const target=replay?snippetSnapshot(replay.target):undefined,selected=target&&original.newDoc.toString()===target.source?target.selection:undefined;
  const transaction=state===original.startState&&!selected?original:state.update(original,{...(selected?{selection:selected}:{}),sequential:true,filter:false});
  updates.push(transaction);state=transaction.state;
 }
 view.update(updates);
}
const snippetState=StateField.define<SnippetSession|null>({
 create:()=>null,
 update(session,tr){
  let explicit=false;
  for(const effect of tr.effects){if(effect.is(setSourceSnippet)){session=effect.value;explicit=true;}else if(effect.is(replaySourceSnippet)){session=snippetSnapshot(effect.value.target)?.session??null;explicit=true;}}
  if(!session)return null;
  if(tr.docChanged&&tr.newDoc.toString()!==session.source||tr.newSelection.ranges.length!==1)return null;
  if(tr.docChanged&&!explicit)return null;
  if(tr.selection){
   const current=session,selected=tr.newSelection.main,candidates=current.fields.filter(field=>field.index>0&&field.from<=selected.from&&field.to>=selected.to);
   const same=candidates.find(field=>field.id===current.current),next=same??candidates.sort((a,b)=>(a.to-a.from)-(b.to-b.from))[0];
   if(!next)return null;if(next.index!==current.active||next.id!==current.current)session={...current,active:next.index,current:next.id!};
  }
  return session;
 },
 provide:field=>EditorView.decorations.from(field,session=>Decoration.set((session?.fields??[]).filter(field=>field.index>0&&field.from<field.to).map(field=>Decoration.mark({class:field.index===session!.active?'simple-snippet-field active':'simple-snippet-field'}).range(field.from,field.to)),true)),
});
export function sourceSnippetEffect(draft:CompletionDraft,after:string){return setSourceSnippet.of(draft.continuing??beginSnippet(after,draft.fields));}
export function continueSourceSnippetCompletion(view:EditorView,draft:CompletionDraft):CompletionDraft {
 const current=readSourceSnippet(view);if(!current)return draft;
 if(draft.fields.length)throw new Error('Finish the current template before inserting another snippet. No edits were applied.');
 const edit=draft.primary;if(!edit)throw new Error('Completion primary edit is unavailable. No edits were applied.');
 const input=completeSnippet(current,draft.before,edit,draft.additional??[]);if(!input)throw new Error('Completion overlaps template fields or is unavailable. Finish the template and retry.');
 return {...draft,plan:{title:draft.plan.title,patches:input.session.source===draft.before?[]:[{path:draft.path,before:draft.before,after:input.session.source,edits:input.edits}]},selection:input.selection,insertionFrom:input.selection.from-edit.text.length,continuing:input.session};
}
export function readSourceSnippet(view:EditorView):SnippetSession|null{return view.state.field(snippetState,false)??null;}
export function finishSourceSnippet(view:EditorView):boolean {if(!readSourceSnippet(view))return false;view.dispatch({effects:setSourceSnippet.of(null)});return true;}
export function moveSourceSnippet(view:EditorView,direction:1|-1):boolean {
 const session=readSourceSnippet(view);if(!session||view.composing)return false;
 const selected=view.state.selection.main,result=advanceSnippet(session,view.state.doc.toString(),selected.from,selected.to,direction);
 if(!result){if(direction===-1&&session.active===Math.min(...session.fields.filter(field=>field.index>0).map(field=>field.index)))return true;finishSourceSnippet(view);return false;}
 closeCompletion(view);view.dispatch({selection:{anchor:result.selection.from,head:result.selection.to},effects:setSourceSnippet.of(result.session),scrollIntoView:true,userEvent:'select.snippet'});return true;
}
export function sourceSnippets(adapter:{writable:()=>boolean;notice:(message:string)=>void}):Extension {
 const choiceSource:CompletionSource=context=>{
  const session=context.state.field(snippetState,false),field=session?.fields.find(field=>field.id===session.current);
  if(!session||!field?.choices.length||context.state.doc.toString()!==session.source)return null;
  return {from:field.from,to:field.to,filter:false,options:field.choices.map(value=>withCompletionOrigin({label:value||'Empty value',section:'Template values',apply:(view,completion)=>{
   if(readSourceSnippet(view)!==session||!adapter.writable()||view.composing||view.state.readOnly||!view.state.facet(EditorView.editable)){adapter.notice('Template choice changed. Reopen the values.');return;}
   const plan=editSnippet(session,view.state.doc.toString(),field.from,field.to,value);if(!plan){adapter.notice('Template choice unavailable. Current draft is retained.');return;}
   const changes=plan.edits.map(edit=>({from:edit.from,to:edit.to,insert:edit.text})),prepared=prepareEditorChanges(session.source,changes);if('reason' in prepared||prepared.expected!==plan.session.source){adapter.notice('Template choice unavailable. Current draft is retained.');return;}
   closeCompletion(view);view.dispatch({changes,selection:{anchor:plan.selection.from,head:plan.selection.to},effects:setSourceSnippet.of(plan.session),annotations:[isolateHistory.of('full'),pickedCompletion.of(completion)],userEvent:'input.snippet.choice'});
  }},'template'))};
 };
 return [snippetState,
  invertedEffects.of(tr=>{
   if(!tr.docChanged)return [];
   const replay=tr.effects.find(effect=>effect.is(replaySourceSnippet))?.value as SnippetReplay|undefined;
   if(replay)return [replaySourceSnippet.of({target:replay.inverse,inverse:replay.target})];
   const before=tr.startState.field(snippetState),effect=tr.effects.find(effect=>effect.is(setSourceSnippet));
   if(!before&&!effect)return [];
   return [replaySourceSnippet.of({target:holdSnippetSnapshot({source:tr.startState.doc.toString(),session:before,selection:tr.startState.selection}),inverse:holdSnippetSnapshot({source:tr.newDoc.toString(),session:effect?.value??null,selection:tr.newSelection})})];
  }),
  exclusiveCompletionSources.of(choiceSource),
  EditorState.transactionFilter.of(tr=>{
   const session=tr.startState.field(snippetState);if(!session||!tr.docChanged||tr.effects.some(effect=>effect.is(setSourceSnippet))||!['input.type','input.paste','input.drop','input.complete','delete'].some(event=>tr.isUserEvent(event))||tr.startState.selection.ranges.length!==1||!adapter.writable()||tr.startState.readOnly||!tr.startState.facet(EditorView.editable))return tr;
   const changes:{from:number;to:number;text:string}[]=[];tr.changes.iterChanges((from,to,_a,_b,text)=>changes.push({from,to,text:text.toString()}));if(changes.length!==1)return tr;
   const changed=changes[0],plan=editSnippet(session,tr.startState.doc.toString(),changed.from,changed.to,changed.text);if(!plan)return tr;
   const edits=plan.edits.map(edit=>({from:edit.from,to:edit.to,insert:edit.text})),prepared=prepareEditorChanges(session.source,edits);if('reason' in prepared||prepared.expected!==plan.session.source)return tr;
   const complete=tr.startState.changes(edits);if(complete.apply(tr.startState.doc).toString()!==plan.session.source)return tr;
   const additional=tr.changes.invert(tr.startState.doc).compose(complete),selected=tr.newSelection.main,end=changed.from+changed.text.length;
   const anchor=plan.selection.from+selected.anchor-end,head=plan.selection.to+selected.head-end;
   if(selected.from<changed.from||selected.to>end)return tr;
   return [tr,{changes:additional,selection:EditorSelection.single(anchor,head),effects:setSourceSnippet.of(plan.session),sequential:true}];
  }),
  Prec.high(keymap.of([
   {key:'Tab',run:view=>view.composing?false:completionStatus(view.state)==='active'?acceptCompletion(view):moveSourceSnippet(view,1),shift:view=>moveSourceSnippet(view,-1)},
   {key:'Escape',run:view=>{if(view.composing)return false;if(readCompletionAcceptance(view)?.phase==='resolving')return cancelCompletionAcceptance(view);return closeCompletion(view)||finishSourceSnippet(view);}},
  ])),
  EditorView.updateListener.of(update=>{const before=update.startState.field(snippetState),after=update.state.field(snippetState);if(before===after)return;if(after)adapter.notice(`Template field ${after.active} · Tab next · Shift-Tab previous · Esc finish`);else if(before)adapter.notice('Template editing finished · draft');}),
  EditorView.baseTheme({'.simple-snippet-field':{outline:'1px dotted color-mix(in srgb,currentColor 30%,transparent)',outlineOffset:'1px'},'.simple-snippet-field.active':{backgroundColor:'color-mix(in srgb,currentColor 8%,transparent)'}}),
 ];
}
export function openSourceSnippetChoices(view:EditorView):boolean {const session=readSourceSnippet(view);if(!session?.fields.some(field=>field.index===session.active&&field.choices.length))return false;return startCompletion(view);}
