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
}
