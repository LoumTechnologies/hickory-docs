<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Review a reusable source template in the Android editor

For maintainers connecting engine-owned template catalogs to the existing floating editor actions. This reading owns catalog policy, the host request/review lifetime and the floating source-template adapter. The full source compiled and the final focused checks plus the maintained actual-engine host flow passed on 2026-11-16. The evidence below distinguishes native controller acceptance from physical Compose input and geometry.

The catalog is a reading, not write authority. Capture the actual workspace, engine, retained editor, resource namespace, text revision, source and full selection. A changed editor, cursor, IME composition, writable state or source revision retires both catalog selection and reviewed application. Closing a floating reading retires its request permanently; reopening cannot revive it. A plan binds the selected opaque catalog identity and catalog fingerprint to this same capture.

Capture local time and random seed once when this reading opens, and send protocol 2 with that immutable variable context. Clipboard is initially unattempted: a verified engine request may demand it once, then the same source lease and clock/seed travel with either its captured text or explicit denial. Catalog identity does not change merely because clipboard was attempted. Apply repeats the reviewed context without reading the clipboard again. Reject additional effects and malformed or substituted context before preparing any draft; the existing editor and source revision checks remain authoritative after every asynchronous response.

<ui:copy id="source-templates-android-policy">
package com.hickorydocs.simple

import org.json.JSONObject
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.verticalScroll
import androidx.compose.ui.unit.dp

private fun templateNumber(value:Any?):Int = (value as? Number)?.toString()?.toIntOrNull()?.takeIf{it>=0} ?: error("Malformed template coordinate or protocol version")
private fun templateText(value:JSONObject,key:String)=value.opt(key) as? String ?: error("Missing template $key")
private fun templateStrings(value:JSONObject,key:String):List<String> {val array=value.optJSONArray(key) ?: error("Missing template $key");return (0 until array.length()).map{array.opt(it) as? String ?: error("Malformed template $key")}}

