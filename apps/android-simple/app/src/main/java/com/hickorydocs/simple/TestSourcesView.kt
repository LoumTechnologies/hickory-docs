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
    private val debugChoices=TestDebugChoices()
    private var debugTicket=UUID.randomUUID()
    private var pointNavigation = 0L
    internal fun retirePointNavigation() { pointNavigation++ }
    internal fun beginPointNavigation(): () -> Boolean {
        val intent = ++pointNavigation
        return { pointNavigation == intent }
    }
    var debugging by mutableStateOf(false);private set
    data class DebugRun(val id:String,val name:String,val candidateID:String,val inputPath:String,val inputHash:String,val framework:String,val selection:String,val fingerprint:String,val contributors:Map<String,String>,val state:String="running",val exit:Int?=null,val output:String="")
    var debugRuns by mutableStateOf<List<DebugRun>>(emptyList());private set
    fun closeDebugPreparation(){debugTicket=UUID.randomUUID();debugChoices.cancel();debugging=false}
    private var ticket=UUID.randomUUID();private val outputTickets=mutableMapOf<String,UUID>()
    fun snapshot(workspace:Workspace):TestSourceSnapshot? {
        val buffer=workspace.currentBuffer ?: return null;if(workspace.engine==null)return null
        return TestSourceSnapshot(identity,TestSourceRequest(buffer.file.path,buffer.text,entry.trim().takeIf {it.isNotEmpty()},workspace.buffers.values.filter {!it.readOnly}.associate {it.file.path to it.text}))
    }
    fun current(workspace:Workspace):Boolean {val (before,facts)=observation ?: return false;val now=snapshot(workspace) ?: return false;return testSourceCurrent(before,facts,now)}
    suspend fun refresh(workspace:Workspace) {
        val before=snapshot(workspace) ?: run {notice="Open a source file first.";return};val engine=workspace.engine ?: return
        closeDebugPreparation();val id=UUID.randomUUID();ticket=id;reading=true;observation=null;notice=""
        try {val facts=SourceTestScan.decode(engine.request("POST","/api/tests/source-candidates",before.request.body()))
            if(ticket!=id||workspace.testSources!==this)return
            val now=snapshot(workspace);if(now==null||!testSourceCurrent(before,facts,now)){notice="Source changed during inspection. Refresh candidates.";return}
            observation=before to facts;page=0
        }catch(error:Exception){if(ticket==id&&workspace.testSources===this)notice=error.message.orEmpty()}
        finally {if(ticket==id)reading=false}
    }
    suspend fun run(candidate:SourceTestCandidate,selection:TestSourceSelection,workspace:Workspace) {
        if(running)return;val (before,facts)=observation ?: return;val now=snapshot(workspace) ?: return;val engine=workspace.engine ?: return
        running=true;notice=""
        try {val reply=engine.request("POST","/api/tests/run-source",sourceTestRun(before,facts,now,candidate.id,selection))
            val id=reply.getString("id");runs=listOf(Run(id,candidate.name,if(selection==TestSourceSelection.NAME_FILTER) "name filter" else candidate.runScope,reply.getString("state"),if(reply.isNull("exit_code")) null else reply.getInt("exit_code"),sourceHash=reply.getString("source_hash"),fingerprint=reply.getString("analysis_fingerprint"),contributors=reply.getJSONObject("checked_sources").let {hashes->hashes.keys().asSequence().associateWith {hashes.getString(it)}}))+runs;selected=id
            notice="Run admitted against saved bytes. Files remain mutable while it executes."
        }catch(error:Exception){notice=error.message.orEmpty()}finally{running=false}
    }
    suspend fun debug(candidate:SourceTestCandidate,selection:TestSourceSelection,workspace:Workspace) {
        if(debugging)return
        val observed=observation ?: return;val before=observed.first;val facts=observed.second
        val buffer=workspace.currentBuffer ?: return;val engine=workspace.engine ?: return;val debugger=workspace.debugger ?: return
        val namespace=debugger.sourceLineage.epoch
        val context=TestDebugContext(workspace,engine,debugger,buffer,observed,namespace)
        val choice=debugChoices.offer(context,before,facts,candidate.id,selection)
        val id=UUID.randomUUID();debugTicket=id;debugging=true;notice="Checking saved test debug provider…"
        fun current():Boolean {
            val now=snapshot(workspace) ?: return false
            return debugTicket==id&&workspace.testSources===this&&workspace.showTests&&workspace.engine===engine&&workspace.debugger===debugger&&workspace.currentBuffer===buffer&&observation===observed&&debugger.sourceLineage.epoch==namespace&&!buffer.dirty&&!buffer.saving&&!buffer.resourceDeleted&&testSourceCurrent(before,facts,now)&&before.request.sources==now.request.sources
        }
        try {
            require(current()&&!debugger.active){"Save this source and inspect or stop the active debugger before choosing Test Debug."}
            val request=sourceTestRun(before,facts,snapshot(workspace)!!,candidate.id,selection)
            val plan=TestDebugPlan.decode(engine.request("POST","/api/tests/debug-plan",request),choice)
            if(!current())return
            require(plan.available){plan.reason}
            // Only the dedicated plan's protocol admits start_test. Ordinary
            // launch profiles may legitimately have no program for this source.
            val body=debugChoices.take(choice,context,snapshot(workspace)!!,TestDebugHostState(!buffer.dirty,buffer.saving,buffer.resourceDeleted,debugger.active),JSONObject().put("test_debug_protocol",1))
            debugger.launchSourceTest(buffer,body,plan,facts,::current,
                admitted={session->debugging=false;notice="Saved test debug admitted. Process outcomes are retained separately from case verdicts.";debugRuns=listOf(DebugRun(session,candidate.name,candidate.id,facts.inputPath,facts.inputHash,plan.framework,selection.wire,facts.fingerprint,LinkedHashMap(facts.sourceHashes)))+debugRuns},
                rejected={detail->if(debugTicket==id){debugging=false;notice=detail}},
                outcome={session,state,exit->debugRuns=debugRuns.map{if(it.id==session)it.copy(state=state,exit=exit,output=debugger.output.joinToString(""){event->event.text})else it}})
        }catch(cancelled:kotlinx.coroutines.CancellationException){throw cancelled}
        catch(error:Exception){if(debugTicket==id)notice=error.message.orEmpty()}
        finally{if(debugTicket==id&&debugger.status!="starting")debugging=false}
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
internal fun Workspace.testPointNavigation(point: TestSourcePoint): suspend () -> Unit {
    val model = testSources
    val reading = model.observation
    val entry = model.entry
    val workspace = sourceNavigationEpoch
    val navigation = model.beginPointNavigation()
    val foreground = captureForegroundNavigation()
    fun current() = showTests && testSources === model && model.observation === reading && model.entry == entry &&
        sourceNavigationEpoch == workspace && navigation()
    return navigate@{
        if (!current() || !foreground()) return@navigate
        fun find(files: List<FileNode>): FileNode? {
            for (file in files) {
                if (file.path == point.path) return file
                find(file.children)?.let { return it }
            }
            return null
        }
        val file = (buffers[point.path]?.file ?: find(rootFiles))?.takeUnless { it.dir }
        if (file == null) {
            model.notice = "Source no longer listed. Refresh files and candidates."
            return@navigate
        }
        try {
            val buffer = loadBuffer(file)
            val source = buffer.text
            val revision = buffer.sourceRevision
            val hash = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Default) { debugHash(source) }
            if (!current() || !foreground()) return@navigate
            fun sourceCurrent() = current() && buffers[point.path] === buffer && buffer.sourceRevision == revision && buffer.text == source
            val position = JSONObject().put("line", point.line - 1).put("character", point.character)
            check(sourceCurrent() && hash == point.hash.lowercase() && languageOffset(source, position) != null) {
                "Source changed or the position is unavailable. Refresh candidates before navigating."
            }
            // The cached file opens synchronously. Its own activation advances
            // foreground intent, while source and reading guards remain current.
            openSemanticLocation(SemanticLocation(languageURI(point.path), point.line - 1, point.character),
                allowDirty = true, expectedSource = source, isCurrent = ::sourceCurrent, onRevealed = { showTests = false })
        } catch (cancelled: kotlinx.coroutines.CancellationException) { throw cancelled }
        catch (error: Exception) { if (current() && foreground()) model.notice = error.message.orEmpty() }
    }
}
@Composable fun TestSourcesView(workspace:Workspace) {
    val model=workspace.testSources;val valid=model.current(workspace)
    val candidates=(model.observation?.second?.candidates ?: emptyList()).filter {"${it.name} ${it.target.path}".contains(model.query,ignoreCase=true)}
    val last=maxOf(0,(candidates.size-1)/100);val page=minOf(model.page,last)
    Column(Modifier.widthIn(min=280.dp,max=760.dp).heightIn(min=240.dp,max=760.dp).verticalScroll(rememberScrollState()).padding(12.dp),verticalArrangement=Arrangement.spacedBy(8.dp)) {
        Row {Text("Source tests",fontSize=16.sp);Spacer(Modifier.weight(1f));TextButton(onClick={model.closeDebugPreparation();workspace.showTests=false;workspace.currentBuffer?.focusSource?.invoke()}){Text("Close")}}
        Text("Source declarations are candidates. Name filters may match several cases; parameterized cases need runner discovery. Terminal runs are separate from per-case verification.",fontSize=12.sp)
        Text(workspace.currentBuffer?.file?.path ?: "No source selected",fontSize=12.sp)
        OutlinedTextField(model.entry,{model.entry=it},label={Text("Construction entry (optional)")},singleLine=true)
        WithDisabledReason(if (model.reading) "Wait for the current reading to load." else if (workspace.currentBuffer == null) "Open a source file to discover test candidates." else null) { available -> TextButton(enabled = available,onClick={workspace.runCommand {model.refresh(workspace)}}){Text(if(model.reading) "Reading source…" else "Refresh candidates")} }
        model.observation?.second?.let {facts->
            Text("${if(valid) "Source observation" else "Stale observation — refresh before running"} · ${facts.scope} · ${if(facts.syntaxComplete) "parsed" else "partial syntax"}",fontSize=12.sp)
            (facts.notices+facts.unsupportedLanguages.map {"No source detector: $it"}+facts.unclassifiedOutputs.map {"Unclassified output: $it"}).forEach {Text(it,fontSize=12.sp)}
        }
        if(model.notice.isNotEmpty())Text(model.notice,fontSize=12.sp)
        OutlinedTextField(model.query,{model.query=it;model.page=0},label={Text("Filter source tests")},singleLine=true)
        candidates.drop(page*100).take(100).forEach {item->
            Text(item.name);Text("${item.confidence} · ${item.target.path}:${item.target.line}",fontSize=12.sp)
            WithDisabledReason(if (!(valid)) "Refresh the candidate against the current source." else null) { available -> TextButton(enabled = available,onClick={workspace.runCommand(workspace.testPointNavigation(item.point))}){Text("${if(item.source==null) "Open runtime source" else "Open authored source"} ${item.point.line}")} }
            WithDisabledReason(if (!(valid)) "Refresh the candidate against the current source." else if (model.running) "A test run is in progress." else if (item.nameFilter == null) "This candidate has no exact name filter. Use its containing scope." else null) { available -> TextButton(enabled = available,onClick={workspace.runCommand {model.run(item,TestSourceSelection.NAME_FILTER,workspace)}}){Text("Run name filter")} }
            WithDisabledReason(if (!(valid)) "Refresh the candidate against the current source." else if (model.running) "A test run is in progress." else null) { available -> TextButton(enabled = available,onClick={workspace.runCommand {model.run(item,TestSourceSelection.CONTAINING_SCOPE,workspace)}}){Text("Run ${item.runScope.replace('_',' ')} scope")} }
            WithDisabledReason(if (!(valid)) "Refresh the candidate against the current source." else if (model.debugging) "A test debugger request is in progress." else if (item.nameFilter == null) "This candidate has no exact name filter. Use its containing scope." else null) { available -> TextButton(enabled = available,onClick={workspace.runCommand {model.debug(item,TestSourceSelection.NAME_FILTER,workspace)}}){Text("Debug name filter")} }
            WithDisabledReason(if (!(valid)) "Refresh the candidate against the current source." else if (model.debugging) "A test debugger request is in progress." else null) { available -> TextButton(enabled = available,onClick={workspace.runCommand {model.debug(item,TestSourceSelection.CONTAINING_SCOPE,workspace)}}){Text("Debug ${item.runScope.replace('_',' ')} scope")} }
        }
        if(model.observation!=null&&candidates.isEmpty())Text("No source candidates returned. This does not prove the project has no tests.",fontSize=12.sp)
        if(last>0)Row {WithDisabledReason(if (page <= 0) "You’re on the first page." else null) { available -> TextButton(enabled = available,onClick={model.page=page-1}){Text("Previous")} };Text("${page+1}/${last+1}");WithDisabledReason(if (page >= last) "You’re on the last page." else null) { available -> TextButton(enabled = available,onClick={model.page=page+1}){Text("Next")} }}
        if(model.debugging)Text("Preparing saved Test Debug… Close cancels admission; it does not retry.",fontSize=12.sp)
        model.debugRuns.forEach {run->
            Text("${run.name} · ${run.selection} · debugger ${run.state}${run.exit?.let {" · process exit $it"} ?: ""}",fontSize=12.sp)
            var revisions by remember(run.id){mutableStateOf(false)}
            TextButton(onClick={revisions=!revisions}){Text("Saved test debug identity and output")}
            if(revisions){
                Text("${run.framework} · ${run.inputPath} · ${run.inputHash}\nCandidate ${run.candidateID}\nProcess outcome only; no collected case verdict.\n${run.fingerprint}\n"+run.contributors.entries.joinToString("\n"){"${it.key}: ${it.value}"},fontSize=12.sp)
                androidx.compose.foundation.text.selection.SelectionContainer{Text(run.output.ifEmpty{"No retained debugger process output."},fontSize=12.sp)}
            }
            TextButton(onClick={workspace.showTests=false}){Text("Return to editor debugger controls")}
        }
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
