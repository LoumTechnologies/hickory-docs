<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep an Android source-test debug choice tied to its observation

For maintainers connecting source-adjacent test debugging to the existing native
debugger. This mechanism prepares a guarded choice; it does not launch a runner,
advertise a supported test framework, or turn a debug process exit into a test
verdict. The engine still owns saved-source admission and runner planning.

Reuse the existing source-test request. Its opaque candidate identity and
analysis fingerprint describe a declaration or family, rather than a discovered
runner case. A containing-scope choice remains explicit. A missing name filter
cannot silently become a file run.

## Capture values and editor lifetimes independently

Defensive copies prevent a caller's mutable source map or candidate list from
changing a held choice. Object identities distinguish a retained editor and its
workspace services from a same-spelling replacement with identical bytes. The
observation identity retires a choice when its source-test reading is refreshed;
the namespace epoch retires it across an authoritative resource transition.

<ui:copy id="test-debug-android-policy">
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
</ui:copy>

## Check the admission boundary without inventing a launch protocol

These focused tests construct the existing saved-source request only. They must
not be described as a usable Debug action or a provider integration test. A later
UI adapter must call the final one-shot check immediately before its existing
debugger write, after asynchronous capability and preparation work. The shared
engine must independently recompute the scan, candidate, target bytes and saved
contributors before any runner effect.

<ui:copy id="test-debug-android-witness">
package com.hickorydocs.simple

import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test

class TestDebugTest {
    private val supported=JSONObject().put("test_debug_protocol",1)
    private val ready=TestDebugHostState(true,false,false,false)
    private fun context()=TestDebugContext(Any(),Any(),Any(),Any(),Any(),0)
    private fun fixture(sources:Map<String,String> = mapOf("helper.md" to "helper","unused.md" to "other")):Pair<TestSourceSnapshot,SourceTestScan> {
        val before=TestSourceSnapshot("workspace",TestSourceRequest("cases.md","source","project.md",sources))
        val point=TestSourcePoint("tests/test_case.py",4,0,"target-hash")
        val candidate=SourceTestCandidate("opaque-candidate","python","file","test_case","test_case","syntax",null,point)
        return before to SourceTestScan(1,"cases.md","analysis-fingerprint",mapOf("cases.md" to "source-hash","helper.md" to "helper-hash"),listOf(candidate),"project",true,emptyList(),emptyList(),emptyList(),listOf("unused.md"))
    }
    private fun refuses(body:()->Unit){assertTrue(runCatching(body).isFailure)}

    @Test fun exactChoiceProducesExistingSavedRequestOnce(){
        val (before,facts)=fixture();val context=context();val model=TestDebugChoices()
        val choice=model.offer(context,before,facts,"opaque-candidate",TestSourceSelection.NAME_FILTER)
        val request=model.take(choice,context,before,ready,supported)
        assertEquals("saved",request.getJSONObject("input").getString("kind"))
        assertEquals("analysis-fingerprint",request.getString("fingerprint"))
        assertEquals("opaque-candidate",request.getString("candidate_id"))
        assertEquals("name_filter",request.getJSONObject("selection").getString("kind"))
        assertFalse(request.has("argv"));assertFalse(request.has("program"));assertFalse(request.has("profile"))
        assertFalse(model.isCurrent(choice));refuses{model.take(choice,context,before,ready,supported)}
    }

    @Test fun heldChoiceCannotBecomeAnotherEditorOrObservation(){
        val (before,facts)=fixture();val context=context();val model=TestDebugChoices()
        val choice=model.offer(context,before,facts,"opaque-candidate",TestSourceSelection.NAME_FILTER)
        for(now in listOf(context.copy(workspace=Any()),context.copy(engine=Any()),context.copy(debugger=Any()),context.copy(editor=Any()),context.copy(observation=Any()),context.copy(namespace=1))){
            refuses{model.take(choice,now,before,ready,supported)};assertTrue(model.isCurrent(choice))
        }
        val replacement=model.offer(context,before,facts,"opaque-candidate",TestSourceSelection.CONTAINING_SCOPE)
        assertFalse(model.isCurrent(choice));refuses{model.take(choice,context,before,ready,supported)}
        model.cancel();refuses{model.take(replacement,context,before,ready,supported)}
    }

