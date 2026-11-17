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
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class SourceSnippetController(private val buffer:DocumentBuffer) {
 // Core input state advances synchronously; publish outside a TextFieldBuffer transformation.
 private var value:SnippetSession?=null
 var intent=0;private set
 private var revision by mutableIntStateOf(0)
 var notice by mutableStateOf("");private set
 private val contexts=SourceUndoContexts<SnippetSession?>({context->snippetHistoryCost(context)})
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
  val before=buffer.text;val oldSelection=buffer.state.selection;buffer.state.undoState.undo();val selected=buffer.state.selection
  if(before!=buffer.text||selected!=oldSelection)buffer.advanceSourceRevision()
  val target=contexts.undo(before,buffer.text,selected.start,selected.end);restore(target?.context)
  if(target?.context==null){notice="Text edit undone · draft";buffer.language?.completionNotice=notice};return true
 }
 fun redoSource():Boolean {
  if(buffer.readOnly||buffer.state.composition!=null||!canRedo)return false
  val before=buffer.text;val oldSelection=buffer.state.selection;buffer.state.undoState.redo();val selected=buffer.state.selection
  if(before!=buffer.text||selected!=oldSelection)buffer.advanceSourceRevision()
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
