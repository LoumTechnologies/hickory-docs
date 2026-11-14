package com.hickorydocs.simple
import org.json.JSONObject

data class TestSourceRequest(val path:String,val source:String,val projectEntry:String?=null,val sources:Map<String,String> = emptyMap()) {
    fun body()=JSONObject().put("path",path).put("source",source).put("sources",JSONObject(sources)).apply {projectEntry?.let {put("project_entry",it)}}
}
data class TestSourceSnapshot(val workspace:String,val request:TestSourceRequest)
data class TestSourcePoint(val path:String,val line:Int,val character:Int,val hash:String) {
    companion object {fun decode(value:JSONObject)=TestSourcePoint(value.getString("path"),value.getInt("line"),value.getInt("character"),value.getString("hash"))}
}
data class SourceTestCandidate(val id:String,val language:String,val runScope:String,val name:String,val nameFilter:String?,val confidence:String,val source:TestSourcePoint?,val target:TestSourcePoint) {
    val point get()=source ?: target
    companion object {fun decode(value:JSONObject)=SourceTestCandidate(value.getString("id"),value.getString("language"),value.getString("run_scope"),value.getString("name"),if(value.isNull("name_filter")) null else value.getString("name_filter"),value.getString("confidence"),value.optJSONObject("source")?.let {TestSourcePoint.decode(it)},TestSourcePoint.decode(value.getJSONObject("target")))}
}
data class SourceTestScan(val version:Int,val inputPath:String,val fingerprint:String,val sourceHashes:Map<String,String>,val candidates:List<SourceTestCandidate>,val scope:String,val syntaxComplete:Boolean,val notices:List<String>,val unsupportedLanguages:List<String>,val unclassifiedOutputs:List<String>,val unusedSources:List<String>) {
    companion object {
        fun decode(value:JSONObject):SourceTestScan {
            fun strings(key:String)=value.getJSONArray(key).let {array->(0 until array.length()).map {array.getString(it)}}
            val hashes=value.getJSONObject("source_hashes");val candidates=value.getJSONArray("candidates")
            return SourceTestScan(value.getInt("version"),value.getString("input_path"),value.getString("fingerprint"),hashes.keys().asSequence().associateWith {hashes.getString(it)},(0 until candidates.length()).map {SourceTestCandidate.decode(candidates.getJSONObject(it))},value.getString("scope"),value.getBoolean("syntax_complete"),strings("notices"),strings("unsupported_languages"),strings("unclassified_outputs"),strings("unused_sources"))
        }
    }
}
fun testSourceCurrent(before:TestSourceSnapshot,facts:SourceTestScan,now:TestSourceSnapshot):Boolean {
    val a=before.request;val b=now.request
    if(facts.version!=1||facts.inputPath!=a.path||before.workspace!=now.workspace||a.path!=b.path||a.source!=b.source||a.projectEntry!=b.projectEntry)return false
    return facts.sourceHashes.keys.all {it==a.path || a.sources[it]==b.sources[it]}
}
enum class TestSourceSelection(val wire:String) {NAME_FILTER("name_filter"),CONTAINING_SCOPE("containing_scope")}
fun sourceTestRun(before:TestSourceSnapshot,facts:SourceTestScan,now:TestSourceSnapshot,id:String,selection:TestSourceSelection):JSONObject {
    require(testSourceCurrent(before,facts,now)) {"Test source changed. Refresh candidates before running."}
    val candidate=facts.candidates.firstOrNull {it.id==id} ?: error("Candidate unavailable. Refresh test candidates.")
    require(selection!=TestSourceSelection.NAME_FILTER||candidate.nameFilter!=null) {"This declaration needs runner discovery or an explicit containing-scope run."}
    return JSONObject().put("version",1).put("input",JSONObject().put("kind","saved")).put("analysis",before.request.body()).put("fingerprint",facts.fingerprint).put("candidate_id",id).put("selection",JSONObject().put("kind",selection.wire))
}
