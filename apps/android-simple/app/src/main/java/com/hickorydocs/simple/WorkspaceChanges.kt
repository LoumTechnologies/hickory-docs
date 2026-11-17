package com.hickorydocs.simple
import androidx.compose.runtime.*
import androidx.compose.runtime.snapshots.Snapshot
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import org.json.JSONObject
import org.json.JSONArray
fun fileWideFixActions(actions:List<JSONObject>)=actions.filter{val kind=it.opt("kind") as? String;kind=="source.fixAll"||kind?.startsWith("source.fixAll.")==true}
class WorkspaceChanges {
    var mode by mutableStateOf<String?>(null)
    var name by mutableStateOf(""); private set
    var notice by mutableStateOf("")
    var plan by mutableStateOf<ChangePlan?>(null)
    var choices by mutableStateOf<List<JSONObject>>(emptyList())
    var file by mutableStateOf(0); var page by mutableStateOf(0); var actionPage by mutableStateOf(0)
    var referencePaths by mutableStateOf<List<String>>(emptyList())
    private var revision = 0
    val actionSnapshot get()=revision
    private var choiceRevision=0
    private var resolver:suspend (JSONObject)->JSONObject={error("Prepare provider actions first.")}
    private var acquisition:suspend (String)->SourceSnapshot={error("Prepare provider actions first.")}
    private var actionQuery:suspend ()->List<JSONObject> = {error("Prepare provider actions first.")}
    private var sources: Map<String,SourceSnapshot> = emptyMap()
    private var current: ()->Boolean = { false }
    private sealed interface History {
        data class Draft(val plan:ChangePlan):History
        data class Resource(val record:String,val workspace:String):History
    }
    private val undo=mutableListOf<History>();private val redo=mutableListOf<History>();private var historyRevision=0L
    fun recordResource(record:String,workspace:String){undo.add(History.Resource(record,workspace));while(undo.size>32)undo.removeAt(0);redo.clear();historyRevision++}
    fun reset() { revision++;historyRevision++; mode=null; plan=null; choices=emptyList(); sources=emptyMap(); current={false}; undo.clear(); redo.clear(); notice="" }
    fun close(workspace: Workspace) { revision++; mode=null; workspace.currentBuffer?.focusSource?.invoke() }
    fun updateName(value: String) { revision++; name=value; plan=null; choices=emptyList(); notice="" }
    fun open(kind: String, workspace: Workspace) {
        revision++; mode=kind; name=""; plan=null; choices=emptyList(); notice=""; actionPage=0
        if (kind != "undo" && kind != "redo") return
        val history = if (kind == "undo") undo else redo
        val target = history.lastOrNull() ?: run { notice="No workspace change to $kind."; return }
        if(target is History.Resource){
            val expected=historyRevision;val valid={historyRevision==expected&&history.lastOrNull()===target}
            mode=null;workspace.reviewResourceHistory(target.record,target.workspace,valid){record,sourceWorkspace->
                check(valid()){"Workspace history changed. Inspect recovery before another undo."}
                history.removeAt(history.lastIndex);(if(kind=="undo")redo else undo).add(History.Resource(record,sourceWorkspace));historyRevision++
            }
            return
        }
        target as History.Draft
        runCatching { workspace.applyWorkspacePlan(target.plan) }.onSuccess {
            history.removeAt(history.lastIndex); (if (kind == "undo") redo else undo).add(History.Draft(invertChangePlan(target.plan)));historyRevision++
            notice="Workspace $kind applied to ${target.plan.patches.size} drafts. Not saved or verified."
        }.onFailure { notice=it.message.orEmpty() }
    }
    suspend fun prepare(workspace: Workspace,proposal:(suspend (String,String)->Any)?=null,resolution:(suspend (JSONObject)->JSONObject)?=null,acquisition:(suspend (String)->SourceSnapshot)?=null) {
        if(mode=="cleanup"){prepareCleanup(workspace,proposal);return}
        val origin = workspace.currentBuffer; val language = origin?.language
        if (origin == null || language == null || origin.readOnly) { notice="Choose writable source with a connected language provider."; return }
        val request=++revision; val source=origin.text; val selection=origin.state.selection; val identity=language.proposalIdentity;val sourceIdentity=origin.sourceRevision
        sources=workspace.sourceSnapshots(); current={revision==request && workspace.buffers[origin.file.path]===origin && origin.sourceRevision==sourceIdentity && origin.text==source && origin.state.selection==selection && origin.language===language && language.proposalIdentity==identity}
        resolver=resolution ?: {language.resolveProposal(it)}
        val valid=current
        val propose:suspend (String,String)->Any=proposal ?: {kind,value->language.propose(kind,value)}
                val listed=mutableMapOf<String,FileNode>();fun collect(nodes:List<FileNode>){for(file in nodes){if(!file.dir)listed[file.path]=file;collect(file.children)}};collect(workspace.rootFiles)
        val load:suspend (String)->SourceSnapshot=acquisition ?: {path->
                    check(valid()){"Source, selection or provider changed. Prepare again."}
                    val file=(workspace.buffers[path]?.file ?: listed[path])?.takeUnless{it.dir} ?: error("Referenced source is no longer listed: $path. Refresh files and prepare again.")
                    val target=workspace.loadBuffer(file);val text=target.text
                    check(!target.readOnly&&!target.saving){"Referenced source cannot synchronize: $path. Retry language services."}
                    val service=target.language ?: error("Referenced source cannot synchronize: $path. Retry language services.");service.readOutline(text)
                    check(valid()&&target.text==text){"Source changed while reading $path. Prepare again."}
                    SourceSnapshot(text,!target.readOnly&&!target.saving)
                }
        val actionKind=if(mode=="fixAll")"fixAll" else "actions"
        this.acquisition=load;actionQuery={val actions=propose(actionKind,"") as? JSONArray ?: JSONArray();val listed=(0 until actions.length()).mapNotNull{actions.optJSONObject(it)};if(actionKind=="fixAll")fileWideFixActions(listed) else listed}
        plan=null; choices=emptyList(); referencePaths=emptyList(); notice="Preparing complete source changes…"
        val kind=mode ?: "actions"; val nextName=name
        try {
            if (kind == "rename") {
                val acquired=acquireReferenceSources(sources,{
                    semanticLocations(propose("references","")).map{it.path ?: error("Reference target has no writable workspace source identity. No edits were applied.")}
                },load,valid)
                if(!valid())return;sources=acquired.sources;referencePaths=acquired.references
            }
            val result=propose(kind,nextName)
            if (!valid()) { if (revision==request) notice="Source, selection or provider changed. Prepare again."; return }
            if (kind == "actions" || kind == "fixAll") {
                val actions=result as? JSONArray ?: JSONArray(); choices=(0 until actions.length()).mapNotNull { actions.optJSONObject(it)?.let { item -> JSONObject(item.toString()) } }
                if(kind=="fixAll")choices=fileWideFixActions(choices)
                notice=if(kind=="fixAll"&&choices.isEmpty()) "The current provider offers no file-wide diagnostic fixes. No edits were applied." else if (choices.isEmpty()) "No provider actions here. Support depends on the installed language server." else "Choose a provider action to preview."
            } else install(if (kind == "rename") "Rename to $nextName" else "Format source",result,workspace)
        } catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { if (revision==request) notice=failure.message.orEmpty() }
    }
    private suspend fun prepareCleanup(workspace:Workspace,proposal:(suspend (String,String)->Any)?) {
        val origin=workspace.currentBuffer
        if(origin==null||origin.readOnly||origin.saving){notice="Choose writable source before preparing cleanup.";return}
        val request=++revision;val source=origin.text;val selection=origin.state.selection;val path=origin.file.path
        sources=workspace.sourceSnapshots();current={revision==request&&workspace.buffers[path]===origin&&origin.text==source&&origin.state.selection==selection}
        val valid=current
        plan=null;choices=emptyList();referencePaths=emptyList();notice="Preparing trailing-whitespace cleanup…"
        try{
            val edit=if(proposal!=null)proposal("cleanup","") else {
                val engine=workspace.engine ?: error("Connect to a workspace server first.")
                val uri=languageURI(path)
                engine.request("POST","/api/changes/cleanup",JSONObject().put("uri",uri).put("source",source).put("profile","trim_trailing_horizontal"))
            }
            if(!valid()){if(revision==request)notice="Source or selection changed. Prepare cleanup again.";return}
            install("Trim trailing whitespace",edit,workspace)
        }catch(cancelled:CancellationException){throw cancelled}catch(error:Exception){if(revision==request)notice=error.message.orEmpty()}
    }
    fun offer(title:String,edit:Any,expected:Map<String,String>,workspace:Workspace){
        val snapshots=workspace.sourceSnapshots();val next=planWorkspaceEdit(title,edit,snapshots)
        validateChangePlan(next,snapshots)?.let{error(it)}
        val targets=next.patches.map{patch->val before=expected[patch.path] ?: error("Reviewed source target was not captured. Prepare again.");val buffer=workspace.buffers[patch.path] ?: error("Reviewed source is unavailable. Prepare again.");check(before==patch.before&&buffer.text==before){"Reviewed source changed. Prepare the whole proposal again."};Triple(buffer,before,buffer.state.selection)}
        val request=++revision;sources=snapshots;current={revision==request&&targets.all{(buffer,text,selection)->workspace.buffers[buffer.file.path]===buffer&&buffer.text==text&&buffer.state.selection==selection}}
        mode="proposal";plan=next;choices=emptyList();referencePaths=emptyList();notice="";file=0;page=0;actionPage=0
    }
    private suspend fun install(title: String, edit: Any?, workspace: Workspace) {
        check(current()) { "Source, selection or provider changed. Prepare again." }
        val resourceEdit=edit as? JSONObject
        if(resourceEdit?.optJSONArray("documentChanges")?.let{rows->(0 until rows.length()).any{rows.optJSONObject(it)?.has("kind")==true}}==true){
            val valid=current
            workspace.resourceChanges.prepare(workspace,title,resourceEdit,valid)
            if(workspace.resourceChanges.review!=null){mode=null;workspace.showRecovery=true}else notice=workspace.resourceChanges.notice
            return
        }
        val next=planWorkspaceEdit(title,edit,sources,allowNoop=mode=="cleanup")
        validateChangePlan(next,workspace.sourceSnapshots())?.let { error(it) }
        validateReferencedTargets(next,referencePaths)
        if(next.patches.isEmpty()){plan=null;notice="No trailing ASCII spaces or tabs to remove.";return}
        plan=next; choices=emptyList(); file=0; page=0; notice=""
    }
    suspend fun choose(index:Int,workspace:Workspace,expectedSnapshot:Int?=null) {
        if(expectedSnapshot!=null&&expectedSnapshot!=revision){notice="Code action choices changed. Prepare actions again.";return}
        val action=choices.getOrNull(index) ?: return;val request=revision;val choice=++choiceRevision;val valid=current;val resolve=resolver;val load=acquisition;val queryActions=actionQuery;val initialSources=sources
        try {
            check(valid()){"Source, selection or provider changed. Prepare actions again."}
            action.optJSONObject("disabled")?.let{error(it.optString("reason","Provider action disabled"))}
            plan=null;notice="Resolving complete provider action…";var first:JSONObject?=resolve(action)
            if(mode=="fixAll")check(first?.let{fileWideFixActions(listOf(it)).isNotEmpty()}==true){"The provider changed the complete file-wide action kind. Prepare again."}
            val acquired=acquireSourceProposal(initialSources,{
                val resolved=first?.also{first=null} ?: run {
                    val candidates=queryActions().filter{it.optString("title")==action.optString("title")&&it.optString("kind")==action.optString("kind")}
                    check(candidates.size==1){"The selected provider action changed or is ambiguous. Prepare actions again."}
                    resolve(candidates.single())
                }
                check(revision==request&&choiceRevision==choice&&valid()){"Source, selection or provider changed. Prepare actions again."}
                if(mode=="fixAll")check(fileWideFixActions(listOf(resolved)).isNotEmpty()){"The provider changed the complete file-wide action kind. Prepare again."}
                resolved.optJSONObject("disabled")?.let{error(it.optString("reason","Provider action disabled"))}
                check(resolved.isNull("command")){"This action requires a provider command; its edit has not been applied."}
                val edit=resolved.optJSONObject("edit")
                val resources=edit?.optJSONArray("documentChanges")?.let{rows->(0 until rows.length()).any{rows.optJSONObject(it)?.has("kind")==true}}==true
                SourceProposal(if(resources)emptyList() else providerActionPaths(resolved),resolved)
            },load,{revision==request&&choiceRevision==choice&&valid()})
            if(revision!=request||choiceRevision!=choice)return
            sources=acquired.sources;val resolved=acquired.value
            install(resolved.optString("title","Provider action"),resolved.get("edit"),workspace)
        }catch(cancelled:CancellationException){throw cancelled}catch(error:Exception){if(revision==request&&choiceRevision==choice)notice=error.message.orEmpty()}
    }
    fun apply(workspace: Workspace) {
        val next=plan ?: return
        runCatching {
            check(current()) { "Source, selection or provider changed. Prepare again." }
            workspace.applyWorkspacePlan(next)
        }.onSuccess {
            undo.add(History.Draft(invertChangePlan(next))); while(undo.size>32) undo.removeAt(0); redo.clear();historyRevision++; plan=null; current={false}
            notice="Applied ${next.patches.size} source drafts. Not saved or verified. Use Save All after reviewing; Undo workspace change restores this batch."
        }.onFailure { notice=it.message.orEmpty() }
    }
}
fun Workspace.sourceSnapshots(): Map<String,SourceSnapshot> = buffers.mapValues { (_,buffer)->SourceSnapshot(buffer.text,!buffer.readOnly && !buffer.saving) }
fun Workspace.applyWorkspacePlan(plan: ChangePlan) {
    validateChangePlan(plan,sourceSnapshots())?.let { error(it) }
    for (patch in plan.patches) check(buffers.getValue(patch.path).state.composition == null) { "Finish composition before applying: ${patch.path}" }
    Snapshot.withMutableSnapshot {
        for (patch in plan.patches) {
            val edits=patch.edits.map { NativeSourceEdit(it.from,it.to,it.text) }
            check(buffers.getValue(patch.path).applySourceEdits(edits,patch.before)) { "Editor refused source change: ${patch.path}" }
        }
    }
}
@Composable fun WorkspaceChangesView(workspace: Workspace) {
    val changes=workspace.changes; val scope=rememberCoroutineScope();val actionSnapshot=changes.actionSnapshot
    Column(Modifier.widthIn(min=260.dp,max=400.dp).heightIn(max=480.dp).verticalScroll(rememberScrollState()).padding(8.dp)) {
        Text("Workspace change preview",fontSize=16.sp); Text("Source drafts · not saved or verified",fontSize=11.sp)
        TextButton(onClick={changes.close(workspace)}) { Text("Close") }
        if (changes.mode=="rename") OutlinedTextField(changes.name,{changes.updateName(it)},label={Text("New symbol name")},singleLine=true)
        if (changes.mode!="undo" && changes.mode!="redo" && changes.mode!="proposal") TextButton(enabled=changes.mode!="rename" || changes.name.isNotBlank(),onClick={scope.launch {changes.prepare(workspace)}}) { Text(if(changes.mode=="fixAll") "Prepare file-wide diagnostic fixes" else if(changes.mode=="actions") "Prepare provider actions" else if(changes.mode=="format") "Prepare formatting" else if(changes.mode=="cleanup") "Prepare cleanup" else "Prepare rename") }
        if (changes.notice.isNotEmpty()) Text(changes.notice,fontSize=12.sp)
        for(index in changes.choices.indices.drop(changes.actionPage*100).take(100)) {
            val action=changes.choices[index]
            TextButton(onClick={scope.launch {changes.choose(index,workspace,actionSnapshot)}}) { Text(action.optString("title","Provider action")) }
            action.optJSONObject("disabled")?.let { Text(it.optString("reason"),fontSize=11.sp) }
        }
        if(changes.choices.size>100) Row { TextButton(enabled=changes.actionPage>0,onClick={changes.actionPage--}) {Text("Previous actions")}; TextButton(enabled=(changes.actionPage+1)*100<changes.choices.size,onClick={changes.actionPage++}) {Text("Next actions")} }
        changes.plan?.let { plan ->
            Text("${plan.title} · ${plan.patches.size} source files")
            if(changes.mode=="rename") Text("Reference targets: ${changes.referencePaths.toSet().size} files · project analysis scope not established",fontSize=11.sp)
            var expanded by remember {mutableStateOf(false)}
            val selectedFile=minOf(changes.file,plan.patches.lastIndex); val patch=plan.patches[selectedFile]
            Box { TextButton(onClick={expanded=true}) {Text(patch.path)}; DropdownMenu(expanded,{expanded=false}) {plan.patches.forEachIndexed {index,item->DropdownMenuItem(text={Text(item.path)},onClick={changes.file=index;changes.page=0;expanded=false})} } }
            val before=patch.before.split("\n"); val after=patch.after.split("\n"); val last=maxOf(0,(maxOf(before.size,after.size)+99)/100-1); val page=minOf(changes.page,last)
            Text("Before · lines ${page*100+1}–${minOf(before.size,(page+1)*100)}",fontSize=11.sp); Text(before.drop(page*100).take(100).joinToString("\n"),fontSize=12.sp,fontFamily=androidx.compose.ui.text.font.FontFamily.Monospace)
            Text("After",fontSize=11.sp); Text(after.drop(page*100).take(100).joinToString("\n"),fontSize=12.sp,fontFamily=androidx.compose.ui.text.font.FontFamily.Monospace)
            if(last>0) Row {TextButton(enabled=page>0,onClick={changes.page=page-1}) {Text("Previous lines")}; Text("${page+1}/${last+1}"); TextButton(enabled=page<last,onClick={changes.page=page+1}) {Text("Next lines")} }
            TextButton(onClick={changes.apply(workspace)}) {Text("Apply to source drafts")}
        }
    }
}
