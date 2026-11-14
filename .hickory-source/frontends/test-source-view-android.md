<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Open source tests without moving code on android

For maintainers changing test interaction independently of parsing and execution.

Retain the observation and test-run identities independently of visibility. Refresh is read-only. A request ticket and current source snapshot reject obsolete responses; another read cannot install facts after source or workspace replacement.

<ui:copy id="source-test-view-android-0">
package com.hickorydocs.simple
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.json.JSONObject
import java.util.UUID
class TestSourcesModel {
    val identity=UUID.randomUUID().toString()
    var entry by mutableStateOf("");var query by mutableStateOf("");var page by mutableIntStateOf(0)
    var reading by mutableStateOf(false);var running by mutableStateOf(false);var notice by mutableStateOf("")
    var observation by mutableStateOf<Pair<TestSourceSnapshot,SourceTestScan>?>(null)
    data class Run(val id:String,val name:String,val scope:String,val state:String,val exit:Int?,val output:String="",val sourceHash:String,val fingerprint:String,val contributors:Map<String,String>)
    var runs by mutableStateOf<List<Run>>(emptyList());var selected by mutableStateOf<String?>(null)
    private var ticket=UUID.randomUUID();private val outputTickets=mutableMapOf<String,UUID>()
    fun snapshot(workspace:Workspace):TestSourceSnapshot? {
        val buffer=workspace.currentBuffer ?: return null;if(workspace.engine==null)return null
        return TestSourceSnapshot(identity,TestSourceRequest(buffer.file.path,buffer.text,entry.trim().takeIf {it.isNotEmpty()},workspace.buffers.values.filter {!it.readOnly}.associate {it.file.path to it.text}))
    }
    fun current(workspace:Workspace):Boolean {val (before,facts)=observation ?: return false;val now=snapshot(workspace) ?: return false;return testSourceCurrent(before,facts,now)}
    suspend fun refresh(workspace:Workspace) {
        val before=snapshot(workspace) ?: run {notice="Open a source file first.";return};val engine=workspace.engine ?: return
        val id=UUID.randomUUID();ticket=id;reading=true;observation=null;notice=""
        try {val facts=SourceTestScan.decode(engine.request("POST","/api/tests/source-candidates",before.request.body()))
            if(ticket!=id||workspace.testSources!==this)return
            val now=snapshot(workspace);if(now==null||!testSourceCurrent(before,facts,now)){notice="Source changed during inspection. Refresh candidates.";return}
            observation=before to facts;page=0
        }catch(error:Exception){if(ticket==id&&workspace.testSources===this)notice=error.message.orEmpty()}
        finally {if(ticket==id)reading=false}
    }
</ui:copy>

Execution requires the model to construct an explicit saved-source request. Prevent overlapping run clicks, retain the returned terminal identity, and refresh only a user-selected test run. Exit state and terminal output remain distinct from structured per-case results.

<ui:copy id="source-test-view-android-1">
    suspend fun run(candidate:SourceTestCandidate,selection:TestSourceSelection,workspace:Workspace) {
        if(running)return;val (before,facts)=observation ?: return;val now=snapshot(workspace) ?: return;val engine=workspace.engine ?: return
        running=true;notice=""
        try {val reply=engine.request("POST","/api/tests/run-source",sourceTestRun(before,facts,now,candidate.id,selection))
            val id=reply.getString("id");runs=listOf(Run(id,candidate.name,if(selection==TestSourceSelection.NAME_FILTER) "name filter" else candidate.runScope,reply.getString("state"),if(reply.isNull("exit_code")) null else reply.getInt("exit_code"),sourceHash=reply.getString("source_hash"),fingerprint=reply.getString("analysis_fingerprint"),contributors=reply.getJSONObject("checked_sources").let {hashes->hashes.keys().asSequence().associateWith {hashes.getString(it)}}))+runs;selected=id
            notice="Run admitted against saved bytes. Files remain mutable while it executes."
        }catch(error:Exception){notice=error.message.orEmpty()}finally{running=false}
    }
    suspend fun refreshRun(id:String,workspace:Workspace) {
        val read=UUID.randomUUID();outputTickets[id]=read
        val engine=workspace.engine ?: return
        try {val sessions=engine.request("GET","/api/terminals").getJSONArray("sessions")
            val screen=engine.request("GET","/api/terminals/$id/output?view=screen&offset=0&limit=16384")
            val summary=(0 until sessions.length()).map {sessions.getJSONObject(it)}.firstOrNull {it.getString("id")==id}
            val bytes=screen.getJSONArray("bytes");val output=ByteArray(bytes.length()){bytes.getInt(it).toByte()}.toString(Charsets.UTF_8)+(if(screen.isNull("next")) "" else "\n[Screen page truncated. Open terminal for retained output.]")
            if(outputTickets[id]!=read||workspace.testSources!==this)return
            runs=runs.map {if(it.id!=id) it else it.copy(state=summary?.getString("state") ?: "session unavailable",exit=if(summary==null||summary.isNull("exit_code")) null else summary.getInt("exit_code"),output=output)}
        }catch(error:Exception){if(outputTickets[id]==read&&workspace.testSources===this)notice=error.message.orEmpty()}
    }
}
</ui:copy>

