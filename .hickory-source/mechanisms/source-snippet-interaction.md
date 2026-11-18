<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Connect explicit snippet plans to existing editing surfaces

For maintainers changing template behavior independently of editor layout. Source and native input authority remain in the existing hosts; the optional controller adds no toolbar or permanent code-space allocation.

## Attach source-bound field interaction to the existing editor

Multi-caret insertion uses the same authored source transaction and undo
context as single-caret insertion. Keep logical peer identities even when the
view coalesces two empty selections. An input transaction can stand for both
only when it inserts at their exact shared boundary. One unsupported or stale
peer retires the linked state and leaves native input in charge. Choices shared
by all current fields use the existing completion popup.

A compartment owns snippet state, source checks, linked input, numeric traversal, choices and history effects. The same adapter retains an opaque editor-input token for asynchronous completion leases. Source/selection transactions advance it without making layout or capability updates into input; undo restores text and field context while retaining a fresh intention. Complete mirroring is prepared before a transaction; rejected intent leaves ordinary input authoritative. Compose the original input transaction with the remaining source changes so its input metadata survives. History effects retain both directions of source, field state and intended selection. The dispatch bridge composes the original transaction with a verified replay selection, preserving the framework history annotation and input metadata; it never creates replacement source changes. This avoids inferring a caret from the old selection when a compound edit had an explicit result. History estimates count 128 bytes per snapshot and cap retained tokens at 4096, including empty/no-session snapshots. They count normalized computed-program bytes and 32 bytes of overhead for every choice, even an empty alternative, before retaining a snapshot. Oversized context is not copied into the cache and does not evict unrelated valid snapshots. History effects retain bounded context tokens: old text undo remains owned by CodeMirror when the estimated 32 MiB context budget evicts a field snapshot. Field marks use paint only, and choices reuse the existing completion popup.

<ui:copy id="source-snippets-browser">
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
 editParallelSnippet, moveParallelSnippet, parallelSelections, planParallelSnippet, selectParallelSnippet,
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

