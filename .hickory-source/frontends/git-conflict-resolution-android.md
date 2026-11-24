<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Compose and publish complete Android conflict groups

For maintainers integrating conflict resolution without transferring Git authority to a renderer.

The persistent workspace model retains a composed proposal independently from source editors. Group membership, destination addresses, retained identities, stage OIDs and executable modes are explicit. Omitted members are deletions; no rename heuristic supplies a mapping. Resource capture acquires existing editors without activating or remounting them, and its admission closure binds the entire authored namespace and local source revision, including edit followed by Undo returning identical text.

The server retains the immutable reviewed proposal. Draft application revalidates its canonical source patches and saved metadata immediately before the existing coordinated draft controller acts; namespace changes require guarded publication. Saving and staging are separate explicit one-shot requests. A lost or incomplete response retains its original request for inspection; it never authorizes retry, adoption or staging. Confirmed source receipts route retained editors through the existing resource authority and record inverse recovery. Staging checks the new post-adoption namespace but changes no source editor. Closing a floating review retires pending observations, while an already started publication remains workspace-held and inspectable.

The floating adapter pages the complete reviewed JSON rather than abbreviating generated consequences. Unsupported source names remain readable in stage inspection and refuse at the shared publication boundary. This is controller and service integration; rendered Compose layout, pointer, keyboard, IME and accessibility acceptance remain unexecuted.

<ui:copy id="git-resolution-android">
@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
package com.hickorydocs.simple
import androidx.compose.runtime.*
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import org.json.JSONObject
import org.json.JSONArray

/** Compositions remain drafts of a proposal; only server-held reviewed plans can publish. */
data class GitResolutionOutput(val path:String,val identityFrom:String?,val text:String,val mode:String="100644",val stagePath:String?=null,val stage:Int?=null,val oid:String?=null) {
 fun json()=JSONObject().put("path",path).put("identity_from",identityFrom ?: JSONObject.NULL).put("mode",mode).put("content",if(stage==null)JSONObject().put("kind","text").put("text",text) else JSONObject().put("kind","stage").put("path",stagePath).put("stage",stage).put("oid",oid))
}
data class GitResolutionPreview(val token:String,val fingerprint:String,val patches:List<ResourceEditorPatch>,val saved:Map<String,ResourceSaved>,val needsSave:Boolean,val canStage:Boolean,val stageReason:String?,val raw:String,val facts:String,val current:()->Boolean)
fun gitResolutionPreview(raw:JSONObject,current:()->Boolean):GitResolutionPreview {
 check(raw.get("version")==1&&raw.get("resolution_protocol")==1&&!raw.getBoolean("source_files_modified")&&!raw.getBoolean("index_modified")){"Unsupported conflict resolution response."}
 val token=raw.getString("token");val fingerprint=raw.getString("fingerprint")
 check(token.matches(Regex("[a-f0-9]{32}"))&&fingerprint.matches(Regex("[a-f0-9]{64}"))){"Malformed conflict review identity."}
 return GitResolutionPreview(token,fingerprint,resourcePatches(raw.getJSONObject("plan")),resourceSavedMap(raw.getJSONObject("saved_sources")),raw.get("needs_save") as? Boolean ?: error("Malformed source save capability."),raw.get("stage_available") as? Boolean ?: error("Malformed staging capability."),raw.optString("stage_unavailable").takeUnless{it=="null"||it.isEmpty()},raw.toString(),raw.getJSONObject("consequences").toString(),current)
}
fun gitResolutionDraftable(value:GitResolutionPreview)=value.patches.any{it.before?.text!=it.after?.text}&&value.patches.all{it.before!=null&&it.after!=null&&it.before.identity==it.after.identity}

