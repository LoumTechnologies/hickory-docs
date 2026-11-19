@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
package com.hickorydocs.simple
import androidx.compose.runtime.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.*
import org.json.JSONObject
import org.json.JSONArray

private fun agentHex(value:String,size:Int)=value.matches(Regex("[a-f0-9]{$size}"))
private fun agentString(raw:JSONObject,key:String)=raw.get(key) as? String ?: error("Malformed proposal $key.")
data class AgentProposalEntry(val token:String,val fingerprint:String,val title:String?=null,val historical:Boolean=false){override fun equals(other:Any?)=other is AgentProposalEntry&&token==other.token&&fingerprint==other.fingerprint;override fun hashCode()=31*token.hashCode()+fingerprint.hashCode()}
data class AgentProposalSection(val title:String,val text:String,val starts:List<Int>,val scalars:Int)
private fun agentSection(title:String,text:String):AgentProposalSection {
 check(text.length<=64*1024*1024){"Proposal section exceeds its display budget."};val starts=mutableListOf(0);var index=0;var scalars=0;var bytes=0L
 while(index<text.length){if(scalars>0&&scalars%4000==0)starts.add(index);val c=text[index];val charge=when{c.isHighSurrogate()->{check(index+1<text.length&&text[index+1].isLowSurrogate()){"Malformed Unicode in proposal section."};index+=2;4};c.isLowSurrogate()->error("Malformed Unicode in proposal section.");else->{index++;when{c.code<128->1;c.code<2048->2;else->3}}};bytes+=charge;check(bytes<=64L*1024*1024){"Proposal section exceeds its display budget."};scalars++}
 return AgentProposalSection(title,text,java.util.Collections.unmodifiableList(starts),scalars)
}
data class SealedAgentProposal(val token:String,val fingerprint:String,val workspace:String,val namespace:String,val material:String,val title:String,val origin:String,val explanation:String,val patches:List<ResourceEditorPatch>,val proposal:String,val facts:String,val saved:Map<String,ResourceSaved>,val sections:List<AgentProposalSection>,val reported:Boolean,val historical:Boolean=false,val retentionNotice:String?=null)
private fun agentMaterialBudget(text:String)=nativeReviewMaterialBudget(text,32*1024*1024,"Proposal")

