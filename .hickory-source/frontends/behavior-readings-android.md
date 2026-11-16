<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep connected behavior readings and verification requests distinct

For Android maintainers extending the existing construction popover without adding permanent editor chrome.

A connected reading captures the entry, supplied construction drafts and missing sources. The host binds the canonical Files root_path digest, retained editor identities, complete namespace and monotonic source revisions. A delayed or failed newer reading retires execution authority while prior captured text remains readable. Contracts, explanations, implementation, generated outputs, declared tests and evidence retain the service's declared/observed/manual/stale/missing basis; truncated previews and traversals remain explicitly incomplete.

Only an explicitly selected related source-test candidate or an explicitly chosen engine-discovered launch profile can start relevant verification. A profile association remains user-declared. The request binds the exact context, reading fingerprint, check and profile inventory. Consume its unique ID before sending, retain uncertain requests without automatic retry, and admit only exact scoped receipts. A fresh reading can repeat a known terminal check; uncertain prior work remains consumed. Process exits are retained observations, never inferred case verdicts. Revalidate a retained receipt before attaching its existing terminal; absence leaves historical output readable and starts no replacement.

The graph provider owns an opaque physical-root identity. The host compares the separately echoed canonical namespace and captured engine workspace lifetime; hashing a path spelling cannot prove the same root survived replacement. Profile and verification requests carry that captured runtime identity. Legacy origin-unknown process receipts stay readable but cannot attach a live terminal.

