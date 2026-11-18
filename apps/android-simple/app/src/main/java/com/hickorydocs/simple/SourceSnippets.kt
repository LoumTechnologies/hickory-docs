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
 fun undoSource():Boolean {
  if(buffer.readOnly||buffer.state.composition!=null||!canUndo)return false
  val before=buffer.text;val oldSelection=buffer.state.selection;buffer.state.undoState.undo();val selected=buffer.state.selection
  if(before!=buffer.text||selected!=oldSelection)buffer.advanceSourceRevision()
  val target=contexts.undo(before,buffer.text,selected.start,selected.end);restoreHistory(target?.context)
  if(target?.context==null){notice="Text edit undone · draft";buffer.language?.completionNotice=notice};return true
 }
 fun redoSource():Boolean {
  if(buffer.readOnly||buffer.state.composition!=null||!canRedo)return false
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
  val next = ParallelSnippet.begin(source, expansions, primary) ?: return false
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
