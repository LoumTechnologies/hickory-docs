package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class ConstructionReadingTest {
 fun value(entry:String="project.md")=ConstructionReading(entry,null,emptyList(),0,false,emptyList(),0,false,"declaration","not_queried",emptyList(),0)
 @Test fun closedAndSupersededRepliesCannotReplaceTheCapturedReading()=runBlocking{
  val model=ConstructionReadingModel();val pending=CompletableDeferred<ConstructionReading>();val old=async(start=CoroutineStart.UNDISPATCHED){model.read{_,_->pending.await()}}
  model.read{_,_->value().copy(notices=listOf("new"))};pending.complete(value());old.await();assertEquals(listOf("new"),model.value?.notices)
  val late=CompletableDeferred<ConstructionReading>();val closed=async(start=CoroutineStart.UNDISPATCHED){model.read{_,_->late.await()}};model.close();late.complete(value());closed.await();assertNull(model.value)
  model.read{_,_->value("other.md")};assertNull(model.value);assertTrue(model.notice.contains("identity changed"))
 }
}