<ui:copy id="behavior-readings-android">
@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
package com.hickorydocs.simple
import androidx.compose.runtime.*
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import org.json.JSONObject
import org.json.JSONArray
fun behaviorCanonical(value:Any?):String=when(value){null,JSONObject.NULL->"null";is JSONObject->value.keys().asSequence().sorted().joinToString(",","{","}"){JSONObject.quote(it)+":"+behaviorCanonical(value.get(it))};is JSONArray->(0 until value.length()).joinToString(",","[","]"){behaviorCanonical(value.get(it))};is String->JSONObject.quote(value);else->value.toString()}
private fun behaviorDigest(value:String)=value.matches(Regex("[a-f0-9]{64}"))
data class BehaviorCheck(val id:String,val label:String,val path:String,val available:Boolean,val reason:String)
data class BehaviorProfile(val id:String,val name:String,val fingerprint:String)
data class BehaviorSnapshot(val workspace:String,val fingerprint:String,val context:String,val raw:String,val checks:List<BehaviorCheck>,val verification:Boolean,val namespace:String,val engineWorkspace:String)
data class BehaviorReceipt(val id:String,val workspace:String,val fingerprint:String,val check:String,val profile:BehaviorProfile?,val status:String,val outcome:String,val raw:String,val engine:Engine,val runtime:String?,val origin:String?)
fun behaviorSnapshot(raw:JSONObject,context:JSONObject,address:String,runtime:String):BehaviorSnapshot {
 check(raw.get("version")==1&&raw.get("behavior_protocol")==1&&runtime.matches(Regex("[a-f0-9]{32}"))&&raw.getString("namespace")==address&&raw.getString("engine_workspace_id")==runtime&&(behaviorDigest(raw.getString("workspace_id"))||(raw.opt("verification_available")==false&&raw.getString("workspace_id").isEmpty()))&&behaviorDigest(raw.getString("reading_fingerprint"))&&behaviorCanonical(raw.getJSONObject("context"))==behaviorCanonical(context)){"Foreign or unsupported connected behavior reading. No check was started."}
 check(!raw.has("verification_available")||raw.get("verification_available") is Boolean){"Malformed verification availability."}
 val graph=raw.getJSONObject("graph");graph.getJSONObject("nodes");graph.getJSONArray("edges");val checks=graph.getJSONArray("checks");check(checks.length()<=10000){"Check reading exceeds bounded capacity."};val seen=mutableSetOf<String>()
 return BehaviorSnapshot(raw.getString("workspace_id"),raw.getString("reading_fingerprint"),context.toString(),raw.toString(),(0 until checks.length()).map{val item=checks.getJSONObject(it);val id=item.getString("id");check(id.isNotEmpty()&&seen.add(id)){"Ambiguous check identity."};BehaviorCheck(id,item.getString("label"),item.getString("path"),item.get("available") as? Boolean ?: error("Malformed check availability."),item.getString("reason"))},raw.opt("verification_available")==true,address,runtime)
}
class BehaviorReadingModel(private val transport:(suspend(Engine,String,JSONObject?)->JSONObject)?=null){
 var value by mutableStateOf<BehaviorSnapshot?>(null);private set
 var notice by mutableStateOf("");private set
 var loading by mutableStateOf(false);private set
 var profiles by mutableStateOf<List<BehaviorProfile>>(emptyList());private set
 var receipts by mutableStateOf<List<BehaviorReceipt>>(emptyList());private set
 var sourcePage by mutableStateOf<BehaviorSourcePage?>(null);private set
 var sourceLoading by mutableStateOf(false);private set
 private var sourcePageTicket=-1L
 private var sourceTicket=0L
 private var ticket=0L;private var current:()->Boolean={false};private var engine:Engine?=null
 private data class Consumed(val id:String,val ticket:Long)
 private val consumed=linkedMapOf<String,Consumed>()
 private suspend fun call(engine:Engine,path:String,body:JSONObject?=null)=transport?.invoke(engine,path,body) ?: engine.request(if(body==null)"GET" else "POST",path,body)
 val currentReading get()=value!=null&&current()
 fun close(){ticket++;sourceTicket++;sourceLoading=false;current={false};loading=false;profiles=emptyList()}
 suspend fun read(w:Workspace,entry:String,focus:String?){val engine=w.engine ?: run{notice="Connect to a workspace server.";return};val address=engine.sourceRootAddress ?: run{notice="Refresh Files to bind the exact workspace address.";return};val runtime=engine.sourceWorkspaceID ?: run{notice="Refresh Files to bind this engine workspace lifetime.";return};val rows=w.buffers.toMap();val namespace=w.captureRetainedSourceNamespace()
  val valid={namespace()&&engine.sourceRootAddress==address&&engine.sourceWorkspaceID==runtime&&w.showConstruction}
  val sources=JSONObject();val missing=JSONArray();rows.forEach{(path,b)->if(b.file.generatedBy==null&&(path.endsWith(".md",true)||path.endsWith(".markdown",true))){if(b.resourceDeleted)missing.put(path) else sources.put(path,b.text)}}
  read(engine,address,JSONObject().put("version",1).put("entry",entry).put("sources",sources).put("focus",focus ?: JSONObject.NULL).put("missing_sources",missing),runtime,valid)
 }
 suspend fun read(engine:Engine,address:String,context:JSONObject,runtime:String,current:()->Boolean){val input=JSONObject(context.toString());val id=++ticket;sourceTicket++;sourceLoading=false;this.current={false};loading=true;profiles=emptyList();notice="Reading connected behavior…"
  try{val raw=call(engine,"/api/behavior/reading",input);if(id!=ticket)return;val next=behaviorSnapshot(raw,input,address,runtime);value=next;this.engine=engine;this.current=current
   val retained=raw.optJSONObject("receipts")?.optJSONArray("records");if(retained!=null)for(index in retained.length()-1 downTo 0){val row=retained.getJSONObject(index);row.put("freshness",if(row.optString("reading_fingerprint")==next.fingerprint)"current-reading" else "historical-reading");runCatching{decode(row,null,engine,row.getString("workspace_id"))}.onSuccess{receipt->retain(receipt);if(receipt.workspace==next.workspace&&receipt.runtime==next.engineWorkspace&&receipt.origin!="unknown"&&receipt.status in listOf("admitting","process-running","request-pending")){val key=key(receipt.fingerprint,receipt.check,receipt.profile);if(consumed.size<256)consumed[key]=Consumed(receipt.id,ticket)}}}
   notice=if(current())"Captured declared relationships, observed construction and explicitly qualified evidence." else "Captured reading is historical: editor, ownership or workspace changed."
  }catch(e:CancellationException){throw e}catch(e:Exception){if(id==ticket)notice=e.message.orEmpty()}finally{if(id==ticket)loading=false}
 }
 private fun readingAuthority(value:BehaviorSnapshot){check(this.value===value&&current()){"Source/editor ownership or workspace changed. Refresh the reading."}}
 private fun admission(value:BehaviorSnapshot):JSONObject{readingAuthority(value);check(value.verification){"Construction input authority is unavailable. Repair it before verification."};return JSONObject().put("engine_workspace_id",value.engineWorkspace).put("workspace_id",value.workspace).put("reading_fingerprint",value.fingerprint).put("context",JSONObject(value.context))}
 val currentSourcePage get()=sourcePageTicket==ticket&&sourcePage?.let{it.workspace==value?.workspace&&it.fingerprint==value?.fingerprint&&current()}==true
 suspend fun readSource(nodeID:String,offset:Int?=null){val value=value ?: return;val engine=engine ?: return;val scope=ticket;val request=++sourceTicket;sourceLoading=true
  try{readingAuthority(value);check(offset==null||offset>=0){"Choose a nonnegative Unicode scalar offset."};val node=JSONObject(value.raw).getJSONObject("graph").getJSONObject("nodes").getJSONObject(nodeID);check(node.getString("id")==nodeID&&node.optString("hash").matches(Regex("[a-f0-9]{64}"))){"Full captured text is unavailable for this node; its preview remains readable."}
   val body=JSONObject().put("version",1).put("workspace_id",value.workspace).put("reading_fingerprint",value.fingerprint).put("node_id",nodeID).put("source_hash",node.getString("hash")).put("context",JSONObject(value.context));if(offset!=null)body.put("offset",offset)
   val raw=call(engine,"/api/behavior/source",body);if(scope!=ticket||request!=sourceTicket)return;readingAuthority(value);sourcePage=behaviorSourcePage(raw,value,nodeID,node,offset,sourcePage);sourcePageTicket=ticket;notice="Captured full text · ${sourcePage!!.origin} · ${sourcePage!!.target}. Current editor and later disk bytes were not substituted."
  }catch(e:CancellationException){throw e}catch(e:Exception){if(scope==ticket&&request==sourceTicket)notice=e.message.orEmpty()}finally{if(scope==ticket&&request==sourceTicket)sourceLoading=false}
 }
 suspend fun continueSource(page:BehaviorSourcePage,offset:Int){if(sourcePage!==page||!currentSourcePage){notice="Captured page belongs to a retired reading. Select full captured text in the current reading; prior text remains readable.";return};readSource(page.node,offset)}
 suspend fun openCurrentSource(w:Workspace,path:String){try{check(w.engine===engine&&value?.namespace==w.engine?.sourceRootAddress&&value?.engineWorkspace==w.engine?.sourceWorkspaceID){"This reading belongs to the original workspace connection."}
  fun find(rows:List<FileNode>):FileNode?{for(file in rows){if(file.path==path)return file;find(file.children)?.let{return it}};return null}
  val file=find(w.rootFiles)?.takeUnless{it.dir} ?: error("Current source is not listed. Captured text remains readable; no path fallback was attempted.")
  w.open(file);notice="Opened current readable source. Captured text remains a separate reading; no captured position was inferred."
 }catch(e:CancellationException){throw e}catch(e:Exception){notice=e.message.orEmpty()}}
 suspend fun chooseProfiles(){val value=value ?: return;val engine=engine ?: return;val id=ticket
  try{val raw=call(engine,"/api/behavior/profiles",admission(value));if(id!=ticket)return;admission(value);val fp=raw.getString("fingerprint");check(behaviorDigest(fp)){"Profile inventory changed or unavailable."};val rows=raw.getJSONArray("profiles");val seen=mutableSetOf<String>();profiles=(0 until rows.length()).map{val row=rows.getJSONObject(it);val choice=row.getString("id");check(choice.isNotEmpty()&&seen.add(choice)){"Ambiguous profile choice."};BehaviorProfile(choice,row.getString("name"),fp)};notice="Choose an existing engine-discovered profile explicitly. Its relationship to this check is user-declared."
  }catch(e:CancellationException){throw e}catch(e:Exception){notice=e.message.orEmpty()}
 }
 private fun key(fp:String,check:String,profile:BehaviorProfile?)=behaviorCanonical(JSONArray().put(fp).put(check).put(profile?.id ?: JSONObject.NULL).put(profile?.fingerprint ?: JSONObject.NULL))
 suspend fun run(checkID:String,profile:BehaviorProfile?=null){val value=value ?: return;val engine=engine ?: return;var request:String?=null
  try{val body=admission(value);val selectedCheck=value.checks.firstOrNull{it.id==checkID} ?: error("Choose a captured related check.");check(selectedCheck.available||profile!=null){selectedCheck.reason};if(profile!=null)check(profile in profiles){"Profile choice changed. Choose again."}
   val key=key(value.fingerprint,checkID,profile);val prior=consumed[key];check(prior==null||(prior.ticket!=ticket&&receipts.any{it.id==prior.id&&it.status in listOf("process-finished","admission-refused","request-refused")})){"Verification request already consumed. Refresh its retained outcome; no replacement process was started."};check(consumed.size<256||prior!=null){"Retained verification capacity reached. Inspect uncertain requests."}
   val id=java.util.UUID.randomUUID().toString().replace("-","");request=id;consumed[key]=Consumed(id,ticket)
   val profileJSON=profile?.let{JSONObject().put("id",it.id).put("fingerprint",it.fingerprint)} ?: JSONObject.NULL
   val pending=JSONObject().put("version",1).put("request_id",id).put("engine_workspace_id",value.engineWorkspace).put("workspace_id",value.workspace).put("reading_fingerprint",value.fingerprint).put("check_id",checkID).put("label",selectedCheck.label).put("profile",profileJSON).put("status","request-pending").put("outcome","unknown").put("notice","Uncertain response never permits automatic retry.")
   val held=decode(pending,null,engine,value.workspace);retain(held);val raw=call(engine,"/api/behavior/verify",body.put("version",1).put("check_id",checkID).put("request_id",id).put("profile",profileJSON));retain(decode(raw,held,engine,value.workspace));notice="Verification receipt retained. A process exit is not a collected test verdict."
  }catch(e:CancellationException){if(request!=null)notice="Verification may have started. Inspect original request; never retry automatically.";throw e}catch(e:Exception){notice=e.message.orEmpty()+if(request!=null)" Inspect the original retained request; uncertain responses never authorize replacement." else ""}
 }
 private fun decode(raw:JSONObject,pending:BehaviorReceipt?,engine:Engine,workspace:String):BehaviorReceipt{
  val id=raw.getString("request_id");val fp=raw.getString("reading_fingerprint");val checkedID=raw.getString("check_id");check(raw.get("version")==1&&id.matches(Regex("[a-f0-9]{32}"))&&raw.getString("workspace_id")==workspace&&behaviorDigest(fp)&&checkedID.isNotEmpty()){ "Foreign verification receipt refused; keep the original request." }
  val runtime=if(raw.has("engine_workspace_id"))raw.getString("engine_workspace_id").also{check(it.matches(Regex("[a-f0-9]{32}"))){"Malformed verification engine lifetime."}}else null;val origin=if(raw.isNull("origin"))null else raw.getString("origin")
  val profile=raw.optJSONObject("profile")?.let{BehaviorProfile(it.getString("id"),"",it.getString("fingerprint"))};if(pending!=null)check(runtime==pending.runtime&&id==pending.id&&fp==pending.fingerprint&&checkedID==pending.check&&profile?.id==pending.profile?.id&&profile?.fingerprint==pending.profile?.fingerprint){"Verification receipt belongs to another source/check/profile."}
  return BehaviorReceipt(id,workspace,fp,checkedID,profile,raw.getString("status"),raw.getString("outcome"),raw.toString(),engine,runtime,origin)
 }
 private fun retain(value:BehaviorReceipt){receipts=(listOf(value)+receipts.filter{it.id!=value.id}).take(50)}
 suspend fun refresh(receipt:BehaviorReceipt){try{retain(decode(call(receipt.engine,"/api/behavior/verification?id=${receipt.id}"),receipt,receipt.engine,receipt.workspace));notice="Retained process outcome refreshed; no check was restarted."}catch(e:CancellationException){throw e}catch(e:Exception){notice=e.message.orEmpty()}}
 suspend fun openProcess(w:Workspace,receipt:BehaviorReceipt){try{check(w.engine===receipt.engine&&receipt.runtime!=null&&receipt.runtime==w.engine?.sourceWorkspaceID&&receipt.origin=="current-workspace"){"Process belongs to the original connection; inspect its retained output."};val fresh=decode(call(receipt.engine,"/api/behavior/verification?id=${receipt.id}"),receipt,receipt.engine,receipt.workspace);retain(fresh);check(fresh.runtime==w.engine?.sourceWorkspaceID&&fresh.origin=="current-workspace"){"Retained process is historical or belongs to another engine lifetime."};val terminal=JSONObject(fresh.raw).optJSONObject("terminal")?.getString("id") ?: error("Retained process no longer exists in this engine. No replacement was adopted.");check(terminal.isNotEmpty()){ "Invalid retained process identity." };w.terminal.id=terminal;w.show("terminal")}catch(e:CancellationException){throw e}catch(e:Exception){notice=e.message.orEmpty()}}
}

