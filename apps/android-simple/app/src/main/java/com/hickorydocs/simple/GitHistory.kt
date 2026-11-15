package com.hickorydocs.simple
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.horizontalScroll
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

sealed interface GitHistoryRequest {
 data class Log(val path:String?):GitHistoryRequest
 data class Commit(val sha:String,val title:String):GitHistoryRequest
}
data class HistoryCommit(val sha:String,val short:String,val author:String,val subject:String)
data class HistoryLog(val repository:Boolean,val commits:List<HistoryCommit>)
data class HistoryFile(val path:String,val status:String,val before:String,val after:String,val binary:Boolean)
data class HistoryReading(val sha:String,val message:String,val files:List<HistoryFile>)
fun historyLog(raw:JSONObject):HistoryLog{val rows=raw.getJSONArray("commits");return HistoryLog(raw.getBoolean("repository"),(0 until rows.length()).map{val row=rows.getJSONObject(it);HistoryCommit(row.getString("sha"),row.getString("short"),row.getString("author"),row.getString("subject"))})}
fun historyReading(raw:JSONObject):HistoryReading{val rows=raw.getJSONArray("files");return HistoryReading(raw.getString("sha"),raw.getString("message"),(0 until rows.length()).map{val row=rows.getJSONObject(it);HistoryFile(row.getString("path"),row.getString("status"),row.getString("before"),row.getString("after"),row.getBoolean("binary"))})}
class GitHistoryModel(val request:GitHistoryRequest=GitHistoryRequest.Log(null)) {
 var commits by mutableStateOf<List<HistoryCommit>>(emptyList());var reading by mutableStateOf<HistoryReading?>(null);var file by mutableIntStateOf(0);var before by mutableStateOf(false);var page by mutableIntStateOf(0);var limit by mutableIntStateOf(120);var notice by mutableStateOf("");var loading by mutableStateOf(false);private var ticket=0L
 fun close(){ticket++;loading=false;reading=null;notice=""}
 suspend fun refresh(limit:Int=120,load:suspend (Int,String?)->HistoryLog){if(request !is GitHistoryRequest.Log){notice="Choose a log request before reading commit choices.";return};val path=request.path;val id=++ticket;this.limit=limit.coerceIn(1,1000);commits=emptyList();reading=null;page=0;loading=true;notice="Reading saved history…"
  try{val value=load(this.limit,path);if(ticket!=id)return;check(value.commits.all{it.sha.matches(Regex("([0-9a-f]{40}|[0-9a-f]{64})"))}){"Malformed commit identity. Refresh repository history."};commits=value.commits;notice=if(value.repository)"${commits.size} reported commits · saved history only." else "This workspace is not a Git repository."}catch(error:CancellationException){throw error}catch(error:Exception){if(ticket==id)notice=error.message.orEmpty()}finally{if(ticket==id)loading=false}
 }
 suspend fun select(sha:String,load:suspend (String)->HistoryReading){if(commits.none{it.sha==sha})return;readCommit(sha,load)}
 suspend fun readCommit(sha:String,load:suspend (String)->HistoryReading){if(!sha.matches(Regex("([0-9a-f]{40}|[0-9a-f]{64})"))||(request as? GitHistoryRequest.Commit)?.sha?.let{it!=sha}==true){ticket++;reading=null;loading=false;notice="Commit reference is missing or changed. Refresh its source and choose the commit again.";return};val id=++ticket;reading=null;loading=true;notice="Reading immutable commit…"
  try{val value=load(sha);if(ticket!=id)return;check(value.sha==sha){"Commit identity changed. Refresh repository history."};reading=value;file=0;before=false;notice="Immutable commit reading · current editor draft is retained."}catch(error:CancellationException){throw error}catch(error:Exception){if(ticket==id)notice=error.message.orEmpty()}finally{if(ticket==id)loading=false}
 }
}
@Composable fun GitHistoryView(workspace:Workspace){
 val engine=workspace.engine;val model=workspace.gitHistory;val scope=rememberCoroutineScope();val request=model.request;val path=(request as? GitHistoryRequest.Log)?.path
 suspend fun refresh(count:Int=120){if(engine==null){model.notice="Connect to a workspace server first.";return};if(request is GitHistoryRequest.Commit){model.readCommit(request.sha){historyReading(engine.request("GET","/api/git/reading?sha=$it"))};return};model.refresh(count){count,path->historyLog(engine.request("GET","/api/git/log?limit=$count"+(path?.let{"&path=${URLEncoder.encode(it,"UTF-8")}"} ?: "")))}}
 suspend fun select(sha:String){if(engine==null)return;model.select(sha){historyReading(engine.request("GET","/api/git/reading?sha=$it"))}}
 LaunchedEffect(engine,model,path){refresh()};DisposableEffect(model){onDispose{model.close()}}
 Column(Modifier.widthIn(max=680.dp).heightIn(max=680.dp).padding(16.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(8.dp)){
  Text(if(request is GitHistoryRequest.Commit)request.title else path?.let{"Saved history · $it"} ?: "Repository history",style=MaterialTheme.typography.titleMedium)
  Row{TextButton(onClick={scope.launch{refresh(model.limit)}}){Text("Refresh")};TextButton(onClick={workspace.showHistory=false;workspace.currentBuffer?.focusSource?.invoke()}){Text("Close")}}
  Text(model.notice,fontSize=12.sp)
  if(request is GitHistoryRequest.Log){
  model.commits.drop(model.page*20).take(20).forEach{commit->TextButton(onClick={scope.launch{select(commit.sha)}}){Text("${commit.short} · ${commit.subject} · ${commit.author}")}}
  Row{TextButton(enabled=model.page>0,onClick={model.page--}){Text("Previous")};Text("Page ${model.page+1}");TextButton(enabled=(model.page+1)*20<model.commits.size,onClick={model.page++}){Text("Next")}}
  TextButton(enabled=model.limit<1000&&model.commits.size>=model.limit,onClick={scope.launch{refresh(minOf(1000,model.limit+120))}}){Text("Load more commits")}
  }
  model.reading?.let{reading->SelectionContainer{Text(reading.sha+"\n"+reading.message.take(20000),fontSize=12.sp)};if(reading.message.length>20000)Text("First 20,000 message characters shown.",fontSize=12.sp)
   if(reading.files.isNotEmpty()){Row{TextButton(enabled=model.file>0,onClick={model.file--}){Text("Previous file")};TextButton(enabled=model.file+1<reading.files.size,onClick={model.file++}){Text("Next file")}}
    val file=reading.files[model.file];Text("${file.status} · ${file.path}");Row{TextButton(onClick={model.before=true}){Text("Before commit")};TextButton(onClick={model.before=false}){Text("After commit")}};val content=if(model.before)file.before else file.after
    Text(if(file.binary)"Binary content · text reading unavailable." else if(content.length>200000)"First 200,000 characters shown." else if(content.isEmpty())"No text on this side." else "Immutable file content.",fontSize=12.sp)
    SelectionContainer{Text(if(file.binary)"" else content.take(200000),fontFamily=FontFamily.Monospace,fontSize=12.sp,modifier=Modifier.horizontalScroll(rememberScrollState()))}
   }else Text("No files under this workspace in this commit.")
  }
 }
}
