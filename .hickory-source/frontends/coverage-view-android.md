<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read runner results and coverage on android

For maintainers changing workspace services independently of the editor renderer.

The existing menu opens a floating, replaceable tool surface. The editor remains mounted and retains its own selection, scroll and undo state. A native local-only web renderer delegates only fixed coverage operations to the connected engine; workspace replacement retires its replies. Source navigation opens unbound files without positional claims, and accepts a reported line only after matching its captured hash against available text. Selecting a report retains its record identity; source-line mark binding is a separate capability.

Selected records are read back by identity through the engine. A latest-selection ticket rejects obsolete replies, projection preserves mixed consequences and missing quantities, and the independent digest service binds only matching available text. Source edits retire marks synchronously at lookup; background hashing cannot rewrite the buffer. Both gutter and menu borrow this retained state.

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
 var digests:SourceDigests?=null
 fun stop(){ticket=java.util.UUID.randomUUID();digests?.stop();record=null;projected=emptyMap()}
 fun select(id:String?,workspace:Workspace){
  ticket=java.util.UUID.randomUUID();val read=ticket;selected=id;record=null;projected=emptyMap();notice=""
  if(id==null)return;val engine=workspace.engine ?: return
  workspace.runCommand {try{val reply=engine.request("POST","/api/tests/coverage",JSONObject().put("action","get").put("id",id));val value=reply
   val rows=withContext(Dispatchers.Default){coverageProjection(value)}
   if(workspace.coverage!==this@SimpleCoverageState||ticket!=read||value.optString("id")!=id)return@runCommand
   record=value;projected=rows;workspace.buffers.values.forEach{observe(it)}
  }catch(cancelled:CancellationException){throw cancelled}catch(error:Exception){if(workspace.coverage===this@SimpleCoverageState&&ticket==read)notice=error.message.orEmpty()}}
 }
 fun observe(buffer:DocumentBuffer){if(buffer.file.path !in projected)return;digests?.enqueue(buffer.file.path,buffer.text){buffer.text}}
 fun rows(buffer:DocumentBuffer):List<CoverageProjectedLine>{val expected=record?.optJSONObject("identity")?.optJSONObject("source_hashes")?.optString(buffer.file.path) ?: return emptyList();return if(digests?.current(buffer.file.path,buffer.text)==expected.lowercase())projected[buffer.file.path].orEmpty() else emptyList()}
}
fun DocumentBuffer.coverageRows()=coverageSource?.rows(this).orEmpty()
fun DocumentBuffer.readCoverageSource(){coverageSource?.observe(this)}
suspend fun Workspace.navigateCoverage(value:JSONObject,state:SimpleCoverageState) {
    check(coverage===state){"Workspace changed. Reopen coverage."};val path=value.getString("path")
    fun find(files:List<FileNode>):FileNode? {for(file in files){if(file.path==path)return file;find(file.children)?.let{return it}};return null}
    val file=(buffers[path]?.file ?: find(rootFiles))?.takeUnless{it.dir} ?: error("Source no longer listed. Refresh the workspace.")
    val buffer=loadBuffer(file);val source=buffer.text
    check(coverage===state){"Workspace changed. Reopen coverage."}
    if(value.isNull("line")||value.isNull("sourceHash")){open(file);showCoverage=false;return}
    val line=value.getInt("line");val actual=withContext(Dispatchers.Default){java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}}
    check(coverage===state&&buffer.text==source&&actual==value.getString("sourceHash").lowercase()&&line>0&&line<=source.count{it=='\n'}+1){"Source changed or report position is unavailable. Refresh coverage before navigating."}
    openSemanticLocation(SemanticLocation(languageURI(path),line-1,0),allowDirty=true,expectedSource=source);if(coverage===state)showCoverage=false
}
private class CoverageWebController(val workspace:Workspace,val scope:CoroutineScope) {
    val state=workspace.coverage;val engine=workspace.engine;var view:android.webkit.WebView?=null;var ready=false;val active=mutableSetOf<Int>()
    val current get()=view!=null&&workspace.coverage===state&&workspace.engine===engine
    fun display(){if(!current||!ready)return;val buffer=workspace.currentBuffer
        val value=JSONObject().put("workspace",state.identity).put("path",buffer?.file?.path ?: JSONObject.NULL).put("language",buffer?.file?.language ?: "unsupported").put("selectedId",state.selected ?: JSONObject.NULL)
        view?.evaluateJavascript("globalThis.hickoryCoverage?.update($value)",null)
    }
    fun reply(id:Int,result:Any,error:String?){if(!current)return;val args=org.json.JSONArray().put(id).put(result).put(error ?: JSONObject.NULL);view?.evaluateJavascript("globalThis.hickoryCoverage?.reply(...$args)",null)}
    @android.webkit.JavascriptInterface fun postMessage(json:String){if(json.length>4_000_000)return;view?.post{runCatching{receive(JSONObject(json))}}}
    fun receive(event:JSONObject){if(!current)return;val kind=event.optString("kind")
        if(kind=="ready"){ready=true;display();return};if(event.optString("workspace")!=state.identity)return
        if(kind=="close"){workspace.showCoverage=false;workspace.currentBuffer?.focusSource?.invoke();return}
        if(kind=="select"){state.select(if(event.isNull("id")) null else event.getString("id"),workspace);return}
        val id=event.optInt("id");if(id<=0||active.contains(id)||active.size>=64)return;active.add(id)
        scope.launch{try{if(!current)return@launch
            if(kind=="api"){val body=event.getJSONObject("body");check(body.optString("action") in listOf("list","get","locations","inspect","discover","run","rerun-failed","manual-start","browser-checkpoint","checkpoint","stop","import","compare","annotate")){"Unsupported coverage request. Reopen the tool."}
                val result=(engine ?: error("Connect to a workspace first.")).request("POST","/api/tests/coverage",body);reply(id,result,null)
            }else if(kind=="navigate"){workspace.navigateCoverage(event.getJSONObject("location"),state);reply(id,JSONObject.NULL,null)}
            else error("Unsupported coverage request. Reopen the tool.")
        }catch(cancelled:CancellationException){throw cancelled}catch(error:Exception){reply(id,JSONObject.NULL,error.message)}finally{active.remove(id)}}
    }
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

<ui:copy id="coverage-source-binding-android-witness">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
class CoverageBindingTest {
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
