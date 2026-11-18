import {authoredSource,authoredOffset,editorOffset,editorOffsetIn,authoredOffsetIn,authoredTransactionSource,authoredChanges,normalizedSource} from './SourceText';
import {EditorSelection,EditorState,Prec,StateEffect,StateField,type Transaction,type Extension,type TransactionSpec,type ChangeSet} from '@codemirror/state';
import {Decoration,EditorView,keymap} from '@codemirror/view';
import {invertedEffects,isolateHistory} from '@codemirror/commands';
import {acceptCompletion,closeCompletion,completionStatus,pickedCompletion,startCompletion,type CompletionSource} from '@codemirror/autocomplete';
import {exclusiveCompletionSources,withCompletionOrigin} from '../lsp/completion';
import {cancelCompletionAcceptance,readCompletionAcceptance} from '../lsp/completionAcceptance';
import {prepareEditorChanges} from '../editor/textInput';
import {beginSnippet,advanceSnippet,editSnippet,completeSnippet,nestSnippet,type SnippetSession} from './SnippetSession';
import type {CompletionDraft} from './CompletionPlan';
import {
 beginParallelSnippet, editParallelSnippet, moveParallelSnippet, parallelSelections, selectParallelSnippet,
 type ParallelExpansion, type ParallelReplacement, type ParallelSnippet, type ParallelSnippetChange,
} from './ParallelSnippet';
import {snippetTransformBytes} from './SnippetTransform';
function computedSnippetInput(session:SnippetSession,from:number,to:number):boolean{return session.fields.some(field=>!!field.transform&&field.from<field.to&&field.from<=from&&to<=field.to&&(from<to||from>field.from&&from<field.to));}
export const setSourceSnippet=StateEffect.define<SnippetSession|null>();
interface SnippetSnapshot {
 source:string;
 session:SnippetSession|null;
 selection:EditorSelection;
 parallel?:ParallelSnippet|null;
}
function retainedFieldCost(fields:Readonly<SnippetSession['fields']>):number {
 return fields.reduce((sum,field)=>sum+64+(field.transform?snippetTransformBytes(field.transform):0)
   +field.choices.reduce((total,choice)=>total+32+choice.length*2,0),0);
}
function retainedParallelCost(parallel:ParallelSnippet|null|undefined):number {
 return parallel?.peers.reduce((sum,peer)=>sum+128+peer.text.length*2
   +retainedFieldCost(peer.session?.fields??[]),0)??0;
}
interface SnippetReplay {target:symbol;inverse:symbol}
export function createSourceSnippetHistory(limit=32*1024*1024,maxEntries=4096){
 if(!Number.isSafeInteger(limit)||limit<=0||!Number.isSafeInteger(maxEntries)||maxEntries<1||maxEntries>4096)throw Error('Invalid snippet history budget');
 const snapshots=new Map<symbol,{value:SnippetSnapshot;cost:number}>();let bytes=0;
 return {hold(value:SnippetSnapshot):symbol {
  // Count the retained UTF-16 strings and every alternative, including empty
  // choices, before constructing or retaining a snapshot. Oversized context
  // never evicts unrelated valid history; CodeMirror still owns text undo.
  const token=Symbol('snippet history'),cost=128+value.source.length*2+retainedFieldCost(value.session?.fields??[])+retainedParallelCost(value.parallel);
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
const sourceEditorIntent=StateField.define<symbol>({create:()=>Symbol('source editor intent'),update:(value,tr)=>tr.docChanged||authoredSource(tr.startState)!==authoredTransactionSource(tr)||tr.selection||tr.effects.some(effect=>effect.is(setSourceSnippet)||effect.is(setParallelSourceSnippet)||effect.is(replaySourceSnippet))?Symbol('source editor intent'):value});
export function readSourceEditorIntent(state:EditorState):symbol|undefined{return state.field(sourceEditorIntent,false);}
const snippetState=StateField.define<SnippetSession|null>({
 create:()=>null,
 update(session,tr){
  let explicit=false;
  for(const effect of tr.effects){if(effect.is(setSourceSnippet)){session=effect.value;explicit=true;}else if(effect.is(setParallelSourceSnippet)){session=null;explicit=true;}else if(effect.is(replaySourceSnippet)){session=snippetSnapshot(effect.value.target)?.session??null;explicit=true;}}
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
 if(readParallelSourceSnippet(view.state))throw Error('Finish editing the multiple template fields before accepting a completion.');
 const current=readSourceSnippet(view);if(!current)return draft;
 const edit=draft.primary;if(!edit)throw new Error('Completion primary edit is unavailable. No edits were applied.');
 const input=draft.fields.length?nestSnippet(current,draft.before,edit,draft.additional??[],draft.fields.map(field=>({...field,from:field.from-draft.insertionFrom,to:field.to-draft.insertionFrom}))):completeSnippet(current,draft.before,edit,draft.additional??[]);if(!input)throw new Error('Completion overlaps template fields or is unavailable. Finish the template and retry.');
 return {...draft,plan:{title:draft.plan.title,patches:input.session.source===draft.before?[]:[{path:draft.path,before:draft.before,after:input.session.source,edits:input.edits}]},selection:input.selection,insertionFrom:input.selection.from-edit.text.length,continuing:input.session};
}
export function readSourceSnippet(view:EditorView):SnippetSession|null{return view.state.field(snippetState,false)??null;}
export function finishSourceSnippet(view:EditorView):boolean {if(!sourceSnippetMenuState(view).active)return false;view.dispatch({effects:setSourceSnippet.of(null)});return true;}
export function moveSourceSnippet(view:EditorView,direction:1|-1):boolean {
 if(view.composing)return false;
 const parallel=readParallelSourceSnippet(view.state);
 if(parallel){
  const next=moveParallelSnippet(parallel,authoredSource(view.state),direction);
  if(!next){finishSourceSnippet(view);return false;}
  if(direction===1&&next.peers.every(peer=>!peer.session?.active)
    &&parallel.peers.every(peer=>!peer.session?.active)){finishSourceSnippet(view);return false;}
  closeCompletion(view);
  view.dispatch({selection:parallelSelection(next),effects:setParallelSourceSnippet.of(next),scrollIntoView:true,userEvent:'select.snippet'});
  return true;
 }
 const session=readSourceSnippet(view);if(!session)return false;
 const selected=view.state.selection.main,result=advanceSnippet(session,authoredSource(view.state),authoredOffset(view.state,selected.from),authoredOffset(view.state,selected.to),direction);
 if(!result){if(direction===-1&&session.active===(session.scopes?.at(-1)?.order[0]??Math.min(...session.fields.filter(field=>!field.transform&&field.index>0).map(field=>field.index))))return true;finishSourceSnippet(view);return false;}
 closeCompletion(view);view.dispatch({selection:{anchor:editorOffset(view.state,result.selection.from),head:editorOffset(view.state,result.selection.to)},effects:setSourceSnippet.of(result.session),scrollIntoView:true,userEvent:'select.snippet'});return true;
}
export function sourceSnippets(adapter:{writable:()=>boolean;notice:(message:string)=>void}):Extension {
 const choiceSource:CompletionSource=context=>{
  const parallel=readParallelSourceSnippet(context.state);
  if(parallel)return previewParallelChoices(context,parallel,adapter.writable,adapter.notice);
  const session=context.state.field(snippetState,false),field=session?.fields.find(field=>field.id===session.current),document=context.state.doc,intent=readSourceEditorIntent(context.state);
  if(!session||!field?.choices.length||authoredSource(context.state)!==session.source)return null;
  return {from:editorOffset(context.state,field.from),to:editorOffset(context.state,field.to),filter:false,options:field.choices.map(value=>withCompletionOrigin({label:value||'Empty value',section:'Template values',apply:(view,completion)=>{
   if(readSourceSnippet(view)!==session||view.state.doc!==document||readSourceEditorIntent(view.state)!==intent||!adapter.writable()||view.composing||view.state.readOnly||!view.state.facet(EditorView.editable)){adapter.notice('Template choice changed. Reopen the values.');return;}
   const plan=editSnippet(session,authoredSource(view.state),field.from,field.to,value);if(!plan){adapter.notice('Template choice unavailable. Current draft is retained.');return;}
   const changes=plan.edits.map(edit=>({from:edit.from,to:edit.to,insert:edit.text})),prepared=prepareEditorChanges(session.source,changes);if('reason' in prepared||prepared.expected!==plan.session.source){adapter.notice('Template choice unavailable. Current draft is retained.');return;}
   const authored=authoredChanges(view.state,changes);closeCompletion(view);view.dispatch({...authored,selection:{anchor:editorOffsetIn(plan.session.source,plan.selection.from),head:editorOffsetIn(plan.session.source,plan.selection.to)},effects:[...(authored.effects as readonly StateEffect<unknown>[]??[]),setSourceSnippet.of(plan.session)],annotations:[isolateHistory.of('full'),pickedCompletion.of(completion)],userEvent:'input.snippet.choice'});
  }},'template'))};
 };
 return [sourceEditorIntent,snippetState,parallelSnippetState,
  invertedEffects.of(tr=>{
   if(!tr.docChanged&&authoredSource(tr.startState)===authoredTransactionSource(tr))return [];
   const replay=tr.effects.find(effect=>effect.is(replaySourceSnippet))?.value as SnippetReplay|undefined;
   if(replay)return [replaySourceSnippet.of({target:replay.inverse,inverse:replay.target})];
   const before=tr.startState.field(snippetState),effect=tr.effects.find(effect=>effect.is(setSourceSnippet));
   const parallel=readParallelSourceSnippet(tr.startState),next=tr.effects.find(effect=>effect.is(setParallelSourceSnippet));
   if(!before&&!effect&&!parallel&&!next)return [];
   return [replaySourceSnippet.of({
    target:holdSnippetSnapshot({source:authoredSource(tr.startState),session:before,selection:tr.startState.selection,parallel}),
    inverse:holdSnippetSnapshot({source:authoredTransactionSource(tr),session:effect?.value??null,selection:tr.newSelection,parallel:next?.value??null}),
   })];
  }),
  exclusiveCompletionSources.of(choiceSource),
  EditorState.transactionFilter.of(tr=>{
   const parallel=readParallelSourceSnippet(tr.startState);
   if(parallel&&tr.docChanged&&!tr.isUserEvent('input.type.compose')
     &&['input.type','input.paste','input.drop','delete'].some(event=>tr.isUserEvent(event))
     &&!tr.effects.some(effect=>effect.is(setSourceSnippet)||effect.is(setParallelSourceSnippet)||effect.is(replaySourceSnippet))
     &&adapter.writable()&&!tr.startState.readOnly&&tr.startState.facet(EditorView.editable))return reflectParallelSnippetInput(tr,parallel);
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
  EditorView.updateListener.of(update=>{
   const before=update.startState.field(snippetState),after=update.state.field(snippetState);
   const oldGroup=readParallelSourceSnippet(update.startState),group=readParallelSourceSnippet(update.state);
   if(before===after&&oldGroup===group)return;
   if(group)adapter.notice(`${group.peers.length} template carets · Tab next · Shift-Tab previous · Esc finish`);
   else if(after)adapter.notice(`Template field ${after.active} · Tab next · Shift-Tab previous · Esc finish`);
   else if(before||oldGroup)adapter.notice('Template editing finished · draft');
  }),
  EditorView.baseTheme({'.simple-snippet-field':{outline:'1px dotted color-mix(in srgb,currentColor 30%,transparent)',outlineOffset:'1px'},'.simple-snippet-field.active':{backgroundColor:'color-mix(in srgb,currentColor 8%,transparent)'}}),
 ];
}
export function openSourceSnippetChoices(view:EditorView):boolean {
 return sourceSnippetMenuState(view).choices&&startCompletion(view);
}

const setParallelSourceSnippet = StateEffect.define<ParallelSnippet | null>();

function parallelSelection(session: ParallelSnippet): EditorSelection {
  const {ranges, primary} = parallelSelections(session);
  return EditorSelection.create(
    ranges.map(({from, to}) => EditorSelection.range(
      editorOffsetIn(session.source, from), editorOffsetIn(session.source, to))),
    primary,
  );
}

function readParallelSourceSnippet(state: EditorState): ParallelSnippet | null {
  return state.field(parallelSnippetState, false) ?? null;
}

const parallelSnippetState = StateField.define<ParallelSnippet | null>({
  create: () => null,
  update(session, tr) {
    const source = authoredTransactionSource(tr);
    for (const effect of tr.effects) {
      if (effect.is(setSourceSnippet)) session = null;
      if (effect.is(setParallelSourceSnippet)) {
        return effect.value && effect.value.source === source &&
          parallelSelection(effect.value).eq(tr.newSelection) ? effect.value : null;
      }
      if (effect.is(replaySourceSnippet)) {
        const snapshot = snippetSnapshot(effect.value.target);
        return snapshot?.source === source ? snapshot.parallel ?? null : null;
      }
    }
    if (!session || session.source !== source || tr.docChanged ||
        !tr.startState.selection.eq(parallelSelection(session))) return null;
    if (!tr.selection || parallelSelection(session).eq(tr.newSelection)) return session;
    const ranges = tr.newSelection.ranges.map(range => ({
      from: authoredOffsetIn(source, range.from),
      to: authoredOffsetIn(source, range.to),
    }));
    return selectParallelSnippet(session, source, ranges, tr.newSelection.mainIndex);
  },
  provide: field => EditorView.decorations.compute([field], state => {
    const session = state.field(field);
    return Decoration.set((session?.peers ?? []).flatMap(peer =>
      (peer.session?.fields ?? []).filter(field => field.index > 0 && field.from < field.to)
        .map(field => Decoration.mark({
          class: !field.transform && field.index === peer.session?.active ?
            'simple-snippet-field active' : 'simple-snippet-field',
        }).range(editorOffset(state, peer.from + field.from), editorOffset(state, peer.from + field.to))),
    ), true);
  }),
});

function prepareParallelSourceBatch(
  state: EditorState, plan: ParallelSnippetChange,
): {spec: TransactionSpec; changes: ChangeSet} | null {
  const source = authoredSource(state);
  const edits = plan.edits.map(edit => ({from: edit.from, to: edit.to, insert: edit.text}));
  const checked = prepareEditorChanges(source, edits);
  if ('reason' in checked || checked.expected !== plan.state.source) return null;
  const authored = authoredChanges(state, edits);
  const changes = state.changes(authored.changes);
  if (changes.apply(state.doc).toString() !== normalizedSource(plan.state.source)) return null;
  return {
    spec: {
      ...authored,
      selection: parallelSelection(plan.state),
      effects: [...(authored.effects as readonly StateEffect<unknown>[] ?? []),
        setParallelSourceSnippet.of(plan.state)],
    },
    changes,
  };
}

function reflectParallelSnippetInput(tr: Transaction, session: ParallelSnippet): Transaction | readonly TransactionSpec[] {
  const selections = parallelSelections(session).ranges;
  const after = authoredTransactionSource(tr);
  const changes: ParallelReplacement[] = [];
  tr.changes.iterChanges((from, to, start, end) => changes.push({
    from: authoredOffset(tr.startState, from), to: authoredOffset(tr.startState, to),
    text: after.slice(authoredOffsetIn(after, start), authoredOffsetIn(after, end)),
  }), true);

  let inputs = changes;
  if (changes.length !== selections.length) {
    // CodeMirror merges coincident empty visual cursors. Duplicate only the
    // exact insertion those peers share; an ambiguous deletion stays native.
    inputs = [];
    let current = 0;
    for (const [index, range] of selections.entries()) {
      const previous = selections[index - 1];
      if (previous && previous.from === range.from && previous.to === range.to) {
        const edit = inputs[index - 1];
        if (edit.from !== range.from || edit.to !== range.to || range.from !== range.to) return tr;
        inputs.push(edit);
      } else {
        if (current >= changes.length) return tr;
        inputs.push(changes[current++]);
      }
    }
    if (current !== changes.length) return tr;
  }
  const plan = editParallelSnippet(session, authoredSource(tr.startState), inputs);
  const result = plan && prepareParallelSourceBatch(tr.startState, plan);
  if (!result) return tr;
  const changesAfterInput = tr.changes.invert(tr.startState.doc).compose(result.changes);
  // Keep the triggering input transaction and its native history metadata.
  return [tr, {...result.spec, changes: changesAfterInput, sequential: true}];
}

export function applyParallelSourceSnippet(
  view: EditorView, source: string, expansions: readonly ParallelExpansion[], primary: number,
  intent: symbol, admitted: () => boolean,
): boolean {
  if (!admitted() || readSourceEditorIntent(view.state) !== intent || view.composing ||
      view.state.readOnly || !view.state.facet(EditorView.editable) ||
      authoredSource(view.state) !== source || !view.state.facet(EditorState.allowMultipleSelections)) return false;
  const selected = view.state.selection;
  if (selected.mainIndex !== primary || expansions.length !== selected.ranges.length) return false;
  for (const [index, range] of selected.ranges.entries()) {
    const edit = expansions[index];
    if (edit.from !== authoredOffset(view.state, range.from) ||
        edit.to !== authoredOffset(view.state, range.to)) return false;
  }
  const plan = beginParallelSnippet(source, expansions, primary);
  if (!plan) return false;
  const prepared = prepareParallelSourceBatch(view.state, plan);
  if (!prepared || !admitted() || readSourceEditorIntent(view.state) !== intent) return false;
  view.dispatch({...prepared.spec, annotations: isolateHistory.of('full'), userEvent: 'input.snippet.parallel'});
  return authoredSource(view.state) === plan.state.source && readParallelSourceSnippet(view.state) === plan.state;
}

function commonParallelChoices(session: ParallelSnippet): string[] {
  const fields = session.peers.flatMap(peer => {
    const live = peer.session;
    if (!live?.active) return [];
    const field = live.fields.find(field => field.id === live.current);
    return field ? [{peer, field}] : [];
  });
  if (fields.length !== session.peers.length) return [];
  return fields[0].field.choices.filter(value => fields.every(({field}) => field.choices.includes(value)));
}

function previewParallelChoices(
  context: Parameters<CompletionSource>[0], session: ParallelSnippet,
  writable: () => boolean, notice: (message: string) => void,
) {
  const values = commonParallelChoices(session);
  if (!values.length) return null;
  const intent = readSourceEditorIntent(context.state);
  const selected = parallelSelection(session).main;
  return {from: selected.from, to: selected.to, filter: false, options: values.map(value =>
    withCompletionOrigin({
      label: value || 'Empty value', section: 'Template values',
      apply: (view, completion) => {
        if (readParallelSourceSnippet(view.state) !== session || readSourceEditorIntent(view.state) !== intent ||
            !writable() || view.composing || view.state.readOnly || !view.state.facet(EditorView.editable)) {
          notice('Template choice changed. Reopen the values.');
          return;
        }
        const input = session.peers.map(peer => {
          const field = peer.session!.fields.find(field => field.id === peer.session!.current)!;
          return {from: peer.from + field.from, to: peer.from + field.to, text: value};
        });
        const plan = editParallelSnippet(session, authoredSource(view.state), input);
        const prepared = plan && prepareParallelSourceBatch(view.state, plan);
        if (!prepared) {
          notice('Template choice unavailable. Current draft is retained.');
          return;
        }
        closeCompletion(view);
        view.dispatch({...prepared.spec, annotations: [isolateHistory.of('full'), pickedCompletion.of(completion)],
          userEvent: 'input.snippet.choice'});
      },
    }, 'template'),
  )};
}

export function sourceSnippetMenuState(view: EditorView): {active: boolean; choices: boolean} {
  const parallel = readParallelSourceSnippet(view.state);
  const single = readSourceSnippet(view);
  return {
    active: !!parallel || !!single,
    choices: parallel ? commonParallelChoices(parallel).length > 0 :
      !!single?.fields.some(field => !field.transform && field.index === single.active && field.choices.length),
  };
}
