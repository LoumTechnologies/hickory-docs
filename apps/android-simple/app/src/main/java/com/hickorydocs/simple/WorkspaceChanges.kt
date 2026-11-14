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
class WorkspaceChanges {
    var mode by mutableStateOf<String?>(null)
    var name by mutableStateOf(""); private set
    var notice by mutableStateOf("")
    var plan by mutableStateOf<ChangePlan?>(null)
    var choices by mutableStateOf<List<JSONObject>>(emptyList())
    var file by mutableStateOf(0); var page by mutableStateOf(0); var actionPage by mutableStateOf(0)
    var referencePaths by mutableStateOf<List<String>>(emptyList())
    private var revision = 0
    private var sources: Map<String,SourceSnapshot> = emptyMap()
    private var current: ()->Boolean = { false }
    private val undo = mutableListOf<ChangePlan>(); private val redo = mutableListOf<ChangePlan>()
    fun reset() { revision++; mode=null; plan=null; choices=emptyList(); sources=emptyMap(); current={false}; undo.clear(); redo.clear(); notice="" }
    fun close(workspace: Workspace) { revision++; mode=null; workspace.currentBuffer?.focusSource?.invoke() }
    fun updateName(value: String) { revision++; name=value; plan=null; choices=emptyList(); notice="" }
    fun open(kind: String, workspace: Workspace) {
        revision++; mode=kind; name=""; plan=null; choices=emptyList(); notice=""; actionPage=0
        if (kind != "undo" && kind != "redo") return
        val history = if (kind == "undo") undo else redo
        val target = history.lastOrNull() ?: run { notice="No workspace change to $kind."; return }
        runCatching { workspace.applyWorkspacePlan(target) }.onSuccess {
            history.removeAt(history.lastIndex); (if (kind == "undo") redo else undo).add(invertChangePlan(target))
            notice="Workspace $kind applied to ${target.patches.size} drafts. Not saved or verified."
        }.onFailure { notice=it.message.orEmpty() }
    }
    suspend fun prepare(workspace: Workspace) {
        val origin = workspace.currentBuffer; val language = origin?.language
        if (origin == null || language == null || origin.readOnly) { notice="Choose writable source with a connected language provider."; return }
        val request=++revision; val source=origin.text; val selection=origin.state.selection; val identity=language.proposalIdentity
        sources=workspace.sourceSnapshots(); current={revision==request && origin.text==source && origin.state.selection==selection && origin.language===language && language.proposalIdentity==identity}
        plan=null; choices=emptyList(); referencePaths=emptyList(); notice="Preparing complete source changes…"
        val kind=mode ?: "actions"; val nextName=name
        try {
            if (kind == "rename") {
                val references=language.propose("references","")
                if (!current()) { if (revision==request) notice="Source, selection or provider changed. Prepare again."; return }
                referencePaths=semanticLocations(references).mapNotNull { it.path }
            }
            val result=language.propose(kind,nextName)
            if (!current()) { if (revision==request) notice="Source, selection or provider changed. Prepare again."; return }
            if (kind == "actions") {
                val actions=result as? JSONArray ?: JSONArray(); choices=(0 until actions.length()).mapNotNull { actions.optJSONObject(it)?.let { item -> JSONObject(item.toString()) } }
                notice=if (choices.isEmpty()) "No provider actions here. Support depends on the installed language server." else "Choose a provider action to preview."
            } else install(if (kind == "rename") "Rename to $nextName" else "Format source",result,workspace)
        } catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { if (revision==request) notice=failure.message.orEmpty() }
    }
    private fun install(title: String, edit: Any?, workspace: Workspace) {
        check(current()) { "Source, selection or provider changed. Prepare again." }
        val next=planWorkspaceEdit(title,edit,sources)
        validateChangePlan(next,workspace.sourceSnapshots())?.let { error(it) }
        validateReferencedTargets(next,referencePaths)
        plan=next; choices=emptyList(); file=0; page=0; notice=""
    }
    fun choose(index: Int, workspace: Workspace) {
        val action=choices.getOrNull(index) ?: return
        runCatching {
            action.optJSONObject("disabled")?.let { error(it.optString("reason","Provider action disabled")) }
            check(!action.has("command")) { "This action requires a provider command; its edit has not been applied." }
            install(action.optString("title","Provider action"),action.opt("edit"),workspace)
        }.onFailure { notice=it.message.orEmpty() }
    }
    fun apply(workspace: Workspace) {
        val next=plan ?: return
        runCatching {
            check(current()) { "Source, selection or provider changed. Prepare again." }
            workspace.applyWorkspacePlan(next)
        }.onSuccess {
            undo.add(invertChangePlan(next)); while(undo.size>32) undo.removeAt(0); redo.clear(); plan=null; current={false}
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
    val changes=workspace.changes; val scope=rememberCoroutineScope()
    Column(Modifier.widthIn(min=260.dp,max=400.dp).heightIn(max=480.dp).verticalScroll(rememberScrollState()).padding(8.dp)) {
        Text("Workspace change preview",fontSize=16.sp); Text("Source drafts · not saved or verified",fontSize=11.sp)
        TextButton(onClick={changes.close(workspace)}) { Text("Close") }
        if (changes.mode=="rename") OutlinedTextField(changes.name,{changes.updateName(it)},label={Text("New symbol name")},singleLine=true)
        if (changes.mode!="undo" && changes.mode!="redo") TextButton(enabled=changes.mode!="rename" || changes.name.isNotBlank(),onClick={scope.launch {changes.prepare(workspace)}}) { Text(if(changes.mode=="actions") "Prepare provider actions" else if(changes.mode=="format") "Prepare formatting" else "Prepare rename") }
        if (changes.notice.isNotEmpty()) Text(changes.notice,fontSize=12.sp)
        for(index in changes.choices.indices.drop(changes.actionPage*100).take(100)) {
            val action=changes.choices[index]
            TextButton(onClick={changes.choose(index,workspace)}) { Text(action.optString("title","Provider action")) }
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
