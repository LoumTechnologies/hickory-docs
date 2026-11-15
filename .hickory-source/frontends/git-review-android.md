<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read saved repository changes through existing menus

For maintainers changing workspace services independently of the editor renderer.

Review saved Git state independently of editor drafts. A small model owns list/diff requests and a monotonic intent: refreshing, changing sides or closing retires older replies. The UI only presents that state in the existing floating surface; no source, save, staging or history command runs during inspection. Keep staged and working-tree columns distinct and bound displayed diff text without presenting a truncated view as complete.

<ui:copy id="simple-git-review-android">
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

data class ReviewFile(val path:String,val from:String?,val index:String,val tree:String)
data class ReviewChanges(val repository:Boolean,val branch:String?,val files:List<ReviewFile>)
data class ReviewDiff(val path:String,val diff:String,val binary:Boolean)
fun reviewChanges(raw:JSONObject):ReviewChanges {val rows=raw.getJSONArray("files");return ReviewChanges(raw.getBoolean("repository"),if(raw.isNull("branch"))null else raw.getString("branch"),(0 until rows.length()).map{val row=rows.getJSONObject(it);ReviewFile(row.getString("path"),if(row.isNull("from"))null else row.getString("from"),row.getString("index"),row.getString("tree"))})}
class GitReviewModel {
 var files by mutableStateOf<List<ReviewFile>>(emptyList());var branch by mutableStateOf("");var selected by mutableStateOf<String?>(null);var staged by mutableStateOf(false);var detail by mutableStateOf("");var notice by mutableStateOf("");var loading by mutableStateOf(false);var page by mutableIntStateOf(0)
 private var ticket=0L
 fun close(){ticket++;loading=false}
 suspend fun refresh(load:suspend ()->ReviewChanges){val id=++ticket;loading=true;files=emptyList();branch="";selected=null;detail="";notice="Reading saved Git state…"
  try{val value=load();if(ticket!=id)return;files=value.files;branch=value.branch.orEmpty();page=0;notice=if(value.repository)"Saved Git state · unsaved editor drafts are not included." else "This workspace is not a Git repository."}catch(error:CancellationException){throw error}catch(error:Exception){if(ticket==id)notice=error.message ?: "Cannot read Git changes. Refresh and retry."}finally{if(ticket==id)loading=false}
 }
 suspend fun select(path:String,staged:Boolean,load:suspend (String,Boolean)->ReviewDiff){if(files.none{it.path==path})return;val id=++ticket;selected=path;this.staged=staged;detail="";loading=true;notice="Reading saved diff…"
  try{val value=load(path,staged);if(ticket!=id)return;check(value.path==path){"Diff identity changed. Refresh repository changes."};detail=value.diff.take(200000);notice=if(value.binary)"Binary change · text comparison unavailable." else if(value.diff.length>200000)"First 200,000 characters shown. Use Terminal for the complete diff." else if(value.diff.isEmpty())"No changes on this side." else "Captured Git diff · refresh after external changes."}catch(error:CancellationException){throw error}catch(error:Exception){if(ticket==id)notice=error.message ?: "Cannot read this diff. Refresh and retry."}finally{if(ticket==id)loading=false}
 }
}
</ui:copy>

## Present the captured reading without moving code

Use the existing floating surface and menu vocabulary. Paging limits the visible file list; staged and working-tree buttons select an explicit side. Keep saved Git state visibly separate from unsaved editor drafts, preserve code focus on close and retire pending requests when the surface disappears. Diff text stays selectable and never becomes an editor mutation.

<ui:copy id="simple-git-review-android-view">
@Composable fun GitReviewView(workspace:Workspace){
 val engine=workspace.engine;val model=remember(engine){GitReviewModel()};val scope=rememberCoroutineScope()
 suspend fun refresh(){if(engine==null){model.notice="Connect to a workspace server first.";return};model.refresh{reviewChanges(engine.request("GET","/api/git/changes"))}}
 suspend fun select(path:String,staged:Boolean){if(engine==null)return;model.select(path,staged){value,index->val raw=engine.request("GET","/api/git/diff?path=${URLEncoder.encode(value,"UTF-8")}&staged=$index");ReviewDiff(raw.getString("path"),raw.getString("diff"),raw.getBoolean("binary"))}}
 LaunchedEffect(engine){refresh()};DisposableEffect(model){onDispose{model.close()}}
 Column(Modifier.widthIn(max=680.dp).heightIn(max=620.dp).padding(16.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(8.dp)){
  Text("Repository changes ${model.branch}",style=MaterialTheme.typography.titleMedium)
  Row{TextButton(onClick={scope.launch{refresh()}}){Text("Refresh")};TextButton(onClick={workspace.showReview=false;workspace.currentBuffer?.focusSource?.invoke()}){Text("Close")}}
  Text(model.notice,fontSize=12.sp);Text("Saved working tree and index. Unsaved editor drafts are not included.",fontSize=12.sp)
  model.files.drop(model.page*20).take(20).forEach{file->Text(file.from?.let{"$it → ${file.path}"} ?: file.path);Row{TextButton(onClick={scope.launch{select(file.path,false)}}){Text("Working tree ${file.tree}")};TextButton(onClick={scope.launch{select(file.path,true)}}){Text("Staged ${file.index}")}}}
  Row{TextButton(enabled=model.page>0,onClick={model.page--}){Text("Previous")};Text("${model.files.size} files · page ${model.page+1}");TextButton(enabled=(model.page+1)*20<model.files.size,onClick={model.page++}){Text("Next")}}
  model.selected?.let{Text("$it · ${if(model.staged)"Staged" else "Working tree"}");androidx.compose.foundation.text.selection.SelectionContainer{Text(model.detail,fontFamily=FontFamily.Monospace,fontSize=12.sp,modifier=Modifier.horizontalScroll(rememberScrollState()))}}
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/GitReview.kt"><ui:paste select="#simple-git-review-android"/><ui:paste select="#simple-git-review-android-view"/></ui:file>
## Reject obsolete and foreign diff replies

Hold two diff replies and release them in reverse order. The newer path and side must remain selected; closing retires a held reply. A mismatched response identity refuses before displaying its text. These are service safety witnesses, independent of popover layout and native physical input.

<ui:copy id="simple-git-review-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class GitReviewTest {
 @Test fun latestSideAndClosedRepliesRetainTheirOwnIdentity()=runBlocking {
  val model=GitReviewModel();model.refresh{ReviewChanges(true,"main",listOf("a.py","b.py").map{ReviewFile(it,null,"M","M")})}
  val held=CompletableDeferred<ReviewDiff>();val first=async(start=CoroutineStart.UNDISPATCHED){model.select("a.py",false){_,_->held.await()}}
  model.select("b.py",true){path,_->ReviewDiff(path,"new staged",false)};held.complete(ReviewDiff("a.py","obsolete",false));first.await()
  assertEquals("b.py",model.selected);assertTrue(model.staged);assertEquals("new staged",model.detail)
  val late=CompletableDeferred<ReviewDiff>();val closed=async(start=CoroutineStart.UNDISPATCHED){model.select("a.py",false){_,_->late.await()}};model.close();late.complete(ReviewDiff("a.py","late after close",false));closed.await();assertEquals("",model.detail)
  model.select("a.py",false){_,_->ReviewDiff("b.py","wrong source",false)};assertEquals("",model.detail);assertTrue(model.notice.contains("identity changed"))
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/GitReviewTest.kt"><ui:paste select="#simple-git-review-android-tests"/></ui:file>
</ui:doc>
