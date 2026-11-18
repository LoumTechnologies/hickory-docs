package com.hickorydocs.simple
import org.json.JSONObject

data class WorkspaceEntryStamp(val version:Long,val identity:String,val generation:Long,val path:String?)
class WorkspaceEntrySelectionProof(value:JSONObject) {
 private val material=value.toString()
 val workspace:String
 val namespace:String
 val entry:WorkspaceEntryStamp
 init {
  val body=JSONObject(material)
  require(body.keys().asSequence().all{it in setOf("version","entry_protocol","workspace","namespace","entry","guard","source_guard","source_hash","movable","source_effects")})
  fun text(value:JSONObject,key:String)=value.opt(key) as? String ?: error("Workspace entry $key must be an exact string.")
  fun integer(value:JSONObject,key:String):Long {val raw=value.opt(key);require(raw is Int||raw is Long){"Workspace entry $key must be an exact integer."};return (raw as Number).toLong()}
  fun hex(value:String,count:Int)=value.length==count&&value.all{it in '0'..'9'||it in 'a'..'f'}
  require(integer(body,"version")==1L&&integer(body,"entry_protocol")==1L)
  workspace=text(body,"workspace");namespace=text(body,"namespace")
  require(hex(workspace,32)&&namespace.isNotEmpty()&&hex(text(body,"guard"),64))
  require(body.opt("source_effects") is Boolean&&body.get("source_effects")==false)
  require(body.opt("movable") is Boolean)
  val source=body.opt("entry") as? JSONObject ?: error("Workspace entry stamp is malformed.")
  require(source.keys().asSequence().all{it in setOf("version","identity","generation","path")})
  val generation=integer(source,"generation");val identity=text(source,"identity")
  require(integer(source,"version")==1L&&hex(identity,32)&&generation in 0L..0xffffffffL&&source.has("path"))
  val rawPath=source.get("path");val path=if(rawPath===JSONObject.NULL)null else rawPath as? String ?: error("Workspace entry path is malformed.")
  if(path==null){require(!body.has("source_guard")&&!body.has("source_hash")&&body.get("movable")==false)}
  else {
   require(path.isNotEmpty()&&!path.contains(':')&&!path.contains('\\')&&path.none{it.code<32||it.code==127}&&path.split('/').none{it.isEmpty()||it=="."||it==".."})
   val guard=text(body,"source_guard");val hash=text(body,"source_hash")
   require(guard.isNotEmpty()&&guard.toByteArray(Charsets.UTF_8).size<=16384&&hash.startsWith("sha256:")&&hex(hash.removePrefix("sha256:"),64))
  }
  entry=WorkspaceEntryStamp(1,identity,generation,path)
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
