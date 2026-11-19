<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read runner results and coverage on android

For maintainers changing workspace services independently of the editor renderer.

The existing menu opens a floating, replaceable tool surface. The editor remains mounted and retains its own selection, scroll and undo state. A native local-only web renderer delegates only fixed coverage operations to the connected engine; workspace replacement retires its replies. Source navigation opens unbound files without positional claims, and accepts a reported line only after matching its captured hash against available text. Selecting a report retains its record identity; source-line mark binding is a separate capability.

Selected records are read back by identity through the engine. A latest-selection ticket rejects obsolete replies, projection preserves mixed consequences and missing quantities, and the independent digest service binds only matching available text. Source edits retire marks synchronously at lookup; background hashing cannot rewrite the buffer. Both gutter and menu borrow this retained state.

The fixed native message controller is independent of WebView. Its attached/reply/display callbacks let the exact production protocol run against an owned engine without a fake Android renderer; WebView only serializes local messages. Closing the surface retires its controller immediately, and every reply still checks the original engine and coverage identity.

A selected report retains the original engine, and navigation carries the original renderer lifetime through source loading, hashing and reveal. The editor object and edit intent must remain current even when edited text returns to equal bytes. Malformed coordinates refuse instead of becoming an unbound position, and only an accepted source reveal closes the report.

Choose the latest source link before starting its file read. A newer link,
report selection, dismissal or foreground tab choice can retire the older
link while its file enters the cache. Reply false to a retired request so the
web panel can stay open without claiming the source was revealed.

