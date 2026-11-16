package com.hickorydocs.simple
// The native text state owns text undo. This bounded journal restores optional context only.
data class SourceUndoSnapshot<C>(val source:String,val anchor:Int,val head:Int,val context:C)
class SourceUndoContexts<C>(private val contextCost:(C)->Long,private val budget:Long=32L*1024*1024) {
 private data class Step<C>(val before:SourceUndoSnapshot<C>,val after:SourceUndoSnapshot<C>)
 private val past=mutableListOf<Step<C>>();private val future=mutableListOf<Step<C>>()
 var serial=0L;private set
 val retained:Boolean get()=past.isNotEmpty()||future.isNotEmpty()
 init{require(budget in 0 until Long.MAX_VALUE)}
 private fun sum(vararg values:Long):Long {var total=0L;for(value in values){if(value<0||value>budget-total)return budget+1;total+=value};return total}
 private fun cost(step:Step<C>)=sum(2L*step.before.source.length,2L*step.after.source.length,contextCost(step.before.context),contextCost(step.after.context))
 private fun trim(){var bytes=past.sumOf{cost(it)}+future.sumOf{cost(it)};while((bytes>budget||past.size+future.size>4096)&&past.isNotEmpty()){bytes-=cost(past.removeAt(0))};while((bytes>budget||past.size+future.size>4096)&&future.isNotEmpty()){bytes-=cost(future.removeAt(future.lastIndex))}}
 private fun clear(){past.clear();future.clear()}
 fun record(before:SourceUndoSnapshot<C>,after:SourceUndoSnapshot<C>) {
  if(before.source==after.source)return
  val next=Step(before,after);if(cost(next)>budget)return
  if(past.lastOrNull()?.after?.source?.let{it!=before.source}==true)clear()
  future.clear();past.add(next);serial++;trim()
 }
 fun replaceLastAfter(beforeSource:String,after:SourceUndoSnapshot<C>,since:Long):Boolean {
  val last=past.lastOrNull() ?: return false
  if(serial<=since||last.before.source!=beforeSource||last.after.source!=after.source||last.after.anchor!=after.anchor||last.after.head!=after.head)return false
  val next=last.copy(after=after);if(cost(next)>budget)return false
  past[past.lastIndex]=next;trim();return true
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
