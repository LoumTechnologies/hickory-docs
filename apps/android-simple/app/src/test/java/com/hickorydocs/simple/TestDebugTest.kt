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