export function readParallelSourceSnippet(state: EditorState): ParallelSnippet | null {
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
  const continuing = readParallelSourceSnippet(view.state);
  if (continuing && !parallelSelection(continuing).eq(selected)) return false;
  const capture = continuing ? parallelSelections(continuing) : {
    primary: selected.mainIndex,
    ranges: selected.ranges.map(range => ({
      from: authoredOffset(view.state, range.from), to: authoredOffset(view.state, range.to),
    })),
  };
  if (capture.primary !== primary || expansions.length !== capture.ranges.length ||
      capture.ranges.some((range, index) => range.from !== expansions[index].from ||
        range.to !== expansions[index].to)) return false;
  const plan = planParallelSnippet(source, expansions, primary, continuing);
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
</ui:copy>

<ui:file path="apps/web/src/simple/SourceSnippets.ts"><ui:paste select="#source-snippets-browser"/></ui:file>

The browser session keeps authored UTF-16 field positions throughout. Only paint, popup ranges and editor selection convert to display positions. Linked typing extracts its exact authored insertion before preparing the shared pure plan, and the final raw edit effect travels with the original input metadata. Raw EOL-only changes advance intention even when display text is equal.

## Keep Apple snippet intent outside AppKit and UIKit

A source-bound controller exposes navigation and choices through existing selection and edit callbacks. The native editor still owns composition, permissions and actual batch/undo application. Stale intent and failed reflection retire the capability rather than blocking ordinary text input. A prepared change plan verifies its entire predicted source before entering history.

Native template batches keep the primary selection first in the existing
multi-selection overlay. Each linked change lands through the per-view undo
manager once. The prior selections, and any live peer state, travel with source
history so Undo can return to the exact selections from which the template was
opened.

The existing native history pool retains optional field contexts under a 32 MiB logical quota and 4096-token limit. Compute field and choice overhead before retaining its copy-on-write value; empty alternatives must not make a large context appear free. Oversized or evicted field context never disables ordinary source undo. Computed occurrences are never traversal or choice targets. Explicit selections within computed text and typing strictly inside it retire linked intent, including when an editable parent surrounds the computed child. The ordinary primary edit remains owned by the editor; adjacent source/empty boundaries retain their existing intent.

<ui:copy id="source-snippets-apple">
import Foundation
import Observation
private func computedSnippetInput(_ session:SnippetSession,_ from:Int,_ to:Int)->Bool {session.fields.contains{field in field.transform != nil && field.from<field.to && field.from<=from && to<=field.to && (from<to || from>field.from && from<field.to)}}
struct SourceSnippetHistoryContext {var fields:[SnippetField];var active:Int;var current:Int;var scopes:[SnippetScope]=[];var parallel:ParallelSnippet?=nil;var ranges:[NSRange]=[]}
func snippetHistoryCost(_ value:SourceSnippetHistoryContext?)->Int {
    guard let value else{return 0}
    let limit=32*1024*1024
    let peers=value.parallel?.peers ?? []
    var bytes=value.ranges.count*32 + (value.parallel?.source.utf8.count ?? 0)
    for peer in peers {bytes+=128+peer.text.utf8.count}
    for field in value.fields+peers.flatMap({$0.session?.fields ?? []}) {
        guard bytes<=limit-64 else{return limit+1};bytes+=64
        if let transform=field.transform {guard transform.bytes<=limit,bytes<=limit-transform.bytes else{return limit+1};bytes+=transform.bytes}
        for choice in field.choices {
            let count=choice.utf8.count
            guard count<=limit-32,bytes<=limit-32-count else{return limit+1};bytes+=32+count
        }
    }
    return bytes
}
@MainActor @Observable final class SourceSnippetController {
    private weak var buffer:DocumentBuffer?
    private(set) var intent=0
    private(set) var session:SnippetSession?
    private var group:ParallelSnippet?
    private(set) var notice=""
    @ObservationIgnored var commit:((SnippetInput,SnippetSession)->Bool)?
    @ObservationIgnored var commitParallel:((ParallelSnippetChange,String)->Bool)?
    init(buffer:DocumentBuffer){self.buffer=buffer}
    var parallel: ParallelSnippet? {
        guard let buffer, let group, exactUTF8(buffer.text, group.source) else { return nil }
        return group
    }

    var isActive: Bool { active != nil || parallel != nil }

    private func orderedRanges(_ session: ParallelSnippet) -> [NSRange] {
        let ranges = session.selections
        let primary = session.primaryIndex
        return [ranges[primary]] + ranges.enumerated().compactMap { $0.offset == primary ? nil : $0.element }
    }

    private func reflectSelections(_ ranges: [NSRange], source: String) -> Bool {
        guard let buffer, let first = ranges.first, let select = buffer.selectSourceRange else { return false }
        let previous = buffer.applyingMulti
        buffer.applyingMulti = true
        defer { buffer.applyingMulti = previous }
        guard select(first, source) else { return false }
        if ranges.count > 1 {
            buffer.multiSource = source
            buffer.multiRanges = ranges
        } else {
            buffer.clearMulti()
        }
        return true
    }

    // A provider preview captures a document intent before requesting any
    // plans. Only the original selections and source may enter source history.
    func applyParallel(
        source: String, expansions: [ParallelSnippetExpansion], primary: Int,
        expectedIntent: Int, admitted: () -> Bool
    ) -> Bool {
        guard let buffer, expectedIntent == intent, admitted(), !buffer.readOnly, !buffer.saving,
              exactUTF8(buffer.text, source), exactUTF8(buffer.multiSource, source),
              buffer.multiRanges.count == expansions.count,
              expansions.indices.contains(primary),
              expansions[primary].edit.range == buffer.selection,
              Set(expansions.map { $0.edit.range }) == Set(buffer.multiRanges),
              let prepared = ParallelSnippet.plan(source, expansions: expansions, primary: primary, continuing: parallel),
              expectedIntent == intent, admitted() else { return false }
        return commitParallel?(prepared, source) == true
    }

    // Native typing produces one edit for the primary selection. Linked
    // replacements use each peer's local field, including distinct grapheme
    // widths when deleting at an empty caret.
    func inputParallel(source: String, primaryRange: NSRange, text: String) -> Bool {
        guard let buffer, let parallel, !buffer.readOnly, !buffer.saving,
              exactUTF8(source, parallel.source), buffer.multiRanges == orderedRanges(parallel),
              let primary = buffer.multiRanges.first, buffer.selection == primary else { return false }
        let original = source as NSString
        let ranges = parallel.selections
        let edited: [NSRange]
        if primaryRange == primary {
            edited = ranges
        } else if text.isEmpty, primary.length == 0, NSMaxRange(primaryRange) == primary.location {
            edited = ranges.map { $0.length == 0 && $0.location > 0 ?
                original.rangeOfComposedCharacterSequence(at: $0.location - 1) : $0 }
        } else if text.isEmpty, primary.length == 0, primaryRange.location == primary.location {
            edited = ranges.map { $0.length == 0 && $0.location < original.length ?
                original.rangeOfComposedCharacterSequence(at: $0.location) : $0 }
        } else {
            return false
        }
        guard edited[parallel.primaryIndex] == primaryRange,
              let result = parallel.editing(source, input: edited.map { NativeSourceEdit(range: $0, text: text) }) else { return false }
        return commitParallel?(result, source) == true
    }
    var active:SnippetSession? {guard let buffer,let session,exactUTF8(buffer.text,session.source) else{return nil};return session}
    var choices:[String] {
        if let parallel {
            let fields=parallel.peers.compactMap{peer in peer.session.flatMap{live in live.fields.first{$0.id==live.current&&$0.index>0}}}
            guard fields.count==parallel.peers.count,let first=fields.first else{return []}
            return first.choices.filter{value in fields.allSatisfy{field in field.choices.contains{exactUTF8($0,value)}}}
        }
        return active.flatMap{live in live.fields.first{$0.id==live.current}}?.choices ?? []
    }
    var historyContext:SourceSnippetHistoryContext? {
        let ranges=buffer.flatMap{exactUTF8($0.text,$0.multiSource)&&$0.multiRanges.count>1 ? $0.multiRanges:nil} ?? []
        if let parallel{return SourceSnippetHistoryContext(fields:[],active:0,current:0,parallel:parallel,ranges:ranges)}
        return active.map{SourceSnippetHistoryContext(fields:$0.fields,active:$0.active,current:$0.current,scopes:$0.scopes)}
            ?? (ranges.isEmpty ? nil:SourceSnippetHistoryContext(fields:[],active:0,current:0,ranges:ranges))
    }
    func restoreHistory(_ context:SourceSnippetHistoryContext?){
        guard let context,let buffer else{restore(nil);return}
        if let parallel=context.parallel,exactUTF8(parallel.source,buffer.text),
           reflectSelections(orderedRanges(parallel),source:buffer.text) {
            session=nil;group=parallel;intent+=1;report();return
        }
        if !context.ranges.isEmpty {_ = reflectSelections(context.ranges,source:buffer.text)}
        guard beginSnippet(buffer.text,fields:context.fields) != nil else{restore(nil);return}
        restore(SnippetSession(source:buffer.text,fields:context.fields,active:context.active,current:context.current,scopes:context.scopes))
    }
    func restore(_ value:SnippetSession?) {session=value;group=nil;intent+=1;report()}
    func sourceChanged(_ source:String) {
        if let expected=group?.source ?? session?.source,!exactUTF8(source,expected){finish()}
    }
    func selectionChanged(_ selected:NSRange) {
        if let buffer,!buffer.applyingMulti,let parallel,orderedRanges(parallel).first != selected {_ = finish()}
    }
    func reportFailure(_ message:String){notice=message;buffer?.language?.completionNotice=message}
    private func report(){
        notice=group.map{"\($0.peers.count) template carets · Tab next · Shift-Tab previous · Esc finish"}
            ?? session.map{"Template field \($0.active) · Tab next · Shift-Tab previous · Esc finish"}
            ?? "Template editing finished · draft"
        buffer?.language?.completionNotice=notice
    }
    @discardableResult func finish()->Bool {guard session != nil || group != nil else{return false};restore(nil);return true}
    @discardableResult func move(_ direction:Int)->Bool {
        if let parallel,let buffer {
            guard !buffer.readOnly,!buffer.saving,buffer.multiRanges==orderedRanges(parallel),
                  let next=parallel.moving(buffer.text,direction:direction) else{_ = finish();return false}
            if direction==1 && parallel.peers.allSatisfy({($0.session?.active ?? 0)==0}){_ = finish();return false}
            guard reflectSelections(orderedRanges(next),source:next.source) else{return false}
            group=next;intent+=1;buffer.focusSource?();report();return true
        }
        guard let buffer,let current=active else{_ = finish();return false}
        guard let next=advanceSnippet(current,source:buffer.text,from:buffer.selection.location,to:NSMaxRange(buffer.selection),direction:direction) else{if direction == -1,current.active==(current.scopes.last?.order.first ?? current.fields.filter({$0.transform==nil}).map({$0.index}).filter({$0>0}).min()){return true};_ = finish();return false}
        let old=session;session=next.session
        guard buffer.selectSourceRange?(NSRange(location:next.selection.from,length:next.selection.to-next.selection.from),current.source)==true else{session=old;return false}
        intent+=1;buffer.focusSource?();report();return true
    }
    // nil leaves ordinary native input in charge; true means one linked batch committed.
    func input(source:String,range:NSRange,text:String,caret:Int?=nil)->Bool? {
        guard let buffer,let current=active,!buffer.readOnly,!buffer.saving,exactUTF8(source,buffer.text),buffer.multiRanges.count<2,!computedSnippetInput(current,range.location,NSMaxRange(range)),var input=editSnippet(current,source:source,from:range.location,to:NSMaxRange(range),text:text),let commit else{_ = finish();return nil}
        if let caret {
            guard caret>=range.location,caret<=range.location+text.utf16.count else{_ = finish();return nil}
            let selected=input.selection.from+caret-(range.location+text.utf16.count);input.selection=CompletionSelection(from:selected,to:selected)
        }
        guard commit(input,current) else{_ = finish();return nil};return true
    }
    func prepareCompletion(_ draft:CompletionDraft)throws->CompletionDraft {
        if parallel != nil {throw EngineError(message:"Finish editing the multiple template fields before accepting a completion.")}
        guard let current=active else{return draft}
        var result=draft
        guard let edit=draft.primary else{throw EngineError(message:"Completion primary edit is unavailable.")}
        let inner=draft.fields.map{field in var value=field;value.from-=draft.insertionFrom;value.to-=draft.insertionFrom;return value}
        guard let input=inner.isEmpty ? completeSnippet(current,source:draft.before,primary:edit,additional:draft.additional):nestSnippet(current,source:draft.before,primary:edit,additional:draft.additional,inner:inner) else{throw EngineError(message:"Completion overlaps template fields or is unavailable. Finish the template and retry.")}
        result.plan=ChangePlan(title:draft.plan.title,patches:exactUTF8(input.session.source,draft.before) ? []:[SourcePatch(path:draft.path,before:draft.before,after:input.session.source,edits:input.edits.map{SourceReplacement(from:$0.range.location,to:NSMaxRange($0.range),text:$0.text)})]);result.selection=input.selection;result.insertionFrom=input.selection.from-edit.text.utf16.count;result.continuing=input.session;return result
    }
    @discardableResult func choose(_ value:String)->Bool {
        if let parallel,let buffer {
            guard !buffer.readOnly,!buffer.saving,buffer.multiRanges==orderedRanges(parallel),
                  choices.contains(where:{exactUTF8($0,value)}) else{return false}
            let edits=parallel.peers.compactMap {peer -> NativeSourceEdit? in
                guard let live=peer.session,let field=live.fields.first(where:{$0.id==live.current}) else{return nil}
                return NativeSourceEdit(range:NSRange(location:peer.range.location+field.from,length:field.to-field.from),text:value)
            }
            guard let next=parallel.editing(buffer.text,input:edits) else{return false}
            return commitParallel?(next,buffer.text)==true
        }
        guard let buffer,let current=active,!buffer.readOnly,!buffer.saving,let field=current.fields.first(where:{$0.id==current.current}),field.choices.contains(where:{exactUTF8($0,value)}) else{return false}
        return input(source:buffer.text,range:NSRange(location:field.from,length:field.to-field.from),text:value)==true
    }
}
@MainActor func snippetChangePlan(path:String,before:String,input:SnippetInput)->ChangePlan? {
    guard validSourceEdits(input.edits,length:before.utf16.count) else{return nil}
    let value=NSMutableString(string:before)
    for edit in input.edits.sorted(by:{$0.range.location>$1.range.location}){value.replaceCharacters(in:edit.range,with:edit.text)}
    guard exactUTF8(value as String,input.session.source) else{return nil}
    return ChangePlan(title:"Edit template field",patches:[SourcePatch(path:path,before:before,after:input.session.source,edits:input.edits.map{SourceReplacement(from:$0.range.location,to:NSMaxRange($0.range),text:$0.text)})])
}

private enum MarkedFieldGroup {
    case single(SnippetSession)
    case parallel(ParallelSnippet)
}

struct MarkedSnippetResult {
    let source: String
    let edits: [NativeSourceEdit]
    let selection: NSRange
    let context: SourceSnippetHistoryContext?
}

@MainActor struct CapturedMarkedSnippet {
    let path: String
    let source: String
    let selection: NSRange
    let history: SourceSnippetHistoryContext?
    private let originalRange: NSRange
    private let group: MarkedFieldGroup
    private let intent: Int

    init?(buffer: DocumentBuffer, source: String, selection: NSRange, range: NSRange) {
        guard !buffer.readOnly, !buffer.saving, exactUTF8(buffer.text, source),
              range.location >= 0, range.length >= 0, range.location <= source.utf16.count,
              range.length <= source.utf16.count - range.location,
              source.utf8.count <= 8 * 1024 * 1024 else { return nil }
        if let parallel = buffer.snippets.parallel {
            guard parallel.selections[parallel.primaryIndex] == range else { return nil }
            group = .parallel(parallel)
        } else if let active = buffer.snippets.active {
            guard active.fields.contains(where: {
                $0.id == active.current && $0.transform == nil &&
                $0.from <= range.location && NSMaxRange(range) <= $0.to
            }), !computedSnippetInput(active, range.location, NSMaxRange(range)) else { return nil }
            group = .single(active)
        } else { return nil }

        self.path = buffer.file.path
        self.source = source
        self.selection = selection
        self.originalRange = range
        self.history = buffer.snippets.historyContext
        // Earlier completion and choice offers cannot survive marked input.
        _ = buffer.snippets.finish()
        buffer.clearMulti()
        self.intent = buffer.snippets.intent
    }

    func finish(buffer: DocumentBuffer, observed: String, selected: NSRange) -> MarkedSnippetResult? {
        guard exactUTF8(path, buffer.file.path), exactUTF8(observed, buffer.text),
              !buffer.readOnly, !buffer.saving, buffer.snippets.intent == intent else { return nil }
        let raw = source as NSString
        let prefix = raw.substring(to: originalRange.location)
        let suffix = raw.substring(from: NSMaxRange(originalRange))
        guard let inserted = markedSourceReplacement(observed, prefix: prefix, suffix: suffix),
              inserted.utf8.count <= 1024 * 1024, selected.location >= originalRange.location,
              selected.length >= 0, NSMaxRange(selected) <= originalRange.location + inserted.utf16.count else { return nil }
        let offset = selected.location - originalRange.location - inserted.utf16.count
        switch group {
        case .single(let current):
            guard let plan = editSnippet(current, source: source, from: originalRange.location,
                                         to: NSMaxRange(originalRange), text: inserted) else { return nil }
            let target = NSRange(location: plan.selection.from + offset, length: selected.length)
            let session = plan.session
            return MarkedSnippetResult(source: session.source, edits: plan.edits, selection: target,
                context: SourceSnippetHistoryContext(fields: session.fields, active: session.active,
                                                     current: session.current, scopes: session.scopes))
        case .parallel(let current):
            let edits = current.selections.map { NativeSourceEdit(range: $0, text: inserted) }
            guard let plan = current.editing(source, input: edits) else { return nil }
            let primary = plan.state.primaryIndex
            var ranges = plan.state.selections
            ranges[primary].location += offset
            ranges[primary].length = selected.length
            guard let next = plan.state.selecting(plan.state.source, ranges: ranges, primary: primary) else { return nil }
            return MarkedSnippetResult(source: next.source, edits: plan.edits, selection: ranges[primary],
                context: SourceSnippetHistoryContext(fields: [], active: 0, current: 0, parallel: next))
        }
    }
}

private func markedSourceReplacement(_ observed: String, prefix: String, suffix: String) -> String? {
    let text = observed as NSString
    let leading = prefix.utf16.count
    let trailing = suffix.utf16.count
    guard text.length >= leading + trailing,
          exactUTF8(text.substring(to: leading), prefix),
          exactUTF8(text.substring(from: text.length - trailing), suffix) else { return nil }
    return text.substring(with: NSRange(location: leading, length: text.length - leading - trailing))
}

// Both sides are already observed source revisions. Use a single minimal
// exact replacement for native history or for reflecting the mirror delta.
func markedSourceEdit(from before: String, to after: String) -> NativeSourceEdit? {
    if exactUTF8(before, after) { return nil }
    let old = before as NSString
    let new = after as NSString
    let shortest = min(old.length, new.length)
    func boundary(_ value: NSString, at: Int) -> Bool {
        guard at > 0, at < value.length else { return true }
        let left = value.character(at: at - 1)
        let right = value.character(at: at)
        return !(left == 13 && right == 10 ||
                 (0xd800...0xdbff).contains(left) && (0xdc00...0xdfff).contains(right))
    }
    var start = 0
    while start < shortest && old.character(at: start) == new.character(at: start) { start += 1 }
    while !boundary(old, at: start) || !boundary(new, at: start) { start -= 1 }
    var suffix = 0
    while suffix < shortest - start &&
          old.character(at: old.length - suffix - 1) == new.character(at: new.length - suffix - 1) {
        suffix += 1
    }
    while !boundary(old, at: old.length - suffix) || !boundary(new, at: new.length - suffix) { suffix -= 1 }
    return NativeSourceEdit(range: NSRange(location: start, length: old.length - start - suffix),
                            text: new.substring(with: NSRange(location: start, length: new.length - start - suffix)))
}

@MainActor func completeMarkedSnippet(
    _ capture: CapturedMarkedSnippet, buffer: DocumentBuffer, observed: String, selected: NSRange,
    history: SourceDraftHistory, manager: UndoManager?, admitted: Bool,
    apply: @escaping SourceDraftHistory.Apply
) {
    guard exactUTF8(capture.path, buffer.file.path), exactUTF8(observed, buffer.text) else { return }
    var finalSource = observed
    var finalSelection = selected
    var finalEdits: [NativeSourceEdit]?
    var context: SourceSnippetHistoryContext?
    if admitted, let next = capture.finish(buffer: buffer, observed: observed, selected: selected) {
        let delta = markedSourceEdit(from: observed, to: next.source)
        manager?.disableUndoRegistration()
        let reflected = apply(delta.map { [$0] } ?? [], observed, next.selection)
        manager?.enableUndoRegistration()
        if reflected, exactUTF8(buffer.text, next.source) {
            finalSource = next.source
            finalSelection = next.selection
            finalEdits = next.edits
            context = next.context
        }
    }
    guard exactUTF8(buffer.text, finalSource) else { return }
    let edits = finalEdits ?? markedSourceEdit(from: capture.source, to: finalSource).map { [$0] } ?? []
    let patches = exactUTF8(capture.source, finalSource) ? [] : [
        SourcePatch(path: capture.path, before: capture.source, after: finalSource,
                    edits: edits.map { SourceReplacement(from: $0.range.location, to: NSMaxRange($0.range), text: $0.text) })
    ]
    let plan = ChangePlan(title: "Compose template field", patches: patches)
    _ = history.commit(plan, path: capture.path, before: capture.source,
        selectionBefore: capture.selection, selectionAfter: finalSelection,
        contextBefore: capture.history, contextAfter: context, manager: manager,
        read: { [weak buffer] in buffer?.text ?? "" }, apply: apply,
        notice: { [weak buffer] message in buffer?.snippets.reportFailure(message) },
        restoreContext: { [weak buffer] context in buffer?.snippets.restoreHistory(context) },
        contextCost: snippetHistoryCost,
        discardContext: { [weak buffer] in _ = buffer?.snippets.finish() },
        alreadyObserved: true)
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/SourceSnippets.swift"><ui:paste select="#source-snippets-apple"/></ui:file>

## Advance Android input intent before publishing UI state

The plain controller core stages a complete next session synchronously within an input transformation, so a second keystroke never waits on a UI callback. Observable revision and status publish after the text transaction; late publication cannot replace a newer session. Navigation and choices use existing source selection and edit operations.

<ui:copy id="source-snippets-android">
package com.hickorydocs.simple
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.text.TextRange
private fun computedSnippetInput(session:SnippetSession,from:Int,to:Int)=session.fields.any{field->field.transform!=null&&field.from<field.to&&field.from<=from&&to<=field.to&&(from<to||from>field.from&&from<field.to)}
// Charge actual retained vector slots, including empty alternatives. Stop at
// budget+1 before retaining a context; the journal owns source text separately.
internal fun snippetHistoryCost(context:SnippetSession?,budget:Long=32L*1024*1024):Long {
 if(context==null)return 0
 val cap=budget+1;var bytes=0L
 fun add(value:Long):Boolean {if(value<0||value>budget-bytes){bytes=cap;return false};bytes+=value;return true}
 for(field in context.fields){if(!add(64L)||!add(field.transform?.bytes?.toLong()?:0L)||field.choices.size>4096||!add(32L*field.choices.size))return cap;for(choice in field.choices)if(!add(2L*choice.length))return cap}
 for(scope in context.scopes)if(!add(64L)||!add(8L*scope.order.size))return cap
 return bytes
}
internal data class AndroidSnippetHistory(
    val session: SnippetSession?,
    val group: ParallelSnippet?,
    val ranges: List<NativeSourceEdit>,
)

private fun androidSnippetHistoryCost(context: AndroidSnippetHistory?): Long {
    if (context == null) return 0
    val limit = 32L * 1024 * 1024
    var total = context.ranges.size * 32L + (context.group?.source?.length ?: 0) * 2L
    total += snippetHistoryCost(context.session)
    for (peer in context.group?.peers.orEmpty()) {
        total += 128 + 2L * peer.text.length + snippetHistoryCost(peer.session, (limit - total).coerceAtLeast(0))
        if (total > limit) return limit + 1
    }
    return total
}

@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class SourceSnippetController(private val buffer:DocumentBuffer) {
 // Core input state advances synchronously; publish outside a TextFieldBuffer transformation.
 private var value:SnippetSession?=null
 private var group:ParallelSnippet?=null
 var intent=0;private set
 private var revision by mutableIntStateOf(0)
 var notice by mutableStateOf("");private set
 private val contexts=SourceUndoContexts<AndroidSnippetHistory?>(::androidSnippetHistoryCost)
 private var markedInput: MarkedSnippetInput? = null
 private var markedConnection: Any? = null
 private var markedCommitRequested = false

 private var nativeMarkingDepth = 0
 internal val retainsMarkedInput: Boolean get() = markedInput != null
 internal val inMarkedInput: Boolean get() = nativeMarkingDepth > 0 || retainsMarkedInput
 internal fun enteringMarkedEvent() { nativeMarkingDepth++ }
 internal fun leavingMarkedEvent() { nativeMarkingDepth-- }

 internal fun beginMarkedInput(connection: Any, supplied: CompletionSelection? = null) {
  if (markedConnection === connection && markedInput != null) {
   markedCommitRequested = false
   return
  }
  if (markedInput != null || buffer.state.composition != null || buffer.readOnly || buffer.saving) return
  val source = buffer.text
  if (source.length > 8 * 1024 * 1024 || source.toByteArray(Charsets.UTF_8).size > 8 * 1024 * 1024) return
  val selected = buffer.state.selection
  val selection = CompletionSelection(selected.min, selected.max)
  val range = supplied ?: selection
  if (range.from < 0 || range.to < range.from || range.to > source.length) return
  val multiple = parallel
  val single = active
  if (multiple != null) {
   if (multiple.selections[multiple.primaryIndex] != range) return
  } else if (single == null || computedSnippetInput(single, range.from, range.to) ||
      single.fields.none { it.id == single.current && it.from <= range.from && range.to <= it.to }) {
   return
  }

  val initial = captureBefore(source, selected, true) ?: return
  finish()
  buffer.clearMulti()
  markedInput = MarkedSnippetInput(buffer.file.path, source, selection, range, initial, intent)
  markedConnection = connection
 }

 internal fun markCommitRequested(connection: Any) {
  if (markedConnection === connection) markedCommitRequested = true
 }

 internal fun observeMarkedInput(source: String) {
  markedInput?.observe(source)
 }

 internal fun settleMarkedInput(connection: Any, closing: Boolean = false) {
  if (markedConnection !== connection || !closing && (!markedCommitRequested || buffer.state.composition != null)) return
  val capture = markedInput ?: return
  markedInput = null
  markedConnection = null
  markedCommitRequested = false
  if (capture.path != buffer.file.path) return

  val source = buffer.text
  val rawSelection = buffer.state.selection
  val observed = CompletionSelection(rawSelection.min, rawSelection.max)
  capture.observe(source)
  var follow: AndroidSnippetHistory? = null
  if (!closing && capture.intent == intent && !buffer.readOnly && !buffer.saving) {
   val next = capture.finish(source, observed)
   if (next != null) {
    val delta = changedMarkedSource(source, next.source)
    if (reflectMarkedInput(delta, source, next.selection) && buffer.text == next.source) {
     val ranges = next.parallel?.let(::parallelRanges).orEmpty()
     follow = AndroidSnippetHistory(next.session, next.parallel, ranges)
     restoreHistory(follow)
    }
   }
  }
  val finalSelection = buffer.state.selection
  val history = SourceUndoSnapshot<AndroidSnippetHistory?>(buffer.text, finalSelection.start, finalSelection.end, follow)
  val replay = capture.replay(buffer.text, mirrored = follow != null)
  if (replay != null) contexts.recordMarked(capture.history, history, replay)
 }

 // The native text state records the supplemental edit. Its observed trail is
 // rebound to the original composition as one action in the source menu.
 private fun reflectMarkedInput(edit: NativeSourceEdit?, source: String, selected: CompletionSelection): Boolean {
  if (buffer.text != source || buffer.state.composition != null || buffer.readOnly || buffer.saving) return false
  val length = source.length + (edit?.let { it.text.length - (it.to - it.from) } ?: 0)
  if (selected.from < 0 || selected.to < selected.from || selected.to > length) return false
  if (edit != null && !validSourceEdits(listOf(edit), source.length)) return false
  val previousSelection = buffer.state.selection
  buffer.state.edit {
   if (edit != null) replace(edit.from, edit.to, edit.text)
   selection = TextRange(selected.from, selected.to)
  }
  if (buffer.text != source || buffer.state.selection != previousSelection) buffer.advanceSourceRevision()
  buffer.clearFolds()
  return true
 }

 val recordSerial get()=contexts.serial
 val canUndo get()=buffer.state.undoState.canUndo
 val canRedo get()=buffer.state.undoState.canRedo
 internal fun captureBefore(source:CharSequence,selection:TextRange,force:Boolean=false):SourceUndoSnapshot<AndroidSnippetHistory?>? {
  val text=value?.source ?: group?.source ?: source.toString()
  val context=historyContext(text)
  if(context==null&&!contexts.retained&&!force)return null
  return SourceUndoSnapshot(text,selection.start,selection.end,context)
 }
 private fun historyContext(source:String):AndroidSnippetHistory? {
  val single=value?.takeIf{it.source==source}
  val multiple=group?.takeIf{it.source==source}
  val ranges=multiple?.let(::parallelRanges) ?: buffer.multiRanges.takeIf{buffer.multiSource==source}.orEmpty()
  return if(single==null&&multiple==null&&ranges.isEmpty())null else AndroidSnippetHistory(single,multiple,ranges)
 }
 private fun restoreHistory(context:AndroidSnippetHistory?) {
  if(context==null){restore(null);return}
  if(context.ranges.isNotEmpty())reflectRanges(buffer.text,context.ranges,true)
  stage(context.session,context.group?.takeIf{it.source==buffer.text})
  publish()
 }
 internal fun finishTextChange(before:SourceUndoSnapshot<AndroidSnippetHistory?>?,source:CharSequence,selected:TextRange){
  if(before==null)return
  val text=source.toString()
  val retained=historyContext(text)
  contexts.record(before,SourceUndoSnapshot(text,selected.start,selected.end,retained))
  stage(retained?.session,retained?.group)
 }
 internal fun attachCompletionContext(before:SourceUndoSnapshot<AndroidSnippetHistory?>,after:SnippetSession?,since:Long){
  val selected=buffer.state.selection
  val snapshot=SourceUndoSnapshot(after?.source ?: buffer.text,selected.start,selected.end,after?.let{AndroidSnippetHistory(it,null,emptyList())})
  if(!contexts.replaceLastAfter(before.source,snapshot,since))contexts.record(before,snapshot)
  restore(after)
 }

 private fun replayMarked(backwards: Boolean): Boolean? {
  val before = buffer.text
  val replay = contexts.currentMarked(before, backwards) ?: return null
  val outcome = replay.run(backwards, { buffer.text }) { undo ->
   if (undo && canUndo) { buffer.state.undoState.undo(); true }
   else if (!undo && canRedo) { buffer.state.undoState.redo(); true }
   else false
  }
  if (outcome == MarkedSourceReplay.Result.Ordinary) return null
  buffer.advanceSourceRevision()
  if (outcome == MarkedSourceReplay.Result.Interrupted) {
   finish()
   notice = "Native history changed during composition replay. Current draft is retained."
   buffer.language?.completionNotice = notice
   return false
  }
  val target = contexts.acceptMarkedReplay(replay, before, buffer.text, backwards) ?: return false
  if (target.anchor in 0..buffer.text.length && target.head in 0..buffer.text.length) {
   buffer.state.edit { selection = TextRange(target.anchor, target.head) }
  }
  restoreHistory(target.context)
  return true
 }
 fun undoSource():Boolean {
  if(buffer.readOnly||buffer.state.composition!=null||inMarkedInput||!canUndo)return false
  replayMarked(true)?.let { return it }
  val before=buffer.text;val oldSelection=buffer.state.selection;buffer.state.undoState.undo();val selected=buffer.state.selection
  if(before!=buffer.text||selected!=oldSelection)buffer.advanceSourceRevision()
  val target=contexts.undo(before,buffer.text,selected.start,selected.end);restoreHistory(target?.context)
  if(target?.context==null){notice="Text edit undone · draft";buffer.language?.completionNotice=notice};return true
 }
 fun redoSource():Boolean {
  if(buffer.readOnly||buffer.state.composition!=null||inMarkedInput||!canRedo)return false
  replayMarked(false)?.let { return it }
  val before=buffer.text;val oldSelection=buffer.state.selection;buffer.state.undoState.redo();val selected=buffer.state.selection
  if(before!=buffer.text||selected!=oldSelection)buffer.advanceSourceRevision()
  val target=contexts.redo(before,buffer.text,selected.start,selected.end);restoreHistory(target?.context)
  if(target?.context==null){notice="Text edit redone · draft";buffer.language?.completionNotice=notice};return true
 }
 val session:SnippetSession? get(){revision;return value}
 val active:SnippetSession? get()=session?.takeIf{it.source==buffer.text}
 val choices:List<String> get(){
  val multiple=parallel
  if(multiple!=null){
   val fields=multiple.peers.mapNotNull{peer->peer.session?.let{live->live.fields.firstOrNull{it.id==live.current&&it.index>0}}}
   if(fields.size!=multiple.peers.size)return emptyList()
   return fields.first().choices.filter{choice->fields.all{choice in it.choices}}
  }
  return active?.let{current->current.fields.firstOrNull{it.id==current.current}?.choices}.orEmpty()
 }
 internal val parallel: ParallelSnippet? get() { revision; return group?.takeIf { it.source == buffer.text } }
 val isActive get() = active != null || parallel != null

 private fun parallelRanges(next: ParallelSnippet): List<NativeSourceEdit> {
  val ordered = next.selections.map { NativeSourceEdit(it.from, it.to, "") }
  val primary = next.primaryIndex
  return listOf(ordered[primary]) + ordered.filterIndexed { index, _ -> index != primary }
 }

 private fun reflectRanges(source: String, ranges: List<NativeSourceEdit>, select: Boolean): Boolean {
  val first = ranges.firstOrNull() ?: return false
  val previous = buffer.applyingMulti
  buffer.applyingMulti = true
  try {
   if (select && !buffer.selectSourceRange(first.from, first.to, source)) return false
   if (buffer.text != source || buffer.state.selection.min != first.from || buffer.state.selection.max != first.to) return false
   if (ranges.size > 1) {
    buffer.multiSource = source
    buffer.multiRanges = ranges
   } else buffer.clearMulti()
   return true
  } finally { buffer.applyingMulti = previous }
 }

 internal fun applyParallel(
  source: String, expansions: List<ParallelSnippetExpansion>, primary: Int,
  expectedIntent: Int, admitted: () -> Boolean,
 ): Boolean {
  if (intent != expectedIntent || !admitted() || source != buffer.text || source != buffer.multiSource ||
      primary !in expansions.indices || expansions.size != buffer.multiRanges.size ||
      buffer.state.composition != null || buffer.readOnly || buffer.saving) return false
  val ranges = expansions.map { it.edit.copy(text = "") }
  if (ranges.toSet() != buffer.multiRanges.toSet() || ranges[primary] != buffer.multiRanges.firstOrNull()) return false
  val next = ParallelSnippet.plan(source, expansions, primary, continuing = parallel) ?: return false
  return intent == expectedIntent && admitted() && commitParallel(next, source)
 }

 private fun commitParallel(next: ParallelSnippetChange, source: String): Boolean {
  val before = captureBefore(source, buffer.state.selection, true) ?: return false
  val serial = recordSerial
  val ranges = parallelRanges(next.state)
  val primary = ranges.first()
  if (!buffer.applySourceEdits(next.edits, source, TextRange(primary.from, primary.to)) ||
      !reflectRanges(next.state.source, ranges, false)) return false
  val selected = buffer.state.selection
  val after = SourceUndoSnapshot<AndroidSnippetHistory?>(buffer.text, selected.start, selected.end, AndroidSnippetHistory(null, next.state, ranges))
  if (!contexts.replaceLastAfter(before.source, after, serial)) contexts.record(before, after)
  stage(null, next.state)
  publish()
  return true
 }

 internal fun inputParallel(text: String): Boolean {
  val current = parallel ?: return false
  if (buffer.multiRanges != parallelRanges(current) || buffer.state.composition != null) return false
  val next = current.editing(buffer.text, current.selections.map { NativeSourceEdit(it.from, it.to, text) }) ?: return false
  return commitParallel(next, buffer.text)
 }

 internal fun planParallelInput(source: String, from: Int, to: Int, text: String): ParallelSnippetChange? {
  val current = parallel ?: return null
  if (buffer.multiRanges != parallelRanges(current) || buffer.readOnly || buffer.saving ||
      source != buffer.text || buffer.state.composition != null) return null
  val selections = current.selections
  val primary = selections[current.primaryIndex]
  val edited = when {
   from == primary.from && to == primary.to -> selections
   text.isEmpty() && primary.from == primary.to && to == primary.from -> selections.map { range ->
    val start = if (range.from == range.to) simpleBackspaceStart(source, range.from) else null
    if (start == null) range else CompletionSelection(start, range.to)
   }
   text.isEmpty() && primary.from == primary.to && from == primary.from -> selections.map { range ->
    if (range.from != range.to || range.to >= source.length) range else {
     val boundaries = java.text.BreakIterator.getCharacterInstance(java.util.Locale.ROOT)
     boundaries.setText(source)
     CompletionSelection(range.from, boundaries.following(range.to).takeUnless { it == java.text.BreakIterator.DONE } ?: range.to)
    }
   }
   else -> return null
  }
  if (edited[current.primaryIndex] != CompletionSelection(from, to)) return null
  return current.editing(source, edited.map { NativeSourceEdit(it.from, it.to, text) })
 }

 internal fun publishParallelInput(epoch: Int) {
  if (intent != epoch) return
  val current = parallel ?: return
  if (!reflectRanges(current.source, parallelRanges(current), false)) { finish(); return }
  publish()
 }
 fun stage(session:SnippetSession?)=stage(session,null)
 internal fun stage(session:SnippetSession?,parallel:ParallelSnippet?){if(value!==session||group!==parallel){value=session;group=parallel;intent++}}
 fun publish(expected:SnippetSession?=value){if(value!==expected)return;revision++;notice=group?.let{"${it.peers.size} template carets · Tab next · Shift-Tab previous · Esc finish"} ?: value?.let{"Template field ${it.active} · Tab next · Shift-Tab previous · Esc finish"} ?: "Template editing finished · draft";buffer.language?.completionNotice=notice}
 fun restore(session:SnippetSession?){stage(session);publish(session)}
 fun finish():Boolean {if(value==null&&group==null)return false;restore(null);return true}
 fun sourceChanged(source:String){if((group?.source ?: value?.source)?.let{it!=source}==true)finish()}
 fun move(direction:Int):Boolean {
  val multiple=parallel
  if(multiple!=null){
   if(buffer.readOnly||buffer.saving||buffer.state.composition!=null||buffer.multiRanges!=parallelRanges(multiple))return false
   val next=multiple.moving(buffer.text,direction) ?: return false
   if(direction==1&&multiple.peers.all{(it.session?.active ?: 0)==0}){finish();return false}
   if(!reflectRanges(next.source,parallelRanges(next),true))return false
   stage(null,next);publish();return true
  }
  val current=active ?: return false;if(buffer.state.composition!=null)return false;val selection=buffer.state.selection
  val next=advanceSnippet(current,buffer.text,selection.min,selection.max,direction) ?: return if(direction == -1&&current.active==(current.scopes.lastOrNull()?.order?.firstOrNull() ?: current.fields.filter{it.transform==null}.map{it.index}.filter{it>0}.minOrNull()))true else finish().let{false}
  val previous=value;stage(next.session)
  if(!buffer.selectSourceRange(next.selection.from,next.selection.to,current.source)){stage(previous);return false};publish(next.session);return true
 }
 fun proposal(source:String,from:Int,to:Int,text:String,caret:Int?=null):SnippetInput? {
  val current=active ?: return null;if(buffer.readOnly||buffer.saving||source!=buffer.text||buffer.multiRanges.size>1||computedSnippetInput(current,from,to))return null
  val input=editSnippet(current,source,from,to,text) ?: return null
  if(caret==null)return input;if(caret !in from..(from+text.length))return null;val selected=input.selection.from+caret-(from+text.length);return input.copy(selection=CompletionSelection(selected,selected))
 }
 fun prepareCompletion(draft:CompletionDraft):CompletionDraft {
  check(parallel==null){"Finish editing the multiple template fields before accepting a completion."}
  val current=active ?: return draft
  val edit=draft.primary ?: error("Completion primary edit is unavailable. No edits were applied.")
  val inner=draft.fields.map{it.copy(from=it.from-draft.insertionFrom,to=it.to-draft.insertionFrom)}
  val input=(if(inner.isEmpty())completeSnippet(current,draft.before,edit,draft.additional)else nestSnippet(current,draft.before,edit,draft.additional,inner)) ?: error("Completion overlaps template fields or is unavailable. Finish the template and retry.")
  return draft.copy(plan=ChangePlan(draft.plan.title,if(input.session.source==draft.before)emptyList() else listOf(SourcePatch(draft.path,draft.before,input.session.source,input.edits.map{SourceReplacement(it.from,it.to,it.text)}))),selection=input.selection,insertionFrom=input.selection.from-edit.text.length,continuing=input.session)
 }
 fun applyInput(source:String,from:Int,to:Int,text:String):Boolean {
  val input=proposal(source,from,to,text) ?: return false;val before=captureBefore(source,buffer.state.selection,true)!!;val serial=recordSerial
  if(!buffer.applySourceEdits(input.edits,source,TextRange(input.selection.from,input.selection.to)))return false;attachCompletionContext(before,input.session,serial);return true
 }
 fun choose(choice:String):Boolean {
  val multiple=parallel
  if(multiple!=null){
   if(buffer.multiRanges!=parallelRanges(multiple)||buffer.state.composition!=null||choice !in choices)return false
   val inputs=multiple.peers.map{peer->
    val session=peer.session ?: return false
    val field=session.fields.firstOrNull{it.id==session.current} ?: return false
    NativeSourceEdit(peer.from+field.from,peer.from+field.to,choice)
   }
   val next=multiple.editing(buffer.text,inputs) ?: return false
   return commitParallel(next,buffer.text)
  }
  val current=active ?: return false;val field=current.fields.firstOrNull{it.id==current.current} ?: return false;if(choice !in field.choices)return false
  val input=proposal(buffer.text,field.from,field.to,choice) ?: return false
  val before=captureBefore(buffer.text,buffer.state.selection,true)!!;val serial=recordSerial
  if(!buffer.applySourceEdits(input.edits,current.source,TextRange(input.selection.from,input.selection.to)))return false;attachCompletionContext(before,input.session,serial);return true
 }
}
</ui:copy>

## Apply one linked proposal inside the existing text transaction

Input transformation uses original source/ranges and checks every mirror against the host display policy. Apply the complete batch and caret together in the existing native transaction. Stage only plain core state here; the caller publishes observable state after the transaction. A refusal retains the primary input for the ordinary editor.

<ui:copy id="source-snippets-android-input">
data class SnippetPrimaryEdit(val edit:NativeSourceEdit,val caret:Int?=null)
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
fun androidx.compose.foundation.text.input.TextFieldBuffer.applySourceSnippetInput(buffer:DocumentBuffer,prepare:(String,Int,Int,String)->SnippetPrimaryEdit={_,from,to,text->SnippetPrimaryEdit(NativeSourceEdit(from,to,text))},canReflect:(NativeSourceEdit)->Boolean={true},publish:(SnippetSession?)->Unit):Boolean {
 if(buffer.snippets.active==null)return false
 if(changes.changeCount!=1||buffer.state.composition!=null){buffer.snippets.stage(null);publish(null);return false}
 val old=changes.getOriginalRange(0);val changed=changes.getRange(0)
 val source=originalText.toString();val primary=prepare(source,old.min,old.max,asCharSequence().subSequence(changed.min,changed.max).toString())
 val plan=buffer.snippets.proposal(source,primary.edit.from,primary.edit.to,primary.edit.text,primary.caret)
 if(plan==null||plan.edits.any{!canReflect(it)}){buffer.snippets.stage(null);publish(null);return false}
 revertAllChanges();plan.edits.asReversed().forEach{replace(it.from,it.to,it.text)};selection=TextRange(plan.selection.from,plan.selection.to)
 buffer.snippets.stage(plan.session);publish(plan.session);return true
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SourceSnippets.kt"><ui:paste select="#source-snippets-android"/><ui:paste select="#source-snippets-android-input"/></ui:file>

## Drive actual browser field transactions

Exercise the actual editor with linked input, field-state history, numeric traversal, coincident empty fields, intended smart-enter selection and the existing choice popup. Source-local choices must answer without invoking a failed or slow provider. Rejected mirroring retains ordinary input.

<ui:copy id="source-snippets-browser-tests">
import {expect,it,vi} from 'vitest';
import {EditorSelection,EditorState} from '@codemirror/state';
import {authoredSource,sourceText} from './SourceText';
import {EditorView} from '@codemirror/view';
import {history,undo,redo} from '@codemirror/commands';
import {acceptCompletion,completionStatus,setSelectedCompletion,CompletionContext} from '@codemirror/autocomplete';
import {waitFor} from '@testing-library/react';
import {completions,exclusiveCompletionSources} from '../lsp/completion';
import {beginSnippet} from './SnippetSession';
import {sourceSnippets,setSourceSnippet,readSourceSnippet,moveSourceSnippet,openSourceSnippetChoices,dispatchSourceSnippetTransactions,createSourceSnippetHistory,applyParallelSourceSnippet,sourceSnippetMenuState,readSourceEditorIntent,readParallelSourceSnippet} from './SourceSnippets';
import {parallelSelections} from './ParallelSnippet';
import type {SnippetField} from './SnippetExpansion';
import derived from '../../../simple-contract/snippet-derived-cases.json';
import {readSnippetTransform} from './SnippetTransform';
import {acceptSourceCompletion} from './SourceCompletion';
import {readCompletionAcceptance} from '../lsp/completionAcceptance';
import type {CompletionItem} from '../lsp/client';
const field=(id:number,index:number,from:number,to:number,choices:string[]=[]):SnippetField=>({id,index,from,to,choices,parent:null});
function fixture(source='a + a',fields=[field(0,1,0,1),field(1,1,4,5),field(2,0,5,5)],options:{writable?:()=>boolean;provider?:()=>Promise<never[]>}={}){
 const parent=document.body.appendChild(document.createElement('div')),notices:string[]=[];
 const view=new EditorView({parent,dispatchTransactions:dispatchSourceSnippetTransactions,state:EditorState.create({doc:source,extensions:[history(),sourceSnippets({writable:options.writable??(()=>true),notice:value=>notices.push(value)}),completions(options.provider?{project:options.provider}:{})]})});
 view.dispatch({selection:{anchor:fields[0].from,head:fields[0].to},effects:setSourceSnippet.of(beginSnippet(source,fields))});view.focus();
 return {view,notices,close:()=>{view.destroy();parent.remove();}};
}
it('actual input mirrors all fields and restores field state with one undo/redo',()=>{
 const {view,close}=fixture();try{
  view.dispatch({changes:{from:0,to:1,insert:'long'},selection:{anchor:4},userEvent:'input.type'});
  expect(view.state.doc.toString()).toBe('long + long');expect(view.state.selection.main.head).toBe(4);expect(readSourceSnippet(view)?.source).toBe('long + long');
  expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe('a + a');expect(readSourceSnippet(view)?.source).toBe('a + a');
  expect(redo(view)).toBe(true);expect(view.state.doc.toString()).toBe('long + long');expect(readSourceSnippet(view)?.source).toBe('long + long');expect(view.state.selection.main.anchor).toBe(4);expect(view.state.selection.main.head).toBe(4);
  expect(moveSourceSnippet(view,-1)).toBe(true);expect(readSourceSnippet(view)).not.toBeNull();
  expect(moveSourceSnippet(view,1)).toBe(true);expect(view.state.selection.main.head).toBe(11);expect(readSourceSnippet(view)).toBeNull();
 }finally{close();}
});
it('plain provider completion preserves all current mirrors and their history',async()=>{
 const {view,close}=fixture();try{const item:CompletionItem={label:'long',textEdit:{range:{start:{line:0,character:0},end:{line:0,character:1}},newText:'long'}};
  await acceptSourceCompletion(view,item,'a + a',view.state.selection,0,1,{uri:'hick:///a.py',resolve:async()=>item,current:()=>true,writable:()=>true,notice:()=>{}});
  expect(view.state.doc.toString()).toBe('long + long');expect(readSourceSnippet(view)?.source).toBe('long + long');expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe('a + a');expect(readSourceSnippet(view)?.source).toBe('a + a');
 }finally{close();}
});
it('nested completion fields mirror across outer occurrences and restore outer scope after undo',async()=>{
 const {view,close}=fixture();try{const syntax='${1:x}:$1$0',inner=[field(0,1,0,1),field(1,1,2,3),field(2,0,3,3)];
  const item:CompletionItem={label:'nested',insertTextFormat:2,textEdit:{range:{start:{line:0,character:0},end:{line:0,character:1}},newText:syntax},data:{hickory_snippet:{version:1,syntax,text:'x:x',fields:inner}},additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:'# import\n'}]};
  await acceptSourceCompletion(view,item,'a + a',view.state.selection,0,1,{uri:'hick:///a.py',resolve:async()=>item,current:()=>true,writable:()=>true,notice:()=>{}});
  expect(view.state.doc.toString()).toBe('# import\nx:x + x:x');expect(readSourceSnippet(view)?.scopes?.length).toBe(1);expect(view.state.selection.main.from).toBe(9);
  view.dispatch({changes:{from:9,to:10,insert:'long'},selection:{anchor:13},userEvent:'input.type'});expect(view.state.doc.toString()).toBe('# import\nlong:long + long:long');
  expect(undo(view)).toBe(true);expect(readSourceSnippet(view)?.scopes?.length).toBe(1);expect(redo(view)).toBe(true);expect(readSourceSnippet(view)?.scopes?.length).toBe(1);expect(view.state.selection.main.head).toBe(13);
  expect(moveSourceSnippet(view,1)).toBe(true);expect(view.state.selection.main.head).toBe(18);expect(readSourceSnippet(view)?.active).toBe(1);expect(moveSourceSnippet(view,1)).toBe(true);expect(view.state.selection.main.head).toBe(30);expect(readSourceSnippet(view)).toBeNull();
 }finally{close();}
});
it('source-equal completion retains template scope without a text history step',async()=>{
 const {view,close}=fixture();try{const item:CompletionItem={label:'a',insertText:'a'};
  await acceptSourceCompletion(view,item,'a + a',view.state.selection,0,1,{uri:'hick:///a.py',resolve:async()=>item,current:()=>true,writable:()=>true,notice:()=>{}});
  expect(view.state.doc.toString()).toBe('a + a');expect(readCompletionAcceptance(view)).toMatchObject({phase:'applied',source_changed:false});expect(readSourceSnippet(view)?.active).toBe(1);expect(undo(view)).toBe(false);
 }finally{close();}
});
it('completion imports apply once while the primary replacement updates every mirror',async()=>{
 const {view,close}=fixture();try{const item:CompletionItem={label:'long',textEdit:{range:{start:{line:0,character:0},end:{line:0,character:1}},newText:'long'},additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:'# import\n'},{range:{start:{line:0,character:5},end:{line:0,character:5}},newText:'\n# end'}]};
  await acceptSourceCompletion(view,item,'a + a',view.state.selection,0,1,{uri:'hick:///a.py',resolve:async()=>item,current:()=>true,writable:()=>true,notice:()=>{}});
  expect(view.state.doc.toString()).toBe('# import\nlong + long\n# end');expect(view.state.selection.main.head).toBe(13);expect(readSourceSnippet(view)?.fields.map(field=>[field.from,field.to])).toEqual([[9,13],[16,20],[20,20]]);
  expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe('a + a');expect(readSourceSnippet(view)?.source).toBe('a + a');expect(redo(view)).toBe(true);expect(view.state.selection.main.head).toBe(13);expect(readSourceSnippet(view)?.source).toBe(view.state.doc.toString());
 }finally{close();}
});
it('additional edits inside the active template forest refuse as a whole',async()=>{
 const {view,close}=fixture();try{const item:CompletionItem={label:'long',textEdit:{range:{start:{line:0,character:0},end:{line:0,character:1}},newText:'long'},additionalTextEdits:[{range:{start:{line:0,character:2},end:{line:0,character:2}},newText:'\nimport other'}]};
  await acceptSourceCompletion(view,item,'a + a',view.state.selection,0,1,{uri:'hick:///a.py',resolve:async()=>item,current:()=>true,writable:()=>true,notice:()=>{}});
  expect(view.state.doc.toString()).toBe('a + a');expect(readCompletionAcceptance(view)).toMatchObject({phase:'unavailable',applied:false,source_changed:false});expect(readSourceSnippet(view)?.source).toBe('a + a');
 }finally{close();}
});
it('equal-position field navigation retires a held completion intent',async()=>{
 const source='a',fields=[{...field(1,2,0,1),parent:0},field(0,1,0,1),field(2,0,1,1)],{view,close}=fixture(source,fields);
 let resolve!:(item:CompletionItem)=>void;const held=new Promise<CompletionItem>(done=>{resolve=done;});
 try{const selection=view.state.selection,item:CompletionItem={label:'long'},work=acceptSourceCompletion(view,item,source,selection,0,1,{uri:'hick:///a.py',resolve:()=>held,current:()=>true,writable:()=>true,notice:()=>{}});
  expect(moveSourceSnippet(view,1)).toBe(true);expect(view.state.selection.eq(selection)).toBe(true);expect(readSourceSnippet(view)?.active).toBe(2);resolve({label:'long',insertText:'long'});await work;
  expect(view.state.doc.toString()).toBe(source);expect(readCompletionAcceptance(view)?.phase).toBe('unavailable');
 }finally{close();}
});
it('smart-enter input retains its intended caret inside the linked replacement',()=>{
 const source='call() + call()',fields=[field(0,1,0,6),field(1,1,9,15),field(2,0,15,15)],{view,close}=fixture(source,fields);
 try{view.dispatch({selection:{anchor:5}});view.dispatch({changes:{from:5,insert:'\n  \n'},selection:{anchor:8},userEvent:'input.type'});expect(view.state.doc.toString()).toBe('call(\n  \n) + call(\n  \n)');expect(view.state.selection.main.head).toBe(8);expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe(source);}finally{close();}
});
it('source-local choices use the existing popup while providers are unavailable',async()=>{
 const provider=vi.fn(async()=>[] as never[]),source='first + first',fields=[field(0,1,0,5,['first','second']),field(1,1,8,13,['first','second']),field(2,0,13,13)],{view,close}=fixture(source,fields,{provider});
 try{expect(openSourceSnippetChoices(view)).toBe(true);await waitFor(()=>expect(completionStatus(view.state)).toBe('active'));expect(provider).not.toHaveBeenCalled();view.dispatch({effects:setSelectedCompletion(1)});await waitFor(()=>expect(acceptCompletion(view)).toBe(true));expect(view.state.doc.toString()).toBe('second + second');expect(view.state.selection.main.head).toBe(6);expect(readSourceSnippet(view)?.active).toBe(1);expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe(source);expect(readSourceSnippet(view)?.source).toBe(source);}finally{close();}
});
it('unavailable mirroring retains ordinary input and retires its field context',()=>{
 for(const saving of[false,true]){let writable=!saving;const {view,close}=fixture(undefined,undefined,{writable:()=>writable});try{
  if(!saving)view.dispatch({selection:{anchor:2}});
  view.dispatch({changes:{from:saving?0:2,to:saving?1:2,insert:'x'},selection:{anchor:saving?1:3},userEvent:'input.type'});
  expect(view.state.doc.toString()).toBe(saving?'x + a':'a x+ a');expect(readSourceSnippet(view)).toBeNull();writable=true;
 }finally{close();}}
});
it('coincident empty mirrors preserve every occurrence in actual editor input',()=>{
 const fields=[field(0,1,0,0),field(1,2,0,0),field(2,1,0,0),field(3,0,0,0)],{view,close}=fixture('',fields);
 try{view.dispatch({changes:{from:0,insert:'x'},selection:{anchor:1},userEvent:'input.type'});expect(view.state.doc.toString()).toBe('xx');expect(view.state.selection.main.head).toBe(1);expect(moveSourceSnippet(view,1)).toBe(true);expect(view.state.selection.main.head).toBe(1);view.dispatch({changes:{from:1,insert:'y'},selection:{anchor:2},userEvent:'input.type'});expect(view.state.doc.toString()).toBe('xyx');}finally{close();}
});
it('charges empty alternative overhead before retaining bounded snippet history',()=>{const cache=createSourceSnippetHistory(400),fields=[field(0,1,0,1,['','','','']),field(1,0,1,1)],session=beginSnippet('a',fields)!,snapshot={source:'a',session,selection:EditorState.create({doc:'a'}).selection};const first=cache.hold(snapshot),second=cache.hold(snapshot);expect(cache.read(first)).toBeUndefined();expect(cache.read(second)?.session?.fields[0].choices).toHaveLength(4);const oversized={...snapshot,session:beginSnippet('a',[field(0,1,0,1,Array<string>(16).fill('')),field(1,0,1,1)])!},rejected=cache.hold(oversized);expect(cache.read(rejected)).toBeUndefined();expect(cache.read(second)).toBeDefined();});
it('bounds empty no-session history by both fixed token cost and entry capacity',()=>{const snapshot={source:'',session:null,selection:EditorState.create().selection},count=createSourceSnippetHistory(400,2),a=count.hold(snapshot),b=count.hold(snapshot),c=count.hold(snapshot);expect(count.read(a)).toBeUndefined();expect(count.read(b)).toBeDefined();expect(count.read(c)).toBeDefined();const bytes=createSourceSnippetHistory(250),first=bytes.hold(snapshot),last=bytes.hold(snapshot);expect(bytes.read(first)).toBeUndefined();expect(bytes.read(last)).toBeDefined();});