suspend fun sealedAgentProposal(raw:JSONObject,expected:AgentProposalEntry,workspace:String,namespace:String):SealedAgentProposal {
 val historical=if(raw.has("historical"))raw.get("historical") as? Boolean ?: error("Unsupported proposal retention state") else false
 val readingScope=if(historical)raw.getJSONObject("reading_scope") else raw
 check(agentString(readingScope,"workspace")==workspace&&agentString(readingScope,"namespace")==namespace){"Historical reading or current proposal belongs to a foreign workspace."}
 val retention=raw.optJSONObject("durability");val retentionNotice=if(retention?.opt("retained")==false)(retention.opt("notice") as? String)?.take(4096) ?: "This live review was not retained for a later engine." else null
 val version=raw.get("version");val protocol=raw.get("proposal_protocol");val token=agentString(raw,"token");val fingerprint=agentString(raw,"fingerprint");val material=agentString(raw,"material");val root=agentString(raw,"workspace");val address=agentString(raw,"namespace");val effects=raw.get("source_effects");val checks=raw.get("checks_executed");val reported=!raw.isNull("host_application")
 return withContext(Dispatchers.Default){
  check(version==1&&protocol==1&&token==expected.token&&fingerprint==expected.fingerprint&&agentHex(token,32)&&agentHex(fingerprint,64)&&root.isNotEmpty()&&address.isNotEmpty()&&address.length<=16384&&effects==false&&checks==false){"Foreign or unsupported complete proposal. Nothing was applied."}
  agentMaterialBudget(material);check(debugHash(material)==fingerprint){"Complete proposal seal failed. No explanation, patch or claimed evidence was admitted."}
  val value=JSONObject(material);val intention=value.getJSONObject("intention");val inputs=value.getJSONObject("inputs");val capture=intention.getJSONObject("capture");val plan=value.getJSONObject("plan");val facts=value.getJSONObject("consequences");val title=agentString(intention,"title");val explanation=agentString(intention,"explanation");val origin=agentString(value,"origin_authority")
  check(value.get("version")==1&&intention.get("version")==1&&agentHex(agentString(intention,"request_id"),32)&&agentString(value,"workspace")==root&&agentString(value,"namespace")==address&&title.isNotBlank()&&explanation.isNotBlank()&&title.length<=512&&explanation.length<=65536&&plan.get("version")==1&&inputs.getString("context")==capture.getString("context")&&plan.getString("context")==inputs.getString("context")&&facts.getString("context")==plan.getString("context")&&inputs.getString("workspace")==root&&facts.getString("workspace")==root&&facts.getJSONArray("protected_targets").length()==0){"Complete proposal material is malformed or targets generated output."}
  val claims=intention.getJSONArray("claims");val requested=intention.getJSONArray("check_ids");check(claims.length()<=128&&requested.length()<=256&&(0 until claims.length()).all{claims.get(it) is String&&claims.getString(it).length<=8192}&&(0 until requested.length()).all{requested.get(it) is String}){"Proposal claims or requested checks are malformed."}
  val patches=java.util.Collections.unmodifiableList(resourcePatches(plan));check(patches.isNotEmpty()&&patches.size<=1000&&patches.map{it.path}.distinct().size==patches.size){"Proposal must contain one complete bounded source namespace."};val saved=java.util.Collections.unmodifiableMap(resourceSavedMap(facts.getJSONObject("saved_sources")));check(saved.keys==patches.map{it.path}.toSet()){"Saved consequences do not cover the complete proposal."}
  for(name in listOf("before","after")){val reading=value.getJSONObject(name);check(reading.get("version")==1&&reading.get("behavior_protocol")==1&&reading.getString("namespace")==address&&reading.getString("engine_workspace_id")==root){"Connected readings belong to a different proposal workspace."}}
  val proposal=JSONObject().put("inputs",inputs).put("capture",capture).put("plan",plan)
  val sections=java.util.Collections.unmodifiableList(listOf(agentSection("Explanation · caller/agent text, not observed evidence",explanation),agentSection("Claims · unverified statements",claims.toString(2)),agentSection("Complete ordered source/resource patches",plan.toString(2)),agentSection("Saved and draft generated consequences",facts.toString(2)),agentSection("Before connected contract/explanation/implementation/tests/evidence",value.getJSONObject("before").toString(2)),agentSection("After connected readings · no checks executed",value.getJSONObject("after").toString(2)),agentSection("Requested check identities · declarations only",requested.toString(2))))
  SealedAgentProposal(token,fingerprint,root,address,material,title,origin,explanation,patches,proposal.toString(),facts.toString(),saved,sections,reported,historical,retentionNotice)
 }
}
fun agentProposalTextOnly(value:SealedAgentProposal)=value.patches.all{it.before!=null&&it.after!=null&&it.before.identity==it.after.identity}&&value.patches.any{it.before?.text!=it.after?.text}
internal fun simpleAgentContext(w:Workspace):JSONObject {
 val ownership=behaviorRetainedContext(w,"project.md",null).getJSONObject("sources");val buffers=JSONArray();val roster=(w.buffers.values+w.retainedResourceEditors.values+w.retainedResourceTabs.values).distinct().filter{it.file.generatedBy==null}
 for(b in roster){val named=ownership.has(b.file.path)&&w.buffers[b.file.path]===b;buffers.put(JSONObject().put("id",b.debugIdentity).put("name",b.file.name).put("path",if(named)b.file.path else JSONObject.NULL).put("content",b.text).put("focused",w.currentBuffer===b).put("observations",JSONArray().put(JSONObject().put("kind","editor_revision").put("version",1).put("source_revision",b.sourceRevision.toString()))).put("kind",if(b.file.docID==null)"source" else "construction-source").put("document",b.file.docID ?: JSONObject.NULL).apply{if(named&&!b.saveGuard.isNullOrEmpty())put("save_guard",b.saveGuard)})}
 return JSONObject().put("live",false).put("buffers",buffers)
}

