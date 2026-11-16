package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
class SourceUndoContextsTest {
 private fun snapshot(source:String,caret:Int,value:String)=SourceUndoSnapshot(source,caret,caret,value)
 @Test fun nativeCoalescingRestoresOneCapturedBoundary(){
  val journal=SourceUndoContexts<String>({0});journal.record(snapshot("a",1,"first"),snapshot("ab",2,"second"));journal.record(snapshot("ab",2,"second"),snapshot("abc",3,"third"))
  assertEquals("first",journal.undo("abc","a",1,1)?.context);assertEquals("third",journal.redo("a","abc",3,3)?.context)
 }
 @Test fun staleAndAmbiguousContextNeverChoosesAnArbitraryState(){
  val stale=SourceUndoContexts<String>({0});stale.record(snapshot("a",1,"first"),snapshot("b",1,"second"));assertNull(stale.undo("foreign","a",1,1));assertFalse(stale.retained)
  val cycle=SourceUndoContexts<String>({0});cycle.record(snapshot("a",1,"one"),snapshot("b",1,"two"));cycle.record(snapshot("b",1,"two"),snapshot("a",1,"three"));cycle.record(snapshot("a",1,"three"),snapshot("b",1,"four"));assertNull(cycle.undo("b","a",1,1));assertFalse(cycle.retained)
 }
 @Test fun evictingMetadataDoesNotProposeAnySourceWrite(){
  val journal=SourceUndoContexts<String>({0},budget=8);journal.record(snapshot("aa",2,"one"),snapshot("bbb",3,"two"));assertFalse(journal.retained);assertNull(journal.undo("bbb","aa",2,2))
 }
 @Test fun oversizedContextDoesNotEvictUnrelatedHistoryOrOverflow(){
  val journal=SourceUndoContexts<String>({if(it=="huge")Long.MAX_VALUE else 0L},budget=400)
  journal.record(snapshot("a",1,"first"),snapshot("ab",2,"second"));val serial=journal.serial
  journal.record(snapshot("ab",2,"second"),snapshot("abc",3,"huge"));assertEquals(serial,journal.serial)
  assertFalse(journal.replaceLastAfter("a",snapshot("ab",2,"huge"),0));assertEquals("first",journal.undo("ab","a",1,1)?.context)
 }
 @Test fun emptyChoiceSlotsConsumeNativeHistoryBudget(){
  val field=SnippetField(1,0,0,List(10){""},0,null);val session=SnippetSession("",listOf(field),1,0)
  assertEquals(384L,snippetHistoryCost(session,400));assertEquals(401L,snippetHistoryCost(session.copy(fields=listOf(field.copy(choices=List(11){""}))),400))
  val journal=SourceUndoContexts<SnippetSession?>({snippetHistoryCost(it,400)},budget=400)
  journal.record(SourceUndoSnapshot("a",1,1,null),SourceUndoSnapshot("ab",2,2,null))
  journal.record(SourceUndoSnapshot("ab",2,2,session),SourceUndoSnapshot("abc",3,3,session))
  assertNotNull(journal.undo("ab","a",1,1))
 }

}