@Composable fun BehaviorReadingView(w:Workspace){
 val m=w.behaviorReading;val scope=rememberCoroutineScope();var nodeID by remember{mutableStateOf("")};var checkID by remember{mutableStateOf("")};var profileID by remember{mutableStateOf("")};var page by remember{mutableIntStateOf(0)}
 Column(verticalArrangement=Arrangement.spacedBy(androidx.compose.ui.unit.Dp(6f))){
  Text("Connected behavior and relevant verification");Text(m.notice,fontSize=12.sp);if(m.value!=null&&!m.currentReading)Text("Historical reading: editor/namespace authority changed. Refresh before verification.",fontSize=12.sp)
  m.value?.let{value->val raw=JSONObject(value.raw);val graph=raw.getJSONObject("graph");Text(raw.optString("evidence_limit"),fontSize=12.sp);val notices=graph.getJSONArray("notices");for(i in 0 until notices.length())Text(notices.getString(i),fontSize=12.sp);if(graph.optBoolean("truncated"))Text("Relationship reading truncated; omitted relationships remain unknown.",fontSize=12.sp)
   val nodes=graph.getJSONObject("nodes");val keys=nodes.keys().asSequence().toList();keys.forEach{id->val node=nodes.getJSONObject(id);TextButton(onClick={nodeID=id;page=0}){Text("${node.getString("kind")} · ${node.getString("label")} · ${node.getString("status")}")}}
   (nodeID.takeIf{nodes.has(it)} ?: keys.firstOrNull())?.let{id->val node=nodes.getJSONObject(id);Text(node.getString("path"),fontSize=12.sp);Text("${node.getString("status")} · captured hash ${node.opt("hash")}",fontSize=12.sp);val preview=if(node.isNull("preview"))"Readable captured preview unavailable; missing content is not inferred." else node.getString("preview");val count=((preview.length+7999)/8000).coerceAtLeast(1);val current=page.coerceIn(0,count-1);Text(preview.substring(current*8000,((current+1)*8000).coerceAtMost(preview.length)),fontSize=12.sp);Text("Captured previews may be abbreviated by the provider.",fontSize=12.sp);Row{TextButton(enabled=current>0,onClick={page=current-1}){Text("Previous preview")};TextButton(enabled=current<count-1,onClick={page=current+1}){Text("Next preview")}}
    val path=if(node.isNull("source_path"))node.getString("path") else node.getString("source_path");TextButton(onClick={scope.launch{m.openCurrentSource(w,path)}}){Text("Open current readable source · ${JSONObject.quote(path)}")}
    TextButton(enabled=!m.sourceLoading&&m.currentReading,onClick={scope.launch{m.readSource(id)}}){Text("Read full captured text") }
    val edges=graph.getJSONArray("edges");for(i in 0 until edges.length()){val edge=edges.getJSONObject(i);if(edge.getString("from")==id||edge.getString("to")==id)Text("${edge.getString("relationship")} · ${edge.getString("basis")}: ${edge.getString("reason")} Authority: ${edge.getString("authority")}",fontSize=12.sp)}
   }
   raw.optJSONObject("impact")?.let{impact->Text("Change impact · ${impact.getString("basis")}: ${impact.getString("reason")}",fontSize=12.sp);if(impact.optBoolean("truncated"))Text("Impact traversal truncated; more relationships may be unknown.",fontSize=12.sp);val via=impact.getJSONArray("via");for(i in 0 until via.length())Text(via.getJSONObject(i).getString("reason"),fontSize=12.sp)}
   val evidence=raw.getJSONObject("evidence");Text("Retained evidence · ${evidence.getString("status")}",fontSize=12.sp);if(evidence.has("notice"))Text(evidence.getString("notice"),fontSize=12.sp);val records=evidence.getJSONArray("records");for(i in 0 until records.length())Text(records.getJSONObject(i).toString(2),fontSize=11.sp);if(evidence.optBoolean("truncated"))Text("Evidence projection truncated; omitted records remain unknown.",fontSize=12.sp)
   value.checks.forEach{check->TextButton(onClick={checkID=check.id;profileID=""}){Text("Related check · ${check.label} · ${if(check.available)"source candidate" else "unsupported automatic provider"}")}}
   value.checks.firstOrNull{it.id==checkID}?.let{check->Text(check.reason,fontSize=12.sp);TextButton(enabled=check.available&&value.verification,onClick={scope.launch{m.run(check.id)}}){Text("Run selected relevant test")};TextButton(enabled=value.verification,onClick={scope.launch{m.chooseProfiles()}}){Text("Choose profile for selected check")};m.profiles.forEach{profile->TextButton(onClick={profileID=profile.id}){Text("Verification profile · ${profile.name}")}};m.profiles.firstOrNull{it.id==profileID}?.let{profile->TextButton(onClick={scope.launch{m.run(check.id,profile)}}){Text("Run user-declared profile for selected check")}}}
   if(value.checks.isEmpty())Text("No explicitly related checks. Undeclared test filenames are not relevance evidence.",fontSize=12.sp)
  }
  m.sourcePage?.let{page->
   Text("Captured full reading · ${page.origin} · ${page.state}${if(!m.currentSourcePage)" · historical context" else ""}",fontSize=12.sp)
   Text("${page.path ?: "Retained summary"}${page.fragment?.let{" · declared fragment #$it"} ?: ""} · ${page.target}",fontSize=12.sp)
   Text("Unicode scalars ${page.offset}–${page.offset+page.text.codePointCount(0,page.text.length)} / ${page.total} · source hash ${page.sourceHash}",fontSize=11.sp)
   androidx.compose.foundation.text.selection.SelectionContainer{Text(page.text,fontSize=12.sp)};Text(page.notice,fontSize=12.sp)
   Row{TextButton(enabled=m.currentSourcePage&&!m.sourceLoading&&page.offset>0,onClick={scope.launch{m.continueSource(page,0)}}){Text("Captured source start")};TextButton(enabled=m.currentSourcePage&&!m.sourceLoading&&page.offset>0,onClick={scope.launch{m.continueSource(page,(page.offset-4000).coerceAtLeast(0))}}){Text("Previous captured page")};TextButton(enabled=m.currentSourcePage&&!m.sourceLoading&&page.next!=null,onClick={scope.launch{page.next?.let{m.continueSource(page,it)}}}){Text("Next captured page")}}
  }
  m.receipts.forEach{receipt->val raw=JSONObject(receipt.raw);Text("${raw.optString("label")} · ${receipt.status} · ${receipt.outcome}${if(raw.has("exit_code"))" · process exit ${raw.get("exit_code")}" else ""}",fontSize=12.sp);Text("${raw.optString("relationship")} · ${raw.optString("freshness","captured request")} · ${raw.optString("notice")} ${raw.optString("error","")}",fontSize=12.sp);Text(receipt.id,fontSize=11.sp);TextButton(onClick={scope.launch{m.refresh(receipt)}}){Text("Refresh retained outcome")};if(raw.optJSONObject("terminal")!=null)TextButton(onClick={scope.launch{m.openProcess(w,receipt)}}){Text("Open verification process")};if(raw.has("output"))Text(raw.getString("output"),fontSize=11.sp);if(raw.optBoolean("output_truncated"))Text("Process output abbreviated.",fontSize=12.sp)}
 }
}
</ui:copy>
## Read full captured text with scalar-indexed pages

A preview is not the complete reading. The source endpoint revalidates the captured graph and returns at most 4000 Unicode scalars with exact node/source/path/fragment identity and a page hash. Reject unpaired UTF-16 surrogates, wrong units, changed totals and invalid next offsets before replacing the retained page. Source-origin, deleted-baseline and missing/ambiguous fragment states remain explicit. A slow, closed, stale or failed page request leaves prior text readable and never fetches newer disk bytes by the same spelling. Full reading remains independent from whether construction can authorize verification.

<ui:copy id="behavior-source-page-android">
data class BehaviorSourcePage(val workspace:String,val fingerprint:String,val node:String,val sourceHash:String,val path:String?,val fragment:String?,val origin:String,val state:String,val target:String,val offset:Int,val total:Int,val next:Int?,val text:String,val notice:String)
fun behaviorSourcePage(raw:JSONObject,value:BehaviorSnapshot,nodeID:String,node:JSONObject,requested:Int?,previous:BehaviorSourcePage?=null):BehaviorSourcePage{
 fun nullable(value:JSONObject,key:String):String?=if(value.isNull(key))null else value.getString(key)
 fun scalarCount(text:String):Int{check(text.length<=8000){"Captured page exceeds 4000 Unicode scalars."};var index=0;var count=0;while(index<text.length){val char=text[index];if(char.isHighSurrogate()){check(index+1<text.length&&text[index+1].isLowSurrogate()){"Unpaired captured source surrogate."};index+=2}else{check(!char.isLowSurrogate()){"Unpaired captured source surrogate."};index++};count++};check(count<=4000){"Captured page exceeds 4000 Unicode scalars."};return count}
 val hash=node.getString("hash");val path=nullable(node,"source_path");val fragment=nullable(node,"fragment_id")
 check(hash.matches(Regex("[a-f0-9]{64}"))&&raw.has("next")&&raw.get("version")==1&&raw.get("behavior_protocol")==1&&raw.getString("workspace_id")==value.workspace&&raw.getString("reading_fingerprint")==value.fingerprint&&raw.getString("node_id")==nodeID&&raw.getString("source_hash")==hash&&raw.has("source_path")&&raw.has("fragment_id")&&nullable(raw,"source_path")==path&&nullable(raw,"fragment_id")==fragment&&raw.getString("units")=="unicode-scalars"){"Foreign or unsupported captured source page. Prior readable text retained."}
 val offset=raw.get("offset") as? Int ?: error("Malformed scalar offset.");val total=raw.get("total") as? Int ?: error("Malformed scalar total.");val next=if(raw.isNull("next"))null else raw.get("next") as? Int ?: error("Malformed next scalar offset.");val text=raw.getString("text");val count=scalarCount(text);val end=offset.toLong()+count
 check(offset>=0&&total>=0&&offset<=total&&(requested==null||offset==requested)&&end<=total&&if(end==total.toLong())next==null else next!=null&&next.toLong()==end&&next>offset){"Captured source page bounds changed. Prior readable text retained."}
 val pageHash=raw.getString("page_hash");check(pageHash.matches(Regex("[a-f0-9]{64}"))&&debugHash(text)==pageHash){"Captured page bytes do not match their hash."}
 val origin=raw.getString("origin");val state=raw.getString("state");val target=raw.getString("target_status")
 check((path==null)==(origin=="retained-summary")&&(node.optString("status")=="saved-baseline-absent-from-draft")== (origin=="saved-baseline")){"Captured source origin contradicts its observed node."}
 check(origin in listOf("constructed","captured-draft","saved-source","saved-baseline","retained-summary")&&state==(if(origin=="saved-baseline")"historical-deleted-baseline" else "captured")&&if(fragment==null)target=="whole-source" else target in listOf("observed-fragment-context","missing-declared-fragment","ambiguous-declared-fragment")){"Captured source origin or fragment status is unsupported."}
 if(previous!=null&&previous.fingerprint==value.fingerprint&&previous.node==nodeID&&previous.sourceHash==hash)check(previous.total==total&&previous.path==path&&previous.fragment==fragment&&previous.origin==origin&&previous.state==state&&previous.target==target){"Captured source paging changed its immutable reading."}
 return BehaviorSourcePage(value.workspace,value.fingerprint,nodeID,hash,path,fragment,origin,state,target,offset,total,next,text,raw.getString("notice"))
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/BehaviorReading.kt"><ui:paste select="#behavior-readings-android"/><ui:paste select="#behavior-source-page-android"/></ui:file>

## Keep source authority and process observations independently testable

Focused model witnesses admit exact namespace/context digests, preserve immutable previews, retire delayed/closed/failed reads, refuse stale editor execution, consume uncertain requests, require an actual selected profile for unsupported source providers, and correlate refreshes to the original request/check/profile. A known terminal result can run again only after an explicit fresh reading. Native rendering, physical input and accessibility are not established by these witnesses.

<ui:copy id="behavior-readings-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import org.json.JSONArray
import kotlinx.coroutines.*
import kotlinx.coroutines.channels.Channel
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class BehaviorReadingTest{
 private val engine=Engine("http://fixture.invalid")
 private fun context()=JSONObject().put("version",1).put("entry","project.md").put("focus",JSONObject.NULL).put("sources",JSONObject().put("project.md","source")).put("missing_sources",JSONArray())
 private fun reading(available:Boolean=true)=JSONObject().put("version",1).put("behavior_protocol",1).put("namespace","/fixture").put("engine_workspace_id","e".repeat(32)).put("workspace_id","d".repeat(64)).put("reading_fingerprint","a".repeat(64)).put("context",context()).put("verification_available",true).put("evidence_limit","process outcomes are not test verdicts").put("evidence",JSONObject().put("status","not-observed").put("records",JSONArray())).put("graph",JSONObject().put("nodes",JSONObject()).put("edges",JSONArray()).put("notices",JSONArray()).put("truncated",false).put("checks",JSONArray().put(JSONObject().put("id","selected").put("label","Selected check").put("path","test.py").put("available",available).put("reason","declared relationship"))))
 private fun receipt(body:JSONObject,status:String="process-finished")=JSONObject(body.toString()).put("label","Selected check").put("status",status).put("outcome","process-only").put("notice","Not a case verdict").put("exit_code",0)
 @Test fun namespaceContextAndProtocolAreExactAndSnapshotIsImmutable(){val raw=reading();val missing=reading().also{it.remove("verification_available")};assertFalse(behaviorSnapshot(missing,context(),"/fixture","e".repeat(32)).verification);val value=behaviorSnapshot(raw,context(),"/fixture","e".repeat(32));raw.getJSONObject("graph").getJSONArray("checks").getJSONObject(0).put("label","mutated");assertEquals("Selected check",value.checks.single().label)
  for((key,bad) in listOf("version" to true,"behavior_protocol" to 1.5,"workspace_id" to "foreign","namespace" to "/other","engine_workspace_id" to "f".repeat(32),"reading_fingerprint" to "foreign","verification_available" to "false","verification_available" to JSONObject.NULL))assertTrue(runCatching{behaviorSnapshot(reading().put(key,bad),context(),"/fixture","e".repeat(32))}.isFailure)
  assertTrue(runCatching{behaviorSnapshot(reading(),context().put("focus","different.py"),"/fixture","e".repeat(32))}.isFailure)
 }
 @Test fun delayedClosedAndFailedNewReadingRetireExecutionButKeepReadableHistory()=runBlocking{
  val wait=CompletableDeferred<JSONObject>();var count=0;var starts=0;val model=BehaviorReadingModel{_,path,_->if(path.endsWith("reading")){count++;if(count==1)wait.await() else if(count==3)error("optional provider failed") else reading()}else{starts++;error("No start")}}
  val old=async(start=CoroutineStart.UNDISPATCHED){model.read(engine,"/fixture",context(),"e".repeat(32)){true}};model.read(engine,"/fixture",context(),"e".repeat(32)){true};val value=model.value;wait.complete(reading().put("reading_fingerprint","b".repeat(64)));old.await();assertSame(value,model.value)
  model.read(engine,"/fixture",context(),"e".repeat(32)){true};assertSame(value,model.value);model.run("selected");assertEquals(0,starts);model.close();model.run("selected");assertEquals(0,starts)
 }
 @Test fun staleEditorAndUncertainConsumedRequestNeverStartReplacement()=runBlocking{var current=true;var starts=0;val model=BehaviorReadingModel{_,path,_->if(path.endsWith("reading"))reading() else{starts++;error("lost response")}};model.read(engine,"/fixture",context(),"e".repeat(32)){current};current=false;model.run("selected");assertEquals(0,starts);current=true;model.run("selected");assertEquals(1,starts);assertEquals("request-pending",model.receipts.single().status);model.run("selected");model.close();model.read(engine,"/fixture",context(),"e".repeat(32)){true};model.run("selected");assertEquals(1,starts)}
 @Test fun explicitUnsupportedCheckNeedsActualChosenProfileAndProfileReceiptScope()=runBlocking{
  var starts=0;val model=BehaviorReadingModel{_,path,body->when{path.endsWith("reading")->reading(false);path.endsWith("profiles")->JSONObject().put("fingerprint","c".repeat(64)).put("profiles",JSONArray().put(JSONObject().put("id","manual").put("name","Manual check")));else->{starts++;receipt(body!!)}}};model.read(engine,"/fixture",context(),"e".repeat(32)){true};model.run("selected");assertEquals(0,starts);model.run("selected",BehaviorProfile("invented","Wrong","c".repeat(64)));assertEquals(0,starts);model.chooseProfiles();model.run("selected",model.profiles.single());assertEquals(1,starts);assertEquals("process-only",model.receipts.single().outcome);assertEquals("manual",model.receipts.single().profile?.id)
 }
 @Test fun exactRetainedReceiptRefreshDoesNotLaunchAndFreshReadingCanRepeatKnownTerminal()=runBlocking{
  var starts=0;var tamper=false;var latest:JSONObject?=null;val model=BehaviorReadingModel{_,path,body->when{path.endsWith("reading")->reading();path.endsWith("verify")->{starts++;receipt(body!!).also{latest=it}};else->JSONObject(latest.toString()).apply{if(tamper)put("check_id","foreign")}}};model.read(engine,"/fixture",context(),"e".repeat(32)){true};model.run("selected");val first=model.receipts.single();model.run("selected");assertEquals(1,starts);tamper=true;model.refresh(first);assertEquals(first,model.receipts.single());assertTrue(model.notice.contains("another"));tamper=false;model.close();model.refresh(first);assertEquals(1,starts);model.read(engine,"/fixture",context(),"e".repeat(32)){true};model.run("selected");assertEquals(2,starts)
 }
 @Test fun opaquePhysicalIdentityAndRuntimeReceiptsDoNotBorrowPathOrLegacyProcessAuthority()=runBlocking{
  assertEquals("f".repeat(64),behaviorSnapshot(reading().put("workspace_id","f".repeat(64)),context(),"/fixture","e".repeat(32)).workspace)
  var tamper=false;var latest:JSONObject?=null;val model=BehaviorReadingModel{_,path,body->when{path.endsWith("reading")->reading().put("receipts",JSONObject().put("records",JSONArray().put(JSONObject().put("version",1).put("request_id","1".repeat(32)).put("workspace_id","f".repeat(64)).put("reading_fingerprint","a".repeat(64)).put("check_id","legacy").put("profile",JSONObject.NULL).put("status","process-finished").put("outcome","process-only").put("origin","unknown").put("terminal",JSONObject().put("id","foreign")))));path.endsWith("verify")->{assertEquals("e".repeat(32),body!!.getString("engine_workspace_id"));receipt(body).also{latest=it}};else->JSONObject(latest.toString()).apply{if(tamper)put("engine_workspace_id","f".repeat(32))}}}
  model.read(engine,"/fixture",context(),"e".repeat(32)){true};assertEquals("unknown",model.receipts.single().origin);assertNull(model.receipts.single().runtime);model.run("selected");val confirmed=model.receipts.first();tamper=true;model.refresh(confirmed);assertEquals(confirmed,model.receipts.first());assertTrue(model.notice.contains("another"))
 }
 @Test(timeout=120000) fun liveNativeConnectedReadingSelectedVerificationAndRetainedProcesses()=runBlocking{
  val origin=System.getenv("HICKORY_NATIVE_BEHAVIOR_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_NATIVE_BEHAVIOR_WORKSPACE") ?: error("Use the maintained owned behavior fixture"));check(root.resolve(".hickory-native-behavior-fixture").readText()=="owned-native-behavior-v1\n"){"Refuse a non-owned behavior fixture"}
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(background)
  try{
  val engine=Engine(origin!!);w.engine=engine;val (_,tree)=engine.files();assertEquals(root.toPath().toRealPath().toString(),engine.sourceRootAddress);w.rootFiles=tree
  fun find(nodes:List<FileNode>):FileNode? {for(node in nodes){if(node.path=="source.md")return node;find(node.children)?.let{return it}};return null}
  val b=w.loadBuffer(find(tree) ?: error("Construction source missing"));w.show(b.file.path);val source=b.text;val state=b.state;val selection=state.selection;val scroll=b.sourceScroll.value;w.showConstruction=true;val model=w.behaviorReading;model.read(w,"project.md","a.py");assertNotNull(model.notice,model.value)
  val connected=model.value!!;assertEquals(engine.sourceRootAddress,connected.namespace);assertEquals(engine.sourceWorkspaceID,connected.engineWorkspace);assertTrue(connected.workspace.matches(Regex("[a-f0-9]{64}")));val nodes=JSONObject(connected.raw).getJSONObject("graph").getJSONObject("nodes");val contractID=nodes.keys().asSequence().single{nodes.getJSONObject(it).getString("kind")=="contract"};model.readSource(contractID);assertNotNull(model.notice,model.sourcePage);assertEquals(root.resolve("behavior.md").readText(),model.sourcePage!!.text);assertEquals("whole-source",model.sourcePage!!.target)
  val explanationID=nodes.keys().asSequence().single{nodes.getJSONObject(it).getString("kind")=="explanation"};model.readSource(explanationID);assertEquals("source.md",model.sourcePage!!.path);assertEquals("answer",model.sourcePage!!.fragment);assertEquals("observed-fragment-context",model.sourcePage!!.target);assertTrue(model.sourcePage!!.text.contains("Later explanation for the declared answer"))
  model.readSource(explanationID,0);val full=StringBuilder();while(true){val page=model.sourcePage!!;full.append(page.text);val next=page.next ?: break;model.continueSource(page,next)};assertEquals(source,full.toString())
  val value=model.value!!;assertTrue(value.raw.contains("Answer behavior"));assertTrue(value.raw.contains("needs-explanation"));assertFalse(value.checks.any{it.label.contains("unrelated")});val selected=value.checks.single{it.label.contains("test_selected")};assertTrue(selected.available);model.run(selected.id);assertEquals(1,model.receipts.size)
  suspend fun finish(id:String):BehaviorReceipt {withTimeout(30000){while(true){val receipt=model.receipts.first{it.id==id};if(receipt.status=="process-finished")return@withTimeout;check(receipt.status in listOf("process-running","admitting","request-pending")){model.notice+receipt.raw};model.refresh(receipt);delay(50)}};return model.receipts.first{it.id==id}}
  val first=finish(model.receipts.first().id);assertEquals("process-only",first.outcome);assertEquals(0,JSONObject(first.raw).getInt("exit_code"));assertFalse(root.resolve("sibling-ran.txt").exists())
  val unsupported=value.checks.single{it.label.contains("Explicit unsupported check")};assertFalse(unsupported.available);model.run(unsupported.id);assertEquals(1,model.receipts.size);model.chooseProfiles();val profile=model.profiles.single{it.name=="Explicit manual check"};model.run(unsupported.id,profile);val second=finish(model.receipts.first().id);assertEquals("process-only",second.outcome);assertTrue(JSONObject(second.raw).getString("relationship").contains("user-declared"));assertEquals("ran",root.resolve("manual-ran.txt").readText())
  assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(source.length,source.length,"\n")),source));assertTrue(b.snippets.undoSource());assertEquals(source,b.text);model.run(selected.id);assertEquals(2,model.receipts.size);assertTrue(model.notice.contains("changed"));model.close();model.refresh(first);assertEquals(2,model.receipts.size);assertSame(state,b.state);assertSame(b,w.currentBuffer);assertEquals(selection,state.selection);assertEquals(scroll,b.sourceScroll.value);assertEquals(source,root.resolve("source.md").readText());assertFalse(root.resolve("sibling-ran.txt").exists())
  }finally{w.buffers.values.forEach{it.language?.close()};background.cancel()}
 }
}
</ui:copy>
The original five graph-controller methods passed in the integrated sixteen-method Android selection, with zero skips. The full-source paging producer and two additional scalar/lifetime cases are newly authored and await the next canonical compile and execution. The owned native graph fixture is implemented but has not yet run against the final source-page/manual-profile engine. Rendered Compose layout, pointer/IME and accessibility remain unexecuted.