fun gitResolutionStageReceipt(raw:JSONObject,request:String,p:GitResolutionPreview){
 fun members(value:JSONObject,key:String):List<String>{val rows=value.getJSONArray(key);val paths=(0 until rows.length()).map{rows.getString(it)};check(paths.distinct().size==paths.size){"Duplicate staged group receipt."};return paths.sorted()}
 val reviewed=JSONObject(p.raw)
 check(raw.get("version")==1&&raw.getString("operation")=="git_conflict_stage"&&raw.getString("request_id")==request&&raw.getString("token")==p.token&&raw.getString("review_fingerprint")==p.fingerprint&&raw.get("complete")==true&&raw.get("index_modified")==true&&raw.get("index_uncertain")==false&&raw.get("source_files_modified")==false&&raw.getString("fingerprint").matches(Regex("[a-f0-9]{64}"))&&members(raw,"resolved_members")==members(reviewed,"members")&&members(raw,"remaining_conflicts")==members(reviewed,"remaining_conflicts")){"Complete index publication is unconfirmed; inspect the original request."}
}

class GitResolutionModel(private val transport:(suspend(String,String,JSONObject?)->JSONObject)?=null) {
 var reading by mutableStateOf<JSONObject?>(null);private set
 var members by mutableStateOf<Set<String>>(emptySet());private set
 var outputs by mutableStateOf<List<GitResolutionOutput>>(emptyList());private set
 var preview by mutableStateOf<GitResolutionPreview?>(null);private set
 var notice by mutableStateOf("");private set
 var busy by mutableStateOf(false);private set
 var saved by mutableStateOf(false);private set
 private var ticket=0L;private var writeGuard:Long?=null
 private data class Pending(val id:String,val review:GitResolutionPreview,val stage:Boolean,val workspace:Workspace)
 private var pending:Pending?=null
 val hasPending get()=pending!=null
 private fun nonce()=java.util.UUID.randomUUID().toString().replace("-","")
 private suspend fun call(w:Workspace,method:String,route:String,body:JSONObject?=null)=transport?.invoke(method,route,body) ?: (w.engine ?: error("Connect to the original workspace server.")).request(method,route,body)
 fun reset(){ticket++;writeGuard=null;pending=null;reading=null;members=emptySet();outputs=emptyList();preview=null;busy=false;saved=false;notice=""}
 fun close(){if(pending==null){ticket++;preview=null;busy=false}}
 fun choose(path:String){if(busy||hasPending)return;ticket++;preview=null;saved=false;members=if(path in members)members-path else members+path}
 fun output(index:Int,value:GitResolutionOutput?){if(busy||hasPending)return;ticket++;preview=null;saved=false;outputs=outputs.toMutableList().also{if(value==null)it.removeAt(index) else it[index]=value}}
 fun add(){if(busy||hasPending)return;ticket++;preview=null;saved=false;outputs=outputs+GitResolutionOutput("",null,"")}
 suspend fun read(w:Workspace,anchor:String,selected:Boolean=false){if(busy||hasPending)return;val id=++ticket;busy=true;preview=null
  try{val paths=if(selected)members.toList().sorted() else listOf(anchor);val raw=call(w,"POST","/api/git/conflict-resolution/read",JSONObject().put("version",1).put("members",JSONArray(paths)));if(id!=ticket)return
   check(raw.get("version")==1&&raw.get("resolution_protocol")==1){"Unsupported conflict service."};reading=raw;members=paths.toSet()
   if(!selected&&outputs.isEmpty()){val rows=raw.getJSONArray("paths");val row=(0 until rows.length()).map{rows.getJSONObject(it)}.firstOrNull{it.getString("path")==anchor};val working=row?.optJSONObject("working");if(working?.optString("kind")=="file")outputs=listOf(GitResolutionOutput(anchor,anchor,w.buffers[anchor]?.text ?: working.getString("text")))}
   notice="Select the complete conflict group, read its stages, then explicitly choose every retained destination. Omitted members will be deleted."
  }catch(e:CancellationException){throw e}catch(e:Exception){if(id==ticket)notice=e.message.orEmpty()}finally{if(id==ticket)busy=false}
 }
 suspend fun prepare(w:Workspace){if(busy||hasPending)return;val r=reading ?: return;val id=++ticket;val epoch=w.sourceNavigationEpoch;val engine=w.engine;busy=true;preview=null;saved=false
  try{check(!w.resourceChanges.busy&&!w.resourceChanges.hasPending){"Inspect the other pending resource publication first."};check(members.isNotEmpty()){ "Choose the complete conflict group." }
   val paths=(members+outputs.map{it.path}).sorted();check(paths.none{it.isEmpty()}){"Choose every destination explicitly."}
   val changes=JSONObject();paths.forEach{changes.put(languageURI(it),JSONArray())};val edit=JSONObject().put("changes",changes)
   val valid={w.engine===engine&&w.sourceNavigationEpoch==epoch&&(ticket==id||writeGuard==id)}
   val (capture,inputs,current)=w.resourceChanges.acquire(w,paths,edit,valid)
   val raw=call(w,"POST","/api/git/conflict-resolution/prepare",JSONObject().put("version",1).put("reading",r).put("members",JSONArray(members.sorted())).put("outputs",JSONArray(outputs.map{it.json()})).put("inputs",inputs).put("capture",capture))
   check(current()){"Sources changed during review. Read and prepare again."};preview=gitResolutionPreview(raw,current);notice="Review complete source/resource and saved/draft generated consequences. Draft application, saving and staging are separate actions."
  }catch(e:CancellationException){throw e}catch(e:Exception){if(id==ticket)notice=e.message.orEmpty()}finally{if(id==ticket)busy=false}
 }
 private fun body(p:GitResolutionPreview)=JSONObject().put("version",1).put("token",p.token).put("fingerprint",p.fingerprint)
 suspend fun draft(w:Workspace){val p=preview ?: return;if(busy||hasPending)return;busy=true
  try{check(p.current()&&gitResolutionDraftable(p)){"Resource address changes require guarded Save; no drafts were applied."};val raw=call(w,"POST","/api/git/conflict-resolution/validate",body(p));check(raw.getBoolean("current")&&raw.getString("token")==p.token&&raw.getString("fingerprint")==p.fingerprint&&resourcePatches(raw.getJSONObject("plan"))==p.patches&&resourceSavedMap(raw.getJSONObject("saved_sources"))==p.saved&&p.current()){"Conflict or source authority changed. Prepare again."}
   val changes=JSONObject();val expected=mutableMapOf<String,String>();for(patch in p.patches){val before=patch.before!!.text;val after=patch.after!!.text;expected[patch.path]=before;changes.put(languageURI(patch.path),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",languagePosition(before,before.length))).put("newText",after)))}
   w.changes.offer("Compose Git conflict group",JSONObject().put("changes",changes),expected,w);w.changes.apply(w);check(w.changes.plan==null){w.changes.notice};preview=null;notice="Applied coordinated source drafts. Nothing saved or staged. Prepare again before saving."
  }catch(e:CancellationException){throw e}catch(e:Exception){notice=e.message.orEmpty()}finally{busy=false}
 }
 suspend fun act(w:Workspace,stage:Boolean){val p=preview ?: return;if(busy||hasPending)return
  try{check(p.current()){"Source editors changed. Prepare again."};check(if(stage)p.canStage&&(!p.needsSave||saved) else p.needsSave&&!saved){"Save the reviewed complete group before explicit staging."};check(!w.resourceChanges.busy&&!w.resourceChanges.hasPending){"Another publication is pending."}
   if(!stage){busy=true;val checked=call(w,"POST","/api/git/conflict-resolution/validate",body(p));check(checked.getBoolean("current")&&checked.getString("token")==p.token&&checked.getString("fingerprint")==p.fingerprint&&resourcePatches(checked.getJSONObject("plan"))==p.patches&&resourceSavedMap(checked.getJSONObject("saved_sources"))==p.saved&&p.current()){"Reviewed source/resource facts changed. Prepare again."}}
   val id=nonce();pending=Pending(id,p,stage,w);writeGuard=ticket;busy=true;val raw=call(w,"POST","/api/git/conflict-resolution/${if(stage)"stage" else "save"}",body(p).put("request_id",id));accept(w,raw)
  }catch(e:CancellationException){notice="Publication may have started. Inspect the original request; never repeat it.";throw e}catch(e:Exception){notice=e.message.orEmpty()+if(hasPending)" Outcome unconfirmed; inspect original request or Source recovery. No retry or staging is authorized." else ""}finally{busy=false}
 }
 suspend fun status(w:Workspace){val p=pending ?: return;if(busy)return;busy=true
  try{check(p.workspace===w){"Inspect the original workspace."};val raw=call(w,"GET","/api/git/conflict-resolution/status?request_id=${p.id}")
   check(raw.get("version")==1&&raw.getString("request_id")==p.id){"Foreign publication status refused."}
   when(raw.getString("state")){
    "pending","unknown"->notice="Original request ${p.id} is ${raw.getString("state")}. Inspect retained recovery; no retry, adoption or staging is authorized."
    "completed"->{val outcome=raw.optJSONObject("outcome");if(outcome!=null&&runCatching{accept(w,outcome)}.isSuccess)return
     // A completed refusal still requires positive proof of the full original
     // source/index preimage. Never parse human rollback messages as authority.
     check(p.review.current()){"Publication completed without confirmable adoption. Inspect recovery and original saved/index state."}
     val checked=call(w,"POST","/api/git/conflict-resolution/validate",body(p.review).put("stage",p.stage));check(checked.getBoolean("stage")==p.stage&&checked.getBoolean("current")&&checked.getString("token")==p.review.token&&checked.getString("fingerprint")==p.review.fingerprint&&p.review.current()){"Original source/index authority cannot be established. Inspect recovery."}
     pending=null;writeGuard=null;preview=null;saved=false;notice="Completed refused publication; complete original source/index preimage confirmed. Read and prepare a fresh review before any new request."
    }
    else->error("Malformed publication status.")
   }
  }catch(e:CancellationException){throw e}catch(e:Exception){notice=e.message.orEmpty()+" Retain this request and inspect recovery; never retry an uncertain write."}finally{busy=false}
 }
 private fun namespace(w:Workspace)=w.captureRetainedSourceNamespace()
 private fun accept(w:Workspace,raw:JSONObject){val pending=pending ?: error("No owned publication.");check(pending.workspace===w){"Workspace changed."};val p=pending.review
  if(pending.stage){gitResolutionStageReceipt(raw,pending.id,p);this.pending=null;writeGuard=null;preview=null;notice="Complete saved conflict group staged. Unsaved drafts remain separate; no tests were run."}
  else{val receipt=resourcePublication(raw);val recovery=receipt.recovery.substringBeforeLast('/').substringAfterLast('/');check(recovery.matches(Regex("[a-f0-9]{32}"))){"Recovery receipt unavailable."};val facts=JSONObject(p.facts);w.adoptResourceReceipt(p.patches,receipt,pending.id,facts.getString("workspace"),facts.getString("context"),p.saved,p.current);w.changes.recordResource(recovery,receipt.workspace);this.pending=null;writeGuard=null;saved=true;preview=p.copy(current=namespace(w));notice="Saved conflict group confirmed; editor identity and coordinated recovery retained. Choose Stage complete group separately."}
 }
}

@Composable fun GitResolutionView(w:Workspace,path:String){
 val model=w.gitResolution;val scope=rememberCoroutineScope();var consequencePage by remember{mutableIntStateOf(0)}
 DisposableEffect(model){onDispose{model.close()}}
 Column(verticalArrangement=Arrangement.spacedBy(6.dp)){
  WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else null) { available -> TextButton(enabled = available,onClick={scope.launch{model.read(w,path)}}){Text("Compose / resolve conflict group")} }
  if(model.notice.isNotEmpty())Text(model.notice,fontSize=12.sp)
  if(model.hasPending)WithDisabledReason(if (model.busy) "An operation is already in progress." else null) { available -> TextButton(enabled = available,onClick={scope.launch{model.status(w)}}){Text("Inspect original resolution request")} }
  model.reading?.let{reading->
   val rows=reading.getJSONArray("paths");Text("Select every conflict member belonging to this resolution. Rename relationships are explicit choices.",fontSize=12.sp)
   for(i in 0 until rows.length()){val row=rows.getJSONObject(i);val member=row.getString("path");Row{WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else null) { available -> Checkbox(modifier=Modifier.semantics{contentDescription="Include ${JSONObject.quote(member)} in this conflict resolution group"},checked=member in model.members,enabled = available,onCheckedChange={model.choose(member)}) };Text(JSONObject.quote(member),fontSize=12.sp)};if(!row.isNull("unavailable"))Text(row.getString("unavailable"),fontSize=12.sp)}
   WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else if (!(model.members.isNotEmpty())) "Select paths to include in the resolution." else null) { available -> TextButton(enabled = available,onClick={scope.launch{model.read(w,path,true)}}){Text("Read selected group stages")} }
   model.outputs.forEachIndexed{index,output->
    HorizontalDivider();WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else null) { available -> OutlinedTextField(value=output.path,onValueChange={model.output(index,output.copy(path=it))},label={Text("Destination ${index+1}")},enabled = available) }
    Text("Retained editor identity: ${output.identityFrom?.let{JSONObject.quote(it)} ?: "New source"}",fontSize=12.sp)
    Row{WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else null) { available -> TextButton(enabled = available,onClick={model.output(index,output.copy(identityFrom=null))}){Text("New identity")} };model.members.sorted().forEach{member->WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else null) { available -> TextButton(enabled = available,onClick={model.output(index,output.copy(identityFrom=member))}){Text("Retain ${JSONObject.quote(member)}")} }}}
    WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else null) { available -> OutlinedTextField(value=output.text,onValueChange={model.output(index,output.copy(text=it,stage=null,stagePath=null,oid=null))},label={Text("Composed resolution · ${output.path.ifEmpty{"destination ${index+1}"}}")},minLines=3,maxLines=10,enabled = available) }
    Row{WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else null) { available -> TextButton(enabled = available,onClick={model.output(index,output.copy(mode=if(output.mode=="100644")"100755" else "100644"))}){Text("Mode ${output.mode}")} };WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else null) { available -> TextButton(enabled = available,onClick={model.output(index,null)}){Text("Remove ${JSONObject.quote(output.path)} destination (delete if not retained elsewhere)")} }}
    for(i in 0 until rows.length()){val row=rows.getJSONObject(i);val member=row.getString("path");if(member !in model.members)continue;val stages=row.getJSONArray("stages");for(j in 0 until stages.length()){val stage=stages.getJSONObject(j);if(!stage.isNull("text"))WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else null) { available -> TextButton(enabled = available,onClick={model.output(index,output.copy(text=stage.getString("text"),stagePath=member,stage=stage.getInt("stage"),oid=stage.getString("oid")))}){Text("Use ${JSONObject.quote(member)} stage ${stage.getInt("stage")} bytes")} }}}
   }
   WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else null) { available -> TextButton(enabled = available,onClick={model.add()}){Text("Add explicit destination")} }
   Text("Members without a retained destination will be deleted: ${model.members.filter{member->model.outputs.none{it.identityFrom==member}}.joinToString{JSONObject.quote(it)}}",fontSize=12.sp)
   WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else null) { available -> TextButton(enabled = available,onClick={scope.launch{model.prepare(w)}}){Text("Preview complete resolution consequences")} }
  }
  model.preview?.let{review->
   Text("Complete reviewed source/resource proposal, saved metadata and both generated layers",fontSize=12.sp)
   // Page the complete immutable receipt. No consequence is silently omitted.
   val pretty=remember(review.raw){JSONObject(review.raw).toString(2)};val count=((pretty.length+7999)/8000).coerceAtLeast(1);val page=consequencePage.coerceIn(0,count-1)
   Text(pretty.substring(page*8000,((page+1)*8000).coerceAtMost(pretty.length)),fontSize=11.sp)
   Row{WithDisabledReason(if (page <= 0) "You’re on the first page." else null) { available -> TextButton(enabled = available,onClick={consequencePage=page-1}){Text("Previous consequences")} };Text("${page+1}/$count");WithDisabledReason(if (page >= count-1) "You’re on the last page." else null) { available -> TextButton(enabled = available,onClick={consequencePage=page+1}){Text("Next consequences")} }}
   if(!review.canStage)Text(review.stageReason ?: "Staging unavailable",fontSize=12.sp)
   WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else if (!(gitResolutionDraftable(review))) "This resolution changes resources or metadata. Use the reviewed Save action." else null) { available -> TextButton(enabled = available,onClick={scope.launch{model.draft(w)}}){Text("Apply text drafts only")} }
   WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else if (!(review.needsSave)) "This resolution has no pending changes to save." else if (model.saved) "The reviewed resolution has already been saved." else null) { available -> TextButton(enabled = available,onClick={scope.launch{model.act(w,false)}}){Text("Save complete resolution group")} }
   WithDisabledReason(if (model.busy) "An operation is already in progress." else if (model.hasPending) "Inspect the pending publication before starting another change." else if (!(review.canStage)) "This resolution cannot be staged in the current Git context." else if (!((!review.needsSave||model.saved))) "Save the reviewed resolution before staging." else null) { available -> TextButton(enabled = available,onClick={scope.launch{model.act(w,true)}}){Text("Stage complete saved group")} }
  }
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/GitConflictResolution.kt"><ui:paste select="#git-resolution-android"/></ui:file>

