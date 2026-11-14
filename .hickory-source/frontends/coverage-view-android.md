<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read runner results and coverage on android

For maintainers changing workspace services independently of the editor renderer.

The existing menu opens a floating, replaceable tool surface. The editor remains mounted and retains its own selection, scroll and undo state. A native local-only web renderer delegates only fixed coverage operations to the connected engine; workspace replacement retires its replies. Source navigation opens unbound files without positional claims, and accepts a reported line only after matching its captured hash against available text. Selecting a report retains its record identity; source-line mark binding is a separate capability.

<ui:copy id="simple-coverage-android">
package com.hickorydocs.simple
import androidx.compose.runtime.*
import androidx.compose.foundation.layout.*
import androidx.compose.material3.Text
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.*
import org.json.JSONObject
class SimpleCoverageState {val identity=java.util.UUID.randomUUID().toString();var selected by mutableStateOf<String?>(null);var notice by mutableStateOf("")}
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
        if(kind=="select"){state.selected=if(event.isNull("id")) null else event.getString("id");return}
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
</ui:doc>
