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
 var comparing by remember { mutableStateOf(false) }
 val engine=workspace.engine;val model=workspace.gitHistory;val scope=rememberCoroutineScope();val request=model.request;val path=(request as? GitHistoryRequest.Log)?.path
 suspend fun refresh(count:Int=120){if(engine==null){model.notice="Connect to a workspace server first.";return};if(request is GitHistoryRequest.Commit){model.readCommit(request.sha){historyReading(engine.request("GET","/api/git/reading?sha=$it"))};return};model.refresh(count){count,path->historyLog(engine.request("GET","/api/git/log?limit=$count"+(path?.let{"&path=${URLEncoder.encode(it,"UTF-8")}"} ?: "")))}}
 suspend fun select(sha:String){if(engine==null)return;model.select(sha){historyReading(engine.request("GET","/api/git/reading?sha=$it"))}}
 LaunchedEffect(engine,model,path){refresh()};DisposableEffect(model){onDispose{model.close()}}
 Column(Modifier.widthIn(max=680.dp).heightIn(max=680.dp).padding(16.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(8.dp)){
  Text(if(request is GitHistoryRequest.Commit)request.title else path?.let{"Saved history · $it"} ?: "Repository history",style=MaterialTheme.typography.titleMedium)
  Row{TextButton(onClick={scope.launch{refresh(model.limit)}}){Text("Refresh")};TextButton(onClick={workspace.showHistory=false;workspace.currentBuffer?.focusSource?.invoke()}){Text("Close")}}
  Text(model.notice,fontSize=12.sp)
  if(request is GitHistoryRequest.Log && engine != null) {
      TextButton(onClick = { comparing = !comparing }) { Text(if (comparing) "Browse commits" else "Compare two commits") }
      if (comparing) GitComparisonPicker(workspace, engine)
  }
  if(request is GitHistoryRequest.Log && !comparing){
  model.commits.drop(model.page*20).take(20).forEach{commit->TextButton(onClick={scope.launch{select(commit.sha)}}){Text("${commit.short} · ${commit.subject} · ${commit.author}")}}
  Row{WithDisabledReason(if (model.page <= 0) "You’re on the first page." else null) { available -> TextButton(enabled = available,onClick={model.page--}){Text("Previous")} };Text("Page ${model.page+1}");WithDisabledReason(if ((model.page+1)*20 >= model.commits.size) "You’re on the last page." else null) { available -> TextButton(enabled = available,onClick={model.page++}){Text("Next")} }}
  WithDisabledReason(if (model.limit >= 1000) "The history view has reached its 1,000-commit limit." else if (model.commits.size < model.limit) "There are no older commits to load." else null) { available -> TextButton(enabled = available,onClick={scope.launch{refresh(minOf(1000,model.limit+120))}}){Text("Load more commits")} }
  }
  model.reading?.takeUnless { comparing }?.let{reading->SelectionContainer{Text(reading.sha+"\n"+reading.message.take(20000),fontSize=12.sp)};if(reading.message.length>20000)Text("First 20,000 message characters shown.",fontSize=12.sp)
   if(reading.files.isNotEmpty()){Row{WithDisabledReason(if (model.file <= 0) "You’re on the first page." else null) { available -> TextButton(enabled = available,onClick={model.file--}){Text("Previous file")} };WithDisabledReason(if (model.file+1 >= reading.files.size) "Wait for the current reading to load." else null) { available -> TextButton(enabled = available,onClick={model.file++}){Text("Next file")} }}
    val file=reading.files[model.file];Text("${file.status} · ${file.path}");Row{TextButton(onClick={model.before=true}){Text("Before commit")};TextButton(onClick={model.before=false}){Text("After commit")}};val content=if(model.before)file.before else file.after
    Text(if(file.binary)"Binary content · text reading unavailable." else if(content.length>200000)"First 200,000 characters shown." else if(content.isEmpty())"No text on this side." else "Immutable file content.",fontSize=12.sp)
    SelectionContainer{Text(if(file.binary)"" else content.take(200000),fontFamily=FontFamily.Monospace,fontSize=12.sp,modifier=Modifier.horizontalScroll(rememberScrollState()))}
   }else Text("No files under this workspace in this commit.")
  }
 }
}
data class EditorGitComparison(
    val path: String, val base: String, val target: String, val before: String, val after: String,
    val patch: String, val status: String, val binary: Boolean, val truncated: Boolean,
)
data class GitChangedLine(val removed: MutableList<String> = mutableListOf(), var added: String? = null)

