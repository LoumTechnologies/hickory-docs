@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
package com.hickorydocs.simple
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONObject
import androidx.compose.ui.text.TextRange

private fun sourceCreationRoster(w:Workspace)=(w.buffers.values+w.retainedResourceEditors.values+w.retainedResourceTabs.values).distinct()
data class SourceCreationInput(val buffer:DocumentBuffer,val path:String,val text:String,val revision:Long,val base:String,val hash:String,val guard:String?,val file:FileNode,val readonly:Boolean,val deleted:Boolean,val selection:TextRange)
data class SourceCreationAttempt(val engine:Engine,val workspace:String,val namespace:String,val epoch:java.util.UUID,val actor:DocumentBuffer,val tab:String?,val destination:String,val source:String,val hash:String,val id:String,val inputs:List<SourceCreationInput>,val body:String,val addresses:Map<String,DocumentBuffer>,val retained:Map<String,DocumentBuffer>,val tabs:Map<String,DocumentBuffer>){
 fun current(w:Workspace,source:DocumentBuffer):Boolean {
  if(w.engine!==engine||engine.sourceWorkspaceID!=workspace||engine.sourceRootAddress!=namespace||w.sourceNavigationEpoch!=epoch||source!==actor||w.active!=tab||w.bufferForTab(tab)!==actor||source.text!=this.source)return false
  fun same(current:Map<String,DocumentBuffer>,captured:Map<String,DocumentBuffer>)=current.size==captured.size&&captured.all{(path,b)->current[path]===b}
  if(!same(w.buffers,addresses)||!same(w.retainedResourceEditors,retained)||!same(w.retainedResourceTabs,tabs))return false
  val live=sourceCreationRoster(w);if(live.toSet()!=inputs.map{it.buffer}.toSet())return false
  return inputs.all{s->val b=s.buffer;b.file.path==s.path&&b.file===s.file&&b.text==s.text&&b.sourceRevision==s.revision&&b.base==s.base&&b.hash==s.hash&&b.saveGuard==s.guard&&b.readOnly==s.readonly&&b.resourceDeleted==s.deleted&&b.state.selection==s.selection&&b.state.composition==null&&(b===actor||!b.saving)}
 }
}
data class SourceCreated(val path:String,val source:String,val hash:String,val guard:String,val doc:String?,val historyNotice:String)
fun sourceCreated(raw:JSONObject,attempt:SourceCreationAttempt):SourceCreated{
 check(raw.get("version")==1&&raw.get("history_protocol")==1&&raw.getString("workspace")==attempt.workspace&&raw.getString("namespace")==attempt.namespace&&raw.getString("path")==attempt.destination&&raw.getString("request_id")==attempt.id&&raw.getString("source_hash")==attempt.hash&&raw.getString("source")==attempt.source&&raw.getString("save_guard").isNotEmpty()){"Creation reply does not confirm this exact source and namespace. Original editor retained; inspect the destination and Local history."}
 val history=raw.getJSONObject("history");val available=history.get("available") as? Boolean ?: error("Malformed creation history availability.");check(history.getString("outcome")=="published"){"Creation is not confirmed published. Original editor and request retained."}
 val doc=when(val value=raw.opt("doc_id")){null,JSONObject.NULL->null;is String->value.takeIf{it.isNotEmpty()} ?: error("Malformed created document identity.");else->error("Malformed created document identity.")}
 return SourceCreated(attempt.destination,attempt.source,raw.getString("hash"),raw.getString("save_guard"),doc,if(available)"" else history.optString("reason","Source created; durable Local history is unavailable."))
}
private fun creationByteCost(text:String,budget:Long):Long{
 check(text.length.toLong()<=budget){"Source creation capture exceeds its bounded UTF8 budget."};var bytes=0L;var index=0
 while(index<text.length){val char=text[index];val cost=when{char.isHighSurrogate()->{check(index+1<text.length&&text[index+1].isLowSurrogate()){"Finish invalid source input before Save As."};index+=2;4};char.isLowSurrogate()->error("Finish invalid source input before Save As.");else->{index++;when{char.code<=127->1;char.code<=2047->2;else->3}}};bytes+=cost;check(bytes<=budget){"Source creation capture exceeds its bounded UTF8 budget."}}
 return bytes
}
internal suspend fun Workspace.createSourceCopy(source:DocumentBuffer,path:String):DocumentBuffer{
 val server=engine ?: error("Connect before Save As.");val workspace=server.sourceWorkspaceID ?: error("Creation authority is unavailable. Retain the draft and refresh Files before Save As.");val namespace=server.sourceRootAddress ?: error("Workspace address is unavailable. Retain the draft and refresh Files.")
 check(path.isNotEmpty()&&buffers[path]==null){"A retained editor already owns this destination. Choose an unused path; original text is retained."}
 val attempt=source.creationAttempt?.takeIf{it.destination==path&&it.current(this,source)} ?: run{
  val epoch=sourceNavigationEpoch;val tab=active;val addresses=buffers.toMap();val retained=retainedResourceEditors.toMap();val tabs=retainedResourceTabs.toMap();check(addresses.all{(path,b)->path==b.file.path}){"A retained source address aliases another editor. Reconcile it before Save As."};val inputs=sourceCreationRoster(this).map{b->check(b.state.composition==null&&(b===source||!b.saving)){"Finish composition and other source saves before Save As."};SourceCreationInput(b,b.file.path,b.text,b.sourceRevision,b.base,b.hash,b.saveGuard,b.file,b.readOnly,b.resourceDeleted,b.state.selection)}
  check(inputs.size<=4096&&inputs.any{it.buffer===source}){"Save As requires one retained source editor and a bounded complete roster."};val submitted=source.text;val id=java.util.UUID.randomUUID().toString().replace("-","")
  val built=withContext(Dispatchers.Default){
   var bytes=creationByteCost(submitted,8L*1024*1024)+creationByteCost(path,8192);val ownership=JSONObject();val guards=JSONObject();val unaddressed=JSONObject()
   fun charge(text:String){val cost=creationByteCost(text,8L*1024*1024);check(cost<=32L*1024*1024-bytes){"Retained source creation context exceeds 32 MiB. Reconcile or reduce the captured drafts first."};bytes+=cost}
   for(input in inputs){if(input.file.generatedBy!=null)continue
    val named=!input.path.startsWith("Untitled-")&&!input.path.startsWith('/')&&!input.deleted
    if(named){val guard=input.guard?.takeIf{it.isNotEmpty()} ?: error("A named construction source has no captured physical baseline: ${input.path}. Preserve it and reopen its exact source before Save As.");check(!ownership.has(input.path)){"Construction source aliases another retained editor. Reconcile it before Save As."};charge(input.path);charge(guard);charge(input.text);ownership.put(input.path,input.text);guards.put(input.path,guard)}
    else if(!named&&input.buffer!==source){charge(input.buffer.debugIdentity);charge(input.text);unaddressed.put(input.buffer.debugIdentity,input.text)}
   }
   val capture=JSONObject().put("context",id).put("edit",JSONObject().put("changes",JSONObject())).put("drafts",JSONObject()).put("project_entry",JSONObject.NULL).put("ownership_sources",ownership)
   val body=JSONObject().put("version",1).put("workspace",workspace).put("path",path).put("source",submitted).put("request_id",id).put("ownership",capture).put("saved_guards",guards).put("unaddressed_sources",unaddressed)
   debugHash(submitted) to body.toString()
  }
  SourceCreationAttempt(server,workspace,namespace,epoch,source,tab,path,submitted,built.first,id,inputs,built.second,addresses,retained,tabs).also{check(it.current(this,source)){"Source or retained namespace changed before creation. Nothing was sent."};source.creationAttempt=it}
 }
 check(attempt.current(this,source)){"Source or retained construction inputs changed. Original editor retained."}
 val raw=withContext(Dispatchers.IO){server.request("POST","/api/source-history/create",JSONObject(attempt.body))}
 val result=sourceCreated(raw,attempt);check(attempt.current(this,source)&&buffers[path]==null){"Creation outcome belongs to an obsolete editor namespace. Original edits retained; inspect destination and Local history before another request."}
 val file=FileNode(path.substringAfterLast('/'),path,false,emptyList(),result.doc,null)
 val loaded=FileText(result.source,result.hash,result.guard)
 return adoptCreatedSource(source,file,loaded,server).also{it.error=result.historyNotice}

}
