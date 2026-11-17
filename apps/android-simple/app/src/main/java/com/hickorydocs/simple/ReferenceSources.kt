package com.hickorydocs.simple
import kotlinx.coroutines.*
data class ReferenceSourceAcquisition(val sources:Map<String,SourceSnapshot>,val references:List<String>)
data class SourceProposal<Value>(val paths:List<String>,val value:Value)
data class SourceProposalAcquisition<Value>(val sources:Map<String,SourceSnapshot>,val paths:List<String>,val value:Value)
suspend fun <Value> acquireSourceProposal(initial:Map<String,SourceSnapshot>,query:suspend ()->SourceProposal<Value>,acquire:suspend (String)->SourceSnapshot,current:()->Boolean,requireWritable:(String)->Boolean={true}):SourceProposalAcquisition<Value> {
 val sources=initial.toMutableMap();val synchronized=mutableSetOf<String>()
 fun checkCurrent(){check(current()){"Source, selection or provider changed. Prepare again."}}
 repeat(4){
  checkCurrent();val proposal=query();val references=proposal.paths.distinct().sorted();checkCurrent();check(references.size<=10000){"Reference discovery exceeds 10,000 source targets. Narrow the change."}
  val pending=references.filter{it !in synchronized};if(pending.isEmpty())return SourceProposalAcquisition(sources.toMap(),references,proposal.value)
  for(path in pending){
   checkCurrent();val value=acquire(path);checkCurrent();val original=sources[path]
   check(original==null||(original.text==value.text&&original.writable==value.writable)){"Source changed while synchronizing: $path. Prepare again."}
   check(!requireWritable(path)||value.writable){"Source is generated, read-only or saving: $path"}
   if(original==null)sources[path]=value;synchronized.add(path)
  }
  checkCurrent()
 }
 error("Reference targets kept expanding. Refresh project analysis and prepare again. No edits were applied.")
}
suspend fun acquireReferenceSources(initial:Map<String,SourceSnapshot>,query:suspend ()->List<String>,acquire:suspend (String)->SourceSnapshot,current:()->Boolean,requireWritable:(String)->Boolean={true}):ReferenceSourceAcquisition {
 val result=acquireSourceProposal(initial,{SourceProposal(query(),Unit)},acquire,current,requireWritable)
 return ReferenceSourceAcquisition(result.sources,result.paths)
}
