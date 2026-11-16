package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
import org.json.JSONObject
import org.json.JSONArray
import androidx.compose.ui.text.TextRange
import kotlinx.coroutines.async
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class SourceTemplateTest {
    private fun field(id:Int,index:Int,from:Int,to:Int)=JSONObject().put("id",id).put("parent",JSONObject.NULL).put("index",index).put("from",from).put("to",to).put("choices",JSONArray())
    private fun observation(request:JSONObject)=JSONObject().put("version",1).put("template_protocol",1).put("path",request.getString("path")).put("selection",JSONObject(request.getJSONObject("selection").toString())).put("source_hash",debugHash(request.getString("source"))).put("workspace_id","workspace").put("language","python").put("catalog_fingerprint","catalog").put("available",true).put("notices",JSONArray())
    private fun catalog(request:JSONObject)=observation(request).put("entries",JSONArray().put(JSONObject().put("id","pair").put("label","Pair").put("description","Two linked names").put("language","python").put("origin","workspace").put("available",true)))
    private fun plan(request:JSONObject,changed:Boolean=false)=observation(request).put("template_id","pair").put("origin","workspace").put("replacement",JSONObject(request.getJSONObject("selection").toString())).put("snippet",JSONObject().put("version",1).put("syntax",if(changed)"\${1:name} + \${1}\$0" else "\${1:name} + \$1\$0").put("text","name + name").put("fields",JSONArray().put(field(0,1,0,4)).put(field(1,1,7,11)).put(field(2,0,11,11))))
    private class Fixture(val workspace:Workspace,val buffer:DocumentBuffer,val model:SourceTemplatesModel,val calls:java.util.concurrent.atomic.AtomicInteger,val mode:java.util.concurrent.atomic.AtomicReference<String>,val entered:java.util.concurrent.CountDownLatch,val release:java.util.concurrent.CountDownLatch)
    private suspend fun host(block:suspend(Fixture)->Unit)=kotlinx.coroutines.coroutineScope {
        val calls=java.util.concurrent.atomic.AtomicInteger();val mode=java.util.concurrent.atomic.AtomicReference("normal");val entered=java.util.concurrent.CountDownLatch(1);val release=java.util.concurrent.CountDownLatch(1)
        val server=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0)
        server.createContext("/api/templates"){exchange->
            val request=JSONObject(exchange.requestBody.bufferedReader().readText());val preparing=exchange.requestURI.path.endsWith("/plan")
            if(preparing){calls.incrementAndGet();if(mode.get()=="hold"){entered.countDown();release.await(5,java.util.concurrent.TimeUnit.SECONDS)}}
            val status=if(preparing&&mode.get()=="revoked")409 else 200
            val value=if(status!=200)JSONObject().put("error","Template catalog changed; review again") else if(preparing)plan(request,mode.get()=="changed") else catalog(request)
            val bytes=value.toString().toByteArray();exchange.sendResponseHeaders(status,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}
        };server.start()
        val workspace=Workspace(this);workspace.engine=Engine("http://127.0.0.1:${server.address.port}")
        val file=FileNode("main.py","main.py",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText("old","saved"));workspace.buffers[file.path]=buffer;workspace.show(file.path);buffer.selectSourceRange(0,3,buffer.text);workspace.showTemplates=true
        val fixture=Fixture(workspace,buffer,workspace.sourceTemplates,calls,mode,entered,release)
        try{block(fixture)}finally{release.countDown();workspace.showTemplates=false;server.stop(0)}
    }
    private suspend fun review(f:Fixture){f.model.refresh(f.workspace);assertNotNull(f.model.notice,f.model.catalog);f.model.prepare(f.workspace,f.model.catalog!!.entries.single());assertNotNull(f.model.notice,f.model.reviewed)}
    @Test fun captureRejectsSameSpellingEditorsAndReturnedTextRevisions(){
        val capture=SourceTemplateCapture(Any(),Any(),Any(),"namespace",7,"main.py","value",0,5,1)
        assertTrue(capture.matches(capture.copy()))
        for(other in listOf(capture.copy(workspace=Any()),capture.copy(engine=Any()),capture.copy(editor=Any()),capture.copy(namespace="next"),capture.copy(revision=9),capture.copy(path="moved.py"),capture.copy(source="other"),capture.copy(anchor=5,head=0),capture.copy(snippetIntent=2)))assertFalse(capture.matches(other))
        assertTrue(runCatching{capture.copy(source="😀",anchor=1,head=1)}.isFailure)
    }
    @Test fun strictCatalogAndPlanBindingsRefuseWholeMalformedReadings(){
        val capture=SourceTemplateCapture(Any(),Any(),Any(),"namespace",0,"main.py","old",0,3,0);val request=capture.body();val raw=catalog(request);val reading=SourceTemplateCatalog.decode(raw,capture)
        for((key,value) in listOf("version" to true,"template_protocol" to 1.5,"source_hash" to "wrong","path" to "other"))assertTrue(runCatching{SourceTemplateCatalog.decode(JSONObject(raw.toString()).put(key,value),capture)}.isFailure)
        val duplicate=JSONObject(raw.toString());duplicate.getJSONArray("entries").put(duplicate.getJSONArray("entries").getJSONObject(0));assertTrue(runCatching{SourceTemplateCatalog.decode(duplicate,capture)}.isFailure)
        val good=plan(request);assertEquals("name + name",SourceTemplatePlan.decode(good,reading,"pair").draft.primary!!.text)
        for(key in listOf("workspace_id","catalog_fingerprint","template_id","origin","source_hash","language"))assertTrue(runCatching{SourceTemplatePlan.decode(JSONObject(good.toString()).put(key,"other"),reading,"pair")}.isFailure)
        assertTrue(runCatching{SourceTemplatePlan.decode(JSONObject(good.toString()).put("replacement",JSONObject().put("from",0).put("to",0)),reading,"pair")}.isFailure)
        assertTrue(runCatching{SourceTemplatePlan.decode(JSONObject(good.toString()).put("command",JSONObject()),reading,"pair")}.isFailure)
        raw.getJSONArray("entries").getJSONObject(0).put("id","mutated");assertEquals("pair",reading.entries.single().id)
    }
    @Test fun realNativeDraftMirrorsNestedScopesAndMenuUndoRedoRetainEditor()=kotlinx.coroutines.runBlocking {host {f->
        val state=f.buffer.state;val scroll=f.buffer.sourceScroll.value;review(f);assertEquals("old",f.buffer.text);assertTrue(f.model.apply(f.workspace));assertEquals("name + name",f.buffer.text);assertEquals(TextRange(0,4),state.selection);assertFalse(f.workspace.showTemplates);assertNotNull(f.buffer.snippets.active)
        f.workspace.showTemplates=true;review(f);assertTrue(f.model.apply(f.workspace));assertEquals("name + name + name + name",f.buffer.text);assertEquals(1,f.buffer.snippets.active!!.scopes.size)
        val selected=state.selection;assertTrue(f.buffer.snippets.applyInput(f.buffer.text,selected.min,selected.max,"x"));assertEquals("x + x + x + x",f.buffer.text)
        val finalSelection=state.selection
        assertTrue(f.buffer.snippets.undoSource());assertEquals("name + name + name + name",f.buffer.text);assertTrue(f.buffer.snippets.undoSource());assertEquals("name + name",f.buffer.text);assertTrue(f.buffer.snippets.undoSource());assertEquals("old",f.buffer.text);assertEquals(TextRange(0,3),state.selection)
        repeat(3){assertTrue(f.buffer.snippets.redoSource())};assertEquals("x + x + x + x",f.buffer.text);assertEquals(finalSelection,state.selection);assertSame(state,f.buffer.state);assertSame(f.buffer,f.workspace.currentBuffer);assertEquals(scroll,f.buffer.sourceScroll.value);assertEquals("old",f.buffer.base)
    }}
    @Test fun sourceEqualUndoCursorAndSameSpellingReplacementRetireReview()=kotlinx.coroutines.runBlocking {
        host{f->review(f);val revision=f.buffer.sourceRevision;assertTrue(f.buffer.applySourceEdits(listOf(NativeSourceEdit(0,3,"changed")),"old"));f.buffer.snippets.undoSource();assertEquals("old",f.buffer.text);f.buffer.selectSourceRange(0,3,"old");assertTrue(f.buffer.sourceRevision>revision);assertFalse(f.model.apply(f.workspace));assertEquals(1,f.calls.get());assertEquals("old",f.buffer.text)}
        host{f->review(f);f.buffer.selectSourceRange(1,1,"old");assertFalse(f.model.apply(f.workspace));assertEquals("old",f.buffer.text)}
        host{f->review(f);val replacement=DocumentBuffer(f.buffer.file,FileText("old","saved"));f.workspace.buffers["main.py"]=replacement;assertFalse(f.model.apply(f.workspace));assertSame(replacement,f.workspace.currentBuffer);assertEquals("old",replacement.text)}
    }
    @Test fun revokedOrChangedCatalogAndDelayedApplyHaveNoLocalEffects()=kotlinx.coroutines.runBlocking {
        for(mode in listOf("revoked","changed"))host{f->review(f);f.mode.set(mode);assertFalse(f.model.apply(f.workspace));assertEquals("old",f.buffer.text);assertEquals(2,f.calls.get());assertNull(f.model.reviewed)}
        host{f->review(f);f.mode.set("hold");val pending=async{f.model.apply(f.workspace)};repeat(100){if(f.entered.count>0)kotlinx.coroutines.delay(10)};assertEquals(0L,f.entered.count);f.buffer.selectSourceRange(2,2,"old");f.release.countDown();assertFalse(pending.await());assertEquals("old",f.buffer.text)}
    }
    @Test fun closedApplyCannotReviveAfterReopening()=kotlinx.coroutines.runBlocking {host{f->
        review(f);f.mode.set("hold");val pending=async{f.model.apply(f.workspace)}
        repeat(100){if(f.entered.count>0)kotlinx.coroutines.delay(10)};assertEquals(0L,f.entered.count)
        f.workspace.showTemplates=false;f.workspace.showTemplates=true;f.release.countDown()
        assertFalse(pending.await());assertEquals("old",f.buffer.text);assertNull(f.model.catalog);assertNull(f.model.reviewed);assertFalse(f.model.busy)
    }}
    @Test(timeout=90000) fun liveNativeSourceTemplateCatalogAndHistory()=kotlinx.coroutines.runBlocking {
        val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN");org.junit.Assume.assumeNotNull(origin)
        val owned=java.io.File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE") ?: error("Use the maintained native template fixture; workspace ownership is required")).canonicalFile
        check(owned.resolve(".hickory-native-template-fixture").readText()=="owned-native-templates-v1\n"){"Refuse catalog mutation outside the owned native fixture"}
        val input=System.getenv("HICKORY_NATIVE_TEMPLATE_INPUT") ?: "main.py";val engine=Engine(origin!!);val (_,tree)=engine.files()
        fun find(path:String,nodes:List<FileNode> = tree):FileNode?{for(node in nodes){if(node.path==path)return node;find(path,node.children)?.let{return it}};return null}
        val file=find(input) ?: error("Template fixture input missing");val saved=engine.file(file);assertEquals("value = 'old'\n",saved.content)
        // /api/files.root is a display name, not a physical root lease. The
        // template workspace ID hashes the canonical root and binds mutation
        // of the marked fixture catalog to this actual engine connection.
        val identity=engine.request("POST","/api/templates/catalog",JSONObject().put("version",1).put("path",input).put("source",saved.content).put("selection",JSONObject().put("from",9).put("to",12)))
        assertEquals(debugHash(owned.path),identity.getString("workspace_id"))
        val workspace=Workspace(this);workspace.engine=engine;workspace.rootFiles=tree;val buffer=DocumentBuffer(file,saved);workspace.buffers[input]=buffer;workspace.show(input);val state=buffer.state
        assertFalse(buffer.readOnly);buffer.selectSourceRange(9,12,buffer.text);val selected=state.selection;val scroll=buffer.sourceScroll.value
        val descriptors=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()})
        fun command(id:String)=workspace.commands(descriptors).single{it.id==id}
        val model=workspace.sourceTemplates;val manifest=owned.resolve(".hickory/templates.json");val original=manifest.readBytes()
        suspend fun opened(){val action=command("source.templates");assertNull(action.reason);action.run();kotlinx.coroutines.yield();repeat(500){if(model.catalog!=null||!model.busy&&model.notice.isNotEmpty())return;kotlinx.coroutines.delay(20)}}
        try {
            opened();val catalog=model.catalog ?: error(model.notice);assertTrue(catalog.entries.single{it.id=="workspace.wrap"}.available);assertFalse(catalog.entries.single{it.id=="workspace.unsupported"}.available)
            model.prepare(workspace,catalog.entries.single{it.id=="workspace.wrap"});val preview=model.reviewed ?: error(model.notice)
            assertEquals("value = 'old + old'\n",preview.draft.plan.patches.single().after);assertEquals(saved.content,buffer.text);assertEquals(saved.content,engine.file(file).content)
            assertTrue(model.notice,model.apply(workspace));assertEquals("value = 'old + old'\n",buffer.text);assertEquals(TextRange(9,12),state.selection)
            assertTrue(buffer.snippets.applyInput(buffer.text,9,12,"new"));assertEquals("value = 'new + new'\n",buffer.text);val afterSelection=state.selection
            repeat(2){val undo=command("editor.undo");assertNull(undo.reason);undo.run()};assertEquals(saved.content,buffer.text);assertEquals(selected,state.selection)
            repeat(2){val redo=command("editor.redo");assertNull(redo.reason);redo.run()};assertEquals("value = 'new + new'\n",buffer.text);assertEquals(afterSelection,state.selection)
            repeat(2){command("editor.undo").run()};assertEquals(saved.content,buffer.text);buffer.snippets.finish();buffer.selectSourceRange(9,12,buffer.text)
            opened();model.prepare(workspace,model.catalog!!.entries.single{it.id=="workspace.wrap"});assertNotNull(model.reviewed)
            val revoked=JSONObject(original.toString(Charsets.UTF_8));revoked.getJSONArray("templates").getJSONObject(0).put("syntax","\${1:revoked}\$0");manifest.writeText(revoked.toString())
            assertFalse(model.apply(workspace));assertEquals(saved.content,buffer.text);assertNull(model.reviewed);manifest.writeBytes(original)
            workspace.showTemplates=false
            val generatedFile=find("generated.py") ?: error("Generated fixture missing");val generated=DocumentBuffer(generatedFile,engine.file(generatedFile));assertTrue(generated.readOnly);workspace.buffers[generatedFile.path]=generated;workspace.show(generatedFile.path)
            val unavailable=command("source.templates");assertNotNull(unavailable.reason);unavailable.run();assertFalse(workspace.showTemplates)
            assertTrue(runCatching{engine.request("POST","/api/templates/catalog",JSONObject().put("version",1).put("path","generated.py").put("source",generated.text).put("selection",JSONObject().put("from",0).put("to",0)))}.isFailure)
            workspace.show(input);assertSame(buffer,workspace.currentBuffer);assertSame(state,buffer.state);assertEquals(selected,state.selection);assertEquals(scroll,buffer.sourceScroll.value);assertEquals(saved.content,buffer.text);assertEquals(saved.content,engine.file(file).content);assertEquals("generated = True\n",engine.file(generatedFile).content)
        }finally{manifest.writeBytes(original);workspace.showTemplates=false}
    }

}
