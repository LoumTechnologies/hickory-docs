package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class ReferenceSourcesTest {
 @Test fun retainedDraftSynchronizesOnceBeforeFreshProposal()=runBlocking {
  var round=0;val reads=mutableListOf<String>();val original=SourceSnapshot("original draft",true)
  val result=acquireSourceProposal(mapOf("retained" to original),{SourceProposal(listOf("retained","retained"),++round)},{path->reads.add(path);original.copy()},{true})
  assertEquals(2,result.value);assertEquals(listOf("retained"),reads);assertSame(original,result.sources["retained"])
 }
 data class Read(val path:String,val done:CompletableDeferred<SourceSnapshot>)
 private suspend fun settle(condition:()->Boolean){withTimeout(2000){while(!condition())yield()}}
 @Test fun sequentialAcquisitionDeduplicatesAndRequeries()=runBlocking {
  var queries=0;val reads=mutableListOf<Read>()
  val task=async{acquireReferenceSources(mapOf("retained" to SourceSnapshot("original draft",true)),{queries++;listOf("a","b","a","retained")},{path->val done=CompletableDeferred<SourceSnapshot>();reads.add(Read(path,done));done.await()},{true})}
  settle{reads.size==1};assertEquals(listOf("a"),reads.map{it.path});reads[0].done.complete(SourceSnapshot("one",true));settle{reads.size==2};assertEquals(listOf("a","b"),reads.map{it.path});assertEquals(1,queries)
  reads[1].done.complete(SourceSnapshot("two",true));settle{reads.size==3};assertEquals("retained",reads[2].path);reads[2].done.complete(SourceSnapshot("original draft",true));val result=task.await();assertEquals(2,queries);assertEquals(listOf("a","b","retained"),result.references)
 }
 @Test fun textAndWritableDriftAndReadonlySourcesRefuse()=runBlocking {
  for(next in listOf(SourceSnapshot("changed",true),SourceSnapshot("original",false))){try{acquireReferenceSources(mapOf("a" to SourceSnapshot("original",true)),{listOf("a")},{next},{true});fail("drift accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("changed"))}}
  try{acquireReferenceSources(emptyMap(),{listOf("a")},{SourceSnapshot("source",false)},{true});fail("readonly accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("read-only"))}
 }
 @Test fun staleReadStopsSubsequentDispatchAndQuery()=runBlocking {
  supervisorScope {
   var current=true;var queries=0;val reads=mutableListOf<Read>();val task=async{acquireReferenceSources(emptyMap(),{queries++;listOf("a","b","c")},{path->val done=CompletableDeferred<SourceSnapshot>();reads.add(Read(path,done));done.await()},{current})}
   settle{reads.size==1};current=false;reads[0].done.complete(SourceSnapshot("source",true));try{task.await();fail("obsolete accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("changed"))};assertEquals(1,reads.size);assertEquals(1,queries)
  }
 }
 @Test fun expandingAndOversizedDiscoveryRefuse()=runBlocking {
  var round=0;try{acquireReferenceSources(emptyMap(),{listOf("new-${round++}")},{SourceSnapshot("source",true)},{true});fail("expansion accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("kept expanding"))};assertEquals(4,round)
  var reads=0;try{acquireReferenceSources(emptyMap(),{(0..10000).map{it.toString()}},{reads++;SourceSnapshot("",true)},{true});fail("oversized accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("10,000"))};assertEquals(0,reads)
 }
 @Test fun readonlyContextRequiresExplicitPolicyAndStillRefusesDrift()=runBlocking {
  val original=SourceSnapshot("context",false);val result=acquireSourceProposal(mapOf("context" to original),{SourceProposal(listOf("context"),1)},{original.copy()},{true},{false});assertSame(original,result.sources["context"])
  try{acquireSourceProposal(mapOf("context" to original),{SourceProposal(listOf("context"),1)},{SourceSnapshot("context",true)},{true},{false});fail("drift accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("changed"))}
 }
}
