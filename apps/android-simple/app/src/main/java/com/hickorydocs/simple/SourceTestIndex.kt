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
fun indexedTestRows(record:IndexedSourceTests?):List<Pair<Int,SourceTestCandidate>> {
    record?.rows?.let {return it}
    val facts=record?.facts ?: return emptyList()
    val lines=record.source.split("\n")
    return facts.candidates.mapNotNull {candidate->val point=candidate.source ?: candidate.target.takeIf {facts.scope=="plain"};point?.takeIf {it.path==record.path&&it.line>0&&it.line<=lines.size&&it.character>=0&&it.character<=lines[it.line-1].length}?.let {it.line-1 to candidate}}
}