class AgentProposalModel(private val transport:(suspend(Engine,String,String,JSONObject?)->JSONObject)?=null){
 var showing by mutableStateOf(false);private set
 var busy by mutableStateOf(false);private set
 var notice by mutableStateOf("");private set
 var entries by mutableStateOf<List<AgentProposalEntry>>(emptyList());private set
 var value by mutableStateOf<SealedAgentProposal?>(null);private set
 var phase by mutableStateOf("Not applied");private set
 private var recordedAuthority: Pair<AgentProposalEntry, () -> Boolean>? = null
 private var ticket=0L;private var engine:Engine?=null;private var owner:Workspace?=null;private var current:()->Boolean={false};private val consumed=linkedSetOf<String>();private val reports=linkedMapOf<String,Pair<Engine,String>>();private var resourceReview:ResourceChangeReview.Review?=null;private var resourceStarted=false;private var resourceRequest:String?=null
 private suspend fun call(server:Engine,method:String,path:String,body:JSONObject?=null)=transport?.invoke(server,method,path,body) ?: server.request(method,path,body)
 fun close(){showing=false;if(!resourceStarted){ticket++;current={false};busy=false}}
 val authorityCurrent get()=current()
 val canApply get()=value?.let{!it.historical&&!it.reported&&it.token !in consumed&&current()&&!busy}==true
 suspend fun prepareRecordedPatch(w: Workspace, reading: AgentFenceReading, opening: Int) {
  if (resourceStarted || busy || !reading.current()) return
  showing = true
  val server = w.engine ?: run { notice = "Connect to the workspace to review the recorded patch."; return }
  val workspace = server.sourceWorkspaceID
  val namespace = server.sourceRootAddress
  if (workspace == null || namespace == null) { notice = "Refresh Files before reviewing the recorded patch."; return }
  val id = ++ticket
  val messageCurrent = { reading.current() && w.engine === server && server.sourceWorkspaceID == workspace && server.sourceRootAddress == namespace }
  val lease = w.sourceAcquisitionLease(external = messageCurrent)
  busy = true; current = { false }; value = null; entries = emptyList(); phase = "Not applied"; resourceReview = null
  notice = "Preparing a review of the recorded patch…"
  try {
   val entry = "project.md"
   val context = behaviorRetainedContext(w, entry, null)
   val sources = context.getJSONObject("sources")
   val roster = (w.buffers.values + w.retainedResourceEditors.values + w.retainedResourceTabs.values).distinct().filter { it.file.generatedBy == null }
   check(lease.current() && roster.none { it.startupObservation }) { "Wait for source confirmation before preparing the recorded patch." }
   data class Held(val id: String, val path: String?, val text: String, val base: String, val guard: String?, val revision: String, val writable: Boolean)
   val editors = roster.map { editor ->
    val path = editor.file.path.takeIf { sources.has(it) && w.buffers[it] === editor }
    Held(editor.debugIdentity, path, editor.text, editor.base, editor.saveGuard.takeIf { path != null }, editor.sourceRevision.toString(), !editor.readOnly)
   }
   val sourceHash = withContext(Dispatchers.Default) { debugHash(reading.source) }
   val (drafts, binding) = withContext(Dispatchers.Default) {
    val preimages = JSONObject()
    val owners = JSONArray()
    for (item in editors) {
     owners.put(JSONObject().put("id", item.id).put("path", item.path ?: JSONObject.NULL).put("source_hash", debugHash(item.text))
      .put("save_guard", item.guard ?: JSONObject.NULL).put("source_revision", item.revision))
     if (item.writable && item.path != null && !item.guard.isNullOrEmpty()) preimages.put(item.path,
      JSONObject().put("save_guard", item.guard).put("text", item.text).put("base_sha256", debugHash(item.base)).put("version", JSONObject.NULL).put("writable", true))
    }
    preimages to JSONObject().put("live", false).put("buffers", owners)
   }
   check(id == ticket && lease.current()) { "The message or its source context changed. Choose Review again." }
   fun nonce() = java.util.UUID.randomUUID().toString().replace("-", "")
   val capture = JSONObject().put("context", nonce()).put("project_entry", entry).put("edit", JSONObject().put("changes", JSONObject()))
    .put("drafts", drafts).put("ownership_sources", sources).put("unaddressed_sources", context.getJSONObject("unaddressed_sources"))
    .put("saved_guards", context.getJSONObject("saved_guards"))
   val intention = JSONObject().put("version", 1).put("request_id", nonce()).put("origin", "recorded message fence")
    .put("title", "Recorded patch on line ${opening.toLong()+1}").put("explanation", "Review authored changes and their generated consequences before applying the recorded patch.")
    .put("claims", JSONArray()).put("check_ids", JSONArray()).put("capture", capture).put("reading", context).put("editor_snapshot", binding)
   val address = JSONObject().put("side", reading.side).put("source_hash", sourceHash).put("opening_line", opening)
   val raw = call(server, "POST", reading.endpoint("fence-patch"), JSONObject().put("fence", address).put("intention", intention))
   check(id == ticket && lease.current()) { "The message or its source context changed while preparing this patch." }
   val selected = AgentProposalEntry(agentString(raw, "token"), agentString(raw, "fingerprint"), intention.getString("title"))
   val sealed = sealedAgentProposal(raw, selected, workspace, namespace)
   check(id == ticket && lease.current() && !sealed.historical && !sealed.reported &&
    sealed.origin == "recorded ${reading.side} fence, source SHA-256 $sourceHash; claims remain unverified") {
    "The proposal does not match the exact recorded fence. Choose Review again."
   }
   recordedAuthority = selected to messageCurrent
   entries = listOf(selected)
   read(w, selected)
  } catch (cancelled: CancellationException) { throw cancelled }
  catch (failure: Exception) { if (id == ticket) notice = failure.message ?: "Reopen the message and choose Review again." }
  finally { if (id == ticket) busy = false }
 }
 suspend fun open(w:Workspace){showing=true;if(resourceStarted)return;val server=w.engine ?: run{notice="Connect to inspect retained proposals.";return};val workspace=server.sourceWorkspaceID;val namespace=server.sourceRootAddress;if(workspace==null||namespace==null){notice="Refresh Files to bind the proposal workspace lifetime.";return};val id=++ticket;busy=true;current={false};value=null;entries=emptyList();owner=w;engine=server
  try{val raw=call(server,"GET","/api/agent-proposals");check(id==ticket&&w.engine===server&&server.sourceWorkspaceID==workspace&&server.sourceRootAddress==namespace){"Workspace changed while listing proposals."};check(raw.get("version")==1&&raw.get("proposal_protocol")==1&&raw.getString("workspace")==workspace&&raw.getString("namespace")==namespace){"Unsupported or foreign proposal catalogue."};val rows=raw.getJSONArray("reviews");check(rows.length()<=32){"Proposal catalogue exceeds its retained budget."};val next=(0 until rows.length()).map{val row=rows.getJSONObject(it);AgentProposalEntry(row.getString("token"),row.getString("fingerprint"),(row.opt("title") as? String)?.take(512),if(row.has("historical"))row.get("historical") as? Boolean ?: error("Unsupported historical catalogue flag") else false)};check(next.all{agentHex(it.token,32)&&agentHex(it.fingerprint,64)}&&next.map{it.token}.distinct().size==next.size){"Ambiguous retained proposal identities."};entries=next;notice="Read a retained sealed intention. Catalogue labels are not admitted as review material.";if(raw.opt("retention_notice") is String)notice+=" "+raw.getString("retention_notice").take(4096)}catch(e:CancellationException){throw e}catch(e:Exception){if(id==ticket)notice=e.message.orEmpty()}finally{if(id==ticket)busy=false}
 }
 private fun matchesHost(w:Workspace,p:SealedAgentProposal){
  val retained=behaviorRetainedContext(w,"project.md",null);val material=JSONObject(p.material);val capture=material.getJSONObject("intention").getJSONObject("capture");val named=retained.getJSONObject("sources");val expected=capture.getJSONObject("ownership_sources");val unknown=retained.getJSONObject("unaddressed_sources");val before=p.patches.associateBy{it.path};val originalUnknown=capture.optJSONObject("unaddressed_sources") ?: JSONObject();val sealedGuards=material.getJSONObject("saved_guards");val originalGuards=capture.optJSONObject("saved_guards") ?: JSONObject()
  for(path in named.keys()){val text=named.getString(path);if(expected.has(path))check(expected.getString(path)==text){"A retained source differs from the sealed ownership capture: $path"}else check(before[path]?.before?.text==text){"A retained source was not captured by this complete intention: $path"};val expectedGuard=(originalGuards.opt(path) as? String) ?: (sealedGuards.opt(path) as? String);if(expectedGuard!=null)check(w.buffers[path]?.saveGuard==expectedGuard){"A retained physical source baseline differs from the complete intention: $path"}}
  for(id in unknown.keys())check(originalUnknown.has(id)&&originalUnknown.getString(id)==unknown.getString(id)){"An unaddressed retained draft was added or changed. Request a new complete intention."}
  for(path in expected.keys())check(named.has(path)&&named.getString(path)==expected.getString(path)){"An original captured source was closed or changed. Request a new complete intention."}
  for(id in originalUnknown.keys())check(unknown.has(id)&&unknown.getString(id)==originalUnknown.getString(id)){"An original anonymous source was closed or changed. Request a new complete intention."}
  val original=material.getJSONObject("intention").optJSONObject("editor_snapshot");val managed=p.origin.startsWith("engine-managed")
  if(original==null){check(!managed){"The managed proposal has no original editor binding. Request a new intention from this host; current state cannot replace the sender snapshot."}}else{
   check(original.get("live") is Boolean){"Malformed original editor binding."};val originals=original.getJSONArray("buffers");check(originals.length()<=4096){"Original editor roster exceeds its budget."};val roster=(w.buffers.values+w.retainedResourceEditors.values+w.retainedResourceTabs.values).distinct();val ids=linkedSetOf<String>()
   for(index in 0 until originals.length()){val row=originals.getJSONObject(index);val id=row.opt("id") as? String ?: error("Original editor identity is unavailable. Refresh the complete intention.");check(id.isNotEmpty()&&ids.add(id)){"Original editor identities are ambiguous."};val b=roster.singleOrNull{it.debugIdentity==id} ?: error("An original editor was closed or replaced. Request a new complete intention.");val revision=row.opt("source_revision") as? String ?: error("Original editor revision is unknown. A current lease cannot upgrade it.");check(revision.matches(Regex("0|[1-9][0-9]*"))&&revision==b.sourceRevision.toString()&&row.getString("source_hash")==debugHash(b.text)){"Original editor revision changed before review. Request a new complete intention."};val path=if(row.isNull("path"))null else row.get("path") as? String ?: error("Malformed original source address.");if(path==null)check(unknown.has(id)){"Original anonymous source changed identity."}else check(path==b.file.path&&named.has(path)&&w.buffers[path]===b){"Original named source changed identity."};val guard=if(row.isNull("save_guard"))null else row.get("save_guard") as? String ?: error("Malformed original save guard.");check(guard==if(path==null)null else b.saveGuard){"Original physical source baseline changed before review."}
   }
   for(b in roster.filter{it.file.generatedBy==null&&it.debugIdentity !in ids})check(named.has(b.file.path)&&before[b.file.path]?.before?.text==b.text){"A retained editor was not in the original sender context. Request a new intention."}
  }
  for(patch in p.patches){val source=patch.before ?: continue;val b=w.buffers[patch.path] ?: error("Acquire every reviewed source before Apply.");check(!b.readOnly&&!b.saving&&b.text==source.text){"A reviewed source changed or became unavailable: ${patch.path}"};val draft=capture.getJSONObject("drafts").optJSONObject(patch.path);val guard=draft?.opt("save_guard") as? String ?: sealedGuards.opt(patch.path) as? String;check(!guard.isNullOrEmpty()&&guard==b.saveGuard){"This editor has a different original physical baseline: ${patch.path}. Keep its draft and prepare a new intention."};if(draft!=null)check(draft.getString("base_sha256")==debugHash(b.base)){"Reviewed saved baseline changed."}}
 }
 suspend fun read(w:Workspace,entry:AgentProposalEntry){val server=w.engine ?: return;if(entry !in entries||resourceStarted)return;val workspace=server.sourceWorkspaceID ?: return;val namespace=server.sourceRootAddress ?: return;val epoch=w.sourceNavigationEpoch;val id=++ticket;busy=true;current={false};resourceReview=null;value=null;phase="Not applied"
  try{val raw=call(server,"GET","/api/agent-proposals/${entry.token}");val sealed=sealedAgentProposal(raw,entry,workspace,namespace);check(id==ticket&&w.engine===server&&w.sourceNavigationEpoch==epoch){"Proposal reading belongs to an obsolete host."};value=sealed;owner=w;engine=server
   if(sealed.historical){phase="Historical reading · Apply unavailable";notice="This sealed intention has no current live engine admission. Source and host reports are historical readings; request a fresh proposal to change current drafts.";return}
   val recorded = sealed.origin.startsWith("recorded ")
   val readingCurrent = recordedAuthority?.takeIf { it.first == entry }?.second
   if (recorded && readingCurrent?.invoke() != true) { notice = "The recorded patch can be read. Choose Review in its current message before applying it."; return }
   val alive={id==ticket&&value===sealed&&w.engine===server&&w.sourceNavigationEpoch==epoch&&server.sourceWorkspaceID==workspace&&server.sourceRootAddress==namespace&&(!recorded||readingCurrent?.invoke()==true)};val capture=JSONObject(sealed.proposal).getJSONObject("capture");val acquired=w.resourceChanges.acquire(w,sealed.patches.map{it.path},capture.getJSONObject("edit"),alive);check(alive()&&acquired.third()){ "Editor namespace changed while acquiring the proposal." };matchesHost(w,sealed);current=acquired.third;phase=if(sealed.reported)"Host already reported application; no repeat Apply" else if(sealed.token in consumed)"Apply attempt retained; inspect its result" else "Reviewed only";notice="Whole material seal verified. Source patches, generated consequences and check observations are one intention; claims remain unverified.";sealed.retentionNotice?.let{notice+=" "+it}
  }catch(e:CancellationException){throw e}catch(e:Exception){if(id==ticket)notice=e.message.orEmpty()}finally{if(id==ticket)busy=false}
 }
 private suspend fun validate(w:Workspace,p:SealedAgentProposal){val server=engine ?: error("Proposal connection unavailable.");check(value===p&&w.engine===server&&current()){"Complete proposal or host editor lease changed."};val raw=call(server,"POST","/api/agent-proposals/validate",JSONObject().put("version",1).put("token",p.token).put("fingerprint",p.fingerprint));check(raw.get("version")==1&&raw.get("proposal_protocol")==1&&raw.get("admitted")==true&&raw.get("source_effects")==false&&raw.get("checks_executed")==false&&raw.getString("token")==p.token&&raw.getString("fingerprint")==p.fingerprint&&raw.getString("workspace")==p.workspace&&raw.getString("namespace")==p.namespace&&value===p&&w.engine===server&&current()){ "Complete proposal validation or editor lease changed. Nothing was applied." }}
 private fun report(p:SealedAgentProposal,server:Engine){val hashes=JSONObject();p.patches.forEach{hashes.put(it.path,it.after?.text?.let(::debugHash) ?: "missing")};val body=JSONObject().put("version",1).put("token",p.token).put("fingerprint",p.fingerprint).put("workspace",p.workspace).put("namespace",p.namespace).put("request_id",java.util.UUID.randomUUID().toString().replace("-","")).put("host","android-simple").put("after_hashes",hashes);reports[p.token]=server to body.toString()}
 suspend fun sendReport(){val p=value ?: return;val (server,body)=reports[p.token] ?: return;try{val expected=JSONObject(body);val raw=call(server,"POST","/api/agent-proposals/report",expected);check(raw.get("version")==1&&raw.get("proposal_protocol")==1&&raw.getString("token")==p.token&&raw.getString("fingerprint")==p.fingerprint&&raw.getString("workspace")==p.workspace&&raw.getString("namespace")==p.namespace&&raw.getString("request_id")==expected.getString("request_id")&&raw.getString("host")=="android-simple"&&behaviorCanonical(raw.getJSONObject("after_hashes"))==behaviorCanonical(expected.getJSONObject("after_hashes"))&&raw.get("saved")==false&&raw.get("checks_executed")==false&&agentHex(raw.getString("report_fingerprint"),64)){"Host acknowledgment does not match the actual complete draft transition."};reports.remove(p.token);if(value===p)notice="Host draft application acknowledged. This is a host report, not an independently observed save or test verdict."}catch(e:CancellationException){throw e}catch(e:Exception){if(value===p)notice=e.message.orEmpty()+" Drafts stay applied; retry only this exact report, never Apply again."}}
 suspend fun applyDraft(w:Workspace){val p=value ?: return;if(!canApply||!agentProposalTextOnly(p)||stagedSave)return;busy=true
  try{validate(w,p);check(w.changes.mode==null){"Finish the existing workspace change preview first."};val changes=JSONObject();val expected=linkedMapOf<String,String>();for(patch in p.patches){val before=patch.before!!;expected[patch.path]=before.text;changes.put(languageURI(patch.path),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",languagePosition(before.text,before.text.length))).put("newText",patch.after!!.text)))}
   check(current());consumed.add(p.token);w.changes.offer(p.title,JSONObject().put("changes",changes),expected,w);w.changes.apply(w);check(w.changes.plan==null){w.changes.notice};w.changes.close(w);phase="Applied as one unsaved workspace draft batch";current={false};report(p,engine!!);sendReport()
  }catch(e:CancellationException){throw e}catch(e:Exception){notice=e.message.orEmpty()}finally{busy=false}
 }
 suspend fun prepareSave(w:Workspace){val p=value ?: return;if(!canApply)return;busy=true
  try{validate(w,p);check(w.resourceChanges.review==null&&!w.resourceChanges.hasPending){"Finish the current resource review first."};val held=current;val proposal=JSONObject(p.proposal);resourceReview=w.resourceChanges.acceptSealed(w,proposal,JSONObject(p.facts),proposal.getJSONObject("plan"),held,preflight={validate(w,p);check(held());consumed.add(p.token);resourceStarted=true},published={record,namespace->w.changes.recordResource(record,namespace);resourceStarted=false;phase="Saved namespace confirmed · recovery $record";current={false};report(p,engine!!)});notice="Complete saved/draft consequences above are staged. Explicit Save uses the shared guarded publisher; no checks run."}catch(e:CancellationException){throw e}catch(e:Exception){notice=e.message.orEmpty()}finally{busy=false}
 }
 suspend fun save(w:Workspace){val review=resourceReview ?: return;val p=value ?: return;if(!canSave)return;busy=true;try{w.resourceChanges.apply(w,review);notice=w.resourceChanges.notice;if(w.resourceChanges.hasPending&&w.engine===engine&&w.resourceChanges.pendingWorkspace==p.workspace){resourceRequest=w.resourceChanges.pendingRecovery}else{resourceStarted=false;resourceRequest=null;if(reports.containsKey(p.token))sendReport()}}finally{if(resourceStarted&&w.engine===engine&&w.resourceChanges.pendingWorkspace==p.workspace)resourceRequest=w.resourceChanges.pendingRecovery;busy=false}}
 suspend fun publicationStatus(w:Workspace){val p=value ?: return;val request=resourceRequest ?: run{notice="Original publication identity is unavailable. Inspect Source recovery; do not retry Save.";return};if(!resourceStarted)return;if(w.engine!==engine||w.resourceChanges.pendingRecovery!=request||w.resourceChanges.pendingWorkspace!=p.workspace){notice="Original publication status belongs to another workspace or request. Inspect Source recovery there.";return};busy=true;try{w.resourceChanges.status(w);notice=w.resourceChanges.notice;if(!w.resourceChanges.hasPending){resourceStarted=false;resourceRequest=null;sendReport()}}finally{busy=false}}
 val hasReport get()=value?.let{!it.historical&&reports.containsKey(it.token)}==true
 val stagedSave get()=resourceReview!=null
 val canSave get()= !busy&&current()&&resourceReview!=null&&owner?.resourceChanges?.review===resourceReview
 val publicationPending get()=resourceStarted
}
@Composable fun AgentProposalView(w:Workspace){
 val m=w.agentProposals;val scope=rememberCoroutineScope();var section by remember{mutableIntStateOf(0)};var page by remember{mutableIntStateOf(0)}
 Column(Modifier.padding(16.dp).widthIn(max=720.dp).heightIn(max=640.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(7.dp)){
  Row{Text("Agent proposal review",style=MaterialTheme.typography.titleMedium);Spacer(Modifier.weight(1f));TextButton(onClick={m.close();w.currentBuffer?.focusSource?.invoke()}){Text("Close")}}
  Text("Retained intentions only. Agent statements and requested checks are not observed verdicts.",fontSize=12.sp);Text(m.notice,fontSize=12.sp);Text(m.phase,fontSize=12.sp)
  TextButton(enabled=!m.busy&&!m.publicationPending,onClick={scope.launch{m.open(w)}}){Text("Refresh retained intentions")}
  m.entries.forEach{entry->TextButton(enabled=!m.busy&&!m.publicationPending,onClick={section=0;page=0;scope.launch{m.read(w,entry)}}){Text("Read "+(entry.title ?: "sealed proposal ${entry.token.take(12)}")+if(entry.historical)" · historical" else "")}}
  m.value?.let{value->
   Text(value.title,style=MaterialTheme.typography.titleSmall);Text(if(m.authorityCurrent)"Current retained editor lease" else "Historical editor/source lease · refresh the complete intention before another application",fontSize=11.sp);Text("Origin authority: ${value.origin}",fontSize=12.sp);Text("Complete material SHA256 ${value.fingerprint}",fontSize=11.sp)
   value.sections.forEachIndexed{index,item->TextButton(onClick={section=index;page=0}){Text(item.title)}}
   val selected=value.sections.getOrNull(section) ?: value.sections.first();val pages=selected.starts.size;val index=page.coerceIn(0,pages-1);val start=selected.starts[index];val end=selected.starts.getOrNull(index+1) ?: selected.text.length
   Text(selected.title,style=MaterialTheme.typography.titleSmall);androidx.compose.foundation.text.selection.SelectionContainer{Text(selected.text.substring(start,end),fontFamily=androidx.compose.ui.text.font.FontFamily.Monospace,fontSize=12.sp)}
   Row{TextButton(enabled=index>0,onClick={page=index-1}){Text("Previous material page")};Text("${index+1}/$pages",fontSize=11.sp);TextButton(enabled=index+1<pages,onClick={page=index+1}){Text("Next material page")}}
   if(agentProposalTextOnly(value))TextButton(enabled=m.canApply&&!m.stagedSave,onClick={scope.launch{m.applyDraft(w)}}){Text("Apply complete draft batch")}
   TextButton(enabled=m.canApply&&!m.stagedSave,onClick={scope.launch{m.prepareSave(w)}}){Text("Prepare guarded namespace Save")}
   if(m.stagedSave&&!m.publicationPending)TextButton(enabled=m.canSave,onClick={scope.launch{m.save(w)}}){Text("Save reviewed namespace")}
   if(m.publicationPending)TextButton(enabled=!m.busy,onClick={scope.launch{m.publicationStatus(w)}}){Text("Inspect original publication status")}
   if(m.hasReport)TextButton(enabled=!m.busy,onClick={scope.launch{m.sendReport()}}){Text("Retry exact host report · no Apply")}
  }
 }
}