it('computed input is one editor history action with exact source and selection replay',()=>{
 const transform=readSnippetTransform(derived.programs.upper),fields=[field(0,1,0,3),{...field(1,1,4,7),transform},field(2,0,7,7)],{view,close}=fixture('foo FOO',fields);
 try{view.dispatch({changes:{from:0,to:3,insert:'λ😀'},selection:{anchor:3},userEvent:'input.type'});expect(view.state.doc.toString()).toBe('λ😀 Λ😀');expect(view.state.selection.main.head).toBe(3);expect(readSourceSnippet(view)?.fields.find(field=>field.transform)?.to).toBe(7);expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe('foo FOO');expect(readSourceSnippet(view)).not.toBeNull();expect(redo(view)).toBe(true);expect(view.state.doc.toString()).toBe('λ😀 Λ😀');expect(view.state.selection.main.head).toBe(3);}finally{close()}
});
it('computed child selection retires linked intent and leaves ordinary parent input authoritative',()=>{
 const transform=readSnippetTransform(derived.programs.upper),fields=[field(0,1,0,7),{...field(1,2,0,3),parent:0},{...field(2,2,4,7),parent:0,transform},field(3,1,10,17),{...field(4,2,10,13),parent:3},{...field(5,2,14,17),parent:3,transform},field(6,0,17,17)],{view,close}=fixture('foo FOO | foo FOO',fields);
 try{expect(readSourceSnippet(view)).not.toBeNull();view.dispatch({selection:{anchor:4,head:7}});expect(readSourceSnippet(view)).toBeNull();view.dispatch({changes:{from:4,to:7,insert:'bar'},selection:{anchor:7},userEvent:'input.type'});expect(view.state.doc.toString()).toBe('foo bar | foo FOO');expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe('foo FOO | foo FOO');expect(readSourceSnippet(view)).toBeNull();}finally{close()}
});
it('computed program storage is charged before retaining or evicting editor history',()=>{
 const history=createSourceSnippetHistory(200),selection=EditorState.create().selection,token=history.hold({source:'',session:null,selection});const transform=readSnippetTransform(derived.programs.upper),session=beginSnippet('foo FOO',[field(0,1,0,3),{...field(1,1,4,7),transform},field(2,0,7,7)]);expect(session).not.toBeNull();const oversized=history.hold({source:'foo FOO',session,selection});expect(history.read(oversized)).toBeUndefined();expect(history.read(token)).toBeDefined();
});

