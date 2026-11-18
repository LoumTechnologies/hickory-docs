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
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import java.net.URLEncoder
import org.json.JSONObject

// Preparation and publication are separate states. A failed POST is never
// retried: only the original request's read-only status may be consulted.
class ResourceChangeReview(private val transport:(suspend(String,String,JSONObject?)->JSONObject)?=null) {
 data class Review(val proposal:JSONObject,val facts:JSONObject,val plan:JSONObject,val saved:Map<String,ResourceSaved>,val restoration:Boolean,val fingerprint:String?,val current:()->Boolean,val restoring:Map<String,DocumentBuffer>,val published:((String,String)->Unit)?=null,val preflight:(suspend ()->Unit)?=null,val entryIntent:WorkspaceEntryIntent?=null)
 var review by mutableStateOf<Review?>(null);private set
 var notice by mutableStateOf("");private set
 var busy by mutableStateOf(false);private set
 var lastRecovery by mutableStateOf<String?>(null);private set
 var lastWorkspace by mutableStateOf<String?>(null);private set
 private var ticket=0L
 private var pending:Pair<String,Review>?=null
 private var publishedWorkspace:Workspace?=null
 private fun nonce()=java.util.UUID.randomUUID().toString().replace("-","")
 private fun hash(text:String)=java.security.MessageDigest.getInstance("SHA-256").digest(text.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}
 private suspend fun call(workspace:Workspace,method:String,path:String,body:JSONObject?=null)=transport?.invoke(method,path,body) ?: (workspace.engine ?: error("Connect to a workspace server first.")).request(method,path,body)
 suspend fun readEntry(workspace:Workspace,current:()->Boolean):FrozenWorkspaceEntrySelection {
  val server=workspace.engine ?: error("Connect to a workspace server first.")
  val runtime=server.sourceWorkspaceID ?: error("Refresh Files to identify the current workspace.")
  val namespace=server.sourceRootAddress ?: error("Refresh Files to identify the current workspace.")
  val epoch=workspace.sourceNavigationEpoch
  val valid={current()&&workspace.engine===server&&workspace.sourceNavigationEpoch==epoch&&server.sourceWorkspaceID==runtime&&server.sourceRootAddress==namespace}
  return WorkspaceEntrySelections.read({call(workspace,"GET","/api/workspace/entry")},runtime,namespace,valid)
 }
 private suspend fun entryIntent(workspace:Workspace,selection:FrozenWorkspaceEntrySelection,kind:String,operation:JSONObject,path:String?,current:()->Boolean):WorkspaceEntryIntent {
  check(selection.current()&&current()){"The original entry review changed. Prepare again."}
  val body=JSONObject().put("version",1).put("entry_selection",selection.proof.body()).put("operation",operation.put("kind",kind))
  val reply=call(workspace,"POST","/api/workspace/entry/intent",body)
  check(selection.current()&&current()){"The original entry review changed. Prepare again."}
  return WorkspaceEntryIntent(reply,selection.proof,kind,path)
 }
 private suspend fun readInputs(workspace:Workspace,capture:JSONObject,entry:WorkspaceEntryIntent?):JSONObject {
  if(entry==null)return call(workspace,"POST","/api/changes/resource-inputs",capture)
  return call(workspace,"POST","/api/workspace/entry/inputs",JSONObject().put("version",1).put("token",entry.token).put("capture",capture))
 }
 suspend fun prepareEntryMove(workspace:Workspace,selection:FrozenWorkspaceEntrySelection,destination:String,valid:()->Boolean) {
  if(busy||pending!=null){notice="Inspect the pending publication before preparing another resource proposal.";return}
  val original=workspace.captureRetainedSourceNamespace()
  notice="Preparing the main construction move…"
  try {
   val intent=entryIntent(workspace,selection,"move",JSONObject().put("destination",destination),destination){valid()&&original()}
   prepareResourceReview(workspace,intent.title,intent.edit(),{selection.current()&&valid()},intent)
  }catch(cancelled:CancellationException){throw cancelled}catch(error:Exception){if(valid())notice=error.message.orEmpty()}
 }
 internal fun acceptSealed(workspace:Workspace,proposal:JSONObject,facts:JSONObject,plan:JSONObject,current:()->Boolean,preflight:suspend ()->Unit,published:((String,String)->Unit)?=null):Review {
  check(!busy&&pending==null&&current()){ "Finish the current publication and refresh the complete proposal." }
  val copied=JSONObject(proposal.toString());val reviewedFacts=JSONObject(facts.toString());val reviewedPlan=JSONObject(plan.toString());val paths=resourcePatches(reviewedPlan).map{it.path}
  check(paths.isNotEmpty()&&paths.distinct().size==paths.size&&reviewedPlan.get("version")==1&&copied.getJSONObject("inputs").getString("context")==reviewedPlan.getString("context")&&reviewedFacts.getString("context")==reviewedPlan.getString("context")&&reviewedFacts.getString("workspace")==workspace.engine?.sourceWorkspaceID&&reviewedFacts.getJSONArray("protected_targets").length()==0){"Sealed resource consequences do not match this workspace."}
  val saved=resourceSavedMap(reviewedFacts.getJSONObject("saved_sources"));check(saved.keys==paths.toSet()){ "Sealed proposal does not contain the complete saved namespace." }
  ticket++;return Review(copied,reviewedFacts,reviewedPlan,saved,false,null,current,emptyMap(),published,preflight).also{review=it;notice="Complete sealed proposal staged for guarded Save. No file was written."}
 }
 fun dismiss(){if(pending!=null)return;ticket++;review=null;busy=false}
 fun reset(){ticket++;review=null;pending=null;publishedWorkspace=null;lastRecovery=null;lastWorkspace=null;busy=false;notice=""}
 private suspend fun capture(workspace:Workspace,paths:List<String>,edit:JSONObject,valid:()->Boolean,entry:WorkspaceEntryIntent?=null):Pair<JSONObject,()->Boolean> {
  check(paths.isNotEmpty()&&paths.size<=1000&&paths.distinct().size==paths.size){"Choose a bounded complete source proposal."}
  val epoch=workspace.sourceNavigationEpoch;val server=workspace.engine
  fun find(nodes:List<FileNode>,path:String):FileNode?=nodes.firstNotNullOfOrNull{if(it.path==path)it else find(it.children,path)}
  if(entry==null)for(path in paths){check(valid()&&workspace.sourceNavigationEpoch==epoch){"Source or workspace changed. Prepare again."};find(workspace.rootFiles,path)?.takeUnless{it.dir}?.let{workspace.loadBuffer(it)}}
  val addresses=workspace.buffers.toMap();val retained=workspace.retainedResourceEditors.toMap();val tabs=workspace.retainedResourceTabs.toMap()
  fun roster()=(workspace.buffers.values+workspace.retainedResourceEditors.values+workspace.retainedResourceTabs.values).distinct().filter{it.file.generatedBy==null}
  data class Snapshot(val buffer:DocumentBuffer,val text:String,val base:String,val hash:String,val guard:String?,val file:FileNode,val revision:Long,val deleted:Boolean,val readonly:Boolean,val untitled:Boolean,val selection:androidx.compose.ui.text.TextRange)
  val snapshots=roster().map{check(!it.saving&&it.state.composition==null){"Finish source input and saves before preparing resources."};Snapshot(it,it.text,it.base,it.hash,it.saveGuard,it.file,it.sourceRevision,it.resourceDeleted,it.readOnly,it.isUntitledDraft,it.state.selection)}
  check(snapshots.size<=4096){"Retained source roster exceeds its bounded capture."}
  fun same(live:Map<String,DocumentBuffer>,before:Map<String,DocumentBuffer>)=live.size==before.size&&before.all{(path,b)->live[path]===b}
  val current={valid()&&workspace.engine===server&&workspace.sourceNavigationEpoch==epoch&&same(workspace.buffers,addresses)&&same(workspace.retainedResourceEditors,retained)&&same(workspace.retainedResourceTabs,tabs)&&roster().toSet()==snapshots.map{it.buffer}.toSet()&&snapshots.all{val b=it.buffer;b.sourceRevision==it.revision&&b.text==it.text&&b.base==it.base&&b.hash==it.hash&&b.saveGuard==it.guard&&b.file===it.file&&b.resourceDeleted==it.deleted&&b.readOnly==it.readonly&&b.isUntitledDraft==it.untitled&&b.state.selection==it.selection&&!b.saving&&b.state.composition==null}}
  check(current()){"Source changed. Prepare again."}
  val drafts=JSONObject();val ownership=JSONObject();val unaddressed=JSONObject();val guards=JSONObject();var bytes=0L
  for(s in snapshots){val named=!s.deleted&&!s.untitled&&!s.file.path.startsWith('/')&&addresses[s.file.path]===s.buffer
   val key=if(named)s.file.path else s.buffer.debugIdentity;check(s.text.length<=8*1024*1024&&key.length<=8192){"Retained source input exceeds its bounded capture."};bytes+=key.toByteArray(Charsets.UTF_8).size.toLong()+s.text.toByteArray(Charsets.UTF_8).size;check(bytes<=32L*1024*1024){"Retained resource ownership capture exceeds 32 MiB."}
   if(named){check(!ownership.has(key)){"Retained authored source addresses are ambiguous."};ownership.put(key,s.text);s.guard?.takeIf{it.isNotEmpty()}?.let{guards.put(key,it)}}else unaddressed.put(key,s.text)
   if(named&&key in paths){check(!s.readonly){"A selected source is not writable."};check(!s.guard.isNullOrEmpty()){"A selected source has no captured physical save guard. Reopen its saved baseline before preparing resources."};drafts.put(key,JSONObject().put("save_guard",s.guard).put("text",s.text).put("base_sha256",hash(s.base)).put("version",JSONObject.NULL).put("writable",true))}
  }
  val capture=JSONObject().put("context",entry?.context ?: nonce()).put("edit",edit).put("drafts",drafts).put("project_entry",JSONObject.NULL).put("ownership_sources",ownership).put("unaddressed_sources",unaddressed).put("saved_guards",guards)
  return capture to current
 }
 internal suspend fun acquire(workspace:Workspace,paths:List<String>,edit:JSONObject,valid:()->Boolean):Triple<JSONObject,JSONObject,()->Boolean> =
  acquireResourceInputs(workspace,paths,edit,valid,null)
 private suspend fun acquireResourceInputs(workspace:Workspace,paths:List<String>,edit:JSONObject,valid:()->Boolean,entry:WorkspaceEntryIntent?):Triple<JSONObject,JSONObject,()->Boolean> {
  val original=entry?.let{workspace.sourceAcquisitionLease(protectSelection=true,external=valid)}
  val acquireCurrent={valid()&&(original?.current() ?: true)}
  var (captured,current)=capture(workspace,paths,edit,acquireCurrent,entry)
  var inputs=readInputs(workspace,captured,entry)
  check(current()){"Source changed during acquisition. Prepare again."}
  fun requireSavedGuards(raw:JSONObject){
   val guards=raw.getJSONObject("saved_guards")
   check(guards.keys().asSequence().all{guards.get(it) is String&&guards.getString(it).isNotEmpty()}){"Resource inputs contain malformed physical save guards."}
   if(entry!=null)check(raw.opt("context")==entry.context&&raw.opt("source_files_modified")==false&&
    raw.getJSONObject("refusals").length()==0&&raw.getJSONObject("sources").keys().asSequence().toSet()==paths.toSet()){
    "The main construction inputs changed or refused a source."
   }
  }
  requireSavedGuards(inputs)
  val sources=inputs.getJSONObject("sources")
  val missing=paths.filter{sources.getJSONObject(it).getString("kind")=="file"&&workspace.buffers[it]==null}
  if(missing.isNotEmpty()){
   if(original!=null){
    fun find(nodes:List<FileNode>,path:String):FileNode?=nodes.firstNotNullOfOrNull{if(it.path==path)it else find(it.children,path)}
    for(path in missing){
     check(original.current()){"Original source owners changed during acquisition."}
     val saved=inputs.getJSONObject("saved_guards").opt(path) as? String ?: error("The main construction has no captured physical save guard.")
     val expected=sources.getJSONObject(path).getString("text")
     val file=find(workspace.rootFiles,path) ?: FileNode(path.substringAfterLast('/'),path,false,emptyList(),null,null)
     val buffer=workspace.loadBuffer(file)
     check(original.current(path)&&buffer.sourceRevision==0L&&buffer.text==expected&&buffer.base==expected&&buffer.saveGuard==saved){"The acquired source changed from its original physical observation."}
     original.retain(path,buffer)
    }
   }else{
    workspace.refresh()
    check(current()){"Workspace changed during source discovery. Prepare again."}
   }
   val next=capture(workspace,paths,edit,acquireCurrent,entry);captured=next.first;current=next.second
   check(missing.all{workspace.buffers[it]!=null}){"Current source is not available in the file index: ${missing.joinToString()}. Refresh files and prepare again."}
   inputs=readInputs(workspace,captured,entry);requireSavedGuards(inputs)
   check(current()){"Source changed after discovery. Prepare again."}
  }
  return Triple(captured,inputs,current)
 }
 suspend fun prepare(workspace:Workspace,title:String,edit:JSONObject,valid:()->Boolean={true}) {
  prepareResourceReview(workspace,title,edit,valid,null)
 }
 private suspend fun prepareResourceReview(workspace:Workspace,title:String,edit:JSONObject,valid:()->Boolean,entry:WorkspaceEntryIntent?) {
  if(busy||pending!=null){notice="Inspect the pending publication status before preparing another resource change.";return}
  val id=++ticket;busy=true;review=null;notice="Reading current resource authority…"
  try {
   val original=entry?.let{workspace.captureRetainedSourceNamespace()}
   val targets=call(workspace,"POST","/api/changes/resource-targets",JSONObject().put("edit",edit)).getJSONArray("paths");val paths=(0 until targets.length()).map{targets.getString(it)}
   check(original?.invoke()!=false){"Original source owners changed before acquisition."}
   val (capture,inputs,current)=acquireResourceInputs(workspace,paths,edit,valid,entry)
   check(current()&&id==ticket){"Source changed during acquisition. Prepare again."}
   val plan=call(workspace,"POST","/api/changes/plan-resources",JSONObject().put("context",inputs.getString("context")).put("title",title).put("edit",edit).put("sources",inputs.getJSONObject("sources"))).getJSONObject("plan")
   val proposal=JSONObject().put("inputs",inputs).put("capture",capture).put("plan",plan)
   val facts=call(workspace,"POST","/api/changes/resource-consequences",proposal)
   check(current()&&id==ticket){"Source changed during analysis. Prepare again."}
   val patches=resourcePatches(plan);check(patches.all{it.before==null||workspace.buffers[it.path]?.text==it.before.text}){"Refresh files to acquire every existing source editor, then prepare again."}
   check(facts.getJSONArray("protected_targets").length()==0){"A proposed path becomes generated. Change its authored construction instead."}
   val expected=resourceSavedMap(facts.getJSONObject("saved_sources"))
   review=Review(proposal,facts,plan,expected,false,null,current,emptyMap(),entryIntent=entry);workspace.showRecovery=true;notice="Review source drafts, saved publication and generated consequences before Save resources."
  }catch(error:CancellationException){throw error}catch(error:Exception){if(id==ticket)notice=error.message.orEmpty()}finally{if(id==ticket)busy=false}
 }
 suspend fun openHistory(workspace:Workspace,record:String,expectedWorkspace:String,current:()->Boolean,published:(String,String)->Unit){
  if(busy||pending!=null){notice="Settle the pending resource change before workspace undo or redo.";return}
  val id=++ticket;busy=true
  try{
   check(current()){"Workspace history changed. Choose Undo or Redo again."}
   val reading=recoveryReading(call(workspace,"GET","/api/changes/resource-recovery?id="+URLEncoder.encode(record,"UTF-8")))
   check(id==ticket&&current()&&reading.id==record&&reading.workspace==expectedWorkspace){"Workspace history or recovery identity changed. Refresh recovery and choose Undo or Redo again."}
   busy=false;restore(workspace,reading,current,published)
  }catch(error:CancellationException){throw error}catch(error:Exception){if(id==ticket)notice=error.message.orEmpty()}finally{if(id==ticket)busy=false}
 }
 suspend fun restore(workspace:Workspace,reading:RecoveryReading,valid:()->Boolean={true},published:((String,String)->Unit)?=null) {
  if(busy||pending!=null){notice="Inspect the pending publication status before preparing restoration.";return}
  val id=++ticket;busy=true;review=null;notice="Preparing saved baselines and drafts independently…"
  try {
   val transition=WorkspaceEntryTransition.read(reading.entryTransition,reading.id)
   val entry=transition?.let {
    val original=workspace.captureRetainedSourceNamespace()
    val stillCurrent={id==ticket&&valid()&&original()}
    val selection=readEntry(workspace,stillCurrent)
    check(selection.proof.entry==it.after){"This recovery belongs to an earlier main construction generation. Restore the latest transition first."}
    val operation=JSONObject().put("recovery_id",reading.id).put("recovery_fingerprint",reading.fingerprint)
    entryIntent(workspace,selection,"restoration",operation,it.before.path,stillCurrent)
   }
   val paths=reading.paths.map{it.path};val changes=JSONObject();paths.forEach{changes.put(languageURI(it),org.json.JSONArray())}
   val edit=entry?.edit() ?: JSONObject().put("changes",changes)
   val (capture,inputs,current)=acquireResourceInputs(workspace,paths,edit,{id==ticket&&valid()},entry)
   check(current()&&id==ticket){"Source changed during acquisition. Prepare restoration again."}
   val proposal=JSONObject().put("recovery_id",reading.id).put("inputs",inputs).put("capture",capture)
   val prepared=call(workspace,"POST","/api/changes/prepare-restoration",proposal);val restoration=prepared.getJSONObject("restoration");val facts=prepared.getJSONObject("consequences");val plan=restoration.getJSONObject("plan")
   check(current()&&id==ticket&&restoration.getString("recovery_fingerprint")==reading.fingerprint&&restoration.getString("workspace")==reading.workspace){"Recovery evidence or workspace changed. Refresh and prepare again."}
   check(facts.getJSONArray("protected_targets").length()==0){"Restored source becomes generated. Reconcile its authored construction first."}
   val patches=resourcePatches(plan);check(patches.all{it.before==null||workspace.buffers[it.path]?.text==it.before.text}){"Refresh files and acquire every current source before restoring."}
   val retained=patches.mapNotNull{it.after?.let{after->workspace.retainedResourceEditors[after.identity]?.let{buffer->after.identity to buffer}}}.toMap()
   val held=retained.mapValues{it.value.text};val guard={current()&&retained.all{(identity,buffer)->workspace.retainedResourceEditors[identity]===buffer&&buffer.text==held[identity]}}
   review=Review(proposal,facts,plan,resourceSavedMap(restoration.getJSONObject("saved_after")),true,reading.fingerprint,guard,retained,published,entryIntent=entry);notice="Review the complete restoration. Saved baselines and unsaved drafts remain separate."
  }catch(error:CancellationException){throw error}catch(error:Exception){if(id==ticket)notice=error.message.orEmpty()}finally{if(id==ticket)busy=false}
 }
 suspend fun apply(workspace:Workspace,expected:Review?=review) {
  val value=review ?: return;if(busy||pending!=null)return
  if(value!==expected){notice="Resource review changed. Read the current proposal before saving.";return}
  busy=true
  try{check(value.current()){"Source changed after review. Prepare again."};value.preflight?.invoke();check(review===value&&pending==null&&value.current()){ "Proposal or source changed during final admission. Nothing was sent." }}catch(error:CancellationException){throw error}catch(error:Exception){notice=error.message.orEmpty();return}finally{busy=false}
  val request=nonce();pending=request to value;publishedWorkspace=workspace;review=null;busy=true
  val body=JSONObject().put("request_id",request).put("reviewed_fingerprint",value.facts.getString("fingerprint")).put("proposal",value.proposal)
  if(value.restoration)body.put("reviewed_recovery_fingerprint",value.fingerprint)
  try{adopt(workspace,request,value,call(workspace,"POST",if(value.restoration)"/api/changes/restore-resources" else "/api/changes/resource-save",body))}
  catch(error:CancellationException){notice="Publication may have started. Inspect its status or Source recovery; do not repeat Save.";throw error}
  catch(error:Exception){notice="${error.message} Publication is unconfirmed. Inspect status or Source recovery; this action will not be retried."}
  finally{busy=false}
 }
 suspend fun status(workspace:Workspace){val (request,value)=pending ?: return;if(busy)return;busy=true
  try{check(publishedWorkspace===workspace){"Workspace changed. Inspect recovery in the original workspace."};adopt(workspace,request,value,call(workspace,"GET","/api/changes/resource-save?request_id=$request"))}
  catch(error:CancellationException){throw error}catch(error:Exception){notice=error.message.orEmpty()+" Inspect retained recovery before another publication."}finally{busy=false}
 }
 private fun adopt(workspace:Workspace,request:String,value:Review,raw:JSONObject){
  confirmWorkspaceEntryPublication(raw.opt("entry_outcome"),request,value.entryIntent)
  val receipt=resourcePublication(raw);val patches=resourcePatches(value.plan)
  workspace.adoptResourceReceipt(patches,receipt,request,value.facts.getString("workspace"),value.facts.getString("context"),value.saved,value.current,value.restoring)
  val recovery=receipt.recovery.substringBeforeLast('/').substringAfterLast('/')
  value.published?.invoke(recovery,receipt.workspace) ?: workspace.changes.recordResource(recovery,receipt.workspace)
  lastRecovery=recovery;lastWorkspace=receipt.workspace;pending=null;publishedWorkspace=null
  notice="Saved resources confirmed. Drafts and editor state retained. Restore this new recovery record to reverse the change; no checks were executed."
 }
 val hasPending get()=pending!=null
 val pendingRecovery get()=pending?.first
 val pendingWorkspace get()=pending?.second?.facts?.getString("workspace")
}
fun resourcePatches(plan:JSONObject):List<ResourceEditorPatch>{
 val rows=plan.getJSONArray("patches")
 fun side(value:JSONObject):ResourceSource? {require(value.getString("kind") in listOf("file","missing")){"Unsupported resource state. Prepare again."};return if(value.getString("kind")=="file")ResourceSource(value.getString("identity"),value.getString("text")) else null}
 return (0 until rows.length()).map{val row=rows.getJSONObject(it);ResourceEditorPatch(row.getString("path"),side(row.getJSONObject("before")),side(row.getJSONObject("after")))}
}
fun resourceSavedMap(raw:JSONObject)=raw.keys().asSequence().associateWith{key->val value=raw.getJSONObject(key);if(value.getString("kind")=="missing")ResourceSaved("missing") else ResourceSaved("file",value.getString("text"),value.getString("sha256"),value.getBoolean("readonly"),value.getBoolean("executable"),if(value.isNull("mode"))null else value.getInt("mode"))}
fun resourcePublication(raw:JSONObject):ResourcePublication<FileNode>{check(raw.get("version")==2){"Resource publication protocol2 is required for current editor adoption."};val outcome=raw.getJSONObject("outcome");val rows=raw.getJSONArray("source_files");val applied=outcome.getJSONArray("applied");return ResourcePublication(raw.getInt("version"),raw.getString("request_id"),raw.getString("workspace"),raw.getString("context"),outcome.getBoolean("complete"),outcome.getString("recovery"),(0 until applied.length()).map{applied.getString(it)},if(raw.isNull("saved_sources"))null else resourceSavedMap(raw.getJSONObject("saved_sources")),(0 until rows.length()).map{val row=rows.getJSONObject(it);val file=row.getJSONObject("file");ResourcePublishedFile(FileNode(file.getString("name"),file.getString("path"),file.getBoolean("dir"),emptyList(),if(file.isNull("doc_id"))null else file.getString("doc_id"),if(file.isNull("generated_by"))null else file.getString("generated_by"),if(file.isNull("language"))null else file.getString("language")),resourceSavedMap(JSONObject().put("entry",row.getJSONObject("saved"))).getValue("entry"),row.getString("hash"),if(row.isNull("save_guard"))null else row.getString("save_guard").also{check(it.isNotEmpty()){"Malformed source save guard."}})},raw.get("adoption_current") as? Boolean ?: error("Resource publication adoption authority is unavailable or malformed."),if(raw.isNull("adoption_reason"))null else raw.getString("adoption_reason"))}