data class SourceTemplateCapture(
    val workspace:Any,val engine:Any,val editor:Any,val namespace:Any,val revision:Long,
    val path:String,val source:String,val anchor:Int,val head:Int,val snippetIntent:Int,
    val variables:SnippetVariableSnapshot?=null,
) {
    init {
        check(anchor in 0..source.length&&head in 0..source.length){"Template selection is outside the source."}
        sourceChangeOffset(source,languagePosition(source,anchor));sourceChangeOffset(source,languagePosition(source,head))
    }
    val from get()=minOf(anchor,head)
    val to get()=maxOf(anchor,head)
    val protocol get()=if(variables==null)1 else 2
    fun body()=JSONObject().put("version",protocol).put("path",path).put("source",source).put("selection",JSONObject().put("from",from).put("to",to)).apply{variables?.let{put("variable_context",it.body())}}
    fun matches(now:SourceTemplateCapture)=workspace===now.workspace&&engine===now.engine&&editor===now.editor&&namespace==now.namespace&&revision==now.revision&&path==now.path&&source==now.source&&anchor==now.anchor&&head==now.head&&snippetIntent==now.snippetIntent
}
private fun templateJSONMatches(value:Any?,expected:Any?):Boolean {
 if(value==null||value===JSONObject.NULL)return expected==null||expected===JSONObject.NULL
 if(expected==null||expected===JSONObject.NULL)return false
 if(value is JSONObject&&expected is JSONObject){val keys=value.keys().asSequence().toSet();return keys==expected.keys().asSequence().toSet()&&keys.all{templateJSONMatches(value.opt(it),expected.opt(it))}}
 if(value is org.json.JSONArray&&expected is org.json.JSONArray)return value.length()==expected.length()&&(0 until value.length()).all{templateJSONMatches(value.opt(it),expected.opt(it))}
 if(value is Number||expected is Number)return value is Number&&expected is Number&&value.toString()==expected.toString()
 return value==expected
}
private fun templateSnapshotMatches(value:JSONObject?,expected:SnippetVariableSnapshot):Boolean {
 if(value==null||!templateJSONMatches(value,expected.body()))return false
 val clock=value.optJSONObject("clock")
 if(expected.unixMillis!=null&&expected.utcOffsetMinutes!=null){if(clock==null||(clock.opt("unix_millis") as? Number)?.toString()!=expected.unixMillis.toString()||(clock.opt("utc_offset_minutes") as? Number)?.toString()!=expected.utcOffsetMinutes.toString())return false}
 val clipboard=value.optJSONObject("clipboard")
 if(expected.clipboard!=null&&clipboard?.opt("available") !is Boolean)return false
 return true
}
private fun templateVariableIdentity(value:JSONObject,capture:SourceTemplateCapture){
 check(listOf("command","additionalTextEdits","workspaceEdit","edit").none{value.has(it)}){"Template contains unsupported additional effects. No edits were applied."}
 check(templateNumber(value.opt("version"))==1&&templateNumber(value.opt("template_protocol"))==capture.protocol){"This engine does not support the captured template protocol."}
 if(capture.protocol==2)check(value.opt("source_files_modified")==false&&templateSnapshotMatches(value.optJSONObject("variable_context"),capture.variables!!)){"Template variable context changed. Refresh before applying."}else check(value.opt("source_files_modified")!=true&&value.isNull("variable_context")){"Legacy template response carries unsupported context/effects"}
}
data class SourceTemplateEntry(val id:String,val label:String,val description:String,val language:String,val origin:String,val available:Boolean,val reason:String?)
class SourceTemplateCatalog private constructor(
    val capture:SourceTemplateCapture,val workspaceID:String,val language:String,val fingerprint:String,
    val available:Boolean,val reason:String?,val notices:List<String>,val entries:List<SourceTemplateEntry>,
) {
    fun withCapture(next:SourceTemplateCapture):SourceTemplateCatalog {check(capture.matches(next)){"Template source capture changed"};return SourceTemplateCatalog(next,workspaceID,language,fingerprint,available,reason,notices,entries)}
    companion object {
        fun decode(value:JSONObject,capture:SourceTemplateCapture):SourceTemplateCatalog {
            check(!value.has("context_request")){"Unexpected template catalog context request"}
            templateVariableIdentity(value,capture)
            check(templateText(value,"path")==capture.path&&templateText(value,"source_hash")==debugHash(capture.source)){"Template source changed or reply belongs to another source."}
            val selection=value.optJSONObject("selection") ?: error("Missing template selection")
            check(templateNumber(selection.opt("from"))==capture.from&&templateNumber(selection.opt("to"))==capture.to){"Template cursor changed."}
            val available=value.opt("available") as? Boolean ?: error("Malformed template availability")
            val workspace=templateText(value,"workspace_id");val language=templateText(value,"language");val fingerprint=templateText(value,"catalog_fingerprint")
            check(workspace.isNotEmpty()&&fingerprint.isNotEmpty()){ "Template catalog identity is missing." }
            val array=value.optJSONArray("entries") ?: error("Missing template entries");check(array.length()<=4096){"Template catalog exceeds the supported reading."}
            val entries=(0 until array.length()).map{index->val row=array.optJSONObject(index) ?: error("Malformed template entry")
                val origin=templateText(row,"origin");check(origin in listOf("builtin","workspace")){"Unsupported template origin"}
                SourceTemplateEntry(templateText(row,"id"),templateText(row,"label"),templateText(row,"description"),templateText(row,"language"),origin,row.opt("available") as? Boolean ?: error("Malformed template availability"),row.opt("reason") as? String)
            }
            check(entries.map{it.id}.toSet().size==entries.size&&entries.all{it.id.isNotEmpty()}){"Ambiguous template identities"}
            return SourceTemplateCatalog(capture,workspace,language,fingerprint,available,value.opt("reason") as? String,java.util.Collections.unmodifiableList(templateStrings(value,"notices")),java.util.Collections.unmodifiableList(entries))
        }
    }
}
</ui:copy>