<ui:copy id="simple-coverage-android">
package com.hickorydocs.simple
import androidx.compose.runtime.*
import androidx.compose.foundation.layout.*
import androidx.compose.material3.Text
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.*
import org.json.JSONObject
class SimpleCoverageState {
 val identity=java.util.UUID.randomUUID().toString();var selected by mutableStateOf<String?>(null);var notice by mutableStateOf("")
 private var record by mutableStateOf<JSONObject?>(null);private var projected by mutableStateOf<Map<String,List<CoverageProjectedLine>>>(emptyMap());private var ticket=java.util.UUID.randomUUID()
 private var sourceLink = 0L
 internal fun retireSourceLink() { sourceLink++ }
 internal fun beginSourceLink(): () -> Boolean {
  val opening = ++sourceLink
  return { sourceLink == opening }
 }
 var digests:SourceDigests?=null
 fun stop(){ticket=java.util.UUID.randomUUID();digests?.stop();record=null;projected=emptyMap()}
 fun select(id:String?,workspace:Workspace){
  retireSourceLink()
  ticket=java.util.UUID.randomUUID();val read=ticket;selected=id;record=null;projected=emptyMap();notice=""
  if(id==null)return;val engine=workspace.engine ?: return
  workspace.runCommand {try{val reply=engine.request("POST","/api/tests/coverage",JSONObject().put("action","get").put("id",id));val value=reply
   val rows=withContext(Dispatchers.Default){coverageProjection(value)}
   if(workspace.coverage!==this@SimpleCoverageState||workspace.engine!==engine||ticket!=read||value.optString("id")!=id)return@runCommand
   record=value;projected=rows;workspace.buffers.values.forEach{observe(it)}
  }catch(cancelled:CancellationException){throw cancelled}catch(error:Exception){if(workspace.coverage===this@SimpleCoverageState&&workspace.engine===engine&&ticket==read)notice=error.message.orEmpty()}}
 }
 fun observe(buffer:DocumentBuffer){if(buffer.file.path !in projected)return;digests?.enqueue(buffer.file.path,buffer.text){buffer.text}}
 fun rows(buffer:DocumentBuffer):List<CoverageProjectedLine>{val expected=record?.optJSONObject("identity")?.optJSONObject("source_hashes")?.optString(buffer.file.path) ?: return emptyList();return if(digests?.current(buffer.file.path,buffer.text)==expected.lowercase())projected[buffer.file.path].orEmpty() else emptyList()}
}
fun DocumentBuffer.coverageRows()=coverageSource?.rows(this).orEmpty()
fun DocumentBuffer.readCoverageSource(){coverageSource?.observe(this)}
suspend fun Workspace.navigateCoverage(value:JSONObject,state:SimpleCoverageState,isCurrent:()->Boolean={true},
    foreground:()->Boolean=captureForegroundNavigation()): Boolean {
    val server=engine;val epoch=sourceNavigationEpoch
    fun alive()=isCurrent()&&coverage===state&&engine===server&&sourceNavigationEpoch==epoch
    if (!foreground() || !alive()) return false
    if (value.has("runId") && !value.isNull("runId") && value.opt("runId") !is String) return false
    if (value.has("runId") && (value.opt("runId") as? String) != state.selected) return false
    val path=value.getString("path")
    fun find(files:List<FileNode>):FileNode? {for(file in files){if(file.path==path)return file;find(file.children)?.let{return it}};return null}
    val file=(buffers[path]?.file ?: find(rootFiles))?.takeUnless{it.dir} ?: error("Source no longer listed. Refresh the workspace.")
    val buffer=loadBuffer(file);val source=buffer.text;val revision=buffer.sourceRevision;val selection=buffer.state.selection
    if (!foreground() || !alive() || buffers[path] !== buffer) return false
    fun sourceCurrent()=alive()&&buffers[path]===buffer&&buffer.sourceRevision==revision&&buffer.text==source&&buffer.state.selection==selection
    if(value.isNull("line")||value.isNull("sourceHash")){
        if (!sourceCurrent()) return false
        val departed=currentSemanticLocation();show(sourceTab(path));currentSemanticLocation()?.let{navigationTrail=navigationTrail.record(departed,it)}
        showCoverage=false
        return true
    }
    val rawLine=value.opt("line") as? Number ?: error("Malformed coverage source position. Reopen the report.");val line=rawLine.toInt();check(rawLine.toDouble()==line.toDouble()){"Malformed coverage source position. Reopen the report."};val hash=value.opt("sourceHash") as? String ?: error("Malformed coverage source hash. Reopen the report.")
    val actual=withContext(Dispatchers.Default){java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}}
    if (!foreground() || !sourceCurrent()) return false
    check(actual==hash.lowercase()&&line>0&&line<=source.count{it=='\n'}+1){"Source changed or report position is unavailable. Refresh coverage before navigating."}
    var revealed=false;openSemanticLocation(SemanticLocation(languageURI(path),line-1,0),allowDirty=true,expectedSource=source,isCurrent=::sourceCurrent,onRevealed={revealed=true})
    if (!alive() || buffers[path] !== buffer || !revealed) return false
    showCoverage=false
    return true
}
internal class NativeCoverageController(val workspace:Workspace,val scope:CoroutineScope,private val attached:()->Boolean,private val sendReply:(Int,Any,String?)->Unit,private val display:()->Unit) {
    val state=workspace.coverage;val engine=workspace.engine;var ready=false;private var closed=false;private val active=mutableSetOf<Int>()
    val current get()=!closed&&attached()&&workspace.coverage===state&&workspace.engine===engine
    private fun reply(id:Int,result:Any,error:String?){if(current)sendReply(id,result,error)}
    fun receive(event:JSONObject){if(!current)return;val kind=event.optString("kind")
        if(kind=="ready"){ready=true;display();return};if(event.optString("workspace")!=state.identity)return
        if(kind=="close"){closed=true;workspace.showCoverage=false;workspace.currentBuffer?.focusSource?.invoke();return}
        if(kind=="retire-navigation"){state.retireSourceLink();return}
        if(kind=="select"){state.select(if(event.isNull("id")) null else event.getString("id"),workspace);return}
        val rawID=event.opt("id") as? Number ?: return;val id=rawID.toInt();if(rawID.toDouble()!=id.toDouble()||id<=0||active.contains(id)||active.size>=64)return;active.add(id)
        val link = if (kind == "navigate") state.beginSourceLink() else null
        val foreground = workspace.captureForegroundNavigation()
        scope.launch{try{if(!current)return@launch
            if(kind=="api"){val body=event.getJSONObject("body");check(body.optString("action") in listOf("list","get","locations","inspect","discover","run","rerun-failed","manual-start","browser-checkpoint","checkpoint","stop","import","compare","annotate")){"Unsupported coverage request. Reopen the tool."}
                val result=(engine ?: error("Connect to a workspace first.")).request("POST","/api/tests/coverage",body);reply(id,result,null)
            }else if(link != null){
                val revealed = workspace.navigateCoverage(event.getJSONObject("location"),state,
                    isCurrent = { current && workspace.showCoverage && link() }, foreground = foreground)
                reply(id,revealed,null)
            }
            else error("Unsupported coverage request. Reopen the tool.")
        }catch(cancelled:CancellationException){throw cancelled}catch(error:Exception){reply(id,JSONObject.NULL,error.message)}finally{active.remove(id)}}
    }
}
private class CoverageWebController(val workspace:Workspace,val scope:CoroutineScope) {
    var view:android.webkit.WebView?=null
    val bridge=NativeCoverageController(workspace,scope,{view!=null},{id,result,error->val args=org.json.JSONArray().put(id).put(result).put(error ?: JSONObject.NULL);view?.evaluateJavascript("globalThis.hickoryCoverage?.reply(...$args)",null)},{display()})
    fun display(){if(!bridge.current||!bridge.ready)return;val buffer=workspace.currentBuffer
        val value=JSONObject().put("workspace",bridge.state.identity).put("path",buffer?.file?.path ?: JSONObject.NULL).put("language",buffer?.file?.language ?: "unsupported").put("selectedId",bridge.state.selected ?: JSONObject.NULL)
        view?.evaluateJavascript("globalThis.hickoryCoverage?.update($value)",null)
    }
    @android.webkit.JavascriptInterface fun postMessage(json:String){if(json.length>4_000_000)return;view?.post{runCatching{bridge.receive(JSONObject(json))}}}
}
@android.annotation.SuppressLint("SetJavaScriptEnabled")
@Composable fun SimpleCoverageView(workspace:Workspace){
    val scope=rememberCoroutineScope();val controller=remember(workspace.coverage){CoverageWebController(workspace,scope)}
    Column(Modifier.widthIn(min=280.dp,max=900.dp).heightIn(min=240.dp,max=800.dp).fillMaxSize()){
        if(workspace.coverage.notice.isNotEmpty())Text(workspace.coverage.notice)
        androidx.compose.ui.viewinterop.AndroidView(modifier=Modifier.fillMaxSize(),factory={context->android.webkit.WebView(context).apply{
            controller.view=this;settings.javaScriptEnabled=true;settings.allowFileAccess=false;settings.allowContentAccess=false;settings.blockNetworkLoads=true
            addJavascriptInterface(controller,"HickoryCoverage")
            webViewClient=object:android.webkit.WebViewClient(){
                override fun shouldOverrideUrlLoading(view:android.webkit.WebView,request:android.webkit.WebResourceRequest)=request.url.toString()!="file:///android_asset/coverage.html"
                override fun onRenderProcessGone(view:android.webkit.WebView,detail:android.webkit.RenderProcessGoneDetail):Boolean{workspace.coverage.notice="Coverage renderer stopped. Close and reopen it; source remains available.";controller.view=null;view.destroy();return true}
            };loadUrl("file:///android_asset/coverage.html")
        }},onRelease={controller.view=null;it.removeJavascriptInterface("HickoryCoverage");it.stopLoading();it.destroy()})
    }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/CoverageView.kt"><ui:paste select="#simple-coverage-android"/></ui:file>
## Bind captured measurements to available text

This native service witness reads a measured-record fixture through the real engine client, binds two mixed generated consequences to their authored owner, and checks that a normalization-only edit immediately removes the mark. A digest of the changed source cannot revive it; restoring exact bytes can. Clearing the selection also retires the observation. It validates binding and revision safety, not a collector execution or device pointer interaction.

The held-source witness uses the native message bridge to start parallel file
reads. It releases an older link first, then retires the latest link by tab
choice and by the panel's report event. Each reply remains associated with its
request ID. A fresh click opens the cached file at the exact reported line.

<ui:copy id="coverage-source-binding-android-witness">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
class CoverageBindingTest {
 @Test fun liveOwnedCollectorBridgeResultsRerunAndSourceGuards()=runBlocking {
  val origin=System.getenv("HICKORY_COLLECTOR_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_COLLECTOR_WORKSPACE") ?: error("Owned collector workspace required"));check(java.io.File(root,".hickory-collector-fixture").readText()=="owned-collector-v1\n")
  val dispatcher=java.util.concurrent.Executors.newSingleThreadExecutor().asCoroutineDispatcher();val scope=CoroutineScope(SupervisorJob()+dispatcher);val w=Workspace(scope);val engine=Engine(origin!!);w.engine=engine;val (_,tree)=engine.files();w.rootFiles=tree
  try{val file=tree.first{it.path=="app.py"};val buffer=w.loadBuffer(file);w.show(sourceTab(file.path));w.showCoverage=true;val source=buffer.text;val textState=buffer.state;val scroll=buffer.sourceScroll;val state=w.coverage
   val pending=java.util.concurrent.ConcurrentHashMap<Int,CompletableDeferred<JSONObject>>();var sequence=0;var attached=true
   val bridge=NativeCoverageController(w,scope,{attached},{id,value,error->pending.remove(id)?.let{if(error!=null)it.completeExceptionally(IllegalStateException(error)) else it.complete(value as? JSONObject ?: JSONObject())}},{})
   suspend fun request(kind:String,payload:JSONObject):JSONObject {val id=++sequence;val answer=CompletableDeferred<JSONObject>();pending[id]=answer;val message=JSONObject().put("kind",kind).put("id",id).put("workspace",state.identity);if(kind=="api")message.put("body",payload) else message.put("location",payload);withContext(dispatcher){bridge.receive(message)};return withTimeout(60000){answer.await()}}
   suspend fun api(action:String,id:String?=null)=request("api",JSONObject().put("action",action).put("path",".").put("language","python").also{if(id!=null)it.put("id",id)})
   suspend fun finished(id:String):JSONObject {return withTimeout(60000){var result=api("get",id);while(result.getString("status")=="running"){delay(50);result=api("get",id)};result}}
   val listing=api("discover");assertEquals(2,listing.getJSONArray("tests").length());assertFalse(java.io.File(root,".hick-cache/executions").exists());val first=finished(api("run").getString("id"));assertEquals("failed",first.getString("status"));assertEquals(2,first.getJSONArray("tests").length());assertEquals("current",first.getString("source_state"));assertTrue(first.getJSONObject("evidence").getJSONObject("report").getJSONObject("files").has("app.py"))
   withContext(dispatcher){bridge.receive(JSONObject().put("kind","select").put("workspace",state.identity).put("id",first.getString("id")))};withTimeout(10000){while(buffer.coverageRows().isEmpty())delay(20)}
   val second=finished(api("rerun-failed",first.getString("id")).getString("id"));assertEquals("failed",second.getString("status"));assertEquals(1,second.getJSONArray("tests").length());assertEquals("test_app.py::test_negative",second.getJSONArray("tests").getJSONObject(0).getString("id"));assertNotEquals(first.getString("id"),second.getString("id"));assertEquals(2,api("list").getJSONArray("runs").length())
   val location=JSONObject().put("path",file.path).put("line",2).put("sourceHash",debugHash(source));request("navigate",location);assertFalse(w.showCoverage);assertEquals(source.indexOf('\n')+1,buffer.state.selection.start);assertSame(textState,buffer.state);assertSame(scroll,buffer.sourceScroll)
   buffer.applySourceEdits(listOf(NativeSourceEdit(buffer.text.length,buffer.text.length,"# unsaved after measurement\n")),buffer.text);val draft=buffer.text;val selection=buffer.state.selection;assertTrue(buffer.coverageRows().isEmpty());w.showCoverage=true;val stale=runCatching{request("navigate",location)};assertTrue(stale.isFailure);assertTrue(w.showCoverage);assertEquals(draft,buffer.text);assertEquals(selection,buffer.state.selection);assertSame(textState,buffer.state);assertSame(scroll,buffer.sourceScroll)
   attached=false;withContext(dispatcher){bridge.receive(JSONObject().put("kind","select").put("workspace",state.identity).put("id","foreign"))};assertEquals(first.getString("id"),state.selected);assertEquals(source,java.io.File(root,"app.py").readText())
  }finally{w.buffers.values.forEach{it.language?.close()};w.coverage.stop();w.sourceTestIndex?.stop();scope.cancel();dispatcher.close()}
 }

 @Test fun navigationRetiresClosedForeignAndEqualByteReplies()=runBlocking {
  for(mode in listOf("closed","engine","owner","undo","selection","fraction","boolean","success","unbound")){
   val w=Workspace(this);val engine=Engine("http://127.0.0.1:1");val path="literal %?#.py";val f=FileNode(path,path,false,emptyList(),null,null);val source="first\nsecond\n";val b=DocumentBuffer(f,FileText(source,"base"));w.engine=engine;w.buffers[path]=b;w.show(sourceTab(path));w.showCoverage=true;b.state.edit{selection=androidx.compose.ui.text.TextRange(2)}
   val state=w.coverage;var calls=0;val value=JSONObject().put("path",path).put("line",2).put("sourceHash",debugHash(source));if(mode=="fraction")value.put("line",1.5);if(mode=="boolean")value.put("line",true);if(mode=="unbound"){value.put("line",JSONObject.NULL);value.put("sourceHash",JSONObject.NULL)}
   val current:()->Boolean={calls++;if(mode=="engine"&&calls==2)w.engine=Engine("http://127.0.0.1:2");if(calls==3){when(mode){"undo"->{b.state.edit{replace(0,length,"later")};b.advanceSourceRevision();b.state.edit{replace(0,length,source)};b.advanceSourceRevision()};"selection"->b.state.edit{selection=androidx.compose.ui.text.TextRange(1)};"owner"->w.buffers[path]=DocumentBuffer(f,FileText(source,"base"))}};mode!="closed"||calls<2}
   val result=runCatching{w.navigateCoverage(value,state,current)};if(mode=="success"||mode=="unbound")assertEquals("$mode: "+result.exceptionOrNull(),true,result.getOrNull()) else assertNotEquals(mode,true,result.getOrNull())
   if(mode=="success"){assertEquals(6,b.state.selection.start);assertFalse(w.showCoverage)}else if(mode=="unbound"){assertEquals(2,b.state.selection.start);assertFalse(w.showCoverage)}else assertTrue(mode,w.showCoverage)
   assertEquals(source,b.text);assertEquals(source,b.base)
  }
 }

 @Test fun heldCoverageLinksRespectLatestRequestAndForegroundChoice() = runBlocking {
  val source = "first\nsecond\n"
  val paths = listOf("older.py", "newer.py", "retired.py")
  val arrived = paths.associateWith { CompletableDeferred<Unit>() }
  val gates = paths.associateWith { java.util.concurrent.CountDownLatch(1) }
  val listener = com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1", 0), 0)
  val requests = java.util.concurrent.Executors.newFixedThreadPool(3)
  listener.executor = requests
  listener.createContext("/api/file") { exchange ->
   val path = exchange.requestURI.query.removePrefix("path=")
   arrived.getValue(path).complete(Unit)
   check(gates.getValue(path).await(8, java.util.concurrent.TimeUnit.SECONDS))
   val bytes = JSONObject().put("content", source).put("hash", "saved").toString().toByteArray()
   exchange.sendResponseHeaders(200, bytes.size.toLong())
   exchange.responseBody.use { it.write(bytes) }
  }
  listener.start()
  fun file(path: String) = FileNode(path, path, false, emptyList(), null, null)
  val workspace = Workspace(this)
  workspace.engine = Engine("http://127.0.0.1:${listener.address.port}")
  val draft = DocumentBuffer(file("draft.py"), FileText("local", "saved"))
  val other = DocumentBuffer(file("other.py"), FileText("other", "saved"))
  val selection = androidx.compose.ui.text.TextRange(2, 4)
  draft.state.edit { append(" draft"); this.selection = selection }
  workspace.rootFiles = paths.map(::file)
  workspace.buffers[draft.file.path] = draft
  workspace.buffers[other.file.path] = other
  workspace.show(sourceTab(other.file.path))
  workspace.show(sourceTab(draft.file.path))
  workspace.showCoverage = true
  val state = workspace.coverage
  val pending = mutableMapOf<Int, CompletableDeferred<Pair<Any, String?>>>()
  val bridge = NativeCoverageController(workspace, this, { true }, { id, value, error ->
   pending.getValue(id).complete(value to error)
  }, {})
  fun message(kind: String) = JSONObject().put("kind", kind).put("workspace", state.identity)
  fun navigate(id: Int, path: String, runId: String? = null): CompletableDeferred<Pair<Any, String?>> {
   val answer = CompletableDeferred<Pair<Any, String?>>()
   pending[id] = answer
   bridge.receive(message("navigate").put("id", id).put("location", JSONObject().put("path", path)
    .put("line", 2).put("sourceHash", debugHash(source)).apply { if (runId != null) put("runId", runId) }))
   return answer
  }
  try {
   assertEquals(false to null, withTimeout(4000) { navigate(5, paths[0], "foreign-report").await() })
   assertFalse(arrived.getValue(paths[0]).isCompleted)
   val older = navigate(1, paths[0])
   withTimeout(4000) { arrived.getValue(paths[0]).await() }
   val newer = navigate(2, paths[1])
   withTimeout(4000) { arrived.getValue(paths[1]).await() }
   gates.getValue(paths[0]).countDown()
   assertEquals(false to null, withTimeout(4000) { older.await() })
   assertTrue(workspace.showCoverage)
   assertSame(draft, workspace.currentBuffer)

   workspace.show(sourceTab(other.file.path))
   workspace.show(sourceTab(draft.file.path))
   val layout = workspace.layout
   val trail = workspace.navigationTrail
   gates.getValue(paths[1]).countDown()
   assertEquals(false to null, withTimeout(4000) { newer.await() })

   val retired = navigate(3, paths[2])
   withTimeout(4000) { arrived.getValue(paths[2]).await() }
   bridge.receive(message("retire-navigation"))
   gates.getValue(paths[2]).countDown()
   assertEquals(false to null, withTimeout(4000) { retired.await() })
   assertSame(layout, workspace.layout)
   assertSame(trail, workspace.navigationTrail)
   assertSame(draft, workspace.currentBuffer)
   assertEquals("local draft", draft.text)
   assertEquals(selection, draft.state.selection)
   assertTrue(workspace.showCoverage)
   paths.forEach { assertEquals(source, workspace.buffers.getValue(it).text) }

   assertEquals(true to null, withTimeout(4000) { navigate(4, paths[1]).await() })
   assertFalse(workspace.showCoverage)
   assertSame(workspace.buffers.getValue(paths[1]), workspace.currentBuffer)
   assertEquals(androidx.compose.ui.text.TextRange(6), workspace.currentBuffer?.state?.selection)
   assertEquals(selection, draft.state.selection)
  } finally {
   gates.values.forEach { it.countDown() }
   workspace.buffers.values.forEach { it.language?.close() }
   workspace.coverage.stop()
   listener.stop(0)
   requests.shutdownNow()
  }
 }

 @Test fun mixedOwnerMarksRetireImmediatelyAfterExactByteEdit()=runBlocking {
  val source="// é\nvalue\n";val path="owner.kt";val hash=java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}
  val record=JSONObject().put("id","measured").put("source_changed_during_run",false).put("identity",JSONObject().put("source_hashes",JSONObject().put(path,hash).put("generated.kt","a".repeat(64))))
   .put("evidence",JSONObject().put("report",JSONObject().put("files",JSONObject().put("generated.kt",JSONObject().put("line_counts",false).put("lines",JSONObject().put("1",1).put("2",0))))))
   .put("mappings",JSONObject().put("generated.kt",JSONObject().put("1",JSONObject().put("path",path).put("line",2)).put("2",JSONObject().put("path",path).put("line",2))))
  val listener=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0)
  listener.createContext("/api/tests/coverage"){exchange->val bytes=record.toString().toByteArray();exchange.requestBody.close();exchange.sendResponseHeaders(200,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}};listener.start()
  val workspace=Workspace(this);workspace.engine=Engine("http://127.0.0.1:${listener.address.port}");val state=workspace.coverage;state.digests=SourceDigests(this)
  val buffer=DocumentBuffer(FileNode(path,path,false,emptyList(),null,null),FileText(source,hash));buffer.coverageSource=state;workspace.buffers[path]=buffer
  suspend fun settle(condition:()->Boolean){withTimeout(3000){while(!condition()){delay(2)}}}
  try{state.select("measured",workspace);settle{buffer.coverageRows().isNotEmpty()};val row=buffer.coverageRows().first();assertEquals(1,row.line);assertEquals("mixed",row.state);assertNull(row.count);assertEquals(2,row.origins.size)
   buffer.state.edit{replace(0,length,"// e\u0301\nvalue\n")};assertTrue(buffer.coverageRows().isEmpty());buffer.readCoverageSource();settle{state.digests!!.current(path,buffer.text)!=null};assertTrue(buffer.coverageRows().isEmpty())
   buffer.state.edit{replace(0,length,source)};buffer.readCoverageSource();settle{buffer.coverageRows().isNotEmpty()};assertEquals("mixed",buffer.coverageRows().first().state);state.select(null,workspace);assertTrue(buffer.coverageRows().isEmpty())
  }finally{state.stop();listener.stop(0)}
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/CoverageBindingTest.kt"><ui:paste select="#coverage-source-binding-android-witness"/></ui:file>
</ui:doc>
