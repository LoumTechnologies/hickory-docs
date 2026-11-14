<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Index test declarations without holding the editor on android

For maintainers replacing source discovery without coupling it to a gutter renderer.

Keep one observed source value per file. Repeated requests for the same active or successful observation do not multiply reads. A changed draft retires precise access immediately through the source equality check. Debounce pending work while retaining only its latest text.

<ui:copy id="source-test-index-android-observations">
package com.hickorydocs.simple
import androidx.compose.runtime.*
import kotlinx.coroutines.*
data class IndexedSourceTests(val path:String,val source:String,val facts:SourceTestScan?,val notice:String,val rows:List<Pair<Int,SourceTestCandidate>>?=null)
class SourceTestIndex(private val scope:CoroutineScope,private val debounce:Long=250,private val read:suspend (String,String)->SourceTestScan) {
    var records by mutableStateOf<Map<String,IndexedSourceTests>>(emptyMap());private set
    private data class Request(val path:String,val source:String,val due:Long,val current:()->String?)
    private val pending=linkedMapOf<String,Request>();private val active=mutableMapOf<String,Request>()
    private var timer:Job?=null;private var closed=false
    val inFlight get()=active.size;val queued get()=pending.size
    fun current(path:String,source:String)=records[path]?.takeIf {it.source==source&&it.facts!=null}
    fun enqueue(path:String,source:String,current:()->String?) {
        if(closed)return
        if(this.current(path,source)!=null||active[path]?.source==source){pending.remove(path);pump();return}
        if(pending[path]?.source==source)return
        pending[path]=Request(path,source,System.nanoTime()+debounce*1_000_000,current);pump()
    }
    fun stop(){closed=true;timer?.cancel();timer=null;pending.clear()}
</ui:copy>

Admit at most two reads and never overlap reads of the same path. Only ready jobs consume workers; another file can progress while one fails. The supplied current-source callback and absence of a newer queued value decide whether a reply can be installed. Stop refuses new work and late replies without claiming the underlying provider is immediately terminated.

<ui:copy id="source-test-index-android-workers">
    private fun pump(){
        timer?.cancel();timer=null;if(closed)return
        val now=System.nanoTime()
        for(request in pending.values.toList()){
            if(active.size>=2)break
            if(request.path in active||request.due>now)continue
            pending.remove(request.path);active[request.path]=request
            scope.launch {perform(request)}
        }
        if(active.size<2){pending.values.filter {it.path !in active}.minOfOrNull {it.due}?.let {due->
            timer=scope.launch {delay(maxOf(0,(due-System.nanoTime()+999_999)/1_000_000));pump()}
        }}
    }
    private suspend fun perform(request:Request){
        var facts:SourceTestScan?=null;var notice=""
        try {facts=read(request.path,request.source);require(facts.version==1&&facts.inputPath==request.path){"Unsupported source test observation. Refresh the app and engine together."}}
        catch(error:Exception){facts=null;notice=error.message.orEmpty()}
        if(!closed&&request.current()==request.source&&request.path !in pending){val record=IndexedSourceTests(request.path,request.source,facts,notice);records=records+(request.path to record.copy(rows=indexedTestRows(record)))}
        active.remove(request.path);pump()
    }
}
</ui:copy>

Use compiler-owned authored locations. An unmapped constructed runtime line cannot become a source gutter mark, and a foreign authored owner belongs to its own file. Plain files may use their runtime location because that is the supplied source. Validate line and UTF-16 column bounds once, and cache rows on the installed observation so each rendered gutter line does not split the whole source again. This primary-source index supplies inspection hints; opening a hint must refresh the full source-test view before offering execution against current contributors.

<ui:copy id="source-test-index-android-locations">
fun indexedTestRows(record:IndexedSourceTests?):List<Pair<Int,SourceTestCandidate>> {
    record?.rows?.let {return it}
    val facts=record?.facts ?: return emptyList()
    val lines=record.source.split("\n")
    return facts.candidates.mapNotNull {candidate->val point=candidate.source ?: candidate.target.takeIf {facts.scope=="plain"};point?.takeIf {it.path==record.path&&it.line>0&&it.line<=lines.size&&it.character>=0&&it.character<=lines[it.line-1].length}?.let {it.line-1 to candidate}}
}
</ui:copy>

These focused witnesses hold real asynchronous reads to prove the two-worker bound, duplicate suppression, edit coalescing, stale refusal, independent failure and stop behavior. Location witnesses distinguish primary, foreign and missing compiler ownership. They do not execute a language runner or establish native pointer acceptance.

<ui:copy id="source-test-index-android-witness">
package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
import kotlinx.coroutines.*
class SourceTestIndexTest {
    private fun facts(path:String)=SourceTestScan(1,path,"scan",mapOf(path to "hash"),emptyList(),"plain",true,emptyList(),emptyList(),emptyList(),emptyList())
    private data class Read(val path:String,val source:String,val result:CompletableDeferred<SourceTestScan>)
    private suspend fun settle(condition:()->Boolean){withTimeout(2000){while(!condition())yield()}}
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
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SourceTestIndex.kt"><ui:paste select="#source-test-index-android-observations"/><ui:paste select="#source-test-index-android-workers"/><ui:paste select="#source-test-index-android-locations"/></ui:file>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SourceTestIndexTest.kt"><ui:paste select="#source-test-index-android-witness"/></ui:file>
</ui:doc>
