package com.hickorydocs.simple
import androidx.compose.runtime.*
import org.json.JSONObject
import org.json.JSONArray
import java.util.UUID
internal fun debugInteger(value:Any?):Long?=(value as? Number)?.toString()?.toLongOrNull()?.takeIf{it in Int.MIN_VALUE.toLong()..Int.MAX_VALUE.toLong()}
internal fun debugStop(value:Any?):String? {
 val text=(value as? Number)?.toString() ?: return null
 if(!text.matches(Regex("0|[1-9][0-9]{0,19}")))return null
 return text.takeIf{java.math.BigInteger(it)<=java.math.BigInteger("18446744073709551615")}
}
internal fun debugHash(text:String)=java.security.MessageDigest.getInstance("SHA-256").digest(text.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}
internal fun nativeCatalogNotices(catalog:JSONObject):List<String> {
 if(!catalog.has("notices")||catalog.isNull("notices"))return emptyList()
 val rows=catalog.optJSONArray("notices") ?: return listOf("Launch input notices could not be read. Inspect the current launch catalog before relying on complete input coverage.")
 return (0 until minOf(rows.length(),64)).map{(rows.opt(it) as? String)?.take(4096) ?: "An unreadable launch input notice was reported."} + if(rows.length()>64)listOf("More launch input notices are available from the workspace server.") else emptyList()
}
data class NativeDebugScope(val session:String,val stop:String,val thread:Long,val frame:Long){
 fun matches(event:JSONObject)=event.optString("session")==session&&debugStop(event.opt("stop"))==stop&&debugInteger(event.opt("thread"))==thread&&debugInteger(event.opt("frame"))==frame
 fun put(body:JSONObject):JSONObject=body.put("session",session).put("stop",java.math.BigInteger(stop)).put("thread",thread).put("frame",frame)
}
data class NativeSourceSnapshot(val identity:String,val path:String,val text:String,val saved:Boolean,val writable:Boolean)
data class NativeBreakpointIntent(val source:NativeSourceSnapshot,val revision:Long,val marks:List<JSONObject>,val hash:String?=null)
data class NativeBreakpointRequest(val identity:String,val path:String,val text:String,val revision:Long,val hash:String,val session:String,val request:String)
data class NativeBreakpointReceipt(val request:NativeBreakpointRequest,val marks:List<JSONObject>)
class NativeBreakpointModel {
 var refusals by mutableStateOf<Map<String,String>>(emptyMap());private set
 var intents by mutableStateOf<Map<String,NativeBreakpointIntent>>(emptyMap());private set
 var receipts by mutableStateOf<Map<String,NativeBreakpointReceipt>>(emptyMap());private set
 private val pending=mutableMapOf<String,NativeBreakpointRequest>();private var revision=0L
 fun matches(intent:NativeBreakpointIntent,source:NativeSourceSnapshot)=intent.source.identity==source.identity&&intent.source.path==source.path&&intent.source.text==source.text&&source.writable
 fun toggle(source:NativeSourceSnapshot,line:Int,condition:String?=null):NativeBreakpointIntent {
  require(source.writable&&line>=0&&line<source.text.split('\n').size){"Choose a writable source line before setting a breakpoint."}
  val previous=intents[source.identity]
  require(previous==null||matches(previous,source)){"Breakpoint intentions belong to older text or another address. Stop and discard those stale intentions before choosing new positions."}
  val requested=receipts[source.identity]?.takeIf{it.request.revision==previous?.revision}?.marks?.firstOrNull{it.optInt("line",-1)==line}?.optInt("requested_line",line) ?: line
  val marks=previous?.marks.orEmpty().map{JSONObject(it.toString())}.toMutableList();val index=marks.indexOfFirst{it.optInt("line",-1)==requested}
  if(index>=0){if(condition==null)marks.removeAt(index) else marks[index]=JSONObject().put("line",requested).put("condition",condition)}else marks.add(JSONObject().put("line",line).apply{if(condition!=null)put("condition",condition)})
  val next=NativeBreakpointIntent(source,++revision,marks,previous?.hash);refusals=refusals-source.identity;intents=intents+(source.identity to next);receipts=receipts-source.identity;pending.remove(source.identity);return next
 }
 fun stamp(identity:String,expectedRevision:Long,hash:String):NativeBreakpointIntent? {
  val old=intents[identity]?.takeIf{it.revision==expectedRevision} ?: return null
  require(hash.matches(Regex("[0-9a-f]{64}"))){"Invalid breakpoint source stamp. Prepare the source again."}
  return old.copy(hash=hash).also{intents=intents+(identity to it)}
 }
 fun request(source:NativeSourceSnapshot,session:String,id:String):NativeBreakpointRequest? {
  val intent=intents[source.identity]?.takeIf{matches(it,source)&&source.saved} ?: return null
  val hash=intent.hash ?: return null
  return NativeBreakpointRequest(source.identity,source.path,source.text,intent.revision,hash,session,id).also{pending[source.identity]=it;refusals=refusals-source.identity;receipts=receipts-source.identity}
 }
 fun admit(source:NativeSourceSnapshot,event:JSONObject,session:String,request:String,hash:String):Boolean {
  val expected=pending[source.identity] ?: return false;val intent=intents[source.identity] ?: return false
  if(!matches(intent,source)||!source.saved||intent.revision!=expected.revision||expected.session!=session||expected.request!=request||expected.hash!=hash||expected.path!=source.path)return false
  val rows=event.optJSONArray("breakpoints") ?: return false
  if(rows.length()>4096)return false
  val marks=(0 until rows.length()).map{rows.optJSONObject(it) ?: return false}
  if(marks.any{debugInteger(it.opt("line"))?.let{line->line<0}!=false||it.has("moved_to")&&!it.isNull("moved_to")&&debugInteger(it.opt("moved_to"))?.let{line->line<0}!=false})return false
  val requested=intent.marks.map{it.getInt("line")}.toSet()
  if(marks.map{it.getInt("line")}.toSet().size!=marks.size||marks.any{it.getInt("line") !in requested||it.optInt("moved_to",it.getInt("line"))>=source.text.split('\n').size})return false
  val merged=marks.map{state->JSONObject(intent.marks.firstOrNull{it.optInt("line",-1)==state.optInt("line",-2)}?.toString() ?: "{}").also{mark->mark.put("requested_line",state.getInt("line"));state.keys().forEach{key->mark.put(key,state.get(key))};if(!state.isNull("moved_to"))mark.put("line",state.getInt("moved_to"))}}
  receipts=receipts+(source.identity to NativeBreakpointReceipt(expected,merged));pending.remove(source.identity);return true
 }
 fun refuse(session:String,request:String,message:String):Boolean {
  val expected=pending.values.firstOrNull{it.session==session&&it.request==request} ?: return false
  val intent=intents[expected.identity]?.takeIf{it.revision==expected.revision} ?: return false
  // A provider failure leaves the applied set uncertain. Other held receipts
  // cannot grant authority again, even if their source text still matches.
  pending.clear();refusals=intents.keys.associateWith{message};receipts=mapOf(expected.identity to NativeBreakpointReceipt(expected,intent.marks.map{JSONObject(it.toString()).put("state","refused").put("message",message)}));return true
 }
 fun rows(source:NativeSourceSnapshot,session:String?,runHash:String?):List<JSONObject> {
  val intent=intents[source.identity]?.takeIf{matches(it,source)&&source.saved} ?: return emptyList()
  if(session!=null&&intent.hash!=runHash)return emptyList()
  val receipt=receipts[source.identity]?.takeIf{session!=null&&it.request.session==session&&it.request.revision==intent.revision&&it.request.hash==runHash}
  return receipt?.marks ?: intent.marks.map{JSONObject(it.toString()).put("state","pending")}
 }
 fun notice(source:NativeSourceSnapshot,session:String?,runHash:String?):String {
  val intent=intents[source.identity] ?: return ""
  if(!matches(intent,source)||!source.saved)return "${intent.marks.size} breakpoint intentions retained from a different source snapshot. Positional marks and provider effects are unavailable; save/review the source or stop and discard stale intentions."
  if(session!=null&&intent.hash!=runHash)return "Breakpoint intentions are readable, but this source is not bound to the running snapshot. Restart from saved source before applying positions."
  return refusals[source.identity]?.let{"Breakpoint update refused: $it Review the current source and explicitly apply the desired condition again."} ?: "${intent.marks.size} source-bound breakpoint intentions. Adapter binding requires a matching receipt."
 }
 fun discard(identity:String){refusals=refusals-identity;intents=intents-identity;receipts=receipts-identity;pending.remove(identity)}
 fun readdress(source:NativeSourceSnapshot){val old=intents[source.identity] ?: return;if(old.source.path!=source.path&&old.source.text==source.text&&source.writable){intents=intents+(source.identity to old.copy(source=source,revision=++revision));refusals=refusals-source.identity;receipts=receipts-source.identity;pending.remove(source.identity)}}
 fun adoptStart(id:String,session:String){pending.keys.toList().forEach{identity->val request=pending.getValue(identity);if(request.request==id&&request.session=="pending-start")pending[identity]=request.copy(session=session)}}
 fun retireSession(){refusals=emptyMap();receipts=emptyMap();pending.clear()}
}
data class NativeSourceRoute(val identity:String,val from:String?,val to:String?,val text:String?,val saved:Boolean)
data class NativeRuntimeSource(val runtime:String,val identity:String,val text:String,val hash:String,val address:String?,val bound:Boolean)
class NativeSourceLineage {
 var epoch by mutableStateOf(0L);private set
 private var launched=0L
 var sources by mutableStateOf<Map<String,NativeRuntimeSource>>(emptyMap());private set
 fun clear(){sources=emptyMap();launched=epoch}
 fun begin(captured:List<Pair<NativeSourceSnapshot,String>>){launched=epoch;sources=captured.associate{(source,hash)->source.path to NativeRuntimeSource(source.path,source.identity,source.text,hash,source.path,true)}}
 fun matches(source:NativeSourceSnapshot):Boolean=source.saved&&sources.values.any{it.bound&&it.identity==source.identity&&it.address==source.path&&it.runtime==source.path&&it.text==source.text}
 fun hash(source:NativeSourceSnapshot):String?=sources.values.firstOrNull{it.bound&&it.identity==source.identity&&it.address==source.path&&it.runtime==source.path&&it.text==source.text&&source.saved}?.hash
 fun learn(runtime:String,source:NativeSourceSnapshot,hash:String,expectedEpoch:Long):Boolean {
  if(epoch!=expectedEpoch||runtime!=source.path||!source.saved)return false
  val known=sources[runtime]
  if(known!=null)return known.bound&&known.identity==source.identity&&known.address==source.path&&known.text==source.text&&known.hash==hash
  if(epoch!=launched)return false
  sources=sources+(runtime to NativeRuntimeSource(runtime,source.identity,source.text,hash,source.path,true));return true
 }
 fun transition(routes:List<NativeSourceRoute>){
  if(routes.isEmpty())return
  epoch++
  // A namespace transition retires all positional authority. Only this guarded
  // receipt can prove an original identity back at its immutable runtime address.
  sources=sources.mapValues{(_,source)->
   val route=routes.firstOrNull{it.identity==source.identity}
   if(route==null)source.copy(bound=false) else source.copy(address=route.to,bound=route.to==source.runtime&&route.text==source.text&&route.saved)
  }
 }
}

