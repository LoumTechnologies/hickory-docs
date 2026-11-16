<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Preview Android changes without owning the editor

For maintainers changing workspace services independently of the editor renderer.

The existing Edit menu requests a rename, formatting or provider action. Capture all retained source buffers and the originating selection before asking; an obsolete provider answer or edited target cannot become a preview. Text-only previews page every before/after line without reserving editor space. Application preflights every target, including retained editors and IME state, before synchronously committing the prepared draft replacements. Ordered resource edits transfer whole to the engine-backed source review before text-only discovery can reject or partially apply them. Provider commands remain explicit unsupported results.

Bounded workspace history orders draft inverses and resource recovery identities together. A draft entry uses the exact inverse model. A resource entry prepares its saved/draft restoration through the shared engine and waits for explicit review and confirmed publication before moving the new inverse identity to the opposite stack. Capture the history revision as part of review authority: an interleaved applied change must retire the older undo proposal even when its source text later returns to the same bytes. Local text edits still fail the source capture instead of being overwritten.

Rename discovery acquires previously unopened reference targets through the existing workspace buffer loader. Their language sessions synchronize before a fresh reference query; the discovering answer cannot authorize edits against later-loaded text. Acquisition never activates a tab. Captured source snapshots remain immutable inputs to the final provider proposal, and complete draft-plan validation still precedes application. Native preparation retains its own fixed request-validity closure so an older reply cannot borrow a newer request's authority.

Provider choices resolve through their captured provider before entering the same complete change-plan preview. A latest-choice ticket and the fixed preparation snapshot reject late answers or a replaced choice list. Disabled actions and complete edit-plus-command results refuse explicitly; resolving never applies an edit portion or saves source. The native view passes the rendered preparation identity so an old row index cannot select a newer list item.

Provider edit discovery uses the same bounded acquisition boundary. Newly loaded targets synchronize before the host re-queries actions at the captured selection and resolves a uniquely matching title and kind. Changed or ambiguous choices refuse instead of selecting an arbitrary replacement. Tokens from the discovering response never authorize the final plan. Original draft snapshots remain fixed; only missing sources are added, and acquisition does not activate tabs.

Explicit cleanup enters before language-provider acquisition. Its fixed source, selection, buffer identity and request ticket authorize only that preparation. Returning ordinary source replacements lets the existing preview, writability checks, coordinated apply and inverse history handle it; no second editor or save path is introduced. A no-op reports the observation without creating an undo entry. The cleanup profile is literal and explicit: significant trailing spaces can change, so review the before/after text.

Reviewed capabilities can offer a complete source proposal without a language provider. Every target must have the exact supplied reviewed text, and the ordinary planner and writability checks must accept the whole proposal before it becomes a preview. Capture target buffer identities, text and selections for the approval boundary. Source or selection changes refuse; successful approval uses the same coordinated inverse history as provider changes. The proposal mode has no provider-preparation button and never saves implicitly.

