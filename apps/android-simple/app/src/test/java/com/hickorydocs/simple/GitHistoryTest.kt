package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class GitHistoryTest {
 @Test fun heldAndClosedReadingsRetainTheSelectedCommitIdentity()=runBlocking {
  val model=GitHistoryModel();val a="a".repeat(40);val b="b".repeat(40)
  model.refresh(){_,_->HistoryLog(true,listOf(a,b).map{HistoryCommit(it,it.take(7),"fixture","saved commit")})}
  val pending=CompletableDeferred<HistoryReading>();val first=async(start=CoroutineStart.UNDISPATCHED){model.select(a){pending.await()}}
  model.select(b){HistoryReading(it,"new",emptyList())};pending.complete(HistoryReading(a,"obsolete",emptyList()));first.await();assertEquals(b,model.reading?.sha);assertEquals("new",model.reading?.message)
  val late=CompletableDeferred<HistoryReading>();val closed=async(start=CoroutineStart.UNDISPATCHED){model.select(a){late.await()}};model.close();late.complete(HistoryReading(a,"closed",emptyList()));closed.await();assertNull(model.reading)
  model.select(a){HistoryReading(b,"foreign",emptyList())};assertNull(model.reading);assertTrue(model.notice.contains("identity changed"))
 }
 @Test fun malformedLogIdentityNeverBecomesAReadingChoice()=runBlocking {
  val model=GitHistoryModel();model.refresh(2000){_,_->HistoryLog(true,listOf(HistoryCommit("HEAD","HEAD","fixture","ambiguous")))};assertEquals(1000,model.limit);assertTrue(model.commits.isEmpty());assertTrue(model.notice.contains("Malformed commit identity"))
 }
 @Test fun observedFullCommitRequestDoesNotManufactureALogChoice()=runBlocking {
  val sha="c".repeat(40);val model=GitHistoryModel(GitHistoryRequest.Commit(sha,"Saved attribution"));val calls=mutableListOf<String>()
  suspend fun load(value:String):HistoryReading{calls.add(value);return HistoryReading(value,"observed",emptyList())}
  model.readCommit(sha,::load);assertEquals(listOf(sha),calls);assertEquals(sha,model.reading?.sha);model.readCommit("12345678",::load);assertEquals(listOf(sha),calls);assertNull(model.reading);model.readCommit("d".repeat(40),::load);assertEquals(listOf(sha),calls);assertNull(model.reading)
 }

}
