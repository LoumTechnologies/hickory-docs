<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retain source test observations on android

For maintainers replacing the test UI without replacing source and execution admission.

Keep source snapshots and provider observations as distinct values. Unknown source mappings stay absent; target runtime locations remain available without guessing an authored owner.

<ui:copy id="test-sources-android-values">
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
data class SourceTestScan(val version:Int,val inputPath:String,val fingerprint:String,val sourceHashes:Map<String,String>,val candidates:List<SourceTestCandidate>,val scope:String,val syntaxComplete:Boolean,val notices:List<String>,val unsupportedLanguages:List<String>,val unclassifiedOutputs:List<String>,val unusedSources:List<String>,val workspaceID:String="",val inputHash:String="") {
    companion object {
        fun decode(value:JSONObject):SourceTestScan {
            fun strings(key:String)=value.getJSONArray(key).let {array->(0 until array.length()).map {array.getString(it)}}
            val hashes=value.getJSONObject("source_hashes");val candidates=value.getJSONArray("candidates")
            return SourceTestScan(value.getInt("version"),value.getString("input_path"),value.getString("fingerprint"),hashes.keys().asSequence().associateWith {hashes.getString(it)},(0 until candidates.length()).map {SourceTestCandidate.decode(candidates.getJSONObject(it))},value.getString("scope"),value.getBoolean("syntax_complete"),strings("notices"),strings("unsupported_languages"),strings("unclassified_outputs"),strings("unused_sources"),value.optString("workspace_id"),value.optString("input_hash"))
        }
    }
}
</ui:copy>

A view becomes stale when its primary draft, selected construction entry, workspace identity or any contributing supplied draft changes. An unrelated unused draft must not retire it. Newly retained contributor text requires a fresh scan. Saved files are checked independently by the engine at execution admission.

<ui:copy id="test-sources-android-current">
fun testSourceCurrent(before:TestSourceSnapshot,facts:SourceTestScan,now:TestSourceSnapshot):Boolean {
    val a=before.request;val b=now.request
    if(facts.version!=1||facts.inputPath!=a.path||before.workspace!=now.workspace||a.path!=b.path||a.source!=b.source||a.projectEntry!=b.projectEntry)return false
    return facts.sourceHashes.keys.all {it==a.path || a.sources[it]==b.sources[it]}
}
</ui:copy>

Build a versioned saved-run request only for a candidate in the current observation. Dynamic declarations cannot become name filters; a user must explicitly select the containing runner scope or discover concrete cases.

<ui:copy id="test-sources-android-selection">
enum class TestSourceSelection(val wire:String) {NAME_FILTER("name_filter"),CONTAINING_SCOPE("containing_scope")}
fun sourceTestRun(before:TestSourceSnapshot,facts:SourceTestScan,now:TestSourceSnapshot,id:String,selection:TestSourceSelection):JSONObject {
    require(testSourceCurrent(before,facts,now)) {"Test source changed. Refresh candidates before running."}
    val candidate=facts.candidates.firstOrNull {it.id==id} ?: error("Candidate unavailable. Refresh test candidates.")
    require(selection!=TestSourceSelection.NAME_FILTER||candidate.nameFilter!=null) {"This declaration needs runner discovery or an explicit containing-scope run."}
    return JSONObject().put("version",1).put("input",JSONObject().put("kind","saved")).put("analysis",before.request.body()).put("fingerprint",facts.fingerprint).put("candidate_id",id).put("selection",JSONObject().put("kind",selection.wire))
}
</ui:copy>

The focused witnesses change primary/contributor/workspace observations, preserve an unrelated draft, reject a future protocol version, and distinguish dynamic declaration filters from explicit containing-scope execution.

<ui:copy id="test-sources-android-witness">
package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
class TestSourcesTest {
    private fun fixture():Pair<TestSourceSnapshot,SourceTestScan> {
        val before=TestSourceSnapshot("w1",TestSourceRequest("cases.md","draft","project.md",mapOf("helper.md" to "helper","unused.md" to "old")))
        val point=TestSourcePoint("test.py",1,4,"target");val candidate=SourceTestCandidate("case","python","file","test_case","test_case","syntax",null,point)
        return before to SourceTestScan(1,"cases.md","scan",mapOf("cases.md" to "draft-hash","helper.md" to "helper-hash"),listOf(candidate),"project",true,emptyList(),emptyList(),emptyList(),listOf("unused.md"))
    }
    @Test fun contributorAndWorkspaceIdentity() {
        val (before,facts)=fixture();assertTrue(testSourceCurrent(before,facts,before))
        assertFalse(testSourceCurrent(before,facts,before.copy(workspace="w2")))
        assertFalse(testSourceCurrent(before,facts,before.copy(request=before.request.copy(source="changed"))))
        assertFalse(testSourceCurrent(before,facts,before.copy(request=before.request.copy(sources=mapOf("helper.md" to "changed")))))
        assertTrue(testSourceCurrent(before,facts,before.copy(request=before.request.copy(sources=mapOf("helper.md" to "helper","unused.md" to "new")))))
        assertFalse(testSourceCurrent(before,facts.copy(version=2),before))
    }
    @Test fun explicitSelectionAndDynamicRefusal() {
        val (before,facts)=fixture();val value=sourceTestRun(before,facts,before,"case",TestSourceSelection.NAME_FILTER)
        assertEquals("saved",value.getJSONObject("input").getString("kind"));assertEquals("scan",value.getString("fingerprint"))
        val dynamic=facts.copy(candidates=listOf(facts.candidates[0].copy(nameFilter=null)))
        assertTrue(runCatching {sourceTestRun(before,dynamic,before,"case",TestSourceSelection.NAME_FILTER)}.isFailure)
        assertEquals("containing_scope",sourceTestRun(before,dynamic,before,"case",TestSourceSelection.CONTAINING_SCOPE).getJSONObject("selection").getString("kind"))
        assertTrue(runCatching {sourceTestRun(before,facts,before,"missing",TestSourceSelection.NAME_FILTER)}.isFailure)
    }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/TestSources.kt"><ui:paste select="#test-sources-android-values"/><ui:paste select="#test-sources-android-current"/><ui:paste select="#test-sources-android-selection"/></ui:file>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/TestSourcesTest.kt"><ui:paste select="#test-sources-android-witness"/></ui:file>
</ui:doc>