A compiler-reported source point is precise only while its SHA-256 matches the current loaded buffer. Recheck after asynchronous loading and pass the exact source through existing guarded navigation. Reading an unopened destination does not activate or replace the current editor.

<ui:copy id="source-test-view-android-2">
suspend fun Workspace.navigateTestPoint(point:TestSourcePoint) {
    fun find(files:List<FileNode>):FileNode? {for(file in files){if(file.path==point.path)return file;find(file.children)?.let {return it}};return null}
    val file=(buffers[point.path]?.file ?: find(rootFiles))?.takeUnless {it.dir} ?: run {testSources.notice="Source no longer listed. Refresh files and candidates.";return}
    val model=testSources
    try {val buffer=loadBuffer(file);val source=buffer.text
        val hash=kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Default){java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}}
        if(testSources!==model||buffer.text!=source||hash!=point.hash.lowercase()){model.notice="Source changed. Refresh candidates before navigating.";return}
        openSemanticLocation(SemanticLocation(languageURI(point.path),point.line-1,point.character),allowDirty=true,expectedSource=source)
        if(testSources===model)showTests=false
    }catch(error:Exception){model.notice=error.message.orEmpty()}
}
</ui:copy>

Render paged candidates, explicit name-filter/containing-scope choices, partial/unsupported notices and retained output. The floating surface leaves editor geometry unchanged. Closing it does not stop a runner or discard the observed terminal identity. No automatic runner discovery or result-status inference is hidden in rendering.