it('old local choice objects cannot revive after undo or field-only scope restoration',async()=>{
 for(const mode of['undo','scope'] as const){const fields=[field(0,1,0,1,['a','long']),field(1,1,4,5,['a','long']),field(2,0,5,5)],{view,notices,close}=fixture('a + a',fields);try{const original=readSourceSnippet(view),source=view.state.facet(exclusiveCompletionSources)[0],result=await source(new CompletionContext(view.state,1,true)),choice=result!.options.find(item=>item.label==='long')!;if(mode==='undo'){view.dispatch({changes:{from:0,to:1,insert:'x'},selection:{anchor:1},userEvent:'input.type'});expect(undo(view)).toBe(true)}else{view.dispatch({effects:setSourceSnippet.of(null)});view.dispatch({effects:setSourceSnippet.of(original)})}if(mode==='scope')expect(readSourceSnippet(view)).toBe(original);else expect(readSourceSnippet(view)).toEqual(original);expect(typeof choice.apply).toBe('function');(choice.apply as Exclude<typeof choice.apply,string|undefined>)(view,choice,0,1);expect(view.state.doc.toString()).toBe('a + a');expect(notices.at(-1)).toContain('changed');}finally{close()}}
});
it('cosmetic popup effects keep a current source-local choice usable',async()=>{
 const fields=[field(0,1,0,1,['a','long']),field(1,1,4,5,['a','long']),field(2,0,5,5)],{view,close}=fixture('a + a',fields);try{const result=await view.state.facet(exclusiveCompletionSources)[0](new CompletionContext(view.state,1,true)),choice=result!.options.find(item=>item.label==='long')!;view.dispatch({effects:setSelectedCompletion(0)});(choice.apply as Exclude<typeof choice.apply,string|undefined>)(view,choice,0,1);expect(view.state.doc.toString()).toBe('long + long');expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe('a + a');expect(redo(view)).toBe(true);expect(view.state.doc.toString()).toBe('long + long')}finally{close()}
});

