<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read immutable repository history without replacing the editor

For maintainers changing workspace services independently of the editor renderer.

History is a read-only capability over saved Git facts. Capture the requested workspace/path and request intent independently of the active editor. Retain full commit identities from the log, require an exact identity match when fetching immutable before/after blobs, and retire older or closed replies. Local paging and a bounded log window keep the floating reading compact. No checkout, save, index or source mutation occurs.

<ui:copy id="simple-git-history-android">
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

data class HistoryCommit(val sha:String,val short:String,val author:String,val subject:String)
data class HistoryLog(val repository:Boolean,val commits:List<HistoryCommit>)
data class HistoryFile(val path:String,val status:String,val before:String,val after:String,val binary:Boolean)
data class HistoryReading(val sha:String,val message:String,val files:List<HistoryFile>)
fun historyLog(raw:JSONObject):HistoryLog{val rows=raw.getJSONArray("commits");return HistoryLog(raw.getBoolean("repository"),(0 until rows.length()).map{val row=rows.getJSONObject(it);HistoryCommit(row.getString("sha"),row.getString("short"),row.getString("author"),row.getString("subject"))})}
fun historyReading(raw:JSONObject):HistoryReading{val rows=raw.getJSONArray("files");return HistoryReading(raw.getString("sha"),raw.getString("message"),(0 until rows.length()).map{val row=rows.getJSONObject(it);HistoryFile(row.getString("path"),row.getString("status"),row.getString("before"),row.getString("after"),row.getBoolean("binary"))})}
class GitHistoryModel {
 var commits by mutableStateOf<List<HistoryCommit>>(emptyList());var reading by mutableStateOf<HistoryReading?>(null);var file by mutableIntStateOf(0);var before by mutableStateOf(false);var page by mutableIntStateOf(0);var limit by mutableIntStateOf(120);var notice by mutableStateOf("");var loading by mutableStateOf(false);private var ticket=0L
 fun close(){ticket++;loading=false;reading=null;notice=""}
 suspend fun refresh(path:String?,limit:Int=120,load:suspend (Int,String?)->HistoryLog){val id=++ticket;this.limit=limit.coerceIn(1,1000);commits=emptyList();reading=null;page=0;loading=true;notice="Reading saved history…"
  try{val value=load(this.limit,path);if(ticket!=id)return;check(value.commits.all{it.sha.matches(Regex("([0-9a-f]{40}|[0-9a-f]{64})"))}){"Malformed commit identity. Refresh repository history."};commits=value.commits;notice=if(value.repository)"${commits.size} reported commits · saved history only." else "This workspace is not a Git repository."}catch(error:CancellationException){throw error}catch(error:Exception){if(ticket==id)notice=error.message.orEmpty()}finally{if(ticket==id)loading=false}
 }
 suspend fun select(sha:String,load:suspend (String)->HistoryReading){if(commits.none{it.sha==sha})return;val id=++ticket;reading=null;loading=true;notice="Reading immutable commit…"
  try{val value=load(sha);if(ticket!=id)return;check(value.sha==sha){"Commit identity changed. Refresh repository history."};reading=value;file=0;before=false;notice="Immutable commit reading · current editor draft is retained."}catch(error:CancellationException){throw error}catch(error:Exception){if(ticket==id)notice=error.message.orEmpty()}finally{if(ticket==id)loading=false}
 }
}
</ui:copy>

## Present immutable file sides without replacing live source

The existing floating surface pages commit choices and reads exact before/after blobs. Historical file selection is local to the reading; it does not activate, reload or edit the source buffer. Render text literally, retain binary and empty-side distinctions, and report display limits. Closing returns focus and retires outstanding reads. A fresh request context keeps its title and data together: browser models are keyed by workspace/path, and native menu adapters replace the read session before presenting a different request. Closed readings clear their visible content before reuse.

