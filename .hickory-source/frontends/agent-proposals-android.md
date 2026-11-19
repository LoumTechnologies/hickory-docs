<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Review one sealed Android agent intention before changing source

For maintainers following a retained agent proposal from its explanation into one guarded host transition.

Catalogue rows supply opaque lookup identities only. Do not render their unverified titles as review content. A complete response must match the requested token, fingerprint, connected workspace and canonical namespace. Bound UTF8, nesting and container count before parsing the nested material; verify SHA256 of its exact bytes before admitting any explanation, patch, generated consequence or check reading. Immutable derived collections prevent a renderer from mutating an admitted plan. Agent claims remain text; preparation executes no checks and writes no source.

Historical material preserves its original scope and seal. Only its reading scope matches the current engine; it never acquires current source editors or enters Apply. A failed private archive remains visible for otherwise usable live review.

<ui:copy id="agent-proposals-android-seal">
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

</ui:copy>

## Keep the entire host lease through admission and application

Acquire missing targets through the existing resource capture service without activating another editor. Every retained named draft must match the sealed ownership capture or an exact reviewed target, including its originally loaded guard; every anonymous draft retains its opaque identity and text. A new retained source, changed baseline, text/selection/revision, namespace, composition or provider retires admission after an await. Authenticated engine-managed intentions additionally bind their original editor IDs, paths, source hashes, original guards and canonical sender revision strings in the sealed editor_snapshot. Missing or unknown original revisions disable managed application; an equal-text edit/Undo or an independently replaced editor before opening the review cannot be upgraded by a fresh lease. Caller-provided intentions remain explicitly labelled and use the complete declared text/maps/guards plus the current host lease. Static agent prompt context is explicitly live:false; it never pretends to have a refresh bridge or learns replacement guards from a later read.

Text-only intentions validate the complete seal immediately before one existing coordinated workspace draft batch, retain its inverse in workspace Undo, and consume application before the local effect. A failed host acknowledgment retains one exact report body; retrying it cannot repeat Apply. Namespace Save uses the existing resource review/publisher and v2 adoption proof, with another asynchronous whole-seal preflight before the one-shot write. Unknown publication keeps its original status identity. The engine host report remains a host claim; confirmed saved state comes only from the independent guarded publication receipt.

Reviewing a recorded patch first pins its turn, side and message hash along
with the current guarded editor preimages and revisions. Send an empty edit to
the recorded-patch endpoint so the engine derives the complete target set from
the stored fence. Select the returned seal directly in the existing review.
The message binding joins the editor lease through the apply action.

<ui:copy id="agent-proposals-android-controller">
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
</ui:copy>

## Reuse the floating menu surface without moving code

The shared main-menu/action-search descriptor opens one workspace-owned floating reading. Its read-only sections page at scalar boundaries prepared off the UI thread. Explanation, claims, ordered source/resource edits, saved/draft construction consequences, connected before/after readings and requested checks remain one immutable intention. Explicit draft Apply, guarded Save and exact report retry are separate actions; no permanent sidebar or toolbar is introduced.

<ui:copy id="agent-proposals-android-view">
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
</ui:copy>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/AgentProposals.kt"><ui:paste select="#agent-proposals-android-seal"/><ui:paste select="#agent-proposals-android-controller"/><ui:paste select="#agent-proposals-android-view"/></ui:file>
## Exercise the complete seal and host transition before calling the path usable

Focused verification executed nine Android methods with zero skips or failures, including original managed editor binding, before-review edit/Undo ABA, independent editor identity, and cancelled publication status retention. The maintained owned-engine caller-provided fixture also executed one native controller flow with zero skips: menu review, coordinated unsaved Apply and Undo/Redo, literal `moved λ%?#.py` Save, confirmed v2 adoption, guarded inverse recovery, and stale-host refusal. It retained the same buffer, TextFieldState and scroll, left generated bytes unchanged, and cleaned its owned engine. This caller-provided intention receipt is separate from the second executed owned MCP-tool native flow (one test, zero skips): the real agent endpoint sends native context, the service seals original editor identity/revision, before-review edit/Undo ABA refuses, and a fresh managed intention applies and replays coordinated Undo/Redo while preserving the same editor and saved/generated bytes. No external provider credentials are involved. It is not a rendered Compose or physical-input/accessibility acceptance.

The focused fixtures keep catalogue text untrusted, mutate sealed material, replace physical guard/roster/revision observations, and release held validation after a new editor appears. They distinguish an applied draft from its optional host acknowledgment: losing that acknowledgment retains one exact report and cannot repeat Apply. Namespace saving must revalidate the seal and admit a current v2 receipt before adopting baselines. The actual owned-engine fixture is separate from these protocol/controller models; rendered Compose, physical input/IME and accessibility remain unverified.

