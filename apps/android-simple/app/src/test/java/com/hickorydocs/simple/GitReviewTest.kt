package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class GitReviewTest {
 @Test fun latestSideAndClosedRepliesRetainTheirOwnIdentity()=runBlocking {
  val model=GitReviewModel();model.refresh{ReviewChanges(true,"main",listOf("a.py","b.py").map{ReviewFile(it,null,"M","M")})}
  val held=CompletableDeferred<ReviewDiff>();val first=async(start=CoroutineStart.UNDISPATCHED){model.select("a.py",false){_,_->held.await()}}
  model.select("b.py",true){path,_->ReviewDiff(path,"new staged",false)};held.complete(ReviewDiff("a.py","obsolete",false));first.await()
  assertEquals("b.py",model.selected);assertTrue(model.staged);assertEquals("new staged",model.detail)
  val late=CompletableDeferred<ReviewDiff>();val closed=async(start=CoroutineStart.UNDISPATCHED){model.select("a.py",false){_,_->late.await()}};model.close();late.complete(ReviewDiff("a.py","late after close",false));closed.await();assertEquals("",model.detail)
  model.select("a.py",false){_,_->ReviewDiff("b.py","wrong source",false)};assertEquals("",model.detail);assertTrue(model.notice.contains("identity changed"))
 }
 @Test fun renameIndexActionsDoNotReplayAfterUnconfirmedResults()=runBlocking {
  val model=GitReviewModel();val file=ReviewFile("new.py","old.py","R","M");val calls=mutableListOf<List<String>>();var fail=false
  suspend fun load()=ReviewChanges(true,null,listOf(file))
  suspend fun write(paths:List<String>,unstage:Boolean):Boolean{calls.add(paths);if(fail)error("connection ended");return true}
  model.refresh(::load);model.changeIndex("new.py",true,::write,::load);assertEquals(listOf(listOf("old.py","new.py")),calls);assertFalse(model.needsRefresh)
  fail=true;model.changeIndex("new.py",false,::write,::load);assertEquals(2,calls.size);assertTrue(model.needsRefresh);model.changeIndex("new.py",false,::write,::load);assertEquals(2,calls.size)
  model.refresh(::load);fail=false;model.changeIndex("new.py",false,::write,::load);assertEquals(3,calls.size);assertFalse(model.needsRefresh)
  assertTrue(runCatching{reviewIndexPaths(file.copy(fromOutside=true),true)}.isFailure)
 }

 @Test fun pendingIndexWriteSurvivesCloseAndRequiresAReadAfterSettlement()=runBlocking {
  val model=GitReviewModel();val pending=CompletableDeferred<Boolean>();var calls=0
  suspend fun load()=ReviewChanges(true,null,listOf(ReviewFile("a.py",null," ","M")))
  model.refresh(::load);val write=async(start=CoroutineStart.UNDISPATCHED){model.changeIndex("a.py",false,{_,_->calls++;pending.await()},::load)}
  model.close();model.refresh(::load);assertTrue(model.indexPending);assertTrue(model.needsRefresh);model.changeIndex("a.py",false,{_,_->calls++;true},::load);assertEquals(1,calls)
  pending.complete(true);write.await();assertFalse(model.indexPending);assertTrue(model.needsRefresh);model.refresh(::load);assertFalse(model.needsRefresh)
 }

}
