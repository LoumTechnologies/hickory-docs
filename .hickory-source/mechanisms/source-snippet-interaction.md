<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Connect explicit snippet plans to existing editing surfaces

For maintainers changing template behavior independently of editor layout. Source and native input authority remain in the existing hosts; the optional controller adds no toolbar or permanent code-space allocation.

## Attach source-bound field interaction to the existing editor

A compartment owns snippet state, source checks, linked input, numeric traversal, choices and history effects. Complete mirroring is prepared before a transaction; rejected intent leaves ordinary input authoritative. Compose the original input transaction with the remaining source changes so its input metadata survives. History effects retain both directions of source, field state and intended selection. The dispatch bridge composes the original transaction with a verified replay selection, preserving the framework history annotation and input metadata; it never creates replacement source changes. This avoids inferring a caret from the old selection when a compound edit had an explicit result. History effects retain bounded context tokens: old text undo remains owned by CodeMirror when the estimated 32 MiB context budget evicts a field snapshot. Field marks use paint only, and choices reuse the existing completion popup.

<ui:copy id="source-snippets-browser">
import {EditorSelection,EditorState,Prec,StateEffect,StateField,type Transaction,type Extension} from '@codemirror/state';
import {Decoration,EditorView,keymap} from '@codemirror/view';
import {invertedEffects,isolateHistory} from '@codemirror/commands';
import {acceptCompletion,closeCompletion,completionStatus,pickedCompletion,startCompletion,type CompletionSource} from '@codemirror/autocomplete';
import {exclusiveCompletionSources,withCompletionOrigin} from '../lsp/completion';
import {cancelCompletionAcceptance,readCompletionAcceptance} from '../lsp/completionAcceptance';
import {prepareEditorChanges} from '../editor/textInput';
import {beginSnippet,advanceSnippet,editSnippet,completeSnippet,nestSnippet,type SnippetSession} from './SnippetSession';
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
 const edit=draft.primary;if(!edit)throw new Error('Completion primary edit is unavailable. No edits were applied.');
 const input=draft.fields.length?nestSnippet(current,draft.before,edit,draft.additional??[],draft.fields.map(field=>({...field,from:field.from-draft.insertionFrom,to:field.to-draft.insertionFrom}))):completeSnippet(current,draft.before,edit,draft.additional??[]);if(!input)throw new Error('Completion overlaps template fields or is unavailable. Finish the template and retry.');
 return {...draft,plan:{title:draft.plan.title,patches:input.session.source===draft.before?[]:[{path:draft.path,before:draft.before,after:input.session.source,edits:input.edits}]},selection:input.selection,insertionFrom:input.selection.from-edit.text.length,continuing:input.session};
}
export function readSourceSnippet(view:EditorView):SnippetSession|null{return view.state.field(snippetState,false)??null;}
export function finishSourceSnippet(view:EditorView):boolean {if(!readSourceSnippet(view))return false;view.dispatch({effects:setSourceSnippet.of(null)});return true;}
export function moveSourceSnippet(view:EditorView,direction:1|-1):boolean {
 const session=readSourceSnippet(view);if(!session||view.composing)return false;
 const selected=view.state.selection.main,result=advanceSnippet(session,view.state.doc.toString(),selected.from,selected.to,direction);
 if(!result){if(direction===-1&&session.active===(session.scopes?.at(-1)?.order[0]??Math.min(...session.fields.filter(field=>field.index>0).map(field=>field.index))))return true;finishSourceSnippet(view);return false;}
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
</ui:copy>

<ui:file path="apps/web/src/simple/SourceSnippets.ts"><ui:paste select="#source-snippets-browser"/></ui:file>

## Keep Apple snippet intent outside AppKit and UIKit

A source-bound controller exposes navigation and choices through existing selection and edit callbacks. The native editor still owns composition, permissions and actual batch/undo application. Stale intent and failed reflection retire the capability rather than blocking ordinary text input. A prepared change plan verifies its entire predicted source before entering history.

<ui:copy id="source-snippets-apple">
import Foundation
import Observation
struct SourceSnippetHistoryContext {var fields:[SnippetField];var active:Int;var current:Int;var scopes:[SnippetScope]=[]}
func snippetHistoryCost(_ value:SourceSnippetHistoryContext?)->Int {value?.fields.reduce(0){sum,field in sum+64+field.choices.reduce(0){$0+$1.utf16.count*2}} ?? 0}
@MainActor @Observable final class SourceSnippetController {
    private weak var buffer:DocumentBuffer?
    private(set) var intent=0
    private(set) var session:SnippetSession?
    private(set) var notice=""
    @ObservationIgnored var commit:((SnippetInput,SnippetSession)->Bool)?
    init(buffer:DocumentBuffer){self.buffer=buffer}
    var active:SnippetSession? {guard let buffer,let session,exactUTF8(buffer.text,session.source) else{return nil};return session}
    var choices:[String] {guard let active else{return []};return active.fields.first(where:{$0.id==active.current})?.choices ?? []}
    var historyContext:SourceSnippetHistoryContext? {active.map{SourceSnippetHistoryContext(fields:$0.fields,active:$0.active,current:$0.current,scopes:$0.scopes)}}
    func restoreHistory(_ context:SourceSnippetHistoryContext?){
        guard let context,let buffer,beginSnippet(buffer.text,fields:context.fields) != nil else{restore(nil);return}
        restore(SnippetSession(source:buffer.text,fields:context.fields,active:context.active,current:context.current,scopes:context.scopes))
    }
    func restore(_ value:SnippetSession?) {session=value;intent+=1;report()}
    func sourceChanged(_ source:String) {if let session,!exactUTF8(source,session.source){finish()}}
    func reportFailure(_ message:String){notice=message;buffer?.language?.completionNotice=message}
    private func report(){notice=session.map{"Template field \($0.active) · Tab next · Shift-Tab previous · Esc finish"} ?? "Template editing finished · draft";buffer?.language?.completionNotice=notice}
    @discardableResult func finish()->Bool {guard session != nil else{return false};session=nil;intent+=1;report();return true}
    @discardableResult func move(_ direction:Int)->Bool {
        guard let buffer,let current=active else{_ = finish();return false}
        guard let next=advanceSnippet(current,source:buffer.text,from:buffer.selection.location,to:NSMaxRange(buffer.selection),direction:direction) else{if direction == -1,current.active==(current.scopes.last?.order.first ?? current.fields.map({$0.index}).filter({$0>0}).min()){return true};_ = finish();return false}
        let old=session;session=next.session
        guard buffer.selectSourceRange?(NSRange(location:next.selection.from,length:next.selection.to-next.selection.from),current.source)==true else{session=old;return false}
        intent+=1;buffer.focusSource?();report();return true
    }
    // nil leaves ordinary native input in charge; true means one linked batch committed.
    func input(source:String,range:NSRange,text:String,caret:Int?=nil)->Bool? {
        guard let buffer,let current=active,!buffer.readOnly,!buffer.saving,exactUTF8(source,buffer.text),buffer.multiRanges.count<2,var input=editSnippet(current,source:source,from:range.location,to:NSMaxRange(range),text:text),let commit else{_ = finish();return nil}
        if let caret {
            guard caret>=range.location,caret<=range.location+text.utf16.count else{_ = finish();return nil}
            let selected=input.selection.from+caret-(range.location+text.utf16.count);input.selection=CompletionSelection(from:selected,to:selected)
        }
        guard commit(input,current) else{_ = finish();return nil};return true
    }
    func prepareCompletion(_ draft:CompletionDraft)throws->CompletionDraft {
        guard let current=active else{return draft}
        var result=draft
        guard let edit=draft.primary else{throw EngineError(message:"Completion primary edit is unavailable.")}
        let inner=draft.fields.map{field in var value=field;value.from-=draft.insertionFrom;value.to-=draft.insertionFrom;return value}
        guard let input=inner.isEmpty ? completeSnippet(current,source:draft.before,primary:edit,additional:draft.additional):nestSnippet(current,source:draft.before,primary:edit,additional:draft.additional,inner:inner) else{throw EngineError(message:"Completion overlaps template fields or is unavailable. Finish the template and retry.")}
        result.plan=ChangePlan(title:draft.plan.title,patches:exactUTF8(input.session.source,draft.before) ? []:[SourcePatch(path:draft.path,before:draft.before,after:input.session.source,edits:input.edits.map{SourceReplacement(from:$0.range.location,to:NSMaxRange($0.range),text:$0.text)})]);result.selection=input.selection;result.insertionFrom=input.selection.from-edit.text.utf16.count;result.continuing=input.session;return result
    }
    @discardableResult func choose(_ value:String)->Bool {
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
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class SourceSnippetController(private val buffer:DocumentBuffer) {
 // Core input state advances synchronously; publish outside a TextFieldBuffer transformation.
 private var value:SnippetSession?=null
 var intent=0;private set
 private var revision by mutableIntStateOf(0)
 var notice by mutableStateOf("");private set
 private val contexts=SourceUndoContexts<SnippetSession?>({context->context?.fields?.sumOf{field->64L+field.choices.sumOf{it.length.toLong()*2}} ?: 0L})
 val recordSerial get()=contexts.serial
 val canUndo get()=buffer.state.undoState.canUndo
 val canRedo get()=buffer.state.undoState.canRedo
 fun captureBefore(source:CharSequence,selection:TextRange,force:Boolean=false):SourceUndoSnapshot<SnippetSession?>? {
  val current=active;if(current==null&&!contexts.retained&&!force)return null
  return SourceUndoSnapshot(current?.source ?: source.toString(),selection.start,selection.end,current)
 }
 fun finishTextChange(before:SourceUndoSnapshot<SnippetSession?>?,source:CharSequence,selected:TextRange){
  if(before==null)return
  val current=value?.takeIf{source.toString()==it.source};val after=SourceUndoSnapshot(current?.source ?: source.toString(),selected.start,selected.end,current)
  contexts.record(before,after);stage(current)
 }
 fun attachCompletionContext(before:SourceUndoSnapshot<SnippetSession?>,after:SnippetSession?,since:Long){
  val selected=buffer.state.selection;val snapshot=SourceUndoSnapshot(after?.source ?: buffer.text,selected.start,selected.end,after)
  if(!contexts.replaceLastAfter(before.source,snapshot,since))contexts.record(before,snapshot)
  restore(after)
 }
 fun undoSource():Boolean {
  if(buffer.readOnly||buffer.state.composition!=null||!canUndo)return false
  val before=buffer.text;buffer.state.undoState.undo();val selected=buffer.state.selection
  val target=contexts.undo(before,buffer.text,selected.start,selected.end);restore(target?.context)
  if(target?.context==null){notice="Text edit undone · draft";buffer.language?.completionNotice=notice};return true
 }
 fun redoSource():Boolean {
  if(buffer.readOnly||buffer.state.composition!=null||!canRedo)return false
  val before=buffer.text;buffer.state.undoState.redo();val selected=buffer.state.selection
  val target=contexts.redo(before,buffer.text,selected.start,selected.end);restore(target?.context)
  if(target?.context==null){notice="Text edit redone · draft";buffer.language?.completionNotice=notice};return true
 }
 val session:SnippetSession? get(){revision;return value}
 val active:SnippetSession? get()=session?.takeIf{it.source==buffer.text}
 val choices:List<String> get()=active?.let{current->current.fields.firstOrNull{it.id==current.current}?.choices}.orEmpty()
 fun stage(session:SnippetSession?){if(value!==session){value=session;intent++}}
 fun publish(expected:SnippetSession?=value){if(value!==expected)return;revision++;notice=value?.let{"Template field ${it.active} · Tab next · Shift-Tab previous · Esc finish"} ?: "Template editing finished · draft";buffer.language?.completionNotice=notice}
 fun restore(session:SnippetSession?){stage(session);publish(session)}
 fun finish():Boolean {if(value==null)return false;restore(null);return true}
 fun sourceChanged(source:String){if(value?.source?.let{it!=source}==true)finish()}
 fun move(direction:Int):Boolean {
  val current=active ?: return false;if(buffer.state.composition!=null)return false;val selection=buffer.state.selection
  val next=advanceSnippet(current,buffer.text,selection.min,selection.max,direction) ?: return if(direction == -1&&current.active==(current.scopes.lastOrNull()?.order?.firstOrNull() ?: current.fields.map{it.index}.filter{it>0}.minOrNull()))true else finish().let{false}
  val previous=value;stage(next.session)
  if(!buffer.selectSourceRange(next.selection.from,next.selection.to,current.source)){stage(previous);return false};publish(next.session);return true
 }
 fun proposal(source:String,from:Int,to:Int,text:String,caret:Int?=null):SnippetInput? {
  val current=active ?: return null;if(buffer.readOnly||buffer.saving||source!=buffer.text||buffer.multiRanges.size>1)return null
  val input=editSnippet(current,source,from,to,text) ?: return null
  if(caret==null)return input;if(caret !in from..(from+text.length))return null;val selected=input.selection.from+caret-(from+text.length);return input.copy(selection=CompletionSelection(selected,selected))
 }
 fun prepareCompletion(draft:CompletionDraft):CompletionDraft {
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
import {EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {history,undo,redo} from '@codemirror/commands';
import {acceptCompletion,completionStatus,setSelectedCompletion} from '@codemirror/autocomplete';
import {waitFor} from '@testing-library/react';
import {completions} from '../lsp/completion';
import {beginSnippet} from './SnippetSession';
import {sourceSnippets,setSourceSnippet,readSourceSnippet,moveSourceSnippet,openSourceSnippetChoices,dispatchSourceSnippetTransactions} from './SourceSnippets';
import type {SnippetField} from './SnippetExpansion';
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
    func fields()->[SnippetField]{[SnippetField(index:1,from:0,to:1,choices:["a","long"],id:0),SnippetField(index:1,from:4,to:5,choices:["a","long"],id:1),SnippetField(index:0,from:5,to:5,choices:[],id:2)]}
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
 private data class Step<C>(val before:SourceUndoSnapshot<C>,val after:SourceUndoSnapshot<C>)
 private val past=mutableListOf<Step<C>>();private val future=mutableListOf<Step<C>>()
 var serial=0L;private set
 val retained:Boolean get()=past.isNotEmpty()||future.isNotEmpty()
 private fun cost(step:Step<C>)=2L*step.before.source.length+2L*step.after.source.length+contextCost(step.before.context)+contextCost(step.after.context)
 private fun trim(){var bytes=past.sumOf{cost(it)}+future.sumOf{cost(it)};while((bytes>budget||past.size+future.size>4096)&&past.isNotEmpty()){bytes-=cost(past.removeAt(0))};while((bytes>budget||past.size+future.size>4096)&&future.isNotEmpty()){bytes-=cost(future.removeAt(future.lastIndex))}}
 private fun clear(){past.clear();future.clear()}
 fun record(before:SourceUndoSnapshot<C>,after:SourceUndoSnapshot<C>) {
  if(before.source==after.source)return
  if(past.lastOrNull()?.after?.source?.let{it!=before.source}==true)clear()
  future.clear();past.add(Step(before,after));serial++;trim()
 }
 fun replaceLastAfter(beforeSource:String,after:SourceUndoSnapshot<C>,since:Long):Boolean {
  val last=past.lastOrNull() ?: return false
  if(serial<=since||last.before.source!=beforeSource||last.after.source!=after.source||last.after.anchor!=after.anchor||last.after.head!=after.head)return false
  past[past.lastIndex]=last.copy(after=after);trim();return true
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
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SourceUndoContextsTest.kt"><ui:paste select="#source-undo-context-journal-tests"/></ui:file>

</ui:doc>
