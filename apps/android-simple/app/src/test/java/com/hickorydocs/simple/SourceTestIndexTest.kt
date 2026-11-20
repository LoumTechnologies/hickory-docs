package com.hickorydocs.simple
import androidx.compose.runtime.snapshotFlow
import androidx.compose.runtime.snapshots.Snapshot
import androidx.compose.ui.text.TextRange
import org.junit.Assert.*
import org.junit.Test
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.flow.toList
class SourceTestIndexTest {
    private fun facts(path:String)=SourceTestScan(1,path,"scan",mapOf(path to "hash"),emptyList(),"plain",true,emptyList(),emptyList(),emptyList(),emptyList())
    private data class Read(val path:String,val source:String,val result:CompletableDeferred<SourceTestScan>)
    private suspend fun settle(condition:()->Boolean){withTimeout(2000){while(!condition())yield()}}
    @Test fun observesIndependentLateCapabilitiesOnTheSameStartupEditor()=runBlocking {
        val source = "fn test_selected() {}\n"
        val path = "src/lib.rs"
        val buffer = DocumentBuffer(FileNode("lib.rs",path,false,emptyList(),null,null),FileText(source,"hash"),startupObservation=true)
        val selection = TextRange(3,7)
        buffer.state.edit { this.selection = selection }
        val index = SourceTestIndex(this,0){name,_->facts(name)}
        val coverage = SimpleCoverageState()
        val seen = mutableListOf<Pair<SourceTestIndex?,SimpleCoverageState?>>()
        val observer = launch(start=CoroutineStart.UNDISPATCHED) {
            snapshotFlow { buffer.testIndex to buffer.coverageSource }.take(3).toList(seen)
        }
        try {
            assertEquals(listOf(null to null),seen.toList())
            buffer.coverageSource = coverage
            Snapshot.sendApplyNotifications()
            settle { seen.size == 2 }
            assertEquals(null to coverage,seen.last())
            buffer.testIndex = index
            Snapshot.sendApplyNotifications()
            withTimeout(2000) { observer.join() }
            assertEquals(index to coverage,seen.last())
            assertEquals(path,buffer.file.path)
            assertEquals(source,buffer.text)
            assertEquals(selection,buffer.state.selection)
        } finally {
            observer.cancel()
            index.stop()
        }
    }
    @Test fun coalescesDraftsAndDrainsAfterIndependentFailure()=runBlocking {
        val texts=mutableMapOf("a" to "first","b" to "source","c" to "source");val reads=mutableListOf<Read>()
        val index=SourceTestIndex(this,0){path,source->val result=CompletableDeferred<SourceTestScan>();reads+=Read(path,source,result);result.await()}
        fun enqueue(path:String){index.enqueue(path,texts[path]!!){texts[path]}}
        enqueue("a");enqueue("a");enqueue("b");settle {reads.size==2};assertEquals(2,index.inFlight)
        texts["a"]="middle";enqueue("a");texts["a"]="latest";enqueue("a");enqueue("c");assertEquals(2,index.queued)
        reads.first {it.path=="a"}.result.complete(facts("a"));settle {reads.size==3};assertNull(index.current("a","first"));assertEquals("latest",reads[2].source);assertEquals(2,index.inFlight)
        reads.first {it.path=="b"}.result.completeExceptionally(IllegalStateException("unavailable"));settle {reads.size==4};assertTrue(index.records["b"]!!.notice.contains("unavailable"));assertEquals("c",reads[3].path)
        reads[2].result.complete(facts("a"));reads[3].result.complete(facts("c"));settle {index.inFlight==0}
        assertNotNull(index.current("a","latest"));assertNull(index.current("a","changed"));assertNotNull(index.current("c","source"));enqueue("a");assertEquals(4,reads.size);index.stop()
    }
    @Test fun stopAndCompilerOwnedRows()=runBlocking {
        val reads=mutableListOf<Read>();val index=SourceTestIndex(this,0){path,source->val result=CompletableDeferred<SourceTestScan>();reads+=Read(path,source,result);result.await()}
        for(path in listOf("a","b","c"))index.enqueue(path,"source"){"source"};settle {reads.size==2};index.stop()
        reads.forEach {it.result.complete(facts(it.path))};settle {index.inFlight==0};assertTrue(index.records.isEmpty());assertEquals(0,index.queued)
        val point=TestSourcePoint("generated.py",2,4,"hash");val candidate=SourceTestCandidate("case","python","file","test_case","test_case","syntax",null,point)
        val scan=facts("note.md").copy(scope="document",candidates=listOf(candidate))
        fun record(facts:SourceTestScan)=IndexedSourceTests("note.md","\nsource",facts,"")
        assertTrue(indexedTestRows(record(scan)).isEmpty())
        assertTrue(indexedTestRows(record(scan.copy(candidates=listOf(candidate.copy(source=point.copy(path="other.md")))))).isEmpty())
        assertTrue(indexedTestRows(record(scan.copy(candidates=listOf(candidate.copy(source=point.copy(path="note.md",line=100)))))).isEmpty())
        assertEquals(1,indexedTestRows(record(scan.copy(candidates=listOf(candidate.copy(source=point.copy(path="note.md")))))).first().first)
    }
}