An admitted plan never writes directly. Validate its exact replacement interval and engine snippet metadata, then reuse completion planning and SourceSnippetController nesting. Apply is one explicit draft action through the same source edit and native undo machinery. A command, import or broader workspace edit is not an alternative execution path. Template variable values belong to the engine's explicit context; unavailable contexts and unsupported transforms must refuse the whole plan.

<ui:copy id="source-templates-android-plan">
private fun templatePlanIdentity(value:JSONObject,catalog:SourceTemplateCatalog,id:String):SourceTemplateEntry{
            val entry=catalog.entries.singleOrNull{it.id==id} ?: error("Template entry no longer exists")
            check(catalog.available&&entry.available){entry.reason ?: "Template unavailable for this source."}
            val context=catalog.capture
            templateVariableIdentity(value,context)
            val expected=mapOf("path" to context.path,"source_hash" to debugHash(context.source),"workspace_id" to catalog.workspaceID,"language" to catalog.language,"catalog_fingerprint" to catalog.fingerprint,"template_id" to entry.id,"origin" to entry.origin)
            check(expected.all{(key,text)->templateText(value,key)==text}){"The template plan does not match the captured catalog and source."}
            for(key in listOf("selection","replacement")){val range=value.optJSONObject(key) ?: error("Missing template $key");check(templateNumber(range.opt("from"))==context.from&&templateNumber(range.opt("to"))==context.to){"Template replacement must equal the captured selection."}}
            check(listOf("command","additionalTextEdits","workspaceEdit","edit").none{value.has(it)}){"Template plan contains unsupported effects; no partial application is allowed."}
 return entry
}
class SourceTemplatePlan private constructor(val catalog:SourceTemplateCatalog,val entry:SourceTemplateEntry,val syntax:String,val draft:CompletionDraft,val notices:List<String>,val unavailable:List<String>) {
    companion object {
        fun decode(value:JSONObject,catalog:SourceTemplateCatalog,id:String):SourceTemplatePlan {
            check(!value.has("context_request")){"Unexpected template plan context request"}
            val entry=templatePlanIdentity(value,catalog,id);val context=catalog.capture;check(value.opt("available")==true){"Template is unavailable for this captured context."}
            val snippet=value.optJSONObject("snippet") ?: error("Missing template snippet")
            val syntax=templateText(snippet,"syntax")
            val item=JSONObject().put("label",entry.label).put("insertText",syntax).put("insertTextFormat",2).put("data",JSONObject().put("hickory_snippet",snippet))
            val expansion=completionExpansion(item) ?: error("Template expansion unavailable")
            val draft=planCompletion(languageURI(context.path),item,SourceSnapshot(context.source,true),context.from,context.to,expansion)
            val unavailable=if(value.has("context_unavailable"))templateStrings(value,"context_unavailable") else emptyList()
            check(unavailable.size<=40&&unavailable.all{it.length<=64}){"Malformed template context availability"}
            return SourceTemplatePlan(catalog,entry,syntax,draft,if(value.isNull("notices")&&context.protocol==1)emptyList() else java.util.Collections.unmodifiableList(templateStrings(value,"notices")),java.util.Collections.unmodifiableList(unavailable))
        }
    }
}
</ui:copy>

## Pin the host revision before attaching UI

The capture policy is independently testable. All six focused methods passed with zero skips on 2026-11-16. They cover immutable identity, strict observations, nested native history, source-equal edit/undo, same-spelling replacements, provider revocation/change and held Apply replies after movement or dismissal. These pure-policy and fake-provider checks are separate from actual service acceptance.

