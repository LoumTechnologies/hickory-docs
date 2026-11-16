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
  TextButton(enabled=!model.busy&&!model.hasPending,onClick={scope.launch{model.read(w,path)}}){Text("Compose / resolve conflict group")}
  if(model.notice.isNotEmpty())Text(model.notice,fontSize=12.sp)
  if(model.hasPending)TextButton(enabled=!model.busy,onClick={scope.launch{model.status(w)}}){Text("Inspect original resolution request")}
  model.reading?.let{reading->
   val rows=reading.getJSONArray("paths");Text("Select every conflict member belonging to this resolution. Rename relationships are explicit choices.",fontSize=12.sp)
   for(i in 0 until rows.length()){val row=rows.getJSONObject(i);val member=row.getString("path");Row{Checkbox(modifier=Modifier.semantics{contentDescription="Include ${JSONObject.quote(member)} in this conflict resolution group"},checked=member in model.members,enabled=!model.busy&&!model.hasPending,onCheckedChange={model.choose(member)});Text(JSONObject.quote(member),fontSize=12.sp)};if(!row.isNull("unavailable"))Text(row.getString("unavailable"),fontSize=12.sp)}
   TextButton(enabled=!model.busy&&!model.hasPending&&model.members.isNotEmpty(),onClick={scope.launch{model.read(w,path,true)}}){Text("Read selected group stages")}
   model.outputs.forEachIndexed{index,output->
    HorizontalDivider();OutlinedTextField(value=output.path,onValueChange={model.output(index,output.copy(path=it))},label={Text("Destination ${index+1}")},enabled=!model.busy&&!model.hasPending)
    Text("Retained editor identity: ${output.identityFrom?.let{JSONObject.quote(it)} ?: "New source"}",fontSize=12.sp)
    Row{TextButton(enabled=!model.busy&&!model.hasPending,onClick={model.output(index,output.copy(identityFrom=null))}){Text("New identity")};model.members.sorted().forEach{member->TextButton(enabled=!model.busy&&!model.hasPending,onClick={model.output(index,output.copy(identityFrom=member))}){Text("Retain ${JSONObject.quote(member)}")}}}
    OutlinedTextField(value=output.text,onValueChange={model.output(index,output.copy(text=it,stage=null,stagePath=null,oid=null))},label={Text("Composed resolution · ${output.path.ifEmpty{"destination ${index+1}"}}")},minLines=3,maxLines=10,enabled=!model.busy&&!model.hasPending)
    Row{TextButton(enabled=!model.busy&&!model.hasPending,onClick={model.output(index,output.copy(mode=if(output.mode=="100644")"100755" else "100644"))}){Text("Mode ${output.mode}")};TextButton(enabled=!model.busy&&!model.hasPending,onClick={model.output(index,null)}){Text("Remove ${JSONObject.quote(output.path)} destination (delete if not retained elsewhere)")}}
    for(i in 0 until rows.length()){val row=rows.getJSONObject(i);val member=row.getString("path");if(member !in model.members)continue;val stages=row.getJSONArray("stages");for(j in 0 until stages.length()){val stage=stages.getJSONObject(j);if(!stage.isNull("text"))TextButton(enabled=!model.busy&&!model.hasPending,onClick={model.output(index,output.copy(text=stage.getString("text"),stagePath=member,stage=stage.getInt("stage"),oid=stage.getString("oid")))}){Text("Use ${JSONObject.quote(member)} stage ${stage.getInt("stage")} bytes")}}}
   }
   TextButton(enabled=!model.busy&&!model.hasPending,onClick={model.add()}){Text("Add explicit destination")}
   Text("Members without a retained destination will be deleted: ${model.members.filter{member->model.outputs.none{it.identityFrom==member}}.joinToString{JSONObject.quote(it)}}",fontSize=12.sp)
   TextButton(enabled=!model.busy&&!model.hasPending,onClick={scope.launch{model.prepare(w)}}){Text("Preview complete resolution consequences")}
  }
  model.preview?.let{review->
   Text("Complete reviewed source/resource proposal, saved metadata and both generated layers",fontSize=12.sp)
   // Page the complete immutable receipt. No consequence is silently omitted.
   val pretty=remember(review.raw){JSONObject(review.raw).toString(2)};val count=((pretty.length+7999)/8000).coerceAtLeast(1);val page=consequencePage.coerceIn(0,count-1)
   Text(pretty.substring(page*8000,((page+1)*8000).coerceAtMost(pretty.length)),fontSize=11.sp)
   Row{TextButton(enabled=page>0,onClick={consequencePage=page-1}){Text("Previous consequences")};Text("${page+1}/$count");TextButton(enabled=page<count-1,onClick={consequencePage=page+1}){Text("Next consequences")}}
   if(!review.canStage)Text(review.stageReason ?: "Staging unavailable",fontSize=12.sp)
   TextButton(enabled=!model.busy&&!model.hasPending&&gitResolutionDraftable(review),onClick={scope.launch{model.draft(w)}}){Text("Apply text drafts only")}
   TextButton(enabled=!model.busy&&!model.hasPending&&review.needsSave&&!model.saved,onClick={scope.launch{model.act(w,false)}}){Text("Save complete resolution group")}
   TextButton(enabled=!model.busy&&!model.hasPending&&review.canStage&&(!review.needsSave||model.saved),onClick={scope.launch{model.act(w,true)}}){Text("Stage complete saved group")}
  }
 }
}
