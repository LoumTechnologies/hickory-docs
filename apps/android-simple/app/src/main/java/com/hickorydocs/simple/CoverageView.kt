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
   if(workspace.coverage!==this@SimpleCoverageState||workspace.engine!==engine||ticket!=read||value.optString("id")!=id)return@runCommand
   record=value;projected=rows;workspace.buffers.values.forEach{observe(it)}
  }catch(cancelled:CancellationException){throw cancelled}catch(error:Exception){if(workspace.coverage===this@SimpleCoverageState&&workspace.engine===engine&&ticket==read)notice=error.message.orEmpty()}}
 }
 fun observe(buffer:DocumentBuffer){if(buffer.file.path !in projected)return;digests?.enqueue(buffer.file.path,buffer.text){buffer.text}}
 fun rows(buffer:DocumentBuffer):List<CoverageProjectedLine>{val expected=record?.optJSONObject("identity")?.optJSONObject("source_hashes")?.optString(buffer.file.path) ?: return emptyList();return if(digests?.current(buffer.file.path,buffer.text)==expected.lowercase())projected[buffer.file.path].orEmpty() else emptyList()}
}
fun DocumentBuffer.coverageRows()=coverageSource?.rows(this).orEmpty()
fun DocumentBuffer.readCoverageSource(){coverageSource?.observe(this)}
suspend fun Workspace.navigateCoverage(value:JSONObject,state:SimpleCoverageState,isCurrent:()->Boolean={true}) {
    val server=engine;val epoch=sourceNavigationEpoch
    fun alive()=isCurrent()&&coverage===state&&engine===server&&sourceNavigationEpoch==epoch
    check(alive()){"Workspace or coverage reader changed. Reopen coverage."};val path=value.getString("path")
    fun find(files:List<FileNode>):FileNode? {for(file in files){if(file.path==path)return file;find(file.children)?.let{return it}};return null}
    val file=(buffers[path]?.file ?: find(rootFiles))?.takeUnless{it.dir} ?: error("Source no longer listed. Refresh the workspace.")
    val buffer=loadBuffer(file);val source=buffer.text;val revision=buffer.sourceRevision;val selection=buffer.state.selection
    check(alive()&&buffers[path]===buffer){"Workspace or source editor changed. Reopen coverage."}
    fun sourceCurrent()=alive()&&buffers[path]===buffer&&buffer.sourceRevision==revision&&buffer.text==source&&buffer.state.selection==selection
    if(value.isNull("line")||value.isNull("sourceHash")){check(sourceCurrent()){"Source or coverage reader changed. Reopen coverage."};val departed=currentSemanticLocation();show(sourceTab(path));currentSemanticLocation()?.let{navigationTrail=navigationTrail.record(departed,it)};showCoverage=false;return}
    val rawLine=value.opt("line") as? Number ?: error("Malformed coverage source position. Reopen the report.");val line=rawLine.toInt();check(rawLine.toDouble()==line.toDouble()){"Malformed coverage source position. Reopen the report."};val hash=value.opt("sourceHash") as? String ?: error("Malformed coverage source hash. Reopen the report.")
    val actual=withContext(Dispatchers.Default){java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}}
    check(sourceCurrent()&&actual==hash.lowercase()&&line>0&&line<=source.count{it=='\n'}+1){"Source changed or report position is unavailable. Refresh coverage before navigating."}
    var revealed=false;openSemanticLocation(SemanticLocation(languageURI(path),line-1,0),allowDirty=true,expectedSource=source,isCurrent=::sourceCurrent,onRevealed={revealed=true})
    check(alive()&&buffers[path]===buffer&&revealed){"Coverage source navigation did not complete. The report remains readable."};showCoverage=false
}
internal class NativeCoverageController(val workspace:Workspace,val scope:CoroutineScope,private val attached:()->Boolean,private val sendReply:(Int,Any,String?)->Unit,private val display:()->Unit) {
    val state=workspace.coverage;val engine=workspace.engine;var ready=false;private var closed=false;private val active=mutableSetOf<Int>()
    val current get()=!closed&&attached()&&workspace.coverage===state&&workspace.engine===engine
    private fun reply(id:Int,result:Any,error:String?){if(current)sendReply(id,result,error)}
    fun receive(event:JSONObject){if(!current)return;val kind=event.optString("kind")
        if(kind=="ready"){ready=true;display();return};if(event.optString("workspace")!=state.identity)return
        if(kind=="close"){closed=true;workspace.showCoverage=false;workspace.currentBuffer?.focusSource?.invoke();return}
        if(kind=="select"){state.select(if(event.isNull("id")) null else event.getString("id"),workspace);return}
        val rawID=event.opt("id") as? Number ?: return;val id=rawID.toInt();if(rawID.toDouble()!=id.toDouble()||id<=0||active.contains(id)||active.size>=64)return;active.add(id)
        scope.launch{try{if(!current)return@launch
            if(kind=="api"){val body=event.getJSONObject("body");check(body.optString("action") in listOf("list","get","locations","inspect","discover","run","rerun-failed","manual-start","browser-checkpoint","checkpoint","stop","import","compare","annotate")){"Unsupported coverage request. Reopen the tool."}
                val result=(engine ?: error("Connect to a workspace first.")).request("POST","/api/tests/coverage",body);reply(id,result,null)
            }else if(kind=="navigate"){workspace.navigateCoverage(event.getJSONObject("location"),state){current};reply(id,JSONObject.NULL,null)}
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