<ui:copy id="simple-android-workspace-changes">
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
        val request=++revision; val source=origin.text; val selection=origin.state.selection; val identity=language.proposalIdentity
        sources=workspace.sourceSnapshots(); current={revision==request && origin.text==source && origin.state.selection==selection && origin.language===language && language.proposalIdentity==identity}
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
        this.acquisition=load;actionQuery={val actions=propose("actions","") as? JSONArray ?: JSONArray();(0 until actions.length()).mapNotNull{actions.optJSONObject(it)}}
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
            if (kind == "actions") {
                val actions=result as? JSONArray ?: JSONArray(); choices=(0 until actions.length()).mapNotNull { actions.optJSONObject(it)?.let { item -> JSONObject(item.toString()) } }
                notice=if (choices.isEmpty()) "No provider actions here. Support depends on the installed language server." else "Choose a provider action to preview."
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
            val acquired=acquireSourceProposal(initialSources,{
                val resolved=first?.also{first=null} ?: run {
                    val candidates=queryActions().filter{it.optString("title")==action.optString("title")&&it.optString("kind")==action.optString("kind")}
                    check(candidates.size==1){"The selected provider action changed or is ambiguous. Prepare actions again."}
                    resolve(candidates.single())
                }
                check(revision==request&&choiceRevision==choice&&valid()){"Source, selection or provider changed. Prepare actions again."}
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
</ui:copy>

## Apply all source drafts in one Compose snapshot

Validate every target and reject active IME composition before entering a mutable snapshot. TextFieldState replacements then commit together; an exception discards the snapshot rather than publishing a prefix of the batch. Files remain untouched and the exact inverse is retained by the controller.

<ui:copy id="simple-android-workspace-changes-application">
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
</ui:copy>

## Page the complete preview in a floating Android dialog

The existing Edit menu presents this bounded floating dialog. Every affected source and every before/after line remains reachable through file selection and line pages. The view reads the controller and requests operations; it does not open sockets, fabricate provider edits or save source.

<ui:copy id="simple-android-workspace-changes-presentation">
@Composable fun WorkspaceChangesView(workspace: Workspace) {
    val changes=workspace.changes; val scope=rememberCoroutineScope();val actionSnapshot=changes.actionSnapshot
    Column(Modifier.widthIn(min=260.dp,max=400.dp).heightIn(max=480.dp).verticalScroll(rememberScrollState()).padding(8.dp)) {
        Text("Workspace change preview",fontSize=16.sp); Text("Source drafts · not saved or verified",fontSize=11.sp)
        TextButton(onClick={changes.close(workspace)}) { Text("Close") }
        if (changes.mode=="rename") OutlinedTextField(changes.name,{changes.updateName(it)},label={Text("New symbol name")},singleLine=true)
        if (changes.mode!="undo" && changes.mode!="redo" && changes.mode!="proposal") TextButton(enabled=changes.mode!="rename" || changes.name.isNotBlank(),onClick={scope.launch {changes.prepare(workspace)}}) { Text(if(changes.mode=="actions") "Prepare provider actions" else if(changes.mode=="format") "Prepare formatting" else if(changes.mode=="cleanup") "Prepare cleanup" else "Prepare rename") }
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
</ui:copy>



<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/WorkspaceChanges.kt"><ui:paste select="#simple-android-workspace-changes"/><ui:paste select="#simple-android-workspace-changes-application"/><ui:paste select="#simple-android-workspace-changes-presentation"/></ui:file>
## Retire older proposal requests

Hold two provider replies against the same unchanged source and selection. Completing the newer rename first must leave its preview intact when the older result arrives. Each preparation therefore retains its own validity closure; consulting a mutable current-request property would let old work borrow new authority. The injected provider boundary tests asynchronous intent safety without claiming an actual language-server run; source stays unchanged until explicit application.

<ui:copy id="native-change-request-android-witness">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
import org.json.JSONArray
class ChangeRequestTest {
 @Test fun olderRenameReplyCannotBorrowNewerRequestAuthority()=runBlocking {
  val workspace=Workspace(this);val engine=Engine("http://127.0.0.1:1");val file=FileNode("a.py","a.py",false,emptyList(),null,null)
  val buffer=DocumentBuffer(file,FileText("value\n","base"));buffer.language=NativeLanguage(engine,buffer,this);workspace.buffers[file.path]=buffer;workspace.show(file.path);assertSame(buffer,workspace.currentBuffer)
  val changes=workspace.changes;changes.open("rename",workspace);val held=mutableMapOf<String,CompletableDeferred<Any>>()
  val proposal:suspend (String,String)->Any={kind,name->if(kind=="references")JSONArray() else {val done=CompletableDeferred<Any>();held[name]=done;done.await()}}
  fun edit(name:String)=JSONObject().put("changes",JSONObject().put(languageURI(file.path),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",5))).put("newText",name))))
  suspend fun settle(condition:()->Boolean){withTimeout(2000){while(!condition())yield()}}
  changes.updateName("first");val first=async{changes.prepare(workspace,proposal)};settle{held["first"]!=null}
  changes.updateName("second");val second=async{changes.prepare(workspace,proposal)};settle{held["second"]!=null};held.getValue("second").complete(edit("second"));second.await();assertEquals("second\n",changes.plan?.patches?.first()?.after)
  held.getValue("first").complete(edit("first"));first.await();assertEquals("second\n",changes.plan?.patches?.first()?.after);assertEquals("value\n",buffer.text)
 }
}
</ui:copy>

## Exercise provider-independent cleanup and its authority

Use the real workspace change model without a language service. Preview, apply, workspace undo/redo and a no-op observation share the existing change machinery. Reorder held replies and close during preparation to check that an older response cannot borrow a newer request’s authority. Physical device input is not established by these tests.

<ui:copy id="cleanup-android-witness">
class CleanupRequestTest {
 @Test fun cleanupUsesCapturedSourceWithoutAProviderAndRetainsWorkspaceUndo()=runBlocking {
  val workspace=Workspace(this);val file=FileNode("a.py","a.py",false,emptyList(),null,null)
  val before="😀 value \t\r\n";val after="😀 value\r\n";val buffer=DocumentBuffer(file,FileText(before,"base"))
  workspace.buffers[file.path]=buffer;workspace.show(file.path);val changes=workspace.changes
  fun edit()=JSONObject("""{"changes":{"hick:///a.py":[{"range":{"start":{"line":0,"character":8},"end":{"line":0,"character":10}},"newText":""}]}}""")
  changes.open("cleanup",workspace);changes.prepare(workspace,{kind,_->assertEquals("cleanup",kind);edit()})
  assertEquals(before,buffer.text);assertEquals(after,changes.plan?.patches?.first()?.after)
  changes.apply(workspace);assertEquals(after,buffer.text);changes.open("undo",workspace);assertEquals(before,buffer.text)
  changes.open("redo",workspace);assertEquals(after,buffer.text);assertEquals(before,buffer.base)
  changes.open("cleanup",workspace);changes.prepare(workspace,{_,_->JSONObject("""{"changes":{"hick:///a.py":[]}}""")})
  assertNull(changes.plan);assertTrue(changes.notice.contains("No trailing"))
 }
 @Test fun cleanupCannotBorrowNewerAuthorityOrApplyAfterClose()=runBlocking {
  val workspace=Workspace(this);val file=FileNode("a.py","a.py",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText("x ","base"))
  workspace.buffers[file.path]=buffer;workspace.show(file.path);val changes=workspace.changes;changes.open("cleanup",workspace)
  val held=mutableListOf<CompletableDeferred<Any>>()
  val proposal:suspend (String,String)->Any={_,_->CompletableDeferred<Any>().also{held.add(it)}.await()}
  suspend fun settle(count:Int){withTimeout(2000){while(held.size<count)yield()}}
  fun edit(text:String)=JSONObject().put("changes",JSONObject().put("hick:///a.py",JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",1)).put("end",JSONObject().put("line",0).put("character",2))).put("newText",text))))
  val first=async{changes.prepare(workspace,proposal)};settle(1);val second=async{changes.prepare(workspace,proposal)};settle(2)
  held[1].complete(edit(""));second.await();assertEquals("x",changes.plan?.patches?.first()?.after)
  held[0].complete(edit("old"));first.await();assertEquals("x",changes.plan?.patches?.first()?.after)
  val closed=async{changes.prepare(workspace,proposal)};settle(3);changes.close(workspace);held[2].complete(edit(""));closed.await()
  assertNull(changes.plan);assertEquals("x ",buffer.text)
 }
}
</ui:copy>

## Keep reviewed-source proposals inside coordinated draft history

Offer exact reviewed text without a language provider. Missing or stale captures refuse before becoming previews. Approval applies through the same workspace history, while source or selection changes after preparation refuse. The saved baseline remains untouched until a separate save.

<ui:copy id="reviewed-source-android-tests">
class ReviewedSourceProposalTest {
 @Test fun reviewedSourceUsesSharedPreviewApplyAndUndo()=runBlocking{
  val workspace=Workspace(this);val file=FileNode("a.txt","a.txt",false,emptyList(),null,null);val before="draft\n";val after="incoming\n";val buffer=DocumentBuffer(file,FileText(before,"base"));workspace.buffers[file.path]=buffer;workspace.show(file.path)
  val edit=JSONObject("""{"changes":{"hick:///a.txt":[{"range":{"start":{"line":0,"character":0},"end":{"line":1,"character":0}},"newText":"incoming\n"}]}}""")
  assertTrue(runCatching{workspace.changes.offer("Reviewed source",edit,emptyMap(),workspace)}.isFailure);assertNull(workspace.changes.mode);assertEquals(before,buffer.text)
  workspace.changes.offer("Reviewed source",edit,mapOf(file.path to before),workspace);assertEquals("proposal",workspace.changes.mode);assertEquals(before,buffer.text)
  workspace.changes.apply(workspace);assertEquals(after,buffer.text);assertEquals(before,buffer.base)
  workspace.changes.open("undo",workspace);assertEquals(before,buffer.text);workspace.changes.open("redo",workspace);assertEquals(after,buffer.text)
 }
 @Test fun reviewedSourceRefusesEditsOrSelectionChangesBeforeApproval()=runBlocking{
  val workspace=Workspace(this);val file=FileNode("a.txt","a.txt",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText("x","base"));workspace.buffers[file.path]=buffer;workspace.show(file.path)
  val edit=JSONObject("""{"changes":{"hick:///a.txt":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"newText":"y"}]}}""")
  workspace.changes.offer("Reviewed source",edit,mapOf(file.path to "x"),workspace);buffer.state.edit{replace(0,length,"typed")};workspace.changes.apply(workspace);assertEquals("typed",buffer.text)
  assertTrue(runCatching{workspace.changes.offer("Reviewed source",edit,mapOf(file.path to "x"),workspace)}.isFailure)
  buffer.state.edit{replace(0,length,"x")};workspace.changes.offer("Reviewed source",edit,mapOf(file.path to "x"),workspace);buffer.state.edit{selection=androidx.compose.ui.text.TextRange(if(selection.start==0&&selection.end==0)1 else 0)};workspace.changes.apply(workspace);assertEquals("x",buffer.text)
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/ChangeRequestTest.kt"><ui:paste select="#native-change-request-android-witness"/><ui:paste select="#cleanup-android-witness"/><ui:paste select="#reviewed-source-android-tests"/></ui:file>
## Keep resolved choices with their preparation

Complete a newer choice before an older resolver reply, then reject a row from a replaced preparation and a reply after closing. These controlled callbacks exercise the actual native change model; source stays unchanged until explicit application. They do not claim native pointer or provider-language acceptance.

<ui:copy id="native-action-choice-android-witness">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
import org.json.JSONArray
class ActionChoiceTest {
 @Test fun unopenedActionTargetsAcquireThenRequeryAndRefuseAmbiguousChoices()=runBlocking {
  val workspace=Workspace(this);val engine=Engine("http://127.0.0.1:1");val file=FileNode("a.py","a.py",false,emptyList(),null,null)
  val buffer=DocumentBuffer(file,FileText("old\n","base"));buffer.language=NativeLanguage(engine,buffer,this);workspace.buffers[file.path]=buffer;workspace.show(file.path)
  val changes=workspace.changes;val action=JSONObject().put("title","Fix both").put("kind","quickfix").put("data",JSONObject().put("hickory_action","first"))
  var queries=0;var resolutions=0;val loads=mutableListOf<String>();var ambiguous=false
  val propose:suspend (String,String)->Any={_,_->queries++;JSONArray().put(action).also{if(ambiguous&&queries>1)it.put(action)}}
  val resolve:suspend (JSONObject)->JSONObject={
   resolutions++;if(resolutions==1)JSONObject().put("title","Fix both").put("data",JSONObject().put("hickory_acquire",JSONArray().put(JSONObject().put("uri",languageURI("other.py"))))) else {val replacement=JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",3))).put("newText","new")
   JSONObject().put("title","Fix both").put("edit",JSONObject().put("changes",JSONObject().put(languageURI("a.py"),JSONArray().put(replacement)).put(languageURI("other.py"),JSONArray().put(replacement))))}
  }
  val load:suspend (String)->SourceSnapshot={path->loads.add(path);val target=DocumentBuffer(FileNode(path,path,false,emptyList(),null,null),FileText("old\n","base"));workspace.buffers[path]=target;SourceSnapshot(target.text,true)}
  changes.open("actions",workspace);changes.prepare(workspace,propose,resolve,load);changes.choose(0,workspace)
  assertEquals(2,queries);assertEquals(2,resolutions);assertEquals(listOf("other.py"),loads);assertEquals(2,changes.plan?.patches?.size);assertEquals("a.py",workspace.currentBuffer?.file?.path);assertEquals("old\n",buffer.text)
  workspace.buffers.getValue("other.py").state.edit{replace(0,length,"later\n")};changes.apply(workspace);assertTrue(changes.notice.contains("Source changed"));assertEquals("old\n",buffer.text)
  workspace.buffers.remove("other.py");queries=0;ambiguous=true
  changes.open("actions",workspace);changes.prepare(workspace,propose,resolve,load);changes.choose(0,workspace);assertNull(changes.plan);assertTrue(changes.notice.contains("ambiguous"));assertEquals("old\n",buffer.text)
 }

 @Test fun latestChoiceWinsAndOldRowsCannotBorrowNewLists()=runBlocking {
  val workspace=Workspace(this);val engine=Engine("http://127.0.0.1:1");val file=FileNode("a.py","a.py",false,emptyList(),null,null)
  val buffer=DocumentBuffer(file,FileText("old\n","base"));buffer.language=NativeLanguage(engine,buffer,this);workspace.buffers[file.path]=buffer;workspace.show(file.path)
  val changes=workspace.changes;changes.open("actions",workspace);val held=mutableMapOf<String,CompletableDeferred<JSONObject>>()
  val actions=JSONArray().put(JSONObject().put("title","first").put("data",JSONObject().put("hickory_action","first"))).put(JSONObject().put("title","second").put("data",JSONObject().put("hickory_action","second")))
  val resolve:suspend (JSONObject)->JSONObject={action->val result=CompletableDeferred<JSONObject>();held[action.getString("title")]=result;result.await()}
  suspend fun prepare(){changes.prepare(workspace,{_,_->actions},resolve)}
  suspend fun settle(condition:()->Boolean){withTimeout(2000){while(!condition())yield()}}
  fun resolved(name:String)=JSONObject().put("title",name).put("edit",JSONObject().put("changes",JSONObject().put(languageURI(file.path),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",3))).put("newText",name)))))
  prepare();val snapshot=changes.actionSnapshot;val first=async{changes.choose(0,workspace,snapshot)};settle{held["first"]!=null};val second=async{changes.choose(1,workspace,snapshot)};settle{held["second"]!=null}
  held.getValue("second").complete(resolved("second"));second.await();held.getValue("first").complete(resolved("first"));first.await();assertEquals("second\n",changes.plan?.patches?.first()?.after)
  held.clear();prepare();changes.choose(0,workspace,snapshot);assertTrue(held.isEmpty());assertNull(changes.plan)
  val current=changes.actionSnapshot;val closing=async{changes.choose(0,workspace,current)};settle{held["first"]!=null};changes.close(workspace);held.getValue("first").complete(resolved("first"));closing.await();assertNull(changes.plan);assertEquals("old\n",buffer.text)
  changes.open("actions",workspace);changes.prepare(workspace,{_,_->actions},{resolved("partial").put("command",JSONObject().put("title","Effect").put("command","provider.effect"))})
  changes.choose(0,workspace,changes.actionSnapshot);assertNull(changes.plan);assertTrue(changes.notice.contains("provider command"));assertEquals("old\n",buffer.text)
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/ActionChoiceTest.kt"><ui:paste select="#native-action-choice-android-witness"/></ui:file>
</ui:doc>