The protocol 2 witnesses below extend the earlier executed protocol 1 evidence. They require exact variable-context structure and typed false effects, test one demanded clipboard capture including explicit denial, refuse repeated or stale requests, and retain the same snapshot through Apply. The actual-engine case also exercises captured filename/time/random values, derived uppercase mirrors, native Undo/Redo, unused clipboard defaults and source-lease retirement. On 2026-11-17 complete Kotlin source/test compilation and 13 focused checks passed: these 8 template checks, three shared context-handshake checks and two connected completion-plan checks. The exact real-engine template case then passed with zero skips through `python3 scripts/test-native-templates.py android --engine target/source-edit-focus/debug/examples/engine_client`. Its distinct provider-completion context case also passed through `--provider-context`; the [native provider reading](native-snippet-context-provider-witness.md) records that scope. Durable native receipts and logs live under `target/native-template-smoke/`. Physical pointer, IME, operating-system clipboard prompts and accessibility acceptance remain unverified; these counts do not establish iOS runtime acceptance.

<ui:copy id="source-templates-android-witness">
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
    private fun observation(request:JSONObject)=JSONObject().put("version",1).put("template_protocol",request.optInt("version",1)).put("variable_context",request.opt("variable_context")?:JSONObject.NULL).put("source_files_modified",false).put("path",request.getString("path")).put("selection",JSONObject(request.getJSONObject("selection").toString())).put("source_hash",debugHash(request.getString("source"))).put("workspace_id","workspace").put("language","python").put("catalog_fingerprint","catalog").put("available",true).put("notices",JSONArray())
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
</ui:copy>

The host adapter keeps transport separate from policy. It captures one retained editor and invokes the engine's two read-only endpoints. Each request has a current ticket; the captured native revision, selection and snippet intent must still match after the await. A reviewed draft includes nested mirrors before it is shown, so Apply does not discover additional effects after review.

