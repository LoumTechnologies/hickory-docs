package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class ConstructionReadingTest {
 @Test fun boundEntryOverridePreservesLiteralAddress()=kotlinx.coroutines.runBlocking {
  val model=ConstructionReadingModel()
  val address=" folder/entry.md "
  model.read(entryOverride=address){entry,focus->
   assertEquals(address,entry)
   assertNull(focus)
   value(entry)
  }
  assertEquals(address,model.value?.entry)
 }
 fun value(entry:String="project.md")=ConstructionReading(entry,null,emptyList(),0,false,emptyList(),0,false,"declaration","not_queried",emptyList(),0)
 @Test fun closedAndSupersededRepliesCannotReplaceTheCapturedReading()=runBlocking{
  val model=ConstructionReadingModel();val pending=CompletableDeferred<ConstructionReading>();val old=async(start=CoroutineStart.UNDISPATCHED){model.read{_,_->pending.await()}}
  model.read{_,_->value().copy(notices=listOf("new"))};pending.complete(value());old.await();assertEquals(listOf("new"),model.value?.notices)
  val late=CompletableDeferred<ConstructionReading>();val closed=async(start=CoroutineStart.UNDISPATCHED){model.read{_,_->late.await()}};model.close();late.complete(value());closed.await();assertNull(model.value)
  model.read{_,_->value("other.md")};assertNull(model.value);assertTrue(model.notice.contains("identity changed"))
 }
 @Test fun sourcePointsRequireExactHashesAndScalarBoundariesAndRetireOlderLinks()=runBlocking{
  val source="😀e\u0301\né😀code";val hash=java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}
  val point=TestSourcePoint("part.md",2,3,hash)
  assertEquals(source.indexOf("code"),constructionPointOffset(source,point,hash))
  assertNull(constructionPointOffset(source,point.copy(character=2),hash));assertNull(constructionPointOffset(source,point.copy(line=0),hash));assertNull(constructionPointOffset(source,point.copy(character=99),hash));assertNull(constructionPointOffset(source,point,"b".repeat(64)))
  val model=ConstructionReadingModel();model.read{_,_->value()};val old=model.sourceLink();val latest=model.sourceLink();assertFalse(old());assertTrue(latest());model.close();assertFalse(latest())
 }

}
