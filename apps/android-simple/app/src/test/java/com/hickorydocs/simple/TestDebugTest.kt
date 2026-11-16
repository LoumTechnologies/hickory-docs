package com.hickorydocs.simple

import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import kotlinx.coroutines.async

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

    @Test fun planAndReceiptBindEverySavedDescriptorAndContributor(){
        val (before,old)=fixture();val hash=debugHash(before.request.source)
        val facts=old.copy(workspaceID="server-workspace",inputHash=hash,sourceHashes=mapOf("cases.md" to hash,"helper.md" to "helper-hash"))
        val model=TestDebugChoices();val choice=model.offer(context(),before,facts,"opaque-candidate",TestSourceSelection.NAME_FILTER)
        val value=JSONObject().put("version",1).put("test_debug_protocol",1).put("available",true).put("workspace_id",facts.workspaceID).put("input_path",facts.inputPath).put("input_hash",hash).put("source_hash",hash).put("analysis_fingerprint",facts.fingerprint).put("candidate_id",choice.candidate.id).put("candidate_name",choice.candidate.name).put("selection",JSONObject().put("kind","name_filter")).put("language","python").put("framework","pytest").put("source_uri",languageURI(facts.inputPath)).put("notices",org.json.JSONArray())
        val plan=TestDebugPlan.decode(value,choice)
        val binding=JSONObject(value.toString()).put("checked_sources",JSONObject(facts.sourceHashes))
        assertTrue(plan.admits(binding,facts));assertFalse(plan.admits(null,facts))
        for(key in listOf("workspace_id","input_path","input_hash","analysis_fingerprint","candidate_id","candidate_name","language","framework"))assertFalse(plan.admits(JSONObject(binding.toString()).put(key,"changed"),facts))
        assertFalse(plan.admits(JSONObject(binding.toString()).put("checked_sources",JSONObject(mapOf("cases.md" to hash))),facts))
        assertFalse(plan.admits(JSONObject(binding.toString()).put("checked_sources",JSONObject(facts.sourceHashes+mapOf("extra" to "hash"))),facts))
        for(key in listOf("workspace_id","input_path","input_hash","source_hash","analysis_fingerprint","candidate_id","candidate_name","language","source_uri"))refuses{TestDebugPlan.decode(JSONObject(value.toString()).put(key,"changed"),choice)}
        value.put("candidate_name","mutated after decode");assertTrue(plan.admits(binding,facts))
    }

    @Test fun dedicatedTransportRejectsStaleOwnedReceiptAndRetainsProcessOutcome()=kotlinx.coroutines.runBlocking {
        val (before,old)=fixture();val hash=debugHash(before.request.source)
        val facts=old.copy(workspaceID="server-workspace",inputHash=hash,sourceHashes=mapOf("cases.md" to hash))
        val choices=TestDebugChoices();val choice=choices.offer(context(),before,facts,"opaque-candidate",TestSourceSelection.NAME_FILTER)
        val descriptor=JSONObject().put("version",1).put("test_debug_protocol",1).put("available",true).put("workspace_id",facts.workspaceID).put("input_path",facts.inputPath).put("input_hash",hash).put("source_hash",hash).put("analysis_fingerprint",facts.fingerprint).put("candidate_id",choice.candidate.id).put("candidate_name",choice.candidate.name).put("selection",JSONObject().put("kind","name_filter")).put("language","python").put("framework","pytest").put("source_uri",languageURI(facts.inputPath)).put("notices",org.json.JSONArray())
        val plan=TestDebugPlan.decode(descriptor,choice);val binding=JSONObject(descriptor.toString()).put("checked_sources",JSONObject(facts.sourceHashes))
        val file=FileNode("cases.md","cases.md",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText(before.request.source,"saved"))
        val debug=NativeDebugger(Engine("http://127.0.0.1:1"),this);val sent=mutableListOf<JSONObject>();debug.emitForTest={sent+=it};var current=true;var admitted:String?=null;var rejected="";var outcome=""
        fun started(request:JSONObject,session:String)=JSONObject().put("event","started").put("request_id",request.getString("request_id")).put("doc",request.getString("doc")).put("session",session).put("source_hash",hash).put("source_test",binding)
        suspend fun launch(){debug.launchSourceTest(buffer,JSONObject().put("candidate_id",choice.candidate.id),plan,facts,{current},{admitted=it},{rejected=it},{id,state,exit->outcome="$id:$state:$exit"})}
        try {
            val unsupported=TestDebugPlan.decode(JSONObject(descriptor.toString()).put("available",false).put("reason","Runner does not provide test debugging"),choice)
            debug.launchSourceTest(buffer,JSONObject(),unsupported,facts,{true},{error("unexpected admission")},{rejected=it},{_,_,_->})
            assertTrue(sent.isEmpty());assertTrue(rejected.contains("No Start"))
            launch();val first=sent.last();assertEquals("start_test",first.getString("op"));assertFalse(first.has("profile"));assertFalse(first.has("program"));assertEquals(0,first.getJSONArray("breakpoints").length())
            current=false;debug.receive(started(first,"retired"));assertNull(admitted);assertTrue(rejected.isNotBlank());assertEquals("stop",sent.last().getString("op"));assertEquals("retired",sent.last().getString("session"))
            current=true;launch();val second=sent.last();val count=sent.size
            debug.receive(started(first,"foreign").put("request_id","unknown"));assertEquals(count,sent.size);assertNull(admitted)
            debug.receive(started(second,"current"));assertEquals("current",admitted)
            val activeCount=sent.size;launch();assertEquals(activeCount,sent.size);assertTrue(rejected.contains("another debugger"))
            current=false;debug.receive(JSONObject().put("event","finished").put("session","current").put("exit_code",0));assertEquals("current:finished:0",outcome)
        }finally{debug.close()}
    }

    @Test(timeout=10000) fun dismissAndReopenCannotReviveDelayedPlan()=kotlinx.coroutines.runBlocking {
        val entered=java.util.concurrent.CountDownLatch(1);val release=java.util.concurrent.CountDownLatch(1);val reply=java.util.concurrent.atomic.AtomicReference("{}")
        val server=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0)
        server.createContext("/api/tests/debug-plan"){exchange->entered.countDown();release.await(5,java.util.concurrent.TimeUnit.SECONDS);val bytes=reply.get().toByteArray();exchange.sendResponseHeaders(200,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}};server.start()
        val engine=Engine("http://127.0.0.1:${server.address.port}");val workspace=Workspace(this);workspace.engine=engine
        val file=FileNode("cases.md","cases.md",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText("source","saved"));workspace.buffers[file.path]=buffer;workspace.show(file.path);workspace.showTests=true
        val debug=NativeDebugger(engine,this);workspace.debugger=debug;val sent=mutableListOf<JSONObject>();debug.emitForTest={sent+=it}
        val model=workspace.testSources;val before=model.snapshot(workspace)!!;val (_,old)=fixture();val hash=debugHash(before.request.source);val facts=old.copy(workspaceID="server-workspace",inputHash=hash,sourceHashes=mapOf("cases.md" to hash));model.observation=before to facts
        val candidate=facts.candidates.single()
        val descriptor=JSONObject().put("version",1).put("test_debug_protocol",1).put("available",true).put("workspace_id",facts.workspaceID).put("input_path",facts.inputPath).put("input_hash",hash).put("source_hash",hash).put("analysis_fingerprint",facts.fingerprint).put("candidate_id",candidate.id).put("candidate_name",candidate.name).put("selection",JSONObject().put("kind","name_filter")).put("language","python").put("framework","pytest").put("source_uri",languageURI(facts.inputPath)).put("notices",org.json.JSONArray())
        assertTrue(TestDebugPlan.decode(descriptor,TestDebugChoices().offer(context(),before,facts,candidate.id,TestSourceSelection.NAME_FILTER)).available);reply.set(descriptor.toString())
        try {
            val pending=async{model.debug(facts.candidates.single(),TestSourceSelection.NAME_FILTER,workspace)}
            repeat(100){if(entered.count>0)kotlinx.coroutines.delay(10)};assertEquals(0L,entered.count);assertTrue(model.debugging)
            workspace.showTests=false;assertFalse(model.debugging);workspace.showTests=true;release.countDown();pending.await()
            assertTrue(sent.isEmpty());assertFalse(model.debugging);assertEquals("idle",debug.status)
        }finally{release.countDown();debug.close();server.stop(0)}
    }

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