<ui:copy id="source-templates-android-host">
class SourceTemplatesModel {
    var captureVariables:()->SnippetVariableSnapshot={SnippetVariableSnapshot.capture()}
    var catalog by androidx.compose.runtime.mutableStateOf<SourceTemplateCatalog?>(null);private set
    data class Review(val plan:SourceTemplatePlan,val draft:CompletionDraft)
    var reviewed by androidx.compose.runtime.mutableStateOf<Review?>(null);private set
    var busy by androidx.compose.runtime.mutableStateOf(false);private set
    var notice by androidx.compose.runtime.mutableStateOf("");private set
    private var ticket=java.util.UUID.randomUUID()
    fun close(){ticket=java.util.UUID.randomUUID();busy=false;reviewed=null;catalog=null}
    private fun capture(workspace:Workspace,variables:SnippetVariableSnapshot?=null):SourceTemplateCapture {
        val engine=workspace.engine ?: error("Connect to this workspace before reading templates.")
        val buffer=workspace.currentBuffer ?: error("Open writable source before reading templates.")
        check(!buffer.readOnly&&!buffer.saving&&!buffer.resourceDeleted&&buffer.state.composition==null){"Finish composition and choose writable source that is not saving or deleted."}
        check(!(buffer.multiSource==buffer.text&&buffer.multiRanges.size>1)){"Choose one source selection before inserting a template."}
        val selected=buffer.state.selection
        return SourceTemplateCapture(workspace,engine,buffer,workspace.sourceNavigationEpoch,buffer.sourceRevision,buffer.file.path,buffer.text,selected.start,selected.end,buffer.snippets.intent,variables)
    }
    fun current(workspace:Workspace,context:SourceTemplateCapture)=workspace.sourceTemplates===this&&workspace.showTemplates&&runCatching{context.matches(capture(workspace))}.getOrDefault(false)
    suspend fun refresh(workspace:Workspace){
        close();val id=ticket;busy=true;notice="Reading source templates…"
        try {
            val context=capture(workspace,captureVariables());val engine=context.engine as Engine
            val reply=engine.request("POST","/api/templates/catalog",context.body())
            if(ticket!=id)return
            check(current(workspace,context)){"Source or selection changed while reading templates. Refresh the catalog."}
            catalog=SourceTemplateCatalog.decode(reply,context);notice=if(catalog!!.available)"Choose a template to review its complete draft." else catalog!!.reason ?: "Templates are unavailable for this source."
        }catch(cancelled:kotlinx.coroutines.CancellationException){throw cancelled}
        catch(error:Exception){if(ticket==id)notice=error.message.orEmpty()}
        finally{if(ticket==id)busy=false}
    }
    suspend fun prepare(workspace:Workspace,entry:SourceTemplateEntry){
        val catalog=catalog ?: return
        val id=java.util.UUID.randomUUID();ticket=id;reviewed=null;busy=true;notice="Preparing template preview…"
        try {
            check(current(workspace,catalog.capture)){"Source context changed. Refresh templates."}
            check(catalog.entries.any{it===entry}&&entry.available){entry.reason ?: "Template unavailable."}
            var reading=catalog
            for(attempt in 0 until 2){
                if(ticket!=id)return
                val capture=reading.capture;val request=capture.body().put("catalog_fingerprint",reading.fingerprint).put("template_id",entry.id);val reply=(capture.engine as Engine).request("POST","/api/templates/plan",request)
                if(ticket!=id)return;check(current(workspace,capture)){"Source, selection or snippet fields changed while preparing the template. Review again."}
                templatePlanIdentity(reply,reading,entry.id)
                if(reply.has("context_request")){val required=reply.optJSONObject("context_request");val names=required?.optJSONArray("variables");check(attempt==0&&capture.variables!=null&&capture.variables.clipboard==null&&reply.opt("available")==false&&required!=null&&required.length()==2&&(required.opt("version") as? Number)?.toString()=="1"&&names?.length()==1&&names.opt(0)=="CLIPBOARD"){"Unsupported or repeated template context request"}
                    val clipboard=captureSnippetClipboard((capture.engine as Engine).snippetClipboardReader);if(ticket!=id)return;check(current(workspace,capture)){"Source or selection changed while reading clipboard context."};reading=reading.withCapture(capture.copy(variables=capture.variables.copy(clipboard=clipboard)));this.catalog=reading;continue
                }
                val plan=SourceTemplatePlan.decode(reply,reading,entry.id);val draft=(capture.editor as DocumentBuffer).snippets.prepareCompletion(plan.draft);reviewed=Review(plan,draft);notice="Review the complete draft. Apply does not save or run a command."+plan.notices.filter{it !in reading.notices}.joinToString("\n",prefix=if(plan.notices.any{it !in reading.notices})"\n" else "");return
            }
            error("Template context is unavailable. No edits were applied.")
        }catch(cancelled:kotlinx.coroutines.CancellationException){throw cancelled}
        catch(error:Exception){if(ticket==id)notice=error.message.orEmpty()}
        finally{if(ticket==id)busy=false}
    }
    @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
    suspend fun apply(workspace:Workspace):Boolean {
        if(busy)return false
        val review=reviewed ?: return false
        val id=java.util.UUID.randomUUID();reviewed=null;ticket=id;busy=true
        try {
            check(current(workspace,review.plan.catalog.capture)){"Source, selection or editor changed. Review a fresh template before applying."}
            val capture=review.plan.catalog.capture
            val request=capture.body().put("catalog_fingerprint",review.plan.catalog.fingerprint).put("template_id",review.plan.entry.id)
            val reply=(capture.engine as Engine).request("POST","/api/templates/plan",request)
            if(ticket!=id)return false
            val fresh=SourceTemplatePlan.decode(reply,review.plan.catalog,review.plan.entry.id)
            check(fresh.syntax==review.plan.syntax&&fresh.draft==review.plan.draft&&fresh.notices==review.plan.notices&&fresh.unavailable==review.plan.unavailable){"Template changed after preview. Review a fresh plan before applying."}
            check(current(workspace,capture)){"Source, selection, composition or editor changed while verifying the reviewed template. No edits were applied."}
            val buffer=capture.editor as DocumentBuffer;val draft=review.draft
            val before=buffer.snippets.captureBefore(draft.before,buffer.state.selection,true)!!;val serial=buffer.snippets.recordSerial
            check(draft.plan.patches.size<=1&&draft.plan.patches.all{it.path==review.plan.catalog.capture.path}){"Template preview contains unsupported additional source effects."}
            val edits=draft.plan.patches.singleOrNull()?.edits.orEmpty().map{NativeSourceEdit(it.from,it.to,it.text)}
            check(buffer.applySourceEdits(edits,draft.before,androidx.compose.ui.text.TextRange(draft.selection.from,draft.selection.to))){"Editor refused the complete template. No replacement request was sent; review again."}
            buffer.snippets.attachCompletionContext(before,draft.continuing ?: beginSnippet(buffer.text,draft.fields),serial)
            notice="Template applied · draft. Existing source Undo/Redo retains template fields.";catalog=null;workspace.showTemplates=false;buffer.focusSource?.invoke();return true
        }catch(cancelled:kotlinx.coroutines.CancellationException){throw cancelled}
        catch(error:Exception){if(ticket==id)notice=error.message.orEmpty();return false}
        finally{if(ticket==id)busy=false}
    }
}
</ui:copy>