internal fun editorGitLines(before: String, draft: String): Map<Int, GitChangedLine>? {
    if (before.length > 200_000 || draft.length > 200_000) return null
    if (before == draft) return emptyMap()
    fun rows(source: String): List<String> {
        val pieces = source.split('\n')
        return pieces.mapIndexedNotNull { index, row ->
            if (index < pieces.lastIndex) "$row\n" else row.takeIf { it.isNotEmpty() }
        }
    }
    val beforeRows = rows(before)
    val draftRows = rows(draft)
    var first = 0
    var oldEnd = beforeRows.size
    var newEnd = draftRows.size
    while (first < oldEnd && first < newEnd && beforeRows[first] == draftRows[first]) first++
    while (oldEnd > first && newEnd > first && beforeRows[oldEnd - 1] == draftRows[newEnd - 1]) {
        oldEnd--; newEnd--
    }
    val old = beforeRows.subList(first, oldEnd)
    val new = draftRows.subList(first, newEnd)
    val width = new.size + 1
    if ((old.size + 1L) * width > 4_000_000) return null
    val lengths = IntArray((old.size + 1) * width)
    for (i in old.indices.reversed()) for (j in new.indices.reversed()) {
        lengths[i * width + j] = if (old[i] == new[j]) 1 + lengths[(i + 1) * width + j + 1]
        else maxOf(lengths[(i + 1) * width + j], lengths[i * width + j + 1])
    }
    val changes = mutableMapOf<Int, GitChangedLine>()
    val eof = draft.count { it == '\n' }
    var i = 0
    var j = 0
    while (i < old.size || j < new.size) {
        if (i < old.size && j < new.size && old[i] == new[j]) { i++; j++; continue }
        val row = changes.getOrPut(minOf(first + j, eof)) { GitChangedLine() }
        if (j == new.size || i < old.size && lengths[(i + 1) * width + j] >= lengths[i * width + j + 1]) {
            row.removed.add(old[i++].removeSuffix("\n").removeSuffix("\r"))
        } else row.added = new[j++].removeSuffix("\n").removeSuffix("\r")
    }
    return changes
}

