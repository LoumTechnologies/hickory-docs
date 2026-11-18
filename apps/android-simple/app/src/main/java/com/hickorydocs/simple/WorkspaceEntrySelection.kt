package com.hickorydocs.simple
import org.json.JSONObject

data class WorkspaceEntryStamp(val version:Long,val identity:String,val generation:Long,val path:String?) {
 fun permitsRestorationReview(historical:WorkspaceEntryStamp)=
  identity==historical.identity&&path==historical.path&&generation>=historical.generation
}
private fun entryText(value:JSONObject,key:String)=value.opt(key) as? String ?: error("Workspace entry $key must be an exact string.")
private fun entryInteger(value:JSONObject,key:String):Long {
 val raw=value.opt(key)
 require(raw is Int||raw is Long){"Workspace entry $key must be an exact integer."}
 return (raw as Number).toLong()
}
private fun entryHex(value:String,count:Int)=value.length==count&&value.all{it in '0'..'9'||it in 'a'..'f'}
private fun entryStamp(source:JSONObject):WorkspaceEntryStamp {
 require(source.keys().asSequence().all{it in setOf("version","identity","generation","path")})
 val generation=entryInteger(source,"generation");val identity=entryText(source,"identity")
 require(entryInteger(source,"version")==1L&&entryHex(identity,32)&&generation in 0L..0xffffffffL&&source.has("path"))
 val rawPath=source.get("path")
 val path=if(rawPath===JSONObject.NULL)null else rawPath as? String ?: error("Workspace entry path is malformed.")
 if(path!=null)require(path.isNotEmpty()&&!path.contains(':')&&!path.contains('\\')&&path.none{it.code<32||it.code==127}&&path.split('/').none{it.isEmpty()||it=="."||it==".."})
 return WorkspaceEntryStamp(1,identity,generation,path)
}
class WorkspaceEntrySelectionProof(value:JSONObject) {
 private val material=value.toString()
 val workspace:String
 val namespace:String
 val entry:WorkspaceEntryStamp
 val movable:Boolean
 init {
  val body=JSONObject(material)
  require(body.keys().asSequence().all{it in setOf("version","entry_protocol","workspace","namespace","entry","guard","source_guard","source_hash","movable","source_effects")})
  require(entryInteger(body,"version")==1L&&entryInteger(body,"entry_protocol")==1L)
  workspace=entryText(body,"workspace");namespace=entryText(body,"namespace")
  require(entryHex(workspace,32)&&namespace.isNotEmpty()&&entryHex(entryText(body,"guard"),64))
  require(body.opt("source_effects") is Boolean&&body.get("source_effects")==false)
  movable=body.opt("movable") as? Boolean ?: error("Workspace entry move capability is malformed.")
  entry=entryStamp(body.getJSONObject("entry"))
  if(entry.path==null){require(!body.has("source_guard")&&!body.has("source_hash")&&!movable)}
  else {
   val guard=entryText(body,"source_guard");val hash=entryText(body,"source_hash")
   require(guard.isNotEmpty()&&guard.toByteArray(Charsets.UTF_8).size<=16384&&hash.startsWith("sha256:")&&entryHex(hash.removePrefix("sha256:"),64))
  }
 }
 fun body()=JSONObject(material)
}
data class FrozenWorkspaceEntrySelection(val entryPath:String?,val proof:WorkspaceEntrySelectionProof,val current:()->Boolean)
object WorkspaceEntrySelections {
 suspend fun read(load:suspend()->JSONObject,workspace:String,namespace:String,current:()->Boolean):FrozenWorkspaceEntrySelection {
  check(current()){"Original source context changed before entry acquisition."}
  val proof=WorkspaceEntrySelectionProof(load())
  check(current()&&proof.workspace==workspace&&proof.namespace==namespace){"Workspace entry response belongs to an obsolete source context."}
  return FrozenWorkspaceEntrySelection(proof.entry.path,proof,current)
 }
}

class WorkspaceEntryIntent private constructor(value:JSONObject) {
 val token=entryText(value,"token")
 val context=entryText(value,"context")
 val title=entryText(value,"title")
 val before=entryStamp(value.getJSONObject("before_entry"))
 val after=entryStamp(value.getJSONObject("after_entry"))
 private val material=value.getJSONObject("edit").toString()
 var interrupted:InterruptedWorkspaceEntry?=null
  private set

 constructor(value:JSONObject,proof:WorkspaceEntrySelectionProof,kind:String,path:String?):this(value) {
  check(entryInteger(value,"version")==1L&&value.opt("workspace")==proof.workspace&&value.opt("namespace")==proof.namespace&&
   value.opt("kind")==kind&&value.opt("source_files_modified")==false&&entryHex(token,64)&&context==token&&title.isNotEmpty()&&
   before==proof.entry&&after.identity==before.identity&&after.generation==before.generation+1&&after.path==path){
   "The entry review did not retain the requested construction and address."
  }
 }
 fun edit()=JSONObject(material)