## Retire paged source replies independently of process receipts

Scalar fixtures cross a supplementary Unicode character, reject wrong identity/hash/path/fragment/units/ranges and malformed surrogates, and preserve explicit deleted-baseline and missing-fragment labels. Held, closed and foreign page replies cannot replace admitted text. A construction that cannot authorize verification can still supply a hash-bound full reading. These witnesses do not mutate editor state or infer a collector verdict.

<ui:copy id="behavior-source-page-android-tests">
class BehaviorSourcePageTest {
 private val engine=Engine("http://fixture.invalid")
 private val source="😀"+"x".repeat(5000)
 private fun context()=JSONObject().put("version",1).put("entry","project.md").put("sources",JSONObject()).put("focus",JSONObject.NULL).put("missing_sources",JSONArray())
 private fun node()=JSONObject().put("id","source").put("kind","source").put("path","source.md").put("source_path","source.md").put("fragment_id",JSONObject.NULL).put("label","Source").put("hash",debugHash(source)).put("preview","abbreviated").put("status","captured")
 private fun reading()=JSONObject().put("version",1).put("behavior_protocol",1).put("namespace","/fixture").put("engine_workspace_id","e".repeat(32)).put("workspace_id","d".repeat(64)).put("reading_fingerprint","a".repeat(64)).put("context",context()).put("verification_available",false).put("graph",JSONObject().put("nodes",JSONObject().put("source",node())).put("edges",JSONArray()).put("checks",JSONArray()))
 private fun page(offset:Int=0):JSONObject{val total=source.codePointCount(0,source.length);val end=(offset+4000).coerceAtMost(total);val text=source.substring(source.offsetByCodePoints(0,offset),source.offsetByCodePoints(0,end));return JSONObject().put("version",1).put("behavior_protocol",1).put("namespace","/fixture").put("engine_workspace_id","e".repeat(32)).put("workspace_id","d".repeat(64)).put("reading_fingerprint","a".repeat(64)).put("node_id","source").put("source_hash",debugHash(source)).put("page_hash",debugHash(text)).put("source_path","source.md").put("fragment_id",JSONObject.NULL).put("origin","captured-draft").put("state","captured").put("target_status","whole-source").put("units","unicode-scalars").put("offset",offset).put("total",total).put("next",if(end<total)end else JSONObject.NULL).put("text",text).put("notice","Captured source only; current editor remains unchanged.")}
 @Test fun sourcePagesUseExactHashesEchoesScalarBoundsAndExplicitOrigins(){val value=behaviorSnapshot(reading(),context(),"/fixture","e".repeat(32));val first=behaviorSourcePage(page(),value,"source",node(),null);assertEquals(4000,first.next);assertTrue(first.text.startsWith("😀"));assertEquals(4001,first.text.length);val last=behaviorSourcePage(page(4000),value,"source",node(),4000,first);assertNull(last.next)
  for((key,bad) in listOf("version" to true,"workspace_id" to "foreign","reading_fingerprint" to "b".repeat(64),"node_id" to "other","source_hash" to "b".repeat(64),"page_hash" to "b".repeat(64),"source_path" to "other.md","fragment_id" to "other","units" to "utf16","offset" to 0.5,"total" to 2,"next" to 1,"target_status" to "missing-declared-fragment","state" to "historical-deleted-baseline"))assertTrue(key,runCatching{behaviorSourcePage(page().put(key,bad),value,"source",node(),null)}.isFailure)
  val unpaired=page().put("text","\uD800").put("page_hash",debugHash("\uD800"));assertTrue(runCatching{behaviorSourcePage(unpaired,value,"source",node(),null)}.isFailure)
  val deleted=behaviorSourcePage(page().put("origin","saved-baseline").put("state","historical-deleted-baseline"),value,"source",node().put("status","saved-baseline-absent-from-draft"),null);assertEquals("historical-deleted-baseline",deleted.state)
  val missingNode=node().put("fragment_id","missing");val missing=behaviorSourcePage(page().put("fragment_id","missing").put("target_status","missing-declared-fragment"),value,"source",missingNode,null);assertEquals("missing-declared-fragment",missing.target)
 }
 @Test fun fullReadingIsIndependentOfVerificationAndRetainsPagesOnStaleClosedOrForeignReplies()=runBlocking{
  var current=true;var held:CompletableDeferred<JSONObject>?=null;var foreign=false;val entered=Channel<Unit>(Channel.UNLIMITED)
  val model=BehaviorReadingModel{_,path,body->if(path.endsWith("reading"))reading() else{check(path.endsWith("source"));assertEquals(debugHash(source),body!!.getString("source_hash"));assertEquals(behaviorCanonical(context()),behaviorCanonical(body.getJSONObject("context")));entered.send(Unit);held?.await() ?: page(body.optInt("offset",0)).apply{if(foreign)put("page_hash","b".repeat(64))}}}
  model.read(engine,"/fixture",context(),"e".repeat(32)){current};assertFalse(model.value!!.verification);model.readSource("source");entered.receive();val first=model.sourcePage!!;assertEquals(4000,first.next)
  val delayed=CompletableDeferred<JSONObject>();held=delayed;val stale=async(start=CoroutineStart.UNDISPATCHED){model.readSource("source",4000)};entered.receive();current=false;delayed.complete(page(4000));stale.await();assertSame(first,model.sourcePage)
  current=true;val closedReply=CompletableDeferred<JSONObject>();held=closedReply;val closed=async(start=CoroutineStart.UNDISPATCHED){model.readSource("source",4000)};entered.receive();model.close();closedReply.complete(page(4000));closed.await();assertSame(first,model.sourcePage);assertFalse(model.sourceLoading)
  held=null;foreign=true;model.read(engine,"/fixture",context(),"e".repeat(32)){true};assertFalse(model.currentSourcePage);model.continueSource(first,4000);assertSame(first,model.sourcePage);model.readSource("source",4000);entered.receive();assertSame(first,model.sourcePage);assertTrue(model.notice.contains("hash"))
  foreign=false;model.readSource("source",4000);entered.receive();assertEquals(4000,model.sourcePage!!.offset);assertNull(model.sourcePage!!.next)
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/BehaviorReadingTest.kt"><ui:paste select="#behavior-readings-android-tests"/><ui:paste select="#behavior-source-page-android-tests"/></ui:file>
</ui:doc>
