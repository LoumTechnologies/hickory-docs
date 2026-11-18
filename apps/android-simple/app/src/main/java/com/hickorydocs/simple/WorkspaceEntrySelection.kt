package com.hickorydocs.simple
import org.json.JSONObject

data class WorkspaceEntryStamp(val version:Long,val identity:String,val generation:Long,val path:String?)
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

class WorkspaceEntryIntent(value:JSONObject,proof:WorkspaceEntrySelectionProof,kind:String,path:String?) {
 val token=entryText(value,"token")
 val context=entryText(value,"context")
 val title=entryText(value,"title")
 val before=entryStamp(value.getJSONObject("before_entry"))
 val after=entryStamp(value.getJSONObject("after_entry"))
 private val material=value.getJSONObject("edit").toString()
 init {
  check(entryInteger(value,"version")==1L&&value.opt("workspace")==proof.workspace&&value.opt("namespace")==proof.namespace&&
   value.opt("kind")==kind&&value.opt("source_files_modified")==false&&entryHex(token,64)&&context==token&&title.isNotEmpty()&&
   before==proof.entry&&after.identity==before.identity&&after.generation==before.generation+1&&after.path==path){
   "The entry review did not retain the requested construction and address."
  }
 }
 fun edit()=JSONObject(material)
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
 check(entryInteger(value,"version")==1L&&value.opt("state")=="confirmed"&&value.opt("request_id")==request&&
  entryStamp(value.getJSONObject("before_entry"))==intent.before&&entryStamp(value.getJSONObject("after_entry"))==intent.after&&
  entryStamp(value.getJSONObject("current_entry"))==intent.after){"The original main construction publication is unconfirmed."}
}

fun workspaceEntryPublicationHadNoEffect(raw: Any?, request: String, intent: WorkspaceEntryIntent?): Boolean {
 val outcome = raw as? JSONObject ?: return false
 if (outcome.opt("state") != "no_effect") return false
 val expected = checkNotNull(intent) { "No original entry review exists for this receipt." }
 val consumed = outcome.opt("reservation_consumed") as? Boolean ?: error("The entry reservation outcome is malformed.")
 val retained = expected.before.copy(generation = expected.before.generation + if (consumed) 1 else 0)
 check(entryInteger(outcome, "version") == 1L && outcome.opt("request_id") == request &&
       entryStamp(outcome.getJSONObject("before_entry")) == expected.before &&
       entryStamp(outcome.getJSONObject("after_entry")) == expected.after &&
       entryStamp(outcome.getJSONObject("current_entry")) == retained) {
  "The no-effect receipt does not match the original entry review."
 }
 return true
}