 companion object {
  fun forCompletion(value:JSONObject,reading:InterruptedWorkspaceEntry,superseding:WorkspaceEntryAttempt?):WorkspaceEntryIntent {
   val intent=WorkspaceEntryIntent(value)
   val supersedes=value.get("supersedes").let { raw ->
    if(raw===JSONObject.NULL)null else WorkspaceEntryAttempt.read(raw as? JSONObject ?: error("Invalid interrupted publication attempt."))
   }
   check(reading.canComplete&&entryInteger(value,"version")==1L&&value.opt("kind")=="interrupted_completion"&&
    value.opt("workspace")==reading.workspace&&value.opt("namespace")==reading.namespace&&value.opt("source_files_modified")==false&&
    value.opt("interrupted_reading_fingerprint")==reading.fingerprint&&supersedes==superseding&&
    entryHex(intent.token,64)&&intent.context==intent.token&&intent.title.isNotEmpty()&&intent.before==reading.currentEntry&&
    intent.after.identity==intent.before.identity&&intent.after.generation==reading.reservationFloor+1&&intent.after.path==reading.destination) {
    "The interrupted Move review did not retain the original evidence and intended destination."
   }
   intent.interrupted=reading
   return intent
  }
 }
}

data class WorkspaceEntryAttempt(val request:String,val context:String,val before:WorkspaceEntryStamp,
 val after:WorkspaceEntryStamp,val fingerprint:String?) {
 companion object {
  fun read(value:JSONObject):WorkspaceEntryAttempt {
   val request=entryText(value,"request_id")
   val context=entryText(value,"context")
   val before=entryStamp(value.getJSONObject("before_entry"))
   val after=entryStamp(value.getJSONObject("after_entry"))
   val fingerprint=value.get("reviewed_fingerprint").let{if(it===JSONObject.NULL)null else it as? String ?: error("Malformed reviewed fingerprint.")}
   check(entryHex(request,32)&&entryHex(context,64)&&before.identity==after.identity&&after.generation>before.generation&&
    (fingerprint==null||entryHex(fingerprint,64))){"Invalid interrupted publication attempt."}
   return WorkspaceEntryAttempt(request,context,before,after,fingerprint)
  }
 }
}

class InterruptedWorkspaceEntry private constructor(val workspace:String,val namespace:String,val fingerprint:String,val request:String,val manifestHash:String,
 val source:String,val destination:String,val currentEntry:WorkspaceEntryStamp,val reservationFloor:Long,
 val canComplete:Boolean,val notice:String?,val missingOriginal:Boolean,val attempts:List<WorkspaceEntryAttempt>,private val encoded:String) {
 fun body()=JSONObject(encoded)

 companion object {
  fun read(response:JSONObject,workspace:String,namespace:String):InterruptedWorkspaceEntry? {
   check(entryInteger(response,"version")==1L&&entryHex(workspace,32)&&namespace.isNotEmpty()&&
    response.opt("workspace")==workspace&&response.opt("namespace")==namespace&&response.opt("source_files_modified")==false) {
    "Interrupted construction evidence belongs to a different workspace."
   }
   if(response.get("interrupted")===JSONObject.NULL)return null
   val reading=response.getJSONObject("interrupted")
   val facts=reading.getJSONObject("material")
   val change=facts.getJSONObject("change")
   val notice=facts.get("notice").let{if(it===JSONObject.NULL)null else it as? String ?: error("Interrupted Move notice is malformed.")}
   val canComplete=facts.get("can_complete") as? Boolean ?: error("Interrupted Move capability is malformed.")
   if(change.opt("kind")!="move"&&!canComplete&&!notice.isNullOrEmpty())error(notice)
   val current=entryStamp(facts.getJSONObject("current_entry"))
   val before=entryStamp(change.getJSONObject("before"))
   val after=entryStamp(change.getJSONObject("after"))
   val floor=entryInteger(facts,"reservation_floor")
   val fingerprint=entryText(reading,"fingerprint")
   val source=before.path ?: error("Interrupted Move has no original source.")
   val destination=after.path ?: error("Interrupted Move has no destination.")
   val paths=facts.getJSONArray("paths")
   val attempts=facts.getJSONArray("attempts")
   val request=entryText(facts,"request_id")
   val manifest=entryText(facts,"manifest_sha256")
   check(entryInteger(facts,"version")==1L&&facts.opt("workspace")==workspace&&facts.opt("namespace")==namespace&&
    facts.opt("source_files_modified")==false&&entryHex(fingerprint,64)&&entryHex(request,32)&&
    entryHex(manifest,64)&&entryInteger(change,"version")==1L&&change.opt("kind")=="move"&&
    current==before&&before.identity==after.identity&&after.generation==before.generation+1&&floor in after.generation..0xffffffffL&&
    source!=destination&&(notice?.isNotEmpty() ?: canComplete)&&paths.length()==2&&attempts.length()<=256){
    "Interrupted construction evidence does not describe one exact Move."
   }
   val states=(0 until paths.length()).associate { index ->
    val item=paths.getJSONObject(index)
    entryText(item,"path") to entryText(item,"saved_kind")
   }
   check(states.keys==setOf(source,destination)&&states.values.all{it=="file"||it=="missing"||(it=="unknown"&&!canComplete)}) {
    "Interrupted construction evidence has invalid source paths."
   }
   val observed=(0 until attempts.length()).map{WorkspaceEntryAttempt.read(attempts.getJSONObject(it))}
   return InterruptedWorkspaceEntry(workspace,namespace,fingerprint,request,manifest,source,destination,current,floor,canComplete,notice,
    states[source]=="missing",observed,reading.toString())
  }
 }
}