function parallelFixture(source:string,ranges:{from:number;to:number}[],primary:number){
 const parent=document.body.appendChild(document.createElement('div'));
 const view=new EditorView({parent,dispatchTransactions:dispatchSourceSnippetTransactions,state:EditorState.create({
  doc:source,extensions:[sourceText(source),history(),EditorState.allowMultipleSelections.of(true),sourceSnippets({writable:()=>true,notice:()=>{}})]
 })});
 view.dispatch({selection:EditorSelection.create(ranges.map(r=>EditorSelection.range(r.from,r.to)),primary)});
 return {view,close:()=>{view.destroy();parent.remove();}};
}
it('replays a multi-caret template and its linked input with the primary selection intact',()=>{
 const source='aa / bb',ranges=[{from:0,to:2},{from:5,to:7}],{view,close}=parallelFixture(source,ranges,1);
 try {
  const expansions=ranges.map((range,index)=>{
   const value=index?'beta':'a';
   return {...range,text:value+':'+value,fields:[
    field(0,1,0,value.length),field(1,1,value.length+1,2*value.length+1),
    field(2,0,2*value.length+1,2*value.length+1),
   ]};
  });
  const expired=readSourceEditorIntent(view.state)!;
  view.dispatch({selection:view.state.selection});
  const intent=readSourceEditorIntent(view.state)!;
  expect(applyParallelSourceSnippet(view,source,expansions,1,expired,()=>true)).toBe(false);
  expect(applyParallelSourceSnippet(view,source,expansions,1,intent,()=>false)).toBe(false);
  expect(applyParallelSourceSnippet(view,source,expansions,0,intent,()=>true)).toBe(false);
  expect(authoredSource(view.state)).toBe(source);
  expect(applyParallelSourceSnippet(view,source,expansions,1,intent,()=>true)).toBe(true);
  const beforeTyping=view.state.selection;
  expect(authoredSource(view.state)).toBe('a:a / beta:beta');
  expect(beforeTyping.mainIndex).toBe(1);
  view.dispatch({...view.state.replaceSelection('λ😀'),userEvent:'input.type'});
  expect(authoredSource(view.state)).toBe('λ😀:λ😀 / λ😀:λ😀');
  const afterTyping=view.state.selection;
  expect(afterTyping.mainIndex).toBe(1);
  expect(undo(view)).toBe(true);
  expect(authoredSource(view.state)).toBe('a:a / beta:beta');
  expect(view.state.selection.eq(beforeTyping)).toBe(true);
  expect(undo(view)).toBe(true);
  expect(authoredSource(view.state)).toBe(source);
  expect(sourceSnippetMenuState(view).active).toBe(false);
  expect(redo(view)).toBe(true);
  expect(sourceSnippetMenuState(view).active).toBe(true);
  expect(redo(view)).toBe(true);
  expect(view.state.selection.eq(afterTyping)).toBe(true);
  expect(moveSourceSnippet(view,1)).toBe(true);
  expect(view.state.selection.ranges.map(range=>range.head)).toEqual([7,17]);
  expect(moveSourceSnippet(view,-1)).toBe(true);
  expect(view.state.selection.ranges.map(range=>view.state.sliceDoc(range.from,range.to))).toEqual(['λ😀','λ😀']);
 } finally {close();}
});

it('keeps two logical empty peers and applies an insertion to both when the view shows one caret',()=>{
 const source='ab',ranges=[{from:0,to:1},{from:1,to:2}],{view,close}=parallelFixture(source,ranges,0);
 try {
  expect(applyParallelSourceSnippet(view,source,ranges.map(range=>({
   ...range,text:'',fields:[field(0,1,0,0),field(1,0,0,0)],
  })),0,readSourceEditorIntent(view.state)!,()=>true)).toBe(true);
  expect(view.state.selection.ranges).toHaveLength(1);
  view.dispatch({...view.state.replaceSelection('x'),userEvent:'input.type'});
  expect(authoredSource(view.state)).toBe('xx');
  expect(view.state.selection.ranges.map(range=>range.head)).toEqual([1,2]);
  expect(undo(view)).toBe(true);
  expect(view.state.doc.length).toBe(0);
  expect(sourceSnippetMenuState(view).active).toBe(true);
  const peers=parallelSelections(readParallelSourceSnippet(view.state)!);
  expect(peers.ranges).toEqual([{from:0,to:0},{from:0,to:0}]);
  expect(applyParallelSourceSnippet(view,'',peers.ranges.map(range=>({
   ...range,text:'q',fields:[field(0,1,0,1),field(1,0,1,1)],
  })),peers.primary,readSourceEditorIntent(view.state)!,()=>true)).toBe(true);
  expect(authoredSource(view.state)).toBe('qq');
  expect(view.state.selection.ranges).toHaveLength(2);
 } finally {close();}
});

it('continues both outer caret scopes through nested template typing, exits and undo',()=>{
 const source='aa / bb',ranges=[{from:0,to:2},{from:5,to:7}],{view,close}=parallelFixture(source,ranges,1);
 const expansion=(range:{from:number;to:number},value:string)=>({
  ...range,text:value+':'+value,fields:[
   field(0,1,0,value.length),field(1,1,value.length+1,2*value.length+1),
   field(2,0,2*value.length+1,2*value.length+1),
  ],
 });
 try {
  const append=(text:string,plans:ReturnType<typeof expansion>[]) =>
   applyParallelSourceSnippet(view,text,plans,1,readSourceEditorIntent(view.state)!,()=>true);
  expect(append(source,[expansion(ranges[0],'a'),expansion(ranges[1],'b')])).toBe(true);
  const outer=readParallelSourceSnippet(view.state)!;
  const context=parallelSelections(outer);
  expect(append(outer.source,context.ranges.map(range=>expansion(range,'q')))).toBe(true);
  const inner=readParallelSourceSnippet(view.state)!;
  expect(inner.source).toBe('q:q:q:q / q:q:q:q');
  expect(inner.peers.every(peer=>peer.session?.scopes?.length===1)).toBe(true);
  view.dispatch({...view.state.replaceSelection('λ'),userEvent:'input.type'});
  expect(authoredSource(view.state)).toBe('λ:λ:λ:λ / λ:λ:λ:λ');
  expect(moveSourceSnippet(view,1)).toBe(true);
  expect(readParallelSourceSnippet(view.state)?.peers.every(peer=>!peer.session?.scopes?.length)).toBe(true);
  expect(undo(view)).toBe(true);
  expect(readParallelSourceSnippet(view.state)?.peers.every(peer=>peer.session?.scopes?.length===1)).toBe(true);
  expect(undo(view)).toBe(true);
  expect(authoredSource(view.state)).toBe('a:a / b:b');
  expect(readParallelSourceSnippet(view.state)?.peers.every(peer=>!peer.session?.scopes?.length)).toBe(true);
 } finally {close();}
});
</ui:copy>

<ui:file path="apps/web/src/simple/SourceSnippets.test.ts"><ui:paste select="#source-snippets-browser-tests"/></ui:file>

## Drive actual AppKit field input and history

Mount the native editor and type through its delegate. Check complete mirrored text, field context on undo/redo, source-local choices without a language connection and ordinary input during saving. These are AppKit API witnesses, not physical-key, device or IME acceptance.

