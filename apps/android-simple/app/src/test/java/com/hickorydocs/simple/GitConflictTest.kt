package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class GitConflictTest {
 private fun reading(path:String,text:String="captured")=ConflictReading(1,path,"a".repeat(64),listOf(ConflictStage(2,"100644","b".repeat(40),text,null)),false,false)
 @Test fun olderClosedAndForeignRepliesAreRetired()=runBlocking{
  val model=GitConflictModel();val held=mutableListOf<CompletableDeferred<ConflictReading>>();val load:suspend (String)->ConflictReading={CompletableDeferred<ConflictReading>().also{held.add(it)}.await()}
  suspend fun settle(count:Int){withTimeout(2000){while(held.size<count)yield()}}
  val first=async{model.read("a.txt",load)};settle(1);val second=async{model.read("b.txt",load)};settle(2)
  held[1].complete(reading("b.txt","new"));second.await();held[0].complete(reading("a.txt","old"));first.await();assertEquals("b.txt",model.value?.path)
  val closed=async{model.read("a.txt",load)};settle(3);model.close();held[2].complete(reading("a.txt"));closed.await();assertNull(model.value)
  model.read("é.txt"){reading("e\u0301.txt")};assertNull(model.value);assertTrue(model.notice.contains("identity changed"))
 }
 @Test fun immutableStageAndAvailabilityValidation(){
  val value=reading("a.txt");validateConflict(value,"a.txt")
  for(bad in listOf(value.copy(stages=value.stages+value.stages),value.copy(stages=listOf(value.stages[0].copy(oid="main"))),value.copy(stages=listOf(value.stages[0].copy(mode="120000"))),value.copy(stages=listOf(value.stages[0].copy(text=null))))){assertTrue(runCatching{validateConflict(bad,"a.txt")}.isFailure)}
  validateConflict(value.copy(stages=listOf(value.stages[0].copy(text=null,unavailable="Binary"))),"a.txt")
 }
}