<ui:copy id="source-templates-android-view">
@androidx.compose.runtime.Composable fun SourceTemplatesView(workspace:Workspace) {
    val model=workspace.sourceTemplates
    androidx.compose.foundation.layout.Column(androidx.compose.ui.Modifier.widthIn(min=280.dp,max=760.dp).heightIn(max=720.dp).verticalScroll(androidx.compose.foundation.rememberScrollState()).padding(12.dp)) {
        androidx.compose.material3.Text("Source templates")
        androidx.compose.material3.TextButton(onClick={workspace.showTemplates=false;workspace.currentBuffer?.focusSource?.invoke()}){androidx.compose.material3.Text("Close")}
        androidx.compose.material3.TextButton(enabled=!model.busy,onClick={workspace.runCommand{model.refresh(workspace)}}){androidx.compose.material3.Text("Refresh templates")}
        androidx.compose.material3.Text(model.notice)
        model.catalog?.let {catalog->
            catalog.notices.forEach{androidx.compose.material3.Text(it)}
            catalog.entries.forEach{entry->
                androidx.compose.material3.TextButton(enabled=catalog.available&&entry.available&&!model.busy&&model.current(workspace,catalog.capture),onClick={workspace.runCommand{model.prepare(workspace,entry)}}){androidx.compose.material3.Text("${entry.label} · ${entry.origin}")}
                androidx.compose.material3.Text(entry.description)
                entry.reason?.let{androidx.compose.material3.Text(it)}
            }
        }
        model.reviewed?.let{review->
            androidx.compose.material3.Text("Reviewed complete draft · ${review.plan.entry.label}")
            androidx.compose.foundation.text.selection.SelectionContainer{androidx.compose.material3.Text(review.draft.plan.patches.singleOrNull()?.after ?: review.draft.before)}
            androidx.compose.material3.TextButton(enabled=!model.busy&&model.current(workspace,review.plan.catalog.capture),onClick={workspace.runCommand{model.apply(workspace)}}){androidx.compose.material3.Text("Apply template to draft")}
        }
    }
}
</ui:copy>

## Planned host adapter acceptance

The host captures a synchronous source-intent revision from the retained buffer. Owned native input, explicit source edits, selection operations, menu undo/redo and resource adoption advance it without replacing the native TextFieldState. A source-equal edit/undo roundtrip therefore cannot revive an earlier review. Installed Compose text exposes a backing CharSequence, so its object identity is not treated as revision authority. During the existing input transaction the core counter advances immediately; publication to Compose observers is deferred until after that transaction, matching the snippet controller pattern. Provider replies read the current counter even while display publication is pending. Resource publication also retires the existing workspace namespace token.

The floating reading exposes Refresh, catalog entries with truthful unavailable reasons, a selectable generated-text preview and Apply. Its request ticket and current capture check guard both catalog and plan responses. Apply consumes the reviewed proposal, re-requests the engine plan with the captured fingerprint, and requires identical syntax and complete initial draft/field metadata. A revoked or changed catalog refuses before any local write. After that await, it checks single-caret/writable/not-saving/not-deleted/no-IME state, namespace, editor identity, revision and selection. It prepares nested mirrors with SourceSnippetController.prepareCompletion before review, then invokes existing applySourceEdits and attachCompletionContext; no LSP provider is required.

