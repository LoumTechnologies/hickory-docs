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
    private fun observation(request:JSONObject)=JSONObject().put("version",1).put("template_protocol",request.optInt("version",1)).put("variable_context",request.opt("variable_context")?:JSONObject.NULL).put("source_files_modified",false).put("path",request.getString("path")).put("selection",JSONObject(request.getJSONObject("selection").toString())).put("source_hash",debugHash(request.getString("source"))).put("workspace_id","workspace").put("language","python").put("catalog_fingerprint","catalog").put("available",true).put("notices",JSONArray()).apply{if(request.has("reading_context")){put("transform_program_versions",JSONArray().put(2));put("source_context",JSONObject().put("version",1).put("scope","authored-source").put("language","python").put("languages",JSONArray().put("python")).put("targets",JSONArray().put(request.getString("path"))).put("reason",JSONObject.NULL).put("project_entry",request.getJSONObject("reading_context").opt("project_entry")?:JSONObject.NULL).put("source_hashes",JSONObject().put(request.getString("path"),debugHash(request.getString("source")))).put("graph_fingerprint","a".repeat(64)).put("fingerprint","b".repeat(64)))}}
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
            val value=if(status!=200)JSONObject().put("error","Template catalog changed; review again") else if(preparing){
                val reply=plan(request,mode.get()=="changed")
                if(mode.get() in listOf("clipboard","repeated")&&(request.getJSONObject("variable_context").isNull("clipboard")||mode.get()=="repeated")){reply.remove("snippet");reply.put("available",false).put("context_request",JSONObject().put("version",1).put("variables",JSONArray().put("CLIPBOARD")))}else if(mode.get()=="clipboard"&&!request.getJSONObject("variable_context").getJSONObject("clipboard").getBoolean("available")){reply.put("context_unavailable",JSONArray().put("CLIPBOARD"))}
                reply
            }else catalog(request)
            val bytes=value.toString().toByteArray();exchange.sendResponseHeaders(status,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}
        };server.start()
        val workspace=Workspace(this);workspace.engine=Engine("http://127.0.0.1:${server.address.port}")
        val file=FileNode("main.py","main.py",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText("old","saved"));workspace.buffers[file.path]=buffer;workspace.show(file.path);buffer.selectSourceRange(0,3,buffer.text);workspace.showTemplates=true
        val fixture=Fixture(workspace,buffer,workspace.sourceTemplates,calls,mode,entered,release)
        try{block(fixture)}finally{release.countDown();workspace.showTemplates=false;server.stop(0)}
    }
    private suspend fun review(f:Fixture){f.model.refresh(f.workspace);assertNotNull(f.model.notice,f.model.catalog);f.model.prepare(f.workspace,f.model.catalog!!.entries.single());assertNotNull(f.model.notice,f.model.reviewed)}
    private fun batchCapture():SourceTemplateCapture {
        val source="old two";val ranges=JSONArray().put(JSONObject().put("from",0).put("to",3)).put(JSONObject().put("from",4).put("to",7))
        val carets=JSONObject().put("version",1).put("selections",ranges).put("primary",0).put("continuing",false)
        val reading=JSONObject().put("version",1).put("project_entry",JSONObject.NULL).put("sources",JSONObject().put("main.py",source)).put("unaddressed_sources",JSONObject()).put("missing_sources",JSONArray())
        return SourceTemplateCapture(Any(),Any(),Any(),"namespace",0,"main.py",source,0,3,0,SnippetVariableSnapshot(1704067200000,0,"0123456789abcdef0123456789abcdef"),reading.toString(),carets=carets.toString())
    }
    private fun batchCatalog(capture:SourceTemplateCapture):JSONObject {
        val request=capture.body();val value=catalog(request);val context=value.getJSONObject("source_context").put("scope","caret-batch")
        val light=JSONObject(context.toString()).apply{remove("source_hashes")};val ranges=JSONObject(capture.carets!!).getJSONArray("selections")
        return value.put("template_batch_protocol",1).put("carets",JSONObject(capture.carets)).put("readings",JSONArray((0 until ranges.length()).map{JSONObject().put("selection",JSONObject(ranges.getJSONObject(it).toString())).put("context",JSONObject(light.toString()))}))
    }
    private fun batchPlan(capture:SourceTemplateCapture):JSONObject {
        val value=batchCatalog(capture).apply{remove("entries")}.put("template_id","pair").put("origin","workspace")
        val ranges=JSONObject(capture.carets!!).getJSONArray("selections")
        return value.put("expansions",JSONArray((0 until ranges.length()).map{index->
            val range=ranges.getJSONObject(index);val selected=capture.source.substring(range.getInt("from"),range.getInt("to"));val text="$selected:$selected"
            JSONObject().put("selection",JSONObject(range.toString())).put("replacement",JSONObject(range.toString())).put("snippet",JSONObject().put("version",1).put("syntax","\${1:$selected}:\$1\$0").put("text",text).put("fields",JSONArray().put(field(0,1,0,selected.length)).put(field(1,1,selected.length+1,text.length)).put(field(2,0,text.length,text.length))))
        }))
    }
    @Test fun completeBatchPlanPreservesDistinctOriginalSelectionsAndRefusesPartialReplies(){
        val capture=batchCapture();val reading=SourceTemplateCatalog.decode(batchCatalog(capture),capture)
        val value=batchPlan(capture);val plan=SourceTemplatePlan.decode(value,reading,"pair")
        assertEquals("old:old two:two",plan.completeSource);assertEquals("old:old",plan.draft.primary!!.text)
        for(mode in listOf("missing","wrongrange","primary","provisional","language","unknowntransform")){
            val bad=JSONObject(value.toString())
            when(mode){
                "missing"->bad.getJSONArray("expansions").remove(1)
                "wrongrange"->bad.getJSONArray("expansions").getJSONObject(1).getJSONObject("replacement").put("from",0)
                "primary"->bad.getJSONObject("carets").put("primary",1)
                "provisional"->bad.put("snippet",JSONObject())
                "language"->bad.getJSONArray("readings").getJSONObject(1).getJSONObject("context").put("language","javascript")
                else->bad.getJSONArray("expansions").getJSONObject(1).getJSONObject("snippet").getJSONArray("fields").getJSONObject(0).put("transform",JSONObject().put("version",3))
            }
            assertTrue("admitted $mode",runCatching{SourceTemplatePlan.decode(bad,reading,"pair")}.isFailure)
        }
        assertFalse(capture.matches(capture.copy(carets=JSONObject(capture.carets!!).put("primary",1).toString())))
    }
    @Test fun completeBatchPreflightRejectsAggregateOutputAndMetadataBeforeReaderMaterialization(){
        val capture=batchCapture();val reading=SourceTemplateCatalog.decode(batchCatalog(capture),capture)
        val output=batchPlan(capture);for(index in 0..1)output.getJSONArray("expansions").getJSONObject(index).getJSONObject("snippet").put("text","x".repeat(600000))
        assertTrue(runCatching{SourceTemplatePlan.decode(output,reading,"pair")}.isFailure)
        val metadata=batchPlan(capture);for(index in 0..1)metadata.getJSONArray("expansions").getJSONObject(index).getJSONObject("snippet").getJSONArray("fields").getJSONObject(0).put("choices",JSONArray().put("x".repeat(600000)))
        assertTrue(runCatching{SourceTemplatePlan.decode(metadata,reading,"pair")}.isFailure)
    }

    @Test fun captureRejectsSameSpellingEditorsAndReturnedTextRevisions(){
        val capture=SourceTemplateCapture(Any(),Any(),Any(),"namespace",7,"main.py","value",0,5,1)
        assertTrue(capture.matches(capture.copy()))
        for(other in listOf(capture.copy(workspace=Any()),capture.copy(engine=Any()),capture.copy(editor=Any()),capture.copy(namespace="next"),capture.copy(revision=9),capture.copy(path="moved.py"),capture.copy(source="other"),capture.copy(anchor=5,head=0),capture.copy(snippetIntent=2)))assertFalse(capture.matches(other))
        assertTrue(runCatching{capture.copy(source="😀",anchor=1,head=1)}.isFailure)
    }
    @Test fun embeddedBindingRejectsForeignLanguageOriginAndMetadata(){
        val reading=JSONObject().put("version",1).put("project_entry",JSONObject.NULL).put("sources",JSONObject().put("main.py","old")).put("unaddressed_sources",JSONObject()).put("missing_sources",JSONArray())
        val capture=SourceTemplateCapture(Any(),Any(),Any(),"namespace",0,"main.py","old",0,3,0,SnippetVariableSnapshot.capture(),reading.toString());val raw=catalog(capture.body());val value=SourceTemplateCatalog.decode(raw,capture);assertNotNull(value.sourceContext)
        for(change in listOf("language","origin","extra")){val bad=JSONObject(raw.toString());val context=bad.getJSONObject("source_context");when(change){"language"->context.put("language","javascript");"origin"->context.getJSONObject("source_hashes").put("main.py","a".repeat(64));else->context.put("extra",true)};assertTrue(runCatching{SourceTemplateCatalog.decode(bad,capture)}.isFailure)}
        val changed=plan(capture.body());changed.getJSONObject("source_context").put("fingerprint","c".repeat(64));assertTrue(runCatching{SourceTemplatePlan.decode(changed,value,"pair")}.isFailure)
    }
    @Test fun readonlyContributorABAChangeRetiresReviewedTemplate()=kotlinx.coroutines.runBlocking{host{f->
        val file=FileNode("context.md","context.md",false,emptyList(),null,"fixture.md");val contributor=DocumentBuffer(file,FileText("context","saved"));f.workspace.buffers[file.path]=contributor
        review(f);contributor.state.edit{replace(0,length,"changed")};contributor.advanceSourceRevision();contributor.state.edit{replace(0,length,"context")};contributor.advanceSourceRevision();val before=f.calls.get();assertFalse(f.model.apply(f.workspace));assertEquals(before,f.calls.get());assertEquals("old",f.buffer.text);assertTrue(contributor.readOnly)
    }}
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
    @Test fun variableContextRequiresExactStructureAndTypedNoEffects(){
        val snapshot=SnippetVariableSnapshot(1704067200000,0,"0123456789abcdef0123456789abcdef")
        val capture=SourceTemplateCapture(Any(),Any(),Any(),"namespace",0,"main.py","old",0,3,0,snapshot)
        val raw=catalog(capture.body());val reading=SourceTemplateCatalog.decode(raw,capture);val good=plan(capture.body());assertEquals("name + name",SourceTemplatePlan.decode(good,reading,"pair").draft.primary!!.text)
        for(key in listOf("command","edit","workspaceEdit","additionalTextEdits")){assertTrue(runCatching{SourceTemplateCatalog.decode(JSONObject(raw.toString()).put(key,JSONObject.NULL),capture)}.isFailure);assertTrue(runCatching{SourceTemplatePlan.decode(JSONObject(good.toString()).put(key,JSONObject.NULL),reading,"pair")}.isFailure)}
        for(value in listOf(0,"false",true)){assertTrue(runCatching{SourceTemplateCatalog.decode(JSONObject(raw.toString()).put("source_files_modified",value),capture)}.isFailure)}
        val extra=JSONObject(raw.toString());extra.getJSONObject("variable_context").put("extra",true);assertTrue(runCatching{SourceTemplateCatalog.decode(extra,capture)}.isFailure)
        val numeric=JSONObject(raw.toString());numeric.getJSONObject("variable_context").getJSONObject("clock").put("utc_offset_minutes",0.0);assertTrue(runCatching{SourceTemplateCatalog.decode(numeric,capture)}.isFailure)
    }
    @Test fun demandedClipboardIsCapturedOnceDeniedExplicitlyAndCannotOutliveSourceLease()=kotlinx.coroutines.runBlocking {
        for(mode in listOf("available","denied","stale","repeated"))host{f->
            var reads=0;f.mode.set(if(mode=="repeated")"repeated" else "clipboard")
            f.workspace.engine!!.snippetClipboardReader={reads++;if(mode=="stale")f.buffer.selectSourceRange(0,0,f.buffer.text);if(mode=="denied")null else "copied"}
            f.model.refresh(f.workspace);f.model.prepare(f.workspace,f.model.catalog!!.entries.single());assertEquals(1,reads)
            if(mode=="stale"||mode=="repeated"){assertNull(f.model.reviewed);assertEquals("old",f.buffer.text)}else{
                assertNotNull(f.model.notice,f.model.reviewed);val captured=f.model.reviewed!!.plan.catalog.capture.variables!!;assertEquals(mode!="denied",captured.clipboard!!.available);assertEquals(if(mode=="denied")null else "copied",captured.clipboard!!.text)
                assertTrue(f.model.apply(f.workspace));assertEquals(1,reads);assertEquals(3,f.calls.get());assertEquals("old",f.buffer.base)
            }
        }
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
            model.captureVariables={SnippetVariableSnapshot(1704067200000,0,"0123456789abcdef0123456789abcdef")};var clipboardReads=0;engine.snippetClipboardReader={clipboardReads++;"copied"}
            opened();model.prepare(workspace,model.catalog!!.entries.single{it.id=="workspace.context"});assertNotNull(model.notice,model.reviewed);assertEquals("main.py|2024|012345|old|old|OLD",model.reviewed!!.plan.draft.primary!!.text);assertEquals(0,clipboardReads)
            assertTrue(model.notice,model.apply(workspace));assertEquals("value = 'main.py|2024|012345|old|old|OLD'\n",buffer.text)
            assertTrue(buffer.snippets.applyInput(buffer.text,state.selection.min,state.selection.max,"new"));assertEquals("value = 'main.py|2024|012345|new|new|NEW'\n",buffer.text)
            repeat(2){command("editor.undo").run()};assertEquals(saved.content,buffer.text);repeat(2){command("editor.redo").run()};assertEquals("value = 'main.py|2024|012345|new|new|NEW'\n",buffer.text);repeat(2){command("editor.undo").run()};buffer.snippets.finish()
            for(denied in listOf(false,true)){
                buffer.selectSourceRange(9,12,buffer.text);val before=clipboardReads;engine.snippetClipboardReader={clipboardReads++;if(denied)null else "copied"}
                opened();assertEquals(before,clipboardReads);model.prepare(workspace,model.catalog!!.entries.single{it.id=="workspace.clipboard"});assertNotNull(model.notice,model.reviewed);assertEquals(before+1,clipboardReads)
                val expected=if(denied)"fallback" else "copied";assertEquals(expected,model.reviewed!!.plan.draft.primary!!.text);assertEquals(if(denied)listOf("CLIPBOARD") else emptyList<String>(),model.reviewed!!.plan.unavailable)
                assertTrue(model.notice,model.apply(workspace));assertEquals(before+1,clipboardReads);assertEquals("value = '$expected'\n",buffer.text);command("editor.undo").run();assertEquals(saved.content,buffer.text);command("editor.redo").run();assertEquals("value = '$expected'\n",buffer.text);command("editor.undo").run();buffer.snippets.finish()
            }
            buffer.selectSourceRange(9,12,buffer.text);val beforeUnused=clipboardReads;opened();model.prepare(workspace,model.catalog!!.entries.single{it.id=="workspace.unused-clipboard"});assertNotNull(model.notice,model.reviewed);assertEquals(0,clipboardReads-beforeUnused);assertEquals("old",model.reviewed!!.plan.draft.primary!!.text);workspace.showTemplates=false
            engine.snippetClipboardReader={clipboardReads++;buffer.selectSourceRange(0,0,buffer.text);"stale"};buffer.selectSourceRange(9,12,buffer.text);opened();model.prepare(workspace,model.catalog!!.entries.single{it.id=="workspace.clipboard"});assertNull(model.reviewed);assertEquals(saved.content,buffer.text);workspace.showTemplates=false;buffer.selectSourceRange(9,12,buffer.text)
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
