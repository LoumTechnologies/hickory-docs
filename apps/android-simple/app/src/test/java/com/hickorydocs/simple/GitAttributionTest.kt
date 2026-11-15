package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class GitAttributionTest {
 private fun row(line:Int,full:String?=null)=AttributionLine(line,"12345678",full,"Fixture","base",false,null,"saved")
 @Test fun capturedPathsAndLineIdentitiesRetireObsoleteReplies()=runBlocking {
  val model=GitAttributionModel("a.py");val pending=CompletableDeferred<AttributionReading>();val old=async(start=CoroutineStart.UNDISPATCHED){model.refresh{pending.await()}}
  model.refresh{AttributionReading(it,listOf(row(2,"a".repeat(40))))};pending.complete(AttributionReading("a.py",listOf(row(1))));old.await();assertEquals(listOf(2),model.lines.map{it.line})
  val late=CompletableDeferred<AttributionReading>();val closed=async(start=CoroutineStart.UNDISPATCHED){model.refresh{late.await()}};model.close();late.complete(AttributionReading("a.py",listOf(row(1))));closed.await();assertTrue(model.lines.isEmpty())
  val foreign=GitAttributionModel("cafe\u0301.py");foreign.refresh{AttributionReading("caf\u00e9.py",listOf(row(1)))};assertTrue(foreign.lines.isEmpty());assertTrue(foreign.notice.contains("identity changed"))
  model.refresh{AttributionReading(it,listOf(row(1),row(1)))};assertTrue(model.lines.isEmpty());assertTrue(model.notice.contains("Ambiguous"))
 }
 @Test fun compactLabelsAndDirtyLinesAreNotImmutableReferences(){assertNull(row(1).immutableCommit);assertNull(row(1,"12345678").immutableCommit);assertNull(row(1,"a".repeat(40)).copy(uncommitted=true).immutableCommit);assertEquals("a".repeat(40),row(1,"a".repeat(40)).immutableCommit);assertEquals("b".repeat(64),row(1,"b".repeat(64)).immutableCommit)}
}