Focused acceptance must include changed source/cursor/namespace, identical-byte replacement editors, edit/undo roundtrip, delayed/closed/replaced replies, catalog or template changes at the service boundary, unsupported origins/providers/transforms and whole-proposal refusal. Real native text tests must exercise insertion, nested mirror editing and exact menu undo/redo while retaining editor selection/scroll/identity and unchanged saved bytes. Physical pointer/IME/accessibility acceptance remains separate.

## Focused acceptance evidence

The complete Android production and test Kotlin sources compiled. The final six focused `SourceTemplateTest` methods executed with zero skips (0.277 seconds). The eight existing `SourceSnippetInteractionTest` methods also passed with zero skips (0.186 seconds), checking that the revision seam preserves production input transformations, smart Enter, imports, nested completion, choices and native undo/redo.

The maintained `python3 scripts/test-native-templates.py android --engine target/source-edit-focus/debug/examples/engine_client` then passed exactly one executed `liveNativeSourceTemplateCatalogAndHistory` method with zero skips (0.294 seconds). Startup verified the actual protocol, selected-text expansion, supported/unavailable entries and independent writable/generated ownership. The native case used the shared command descriptor and adapter, actual catalog and plan requests, unchanged-source preview, fresh-plan revalidation, linked selected-text insertion and typing, ordinary menu undo/redo, real catalog revocation and generated-source refusal. It retained the same buffer/TextFieldState, selection and scroll and verified unchanged saved source bytes. The fixture marker and canonical workspace fingerprint bound test-only catalog mutation to the owned engine workspace; the file-tree root label is not treated as a physical path. The runner cleaned its process groups and temporary workspace and retained complete native output at `target/native-template-smoke/android-last.log`.

A subsequent safety review closed empty-alternative amplification at expansion, active-session and history boundaries. Every alternative, including an empty one, occupies a charged slot. The expansion/session quota admits at most 4096 choices per field and totals `64 + Σ(32 + UTF-8 choice bytes)` per field within 1 MiB. Central session admission checks before copying choices, and nested insertion reserves the entire outer-mirror fanout before cloning its inner vector. The counted-read rejection case verifies that an eight-mirror proposal with 4096 empty inner alternatives never reaches the clone.

Native history separately charges field and scope metadata plus `32 + UTF-16 choice bytes` per alternative. Its existing bounded journal saturates at budget+1 and rejects an oversized record/replacement before retaining it or clearing unrelated prior metadata. Native text undo remains authoritative; unavailable replay context still discards fields rather than fabricating a session. Template plan validation no longer deep-clones raw snippet JSON ahead of quota admission.

After the coordinated safety snapshot, full Kotlin compilation and 24 focused JUnit methods passed with zero skips: expansion 3 (including 28 shared scenarios), session 2 (including 37 shared scenarios and the preclone fanout check), native undo contexts 5, source interactions 8 and template lifetime/host checks 6. The maintained actual Android template runner then passed its one normal engine-backed method again with zero skips (0.285 seconds), against the fresh bounded engine. These checks supersede the earlier normal-path-only evidence for the affected quota and history boundaries; no broad regression suite was run.

This is native host/controller and TextFieldState acceptance. A rendered Compose window, measured editor geometry, physical pointer/popover input, device keyboard/IME and accessibility have not been exercised. The catalog retains the shared snippet mechanism's explicit transform/context and multiple-caret limitations; it does not establish semantic code generation or complete snippet syntax.

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SourceTemplates.kt"><ui:paste select="#source-templates-android-policy"/><ui:paste select="#source-templates-android-plan"/><ui:paste select="#source-templates-android-host"/><ui:paste select="#source-templates-android-view"/></ui:file>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SourceTemplateTest.kt"><ui:paste select="#source-templates-android-witness"/></ui:file>
</ui:doc>
