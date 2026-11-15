package com.hickorydocs.simple
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.json.JSONObject
import kotlinx.coroutines.launch
import kotlinx.coroutines.CancellationException
import java.net.URLEncoder

data class AttributionLine(val line:Int,val commit:String,val fullCommit:String?,val author:String,val summary:String,val uncommitted:Boolean,val provider:String?,val text:String?){val immutableCommit:String? get()=fullCommit?.takeIf{!uncommitted&&it.matches(Regex("([0-9a-f]{40}|[0-9a-f]{64})"))}}
data class AttributionReading(val path:String,val lines:List<AttributionLine>)
fun attributionReading(raw:JSONObject):AttributionReading {val rows=raw.getJSONArray("lines");return AttributionReading(raw.getString("path"),(0 until rows.length()).map{val row=rows.getJSONObject(it);AttributionLine(row.getInt("line"),row.getString("commit"),if(row.isNull("full_commit"))null else row.getString("full_commit"),row.getString("author"),row.getString("summary"),row.getBoolean("uncommitted"),if(row.isNull("provider"))null else row.getString("provider"),if(row.isNull("text"))null else row.getString("text"))})}
class GitAttributionModel(val path:String){
 var lines by mutableStateOf<List<AttributionLine>>(emptyList());var notice by mutableStateOf("");var page by mutableIntStateOf(0);var loading by mutableStateOf(false);private var ticket=0L
 fun close(){ticket++;loading=false;lines=emptyList();notice=""}
 suspend fun refresh(load:suspend (String)->AttributionReading){val id=++ticket;lines=emptyList();page=0;loading=true;notice="Reading saved-file attribution…"
  try{val value=load(path);if(ticket!=id)return;check(value.path==path){"Attribution identity changed. Refresh the saved file."};check(value.lines.all{it.line>0}&&value.lines.map{it.line}.distinct().size==value.lines.size){"Ambiguous attribution lines. Refresh the saved file."};lines=value.lines.sortedBy{it.line};notice=if(lines.isEmpty())"No saved Git or recorded attribution is available." else "Captured saved attribution · draft line positions may differ."}catch(error:CancellationException){throw error}catch(error:Exception){if(ticket==id)notice=error.message.orEmpty()}finally{if(ticket==id)loading=false}
 }
}
@Composable fun GitAttributionView(workspace:Workspace,model:GitAttributionModel){
 val engine=workspace.engine;val scope=rememberCoroutineScope()
 suspend fun refresh(){if(engine==null){model.notice="Connect to a workspace server first.";return};model.refresh{attributionReading(engine.request("GET","/api/blame?path=${URLEncoder.encode(it,"UTF-8")}"))}}
 LaunchedEffect(engine,model){refresh()};DisposableEffect(model){onDispose{model.close()}}
 Column(Modifier.widthIn(max=680.dp).heightIn(max=680.dp).padding(16.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(8.dp)){
  Text("Saved attribution · ${model.path}",style=MaterialTheme.typography.titleMedium);Row{TextButton(onClick={scope.launch{refresh()}}){Text("Refresh")};TextButton(onClick={workspace.showAttribution=false;workspace.currentBuffer?.focusSource?.invoke()}){Text("Close")}}
  Text(model.notice,fontSize=12.sp);Text("Saved-file lines only. Unsaved editor changes are not included.",fontSize=12.sp)
  model.lines.drop(model.page*50).take(50).forEach{line->Text("Saved line ${line.line} · ${line.author} · ${line.commit.ifEmpty{"No committed revision"}}",style=MaterialTheme.typography.titleSmall);SelectionContainer{Text("${line.summary} · ${line.provider ?: if(line.uncommitted)"Uncommitted saved line" else "Git blame"}",fontSize=12.sp)};SelectionContainer{Text(line.text?.take(2000) ?: "Line text unavailable.",fontFamily=FontFamily.Monospace,fontSize=12.sp)};if((line.text?.length ?: 0)>2000)Text("Line text abbreviated.",fontSize=12.sp)
   val sha=line.immutableCommit;if(sha!=null)TextButton(onClick={if(model.lines.any{it.line==line.line&&it.immutableCommit==sha}){workspace.gitHistory.close();workspace.gitHistory=GitHistoryModel(GitHistoryRequest.Commit(sha,"Commit from saved attribution · ${model.path}:${line.line}"));workspace.showAttribution=false;workspace.showReview=false;workspace.showHistory=true}}){Text("Read immutable commit for saved line ${line.line}")}else Text("Full immutable commit reference unavailable.",fontSize=12.sp)
  }
  Row{TextButton(enabled=model.page>0,onClick={model.page--}){Text("Previous")};Text("${model.lines.size} reported lines · page ${model.page+1}");TextButton(enabled=(model.page+1)*50<model.lines.size,onClick={model.page++}){Text("Next")}}
 }
}
