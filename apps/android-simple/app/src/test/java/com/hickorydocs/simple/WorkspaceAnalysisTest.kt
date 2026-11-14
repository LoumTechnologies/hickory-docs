package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
import kotlinx.coroutines.*
class WorkspaceAnalysisTest {
    private val files=(0 until 5).map {FileNode("$it.py","$it.py",false,emptyList(),null,null,"python")}
    private fun buffer(file:FileNode)=DocumentBuffer(file,FileText("source","base"))
    @Test fun drainBeforeRestart()=runBlocking {
        val model=WorkspaceAnalysis();var loads=0;var active=0;var peak=0;val held=CompletableDeferred<Unit>();val entered=CompletableDeferred<Unit>()
        val run=async {model.run(files,{file->loads++;buffer(file)},{_,_->active++;peak=maxOf(peak,active);if(active==2) entered.complete(Unit);held.await();active--;emptyList()})}
        withTimeout(2000) {entered.await()};model.stop();assertTrue(model.running)
        assertFalse(model.run(files,{error("must not load")},{_,_->emptyList()}));held.complete(Unit);run.await()
        assertEquals(2,loads);assertEquals(2,peak);assertEquals(2,model.completed);assertFalse(model.running)
    }
    @Test fun staleAndUnavailableAreIndependent()=runBlocking {
        val model=WorkspaceAnalysis();val draft=buffer(files[0]);var reads=0
        model.run(files,{file->if(file.path=="0.py")draft else buffer(file)},{file,_->reads++;if(file.path=="0.py")draft.state.edit {replace(0,length,"draft")};check(file.path!="1.py") {"provider unavailable"};emptyList()})
        assertEquals(5,reads);assertEquals(5,model.completed);assertEquals("draft",draft.text);assertEquals("stale",model.knowledge["0.py"]?.state);assertNull(model.current("0.py","source"))
        assertEquals("provider unavailable",model.knowledge["1.py"]?.notice);assertNotNull(model.current("2.py","source"));assertNull(model.current("2.py","changed"))
    }
}