<ui:copy id="agent-proposals-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import org.json.JSONArray
import kotlinx.coroutines.*
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class AgentProposalsTest {
 private val workspace="b".repeat(32);private val token="a".repeat(32);private val before="value = 1\n";private val after="value = 41\n"
 private fun source(text:String)=JSONObject().put("kind","file").put("identity","source-note").put("text",text).put("writable",true).put("removable",true).put("version",JSONObject.NULL)
 private fun saved(text:String)=JSONObject().put("kind","file").put("text",text).put("sha256",debugHash(text)).put("readonly",false).put("executable",false).put("mode",420)
 private fun reading()=JSONObject().put("version",1).put("behavior_protocol",1).put("namespace","/fixture").put("engine_workspace_id",workspace).put("workspace_id","d".repeat(64)).put("graph",JSONObject().put("nodes",JSONObject()).put("edges",JSONArray()).put("checks",JSONArray()))
 private fun material():String {
  val edit=JSONObject().put("changes",JSONObject().put(languageURI("note.py"),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",1).put("character",0))).put("newText",after))))
  val guards=JSONObject().put("note.py","loaded");val capture=JSONObject().put("context","captured").put("edit",edit).put("project_entry",JSONObject.NULL).put("drafts",JSONObject().put("note.py",JSONObject().put("text",before).put("base_sha256",debugHash(before)).put("save_guard","loaded").put("writable",true).put("version",JSONObject.NULL))).put("ownership_sources",JSONObject().put("note.py",before)).put("unaddressed_sources",JSONObject()).put("saved_guards",guards)
  val plan=JSONObject().put("version",1).put("context","captured").put("title","One complete change").put("annotations",JSONArray()).put("patches",JSONArray().put(JSONObject().put("path","note.py").put("before",source(before)).put("after",source(after))))
  val inputs=JSONObject().put("version",1).put("context","captured").put("workspace",workspace).put("sources",JSONObject().put("note.py",source(before))).put("saved_sources",JSONObject().put("note.py",saved(before))).put("saved_guards",guards).put("source_files_modified",false).put("refusals",JSONObject()).put("ownership",JSONObject().put("saved_sources",JSONObject()).put("draft_sources",JSONObject()).put("protected_entries",JSONArray()))
  val facts=JSONObject().put("version",1).put("context","captured").put("workspace",workspace).put("fingerprint","f".repeat(64)).put("protected_targets",JSONArray()).put("saved_sources",JSONObject().put("note.py",saved(after)))
  val intention=JSONObject().put("version",1).put("request_id","c".repeat(32)).put("origin","owned caller-provided fixture").put("title","One complete change").put("explanation","Change the implementation with its explanation.").put("claims",JSONArray().put("Agent claims passed; unverified text only")).put("check_ids",JSONArray()).put("capture",capture)
  return JSONObject().put("version",1).put("workspace",workspace).put("namespace","/fixture").put("origin_authority","caller-provided intention; not an agent attestation").put("intention",intention).put("inputs",inputs).put("plan",plan).put("consequences",facts).put("before",reading()).put("after",reading()).put("saved_guards",guards).toString()
 }
 private fun envelope(text:String=material())=JSONObject().put("version",1).put("proposal_protocol",1).put("token",token).put("fingerprint",debugHash(text)).put("workspace",workspace).put("namespace","/fixture").put("material",text).put("source_effects",false).put("checks_executed",false).put("host_application",JSONObject.NULL)
 private class Host(val w:Workspace,val b:DocumentBuffer,val model:AgentProposalModel,val mode:java.util.concurrent.atomic.AtomicReference<String>,val requests:MutableList<Pair<String,String>>,val entered:java.util.concurrent.CountDownLatch,val release:java.util.concurrent.CountDownLatch){fun count(path:String)=requests.count{it.first==path}}
 private suspend fun host(originalExtra:String?=null,binding:String?=null,recordedOrigin:String?=null,block:suspend(Host)->Unit)=coroutineScope {
  val b=DocumentBuffer(FileNode("note.py","note.py",false,emptyList(),null,null),FileText(before,"base","loaded"))
  val text=JSONObject(material()).apply{val capture=getJSONObject("intention").getJSONObject("capture");if(originalExtra=="anonymous")capture.getJSONObject("unaddressed_sources").put("original-anonymous","captured plain note");if(originalExtra=="named")capture.getJSONObject("ownership_sources").put("closed.txt","captured named note");if(binding!=null){put("origin_authority","engine-managed agent tool; claims remain unverified");if(binding!="absent")getJSONObject("intention").put("editor_snapshot",JSONObject().put("live",false).put("buffers",JSONArray().put(JSONObject().put("id",if(binding=="missing-id")JSONObject.NULL else b.debugIdentity).put("path","note.py").put("source_hash",debugHash(before)).put("save_guard","loaded").put("source_revision",when(binding){"unknown"->JSONObject.NULL;"noncanonical"->"00";else->b.sourceRevision.toString()}))))};recordedOrigin?.let{put("origin_authority",it)}}.toString();val proof=debugHash(text);val mode=java.util.concurrent.atomic.AtomicReference("normal");val requests=java.util.Collections.synchronizedList(mutableListOf<Pair<String,String>>());val entered=java.util.concurrent.CountDownLatch(1);val release=java.util.concurrent.CountDownLatch(1);var published:JSONObject?=null
  val listener=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0)
  listener.createContext("/api"){exchange->val path=exchange.requestURI.path;val wire=exchange.requestBody.bufferedReader().readText();requests.add(path to wire);val body=wire.takeIf{it.isNotEmpty()}?.let{JSONObject(it)};var status=200
   val reply=when(path){
    "/api/files"->JSONObject().put("root","Fixture").put("root_path","/fixture").put("workspace_id",workspace).put("tree",JSONArray())
    "/api/agent-proposals"->JSONObject().put("version",1).put("proposal_protocol",1).put("workspace",workspace).put("namespace","/fixture").put("reviews",JSONArray().put(JSONObject().put("token",token).put("fingerprint",proof).put("title","UNSEALED CATALOGUE LABEL")))
    "/api/agent-proposals/$token"->envelope(text).apply{if(mode.get()=="tamper")put("material",text+" ");if(mode.get()=="foreign")put("namespace","/other");if(mode.get()=="archive-reading")put("historical",true).put("reading_scope",JSONObject().put("workspace",workspace).put("namespace","/fixture"))}
    "/api/docs/workspace/agent/turns/recorded/fence-patch"->envelope(text)
    "/api/changes/resource-inputs"->JSONObject().put("sources",JSONObject().put("note.py",source(before))).put("saved_guards",JSONObject().put("note.py","loaded"))
    "/api/agent-proposals/validate"->{assertEquals(setOf("version","token","fingerprint"),body!!.keys().asSequence().toSet());if(mode.get()=="hold"){entered.countDown();release.await(5,java.util.concurrent.TimeUnit.SECONDS)};JSONObject().put("version",1).put("proposal_protocol",1).put("token",token).put("fingerprint",proof).put("workspace",workspace).put("namespace","/fixture").put("admitted",mode.get()!="refuse").put("source_effects",false).put("checks_executed",false)}
    "/api/agent-proposals/report"->{if(mode.get()=="report-lost"){status=409;JSONObject().put("error","Host report response unconfirmed")}else JSONObject(body!!.toString()).put("proposal_protocol",1).put("report_fingerprint","e".repeat(64)).put("status","host-reported draft application").put("saved",false).put("checks_executed",false)}
    "/api/changes/resource-save"->{if(exchange.requestMethod=="GET")published ?: error("No original publication")else {val request=body!!.getString("request_id");JSONObject().put("version",2).put("request_id",request).put("workspace",workspace).put("context","captured").put("adoption_current",mode.get()!="historical").put("outcome",JSONObject().put("complete",true).put("recovery",".hick-cache/resource-changes/$request/manifest.json").put("applied",JSONArray().put("note.py"))).put("saved_sources",JSONObject().put("note.py",saved(after))).put("source_files",JSONArray().put(JSONObject().put("file",JSONObject().put("name","note.py").put("path","note.py").put("dir",false)).put("saved",saved(after)).put("hash","saved-new").put("save_guard","published-guard"))).also{published=it;if(mode.get()=="publish-hold"){entered.countDown();release.await(5,java.util.concurrent.TimeUnit.SECONDS)}}}}
    else->{status=404;JSONObject().put("error","Unexpected fixture endpoint $path")}}
   val bytes=reply.toString().toByteArray();exchange.sendResponseHeaders(status,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}
  };listener.start();val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(background);w.engine=Engine("http://127.0.0.1:${listener.address.port}");w.engine!!.files();w.buffers[b.file.path]=b;w.show(b.file.path);val model=w.agentProposals
  try{block(Host(w,b,model,mode,requests,entered,release))}finally{release.countDown();w.buffers.values.forEach{it.language?.close()};background.cancel();listener.stop(0)}
 }
 private suspend fun read(h:Host){h.model.open(h.w);assertNull(h.model.value);assertFalse(h.model.notice.contains("UNSEALED"));h.model.read(h.w,h.model.entries.single());assertNotNull(h.model.notice,h.model.value)}
 @Test fun recordedFenceReviewUsesExactMessageAndExistingGuardedApply()=runBlocking {
  val source = "Read this before applying.\n\n~~~diff\n--- a/note.py\n+++ b/note.py\n@@ -1 +1 @@\n-value = 1\n+value = 41\n~~~\n"
  val hash = debugHash(source)
  val endpoint = "/api/docs/workspace/agent/turns/recorded/fence-patch"
  assertEquals(setOf(2), recordedFenceDebugContext(source).recordedPatches)
  assertTrue(recordedFenceDebugContext(source).launch.isEmpty())
  for (side in listOf("user", "assistant")) host(binding="known", recordedOrigin="recorded $side fence, source SHA-256 $hash; claims remain unverified") { h ->
   var stillCurrent = true
   val reading = AgentFenceReading("recorded", side, source) { stillCurrent }
   h.model.prepareRecordedPatch(h.w, reading, 2)
   val request = JSONObject(h.requests.single { it.first == endpoint }.second)
   val fence = request.getJSONObject("fence")
   assertEquals(side, fence.getString("side")); assertEquals(hash, fence.getString("source_hash")); assertEquals(2, fence.getInt("opening_line"))
   val intention = request.getJSONObject("intention")
   val capture = intention.getJSONObject("capture")
   assertEquals(0, capture.getJSONObject("edit").getJSONObject("changes").length())
   assertEquals("loaded", capture.getJSONObject("drafts").getJSONObject("note.py").getString("save_guard"))
   val owner = intention.getJSONObject("editor_snapshot").getJSONArray("buffers").getJSONObject(0)
   assertEquals(h.b.debugIdentity, owner.getString("id")); assertEquals(h.b.sourceRevision.toString(), owner.getString("source_revision"))
   assertTrue(h.model.notice, h.model.canApply)
   assertEquals(token, h.model.value!!.token)
   assertTrue(h.model.value!!.sections.any { it.title.contains("generated consequences") })
   assertEquals(before, h.b.text)
   assertEquals(0, h.count("/api/agent-proposals/validate"))
   if (side == "assistant") {
    stillCurrent = false
    h.model.read(h.w, h.model.entries.single())
    assertFalse(h.model.canApply)
    assertEquals(token, h.model.value!!.token)
   }
   h.model.applyDraft(h.w)
   assertEquals(if (side == "assistant") before else after, h.b.text)
   assertEquals(before, h.b.base)
   assertEquals(if (side == "assistant") 0 else 1, h.count("/api/agent-proposals/validate"))
   assertEquals(0, h.count("/api/changes/resource-save"))
  }
 }
 @Test fun historicalReadingNeverAcquiresCurrentEditorsOrApplies()=runBlocking{host{h->h.mode.set("archive-reading");read(h);assertTrue(h.model.value!!.historical);assertFalse(h.model.canApply);assertFalse(h.model.authorityCurrent);assertTrue(h.model.phase.contains("Historical"));h.model.applyDraft(h.w);h.model.prepareSave(h.w);assertEquals(before,h.b.text);assertEquals(0,h.count("/api/changes/resource-inputs"));assertEquals(0,h.count("/api/agent-proposals/validate"));assertEquals(0,h.count("/api/agent-proposals/report"))}}
 @Test fun historicalScopeAndSealRemainOriginalAndMalformedScopeRefuses()=runBlocking{
  val original=JSONObject(material());val old="f".repeat(32);original.put("workspace",old);original.getJSONObject("inputs").put("workspace",old);original.getJSONObject("consequences").put("workspace",old);for(key in listOf("before","after"))original.getJSONObject(key).put("engine_workspace_id",old)
  val raw=envelope(original.toString()).put("workspace",old).put("historical",true).put("reading_scope",JSONObject().put("workspace",workspace).put("namespace","/fixture"));val entry=AgentProposalEntry(token,raw.getString("fingerprint"));val value=sealedAgentProposal(raw,entry,workspace,"/fixture");assertTrue(value.historical);assertEquals(old,value.workspace);assertEquals(original.toString(),value.material)
  for(changed in listOf(JSONObject(raw.toString()).put("historical",false),JSONObject(raw.toString()).put("reading_scope",JSONObject().put("workspace","other").put("namespace","/fixture")),JSONObject(raw.toString()).put("material",raw.getString("material")+" ")))assertTrue(runCatching{sealedAgentProposal(changed,entry,workspace,"/fixture")}.isFailure)
  val live=envelope().put("durability",JSONObject().put("retained",false).put("notice","Private archive unavailable"));assertEquals("Private archive unavailable",sealedAgentProposal(live,AgentProposalEntry(token,live.getString("fingerprint")),workspace,"/fixture").retentionNotice)
 }
 @Test fun sealFailsBeforeContentAndAdmittedCollectionsCannotBeMutated()=runBlocking{
  val raw=envelope();val entry=AgentProposalEntry(token,raw.getString("fingerprint"));val valid=sealedAgentProposal(raw,entry,workspace,"/fixture");assertTrue(valid.origin.contains("caller-provided"));assertTrue(runCatching{(valid.patches as MutableList<*>).clear()}.isFailure);assertTrue(runCatching{(valid.sections as MutableList<*>).clear()}.isFailure)
  for(key in listOf("material","fingerprint","workspace","namespace")){val bad=JSONObject(raw.toString());bad.put(key,bad.getString(key)+"tampered");assertTrue(key,runCatching{sealedAgentProposal(bad,entry,workspace,"/fixture")}.isFailure)}
  for(flag in listOf("source_effects","checks_executed")){val bad=JSONObject(raw.toString()).put(flag,true);assertTrue(flag,runCatching{sealedAgentProposal(bad,entry,workspace,"/fixture")}.isFailure)}
  for(kind in listOf("tamper","foreign"))host{h->h.mode.set(kind);h.model.open(h.w);h.model.read(h.w,h.model.entries.single());assertNull(h.model.value);assertEquals(before,h.b.text);assertEquals(0,h.count("/api/agent-proposals/validate"))}
 }
 @Test fun coordinatedDraftUndoAndLostExactReportNeverRepeatApply()=runBlocking{host{h->read(h);val state=h.b.state;h.b.sourceScroll.scrollTo(7);h.mode.set("report-lost");h.model.applyDraft(h.w);assertEquals(after,h.b.text);assertEquals(before,h.b.base);assertSame(state,h.b.state);assertEquals(7,h.b.sourceScroll.value);assertTrue(h.model.hasReport);assertEquals(1,h.count("/api/agent-proposals/validate"));h.model.applyDraft(h.w);assertEquals(1,h.count("/api/agent-proposals/validate"));assertEquals(0,h.count("/api/changes/resource-save"));h.mode.set("normal");h.model.sendReport();assertFalse(h.model.hasReport);val reports=h.requests.filter{it.first.endsWith("/report")};assertEquals(2,reports.size);assertEquals(behaviorCanonical(JSONObject(reports.first().second)),behaviorCanonical(JSONObject(reports.last().second)));h.w.changes.open("undo",h.w);assertEquals(before,h.b.text);h.w.changes.open("redo",h.w);assertEquals(after,h.b.text)}}
 @Test fun changedRevisionGuardAndWholeRosterRetireBeforeAnyEffect()=runBlocking{for(change in listOf("revision","guard","roster","anonymous"))host{h->read(h);when(change){"revision"->h.b.advanceSourceRevision();"guard"->h.b.saveGuard="foreign";"roster"->h.w.buffers["other.txt"]=DocumentBuffer(FileNode("other.txt","other.txt",false,emptyList(),null,null),FileText("other","old","other-guard"));else->h.w.retainedResourceEditors["anonymous"]=DocumentBuffer(FileNode("Note","Untitled-new",false,emptyList(),null,null),FileText("new note",""))};assertFalse(h.model.canApply);h.model.applyDraft(h.w);assertEquals(before,h.b.text);assertEquals(0,h.count("/api/agent-proposals/validate"));assertEquals(0,h.count("/api/agent-proposals/report"))}}
 @Test fun beforeOpenContextAndOriginalGuardCannotBeBlessedByAFreshLease()=runBlocking{
  for(change in listOf("constructor","anonymous","guard"))host{h->when(change){"constructor"->h.w.buffers["other.md"]=DocumentBuffer(FileNode("Other","other.md",false,emptyList(),null,null),FileText("<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"other.py\">new</h:file></h:doc>","saved","other-guard"));"anonymous"->h.w.retainedResourceEditors["new"]=DocumentBuffer(FileNode("Note","Untitled-before",false,emptyList(),null,null),FileText("plain new note",""));else->h.b.saveGuard="new-original-guard"};read(h);assertFalse(change,h.model.canApply);h.model.applyDraft(h.w);h.model.prepareSave(h.w);assertEquals(before,h.b.text);assertEquals(0,h.count("/api/agent-proposals/validate"));assertEquals(0,h.count("/api/changes/resource-save"))}
  for(removed in listOf("named","anonymous"))host(originalExtra=removed){h->read(h);assertFalse(removed,h.model.canApply);h.model.applyDraft(h.w);assertEquals(before,h.b.text);assertEquals(0,h.count("/api/agent-proposals/report"))}
 }
 @Test fun originalStaticBindingRejectsBeforeReviewABAAndIndependentEditor()=runBlocking{
  host(binding="known"){h->read(h);assertTrue(h.model.notice,h.model.canApply)}
  host(binding="known"){h->assertTrue(h.b.applySourceEdits(listOf(NativeSourceEdit(h.b.text.length,h.b.text.length,"# later\n")),h.b.text));assertTrue(h.b.snippets.undoSource());assertEquals(before,h.b.text);read(h);assertFalse(h.model.canApply);h.model.applyDraft(h.w);h.model.prepareSave(h.w);assertEquals(0,h.count("/api/agent-proposals/validate"));assertEquals(before,h.b.text)}
  host(binding="known"){h->val replacement=DocumentBuffer(h.b.file,FileText(before,"base","loaded"));h.w.buffers[replacement.file.path]=replacement;read(h);assertFalse(h.model.canApply);h.model.applyDraft(h.w);assertSame(replacement,h.w.buffers[replacement.file.path]);assertEquals(before,replacement.text);assertEquals(0,h.count("/api/agent-proposals/validate"))}
  for(binding in listOf("unknown","absent","missing-id","noncanonical"))host(binding=binding){h->read(h);assertFalse(binding,h.model.canApply);h.model.applyDraft(h.w);assertEquals(0,h.count("/api/changes/resource-save"));assertEquals(0,h.count("/api/agent-proposals/report"));assertEquals(before,h.b.text)}
 }
 @Test fun heldAdmissionCannotApplyAfterANewRetainedEditor()=runBlocking{host{h->read(h);h.mode.set("hold");val pending=async{h.model.applyDraft(h.w)};withTimeout(5000){while(h.entered.count!=0L)delay(10)};h.w.retainedResourceTabs["new"]=DocumentBuffer(FileNode("Note","Untitled-new",false,emptyList(),null,null),FileText("plain",""));h.release.countDown();pending.await();assertEquals(before,h.b.text);assertEquals(0,h.count("/api/agent-proposals/report"));assertEquals(0,h.count("/api/changes/resource-save"))}}
 @Test fun namespaceSaveRechecksSealAndRequiresCurrentV2Adoption()=runBlocking{
  host{h->read(h);h.model.prepareSave(h.w);assertTrue(h.model.stagedSave);assertEquals(0,h.count("/api/changes/resource-save"));h.mode.set("refuse");h.model.save(h.w);assertEquals(0,h.count("/api/changes/resource-save"));assertEquals(before,h.b.text);h.mode.set("normal");h.model.save(h.w);assertEquals(1,h.count("/api/changes/resource-save"));assertEquals(after,h.b.text);assertEquals(after,h.b.base);assertEquals("published-guard",h.b.saveGuard);assertTrue(h.model.phase.contains("Saved namespace confirmed"));assertEquals(1,h.count("/api/agent-proposals/report"));assertFalse(h.model.publicationPending)}
  host{h->read(h);h.model.prepareSave(h.w);h.mode.set("historical");h.model.save(h.w);assertEquals(1,h.count("/api/changes/resource-save"));assertEquals(before,h.b.text);assertEquals(before,h.b.base);assertTrue(h.model.publicationPending);h.model.save(h.w);assertEquals(1,h.count("/api/changes/resource-save"));assertEquals(0,h.count("/api/agent-proposals/report"))}
 }
 @Test fun cancelledPublicationRetainsItsOriginalStatusIdentityAndNeverRepeatsWrite()=runBlocking{host{h->read(h);h.model.prepareSave(h.w);h.mode.set("publish-hold");val saving=async{h.model.save(h.w)};withTimeout(5000){while(h.entered.count!=0L)delay(10)};saving.cancel();h.release.countDown();saving.join();assertTrue(h.model.publicationPending);assertEquals(before,h.b.text);h.model.close();h.model.open(h.w);h.model.publicationStatus(h.w);assertFalse(h.model.notice,h.model.publicationPending);assertEquals(after,h.b.text);assertEquals(1,h.requests.count{it.first=="/api/changes/resource-save"&&it.second.isNotEmpty()});assertEquals(1,h.requests.count{it.first=="/api/changes/resource-save"&&it.second.isEmpty()});assertFalse(h.model.hasReport)}}
 @Test fun promptSnapshotCarriesOriginalGuardsAndTypedAnonymousSources()=runBlocking{host{h->val note=DocumentBuffer(FileNode("Note","Untitled-note",false,emptyList(),null,null),FileText("plain note",""));h.w.retainedResourceEditors["note"]=note;val context=simpleAgentContext(h.w);assertEquals(false,context.get("live"));val rows=context.getJSONArray("buffers");val named=(0 until rows.length()).map{rows.getJSONObject(it)}.single{it.optString("id")==h.b.debugIdentity};assertEquals("note.py",named.getString("path"));assertEquals("loaded",named.getString("save_guard"));val revision=named.getJSONArray("observations").getJSONObject(0);assertEquals("editor_revision",revision.getString("kind"));assertEquals(1,revision.get("version"));assertEquals(h.b.sourceRevision.toString(),revision.getString("source_revision"));val unnamed=(0 until rows.length()).map{rows.getJSONObject(it)}.single{it.optString("id")==note.debugIdentity};assertTrue(unnamed.isNull("path"));assertFalse(unnamed.has("save_guard"));assertEquals("plain note",unnamed.getString("content"))}}
 @Test(timeout=180000) fun liveHistoricalProposalKeepsDirtyEditorAndHasNoApply()=runBlocking{
  val origin=System.getenv("HICKORY_PROPOSAL_HISTORY_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_PROPOSAL_HISTORY_WORKSPACE") ?: error("Missing owned archive fixture"));check(root.resolve(".hickory-proposal-archive-fixture").readText()=="owned-proposal-archive-v1\n");val token=System.getenv("HICKORY_PROPOSAL_HISTORY_TOKEN") ?: error("Missing token");val fingerprint=System.getenv("HICKORY_PROPOSAL_HISTORY_FINGERPRINT") ?: error("Missing seal");val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(background);val server=Engine(origin!!);w.engine=server
  try{val(_,tree)=server.files();assertEquals(root.toPath().toRealPath().toString(),server.sourceRootAddress);w.rootFiles=tree;fun find(nodes:List<FileNode>):FileNode?{for(file in nodes){if(file.path=="note.py")return file;find(file.children)?.let{return it}};return null};val b=w.loadBuffer(find(tree) ?: error("Owned note missing"));w.show(b.file.path);val original=b.text;assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(original.length,original.length,"# retained archive draft\n")),original));val draft=b.text;val state=b.state;val selection=state.selection;b.sourceScroll.scrollTo(9);val scroll=b.sourceScroll.value;val m=w.agentProposals
   m.open(w);val entry=m.entries.single{it.token==token};assertEquals(fingerprint,entry.fingerprint);m.read(w,entry);assertTrue(m.notice,m.value?.historical==true);assertTrue(m.value!!.sections.any{it.title.startsWith("Explanation")});assertEquals(fingerprint,m.value!!.fingerprint);assertFalse(m.canApply);assertFalse(m.authorityCurrent);assertFalse(m.hasReport);m.applyDraft(w);m.prepareSave(w);assertEquals(draft,b.text);assertSame(state,b.state);assertEquals(selection,b.state.selection);assertEquals(scroll,b.sourceScroll.value);m.close();assertTrue(b.snippets.undoSource());assertEquals(original,b.text);assertFalse(m.canApply);assertEquals(original,root.resolve("note.py").readText())
  }finally{w.buffers.values.forEach{it.language?.close()};background.cancel()}
 }
 @Test(timeout=180000) fun liveOwnedEngineSealedProposalDraftNamespaceAndRecovery()=runBlocking{
  val origin=System.getenv("HICKORY_NATIVE_AGENT_PROPOSAL_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_NATIVE_AGENT_PROPOSAL_WORKSPACE") ?: error("Use the maintained owned proposal fixture"));check(root.resolve(".hickory-native-agent-proposal-fixture").readText()=="owned-native-agent-proposal-v1\n"){"Refuse a non-owned proposal workspace"}
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(background);val server=Engine(origin!!);w.engine=server
  try{val (_,tree)=server.files();assertEquals(root.toPath().toRealPath().toString(),server.sourceRootAddress);w.rootFiles=tree
   fun find(nodes:List<FileNode>,path:String):FileNode?{for(file in nodes){if(file.path==path)return file;find(file.children,path)?.let{return it}};return null}
   val b=w.loadBuffer(find(tree,"note.py") ?: error("Owned note source missing"));w.show(b.file.path);val state=b.state;val original=b.text;val originalGenerated=root.resolve("generated.py").readText();assertNotNull(b.saveGuard);b.selectSourceRange(2,2,b.text);b.sourceScroll.scrollTo(9);val scroll=b.sourceScroll.value
   fun fullEdit(path:String,old:String,text:String)=JSONObject().put("changes",JSONObject().put(languageURI(path),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",languagePosition(old,old.length))).put("newText",text))))
   suspend fun prepare(edit:JSONObject,title:String):String {val targets=server.request("POST","/api/changes/resource-targets",JSONObject().put("edit",edit)).getJSONArray("paths");val paths=(0 until targets.length()).map{targets.getString(it)};val captured=w.resourceChanges.acquire(w,paths,edit){true};val capture=captured.first;capture.put("project_entry","project.md");val intention=JSONObject().put("version",1).put("request_id",java.util.UUID.randomUUID().toString().replace("-","")).put("origin","owned native caller-provided fixture").put("title",title).put("explanation","Review the implementation and its declared consequences together. No check has run.").put("claims",JSONArray().put("Expected behavior only; not an observed verdict")).put("check_ids",JSONArray()).put("capture",capture).put("reading",behaviorRetainedContext(w,"project.md",b.file.path));val envelope=server.request("POST","/api/agent-proposals/prepare",intention);assertTrue(captured.third());return envelope.getString("token")}
   val descriptors=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()});val model=w.agentProposals
   suspend fun show(token:String){val action=w.commands(descriptors).single{it.id=="agent.proposals"};assertNull(action.reason);action.run();withTimeout(10000){while(model.busy||model.entries.none{it.token==token})delay(20)};model.read(w,model.entries.single{it.token==token});assertTrue(model.notice,model.canApply);assertTrue(model.value!!.origin.contains("caller-provided"));assertEquals(debugHash(model.value!!.material),model.value!!.fingerprint)}
   val firstToken=prepare(fullEdit("note.py",original,after),"Owned complete draft");show(firstToken);assertTrue(model.value!!.sections.any{it.title.contains("generated consequences")});assertEquals(original,root.resolve("note.py").readText());model.applyDraft(w);assertEquals(after,b.text);assertEquals(original,b.base);assertFalse(model.hasReport);assertSame(state,b.state);assertSame(b,w.currentBuffer);assertEquals(scroll,b.sourceScroll.value);assertEquals(original,root.resolve("note.py").readText());model.applyDraft(w);assertEquals(after,b.text);w.changes.open("undo",w);assertEquals(original,b.text);w.changes.open("redo",w);assertEquals(after,b.text)
   val movedPath="moved λ%?#.py";val moved="value = 42\n";val resourceEdit=JSONObject().put("documentChanges",JSONArray().put(JSONObject().put("kind","rename").put("oldUri",languageURI("note.py")).put("newUri",languageURI(movedPath))).put(JSONObject().put("textDocument",JSONObject().put("uri",languageURI(movedPath)).put("version",JSONObject.NULL)).put("edits",JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",languagePosition(after,after.length))).put("newText",moved)))))
   val namespaceToken=prepare(resourceEdit,"Owned guarded move");show(namespaceToken);assertFalse(agentProposalTextOnly(model.value!!));model.prepareSave(w);assertTrue(model.notice,model.stagedSave);assertFalse(root.resolve(movedPath).exists());model.save(w);assertFalse(model.notice,model.publicationPending);assertTrue(model.phase,model.phase.contains("Saved namespace confirmed"));assertEquals(moved,root.resolve(movedPath).readText());assertFalse(root.resolve("note.py").exists());assertSame(b,w.currentBuffer);assertSame(state,b.state);assertEquals(movedPath,b.file.path);assertEquals(moved,b.base);assertNotNull(b.saveGuard);assertEquals(scroll,b.sourceScroll.value)
   w.changes.open("undo",w);withTimeout(30000){while(w.resourceChanges.busy||w.resourceChanges.review==null){delay(20);if(!w.resourceChanges.busy&&w.resourceChanges.review==null&&w.resourceChanges.notice.isNotEmpty())error(w.resourceChanges.notice)}};w.resourceChanges.apply(w);assertFalse(w.resourceChanges.notice,w.resourceChanges.hasPending);assertEquals(original,root.resolve("note.py").readText());assertFalse(root.resolve(movedPath).exists());assertSame(b,w.currentBuffer);assertSame(state,b.state);assertEquals(after,b.text);assertEquals(original,b.base)
   val staleToken=prepare(fullEdit("note.py",b.text,"value = 99\n"),"Owned stale host guard");show(staleToken);assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"# later edit\n")),b.text));assertTrue(b.snippets.undoSource());val same=b.text;model.applyDraft(w);assertEquals(same,b.text);assertEquals(original,root.resolve("note.py").readText());assertFalse(model.canApply);assertEquals(scroll,b.sourceScroll.value);assertSame(state,b.state);assertEquals(originalGenerated,root.resolve("generated.py").readText());assertFalse(model.hasReport)
  }finally{w.agentProposals.close();w.buffers.values.forEach{it.language?.close()};background.cancel()}
 }

 @Test(timeout=180000) fun liveOwnedManagedToolOriginalEditorBindingAndBeforeReviewABA()=runBlocking{
  val origin=System.getenv("HICKORY_NATIVE_AGENT_PROPOSAL_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_NATIVE_AGENT_PROPOSAL_WORKSPACE") ?: error("Use the maintained owned fixture"));check(root.resolve(".hickory-native-agent-proposal-fixture").readText()=="owned-native-agent-proposal-v1\n")
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(background);val server=Engine(origin!!);w.engine=server
  try{val (_,tree)=server.files();w.rootFiles=tree;val file=tree.single{it.path=="note.py"};val b=w.loadBuffer(file);w.show(file.path);val state=b.state;val original=b.text;val generated=root.resolve("generated.py").readText();b.selectSourceRange(2,2,b.text);b.sourceScroll.scrollTo(9);val scroll=b.sourceScroll.value
   suspend fun managed():String{val edit=JSONObject().put("changes",JSONObject().put(languageURI("note.py"),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",languagePosition(original,original.length))).put("newText",after))));val intention=JSONObject().put("version",1).put("request_id",java.util.UUID.randomUUID().toString().replace("-","")).put("origin","controlled native provider claim").put("title","Owned managed original binding").put("explanation","Change the implementation; evidence remains unverified.").put("claims",JSONArray()).put("check_ids",JSONArray()).put("capture",JSONObject().put("context",java.util.UUID.randomUUID().toString().replace("-","")).put("project_entry","project.md").put("drafts",JSONObject()).put("ownership_sources",JSONObject()).put("unaddressed_sources",JSONObject()).put("saved_guards",JSONObject()).put("edit",edit)).put("reading",JSONObject().put("version",1).put("entry","project.md").put("sources",JSONObject()).put("missing_sources",JSONArray()).put("focus","note.py").put("unaddressed_sources",JSONObject()).put("saved_guards",JSONObject()));val result=root.parentFile.resolve("tool-result.json");result.delete();root.parentFile.resolve("calls.json").writeText(JSONObject().put("intention",intention).toString());server.request("POST","/api/docs/workspace/agent/acp",JSONObject().put("backend","owned-proposal"));server.request("POST","/api/docs/workspace/agent",JSONObject().put("backend","owned-proposal").put("prompt","Prepare the owned intention without applying or running it.").put("context",simpleAgentContext(w)));var summary:JSONObject?=null;withTimeout(30000){while(summary==null){if(result.exists())try{val response=JSONObject(result.readText()).getJSONObject("result");check(response.get("isError")==false){response.toString()};summary=JSONObject(response.getJSONArray("content").getJSONObject(0).getString("text"))}catch(e:org.json.JSONException){};if(summary==null)delay(30)}};return summary!!.getString("token")}
   val m=w.agentProposals;suspend fun read(token:String){m.open(w);m.read(w,m.entries.single{it.token==token});assertTrue(m.value!!.origin.contains("engine-managed"));val rows=JSONObject(m.value!!.material).getJSONObject("intention").getJSONObject("editor_snapshot").getJSONArray("buffers");val row=(0 until rows.length()).map{rows.getJSONObject(it)}.single{it.optString("id")==b.debugIdentity};assertEquals(b.debugIdentity,row.getString("id"));assertTrue(row.get("source_revision") is String)}
   val stale=managed();assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"# temporary\n")),b.text));assertTrue(b.snippets.undoSource());assertEquals(original,b.text);read(stale);assertFalse(m.canApply);m.applyDraft(w);assertEquals(original,b.text);assertEquals(original,root.resolve("note.py").readText())
   val fresh=managed();read(fresh);assertTrue(m.notice,m.canApply);m.applyDraft(w);assertEquals(after,b.text);assertEquals(original,b.base);assertFalse(m.hasReport);assertSame(state,b.state);assertSame(b,w.currentBuffer);assertEquals(scroll,b.sourceScroll.value);assertEquals(original,root.resolve("note.py").readText());assertEquals(generated,root.resolve("generated.py").readText());w.changes.open("undo",w);assertEquals(original,b.text);w.changes.open("redo",w);assertEquals(after,b.text)
  }finally{w.agentProposals.close();w.buffers.values.forEach{it.language?.close()};background.cancel()}
 }

}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/AgentProposalsTest.kt"><ui:paste select="#agent-proposals-android-tests"/></ui:file>
</ui:doc>
