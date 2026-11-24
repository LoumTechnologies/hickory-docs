package com.hickorydocs.simple
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.horizontalScroll
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

data class ReviewFile(val path:String,val from:String?,val index:String,val tree:String,val fromOutside:Boolean=false)
data class ReviewChanges(val repository:Boolean,val branch:String?,val files:List<ReviewFile>)
data class ReviewDiff(val path:String,val diff:String,val binary:Boolean)
fun reviewChanges(raw:JSONObject):ReviewChanges {val rows=raw.getJSONArray("files");return ReviewChanges(raw.getBoolean("repository"),if(raw.isNull("branch"))null else raw.getString("branch"),(0 until rows.length()).map{val row=rows.getJSONObject(it);ReviewFile(row.getString("path"),if(row.isNull("from"))null else row.getString("from"),row.getString("index"),row.getString("tree"),row.optBoolean("from_outside_workspace"))})}
fun reviewIndexPaths(file:ReviewFile,unstage:Boolean):List<String> {
 val rename=(if(unstage)file.index else file.tree)=="R"
 check(!rename||file.from!=null&&!file.fromOutside){"Open the repository root to change this complete rename. No index request was sent."}
 return (if(rename)listOf(file.from!!,file.path)else listOf(file.path)).distinct()
}
class GitReviewModel {
 var indexPending by mutableStateOf(false);var needsRefresh by mutableStateOf(false)
 var files by mutableStateOf<List<ReviewFile>>(emptyList());var branch by mutableStateOf("");var selected by mutableStateOf<String?>(null);var staged by mutableStateOf(false);var detail by mutableStateOf("");var notice by mutableStateOf("");var loading by mutableStateOf(false);var page by mutableIntStateOf(0)
 private var ticket=0L
 fun close(){ticket++;loading=false}
 suspend fun refresh(load:suspend ()->ReviewChanges){val id=++ticket;loading=true;files=emptyList();branch="";selected=null;detail="";notice="Reading saved Git state…"
  try{val value=load();if(ticket!=id)return;files=value.files;if(!indexPending)needsRefresh=false;branch=value.branch.orEmpty();page=0;notice=if(value.repository)"Saved Git state · unsaved editor drafts are not included." else "This workspace is not a Git repository."}catch(error:CancellationException){throw error}catch(error:Exception){if(ticket==id)notice=error.message ?: "Cannot read Git changes. Refresh and retry."}finally{if(ticket==id)loading=false}
 }
 suspend fun select(path:String,staged:Boolean,load:suspend (String,Boolean)->ReviewDiff){if(indexPending||files.none{it.path==path})return;val id=++ticket;selected=path;this.staged=staged;detail="";loading=true;notice="Reading saved diff…"
  try{val value=load(path,staged);if(ticket!=id)return;check(value.path==path){"Diff identity changed. Refresh repository changes."};detail=value.diff.take(200000);notice=if(value.binary)"Binary change · text comparison unavailable." else if(value.diff.length>200000)"First 200,000 characters shown. Use Terminal for the complete diff." else if(value.diff.isEmpty())"No changes on this side." else "Captured Git diff · refresh after external changes."}catch(error:CancellationException){throw error}catch(error:Exception){if(ticket==id)notice=error.message ?: "Cannot read this diff. Refresh and retry."}finally{if(ticket==id)loading=false}
 }
 suspend fun changeIndex(path:String,unstage:Boolean,write:suspend (List<String>,Boolean)->Boolean,load:suspend ()->ReviewChanges){
  if(indexPending)return;if(needsRefresh){notice="Refresh Git state before another index action.";return};val file=files.firstOrNull{it.path==path} ?: return
  val paths=try{reviewIndexPaths(file,unstage)}catch(error:Exception){notice=error.message.orEmpty();return}
  val id=++ticket;indexPending=true;needsRefresh=true;loading=false;notice=if(unstage)"Unstaging saved paths…" else "Staging saved paths…"
  try{check(write(paths,unstage)){"Index operation was not confirmed."};if(ticket!=id)return;indexPending=false;refresh(load);if(ticket==id+1&&!needsRefresh)notice=if(unstage)"Saved paths unstaged. Editor drafts retained." else "Saved paths staged. Review the staged side before committing."}catch(error:CancellationException){throw error}catch(error:Exception){if(ticket==id)notice="Index result not confirmed: ${error.message}. Refresh Git state before retrying."}finally{indexPending=false}
 }

}
@Composable fun GitReviewView(workspace:Workspace){
 val engine=workspace.engine;val model=workspace.gitReview;val scope=rememberCoroutineScope()
 suspend fun refresh(){if(engine==null){model.notice="Connect to a workspace server first.";return};model.refresh{reviewChanges(engine.request("GET","/api/git/changes"))}}
 suspend fun select(path:String,staged:Boolean){if(engine==null)return;model.select(path,staged){value,index->val raw=engine.request("GET","/api/git/diff?path=${URLEncoder.encode(value,"UTF-8")}&staged=$index");ReviewDiff(raw.getString("path"),raw.getString("diff"),raw.getBoolean("binary"))}}
 suspend fun changeIndex(path:String,unstage:Boolean){if(engine==null)return;model.changeIndex(path,unstage,write={paths,index->engine.request("POST",if(index)"/api/git/unstage" else "/api/git/stage",JSONObject().put("paths",org.json.JSONArray(paths))).getBoolean("ok")},load={reviewChanges(engine.request("GET","/api/git/changes"))})}
 LaunchedEffect(engine){refresh()};DisposableEffect(model){onDispose{model.close()}}
 Column(Modifier.widthIn(max=680.dp).heightIn(max=620.dp).padding(16.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(8.dp)){
  Text("Repository changes ${model.branch}",style=MaterialTheme.typography.titleMedium)
  Row{WithDisabledReason(if (model.indexPending) "A Git index update is in progress." else null) { available -> TextButton(enabled = available,onClick={scope.launch{refresh()}}){Text("Refresh")} };TextButton(onClick={workspace.showReview=false;workspace.currentBuffer?.focusSource?.invoke()}){Text("Close")}}
  Text(model.notice,fontSize=12.sp);Text("Saved working tree and index. Unsaved editor drafts are not included.",fontSize=12.sp)
  model.files.drop(model.page*20).take(20).forEach{file->Text(file.from?.let{"$it → ${file.path}"} ?: file.path);Row{WithDisabledReason(if (model.indexPending) "A Git index update is in progress." else null) { available -> TextButton(enabled = available,onClick={scope.launch{select(file.path,false)}}){Text("Working tree ${file.tree}")} };WithDisabledReason(if (model.indexPending) "A Git index update is in progress." else null) { available -> TextButton(enabled = available,onClick={scope.launch{select(file.path,true)}}){Text("Staged ${file.index}")} }}}
  Row{WithDisabledReason(if (model.page <= 0) "You’re on the first page." else null) { available -> TextButton(enabled = available,onClick={model.page--}){Text("Previous")} };Text("${model.files.size} files · page ${model.page+1}");WithDisabledReason(if ((model.page+1)*20 >= model.files.size) "You’re on the last page." else null) { available -> TextButton(enabled = available,onClick={model.page++}){Text("Next")} }}
  if(model.needsRefresh&&!model.indexPending)Text("Refresh Git state before another index action.",fontSize=12.sp)
  model.selected?.let{path->GitConflictView(workspace,path);Text("$path · ${if(model.staged)"Staged" else "Working tree"}");Row{WithDisabledReason(if (model.indexPending) "A Git index update is in progress." else if (model.needsRefresh) "Refresh repository changes after the previous Git operation." else if (model.files.firstOrNull{it.path==path}?.tree == " ") "The selected file has no saved changes to stage." else null) { available -> TextButton(enabled = available,onClick={scope.launch{changeIndex(path,false)}}){Text("Stage saved file")} };WithDisabledReason(if (model.indexPending) "A Git index update is in progress." else if (model.needsRefresh) "Refresh repository changes after the previous Git operation." else if (!(model.files.firstOrNull{it.path==path}?.index !in listOf(" ","?","!"))) "The selected file has no staged changes to unstage." else null) { available -> TextButton(enabled = available,onClick={scope.launch{changeIndex(path,true)}}){Text("Unstage file")} }};androidx.compose.foundation.text.selection.SelectionContainer{Text(model.detail,fontFamily=FontFamily.Monospace,fontSize=12.sp,modifier=Modifier.horizontalScroll(rememberScrollState()))}}
 }
}
