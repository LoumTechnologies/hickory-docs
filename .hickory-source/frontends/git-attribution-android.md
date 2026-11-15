<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Inspect saved attribution without borrowing draft positions

For maintainers changing workspace services independently of the editor renderer.

Saved-file attribution is a captured reading, not a claim that line positions match an unsaved editor draft. Keep its requested path, reported line text, author, provider and immutable commit reference together. Reject foreign paths, repeated line identities and obsolete replies before presentation. Uncommitted lines and older providers without a full reference remain readable; only a valid reported full identity can open an immutable commit reading. This capability performs no source or index mutation.

<ui:copy id="simple-git-attribution-android">
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
</ui:copy>

## Present saved facts and reuse immutable commit reading

Use the existing floating menu surface, retaining the captured path in its heading. Report saved line numbers and provider labels without anchoring them onto dirty draft positions. Only a current row with a full committed identity can construct a typed commit request for the existing historical reader. Paging and explicit line-text limits keep inspection compact; closing restores code focus and retires the request.

<ui:copy id="simple-git-attribution-android-view">
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
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/GitAttribution.kt"><ui:paste select="#simple-git-attribution-android"/><ui:paste select="#simple-git-attribution-android-view"/></ui:file>
## Reject stale attribution and ambiguous references

Hold attribution replies, replace their request, and close the reading before releasing old work. Foreign paths and repeated line identities must refuse without presenting a mixed snapshot. Only a valid full reference on a committed line admits immutable reading; abbreviated, missing and uncommitted references remain display facts.

<ui:copy id="simple-git-attribution-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class GitAttributionTest {
 private fun row(line:Int,full:String?=null)=AttributionLine(line,"12345678",full,"Fixture","base",false,null,"saved")
 @Test fun capturedPathsAndLineIdentitiesRetireObsoleteReplies()=runBlocking {
  val model=GitAttributionModel("a.py");val pending=CompletableDeferred<AttributionReading>();val old=async(start=CoroutineStart.UNDISPATCHED){model.refresh{pending.await()}}
  model.refresh{AttributionReading(it,listOf(row(2,"a".repeat(40))))};pending.complete(AttributionReading("a.py",listOf(row(1))));old.await();assertEquals(listOf(2),model.lines.map{it.line})
  val late=CompletableDeferred<AttributionReading>();val closed=async(start=CoroutineStart.UNDISPATCHED){model.refresh{late.await()}};model.close();late.complete(AttributionReading("a.py",listOf(row(1))));closed.await();assertTrue(model.lines.isEmpty())
  val foreign=GitAttributionModel("cafe\u0301.py");foreign.refresh{AttributionReading("caf\u00e9.py",listOf(row(1)))};assertTrue(foreign.lines.isEmpty());assertTrue(foreign.notice.contains("identity changed"))
  model.refresh{AttributionReading(it,listOf(row(1),row(1)))};assertTrue(model.lines.isEmpty());assertTrue(model.notice.contains("Ambiguous"))
 }
 @Test fun compactLabelsAndDirtyLinesAreNotImmutableReferences(){assertNull(row(1).immutableCommit);assertNull(row(1,"12345678").immutableCommit);assertNull(row(1,"a".repeat(40)).copy(uncommitted=true).immutableCommit);assertEquals("a".repeat(40),row(1,"a".repeat(40)).immutableCommit);assertEquals("b".repeat(64),row(1,"b".repeat(64)).immutableCommit)}
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/GitAttributionTest.kt"><ui:paste select="#simple-git-attribution-android-tests"/></ui:file>
</ui:doc>