data class WorkspaceEntryTransition(val before:WorkspaceEntryStamp,val after:WorkspaceEntryStamp) {
 companion object {
  fun read(raw:Any?,record:String):WorkspaceEntryTransition? {
   if(raw==null||raw===JSONObject.NULL)return null
   val value=raw as? JSONObject ?: error("This recovery record lacks a supported entry transition.")
   if(entryInteger(value,"version")==1L&&value.opt("state")=="unavailable")error(entryText(value,"notice"))
   check(entryInteger(value,"version")==1L&&value.opt("state")=="retained"&&value.opt("request_id")==record&&
    entryHex(entryText(value,"manifest_sha256"),64)&&value.opt("kind") in listOf("move","restoration","create","repair")){
    "This recovery record lacks a supported entry transition."
   }
   val before=entryStamp(value.getJSONObject("before_entry"))
   val after=entryStamp(value.getJSONObject("after_entry"))
   check(before.identity==after.identity&&before.generation+1==after.generation){"The retained entry transition is inconsistent."}
   return WorkspaceEntryTransition(before,after)
  }
 }
}

fun confirmWorkspaceEntryPublication(raw:Any?,request:String,intent:WorkspaceEntryIntent?) {
 if(intent==null){check(raw==null){"Unexpected main construction publication."};return}
 val value=raw as? JSONObject ?: error("The original main construction publication is unconfirmed.")
 val interrupted=intent.interrupted
 check(entryInteger(value,"version")==if(interrupted==null)1L else 2L) {"The publication uses an unsupported entry receipt."}
 check(value.opt("state")=="confirmed"&&value.opt("request_id")==request&&
  entryStamp(value.getJSONObject("before_entry"))==intent.before&&entryStamp(value.getJSONObject("after_entry"))==intent.after&&
  entryStamp(value.getJSONObject("current_entry"))==intent.after){"The original main construction publication is unconfirmed."}
 if(interrupted!=null)check(value.opt("kind")=="interrupted_completion"&&value.opt("reservation_consumed")==true&&
  entryInteger(value,"reservation_floor_before")==interrupted.reservationFloor&&entryInteger(value,"reservation_floor")==intent.after.generation&&
  value.get("remaining_interrupted")===JSONObject.NULL){"The receipt does not confirm completion of the reviewed interruption."}
}

fun workspaceEntryPublicationHadNoEffect(raw: Any?, request: String, intent: WorkspaceEntryIntent?): Boolean {
 val outcome = raw as? JSONObject ?: return false
 if (outcome.opt("state") != "no_effect") return false
 val expected = checkNotNull(intent) { "No original entry review exists for this receipt." }
 val consumed = outcome.opt("reservation_consumed") as? Boolean ?: error("The entry reservation outcome is malformed.")
 val interrupted=expected.interrupted
 val version=if(interrupted==null)1L else 2L
 check(entryInteger(outcome,"version")==version && outcome.opt("request_id")==request &&
       entryStamp(outcome.getJSONObject("before_entry")) == expected.before &&
       entryStamp(outcome.getJSONObject("after_entry")) == expected.after) {
  "The no-effect receipt does not match the original entry review."
 }
 val current=entryStamp(outcome.getJSONObject("current_entry"))
 val advance=if(consumed)1 else 0
 if(interrupted==null){
  check(current==expected.before.copy(generation=expected.before.generation+advance)) {"The original entry namespace is unconfirmed."}
 }else{
  val remaining=outcome.getJSONObject("remaining_interrupted")
  check(outcome.opt("kind")=="interrupted_completion"&&current==expected.before&&
   entryInteger(outcome,"reservation_floor_before")==interrupted.reservationFloor&&
   entryInteger(outcome,"reservation_floor")==interrupted.reservationFloor+advance&&
   remaining.opt("request_id")==interrupted.request&&remaining.opt("manifest_sha256")==interrupted.manifestHash) {
   "The prior interrupted Move remains unconfirmed."
  }
 }
 return true
}
