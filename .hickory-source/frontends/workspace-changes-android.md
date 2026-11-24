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

File-wide diagnostic fixing asks the provider for the complete `source.fixAll` family. Retain one explicit complete action for review, acquire its targets, then resolve it again under the original source revision. No action means unavailable; individual quick fixes are never guessed or merged.

A pending proposal keeps its original revision lease. A fresh synchronous Undo/Redo instead binds the inverse to its original editor objects and resolves each unique current address immediately before complete preimage validation. This follows the same editor through Save As and guarded namespace moves, while refusing an independent handle that reused the old path. Ordinary Save and late capability metadata do not discard text history: history never writes saved files, and current writability and composition still preflight the whole batch. Reading the inverse and committing it contain no await.

Production provider edits require engine coordinate proofs after the final fresh query. The origin is acknowledged before the initial request; readonly proof context is acquired and synchronized without treating it as an edited target. Every actual existing edited preimage must be writable and covered by a matching captured-byte proof. For ordered resource edits the engine resource-inputs response supplies those initial preimages, so a newly created or renamed destination never becomes an invented old source. The original editor lease, selection, revision and provider still gate review and application. Explicit injected proposal closures remain a trusted test seam; tests can enforce the production proof policy separately. Direct bound rename/format envelopes also require a typed false no-effect flag before their edit is considered.

Complete-file source cleanup retains only the standard `source` family through
query and resolution. Unavailable or changed actions refuse while preserving the
ordinary editor; the existing source lease and history authority still apply.

Closing a review retires its pending replies and dismisses its dialog. The
dialog's native focus handoff restores the previous surface. Requesting source
focus here opens Gboard even when the reader left the editor unfocused.

