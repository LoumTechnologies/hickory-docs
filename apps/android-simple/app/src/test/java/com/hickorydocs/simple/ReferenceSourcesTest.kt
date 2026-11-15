package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class ReferenceSourcesTest {
 @Test fun returnsOnlyProposalQueriedAfterAcquisition()=runBlocking {
  var round=0;val result=acquireSourceProposal(emptyMap(),{SourceProposal(listOf("other"),++round)},{SourceSnapshot("loaded",true)},{true})
  assertEquals(2,result.value);assertEquals("loaded",result.sources["other"]?.text)
 }
 data class Read(val path:String,val done:CompletableDeferred<SourceSnapshot>)
 private suspend fun settle(condition:()->Boolean){withTimeout(2000){while(!condition())yield()}}
 @Test fun boundedAcquisitionRequeriesWithoutReplacingRetainedDrafts()=runBlocking {
  var queries=0;val reads=mutableListOf<Read>()
  val task=async{acquireReferenceSources(mapOf("retained" to SourceSnapshot("original draft",true)),{queries++;listOf("a","b","a","c","retained")},{path->val done=CompletableDeferred<SourceSnapshot>();reads.add(Read(path,done));done.await()},{true})}
  settle{reads.size==2};assertEquals(listOf("a","b"),reads.map{it.path});assertEquals(1,queries);reads[0].done.complete(SourceSnapshot("one",true));settle{reads.size==3};assertEquals("c",reads[2].path);assertEquals(1,queries)
  reads[1].done.complete(SourceSnapshot("two",true));reads[2].done.complete(SourceSnapshot("three",true));val value=task.await();assertEquals(2,queries);assertEquals(listOf("a","b","c","retained"),value.references);assertEquals("original draft",value.sources["retained"]?.text);assertEquals(4,value.sources.size)
 }
 @Test fun staleAcquisitionDrainsAndExpandingTargetsRefuse()=runBlocking {
  supervisorScope {
   var current=true;val reads=mutableListOf<Read>();val task=async{acquireReferenceSources(emptyMap(),{listOf("a","b","c")},{path->val done=CompletableDeferred<SourceSnapshot>();reads.add(Read(path,done));done.await()},{current})}
   settle{reads.size==2};current=false;reads.forEach{it.done.complete(SourceSnapshot("source",true))};try{task.await();fail("obsolete accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("changed"))};assertEquals(2,reads.size)
  }
  var round=0;try{acquireReferenceSources(emptyMap(),{listOf("new-${round++}")},{SourceSnapshot("source",true)},{true});fail("expansion accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("kept expanding"))};assertEquals(4,round)
 }
}