<ui:copy id="source-snippets-apple-tests">
import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class SourceSnippetInteractionTests:XCTestCase {
    func testComputedInputAndNativeHistoryAreOneAction()throws {
        #if os(macOS)
        let url=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().appendingPathComponent("simple-contract/snippet-derived-cases.json"),vectors=try JSONSerialization.jsonObject(with:Data(contentsOf:url)) as! [String:Any],programs=vectors["programs"] as! [String:Any],transform=try readSnippetTransform(programs["upper"]!)
        let source="foo FOO",values=[SnippetField(index:1,from:0,to:3,choices:[],id:0),SnippetField(index:1,from:4,to:7,choices:[],id:1,transform:transform),SnippetField(index:0,from:7,to:7,choices:[],id:2)]
        XCTAssertEqual(snippetHistoryCost(SourceSnippetHistoryContext(fields:values,active:1,current:0)),3*64+transform.bytes)
        let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:source,hash:"base")),editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:0,length:3));buffer.selection=editor.text.selectedRange();buffer.snippets.restore(beginSnippet(source,fields:values));editor.text.insertText("λ😀",replacementRange:editor.text.selectedRange());XCTAssertEqual(Array(buffer.text.utf8),Array("λ😀 Λ😀".utf8));XCTAssertEqual(editor.text.selectedRange(),NSRange(location:3,length:0));XCTAssertNotNil(buffer.snippets.active)
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);XCTAssertNotNil(buffer.snippets.active);editor.text.undoManager?.redo();XCTAssertEqual(Array(buffer.text.utf8),Array("λ😀 Λ😀".utf8));XCTAssertEqual(editor.text.selectedRange(),NSRange(location:3,length:0));XCTAssertNotNil(buffer.snippets.active)
        editor.text.setSelectedRange(NSRange(location:4,length:3));buffer.selection=editor.text.selectedRange();editor.text.insertText("bar",replacementRange:editor.text.selectedRange());XCTAssertEqual(Array(buffer.text.utf8),Array("λ😀 bar".utf8));XCTAssertNil(buffer.snippets.active)
        #endif
    }
    func fields()->[SnippetField]{[SnippetField(index:1,from:0,to:1,choices:["a","long"],id:0),SnippetField(index:1,from:4,to:5,choices:["a","long"],id:1),SnippetField(index:0,from:5,to:5,choices:[],id:2)]}
    func testActualMacParallelInputAndUndoRetainNativeSelections() throws {
        #if os(macOS)
        let source="aa / bb"
        let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:source,hash:"base"))
        let editor=MacEditor(buffer)
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=editor.scroll
        defer {window.contentView=nil}
        let ranges=[NSRange(location:0,length:2),NSRange(location:5,length:2)]
        XCTAssertTrue(buffer.selectMultiple(ranges,source:source))
        func expansion(_ name:String,_ range:NSRange)->ParallelSnippetExpansion {
            let n=name.utf16.count
            return ParallelSnippetExpansion(edit:NativeSourceEdit(range:range,text:name+":"+name),fields:[
                SnippetField(index:1,from:0,to:n,choices:[],id:0),
                SnippetField(index:1,from:n+1,to:n+n+1,choices:[],id:1),
                SnippetField(index:0,from:n+n+1,to:n+n+1,choices:[],id:2),
            ])
        }
        let expansions=[expansion("a",ranges[0]),expansion("beta",ranges[1])]
        let intent=buffer.snippets.intent
        XCTAssertFalse(buffer.snippets.applyParallel(source:source,expansions:expansions,primary:0,expectedIntent:intent,admitted:{false}))
        XCTAssertTrue(buffer.snippets.applyParallel(source:source,expansions:expansions,primary:0,expectedIntent:intent,admitted:{true}))
        let beforeInput=buffer.multiRanges
        XCTAssertEqual(buffer.text,"a:a / beta:beta")
        editor.text.insertText("λ😀",replacementRange:editor.text.selectedRange())
        let afterInput=buffer.multiRanges
        XCTAssertEqual(buffer.text,"λ😀:λ😀 / λ😀:λ😀")
        XCTAssertEqual(editor.text.selectedRange(),afterInput.first)
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,"a:a / beta:beta")
        XCTAssertEqual(buffer.multiRanges,beforeInput)
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,source)
        XCTAssertEqual(buffer.multiRanges,ranges)
        XCTAssertNil(buffer.snippets.parallel)
        editor.text.undoManager?.redo()
        XCTAssertNotNil(buffer.snippets.parallel)
        editor.text.undoManager?.redo()
        XCTAssertEqual(buffer.multiRanges,afterInput)
        XCTAssertTrue(buffer.snippets.move(1))
        XCTAssertEqual(buffer.multiRanges.map(\.location),[7,17])
        XCTAssertTrue(buffer.snippets.move(-1))
        let outer=try XCTUnwrap(buffer.snippets.parallel)
        let nested=outer.selections.map{expansion("q",$0)}
        XCTAssertTrue(buffer.snippets.applyParallel(source:outer.source,expansions:nested,
            primary:outer.primaryIndex,expectedIntent:buffer.snippets.intent,admitted:{true}))
        XCTAssertEqual(buffer.text,"q:q:q:q / q:q:q:q")
        XCTAssertTrue(buffer.snippets.parallel?.peers.allSatisfy{$0.session?.scopes.count==1}==true)
        XCTAssertTrue(buffer.snippets.move(1))
        XCTAssertTrue(buffer.snippets.parallel?.peers.allSatisfy{$0.session?.scopes.isEmpty==true}==true)
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,"λ😀:λ😀 / λ😀:λ😀")
        XCTAssertEqual(buffer.base,source)
        #endif
    }
    func testActualMacMarkedTextCommitsMirrorsAfterNativeCandidateAndKeepsEarlierUndo() {
        #if os(macOS)
        let saved="a + a"
        let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:saved,hash:"base"))
        let editor=MacEditor(buffer)
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=editor.scroll
        window.makeFirstResponder(editor.text)
        defer {window.contentView=nil}
        let prior=NSRange(location:0,length:0)
        XCTAssertTrue(editor.applySourceEdits([NativeSourceEdit(range:prior,text:"!")],source:saved))
        let source="!a + a"
        let choice=NSRange(location:1,length:1)
        XCTAssertTrue(editor.selectSourceRange(choice,source:source))
        let fields=[
            SnippetField(index:1,from:1,to:2,choices:[],id:0),
            SnippetField(index:1,from:5,to:6,choices:[],id:1),
            SnippetField(index:0,from:6,to:6,choices:[],id:2),
        ]
        buffer.snippets.restore(beginSnippet(source,fields:fields))
        let mark=NSRange(location:NSNotFound,length:0)
        editor.text.setMarkedText("に",selectedRange:NSRange(location:1,length:0),replacementRange:mark)
        XCTAssertTrue(editor.text.hasMarkedText())
        XCTAssertEqual(buffer.text,"!に + a")
        editor.text.setMarkedText("日本",selectedRange:NSRange(location:2,length:0),replacementRange:mark)
        XCTAssertTrue(editor.text.hasMarkedText())
        XCTAssertEqual(buffer.text,"!日本 + a")
        editor.text.insertText("日本語",replacementRange:mark)
        XCTAssertFalse(editor.text.hasMarkedText())
        XCTAssertEqual(buffer.text,"!日本語 + 日本語")
        XCTAssertEqual(editor.text.selectedRange(),NSRange(location:4,length:0))
        XCTAssertNotNil(buffer.snippets.active)
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,source)
        XCTAssertEqual(editor.text.selectedRange(),choice)
        XCTAssertNotNil(buffer.snippets.active)
        editor.text.undoManager?.redo()
        XCTAssertEqual(buffer.text,"!日本語 + 日本語")
        editor.text.undoManager?.undo()
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,saved)
        XCTAssertEqual(buffer.base,saved)
        #endif
    }

    func testMacUnmarkRetainsOrdinaryInputWhenSnippetIntentChanged() {
        #if os(macOS)
        let source="a + a"
        let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:source,hash:"base"))
        let editor=MacEditor(buffer)
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=editor.scroll
        window.makeFirstResponder(editor.text)
        defer {window.contentView=nil}
        XCTAssertTrue(editor.selectSourceRange(NSRange(location:0,length:1),source:source))
        buffer.snippets.restore(beginSnippet(source,fields:fields()))
        let mark=NSRange(location:NSNotFound,length:0)
        editor.text.setMarkedText("に",selectedRange:NSRange(location:1,length:0),replacementRange:mark)
        XCTAssertEqual(buffer.text,"に + a")
        buffer.snippets.restore(nil)
        editor.text.unmarkText()
        XCTAssertFalse(editor.text.hasMarkedText())
        XCTAssertEqual(buffer.text,"に + a")
        XCTAssertNil(buffer.snippets.active)
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,source)
        #endif
    }

    func testHistoryCostChargesEveryEmptyChoice() {
        let field=SnippetField(index:1,from:0,to:0,choices:Array(repeating:"",count:4096),id:0)
        XCTAssertEqual(snippetHistoryCost(SourceSnippetHistoryContext(fields:[field],active:1,current:0)),64+4096*32)
        var unicode=field;unicode.choices=["😀e\u{301}"]
        XCTAssertEqual(snippetHistoryCost(SourceSnippetHistoryContext(fields:[unicode],active:1,current:0)),64+32+7)
    }
    func testActualMacInputAndHistoryCarryFieldState()throws {
        #if os(macOS)
        let source="a + a",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"a + a",hash:"base")),editor:MacEditor
        editor=MacEditor(buffer);let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:0,length:1));buffer.selection=editor.text.selectedRange();buffer.snippets.restore(beginSnippet(source,fields:fields()))
        XCTAssertTrue(editor.textView(editor.text,doCommandBy:NSSelectorFromString("insertBacktab:")));XCTAssertEqual(buffer.text,source);XCTAssertNotNil(buffer.snippets.active)
        editor.text.insertText("long",replacementRange:editor.text.selectedRange())
        XCTAssertEqual(buffer.text,"long + long");XCTAssertEqual(editor.text.selectedRange(),NSRange(location:4,length:0));XCTAssertEqual(buffer.snippets.active?.source,"long + long");XCTAssertEqual(buffer.base,source)
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.snippets.active?.source,source)
        editor.text.undoManager?.redo();XCTAssertEqual(buffer.text,"long + long");XCTAssertEqual(buffer.snippets.active?.source,"long + long")
        XCTAssertTrue(buffer.snippets.move(1));XCTAssertEqual(editor.text.selectedRange().location,11);XCTAssertNil(buffer.snippets.active)
        #endif
    }
    func testPreparedNativeEnterMirrorsItsTextAndCaret()throws {
        #if os(macOS)
        let source="call() + call()",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"call() + call()",hash:"base")),editor:MacEditor
        editor=MacEditor(buffer);let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{window.contentView=nil}
        let fields=[SnippetField(index:1,from:0,to:6,choices:[],id:0),SnippetField(index:1,from:9,to:15,choices:[],id:1),SnippetField(index:0,from:15,to:15,choices:[],id:2)]
        editor.text.setSelectedRange(NSRange(location:5,length:0));buffer.selection=editor.text.selectedRange();buffer.snippets.restore(beginSnippet(source,fields:fields))
        XCTAssertEqual(buffer.snippets.input(source:source,range:NSRange(location:5,length:0),text:"\n  \n",caret:8),true);XCTAssertEqual(buffer.text,"call(\n  \n) + call(\n  \n)");XCTAssertEqual(editor.text.selectedRange().location,8)
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);editor.text.undoManager?.redo();XCTAssertEqual(editor.text.selectedRange().location,8)
        #endif
    }
    func testProviderCompletionCombinesImportsAndRefusesInteriorEdits()async throws {
        #if os(macOS)
        let source="a + a",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"a + a",hash:"base")),service:NativeLanguage
        service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{service.close();window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:0,length:1));buffer.selection=editor.text.selectedRange();buffer.snippets.restore(beginSnippet(source,fields:fields()))
        let item=NativeCompletion(id:1,label:"long",detail:"",range:NSRange(location:0,length:1),insert:"long",source:source,selection:buffer.selection,original:["label":"long"])
        let edit:[String:Any]=["range":["start":["line":0,"character":0],"end":["line":0,"character":1]],"newText":"long"]
        let accepted=await service.accept(item,resolution:{_ in ["label":"long","textEdit":edit]});XCTAssertTrue(accepted);XCTAssertEqual(buffer.text,"long + long");XCTAssertEqual(buffer.snippets.active?.source,"long + long")
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.snippets.active?.source,source)
        let imported=await service.accept(item,resolution:{_ in ["label":"long","textEdit":edit,"additionalTextEdits":[["range":["start":["line":0,"character":0],"end":["line":0,"character":0]],"newText":"# import\n"],["range":["start":["line":0,"character":5],"end":["line":0,"character":5]],"newText":"\n# end"]]]})
        XCTAssertTrue(imported);XCTAssertEqual(buffer.text,"# import\nlong + long\n# end");XCTAssertEqual(editor.text.selectedRange().location,13);XCTAssertEqual(buffer.snippets.active?.fields.map{$0.from},[9,16,20])
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.snippets.active?.source,source);editor.text.undoManager?.redo();XCTAssertEqual(editor.text.selectedRange().location,13);editor.text.undoManager?.undo()
        let refused=await service.accept(item,resolution:{_ in ["label":"long","textEdit":edit,"additionalTextEdits":[["range":["start":["line":0,"character":2],"end":["line":0,"character":2]],"newText":"\nimport other"]]]})
        XCTAssertFalse(refused);XCTAssertEqual(buffer.text,source);XCTAssertTrue(service.completionNotice.contains("overlaps template"))
        #endif
    }
    func testNestedCompletionRestoresScopeThroughNativeHistory()async throws {
        #if os(macOS)
        let source="a + a",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:source,hash:"base")),service:NativeLanguage
        service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:0,length:1));buffer.selection=editor.text.selectedRange();buffer.snippets.restore(beginSnippet(source,fields:fields()))
        let syntax="${1:x}:$1$0",inner:[[String:Any]]=[["id":0,"parent":NSNull(),"index":1,"from":0,"to":1,"choices":[]],["id":1,"parent":NSNull(),"index":1,"from":2,"to":3,"choices":[]],["id":2,"parent":NSNull(),"index":0,"from":3,"to":3,"choices":[]]]
        let raw:[String:Any]=["label":"nested","insertTextFormat":2,"textEdit":["range":["start":["line":0,"character":0],"end":["line":0,"character":1]],"newText":syntax],"data":["hickory_snippet":["version":1,"syntax":syntax,"text":"x:x","fields":inner]]]
        let item=NativeCompletion(id:2,label:"nested",detail:"",range:NSRange(location:0,length:1),insert:syntax,source:source,selection:buffer.selection,original:raw)
        let accepted=await service.accept(item,resolution:{_ in raw});XCTAssertTrue(accepted);XCTAssertEqual(buffer.text,"x:x + x:x");XCTAssertEqual(buffer.snippets.active?.scopes.count,1)
        XCTAssertEqual(buffer.snippets.input(source:buffer.text,range:NSRange(location:0,length:1),text:"long"),true);XCTAssertEqual(buffer.text,"long:long + long:long")
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.snippets.active?.scopes.count,1);editor.text.undoManager?.redo();XCTAssertEqual(buffer.snippets.active?.scopes.count,1);XCTAssertEqual(editor.text.selectedRange().location,4)
        XCTAssertTrue(buffer.snippets.move(1));XCTAssertEqual(editor.text.selectedRange().location,9);XCTAssertEqual(buffer.snippets.active?.active,1);XCTAssertTrue(buffer.snippets.move(1));XCTAssertEqual(editor.text.selectedRange().location,21);XCTAssertNil(buffer.snippets.active)
        #endif
    }
    func testEqualPositionNavigationRetiresHeldCompletion()async throws {
        #if os(macOS)
        let source="a",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"a",hash:"base")),service:NativeLanguage
        service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{service.close();window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:0,length:1));buffer.selection=editor.text.selectedRange();let selected=buffer.selection
        buffer.snippets.restore(beginSnippet(source,fields:[SnippetField(index:2,from:0,to:1,choices:[],id:1,parent:0),SnippetField(index:1,from:0,to:1,choices:[],id:0),SnippetField(index:0,from:1,to:1,choices:[],id:2)]))
        var held:CheckedContinuation<[String:Any],Error>?
        let work=Task{await service.accept(NativeCompletion(id:1,label:"long",detail:"",range:selected,insert:"long",source:source,selection:selected,original:["label":"long"]),resolution:{_ in try await withCheckedThrowingContinuation{held=$0}})}
        let end=Date().addingTimeInterval(2);while held==nil&&Date()<end{await Task.yield()}
        XCTAssertTrue(buffer.snippets.move(1));XCTAssertEqual(buffer.selection,selected);try XCTUnwrap(held).resume(returning:["label":"long","insertText":"long"]);let accepted=await work.value;XCTAssertFalse(accepted);XCTAssertEqual(buffer.text,source)
        #endif
    }
    func testActualMacChoiceNeedsNoLanguageConnection()throws {
        #if os(macOS)
        let source="a + a",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"a + a",hash:"base")),editor:MacEditor
        editor=MacEditor(buffer);let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:0,length:1));buffer.selection=editor.text.selectedRange();buffer.snippets.restore(beginSnippet(source,fields:fields()))
        XCTAssertTrue(buffer.snippets.choose("long"));XCTAssertEqual(buffer.text,"long + long");XCTAssertEqual(editor.text.selectedRange().location,4);XCTAssertNil(buffer.language)
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.snippets.active?.source,source)
        #endif
    }
    func testActualMacAcceptsSourceEqualCompletion()async throws {
        #if os(macOS)
        let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"va",hash:"base")),service:NativeLanguage
        service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{service.close();window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:2,length:0));buffer.selection=editor.text.selectedRange()
        let item=NativeCompletion(id:1,label:"va",detail:"",range:NSRange(location:0,length:2),insert:"va",source:"va",selection:buffer.selection,original:["label":"va"])
        let accepted=await service.accept(item,resolution:{_ in ["label":"va","textEdit":["range":["start":["line":0,"character":0],"end":["line":0,"character":2]],"newText":"va"]]})
        XCTAssertTrue(accepted);XCTAssertEqual(buffer.text,"va");XCTAssertEqual(buffer.base,"va");XCTAssertFalse(editor.text.undoManager?.canUndo ?? false)
        #endif
    }
    func testSavingRetiresMirroringAndLeavesOrdinaryNativeInput()throws {
        #if os(macOS)
        let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"a + a",hash:"base")),editor:MacEditor
        editor=MacEditor(buffer);let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:0,length:1));buffer.selection=editor.text.selectedRange();buffer.snippets.restore(beginSnippet(buffer.text,fields:fields()));buffer.saving=true
        editor.text.insertText("x",replacementRange:editor.text.selectedRange());XCTAssertEqual(buffer.text,"x + a");XCTAssertNil(buffer.snippets.active)
        #endif
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/SourceSnippetInteractionTests.swift"><ui:paste select="#source-snippets-apple-tests"/></ui:file>

