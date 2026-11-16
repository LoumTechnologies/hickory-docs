package com.hickorydocs.simple

import java.util.Collections
import java.util.UUID
import org.json.JSONObject

data class TestDebugContext(
    val workspace:Any,
    val engine:Any,
    val debugger:Any,
    val editor:Any,
    val observation:Any,
    val namespace:Long,
) {
    fun same(other:TestDebugContext)=workspace===other.workspace&&engine===other.engine&&debugger===other.debugger&&editor===other.editor&&observation===other.observation&&namespace==other.namespace
}

data class TestDebugHostState(val saved:Boolean,val saving:Boolean,val deleted:Boolean,val activeSession:Boolean)

class TestDebugChoice internal constructor(
    val identity:String,
    val context:TestDebugContext,
    internal val snapshot:TestSourceSnapshot,
    internal val facts:SourceTestScan,
    val candidate:SourceTestCandidate,
    val selection:TestSourceSelection,
)

private fun <K,V> testDebugMap(values:Map<K,V>):Map<K,V> = Collections.unmodifiableMap(LinkedHashMap(values))
private fun <T> testDebugList(values:List<T>):List<T> = Collections.unmodifiableList(ArrayList(values))
private fun testDebugSnapshot(value:TestSourceSnapshot)=value.copy(request=value.request.copy(sources=testDebugMap(value.request.sources)))
private fun testDebugFacts(value:SourceTestScan)=value.copy(
    sourceHashes=testDebugMap(value.sourceHashes),
    candidates=testDebugList(value.candidates.map{it.copy(source=it.source?.copy(),target=it.target.copy())}),
    notices=testDebugList(value.notices),
    unsupportedLanguages=testDebugList(value.unsupportedLanguages),
    unclassifiedOutputs=testDebugList(value.unclassifiedOutputs),
    unusedSources=testDebugList(value.unusedSources),
)

fun testDebugProtocolSupported(catalog:JSONObject)=
    (catalog.opt("test_debug_protocol") as? Number)?.toString()?.toIntOrNull()==1

class TestDebugChoices {
    var current:TestDebugChoice?=null
        private set

    fun cancel(){current=null}
    fun isCurrent(choice:TestDebugChoice)=current===choice

    fun offer(context:TestDebugContext,snapshot:TestSourceSnapshot,facts:SourceTestScan,candidate:String,selection:TestSourceSelection):TestDebugChoice {
        cancel()
        val before=testDebugSnapshot(snapshot)
        val observation=testDebugFacts(facts)
        // Validate the existing saved-run selection semantics before retaining it.
        sourceTestRun(before,observation,before,candidate,selection)
        val selected=observation.candidates.first{it.id==candidate}
        return TestDebugChoice(UUID.randomUUID().toString(),context,before,observation,selected,selection).also{current=it}
    }

    fun take(choice:TestDebugChoice,context:TestDebugContext,now:TestSourceSnapshot,state:TestDebugHostState,catalog:JSONObject):JSONObject {
        require(isCurrent(choice)){"The test debug choice was replaced or closed. Choose the current source test again."}
        require(choice.context.same(context)){"The source editor, observation or workspace namespace changed. Refresh source tests before debugging."}
        require(state.saved&&!state.saving&&!state.deleted){"Save or restore the displayed source before debugging its tests."}
        require(!state.activeSession){"A debugger session is already active. Inspect or stop it before debugging another test."}
        require(testDebugProtocolSupported(catalog)){"Test debugging is unavailable from this engine. Reconnect with test-debug protocol 1 support before launching."}
        val request=sourceTestRun(choice.snapshot,choice.facts,now,choice.candidate.id,choice.selection)
        // The caller invokes take immediately before its wire send. A lost send
        // or response cannot reuse this choice; another explicit choice is needed.
        current=null
        return request
    }
}