<ui:copy id="simple-git-history-android-view">
@Composable fun GitHistoryView(workspace:Workspace){
 val engine=workspace.engine;val model=workspace.gitHistory;val scope=rememberCoroutineScope();val path=workspace.historyPath
 suspend fun refresh(count:Int=120){if(engine==null){model.notice="Connect to a workspace server first.";return};model.refresh(path,count){count,path->historyLog(engine.request("GET","/api/git/log?limit=$count"+(path?.let{"&path=${URLEncoder.encode(it,"UTF-8")}"} ?: "")))}}
 suspend fun select(sha:String){if(engine==null)return;model.select(sha){historyReading(engine.request("GET","/api/git/reading?sha=$it"))}}
 LaunchedEffect(engine,model,path){refresh()};DisposableEffect(model){onDispose{model.close()}}
 Column(Modifier.widthIn(max=680.dp).heightIn(max=680.dp).padding(16.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(8.dp)){
  Text(path?.let{"Saved history · $it"} ?: "Repository history",style=MaterialTheme.typography.titleMedium)
  Row{TextButton(onClick={scope.launch{refresh(model.limit)}}){Text("Refresh")};TextButton(onClick={workspace.showHistory=false;workspace.currentBuffer?.focusSource?.invoke()}){Text("Close")}}
  Text(model.notice,fontSize=12.sp)
  model.commits.drop(model.page*20).take(20).forEach{commit->TextButton(onClick={scope.launch{select(commit.sha)}}){Text("${commit.short} · ${commit.subject} · ${commit.author}")}}
  Row{TextButton(enabled=model.page>0,onClick={model.page--}){Text("Previous")};Text("Page ${model.page+1}");TextButton(enabled=(model.page+1)*20<model.commits.size,onClick={model.page++}){Text("Next")}}
  TextButton(enabled=model.limit<1000&&model.commits.size>=model.limit,onClick={scope.launch{refresh(minOf(1000,model.limit+120))}}){Text("Load more commits")}
  model.reading?.let{reading->SelectionContainer{Text(reading.sha+"\n"+reading.message.take(20000),fontSize=12.sp)};if(reading.message.length>20000)Text("First 20,000 message characters shown.",fontSize=12.sp)
   if(reading.files.isNotEmpty()){Row{TextButton(enabled=model.file>0,onClick={model.file--}){Text("Previous file")};TextButton(enabled=model.file+1<reading.files.size,onClick={model.file++}){Text("Next file")}}
    val file=reading.files[model.file];Text("${file.status} · ${file.path}");Row{TextButton(onClick={model.before=true}){Text("Before commit")};TextButton(onClick={model.before=false}){Text("After commit")}};val content=if(model.before)file.before else file.after
    Text(if(file.binary)"Binary content · text reading unavailable." else if(content.length>200000)"First 200,000 characters shown." else if(content.isEmpty())"No text on this side." else "Immutable file content.",fontSize=12.sp)
    SelectionContainer{Text(if(file.binary)"" else content.take(200000),fontFamily=FontFamily.Monospace,fontSize=12.sp,modifier=Modifier.horizontalScroll(rememberScrollState()))}
   }else Text("No files under this workspace in this commit.")
  }
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/GitHistory.kt"><ui:paste select="#simple-git-history-android"/><ui:paste select="#simple-git-history-android-view"/></ui:file>
## Keep immutable readings bound to exact commit identities

Hold an immutable reading while selecting another commit, then release it after the newer reply. Exact full identities, closed-view retirement and malformed log identities must prevent a foreign or obsolete reading from replacing the selected commit. These service witnesses do not assert native pointer or accessibility acceptance.

<ui:copy id="simple-git-history-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class GitHistoryTest {
 @Test fun heldAndClosedReadingsRetainTheSelectedCommitIdentity()=runBlocking {
  val model=GitHistoryModel();val a="a".repeat(40);val b="b".repeat(40)
  model.refresh("folder/é.md"){_,_->HistoryLog(true,listOf(a,b).map{HistoryCommit(it,it.take(7),"fixture","saved commit")})}
  val pending=CompletableDeferred<HistoryReading>();val first=async(start=CoroutineStart.UNDISPATCHED){model.select(a){pending.await()}}
  model.select(b){HistoryReading(it,"new",emptyList())};pending.complete(HistoryReading(a,"obsolete",emptyList()));first.await();assertEquals(b,model.reading?.sha);assertEquals("new",model.reading?.message)
  val late=CompletableDeferred<HistoryReading>();val closed=async(start=CoroutineStart.UNDISPATCHED){model.select(a){late.await()}};model.close();late.complete(HistoryReading(a,"closed",emptyList()));closed.await();assertNull(model.reading)
  model.select(a){HistoryReading(b,"foreign",emptyList())};assertNull(model.reading);assertTrue(model.notice.contains("identity changed"))
 }
 @Test fun malformedLogIdentityNeverBecomesAReadingChoice()=runBlocking {
  val model=GitHistoryModel();model.refresh(null,2000){_,_->HistoryLog(true,listOf(HistoryCommit("HEAD","HEAD","fixture","ambiguous")))};assertEquals(1000,model.limit);assertTrue(model.commits.isEmpty());assertTrue(model.notice.contains("Malformed commit identity"))
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/GitHistoryTest.kt"><ui:paste select="#simple-git-history-android-tests"/></ui:file>
</ui:doc>
