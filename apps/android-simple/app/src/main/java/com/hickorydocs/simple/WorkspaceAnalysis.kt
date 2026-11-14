package com.hickorydocs.simple
import androidx.compose.runtime.*
import kotlinx.coroutines.*
data class AnalysisKnowledge(val path:String,val source:String?,val locations:List<SemanticLocation>,val state:String,val notice:String)
class WorkspaceAnalysis {
    var running by mutableStateOf(false); private set
    var stopping by mutableStateOf(false); private set
    var completed by mutableStateOf(0); private set
    var scope by mutableStateOf(workspaceAnalysisScope(emptyList())); private set
    val knowledge = mutableStateMapOf<String,AnalysisKnowledge>()
    fun stop() { if(running) stopping=true }
    suspend fun run(files:List<FileNode>,load:suspend (FileNode)->DocumentBuffer,outline:suspend (FileNode,String)->List<SemanticLocation>):Boolean {
        if(running) return false
        scope=workspaceAnalysisScope(files);running=true;stopping=false;completed=0;knowledge.clear()
        var cursor=0
        suspend fun worker() {
            while(!stopping && cursor<scope.files.size) {
                val file=scope.files[cursor++];var source:String?=null
                try {
                    val buffer=load(file);source=buffer.text
                    val locations=outline(file,buffer.text);val current=buffer.text==source
                    knowledge[file.path]=AnalysisKnowledge(file.path,source,if(current) locations else emptyList(),if(current) "ready" else "stale",if(current) "Provider outline returned; diagnostics arrive independently." else "Source changed during analysis. Run again.")
                } catch(cancelled:CancellationException) { throw cancelled }
                catch(error:Exception) {knowledge[file.path]=AnalysisKnowledge(file.path,source,emptyList(),"unavailable",error.message ?: "Analysis unavailable. Retry language services.")}
                completed++
            }
        }
        try {coroutineScope {val first=async {worker()};val second=async {worker()};first.await();second.await()}} finally {running=false;stopping=false}
        return true
    }
    fun current(path:String,source:String):AnalysisKnowledge? = knowledge[path]?.takeIf {it.state=="ready" && it.source==source}
}
