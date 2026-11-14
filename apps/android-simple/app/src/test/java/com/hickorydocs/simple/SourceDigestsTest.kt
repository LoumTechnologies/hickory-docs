package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class SourceDigestsTest {
 @Test fun coalescingRejectsObsoleteBytesAndClosedReplies()=runBlocking {
  val jobs=mutableListOf<Pair<String,CompletableDeferred<String>>>()
  val index=SourceDigests(this){source->val done=CompletableDeferred<String>();jobs.add(source to done);done.await()}
  var source="é";index.enqueue("a",source){source};yield();source="e\u0301";index.enqueue("a","intermediate"){source};index.enqueue("a",source){source};index.enqueue("b","other"){"other"};assertEquals(1,jobs.size)
  jobs[0].second.complete("old");repeat(10){yield()};assertNull(index.current("a","é"));assertEquals(source,jobs[1].first)
  jobs[1].second.complete("new");repeat(10){yield()};assertEquals("new",index.current("a",source));assertNull(index.current("a","é"));assertEquals("other",jobs[2].first)
  index.stop();jobs[2].second.complete("late");repeat(10){yield()};assertNull(index.current("b","other"))
 }
 @Test fun failureDoesNotBlockAnotherSource()=runBlocking {
  val index=SourceDigests(this){source->if(source=="bad")error("unavailable");"good"};index.enqueue("a","bad"){"bad"};index.enqueue("b","ok"){"ok"};repeat(10){yield()};assertNull(index.current("a","bad"));assertEquals("good",index.current("b","ok"));index.stop()
 }
}