@Composable private fun GitComparisonPicker(workspace: Workspace, engine: Engine) {
    val scope = rememberCoroutineScope()
    val epoch = workspace.sourceNavigationEpoch
    var from by remember { mutableStateOf("") }
    var to by remember { mutableStateOf("") }
    var listing by remember { mutableStateOf<JSONObject?>(null) }
    var selected by remember { mutableIntStateOf(0) }
    var reading by remember { mutableStateOf<EditorGitComparison?>(null) }
    var busy by remember { mutableStateOf(false) }
    var notice by remember { mutableStateOf("") }
    var ticket by remember { mutableLongStateOf(0) }
    fun current(id: Long) = workspace.engine === engine && workspace.sourceNavigationEpoch == epoch && ticket == id
    fun retire() { ticket++; listing = null; reading = null; selected = 0; busy = false; notice = "" }
    fun query(base: String, target: String) = "/api/git/reading?base=${URLEncoder.encode(base,"UTF-8")}&sha=${URLEncoder.encode(target,"UTF-8")}"
    val sha = Regex("[0-9a-f]{40}|[0-9a-f]{64}")
    val files = listing?.optJSONArray("files")
    OutlinedTextField(from, { from = it; retire() }, label = { Text("Base commit") }, singleLine = true)
    OutlinedTextField(to, { to = it; retire() }, label = { Text("Target commit") }, singleLine = true)
    WithDisabledReason(if (busy) "An operation is already in progress." else if (!(from.isNotBlank())) "Enter the base Git revision." else if (!(to.isNotBlank())) "Enter the target Git revision." else null) { available -> TextButton(enabled = available, onClick = {
        retire()
        val id = ticket
        val address = query(from.trim(), to.trim())
        busy = true
        scope.launch {
            try {
                val reply = engine.request("GET", address)
                if (!current(id)) return@launch
                check(sha.matches(reply.getString("parent")) && sha.matches(reply.getString("sha"))) { "The comparison did not return two commit identities." }
                listing = reply
            } catch (error: CancellationException) { throw error }
            catch (error: Exception) { if (current(id)) notice = error.message.orEmpty() }
            finally { if (current(id)) busy = false }
        }
    }) { Text("Read comparison") } }
    if (files != null && files.length() == 0) Text("The selected commits have no changes in this workspace.")
    if (files != null && selected < files.length()) {
        val file = files.getJSONObject(selected)
        val path = file.getString("path")
        Text("${file.getString("status")} · $path", fontSize = 12.sp)
        Row {
            WithDisabledReason(if (selected <= 0) "You’re on the first page." else null) { available -> TextButton(enabled = available, onClick = { ticket++; selected--; reading = null; busy = false }) { Text("Previous file") } }
            WithDisabledReason(if (selected+1 >= files.length()) "You’re on the last page." else null) { available -> TextButton(enabled = available, onClick = { ticket++; selected++; reading = null; busy = false }) { Text("Next file") } }
        }
        WithDisabledReason(if (busy) "An operation is already in progress." else null) { available -> TextButton(enabled = available, onClick = {
            val list = listing ?: return@TextButton
            val base = list.getString("parent")
            val target = list.getString("sha")
            val id = ++ticket
            busy = true; notice = ""; reading = null
            scope.launch {
                try {
                    val reply = engine.request("GET", query(base,target) + "&path=" + URLEncoder.encode(path,"UTF-8"))
                    if (!current(id)) return@launch
                    val found = reply.getJSONArray("files")
                    check(reply.getString("parent") == base && reply.getString("sha") == target &&
                        found.length() == 1 && found.getJSONObject(0).getString("path") == path) { "The patch does not match the selected commits and file." }
                    val patch = found.getJSONObject(0)
                    reading = EditorGitComparison(path, base, target, patch.getString("before"), patch.getString("after"),
                        patch.getString("patch"), patch.getString("status"), patch.getBoolean("binary"), patch.getBoolean("patch_truncated"))
                } catch (error: CancellationException) { throw error }
                catch (error: Exception) { if (current(id)) notice = error.message.orEmpty() }
                finally { if (current(id)) busy = false }
            }
        }) { Text("Read file patch") } }
    }
    reading?.let { comparison ->
        Text("${comparison.base.take(12)} → ${comparison.target.take(12)}", fontSize = 12.sp)
        if (comparison.binary) Text("Binary change") else {
            if (comparison.truncated) Text("First 200,000 patch characters shown.")
            SelectionContainer {
                Text(comparison.patch, fontFamily = FontFamily.Monospace, fontSize = 12.sp,
                    modifier = Modifier.heightIn(max = 230.dp).verticalScroll(rememberScrollState()).horizontalScroll(rememberScrollState()))
            }
            WithDisabledReason(if (busy) "An operation is already in progress." else null) { available -> TextButton(enabled = available, onClick = {
                val id = ticket
                busy = true
                scope.launch {
                    try {
                        if (workspace.openGitComparison(comparison) { current(id) && reading === comparison }) workspace.showHistory = false
                        else if (current(id)) notice = "The working file could not be opened. Check Files and try again."
                    } finally { if (current(id)) busy = false }
                }
            }) { Text("Open working file with comparison") } }
        }
    }
    if (notice.isNotEmpty()) Text(notice, fontSize = 12.sp)
}

@Composable internal fun GitComparisonLine(buffer: DocumentBuffer, comparison: EditorGitComparison, line: GitChangedLine?) {
    var showPatch by remember { mutableStateOf(false) }
    var notice by remember { mutableStateOf("") }
    Text("Base ${comparison.base.take(12)} → working draft")
    if (buffer.text == comparison.after) Text("Draft matches target ${comparison.target.take(12)}.", fontSize = 12.sp)
    else if (!buffer.dirty && !buffer.readOnly && !buffer.saving && !buffer.resourceDeleted && !comparison.status.startsWith("D")) {
        TextButton(onClick = {
            val source = buffer.text
            if (buffer.gitComparison === comparison && buffer.file.path == comparison.path && !buffer.dirty && !buffer.readOnly && !buffer.saving) {
                if (!buffer.applySourceEdits(listOf(NativeSourceEdit(0, source.length, comparison.after)), source)) {
                    notice = "The editor could not start the target draft."
                }
            }
        }) { Text("Use target ${comparison.target.take(12)} as draft") }
    }
    if (line != null) SelectionContainer {
        Text((line.removed.map { "− $it" } + listOfNotNull(line.added?.let { "+ $it" })).joinToString("\n"),
            fontFamily = FontFamily.Monospace, fontSize = 12.sp)
    }
    TextButton(onClick = { showPatch = !showPatch }) { Text(if (showPatch) "Hide commit patch" else "Show commit patch") }
    if (showPatch) SelectionContainer { Text(comparison.patch, Modifier.heightIn(max = 180.dp).verticalScroll(rememberScrollState()), fontFamily = FontFamily.Monospace, fontSize = 12.sp) }
    if (notice.isNotEmpty()) Text(notice)
    TextButton(onClick = { buffer.gitComparison = null; buffer.focusSource?.invoke() }) { Text("Close comparison") }
}