data class NativeVariableNode(val id:String,val scope:NativeDebugScope,val name:String,val value:String,val reference:Long?,val depth:Int)
data class NativeVariableExpansion(val pending:String?=null,val children:List<NativeVariableNode> = emptyList(),val message:String="")
data class NativeVariableRequest(val node:NativeVariableNode,val request:String,val inspection:String)
class NativeVariableTree {
 var notice by mutableStateOf("");private set
 var scope by mutableStateOf<NativeDebugScope?>(null);private set
 var groups by mutableStateOf<Map<String,List<NativeVariableNode>>>(emptyMap());private set
 var expansions by mutableStateOf<Map<String,NativeVariableExpansion>>(emptyMap());private set
 private var inspection:String?=null
 private val pending=mutableMapOf<String,NativeVariableRequest>()
 fun open():String=UUID.randomUUID().toString().also{inspection=it;pending.clear();expansions=emptyMap()}
 fun close(token:String){if(inspection==token){inspection=null;pending.clear();expansions=emptyMap()}}
 fun clear(){notice="";scope=null;groups=emptyMap();expansions=emptyMap();pending.clear()}
 private fun nodes(scope:NativeDebugScope,rows:List<JSONObject>,depth:Int)=rows.map{row->NativeVariableNode(UUID.randomUUID().toString(),scope,row.optString("name"),row.optString("value"),debugInteger(row.opt("variables_reference"))?.takeIf{it>0},depth)}
 fun replace(scope:NativeDebugScope?,key:String,rows:List<JSONObject>){if(scope==null){clear();return};if(this.scope!=scope)clear();this.scope=scope;if(rows.size>4096||groups.filterKeys{it!=key}.values.sumOf{it.size}+rows.size>16384){groups=groups-key;pending.clear();expansions=emptyMap();notice="Too many variables; evaluate a named value.";return};notice="";groups=groups+(key to nodes(scope,rows,0));pending.clear();expansions=emptyMap()}
 fun remove(key:String){groups=groups-key;pending.clear();expansions=emptyMap()}
 private fun visible(node:NativeVariableNode):Boolean {
  fun has(rows:List<NativeVariableNode>):Boolean=rows.any{it===node||expansions[it.id]?.children?.let{children->has(children)}==true}
  return groups.values.any{has(it)}
 }
 fun expand(node:NativeVariableNode,token:String):NativeVariableRequest? {
  if(token!=inspection||node.scope!=scope||!visible(node)||node.reference==null)return null
  if(node.depth>=12||groups.values.sumOf{it.size}+expansions.values.sumOf{it.children.size}>=16384){expansions=expansions+(node.id to NativeVariableExpansion(message="Expansion limit reached. Evaluate a narrower expression."));return null}
  val request=NativeVariableRequest(node,UUID.randomUUID().toString(),token);pending.entries.removeAll{it.value.node.id==node.id};pending[request.request]=request;expansions=expansions+(node.id to NativeVariableExpansion(pending=request.request,message="Reading children…"));return request
 }
 fun collapse(node:NativeVariableNode,token:String){
  if(token!=inspection)return
  val discarded=mutableSetOf<String>();fun collect(id:String){if(!discarded.add(id))return;expansions[id]?.children?.forEach{collect(it.id)}};collect(node.id)
  expansions=expansions.filterKeys{it !in discarded};pending.entries.removeAll{it.value.node.id in discarded||!visible(it.value.node)}
 }
 fun admit(event:JSONObject):Boolean {
  val id=event.optString("request_id");val request=pending[id] ?: return false;val node=request.node
  if(inspection!=request.inspection||scope!=node.scope||!visible(node)||!node.scope.matches(event)||debugInteger(event.opt("reference"))!=node.reference||expansions[node.id]?.pending!=id)return false
  val rows=event.optJSONArray("variables") ?: return false
  if(rows.length()>4096||groups.values.sumOf{it.size}+expansions.values.sumOf{it.children.size}+rows.length()>16384)return refuse(id,"Children exceed the supported reading. Evaluate a narrower expression.")
  val values=(0 until rows.length()).map{rows.optJSONObject(it) ?: return refuse(id,"Malformed child reading. Expand again explicitly.")}
  pending.remove(id);expansions=expansions+(node.id to NativeVariableExpansion(children=nodes(node.scope,values,node.depth+1),message=if(values.isEmpty())"No reported children." else ""));return true
 }
 fun refuse(id:String,message:String):Boolean {val request=pending.remove(id) ?: return false;if(inspection!=request.inspection||scope!=request.node.scope||!visible(request.node))return false;expansions=expansions+(request.node.id to NativeVariableExpansion(message=message));return true}
}
