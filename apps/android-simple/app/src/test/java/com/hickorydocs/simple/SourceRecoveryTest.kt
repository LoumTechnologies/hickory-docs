package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class SourceRecoveryTest {
 val id="a".repeat(32);val workspace="b".repeat(32)
 fun page()=RecoveryPage(1,workspace,listOf(id),null,0)
 fun reading(hash:String="c".repeat(64))=RecoveryReading(1,workspace,id,hash,"Move",false,listOf(RecoveryPath("a #😀.md",emptyMap())),false,0,"😀","")
 @Test fun exactSnapshotRequestsAndChangedEvidence()=runBlocking {
  val model=RecoveryReader();model.list{page()};var route="";model.read(id,"a #😀.md","saved_before"){route=it;reading()};assertTrue(route.contains("path=a+%23%F0%9F%98%80.md&saved=before"));assertEquals("",model.value?.savedText);assertEquals("😀",model.value?.draftText)
  model.read(id,"a #😀.md"){reading("d".repeat(64))};assertNull(model.value);assertTrue(model.notice.contains("evidence changed"));model.list{page()};model.list(id){page().copy(workspace="other",ids=emptyList())};assertNull(model.page)
 }
 @Test fun closedAndSupersededReadsCannotReplaceSelection()=runBlocking{
  val model=RecoveryReader();model.list{page()};val pending=CompletableDeferred<RecoveryReading>();val old=async(start=CoroutineStart.UNDISPATCHED){model.read(id){pending.await()}};model.list{page()};pending.complete(reading());old.await();assertNull(model.value)
  val late=CompletableDeferred<RecoveryReading>();val closed=async(start=CoroutineStart.UNDISPATCHED){model.read(id){late.await()}};model.close();late.complete(reading());closed.await();assertNull(model.page);assertNull(model.value)
 }
}