## Drive the production Android input transformation

Use the actual native text state and the same transformation helper the widget calls. Confirm complete mirrored text, caret and source undo/redo, local choices, and ordinary primary input when mirroring cannot reflect hidden source or the buffer is saving. Device gestures and IME behavior remain separate acceptance.

<ui:copy id="source-snippets-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import androidx.compose.ui.text.TextRange
import kotlinx.coroutines.async
class SourceSnippetInteractionTest {
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun markedTemplateCandidatesStayNativeThenUndoAsOneSourceAction() {
  val buffer=buffer()
  val controller=buffer.snippets
  val connection=Any()
  val original=buffer.text
  controller.finish()
  assertTrue(buffer.applySourceEdits(listOf(NativeSourceEdit(0,0,"!")),original,TextRange(1,2)))
  controller.restore(beginSnippet(buffer.text,fields().map{it.copy(from=it.from+1,to=it.to+1)}))
  // These edits exercise the native TextFieldState and the controller's event
  // boundary. The OS input connection itself requires a device IME.
  controller.beginMarkedInput(connection)
  assertTrue(controller.retainsMarkedInput)
  buffer.state.edit { replace(1,2,"に");selection=TextRange(2) }
  controller.observeMarkedInput(buffer.text)
  assertEquals("!に + a",buffer.text)
  buffer.state.edit { replace(1,2,"日本");selection=TextRange(3) }
  controller.observeMarkedInput(buffer.text)
  assertEquals("!日本 + a",buffer.text)
  controller.settleMarkedInput(connection)
  assertEquals("!日本 + a",buffer.text)
  controller.markCommitRequested(connection)
  controller.settleMarkedInput(connection)
  assertEquals("!日本 + 日本",buffer.text)
  assertEquals(TextRange(3),buffer.state.selection)
  assertTrue(controller.undoSource())
  assertEquals("!a + a",buffer.text)
  assertEquals(TextRange(1,2),buffer.state.selection)
  assertNotNull(controller.active)
  assertTrue(controller.redoSource())
  assertEquals("!日本 + 日本",buffer.text)
  assertNotNull(controller.active)
  assertTrue(controller.undoSource())
  assertTrue(controller.undoSource())
  assertEquals(original,buffer.text)
  assertEquals(original,buffer.base)
 }

 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun closingMarkedConnectionKeepsOrdinaryDraftAndRejectsLateMirrors() {
  val buffer=buffer()
  val controller=buffer.snippets
  val closed=Any()
  controller.beginMarkedInput(closed)
  buffer.state.edit { replace(0,1,"に");selection=TextRange(1) }
  controller.observeMarkedInput(buffer.text)
  controller.settleMarkedInput(closed,closing=true)
  controller.markCommitRequested(closed)
  controller.settleMarkedInput(closed)
  assertEquals("に + a",buffer.text)
  assertNull(controller.active)
  assertTrue(controller.undoSource())
  assertEquals("a + a",buffer.text)
 }
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun parallelTemplateRetainsBothSelectionsThroughNativeTextStateHistory() {
  val source="aa / bb"
  val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"))
  val ranges=listOf(NativeSourceEdit(0,2,""),NativeSourceEdit(5,7,""))
  assertTrue(buffer.selectMultiple(ranges,source))
  fun expansion(range:NativeSourceEdit,value:String):ParallelSnippetExpansion {
   val n=value.length
   return ParallelSnippetExpansion(range.copy(text="$value:$value"),listOf(
    SnippetField(1,0,n,emptyList(),0),SnippetField(1,n+1,2*n+1,emptyList(),1),
    SnippetField(0,2*n+1,2*n+1,emptyList(),2),
   ))
  }
  val expansions=listOf(expansion(ranges[0],"a"),expansion(ranges[1],"beta"))
  val intent=buffer.snippets.intent
  assertFalse(buffer.snippets.applyParallel(source,expansions,0,intent){false})
  assertTrue(buffer.snippets.applyParallel(source,expansions,0,intent){true})
  val before=buffer.multiRanges
  assertEquals("a:a / beta:beta",buffer.text)
  assertTrue(buffer.snippets.inputParallel("λ😀"))
  val after=buffer.multiRanges
  assertEquals("λ😀:λ😀 / λ😀:λ😀",buffer.text)
  assertEquals(TextRange(3),buffer.state.selection)
  assertTrue(buffer.snippets.undoSource())
  assertEquals(before,buffer.multiRanges)
  assertTrue(buffer.snippets.undoSource())
  assertEquals(source,buffer.text)
  assertEquals(ranges,buffer.multiRanges)
  assertNull(buffer.snippets.parallel)
  assertTrue(buffer.snippets.redoSource())
  assertNotNull(buffer.snippets.parallel)
  assertTrue(buffer.snippets.redoSource())
  assertEquals(after,buffer.multiRanges)
  assertTrue(buffer.snippets.move(1))
  assertEquals(listOf(7,17),buffer.multiRanges.map{it.from})
  assertTrue(buffer.snippets.move(-1))
  val outer=buffer.snippets.parallel!!
  val nested=outer.selections.map{expansion(NativeSourceEdit(it.from,it.to,""),"q")}
  assertTrue(buffer.snippets.applyParallel(outer.source,nested,outer.primaryIndex,buffer.snippets.intent){true})
  assertEquals("q:q:q:q / q:q:q:q",buffer.text)
  assertTrue(buffer.snippets.parallel!!.peers.all{it.session?.scopes?.size==1})
  assertTrue(buffer.snippets.move(1))
  assertTrue(buffer.snippets.parallel!!.peers.all{it.session?.scopes?.isEmpty()==true})
  assertTrue(buffer.snippets.undoSource())
  assertEquals("λ😀:λ😀 / λ😀:λ😀",buffer.text)
  assertEquals(source,buffer.base)
 }
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun computedInputAndNativeHistoryAreOneAction(){
  val root=generateSequence(java.io.File(System.getProperty("user.dir")).absoluteFile){it.parentFile}.first{it.resolve("apps/simple-contract/snippet-derived-cases.json").isFile};val transform=readSnippetTransform(org.json.JSONObject(root.resolve("apps/simple-contract/snippet-derived-cases.json").readText()).getJSONObject("programs").getJSONObject("upper"));val source="foo FOO";val values=listOf(SnippetField(1,0,3,emptyList(),0),SnippetField(1,4,7,emptyList(),1,transform=transform),SnippetField(0,7,7,emptyList(),2));val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"));buffer.state.edit{selection=TextRange(0,3)};buffer.snippets.restore(beginSnippet(source,values));assertEquals(3*64L+transform.bytes,snippetHistoryCost(buffer.snippets.active));assertTrue(buffer.snippets.applyInput(source,0,3,"λ😀"));assertEquals("λ😀 Λ😀",buffer.text);assertEquals(TextRange(3),buffer.state.selection);assertNotNull(buffer.snippets.active);assertTrue(buffer.snippets.undoSource());assertEquals(source,buffer.text);assertNotNull(buffer.snippets.active);assertTrue(buffer.snippets.redoSource());assertEquals("λ😀 Λ😀",buffer.text);assertEquals(TextRange(3),buffer.state.selection)
  var staged:SnippetSession?=buffer.snippets.active;buffer.state.edit{val before=buffer.snippets.captureBefore(originalText,originalSelection);replace(4,7,"bar");selection=TextRange(7);assertFalse(applySourceSnippetInput(buffer,publish={staged=it}));buffer.snippets.finishTextChange(before,asCharSequence(),selection)};buffer.snippets.publish(staged);assertEquals("λ😀 bar",buffer.text);assertNull(buffer.snippets.active)
 }