    @Test fun currentSourceAndSavedHostStateRemainAuthoritative(){
        val (before,facts)=fixture();val context=context();val model=TestDebugChoices()
        val choice=model.offer(context,before,facts,"opaque-candidate",TestSourceSelection.NAME_FILTER)
        for(now in listOf(before.copy(workspace="other"),before.copy(request=before.request.copy(source="edited")),before.copy(request=before.request.copy(sources=mapOf("helper.md" to "changed"))))){
            refuses{model.take(choice,context,now,ready,supported)}
        }
        for(state in listOf(ready.copy(saved=false),ready.copy(saving=true),ready.copy(deleted=true),ready.copy(activeSession=true))){refuses{model.take(choice,context,before,state,supported)}}
        val unrelated=before.copy(request=before.request.copy(sources=mapOf("helper.md" to "helper","unused.md" to "unrelated draft")))
        assertEquals("opaque-candidate",model.take(choice,context,unrelated,ready,supported).getString("candidate_id"))
    }

    @Test fun missingOrForeignProtocolNeverConsumesALaunchChoice(){
        val (before,facts)=fixture();val context=context();val model=TestDebugChoices()
        val choice=model.offer(context,before,facts,"opaque-candidate",TestSourceSelection.NAME_FILTER)
        for(value in listOf<Any?>(null,0,2,true,"1",1.5)){
            val catalog=JSONObject();if(value!=null)catalog.put("test_debug_protocol",value)
            assertFalse(testDebugProtocolSupported(catalog));refuses{model.take(choice,context,before,ready,catalog)};assertTrue(model.isCurrent(choice))
        }
        assertTrue(testDebugProtocolSupported(supported));model.cancel()
    }

    @Test fun mutableCallerMapsCannotRewriteHeldAnalysis(){
        val sources=mutableMapOf("helper.md" to "helper","unused.md" to "other");val (before,original)=fixture(sources)
        val candidates=original.candidates.toMutableList();val hashes=original.sourceHashes.toMutableMap();val facts=original.copy(candidates=candidates,sourceHashes=hashes)
        val context=context();val model=TestDebugChoices();val choice=model.offer(context,before,facts,"opaque-candidate",TestSourceSelection.NAME_FILTER)
        sources["helper.md"]="changed";candidates.clear();hashes.clear()
        val now=before.copy(request=before.request.copy(sources=mapOf("helper.md" to "helper","unused.md" to "other")))
        val body=model.take(choice,context,now,ready,supported)
        assertEquals("helper",body.getJSONObject("analysis").getJSONObject("sources").getString("helper.md"));assertEquals("opaque-candidate",body.getString("candidate_id"))
    }

    @Test fun dynamicDeclarationRequiresExplicitContainingScope(){
        val (before,facts)=fixture();val dynamic=facts.copy(candidates=facts.candidates.map{it.copy(nameFilter=null,confidence="dynamic")});val context=context();val model=TestDebugChoices()
        refuses{model.offer(context,before,dynamic,"opaque-candidate",TestSourceSelection.NAME_FILTER)};assertNull(model.current)
        val choice=model.offer(context,before,dynamic,"opaque-candidate",TestSourceSelection.CONTAINING_SCOPE)
        assertEquals("containing_scope",model.take(choice,context,before,ready,supported).getJSONObject("selection").getString("kind"))
        refuses{model.offer(context,before,facts,"unknown",TestSourceSelection.NAME_FILTER)}
        refuses{model.offer(context,before,facts.copy(version=2),"opaque-candidate",TestSourceSelection.NAME_FILTER)}
    }
}
</ui:copy>

## Construction and verification status

This mechanism is a foundation, not a connected Debug feature. The construction
is registered and generated; the full Android source compiled on 2026-11-16 and
all six `TestDebugTest` methods passed with zero skips. They exercised only this
choice policy and the existing saved-source request model. No new test-debug
wire, UI action or runner provider is implemented here. Positive acceptance
still requires real runner debugging through the engine and retained session
identity, as well as explicit unsupported-provider and stale-launch paths.
Physical input and accessibility acceptance remain separate.

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/TestDebug.kt"><ui:paste select="#test-debug-android-policy"/></ui:file>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/TestDebugTest.kt"><ui:paste select="#test-debug-android-witness"/></ui:file>
</ui:doc>
