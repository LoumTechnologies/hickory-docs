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
}