 private fun fields()=listOf(SnippetField(1,0,1,listOf("a","long"),0),SnippetField(1,4,5,listOf("a","long"),1),SnippetField(0,5,5,emptyList(),2))
 private fun buffer()=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText("a + a","base")).also{it.state.edit{selection=TextRange(0,1)};it.snippets.restore(beginSnippet(it.text,fields()))}
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun actualTextStateUsesTheProductionInputTransformation() {
  val buffer=buffer();var staged:SnippetSession?=null
  buffer.state.edit{val before=buffer.snippets.captureBefore(originalText,originalSelection);replace(0,1,"long");selection=TextRange(4);assertTrue(applySourceSnippetInput(buffer,publish={staged=it}));buffer.snippets.finishTextChange(before,asCharSequence(),selection)};buffer.snippets.publish(staged)
  assertEquals("long + long",buffer.text);assertEquals(TextRange(4),buffer.state.selection);assertEquals(buffer.text,buffer.snippets.active?.source);assertEquals("a + a",buffer.base)
  buffer.snippets.undoSource();assertEquals("a + a",buffer.text);assertEquals(buffer.text,buffer.snippets.active?.source);buffer.snippets.redoSource();assertEquals("long + long",buffer.text);assertEquals(buffer.text,buffer.snippets.active?.source)
  assertTrue(buffer.snippets.move(-1));assertNotNull(buffer.snippets.active);assertTrue(buffer.snippets.move(1));assertEquals(TextRange(11),buffer.state.selection);assertNull(buffer.snippets.active)
 }
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun preparedNativeEnterPreservesLinkedTextAndCaret(){
  val source="call() + call()";val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"));buffer.state.edit{selection=TextRange(5)}
  buffer.snippets.restore(beginSnippet(source,listOf(SnippetField(1,0,6,emptyList(),0),SnippetField(1,9,15,emptyList(),1),SnippetField(0,15,15,emptyList(),2))));var staged:SnippetSession?=null
  buffer.state.edit{val before=buffer.snippets.captureBefore(originalText,originalSelection);replace(5,5,"\n");selection=TextRange(6);assertTrue(applySourceSnippetInput(buffer,prepare={_,_,_,_->SnippetPrimaryEdit(NativeSourceEdit(5,5,"\n  \n"),8)},publish={staged=it}));buffer.snippets.finishTextChange(before,asCharSequence(),selection)};buffer.snippets.publish(staged)
  assertEquals("call(\n  \n) + call(\n  \n)",buffer.text);assertEquals(TextRange(8),buffer.state.selection);buffer.snippets.undoSource();assertEquals(source,buffer.text);buffer.snippets.redoSource();assertEquals(TextRange(8),buffer.state.selection)
 }
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun providerCompletionCombinesImportsAndRefusesInteriorEdits()=kotlinx.coroutines.runBlocking {
  val buffer=buffer();val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service
  val item=NativeLanguageCompletion("long","",NativeSourceEdit(0,1,"long"),"a + a",buffer.state.selection,org.json.JSONObject().put("label","long"))
  val result=org.json.JSONObject("{\"label\":\"long\",\"textEdit\":{\"range\":{\"start\":{\"line\":0,\"character\":0},\"end\":{\"line\":0,\"character\":1}},\"newText\":\"long\"}}")
  try{assertTrue(service.accept(item){result});assertEquals("long + long",buffer.text);assertEquals(buffer.text,buffer.snippets.active?.source)
   buffer.snippets.undoSource();assertEquals("a + a",buffer.text);assertEquals(buffer.text,buffer.snippets.active?.source)
   val additions=org.json.JSONArray().put(org.json.JSONObject().put("range",org.json.JSONObject().put("start",org.json.JSONObject().put("line",0).put("character",0)).put("end",org.json.JSONObject().put("line",0).put("character",0))).put("newText","# import\n")).put(org.json.JSONObject().put("range",org.json.JSONObject().put("start",org.json.JSONObject().put("line",0).put("character",5)).put("end",org.json.JSONObject().put("line",0).put("character",5))).put("newText","\n# end"))
   assertTrue(service.accept(item){org.json.JSONObject(result.toString()).put("additionalTextEdits",additions)});assertEquals("# import\nlong + long\n# end",buffer.text);assertEquals(TextRange(13),buffer.state.selection);assertEquals(listOf(9,16,20),buffer.snippets.active?.fields?.map{it.from})
   buffer.snippets.undoSource();assertEquals("a + a",buffer.text);assertEquals(buffer.text,buffer.snippets.active?.source);buffer.snippets.redoSource();assertEquals(TextRange(13),buffer.state.selection);buffer.snippets.undoSource()
   val refusal=org.json.JSONObject(result.toString()).put("additionalTextEdits",org.json.JSONArray("[{\"range\":{\"start\":{\"line\":0,\"character\":2},\"end\":{\"line\":0,\"character\":2}},\"newText\":\"\\nimport other\"}]"))
   assertFalse(service.accept(item){refusal});assertEquals("a + a",buffer.text);assertTrue(service.completionNotice.contains("overlaps template"))
  }finally{service.close()}
 }
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun nestedCompletionRestoresScopeThroughNativeHistory()=kotlinx.coroutines.runBlocking {
  val buffer=buffer();val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service
  val syntax="\${1:x}:\$1\$0";val raw=org.json.JSONObject("""{"label":"nested","insertTextFormat":2,"textEdit":{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"newText":"placeholder"},"data":{"hickory_snippet":{"version":1,"syntax":"placeholder","text":"x:x","fields":[{"id":0,"parent":null,"index":1,"from":0,"to":1,"choices":[]},{"id":1,"parent":null,"index":1,"from":2,"to":3,"choices":[]},{"id":2,"parent":null,"index":0,"from":3,"to":3,"choices":[]}]}}}""")
  raw.getJSONObject("textEdit").put("newText",syntax);raw.getJSONObject("data").getJSONObject("hickory_snippet").put("syntax",syntax)
  val item=NativeLanguageCompletion("nested","",NativeSourceEdit(0,1,syntax),"a + a",buffer.state.selection,raw)
  try{assertTrue(service.accept(item){raw});assertEquals("x:x + x:x",buffer.text);assertEquals(1,buffer.snippets.active?.scopes?.size)
   assertTrue(buffer.snippets.applyInput(buffer.text,0,1,"long"));assertEquals("long:long + long:long",buffer.text)
   buffer.snippets.undoSource();assertEquals(1,buffer.snippets.active?.scopes?.size);buffer.snippets.redoSource();assertEquals(1,buffer.snippets.active?.scopes?.size);assertEquals(TextRange(4),buffer.state.selection)
   assertTrue(buffer.snippets.move(1));assertEquals(TextRange(9),buffer.state.selection);assertEquals(1,buffer.snippets.active?.active);assertTrue(buffer.snippets.move(1));assertEquals(TextRange(21),buffer.state.selection);assertNull(buffer.snippets.active)
  }finally{service.close()}
 }
 @Test fun equalPositionNavigationRetiresHeldCompletion()=kotlinx.coroutines.runBlocking {
  val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText("a","base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(0,1)}
  buffer.snippets.restore(beginSnippet("a",listOf(SnippetField(2,0,1,emptyList(),1,0),SnippetField(1,0,1,emptyList(),0),SnippetField(0,1,1,emptyList(),2))))
  val entered=kotlinx.coroutines.CompletableDeferred<Unit>();val held=kotlinx.coroutines.CompletableDeferred<org.json.JSONObject>();val selected=buffer.state.selection
  val work=kotlinx.coroutines.CoroutineScope(coroutineContext).async{service.accept(NativeLanguageCompletion("long","",NativeSourceEdit(0,1,"long"),"a",selected,org.json.JSONObject().put("label","long"))){entered.complete(Unit);held.await()}}
  try{entered.await();assertTrue(buffer.snippets.move(1));assertEquals(selected,buffer.state.selection);held.complete(org.json.JSONObject().put("label","long").put("insertText","long"));assertFalse(work.await());assertEquals("a",buffer.text)}finally{service.close()}
 }
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun sourceLocalChoiceIsOneDraftUndo() {
  val buffer=buffer();assertNull(buffer.language);assertTrue(buffer.snippets.choose("long"));assertEquals("long + long",buffer.text);assertEquals(TextRange(4),buffer.state.selection)
  buffer.snippets.undoSource();assertEquals("a + a",buffer.text);buffer.snippets.redoSource();assertEquals("long + long",buffer.text)
 }
 @Test fun acceptsSourceEqualCompletion()=kotlinx.coroutines.runBlocking {
  val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText("va","base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(2)}
  val original=org.json.JSONObject().put("label","va");val result=org.json.JSONObject("{\"label\":\"va\",\"textEdit\":{\"range\":{\"start\":{\"line\":0,\"character\":0},\"end\":{\"line\":0,\"character\":2}},\"newText\":\"va\"}}")
  try{assertTrue(service.accept(NativeLanguageCompletion("va","",NativeSourceEdit(0,2,"va"),"va",buffer.state.selection,original)){result});assertEquals("va",buffer.text);assertEquals("va",buffer.base)}finally{service.close()}
 }
 @Test fun unavailableMirroringRetainsOrdinaryInput() {
  for(hidden in listOf(false,true)){val buffer=buffer();buffer.saving=!hidden;var staged:SnippetSession?=null
   buffer.state.edit{replace(0,1,"x");selection=TextRange(1);assertFalse(applySourceSnippetInput(buffer,canReflect={!hidden},publish={staged=it}))};buffer.snippets.publish(staged)
   assertEquals("x + a",buffer.text);assertNull(buffer.snippets.active)
  }
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SourceSnippetInteractionTest.kt"><ui:paste select="#source-snippets-android-tests"/></ui:file>

## Restore optional context after authoritative native undo

The native text state remains the only owner of source undo/redo. This typed journal records before/after context alongside exact text and directional selections. An explicit undo/redo restores context only when one recorded state matches the actual native result and the retained chain starts at the prior source. Coalesced native edits may traverse several recorded steps; ambiguous, stale or evicted context is retired while source undo still succeeds. Estimated source/context retention is bounded independently of native text history. No context lookup writes source.

<ui:copy id="source-undo-context-journal-android">
package com.hickorydocs.simple
// The native text state owns text undo. This bounded journal restores optional context only.
data class SourceUndoSnapshot<C>(val source:String,val anchor:Int,val head:Int,val context:C)
class SourceUndoContexts<C>(private val contextCost:(C)->Long,private val budget:Long=32L*1024*1024) {
 private data class Step<C>(
  val before:SourceUndoSnapshot<C>,val after:SourceUndoSnapshot<C>,val marked:MarkedSourceReplay?=null
 )
 private val past=mutableListOf<Step<C>>();private val future=mutableListOf<Step<C>>()
 var serial=0L;private set
 val retained:Boolean get()=past.isNotEmpty()||future.isNotEmpty()
 init{require(budget in 0 until Long.MAX_VALUE)}
 private fun sum(vararg values:Long):Long {var total=0L;for(value in values){if(value<0||value>budget-total)return budget+1;total+=value};return total}
 private fun cost(step:Step<C>)=sum(2L*step.before.source.length,2L*step.after.source.length,contextCost(step.before.context),contextCost(step.after.context),step.marked?.retainedBytes ?: 0)
 private fun trim(){var bytes=past.sumOf{cost(it)}+future.sumOf{cost(it)};while((bytes>budget||past.size+future.size>4096)&&past.isNotEmpty()){bytes-=cost(past.removeAt(0))};while((bytes>budget||past.size+future.size>4096)&&future.isNotEmpty()){bytes-=cost(future.removeAt(future.lastIndex))}}
 private fun clear(){past.clear();future.clear()}
 fun record(before:SourceUndoSnapshot<C>,after:SourceUndoSnapshot<C>) = recordMarked(before,after,null)
 internal fun recordMarked(before:SourceUndoSnapshot<C>,after:SourceUndoSnapshot<C>,marked:MarkedSourceReplay?) {
  if(before.source==after.source)return
  val next=Step(before,after,marked);if(cost(next)>budget)return
  if(past.lastOrNull()?.after?.source?.let{it!=before.source}==true)clear()
  future.clear();past.add(next);serial++;trim()
 }
 fun replaceLastAfter(beforeSource:String,after:SourceUndoSnapshot<C>,since:Long):Boolean {
  val last=past.lastOrNull() ?: return false
  if(serial<=since||last.before.source!=beforeSource||last.after.source!=after.source||last.after.anchor!=after.anchor||last.after.head!=after.head)return false
  val next=last.copy(after=after);if(cost(next)>budget)return false
  past[past.lastIndex]=next;trim();return true
 }
 internal fun currentMarked(source:String,backwards:Boolean):MarkedSourceReplay? {
  val step=if(backwards)past.lastOrNull() else future.firstOrNull()
  val expected=if(backwards)step?.after else step?.before
  return step?.marked?.takeIf{expected?.source==source}
 }
 internal fun acceptMarkedReplay(
  mark:MarkedSourceReplay,before:String,after:String,backwards:Boolean
 ):SourceUndoSnapshot<C>? {
  val entry=if(backwards)past.lastOrNull() else future.firstOrNull()
  val expected=if(backwards)entry?.after else entry?.before
  val target=if(backwards)entry?.before else entry?.after
  if(entry==null||entry.marked!==mark||expected?.source!=before||target?.source!=after)return null
  if(backwards){past.removeAt(past.lastIndex);future.add(0,entry)}
  else{future.removeAt(0);past.add(entry)}
  return target
 }
 fun undo(beforeSource:String,afterSource:String,anchor:Int,head:Int):SourceUndoSnapshot<C>? {
  if(past.lastOrNull()?.after?.source!=beforeSource){clear();return null}
  val matches=past.indices.filter{val value=past[it].before;value.source==afterSource&&value.anchor==anchor&&value.head==head}
  if(matches.size!=1){clear();return null}
  val at=matches[0];val target=past[at].before;val moved=past.subList(at,past.size).toList();past.subList(at,past.size).clear();future.addAll(0,moved);return target
 }
 fun redo(beforeSource:String,afterSource:String,anchor:Int,head:Int):SourceUndoSnapshot<C>? {
  if(future.firstOrNull()?.before?.source!=beforeSource){clear();return null}
  val matches=future.indices.filter{val value=future[it].after;value.source==afterSource&&value.anchor==anchor&&value.head==head}
  if(matches.size!=1){clear();return null}
  val at=matches[0];val target=future[at].after;past.addAll(future.subList(0,at+1).toList());future.subList(0,at+1).clear();return target
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SourceUndoContexts.kt"><ui:paste select="#source-undo-context-journal-android"/></ui:file>

## Check context restoration without owning native text history

Native coalescing may cross several captured input steps. Assert the matching boundary, and refuse stale or ambiguous state rather than choose one by coincidence. A tiny retention budget evicts only metadata; this model exposes no source write at all.

<ui:copy id="source-undo-context-journal-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
class SourceUndoContextsTest {
 private fun snapshot(source:String,caret:Int,value:String)=SourceUndoSnapshot(source,caret,caret,value)
 @Test fun nativeCoalescingRestoresOneCapturedBoundary(){
  val journal=SourceUndoContexts<String>({0});journal.record(snapshot("a",1,"first"),snapshot("ab",2,"second"));journal.record(snapshot("ab",2,"second"),snapshot("abc",3,"third"))
  assertEquals("first",journal.undo("abc","a",1,1)?.context);assertEquals("third",journal.redo("a","abc",3,3)?.context)
 }
 @Test fun staleAndAmbiguousContextNeverChoosesAnArbitraryState(){
  val stale=SourceUndoContexts<String>({0});stale.record(snapshot("a",1,"first"),snapshot("b",1,"second"));assertNull(stale.undo("foreign","a",1,1));assertFalse(stale.retained)
  val cycle=SourceUndoContexts<String>({0});cycle.record(snapshot("a",1,"one"),snapshot("b",1,"two"));cycle.record(snapshot("b",1,"two"),snapshot("a",1,"three"));cycle.record(snapshot("a",1,"three"),snapshot("b",1,"four"));assertNull(cycle.undo("b","a",1,1));assertFalse(cycle.retained)
 }
 @Test fun evictingMetadataDoesNotProposeAnySourceWrite(){
  val journal=SourceUndoContexts<String>({0},budget=8);journal.record(snapshot("aa",2,"one"),snapshot("bbb",3,"two"));assertFalse(journal.retained);assertNull(journal.undo("bbb","aa",2,2))
 }
 @Test fun oversizedContextDoesNotEvictUnrelatedHistoryOrOverflow(){
  val journal=SourceUndoContexts<String>({if(it=="huge")Long.MAX_VALUE else 0L},budget=400)
  journal.record(snapshot("a",1,"first"),snapshot("ab",2,"second"));val serial=journal.serial
  journal.record(snapshot("ab",2,"second"),snapshot("abc",3,"huge"));assertEquals(serial,journal.serial)
  assertFalse(journal.replaceLastAfter("a",snapshot("ab",2,"huge"),0));assertEquals("first",journal.undo("ab","a",1,1)?.context)
 }
 @Test fun emptyChoiceSlotsConsumeNativeHistoryBudget(){
  val field=SnippetField(1,0,0,List(10){""},0,null);val session=SnippetSession("",listOf(field),1,0)
  assertEquals(384L,snippetHistoryCost(session,400));assertEquals(401L,snippetHistoryCost(session.copy(fields=listOf(field.copy(choices=List(11){""}))),400))
  val journal=SourceUndoContexts<SnippetSession?>({snippetHistoryCost(it,400)},budget=400)
  journal.record(SourceUndoSnapshot("a",1,1,null),SourceUndoSnapshot("ab",2,2,null))
  journal.record(SourceUndoSnapshot("ab",2,2,session),SourceUndoSnapshot("abc",3,3,session))
  assertNotNull(journal.undo("ab","a",1,1))
 }

}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SourceUndoContextsTest.kt"><ui:paste select="#source-undo-context-journal-tests"/></ui:file>

</ui:doc>
