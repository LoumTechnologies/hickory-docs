package com.hickorydocs.simple
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.text.TextRange
private fun computedSnippetInput(session:SnippetSession,from:Int,to:Int)=session.fields.any{field->field.transform!=null&&field.from<field.to&&field.from<=from&&to<=field.to&&(from<to||from>field.from&&from<field.to)}
private fun editableSnippetRange(session:SnippetSession,from:Int,to:Int):Boolean =
 session.active>0&&!computedSnippetInput(session,from,to)&&session.fields.any{
  it.transform==null&&it.index==session.active&&it.from<=from&&from<=to&&to<=it.to
 }

internal data class ParallelSnippetInput(val change:ParallelSnippetChange?)
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
 private data class RefusedMarkedRedo(val source:String,var earlierSteps:Int=0)
 private var refusedMarkedRedo:RefusedMarkedRedo?=null

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
   if (next?.refused == true) {
    if (restoreRefusedMarked(capture,source)) return
    notice="Native composition history changed during refusal. Current draft is retained."
    buffer.language?.completionNotice=notice
   } else if (next != null) {
    val delta = changedMarkedSource(source, next.source)
    if (reflectMarkedInput(delta, source, next.selection) && buffer.text == next.source) {
     val ranges = next.parallel?.let(::parallelRanges).orEmpty()
     follow = AndroidSnippetHistory(next.session, next.parallel, ranges)
     restoreHistory(follow)
    }
   }
  }
  if (buffer.text != capture.source) refusedMarkedRedo=null
  val finalSelection = buffer.state.selection
  val history = SourceUndoSnapshot<AndroidSnippetHistory?>(buffer.text, finalSelection.start, finalSelection.end, follow)
  val replay = capture.replay(buffer.text, mirrored = follow != null)
  if (replay != null) contexts.recordMarked(capture.history, history, replay)
 }

 private fun restoreRefusedMarked(capture:MarkedSnippetInput,observed:String):Boolean {
  val replay=capture.replay(observed,mirrored=false) ?: return false
  if(replay.run(true,{buffer.text},::moveNativeHistory)!=MarkedSourceReplay.Result.Finished||buffer.text!=capture.source)return false
  refusedMarkedRedo=RefusedMarkedRedo(capture.source)
  buffer.state.edit {selection=TextRange(capture.history.anchor,capture.history.head)}
  buffer.advanceSourceRevision()
  buffer.clearFolds()
  restoreHistory(capture.history.context)
  reportInputRefusal()
  return true
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
 val canRedo get()=buffer.state.undoState.canRedo&&(refusedMarkedRedo?.let{it.earlierSteps>0||it.source!=buffer.text}!=false)
 internal fun captureBefore(source:CharSequence,selection:TextRange,force:Boolean=false):SourceUndoSnapshot<AndroidSnippetHistory?>? {
  val text=value?.source ?: group?.source ?: source.toString()
  val context=historyContext(text)
  if(context==null&&!contexts.retained&&refusedMarkedRedo==null&&!force)return null
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
  if(before.source!=text)refusedMarkedRedo=null
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


 internal fun applyOrdinaryCompletionGroup(draft:AndroidCompletionGroup,source:String):Boolean {
  if(isActive||inMarkedInput||buffer.state.composition!=null||buffer.resourceDeleted)return false
  val before=captureBefore(source,buffer.state.selection,true) ?: return false
  val serial=recordSerial
  val ranges=draft.carets.map { NativeSourceEdit(it,it,"") }
  val edits=draft.plan.patches.singleOrNull()?.edits.orEmpty().map { NativeSourceEdit(it.from,it.to,it.text) }
  val selected=TextRange(ranges.first().from)
  if(!buffer.applySourceEdits(edits,source,selected))return false
  buffer.multiSource=buffer.text
  buffer.multiRanges=ranges
  if(buffer.text!=source){
   val after=SourceUndoSnapshot<AndroidSnippetHistory?>(buffer.text,selected.start,selected.end,AndroidSnippetHistory(null,null,ranges))
   if(!contexts.replaceLastAfter(source,after,serial))contexts.record(before,after)
  }
  return true
 }

 private fun moveNativeHistory(backwards:Boolean):Boolean {
  val native=buffer.state.undoState
  if(backwards&&native.canUndo){native.undo();return true}
  if(!backwards&&native.canRedo){native.redo();return true}
  return false
 }
 private fun moveBeforeRefusedRedo(backwards:Boolean) {
  refusedMarkedRedo?.let{it.earlierSteps=(it.earlierSteps+if(backwards)1 else -1).coerceAtLeast(0)}
 }
 private fun replayMarked(backwards: Boolean): Boolean? {
  val before = buffer.text
  val replay = contexts.currentMarked(before, backwards) ?: return null
  val outcome = replay.run(backwards, { buffer.text }, ::moveNativeHistory)
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
  replayMarked(true)?.let { if(it)moveBeforeRefusedRedo(true);return it }
  val before=buffer.text;val oldSelection=buffer.state.selection;buffer.state.undoState.undo();val selected=buffer.state.selection
  moveBeforeRefusedRedo(true)
  if(before!=buffer.text||selected!=oldSelection)buffer.advanceSourceRevision()
  val target=contexts.undo(before,buffer.text,selected.start,selected.end);restoreHistory(target?.context)
  if(target?.context==null){notice="Text edit undone · draft";buffer.language?.completionNotice=notice};return true
 }
 fun redoSource():Boolean {
  if(buffer.readOnly||buffer.state.composition!=null||inMarkedInput||!canRedo)return false
  replayMarked(false)?.let { if(it)moveBeforeRefusedRedo(false);return it }
  val before=buffer.text;val oldSelection=buffer.state.selection;buffer.state.undoState.redo();val selected=buffer.state.selection
  moveBeforeRefusedRedo(false)
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
  val next = current.editing(buffer.text, current.selections.map { NativeSourceEdit(it.from, it.to, text) })
  if(next==null||!commitParallel(next,buffer.text))reportInputRefusal()
  return true
 }

 internal fun planParallelInput(source: String, from: Int, to: Int, text: String): ParallelSnippetInput? {
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
  if(!current.peers.zip(edited).all{(peer,range)->
   val session=peer.session
   if(session!=null&&session.active>0)editableSnippetRange(session,range.from-peer.from,range.to-peer.from)
   else peer.from<=range.from&&range.from<=range.to&&range.to<=peer.to
  })return null
  return ParallelSnippetInput(current.editing(source, edited.map { NativeSourceEdit(it.from, it.to, text) }))
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
 internal fun includesInput(source:String,from:Int,to:Int):Boolean {
  val current=active ?: return false
  return !buffer.readOnly&&!buffer.saving&&source==buffer.text&&buffer.multiRanges.size<2&&editableSnippetRange(current,from,to)
 }
 internal fun reportInputRefusal(){
  notice="Linked template edit could not be prepared. Current draft is retained."
  buffer.language?.completionNotice=notice
 }
 fun proposal(source:String,from:Int,to:Int,text:String,caret:Int?=null):SnippetInput? {
  val current=active ?: return null
  if(!includesInput(source,from,to))return null
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
fun androidx.compose.foundation.text.input.TextFieldBuffer.applySourceSnippetInput(buffer:DocumentBuffer,prepare:(String,Int,Int,String)->SnippetPrimaryEdit={_,from,to,text->SnippetPrimaryEdit(NativeSourceEdit(from,to,text))},canReflect:(NativeSourceEdit)->Boolean={true},refused:()->Unit={buffer.snippets.reportInputRefusal()},publish:(SnippetSession?)->Unit):Boolean {
 if(buffer.snippets.active==null)return false
 if(changes.changeCount!=1||buffer.state.composition!=null){buffer.snippets.stage(null);publish(null);return false}
 val old=changes.getOriginalRange(0);val changed=changes.getRange(0)
 val source=originalText.toString();val primary=prepare(source,old.min,old.max,asCharSequence().subSequence(changed.min,changed.max).toString())
 val plan=buffer.snippets.proposal(source,primary.edit.from,primary.edit.to,primary.edit.text,primary.caret)
 if(plan==null&&buffer.snippets.includesInput(source,primary.edit.from,primary.edit.to)){
  revertAllChanges();selection=originalSelection;refused();return true
 }
 if(plan==null||plan.edits.any{!canReflect(it)}){buffer.snippets.stage(null);publish(null);return false}
 revertAllChanges();plan.edits.asReversed().forEach{replace(it.from,it.to,it.text)};selection=TextRange(plan.selection.from,plan.selection.to)
 buffer.snippets.stage(plan.session);publish(plan.session);return true
}