## Preserve reviewed identities across delayed replies

Focused controller tests reject malformed protocol identities, mutation of the decoded preview, namespace-changing draft application and closed read replies. Explicit destination and retained identity remain independent of immutable stage content. Real engine fixtures exercise publication separately; these model tests do not stand in for index or filesystem admission.

<ui:copy id="git-resolution-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import org.json.JSONArray
import kotlinx.coroutines.*
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class GitConflictResolutionTest {
 private fun side(identity:String,text:String)=JSONObject().put("kind","file").put("identity",identity).put("text",text)
 private fun preview()=JSONObject().put("members",JSONArray().put("a.py")).put("remaining_conflicts",JSONArray()).put("version",1).put("resolution_protocol",1).put("token","a".repeat(32)).put("fingerprint","b".repeat(64)).put("source_files_modified",false).put("index_modified",false).put("needs_save",true).put("stage_available",true).put("stage_unavailable",JSONObject.NULL).put("plan",JSONObject().put("patches",JSONArray().put(JSONObject().put("path","a.py").put("before",side("a","before")).put("after",side("a","after"))))).put("saved_sources",JSONObject().put("a.py",JSONObject().put("kind","file").put("text","after").put("sha256",debugHash("after")).put("readonly",false).put("executable",false).put("mode",420))).put("consequences",JSONObject())
 @Test fun immutableReviewRejectsMalformedIdentityAndDistinguishesResourceDrafts(){val raw=preview();val value=gitResolutionPreview(raw){true};assertTrue(gitResolutionDraftable(value));assertFalse(gitResolutionDraftable(value.copy(patches=listOf(ResourceEditorPatch("a.py",ResourceSource("a","same"),ResourceSource("a","same"))))));raw.getJSONObject("plan").getJSONArray("patches").getJSONObject(0).getJSONObject("after").put("text","tampered");assertEquals("after",value.patches.single().after!!.text)
  val staged=JSONObject().put("version",1).put("operation","git_conflict_stage").put("request_id","request").put("token",value.token).put("review_fingerprint",value.fingerprint).put("fingerprint","c".repeat(64)).put("complete",true).put("index_modified",true).put("index_uncertain",false).put("source_files_modified",false).put("resolved_members",JSONArray().put("a.py")).put("remaining_conflicts",JSONArray())
  gitResolutionStageReceipt(staged,"request",value)
  for((key,bad) in listOf("request_id" to "foreign","resolved_members" to JSONArray(),"remaining_conflicts" to JSONArray().put("a.py"),"complete" to "true","fingerprint" to "bad"))assertTrue(runCatching{gitResolutionStageReceipt(JSONObject(staged.toString()).put(key,bad),"request",value)}.isFailure)
  for((key,bad) in listOf("version" to true,"resolution_protocol" to 1.5,"token" to "foreign","fingerprint" to "old","index_modified" to true,"needs_save" to "false","stage_available" to JSONObject.NULL))assertTrue(runCatching{gitResolutionPreview(preview().put(key,bad)){true}}.isFailure)
  assertFalse(gitResolutionDraftable(value.copy(patches=listOf(ResourceEditorPatch("a.py",ResourceSource("a","before"),null)))))
  assertFalse(gitResolutionDraftable(value.copy(patches=listOf(ResourceEditorPatch("a.py",ResourceSource("a","before"),ResourceSource("other","after"))))))
 }
 @Test fun explicitDestinationsDoNotInferRenameAndKeepStageObjectIdentity(){val output=GitResolutionOutput("new λ%?#.py","left.py","composed");val raw=output.json();assertEquals("left.py",raw.getString("identity_from"));assertEquals("text",raw.getJSONObject("content").getString("kind"));assertEquals("composed",raw.getJSONObject("content").getString("text"));val stage=output.copy(stagePath="right.py",stage=3,oid="c".repeat(40));assertEquals("left.py",stage.json().getString("identity_from"));assertEquals("right.py",stage.json().getJSONObject("content").getString("path"));assertEquals("c".repeat(40),stage.json().getJSONObject("content").getString("oid"));assertTrue(output.copy(identityFrom=null).json().isNull("identity_from"))}
 @Test fun closedAndSupersededReadsCannotReviveComposition()=runBlocking{
  val waits=mutableListOf<CompletableDeferred<JSONObject>>();val model=GitResolutionModel{_,_,_->CompletableDeferred<JSONObject>().also{waits.add(it)}.await()};val workspace=Workspace(this)
  fun read(path:String)=JSONObject().put("version",1).put("resolution_protocol",1).put("paths",JSONArray().put(JSONObject().put("path",path).put("working",JSONObject.NULL).put("stages",JSONArray())))
  val pending=async{model.read(workspace,"a.py")};while(waits.isEmpty())yield();model.close();waits[0].complete(read("a.py"));pending.await();assertNull(model.reading);assertFalse(model.busy)
  model.add();model.output(0,GitResolutionOutput("chosen.py",null,"my composition"));val next=async{model.read(workspace,"a.py")};while(waits.size<2)yield();waits[1].complete(read("a.py"));next.await();assertEquals("my composition",model.outputs.single().text);assertEquals("chosen.py",model.outputs.single().path)
 }
 private suspend fun host(block:suspend(Workspace,DocumentBuffer,GitResolutionModel,MutableList<String>,java.util.concurrent.atomic.AtomicReference<String>)->Unit)=coroutineScope{
  val server=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0)
  server.createContext("/api/changes/resource-inputs"){exchange->val bytes=JSONObject().put("saved_guards",JSONObject().put("a.py","saved-guard")).put("sources",JSONObject().put("a.py",JSONObject().put("kind","file"))).toString().toByteArray();exchange.sendResponseHeaders(200,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}};server.start()
  val w=Workspace(this);w.engine=Engine("http://127.0.0.1:${server.address.port}");val file=FileNode("a.py","a.py",false,emptyList(),null,null);val b=DocumentBuffer(file,FileText("before","saved","saved-guard"));w.buffers[file.path]=b;val calls=mutableListOf<String>();val mode=java.util.concurrent.atomic.AtomicReference("pending")
  val model=GitResolutionModel{_,path,body->calls.add(path);when{
   path.endsWith("/read")->JSONObject().put("version",1).put("resolution_protocol",1).put("paths",JSONArray().put(JSONObject().put("path","a.py").put("stages",JSONArray()).put("working",JSONObject().put("kind","file").put("text","before"))))
   path.endsWith("resource-inputs")->JSONObject().put("saved_guards",JSONObject().put("a.py","saved-guard")).put("sources",JSONObject().put("a.py",JSONObject().put("kind","file")))
   path.endsWith("/prepare")->preview()
   path.endsWith("/save")->error("lost response")
   path.contains("/status?")->JSONObject().put("version",1).put("request_id",path.substringAfter("request_id=")).put("state",mode.get()).put("outcome",JSONObject.NULL).put("error",if(mode.get()=="completed")"explicit refusal" else JSONObject.NULL)
   path.endsWith("/validate")->JSONObject().put("current",true).put("stage",body?.optBoolean("stage") ?: false).put("token","a".repeat(32)).put("fingerprint","b".repeat(64)).put("plan",preview().getJSONObject("plan")).put("saved_sources",preview().getJSONObject("saved_sources"))
   else->error("Unexpected route $path")
  }}
  try{block(w,b,model,calls,mode)}finally{server.stop(0)}
 }
 @Test fun returnedTextRevisionAndNewConstructionBufferInvalidateWholeReview()=runBlocking{for(change in listOf("revision","namespace","metadata","baseline"))host{w,b,m,calls,_->m.read(w,"a.py");m.prepare(w);assertNotNull(m.notice,m.preview);val lease=w.captureRetainedSourceNamespace();assertTrue(lease());if(change=="revision")b.advanceSourceRevision() else if(change=="metadata")b.file=b.file.copy(generatedBy="owner") else if(change=="baseline")b.base="foreign baseline" else w.buffers["new.md"]=DocumentBuffer(FileNode("new.md","new.md",false,emptyList(),null,null),FileText("draft","saved"));assertFalse(lease());m.act(w,false);assertFalse(m.hasPending);assertFalse(calls.any{it.endsWith("/save")});assertEquals("before",b.text)}}
 @Test fun uncertainRequestNeverRetriesAndOnlyCompletedOriginalPreimageCanRetire()=runBlocking{host{w,b,m,calls,mode->m.read(w,"a.py");m.prepare(w);m.act(w,false);assertTrue(m.hasPending);m.close();m.act(w,false);assertEquals(1,calls.count{it.endsWith("/save")});m.status(w);assertTrue(m.hasPending);mode.set("unknown");m.status(w);assertTrue(m.hasPending);mode.set("completed");m.status(w);assertFalse(m.notice,m.hasPending);assertNull(m.preview);assertEquals("before",b.text);assertEquals(2,calls.count{it.endsWith("/validate")})}}
 @Test(timeout=120000) fun liveNativeCompositionSaveStageAndEditorHistory()=runBlocking{
  val origin=System.getenv("HICKORY_NATIVE_GIT_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_NATIVE_GIT_WORKSPACE") ?: error("Use the owned Git resolution fixture"));val path=System.getenv("HICKORY_NATIVE_GIT_INPUT") ?: error("Missing literal fixture path");check(root.name=="native"&&root.parentFile.name.startsWith("hickory-git-resolutions-")){"Refuse outside owned temporary fixture"}
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(background)
  try{
  w.engine=Engine(origin!!);val (_,tree)=w.engine!!.files();assertEquals(root.toPath().toRealPath().toString(),w.engine!!.sourceRootAddress);w.rootFiles=tree
  fun find(nodes:List<FileNode>):FileNode?=nodes.firstNotNullOfOrNull{if(it.path==path)it else find(it.children)}
  val b=w.loadBuffer(find(tree) ?: error("Literal conflict source missing"));w.show(path);val state=b.state;val old=b.text;val scroll=b.sourceScroll.value;b.selectSourceRange(0,0,old)
  val m=w.gitResolution;m.read(w,path);assertNotNull(m.notice,m.reading);m.output(0,GitResolutionOutput(path,path,"composed = 41\n"));m.prepare(w);assertNotNull(m.notice,m.preview);assertEquals(old,b.text)
  m.draft(w);assertEquals(m.notice,"composed = 41\n",b.text);w.changes.open("undo",w);assertEquals(old,b.text);w.changes.open("redo",w);assertEquals("composed = 41\n",b.text);w.changes.open("undo",w);assertEquals(old,b.text)
  m.read(w,path,true);m.prepare(w);assertNotNull(m.notice,m.preview);m.act(w,false);assertFalse(m.notice,m.hasPending);assertTrue(m.notice,m.saved);assertEquals("composed = 41\n",root.resolve(path).readText());assertEquals("composed = 41\n",b.base);assertSame(state,b.state);assertSame(b,w.currentBuffer);assertEquals(scroll,b.sourceScroll.value)
  m.act(w,true);assertFalse(m.notice,m.hasPending);assertTrue(m.notice,m.notice.contains("staged"));assertSame(state,b.state);assertSame(b,w.buffers[path]);assertEquals("composed = 41\n",b.text)
  }finally{w.buffers.values.forEach{it.language?.close()};background.cancel()}
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/GitConflictResolutionTest.kt"><ui:paste select="#git-resolution-android-tests"/></ui:file>
</ui:doc>