<ui:copy id="simple-android-workspace-changes">
package com.hickorydocs.simple
import androidx.compose.runtime.*
import androidx.compose.runtime.snapshots.Snapshot
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import org.json.JSONObject
import org.json.JSONArray
fun scopedProviderActions(actions:List<JSONObject>,scope:String?):List<JSONObject>{val prefix=if(scope=="fixAll")"source.fixAll" else if(scope=="sourceActions")"source" else null;return if(prefix==null)actions else actions.filter{val kind=it.opt("kind") as? String;kind==prefix||kind?.startsWith(prefix+".")==true}}
fun fileWideFixActions(actions:List<JSONObject>)=scopedProviderActions(actions,"fixAll")
private data class WorkspaceDraftHistory(val plan:ChangePlan,val read:()->ChangePlan)
private fun captureDraftHistory(plan:ChangePlan,workspace:Workspace):WorkspaceDraftHistory {
 val sealed=plan.copy(patches=plan.patches.map{it.copy(edits=it.edits.toList())});val server=workspace.engine;val targets=sealed.patches.map{it to workspace.buffers[it.path]}
 return WorkspaceDraftHistory(sealed){
  fun refuse():Nothing=error("Workspace history no longer owns the original source editors. Drafts were retained.")
  if(workspace.engine!==server||targets.isEmpty())refuse()
  val patches=targets.map{(patch,b)->if(b==null||b.readOnly||b.saving)refuse();val addresses=workspace.buffers.filterValues{it===b}.keys;if(addresses.size!=1||addresses.single()!=b.file.path)refuse();patch.copy(path=addresses.single())}
  if(patches.map{it.path}.distinct().size!=patches.size)refuse();val current=sealed.copy(patches=patches);validateChangePlan(current,workspace.sourceSnapshots())?.let{error(it)};current
 }
}
class WorkspaceChanges {
    var entrySelection by mutableStateOf<FrozenWorkspaceEntrySelection?>(null);private set
    var entryDestination by mutableStateOf(TextFieldState()); private set
    var preparingEntryMove by mutableStateOf(false); private set
    private var entryScope=0L
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
    private var requiresProviderProofs=true
    private var sources: Map<String,SourceSnapshot> = emptyMap()
    private var current: ()->Boolean = { false }
    private sealed interface History {
        data class Draft(val step:WorkspaceDraftHistory):History
        data class Resource(val record:String,val workspace:String):History
    }
    private val undo=mutableListOf<History>();private val redo=mutableListOf<History>();private var historyRevision=0L
    fun recordDraftApplication(plan:ChangePlan,workspace:Workspace){
        if(plan.patches.none{it.before!=it.after})return
        undo.add(History.Draft(captureDraftHistory(invertChangePlan(plan),workspace)));while(undo.size>32)undo.removeAt(0);redo.clear();historyRevision++
    }
    fun recordResource(record:String,workspace:String){undo.add(History.Resource(record,workspace));while(undo.size>32)undo.removeAt(0);redo.clear();historyRevision++}
    fun reset() { revision++;retireEntryMove();historyRevision++; mode=null; plan=null; choices=emptyList(); sources=emptyMap(); current={false}; undo.clear(); redo.clear(); notice="" }
    fun close(workspace: Workspace) {
        revision++
        if(mode=="entryMove")workspace.resourceChanges.dismiss()
        retireEntryMove();mode=null
    }
    internal fun retireEntryMove() {
        entryScope++
        entrySelection = null
        entryDestination = TextFieldState()
        preparingEntryMove = false
    }
    fun updateName(value: String) {
        revision++
        if (mode == "entryMove") entryDestination.edit { replace(0, length, value) }
        else name = value
        plan = null
        choices = emptyList()
        notice = ""
    }
    fun open(kind: String, workspace: Workspace) {
        revision++;retireEntryMove();mode=kind;name="";plan=null;choices=emptyList();notice="";actionPage=0
        if(kind=="entryMove"){
            workspace.resourceChanges.dismiss()
            val scope=entryScope
            notice="Reading the current main construction…"
            workspace.runCommand{openEntryMove(workspace,scope)}
            return
        }
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
        runCatching { val current=target.step.read();workspace.applyWorkspacePlan(current);current }.onSuccess { current->
            history.removeAt(history.lastIndex); (if (kind == "undo") redo else undo).add(History.Draft(captureDraftHistory(invertChangePlan(current),workspace)));historyRevision++
            notice="Workspace $kind applied to ${current.patches.size} drafts. Not saved or verified."
        }.onFailure { notice=it.message.orEmpty() }
    }
    private suspend fun openEntryMove(workspace:Workspace,scope:Long) {
        val current={entryScope==scope}
        try {
            val selection=workspace.resourceChanges.readEntry(workspace,current)
            check(selection.entryPath!=null){"This workspace has no configured main construction."}
            check(selection.proof.movable){"The current main construction cannot be moved. Check its saved source."}
            entrySelection=selection
            notice=""
        }catch(cancelled:CancellationException){throw cancelled}catch(error:Exception){if(current())notice=error.message.orEmpty()}
    }
    suspend fun prepare(workspace: Workspace,proposal:(suspend (String,String)->Any)?=null,resolution:(suspend (JSONObject)->JSONObject)?=null,acquisition:(suspend (String)->SourceSnapshot)?=null,enforceProviderProofs:Boolean=false) {
        if(mode=="entryMove"){
            val selection=entrySelection ?: return
            val field = entryDestination
            val destination = field.text.toString()
            if (preparingEntryMove || field.composition != null || destination.isEmpty() || destination == selection.entryPath) return
            val request = ++revision
            val valid = {
                revision == request && entryDestination === field && field.text.contentEquals(destination) &&
                    field.composition == null && selection.current()
            }
            preparingEntryMove = true
            notice = "Preparing the main construction move…"
            try {
                workspace.resourceChanges.prepareEntryMove(workspace, selection, destination, valid)
                if (valid()) {
                    if (workspace.resourceChanges.review != null) mode = null
                    else notice = workspace.resourceChanges.notice
                }
            } finally {
                if (entryDestination === field) preparingEntryMove = false
            }
            return
        }
        if(mode=="cleanup"){prepareCleanup(workspace,proposal);return}
        val origin = workspace.currentBuffer; val language = origin?.language
        if (origin == null || language == null || origin.readOnly) { notice="Choose writable source with a connected language provider."; return }
        requiresProviderProofs=proposal==null||enforceProviderProofs;val proofRequired=requiresProviderProofs;val server=workspace.engine
        val request=++revision; val source=origin.text; val selection=origin.state.selection; val identity=language.proposalIdentity;val sourceIdentity=origin.sourceRevision
        sources=workspace.sourceSnapshots(); current={revision==request && workspace.engine===server && workspace.buffers[origin.file.path]===origin && origin.sourceRevision==sourceIdentity && origin.text==source && origin.state.selection==selection && origin.language===language && language.proposalIdentity==identity}
        resolver=resolution ?: {language.resolveProposal(it)}
        val originCurrent=current;val lease=workspace.sourceAcquisitionLease(external=originCurrent);current={lease.current()}
        val valid=current
        val propose:suspend (String,String)->Any=proposal ?: {kind,value->language.propose(kind,value)}
                val listed=mutableMapOf<String,FileNode>();fun collect(nodes:List<FileNode>){for(file in nodes){if(!file.dir)listed[file.path]=file;collect(file.children)}};collect(workspace.rootFiles)
        val acquireTarget:suspend (String)->SourceSnapshot=acquisition ?: {path->
                    check(valid()){"Source, selection or provider changed. Prepare again."}
                    val file=(workspace.buffers[path]?.file ?: listed[path])?.takeUnless{it.dir} ?: error("Referenced source is no longer listed: $path. Refresh files and prepare again.")
                    val wasRetained=workspace.buffers.containsKey(path);val target=workspace.loadBuffer(file);val text=target.text
                    check(wasRetained||target.sourceRevision==0L&&target.text==target.base){"The newly acquired source was edited before original ownership could be sealed."}
                    check(lease.current(path)){"Original source owners changed during acquisition. Prepare again."};lease.retain(path,target)
                    check(!target.saving){"Referenced source cannot synchronize: $path. Retry language services."}
                    val service=target.language ?: error("Referenced source cannot synchronize: $path. Retry language services.");service.readOutline(text)
                    check(valid()&&target.text==text){"Source changed while reading $path. Prepare again."}
                    SourceSnapshot(text,!target.readOnly&&!target.saving)
                }
        val load:suspend (String)->SourceSnapshot={path->
            check(valid()){"Original source owners changed before acquisition. Prepare again."};val wasRetained=workspace.buffers.containsKey(path);val source=acquireTarget(path)
            val target=workspace.buffers[path] ?: error("The acquired editor no longer owns its source.")
            check(lease.current(path)&&(wasRetained||target.sourceRevision==0L&&target.text==target.base)&&target.text==source.text&&!target.saving){"Original source owners or the acquired target changed. Prepare again."};lease.retain(path,target);source
        }
        val actionKind=if(mode=="fixAll")"fixAll" else if(mode=="sourceActions")"sourceActions" else "actions"
        this.acquisition=load;actionQuery={val actions=propose(actionKind,"") as? JSONArray ?: JSONArray();val listed=(0 until actions.length()).mapNotNull{actions.optJSONObject(it)};scopedProviderActions(listed,actionKind)}
        plan=null; choices=emptyList(); referencePaths=emptyList(); notice="Preparing complete source changes…"
        val kind=mode ?: "actions"; val nextName=name
        try {
            if(proposal==null){language.readOutline(source);check(valid()){"Original source preparation changed. Prepare again."}}
            if (kind == "rename") {
                val acquired=acquireReferenceSources(sources,{
                    semanticLocations(propose("references","")).map{it.path ?: error("Reference target has no writable workspace source identity. No edits were applied.")}
                },load,valid)
                if(!valid())return;sources=acquired.sources;referencePaths=acquired.references
            }
            val result=propose(kind,nextName)
            if (!valid()) { if (revision==request) notice="Source, selection or provider changed. Prepare again."; return }
            if (kind == "actions" || kind == "fixAll" || kind == "sourceActions") {
                val actions=result as? JSONArray ?: JSONArray(); choices=(0 until actions.length()).mapNotNull { actions.optJSONObject(it)?.let { item -> JSONObject(item.toString()) } }
                choices=scopedProviderActions(choices,kind)
                notice=if(kind=="fixAll"&&choices.isEmpty()) "The current provider offers no file-wide diagnostic fixes. No edits were applied." else if(kind=="sourceActions"&&choices.isEmpty()) "No source cleanup or generation actions are available here. No edits were applied." else if (choices.isEmpty()) "No provider actions here. Support depends on the installed language server." else "Choose a provider action to preview."
            } else if(proofRequired){
                var first:Any?=result
                val acquired=acquireProviderAction(workspace,sources,{
                    val value=first?.also{first=null} ?: propose(kind,nextName);val action=value as? JSONObject ?: error("Provider did not return a source-proved action envelope.")
                    check(action.opt("source_files_modified") is Boolean&&action.getBoolean("source_files_modified")==false){"Provider did not confirm one complete no-effect source proposal."};action
                },load,valid,true)
                sources=acquired.sources;val action=acquired.value;install(action.optString("title","Provider source change"),action.get("edit"),workspace)
            } else install(if (kind == "rename") "Rename to $nextName" else "Format source",result,workspace)
        } catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { if (revision==request) notice=failure.message.orEmpty() }
    }
    private suspend fun prepareCleanup(workspace:Workspace,proposal:(suspend (String,String)->Any)?) {
        val origin=workspace.currentBuffer
        if(origin==null||origin.readOnly||origin.saving){notice="Choose writable source before preparing cleanup.";return}
        val request=++revision;val source=origin.text;val selection=origin.state.selection;val path=origin.file.path
        sources=workspace.sourceSnapshots();current={revision==request&&workspace.buffers[path]===origin&&origin.text==source&&origin.state.selection==selection}
        val originCurrent=current;val lease=workspace.sourceAcquisitionLease(external=originCurrent);current={lease.current()}
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
        val request=++revision;val lease=workspace.sourceAcquisitionLease();sources=snapshots;current={revision==request&&lease.current()&&targets.all{(buffer,text,selection)->workspace.buffers[buffer.file.path]===buffer&&buffer.text==text&&buffer.state.selection==selection}}
        mode="proposal";plan=next;choices=emptyList();referencePaths=emptyList();notice="";file=0;page=0;actionPage=0
    }
    private fun resourceEdit(edit:JSONObject)=edit.optJSONArray("documentChanges")?.let{rows->(0 until rows.length()).any{rows.optJSONObject(it)?.has("kind")==true}}==true
    private suspend fun providerResourcePreimages(workspace:Workspace,edit:JSONObject,valid:()->Boolean):Set<String>{
        check(valid()){"Original source workspace changed. Prepare again."};val engine=workspace.engine ?: error("Connect to the original workspace first.")
        val observed=engine.request("POST","/api/changes/resource-targets",JSONObject().put("edit",edit));val values=observed.getJSONArray("paths");val paths=(0 until values.length()).map{values.getString(it)}
        check(valid()&&observed.get("version")==1&&paths.isNotEmpty()&&paths.size<=1000&&paths.distinct().size==paths.size){"Complete resource source identities changed. Prepare again."}
        val (_,inputs,current)=workspace.resourceChanges.acquire(workspace,paths,edit,valid)
        check(valid()&&current()){"Original resource preimages changed. Prepare again."}
        val sources=inputs.getJSONObject("sources");return sources.keys().asSequence().filter{sources.getJSONObject(it).getString("kind")=="file"}.toSet()
    }
    private suspend fun acquireProviderAction(workspace:Workspace,initial:Map<String,SourceSnapshot>,query:suspend ()->JSONObject,load:suspend (String)->SourceSnapshot,valid:()->Boolean,proofs:Boolean):SourceProposalAcquisition<JSONObject>{
        var writable=emptySet<String>()
        val acquired=acquireSourceProposal(initial,{
            val action=query();check(valid()){"Source, selection or provider changed. Prepare again."}
            action.optJSONObject("disabled")?.let{error(it.optString("reason","Provider action disabled"))}
            check(action.isNull("command")){"This action requires a provider command; no edit portion was applied."}
            val proofPaths=if(proofs)providerSourceProofPaths(action) else emptyList()
            val pending=if(action.optJSONObject("data")?.has("hickory_acquire")==true)providerActionPaths(action) else emptyList()
            val edit=action.optJSONObject("edit");val edited=if(edit!=null&&!resourceEdit(edit))workspaceEditTargets(edit).map{it.path} else emptyList();writable=edited.toSet()
            val paths=proofPaths+pending+edited
            SourceProposal(if(proofs)paths else paths.filter{it !in initial},action)
        },load,valid,{it in writable})
        check(valid()){"Source, selection or provider changed. Prepare again."};val edit=acquired.value.optJSONObject("edit") ?: error("Provider source acquisition did not produce a complete edit. Prepare again.")
        if(proofs){val required=if(resourceEdit(edit))providerResourcePreimages(workspace,edit,valid) else workspaceEditTargets(edit).map{it.path}.toSet();validateProviderSourceProofs(acquired.value,acquired.sources,required);for(path in required)check(acquired.sources[path]?.writable==true){"Source is generated, read-only or saving: $path"}}
        return acquired
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
            if(mode=="fixAll"||mode=="sourceActions")check(first?.let{scopedProviderActions(listOf(it),mode).isNotEmpty()}==true){"The provider changed the complete file-wide action kind. Prepare again."}
            val acquired=acquireProviderAction(workspace,initialSources,{
                val resolved=first?.also{first=null} ?: run {
                    val candidates=queryActions().filter{it.optString("title")==action.optString("title")&&it.optString("kind")==action.optString("kind")}
                    check(candidates.size==1){"The selected provider action changed or is ambiguous. Prepare actions again."}
                    resolve(candidates.single())
                }
                check(revision==request&&choiceRevision==choice&&valid()){"Source, selection or provider changed. Prepare actions again."}
                if(mode=="fixAll"||mode=="sourceActions")check(scopedProviderActions(listOf(resolved),mode).isNotEmpty()){"The provider changed the complete file-wide action kind. Prepare again."}
                resolved.optJSONObject("disabled")?.let{error(it.optString("reason","Provider action disabled"))}
                check(resolved.isNull("command")){"This action requires a provider command; its edit has not been applied."}
                resolved
            },load,{revision==request&&choiceRevision==choice&&valid()},requiresProviderProofs)
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
            undo.add(History.Draft(captureDraftHistory(invertChangePlan(next),workspace))); while(undo.size>32) undo.removeAt(0); redo.clear();historyRevision++; plan=null; current={false}
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

Each Move form owns a fresh state-based text field. Its native input session follows the dialog window focus when the previous editor gives up the keyboard.
Request the software keyboard on the next composition frame after the field gains focus so the dialog can receive input after the old editor releases it. Prepare reads the same field used by the IME and disallows provisional composing text. Keep that field disabled during review preparation and bind the request to its committed destination and original entry selection. A late response or an edit to a retired form cannot authorize review.

<ui:copy id="simple-android-workspace-changes-presentation">
@Composable fun WorkspaceChangesView(workspace: Workspace) {
    val changes=workspace.changes; val scope=rememberCoroutineScope();val actionSnapshot=changes.actionSnapshot
    Column(Modifier.widthIn(min=260.dp,max=400.dp).heightIn(max=480.dp).verticalScroll(rememberScrollState()).padding(8.dp)) {
        Text("Workspace change preview",fontSize=16.sp); Text("Source drafts · not saved or verified",fontSize=11.sp)
        TextButton(onClick={changes.close(workspace)}) { Text("Close") }
        if (changes.mode=="rename") OutlinedTextField(changes.name,{changes.updateName(it)},label={Text("New symbol name")},singleLine=true)
        if(changes.mode=="entryMove")changes.entrySelection?.entryPath?.let{path->
            Text("Current main construction $path")
            val destination = changes.entryDestination
            val editing = !changes.preparingEntryMove && !workspace.resourceChanges.hasPending
            val keyboard = LocalSoftwareKeyboardController.current
            var destinationFocused by remember(destination) { mutableStateOf(false) }
            LaunchedEffect(destinationFocused) {
                if (destinationFocused) {
                    withFrameNanos { }
                    keyboard?.show()
                }
            }
            WithDisabledReason(if (!(editing)) "The Move destination is no longer editable. Reopen the review." else null) { available -> OutlinedTextField(
                state = destination,
                modifier = Modifier.onFocusChanged { destinationFocused = it.isFocused },
                label = { Text("New workspace-relative path") },
                lineLimits = TextFieldLineLimits.SingleLine,
                keyboardOptions = KeyboardOptions(autoCorrectEnabled = false, imeAction = ImeAction.Done),
                enabled = available,
            ) }
            WithDisabledReason(if (!(editing)) "The Move destination is no longer editable. Reopen the review." else if (workspace.resourceChanges.busy) "A resource change is in progress." else if (destination.composition != null) "Finish composing the path with the keyboard." else if (!(destination.text.isNotEmpty())) "Enter a destination path." else if (!(!destination.text.contentEquals(path))) "Enter a different destination path." else null) { available -> TextButton(
                enabled = available,
                onClick = { scope.launch { changes.prepare(workspace) } },
            ) { Text("Prepare move") } }
        }
        if (changes.mode!="undo" && changes.mode!="redo" && changes.mode!="proposal" && changes.mode!="entryMove") WithDisabledReason(if (!(changes.mode!="rename" || changes.name.isNotBlank())) "Enter a new symbol name." else null) { available -> TextButton(enabled = available,onClick={scope.launch {changes.prepare(workspace)}}) { Text(if(changes.mode=="fixAll") "Prepare file-wide diagnostic fixes" else if(changes.mode=="sourceActions") "Prepare source actions" else if(changes.mode=="actions") "Prepare provider actions" else if(changes.mode=="format") "Prepare formatting" else if(changes.mode=="cleanup") "Prepare cleanup" else "Prepare rename") } }
        if (changes.notice.isNotEmpty()) Text(changes.notice,fontSize=12.sp)
        for(index in changes.choices.indices.drop(changes.actionPage*100).take(100)) {
            val action=changes.choices[index]
            TextButton(onClick={scope.launch {changes.choose(index,workspace,actionSnapshot)}}) { Text(action.optString("title","Provider action")) }
            action.optJSONObject("disabled")?.let { Text(it.optString("reason"),fontSize=11.sp) }
        }
        if(changes.choices.size>100) Row { WithDisabledReason(if (changes.actionPage <= 0) "You’re on the first page." else null) { available -> TextButton(enabled = available,onClick={changes.actionPage--}) {Text("Previous actions")} }; WithDisabledReason(if ((changes.actionPage+1)*100 >= changes.choices.size) "You’re on the last page." else null) { available -> TextButton(enabled = available,onClick={changes.actionPage++}) {Text("Next actions")} } }
        changes.plan?.let { plan ->
            Text("${plan.title} · ${plan.patches.size} source files")
            if(changes.mode=="rename") Text("Reference targets: ${changes.referencePaths.toSet().size} files · project analysis scope not established",fontSize=11.sp)
            var expanded by remember {mutableStateOf(false)}
            val selectedFile=minOf(changes.file,plan.patches.lastIndex); val patch=plan.patches[selectedFile]
            Box { TextButton(onClick={expanded=true}) {Text(patch.path)}; DropdownMenu(expanded,{expanded=false}) {plan.patches.forEachIndexed {index,item->DropdownMenuItem(text={Text(item.path)},onClick={changes.file=index;changes.page=0;expanded=false})} } }
            val before=patch.before.split("\n"); val after=patch.after.split("\n"); val last=maxOf(0,(maxOf(before.size,after.size)+99)/100-1); val page=minOf(changes.page,last)
            Text("Before · lines ${page*100+1}–${minOf(before.size,(page+1)*100)}",fontSize=11.sp); Text(before.drop(page*100).take(100).joinToString("\n"),fontSize=12.sp,fontFamily=androidx.compose.ui.text.font.FontFamily.Monospace)
            Text("After",fontSize=11.sp); Text(after.drop(page*100).take(100).joinToString("\n"),fontSize=12.sp,fontFamily=androidx.compose.ui.text.font.FontFamily.Monospace)
            if(last>0) Row {WithDisabledReason(if (page <= 0) "You’re on the first page." else null) { available -> TextButton(enabled = available,onClick={changes.page=page-1}) {Text("Previous lines")} }; Text("${page+1}/${last+1}"); WithDisabledReason(if (page >= last) "You’re on the last page." else null) { available -> TextButton(enabled = available,onClick={changes.page=page+1}) {Text("Next lines")} } }
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
 @Test fun workspaceHistoryFollowsOriginalEditorsAcrossSaveAndAddressChanges()=runBlocking {
  val w=Workspace(this);val f=FileNode("a.py","a.py",false,emptyList(),null,null);val b=DocumentBuffer(f,FileText("old\n","base"));w.buffers[f.path]=b;w.show(f.path)
  val edit=JSONObject().put("changes",JSONObject().put(languageURI(f.path),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",3))).put("newText","new"))))
  w.changes.offer("Reviewed",edit,mapOf(f.path to b.text),w);w.changes.apply(w);assertEquals("new\n",b.text);b.base=b.text;b.hash="saved";b.saveGuard="confirmed-save";w.buffers.remove(f.path);b.file=FileNode("files","files",false,emptyList(),null,null);w.buffers["files"]=b
  val replacement=DocumentBuffer(f,FileText("new\n","base"));w.buffers[f.path]=replacement;w.changes.open("undo",w);assertEquals("old\n",b.text);assertEquals("new\n",replacement.text);assertEquals("new\n",b.base);w.changes.open("redo",w);assertEquals("new\n",b.text)
  w.buffers["alias.py"]=b;w.changes.open("undo",w);assertEquals("new\n",b.text);assertTrue(w.changes.notice.contains("original source editors"));w.buffers.remove("alias.py")
  val foreign=DocumentBuffer(b.file,FileText(b.text,b.hash));w.buffers["files"]=foreign;w.changes.open("undo",w);assertEquals("new\n",foreign.text);assertEquals("new\n",b.text);assertTrue(w.changes.notice.contains("original source editors"));w.buffers["files"]=b;w.changes.open("undo",w);assertEquals("old\n",b.text);assertEquals("new\n",foreign.text);assertEquals("new\n",replacement.text)
 }

 @Test fun cleanupAndReviewedDraftRefuseEqualByteRevisionRenewal()=runBlocking {
  val w=Workspace(this);val f=FileNode("a.py","a.py",false,emptyList(),null,null);val b=DocumentBuffer(f,FileText("old\n","base"));w.buffers[f.path]=b;w.show(f.path)
  val edit=JSONObject().put("changes",JSONObject().put(languageURI(f.path),org.json.JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",3))).put("newText","new"))))
  fun editUndo(){check(b.applySourceEdits(listOf(NativeSourceEdit(0,b.text.length,"later")),b.text));check(b.applySourceEdits(listOf(NativeSourceEdit(0,b.text.length,"old\n")),b.text))}
  w.changes.offer("Reviewed",edit,mapOf(f.path to b.text),w);editUndo();w.changes.apply(w);assertEquals("old\n",b.text);assertTrue(w.changes.notice.contains("Source, selection or provider changed"))
  w.changes.open("cleanup",w);w.changes.prepare(w,{_,_->editUndo();edit});assertNull(w.changes.plan);assertEquals("old\n",b.text);assertTrue(w.changes.notice.contains("Source or selection changed"))
 }

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
 @Test fun originalTargetOwnersNeverRenewDuringQueries()=runBlocking {
  for(change in listOf("undo","guard","baseline","handle","unknown")){
   val w=Workspace(this);val e=Engine("http://127.0.0.1:1");val f=FileNode("a.py","a.py",false,emptyList(),null,null);val other=FileNode("other.py","other.py",false,emptyList(),null,null);val b=DocumentBuffer(f,FileText("old\n","base"));val target=DocumentBuffer(other,FileText("old\n","base"));b.language=NativeLanguage(e,b,this);w.buffers[f.path]=b;w.buffers[other.path]=target;w.show(f.path)
   val action=JSONObject().put("title","Fix all").put("kind","source.fixAll.provider");w.changes.open("fixAll",w);w.changes.prepare(w,{_,_->when(change){"undo"->{target.state.edit{replace(0,length,"later")};target.advanceSourceRevision();target.state.edit{replace(0,length,"old\n")};target.advanceSourceRevision()};"guard"->target.saveGuard="foreign";"baseline"->target.base="foreign";"handle"->w.buffers[other.path]=DocumentBuffer(other,FileText(target.text,target.hash));else->w.buffers["unknown.md"]=DocumentBuffer(FileNode("unknown.md","unknown.md",false,emptyList(),null,null),FileText("unmodeled construction","new"))};JSONArray().put(action)});assertTrue(change,w.changes.choices.isEmpty());assertNull(w.changes.plan);assertEquals("old\n",b.text)
  }
 }
 @Test fun acquiredEditUndoCannotBecomeANewProviderBaseline()=runBlocking {
  val w=Workspace(this);val e=Engine("http://127.0.0.1:1");val f=FileNode("a.py","a.py",false,emptyList(),null,null);val b=DocumentBuffer(f,FileText("old\n","base"));b.language=NativeLanguage(e,b,this);w.buffers[f.path]=b;w.show(f.path)
  val action=JSONObject().put("title","Fix all").put("kind","source.fixAll.provider").put("data",JSONObject().put("hickory_acquire",JSONArray().put(JSONObject().put("uri",languageURI("other.py")))))
  w.changes.open("fixAll",w);w.changes.prepare(w,{_,_->JSONArray().put(action)},{it},{path->val target=DocumentBuffer(FileNode(path,path,false,emptyList(),null,null),FileText("old\n","base"));w.buffers[path]=target;target.state.edit{replace(0,length,"later")};target.advanceSourceRevision();target.state.edit{replace(0,length,"old\n")};target.advanceSourceRevision();SourceSnapshot(target.text,true)});w.changes.choose(0,w);assertNull(w.changes.plan);assertEquals("old\n",b.text);assertTrue(w.changes.notice,w.changes.notice.contains("acquired target")||w.changes.notice.contains("sealed"))
 }

 @Test fun liveOwnedBulkFixCommandReviewApplyAndUndo()=runBlocking {
  val origin=System.getenv("HICKORY_BULK_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_BULK_WORKSPACE") ?: error("Owned workspace required"));check(java.io.File(root,".hickory-bulk-fixture").readText()=="owned-bulk-v1\n")
  val ownedScope=CoroutineScope(SupervisorJob()+Dispatchers.Default);val w=Workspace(ownedScope);val e=Engine(origin!!);w.engine=e;val (_,tree)=e.files();w.rootFiles=tree
  try{val f=tree.first{it.path=="a.py"};val b=w.loadBuffer(f);w.show(f.path);val state=b.state;val source=b.text;val scroll=b.sourceScroll;b.state.edit{selection=androidx.compose.ui.text.TextRange(2)};val selection=b.state.selection
   val descriptors=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()});val command=w.commands(descriptors).first{it.id=="symbol.fixAll"};assertNull(command.reason);command.run();assertEquals("fixAll",w.changes.mode)
   withTimeout(20000){while(w.changes.choices.isEmpty()){w.changes.prepare(w);if(w.changes.choices.isEmpty())delay(100)}}
   assertEquals(1,w.changes.choices.size);w.changes.choose(0,w);assertEquals(w.changes.notice,2,w.changes.plan?.patches?.size);assertEquals(source,b.text);assertEquals(selection,b.state.selection);assertSame(state,b.state);assertSame(scroll,b.sourceScroll);assertEquals(source,java.io.File(root,"a.py").readText());assertEquals(source,java.io.File(root,"other.py").readText())
   w.changes.apply(w);assertEquals("python_fixed = 1\n",b.text);assertEquals("other_fixed = 1\n",w.buffers["other.py"]?.text);w.changes.open("undo",w);assertEquals(source,b.text);assertEquals(source,w.buffers["other.py"]?.text);w.changes.open("redo",w);assertEquals("python_fixed = 1\n",b.text);assertSame(state,b.state);assertSame(scroll,b.sourceScroll);assertEquals(source,java.io.File(root,"a.py").readText());assertEquals(source,java.io.File(root,"other.py").readText())
  }finally{w.buffers.values.forEach{it.language?.close()};ownedScope.cancel()}
 }

 @Test fun completeFileFixChoiceFiltersPreviewsAndUsesCoordinatedUndo()=runBlocking {
  val w=Workspace(this);val e=Engine("http://127.0.0.1:1");val f=FileNode("a.py","a.py",false,emptyList(),null,null);val b=DocumentBuffer(f,FileText("old\n","base"));b.language=NativeLanguage(e,b,this);w.buffers[f.path]=b;w.show(f.path)
  val action=JSONObject().put("title","Fix all").put("kind","source.fixAll.provider").put("edit",JSONObject().put("changes",JSONObject().put(languageURI(f.path),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",3))).put("newText","new")))))
  val choices=JSONArray().put(action).put(JSONObject().put("title","single").put("kind","quickfix")).put(JSONObject().put("title","unclassified")).put(JSONObject().put("title","prefix").put("kind","source.fixAllWrong"))
  w.changes.open("fixAll",w);w.changes.prepare(w,{kind,_->assertEquals("fixAll",kind);choices});assertEquals(1,w.changes.choices.size);w.changes.choose(0,w);assertEquals("new\n",w.changes.plan?.patches?.first()?.after);assertEquals("old\n",b.text)
  w.changes.apply(w);assertEquals("new\n",b.text);w.changes.open("undo",w);assertEquals("old\n",b.text);w.changes.open("redo",w);assertEquals("new\n",b.text)
  w.changes.open("fixAll",w);w.changes.prepare(w,{_,_->JSONArray().put(JSONObject().put("kind","quickfix"))});assertTrue(w.changes.choices.isEmpty());assertTrue(w.changes.notice.contains("no file-wide"))
  w.changes.open("fixAll",w);w.changes.prepare(w,{_,_->b.state.edit{replace(0,length,"later")};b.advanceSourceRevision();b.state.edit{replace(0,length,"new\n")};b.advanceSourceRevision();JSONArray().put(action)});assertTrue(w.changes.choices.isEmpty());assertNull(w.changes.plan)
  w.changes.open("fixAll",w);w.changes.prepare(w,{_,_->JSONArray().put(action)},{JSONObject(action.toString()).put("kind","quickfix")});w.changes.choose(0,w);assertNull(w.changes.plan);assertEquals("new\n",b.text)
 }

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
  workspace.buffers.getValue("other.py").state.edit{replace(0,length,"later\n")};changes.apply(workspace);assertTrue(changes.notice.contains("Source, selection or provider changed"));assertEquals("old\n",buffer.text)
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
 @Test fun providerProofsSynchronizeReadonlyContextAndRefuseUnprovedOrChangedEdits()=runBlocking {
  for(mode in listOf("success","hash","missing","unproved","readonly")){
   val w=Workspace(this);val e=Engine("http://127.0.0.1:1");val f=FileNode("a.py","a.py",false,emptyList(),null,null);val b=DocumentBuffer(f,FileText("old\n","base"));b.language=NativeLanguage(e,b,this);w.buffers[f.path]=b;w.show(f.path);var queries=0;val loaded=mutableListOf<String>()
   fun action():JSONObject{val target=if(mode=="readonly"&&queries>1)"context.py" else "a.py";val replacement=JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",3))).put("newText","new");val rows=JSONArray();if(mode!="unproved")rows.put(JSONObject().put("uri",languageURI("a.py")).put("source_hash","sha256:"+if(mode=="hash")"0".repeat(64) else debugHash("old\n")));rows.put(JSONObject().put("uri",languageURI("context.py")).put("source_hash","sha256:"+debugHash("old\n")));return JSONObject().put("title","Proof fix").put("kind","quickfix").put("edit",JSONObject().put("changes",JSONObject().put(languageURI(target),JSONArray().put(replacement)))).also{if(mode!="missing")it.put("data",JSONObject().put("hickory_action","owned").put("hickory_sources",rows))}}
   w.changes.open("actions",w);w.changes.prepare(w,{_,_->queries++;JSONArray().put(action())},{it},{path->loaded.add(path);val current=w.buffers[path];if(current!=null)SourceSnapshot(current.text,!current.readOnly) else {val target=DocumentBuffer(FileNode(path,path,false,emptyList(),null,"construction.md"),FileText("old\n","base"));w.buffers[path]=target;SourceSnapshot(target.text,false)}},enforceProviderProofs=true);w.changes.choose(0,w);assertEquals(mode,"old\n",b.text);assertEquals("a.py",w.currentBuffer?.file?.path)
   if(mode=="success"){assertEquals(w.changes.notice,"new\n",w.changes.plan?.patches?.first()?.after);assertTrue(loaded.contains("context.py"));assertTrue(w.buffers["context.py"]?.readOnly==true);w.changes.apply(w);assertEquals("new\n",b.text);w.changes.open("undo",w);assertEquals("old\n",b.text)}else{assertNull(mode+": "+w.changes.notice,w.changes.plan);assertTrue(mode,w.changes.notice.isNotEmpty())}
  }
 }
 @Test fun boundFormattingRequiresTypedNoEffectAndSourceProofs()=runBlocking {
  for(effect in listOf<Any>(false,true,0,"false",JSONObject.NULL)){
   val w=Workspace(this);val e=Engine("http://127.0.0.1:1");val f=FileNode("a.py","a.py",false,emptyList(),null,null);val b=DocumentBuffer(f,FileText("old\n","base"));b.language=NativeLanguage(e,b,this);w.buffers[f.path]=b;w.show(f.path)
   val edit=JSONObject().put("changes",JSONObject().put(languageURI(f.path),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",3))).put("newText","new"))))
   val action=JSONObject().put("title","Bound format").put("edit",edit).put("data",JSONObject().put("hickory_action","owned").put("hickory_sources",JSONArray().put(JSONObject().put("uri",languageURI(f.path)).put("source_hash","sha256:"+debugHash(b.text))))).put("source_files_modified",effect)
   w.changes.open("format",w);w.changes.prepare(w,{_,_->action},acquisition={SourceSnapshot(b.text,true)},enforceProviderProofs=true);if(effect is Boolean&&!effect)assertEquals(w.changes.notice,"new\n",w.changes.plan?.patches?.first()?.after)else{assertNull(w.changes.plan);assertTrue(w.changes.notice,w.changes.notice.contains("no-effect"))};assertEquals("old\n",b.text)
  }
 }

 @Test fun liveOwnedBoundRenameFormatPreviewUndoAndStaleRefusal()=runBlocking {
  val address=System.getenv("HICKORY_NATIVE_BOUND_ORIGIN");org.junit.Assume.assumeNotNull(address);val root=java.io.File(System.getenv("HICKORY_NATIVE_BOUND_WORKSPACE") ?: error("Owned workspace required"));val events=java.io.File(System.getenv("HICKORY_NATIVE_BOUND_EVENTS") ?: error("Owned events required"));val gate=java.io.File(System.getenv("HICKORY_NATIVE_BOUND_GATE") ?: error("Owned gate required"));check(java.io.File(root,".hickory-native-bound-fixture").readText()=="owned-native-bound-v1\n"&&events.parentFile.canonicalFile==root.parentFile.canonicalFile&&gate.parentFile.canonicalFile==root.parentFile.canonicalFile)
  val ownedScope=CoroutineScope(SupervisorJob()+Dispatchers.Default);val w=Workspace(ownedScope);val e=Engine(address!!);w.engine=e;val (_,tree)=e.files();w.rootFiles=tree
  try{val f=tree.first{it.path=="a.py"};val b=w.loadBuffer(f);w.show(f.path);val source=b.text;val state=b.state;val scroll=b.sourceScroll;b.state.edit{selection=androidx.compose.ui.text.TextRange(2)};val selection=b.state.selection
   val descriptors=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()})
   fun command(id:String){val item=w.commands(descriptors).first{it.id==id};check(item.reason==null){item.reason.orEmpty()};item.run()}
   suspend fun prepare(id:String,name:String){command(id);w.changes.updateName(name);withTimeout(20000){while(w.changes.plan==null){w.changes.prepare(w);if(w.changes.plan==null)delay(30)}}}
   prepare("symbol.rename","native_renamed");assertEquals(source,w.changes.plan?.patches?.first()?.before);assertEquals(source,b.text);assertEquals(selection,b.state.selection);w.changes.apply(w);assertEquals(w.changes.notice,"native_renamed = 1\n",b.text);w.changes.open("undo",w);assertEquals(source,b.text);w.changes.open("redo",w);assertEquals("native_renamed = 1\n",b.text);w.changes.open("undo",w);assertEquals(source,b.text)
   prepare("document.format","");assertEquals(source,b.text);w.changes.apply(w);assertEquals(w.changes.notice,"formatted = 1\n",b.text);w.changes.open("undo",w);assertEquals(source,b.text);w.changes.open("redo",w);assertEquals("formatted = 1\n",b.text);w.changes.open("undo",w);assertEquals(source,b.text);assertSame(state,b.state);assertSame(scroll,b.sourceScroll)
   command("symbol.rename");w.changes.updateName("held_native");val held=async{w.changes.prepare(w)};withTimeout(20000){while(!events.exists()||!events.readText().contains("\"held\": \"held_native\""))delay(10)}
   b.state.edit{replace(length,length,"!")};b.advanceSourceRevision();b.state.edit{replace(0,length,source)};b.advanceSourceRevision();gate.writeText("release");held.await();assertNull(w.changes.plan);assertTrue(w.changes.notice.isNotEmpty());assertEquals(source,b.text);assertSame(state,b.state);assertSame(scroll,b.sourceScroll);assertEquals(source,java.io.File(root,"a.py").readText())
  }finally{w.buffers.values.forEach{it.language?.close()};ownedScope.cancel()}
 }

 @Test fun liveOwnedLiterateBulkFixAcquiresContributorAndKeepsProseEditor()=runBlocking {
  val address=System.getenv("HICKORY_NATIVE_BOUND_ORIGIN");org.junit.Assume.assumeNotNull(address);val root=java.io.File(System.getenv("HICKORY_NATIVE_BOUND_WORKSPACE") ?: error("Owned fixture required"));check(java.io.File(root,".hickory-native-bound-fixture").readText()=="owned-native-bound-v1\n");val scope=CoroutineScope(SupervisorJob()+Dispatchers.Default);val w=Workspace(scope);val e=Engine(address!!);w.engine=e;val (_,tree)=e.files();w.rootFiles=tree
  try{val f=tree.first{it.path=="project.md"};val b=w.loadBuffer(f);w.show(f.path);val source=b.text;val contributor=java.io.File(root,"a.md").readText();val state=b.state;val scroll=b.sourceScroll;b.state.edit{selection=androidx.compose.ui.text.TextRange(source.indexOf("# Readable")+3)};val selection=b.state.selection;assertNull(w.buffers["a.md"])
   val descriptors=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()});val command=w.commands(descriptors).first{it.id=="symbol.fixAll"};assertNull(command.reason);command.run()
   withTimeout(20000){while(w.changes.choices.none{it.optString("title").endsWith(" · out/a.py")}){w.changes.prepare(w);if(w.changes.choices.none{it.optString("title").endsWith(" · out/a.py")})delay(30)}}
   val index=w.changes.choices.indexOfFirst{it.optString("title").endsWith(" · out/a.py")};w.changes.choose(index,w);assertEquals(w.changes.notice,listOf("a.md"),w.changes.plan?.patches?.map{it.path});assertEquals(contributor,w.buffers["a.md"]?.text);assertEquals(source,b.text);assertEquals(selection,b.state.selection);assertEquals("project.md",w.currentBuffer?.file?.path)
   w.changes.apply(w);assertEquals(w.changes.notice,contributor.replace("value","fixed"),w.buffers["a.md"]?.text);w.changes.open("undo",w);assertEquals(contributor,w.buffers["a.md"]?.text);w.changes.open("redo",w);assertEquals(contributor.replace("value","fixed"),w.buffers["a.md"]?.text);w.changes.open("undo",w)
   assertEquals(source,b.text);assertEquals(selection,b.state.selection);assertSame(state,b.state);assertSame(scroll,b.sourceScroll);assertEquals(source,java.io.File(root,"project.md").readText());assertEquals(contributor,java.io.File(root,"a.md").readText());assertFalse(java.io.File(root,"out").exists())
  }finally{w.buffers.values.forEach{it.language?.close()};scope.cancel()}
 }


 @Test fun completeSourceActionScopeNeverBorrowsQuickFixes()=runBlocking {
  val w=Workspace(this);val engine=Engine("http://127.0.0.1:1");val file=FileNode("a.py","a.py",false,emptyList(),null,null);val b=DocumentBuffer(file,FileText("old\n","base"));b.language=NativeLanguage(engine,b,this);w.buffers[file.path]=b;w.show(file.path)
  try{val action=JSONObject().put("title","Source cleanup").put("kind","source.organizeImports").put("edit",JSONObject().put("changes",JSONObject().put(languageURI(file.path),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",3))).put("newText","new")))))
   w.changes.open("sourceActions",w);w.changes.prepare(w,{kind,_->assertEquals("sourceActions",kind);JSONArray().put(action).put(JSONObject().put("title","single").put("kind","quickfix")).put(JSONObject().put("title","wrong").put("kind","sourceWrong"))},{JSONObject(action.toString()).put("kind","quickfix")});assertEquals(1,w.changes.choices.size);w.changes.choose(0,w);assertNull(w.changes.plan);assertTrue(w.changes.notice,w.changes.notice.contains("kind"));assertEquals("old\n",b.text)
   w.changes.open("sourceActions",w);w.changes.prepare(w,{_,_->JSONArray().put(JSONObject().put("title","single").put("kind","quickfix"))});assertTrue(w.changes.choices.isEmpty());assertTrue(w.changes.notice,w.changes.notice.contains("No source cleanup"));assertEquals("old\n",b.text)
  }finally{b.language?.close()}
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/ActionChoiceTest.kt"><ui:paste select="#native-action-choice-android-witness"/></ui:file>
</ui:doc>