<ui:copy id="source-test-view-android-3">
@Composable fun TestSourcesView(workspace:Workspace) {
    val model=workspace.testSources;val valid=model.current(workspace)
    val candidates=(model.observation?.second?.candidates ?: emptyList()).filter {"${it.name} ${it.target.path}".contains(model.query,ignoreCase=true)}
    val last=maxOf(0,(candidates.size-1)/100);val page=minOf(model.page,last)
    Column(Modifier.widthIn(min=280.dp,max=760.dp).heightIn(min=240.dp,max=760.dp).verticalScroll(rememberScrollState()).padding(12.dp),verticalArrangement=Arrangement.spacedBy(8.dp)) {
        Row {Text("Source tests",fontSize=16.sp);Spacer(Modifier.weight(1f));TextButton(onClick={workspace.showTests=false;workspace.currentBuffer?.focusSource?.invoke()}){Text("Close")}}
        Text("Source declarations are candidates. Name filters may match several cases; parameterized cases need runner discovery. Terminal runs are separate from per-case verification.",fontSize=12.sp)
        Text(workspace.currentBuffer?.file?.path ?: "No source selected",fontSize=12.sp)
        OutlinedTextField(model.entry,{model.entry=it},label={Text("Construction entry (optional)")},singleLine=true)
        TextButton(enabled=!model.reading&&workspace.currentBuffer!=null,onClick={workspace.runCommand {model.refresh(workspace)}}){Text(if(model.reading) "Reading source…" else "Refresh candidates")}
        model.observation?.second?.let {facts->
            Text("${if(valid) "Source observation" else "Stale observation — refresh before running"} · ${facts.scope} · ${if(facts.syntaxComplete) "parsed" else "partial syntax"}",fontSize=12.sp)
            (facts.notices+facts.unsupportedLanguages.map {"No source detector: $it"}+facts.unclassifiedOutputs.map {"Unclassified output: $it"}).forEach {Text(it,fontSize=12.sp)}
        }
        if(model.notice.isNotEmpty())Text(model.notice,fontSize=12.sp)
        OutlinedTextField(model.query,{model.query=it;model.page=0},label={Text("Filter source tests")},singleLine=true)
        candidates.drop(page*100).take(100).forEach {item->
            Text(item.name);Text("${item.confidence} · ${item.target.path}:${item.target.line}",fontSize=12.sp)
            TextButton(enabled=valid,onClick={workspace.runCommand {workspace.navigateTestPoint(item.point)}}){Text("${if(item.source==null) "Open runtime source" else "Open authored source"} ${item.point.line}")}
            TextButton(enabled=valid&&!model.running&&item.nameFilter!=null,onClick={workspace.runCommand {model.run(item,TestSourceSelection.NAME_FILTER,workspace)}}){Text("Run name filter")}
            TextButton(enabled=valid&&!model.running,onClick={workspace.runCommand {model.run(item,TestSourceSelection.CONTAINING_SCOPE,workspace)}}){Text("Run ${item.runScope.replace('_',' ')} scope")}
        }
        if(model.observation!=null&&candidates.isEmpty())Text("No source candidates returned. This does not prove the project has no tests.",fontSize=12.sp)
        if(last>0)Row {TextButton(enabled=page>0,onClick={model.page=page-1}){Text("Previous")};Text("${page+1}/${last+1}");TextButton(enabled=page<last,onClick={model.page=page+1}){Text("Next")}}
        HorizontalDivider();Text("Retained test runs (${model.runs.size})")
        model.runs.forEach {run->TextButton(onClick={model.selected=run.id;workspace.runCommand {model.refreshRun(run.id,workspace)}}){Text("${run.name} · ${run.scope} · ${run.state}${run.exit?.let {" · exit $it"} ?: ""}")}}
        model.runs.firstOrNull {it.id==model.selected}?.let {run->
            TextButton(onClick={workspace.runCommand {model.refreshRun(run.id,workspace)}}){Text("Refresh this run")}
            TextButton(onClick={workspace.terminal.id=run.id;workspace.terminal.error="";workspace.show("terminal");workspace.showTests=false}){Text("Open terminal output")}
            var showRevisions by remember(run.id) {mutableStateOf(false)}
            TextButton(onClick={showRevisions=!showRevisions}){Text("Saved revisions checked at admission")}
            if(showRevisions)Text("Files were not frozen during execution. These revisions identify this run, not current verification.\n${run.fingerprint}\nTarget: ${run.sourceHash}\n"+run.contributors.entries.joinToString("\n"){"${it.key}: ${it.value}"},fontSize=12.sp)
            androidx.compose.foundation.text.selection.SelectionContainer {Text(run.output.ifEmpty {"Select or refresh this run to read its output."},fontSize=12.sp,fontFamily=androidx.compose.ui.text.font.FontFamily.Monospace)}
        }
    }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/TestSourcesView.kt"><ui:paste select="#source-test-view-android-0"/><ui:paste select="#source-test-view-android-1"/><ui:paste select="#source-test-view-android-2"/><ui:paste select="#source-test-view-android-3"/></ui:file>
</ui:doc>
