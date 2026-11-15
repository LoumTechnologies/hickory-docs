package com.hickorydocs.simple
import kotlinx.coroutines.*
data class ReferenceSourceAcquisition(val sources:Map<String,SourceSnapshot>,val references:List<String>)
data class SourceProposal<Value>(val paths:List<String>,val value:Value)
data class SourceProposalAcquisition<Value>(val sources:Map<String,SourceSnapshot>,val paths:List<String>,val value:Value)
suspend fun <Value> acquireSourceProposal(initial:Map<String,SourceSnapshot>,query:suspend ()->SourceProposal<Value>,acquire:suspend (String)->SourceSnapshot,current:()->Boolean):SourceProposalAcquisition<Value> = coroutineScope {
 val sources=initial.toMutableMap()
 fun checkCurrent(){check(current()){"Source, selection or provider changed. Prepare again."}}
 repeat(4){
  checkCurrent();val proposal=query();val references=proposal.paths.distinct().sorted();checkCurrent();check(references.size<=10000){"Reference discovery exceeds 10,000 source targets. Narrow the change."}
  val missing=references.filter{it !in sources};if(missing.isEmpty())return@coroutineScope SourceProposalAcquisition(sources.toMap(),references,proposal.value)
  var cursor=0;var failure:Exception?=null
  suspend fun worker(){while(failure==null&&cursor<missing.size){try{checkCurrent();val path=missing[cursor++];val value=acquire(path);checkCurrent();check(value.writable){"Source is generated, read-only or saving: $path"};sources[path]=value}
   catch(cancelled:CancellationException){throw cancelled}catch(error:Exception){failure=error}
  }}
  val first=async{worker()};val second=async{worker()};awaitAll(first,second);failure?.let{throw it};checkCurrent()
 }
 error("Reference targets kept expanding. Refresh project analysis and prepare again. No edits were applied.")
}
suspend fun acquireReferenceSources(initial:Map<String,SourceSnapshot>,query:suspend ()->List<String>,acquire:suspend (String)->SourceSnapshot,current:()->Boolean):ReferenceSourceAcquisition {
 val result=acquireSourceProposal(initial,{SourceProposal(query(),Unit)},acquire,current)
 return ReferenceSourceAcquisition(result.sources,result.paths)
}