data class RecoveryPage(val version:Int,val workspace:String,val ids:List<String>,val next:String?,val ignored:Int)
data class RecoverySummary(val kind:String,val bytes:Int?,val sha256:String?)
data class RecoveryPath(val path:String,val snapshots:Map<String,RecoverySummary>)
data class RecoveryReading(val version:Int,val workspace:String,val id:String,val fingerprint:String,val title:String,val titleTruncated:Boolean,val paths:List<RecoveryPath>,val allApplied:Boolean,val unrecognized:Int,val draftText:String?,val savedText:String?,val entryTransition:Any?=null)
private fun JSONObject.recoveryText(key:String)=if(isNull(key))null else getString(key)
fun recoveryPage(raw:JSONObject):RecoveryPage {val ids=raw.getJSONArray("ids");return RecoveryPage(raw.getInt("version"),raw.getString("workspace"),(0 until ids.length()).map{ids.getString(it)},raw.recoveryText("next"),raw.getInt("ignored"))}
fun recoveryReading(raw:JSONObject):RecoveryReading {val paths=raw.getJSONArray("paths");return RecoveryReading(raw.getInt("version"),raw.getString("workspace"),raw.getString("id"),raw.getString("fingerprint"),raw.getString("title"),raw.getBoolean("title_truncated"),(0 until paths.length()).map{val row=paths.getJSONObject(it);RecoveryPath(row.getString("path"),listOf("saved_before","saved_after","draft_before","draft_after").associateWith{key->val value=row.getJSONObject(key);RecoverySummary(value.getString("kind"),if(value.has("bytes"))value.getInt("bytes") else null,value.recoveryText("sha256"))})},raw.getBoolean("all_paths_recorded_applied"),raw.getJSONArray("unrecognized_files").length(),raw.recoveryText("draft_text"),raw.recoveryText("saved_text"),raw.opt("entry_transition"))}
class RecoveryReader {
 var page by mutableStateOf<RecoveryPage?>(null);var value by mutableStateOf<RecoveryReading?>(null);var notice by mutableStateOf("");var loading by mutableStateOf(false);private var ticket=0L
 fun close(){ticket++;loading=false;page=null;value=null}
 suspend fun list(after:String?=null,load:suspend(String)->RecoveryPage){val id=++ticket;loading=true;value=null;notice="Reading retained recovery identities…"
  try{val next=load("/api/changes/resource-recoveries"+(after?.let{"?after="+URLEncoder.encode(it,"UTF-8")} ?: ""));if(id!=ticket)return
   check(next.version==1&&next.workspace.isNotEmpty()&&next.ids.size<=100&&next.ids.all{it.matches(Regex("[0-9a-f]{32}"))}&&next.ids.zipWithNext().all{it.first<it.second}&&(after==null||next.ids.all{it>after}&&next.workspace==page?.workspace)&&(next.next==null||next.next==next.ids.lastOrNull())){"Recovery page identity changed. Refresh from the beginning."}
   page=next;notice=if(next.ids.isEmpty())"No retained resource recovery records." else "Choose a retained record. Identities are sorted, not chronological."
  }catch(error:CancellationException){throw error}catch(error:Exception){if(id==ticket){page=null;notice=error.message.orEmpty()}}finally{if(id==ticket)loading=false}
 }
 suspend fun read(record:String,path:String?=null,snapshot:String="draft_before",expectedWorkspace:String?=null,load:suspend(String)->RecoveryReading){val workspace=expectedWorkspace ?: page?.workspace;val previous=value?.takeIf{it.id==record};val id=++ticket;loading=true;value=null;notice="Reading retained source evidence…"
  try{check(workspace!=null&&record.matches(Regex("[0-9a-f]{32}"))&&(expectedWorkspace!=null||page?.ids?.contains(record)==true||previous?.workspace==workspace)){"Refresh recovery identities before selecting a record."};val parts=snapshot.split('_');check(parts.size==2&&parts[0] in listOf("saved","draft")&&parts[1] in listOf("before","after")){"Choose a supported recovery snapshot."}
   val reply=load("/api/changes/resource-recovery?id="+URLEncoder.encode(record,"UTF-8")+(path?.let{"&path="+URLEncoder.encode(it,"UTF-8")+"&"+parts[0]+"="+parts[1]} ?: ""));if(id!=ticket)return
   check(reply.version==1&&reply.workspace==workspace&&reply.id==record&&reply.fingerprint.matches(Regex("[0-9a-f]{64}"))&&(previous==null||previous.fingerprint==reply.fingerprint)&&(path==null||reply.paths.any{it.path==path})){"Recovery evidence changed. Refresh identities and read again."}
   value=reply;notice="Retained evidence only. Current source and editor drafts are unchanged."
  }catch(error:CancellationException){throw error}catch(error:Exception){if(id==ticket)notice=error.message.orEmpty()}finally{if(id==ticket)loading=false}
 }
}
@Composable fun SourceRecoveryView(workspace:Workspace){
 val model=remember{RecoveryReader()};val resources=workspace.resourceChanges;val scope=rememberCoroutineScope();val engine=workspace.engine;var path by remember{mutableStateOf("")};var snapshot by remember{mutableStateOf("draft_before")};var recordMenu by remember{mutableStateOf(false)};var pathMenu by remember{mutableStateOf(false)}
 var reviewFile by remember(resources.review){mutableStateOf(0)}
 suspend fun list(after:String?=null){if(engine==null){model.notice="Connect to a workspace server first.";return};path="";model.list(after){recoveryPage(engine.request("GET",it))}}
 suspend fun read(id:String,path:String?=null,snapshot:String="draft_before",expectedWorkspace:String?=null){if(engine==null)return;model.read(id,path,snapshot,expectedWorkspace ?: model.value?.takeIf{it.id==id}?.workspace){recoveryReading(engine.request("GET",it))}}
 LaunchedEffect(engine){list()};DisposableEffect(model){onDispose{model.close();resources.dismiss()}}
 Column(Modifier.widthIn(max=680.dp).heightIn(max=680.dp).padding(16.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(8.dp)){
  Text("Source recovery",style=MaterialTheme.typography.titleMedium);Row{TextButton(enabled=!workspace.resourceChanges.busy&&!workspace.resourceChanges.hasPending,onClick={workspace.showRecovery=false;workspace.editorRecovery.showing=true}){Text("Editor sessions…")};TextButton(enabled=!model.loading,onClick={scope.launch{list()}}){Text("Refresh records")};TextButton(onClick={workspace.showRecovery=false;workspace.currentBuffer?.focusSource?.invoke()}){Text("Close")}}
  Text(model.notice,fontSize=12.sp)
  if(resources.notice.isNotEmpty())Text(resources.notice,fontSize=12.sp)
  if(resources.hasPending)TextButton(enabled=!resources.busy,onClick={scope.launch{resources.status(workspace)}}){Text("Inspect original publication status")}
  resources.pendingRecovery?.let{record->SelectionContainer{Text("Original publication: $record",fontSize=12.sp)};TextButton(enabled=!resources.busy,onClick={scope.launch{read(record,expectedWorkspace=resources.pendingWorkspace)}}){Text("Inspect interrupted request recovery")}}
  resources.lastRecovery?.let{record->SelectionContainer{Text("Latest recovery: $record",fontSize=12.sp)};TextButton(enabled=!resources.busy,onClick={scope.launch{read(record,expectedWorkspace=resources.lastWorkspace)}}){Text("Review inverse / redo from latest recovery")}}
  resources.review?.let{review->
   Text(review.plan.getString("title"),style=MaterialTheme.typography.titleSmall)
   Text("Save publishes the reviewed saved states. Draft text can differ from its saved baseline. Generated output is analyzed, never materialized by this action.",fontSize=12.sp)
   val patches=resourcePatches(review.plan)
   Row{TextButton(enabled=reviewFile>0,onClick={reviewFile--}){Text("Previous source")};Text("${reviewFile+1} / ${patches.size}",fontSize=12.sp);TextButton(enabled=reviewFile+1<patches.size,onClick={reviewFile++}){Text("Next source")}}
   patches.getOrNull(reviewFile)?.let{patch->
    Text(patch.path,style=MaterialTheme.typography.titleSmall)
    listOf("Current draft" to patch.before?.text,"Reviewed draft" to patch.after?.text,"Reviewed saved baseline" to review.saved[patch.path]?.text).forEach{(label,text)->Text(label,fontSize=12.sp);SelectionContainer{Text(text?.take(16000) ?: "Source absent",fontFamily=FontFamily.Monospace,fontSize=12.sp)};if(text!=null&&text.length>16000)Text("First 16,000 characters shown.",fontSize=12.sp)}
   }
   listOf("saved","draft").forEach{layer->val outputs=review.facts.getJSONObject(layer).getJSONArray("output_paths");Text("$layer construction consequences: ${outputs.length()} outputs; execution evidence not queried.",fontSize=12.sp);SelectionContainer{Text((0 until minOf(outputs.length(),300)).joinToString("\n"){outputs.getString(it)},fontFamily=FontFamily.Monospace,fontSize=12.sp)};if(outputs.length()>300)Text("First 300 output paths shown.",fontSize=12.sp)}
   Row{TextButton(enabled=!resources.busy,onClick={scope.launch{resources.apply(workspace,review)}}){Text(if(review.restoration)"Save reviewed restoration" else "Save resources")};TextButton(enabled=!resources.busy,onClick={resources.dismiss()}){Text("Discard review")}}
  }
  model.page?.let{page->Box{TextButton(enabled=!model.loading,onClick={recordMenu=true}){Text(model.value?.id ?: "Choose retained record")};DropdownMenu(expanded=recordMenu,onDismissRequest={recordMenu=false}){page.ids.forEach{id->DropdownMenuItem(text={Text(id)},onClick={recordMenu=false;path="";scope.launch{read(id)}})}}};if(page.ignored>0)Text("${page.ignored} unrecognized entries omitted.",fontSize=12.sp);page.next?.let{next->TextButton(enabled=!model.loading,onClick={scope.launch{list(next)}}){Text("Next 100 identities")}}}
  model.value?.let{value->Text(value.title+if(value.titleTruncated)"…" else "",style=MaterialTheme.typography.titleSmall);Text((if(value.allApplied)"Every path has an applied receipt." else "The record does not confirm every path was applied.")+" These receipts describe the recorded operation, not current disk state.",fontSize=12.sp);if(value.unrecognized>0)Text("${value.unrecognized} unrecognized files in this record.",fontSize=12.sp)
   TextButton(enabled=!model.loading&&!resources.busy&&!resources.hasPending,onClick={scope.launch{resources.restore(workspace,value)}}){Text("Prepare complete saved / draft restoration")}
   Box{TextButton(enabled=!model.loading,onClick={pathMenu=true}){Text(path.ifEmpty{"Choose recorded source"})};DropdownMenu(expanded=pathMenu,onDismissRequest={pathMenu=false}){value.paths.forEach{item->DropdownMenuItem(text={Text(item.path)},onClick={pathMenu=false;path=item.path;scope.launch{read(value.id,path,snapshot)}})}}}
   listOf("saved_before","draft_before","saved_after","draft_after").forEach{side->TextButton(enabled=!model.loading,onClick={snapshot=side;if(path.isNotEmpty())scope.launch{read(value.id,path,side)}}){Text((if(snapshot==side)"✓ " else "")+side.replace('_',' '))}}
   value.paths.find{it.path==path}?.snapshots?.get(snapshot)?.let{selected->SelectionContainer{Text(if(selected.kind=="missing")"Source absent in this snapshot." else "${selected.bytes} UTF-8 bytes · ${selected.sha256}",fontSize=12.sp)};val text=if(snapshot.startsWith("saved"))value.savedText else value.draftText;if(text!=null){SelectionContainer{Text(text.take(32000),fontFamily=FontFamily.Monospace,fontSize=12.sp)};if(text.length>32000)Text("First 32,000 characters shown.",fontSize=12.sp)}}
  }
 }
}
