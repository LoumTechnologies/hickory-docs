@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
package com.hickorydocs.simple
import androidx.compose.runtime.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import org.json.JSONObject
import java.net.URLEncoder

data class SourceSaveAttempt(val engine:Engine,val path:String,val source:String,val base:String,val hash:String,val guard:String,val id:String=java.util.UUID.randomUUID().toString().replace("-","")){
 fun matches(engine:Engine,path:String,source:String,base:String,hash:String,guard:String)=this.engine===engine&&this.path==path&&this.source==source&&this.base==base&&this.hash==hash&&this.guard==guard
}
data class SourceSaveConfirmation(val hash:String,val guard:String,val notice:String)
suspend fun Engine.saveSourceWithHistory(file:FileNode,attempt:SourceSaveAttempt):SourceSaveConfirmation {
 check(attempt.engine===this&&file.path==attempt.path&&attempt.guard.isNotEmpty()){"Save belongs to a different source or connection."}
 val body=JSONObject().put("version",1).put("request_id",attempt.id).put("save_guard",attempt.guard)
 val route=if(file.docID==null){body.put("path",file.path).put("content",attempt.source).put("base_hash",attempt.hash);"/api/source-history/file"}else{body.put("source",attempt.source).put("expected",JSONObject().put("source",attempt.base).put("disk",attempt.base));"/api/source-history/docs/${file.docID}"}
 val raw=request("PUT",route,body);check(raw.get("version")==1&&raw.get("history_protocol")==1&&raw.getString("path")==attempt.path&&raw.getString("request_id")==attempt.id&&raw.getString("source_hash")==debugHash(attempt.source)&&raw.getString("save_guard").isNotEmpty()){"Save confirmation belongs to a different intention or physical source. Inspect local history before retrying."}
 val history=raw.getJSONObject("history");val available=history.get("available") as? Boolean ?: error("Malformed history availability.");val outcome=history.getString("outcome");check(outcome=="published"){"Save publication is not confirmed. The prior baseline and physical guard are retained."}
 return SourceSaveConfirmation(if(file.docID==null)raw.getString("hash") else attempt.hash,raw.getString("save_guard"),if(available)"" else history.optString("reason","Source saved; durable local history is unavailable. Inspect current source before continuing."))
}
data class SourceHistoryEntry(val id:String,val fingerprint:String,val outcome:String,val before:ResourceSaved,val after:ResourceSaved,val available:Boolean,val reason:String?,val at:Long=0)
data class SourceHistoryReading(val namespace:String,val workspace:String,val path:String,val current:ResourceSaved,val identity:String,val entries:List<SourceHistoryEntry>,val notices:List<String>,val restoreTarget:String)
fun sourceHistoryReading(raw:JSONObject,path:String,address:String):SourceHistoryReading{
 fun saved(value:JSONObject,missing:Boolean=false):ResourceSaved {val result=resourceSavedMap(JSONObject().put("entry",value)).getValue("entry");if(missing&&result.kind=="missing")return result;check(result.kind=="file"&&result.text!=null&&debugHash(result.text)==result.sha256){"Malformed historical source bytes."};return result}
 check(raw.get("version")==1&&raw.get("history_protocol")==1&&raw.getString("path")==path&&raw.getString("namespace")==address&&raw.getString("workspace").isNotEmpty()&&raw.getString("current_identity").isNotEmpty()){"Local history belongs to another source, workspace or protocol."}
 val rows=raw.getJSONArray("entries");check(rows.length()<=10000){"History exceeds bounded reading."};val seen=mutableSetOf<String>();val entries=(0 until rows.length()).map{val value=rows.getJSONObject(it);val id=value.getString("id");val fp=value.getString("fingerprint");val outcome=value.getString("outcome");val time=value.get("at");check((time is Int||time is Long)&&(time as Number).toLong()>=0){"Malformed save timestamp."};val available=value.get("available") as? Boolean ?: error("Malformed history availability.")
  check(id.matches(Regex("[a-f0-9]{32}"))&&seen.add(id)&&fp.matches(Regex("[a-f0-9]{64}"))&&outcome in listOf("prepared","published","failed","unknown")&&(!available||outcome=="published")){"History evidence identity is malformed."};SourceHistoryEntry(id,fp,outcome,saved(value.getJSONObject("before"),true),saved(value.getJSONObject("after"),true),available,if(value.isNull("reason"))null else value.getString("reason"),(time as Number).toLong())}
 val current=saved(raw.getJSONObject("current"),true);val target=raw.getString("restore_target");check(target==(if(current.kind=="missing")"missing-source-draft" else "current-source-draft")){"Unsupported history restoration target."};val notices=raw.getJSONArray("notices");return SourceHistoryReading(address,raw.getString("workspace"),path,current,raw.getString("current_identity"),entries,(0 until notices.length()).map{notices.getString(it)},target)
}
class SourceHistoryModel(private val transport:(suspend(String,String,JSONObject?)->JSONObject)?=null){
 var showing by mutableStateOf(false);private set
 var busy by mutableStateOf(false);private set
 var notice by mutableStateOf("");private set
 var reading by mutableStateOf<SourceHistoryReading?>(null);private set
 var selected by mutableStateOf<SourceHistoryEntry?>(null);private set
 var side by mutableStateOf("before");private set
 private var ticket=0L
 private data class Capture(val w:Workspace,val engine:Engine,val epoch:java.util.UUID,val address:String,val buffer:DocumentBuffer,val path:String,val source:String,val revision:Long,val selection:androidx.compose.ui.text.TextRange,val intent:Int){
  fun current()=w.engine===engine&&w.sourceNavigationEpoch==epoch&&engine.sourceRootAddress==address&&w.currentBuffer===buffer&&w.buffers[path]===buffer&&buffer.file.path==path&&!buffer.resourceDeleted&&!buffer.readOnly&&!buffer.saving&&buffer.state.composition==null&&buffer.multiRanges.size<2&&buffer.sourceRevision==revision&&buffer.text==source&&buffer.state.selection==selection&&buffer.snippets.intent==intent
 }
 private var capture:Capture?=null
 private suspend fun call(engine:Engine,method:String,path:String,body:JSONObject?=null)=transport?.invoke(method,path,body) ?: engine.request(method,path,body)
 fun close(){ticket++;showing=false;busy=false;selected=null;capture=null}
 suspend fun open(w:Workspace){close();showing=true;reading=null;val id=ticket;busy=true
  try{val b=w.currentBuffer ?: error("Open readable source before local history.");val engine=w.engine ?: error("Connect to the source workspace.");val address=engine.sourceRootAddress ?: error("Workspace address is unavailable. Refresh Files before local history.");check(!b.resourceDeleted&&!b.saving){"Source is deleted or saving; retain its text and inspect recovery."}
   val held=Capture(w,engine,w.sourceNavigationEpoch,address,b,b.file.path,b.text,b.sourceRevision,b.state.selection,b.snippets.intent);capture=held
   val raw=call(engine,"GET","/api/source-history?path=${URLEncoder.encode(held.path,"UTF-8")}");if(ticket!=id||!showing)return;check(w.currentBuffer===b&&w.buffers[held.path]===b&&w.sourceNavigationEpoch==held.epoch&&w.engine===engine&&b.file.path==held.path){"Source lifetime changed. Read history again."};reading=sourceHistoryReading(raw,held.path,address);notice="Local saves retained on this engine machine. Choose a side to compare; Restore changes only the current draft."
  }catch(e:CancellationException){throw e}catch(e:Exception){if(id==ticket)notice=e.message.orEmpty()}finally{if(id==ticket)busy=false}
 }
 fun choose(entry:SourceHistoryEntry,side:String){if(busy||side !in listOf("before","after")||reading?.entries?.contains(entry)!=true)return;selected=entry;this.side=side;notice=entry.reason ?: "Preview only. Current unsaved text is retained until explicit Restore to draft."}
 val preview get()=selected?.let{if(side=="before")it.before.text else it.after.text}
 suspend fun restore():Boolean{val held=capture;val value=reading;val entry=selected;val source=preview
  if(busy||held==null||value==null||entry==null||source==null||!entry.available||!held.current()||(value.current.kind=="file"&&value.current.readonly!=false)){notice="History or editor is stale or read-only. Current draft retained.";return false};val id=++ticket;val chosen=side;busy=true;selected=null
  try{val body=JSONObject().put("version",1).put("namespace",value.namespace).put("workspace",value.workspace).put("path",value.path).put("id",entry.id).put("fingerprint",entry.fingerprint).put("current_identity",value.identity).put("current_hash",if(value.current.kind=="missing")"missing" else value.current.sha256).put("draft_hash",debugHash(held.source)).put("side",chosen)
   val raw=call(held.engine,"POST","/api/source-history/restore-draft",body);check(id==ticket&&showing&&held.current()&&raw.get("version")==1&&raw.get("history_protocol")==1){"History or source changed while checking restoration."};for(key in listOf("namespace","workspace","path","id","fingerprint","current_identity","current_hash","draft_hash","side"))check(raw.getString(key)==body.getString(key)){"Foreign historical draft refused."};check(raw.getString("source")==source&&raw.getString("consequences")=="unknown"&&raw.getString("restore_target")==value.restoreTarget&&raw.get("source_effects")==false){"Historical draft bytes changed or unverified consequences claimed."}
   val plan=ChangePlan("Restore local history to draft",listOf(SourcePatch(held.path,held.source,source,listOf(SourceReplacement(0,held.source.length,source)))))
   check(held.buffer.applySourceEdits(listOf(NativeSourceEdit(0,held.source.length,source)),held.source)){"Native source transaction refused. Draft retained."};if(value.current.kind=="missing"){held.buffer.unmaterializedHistoryDraft=true;held.buffer.resourceDeleted=true;held.buffer.saveGuard=null;held.buffer.saveGuardReason="Unmaterialized restored draft. Use guarded Save As to publish to an unused path."};held.w.changes.recordDraftApplication(plan,held.w);notice=raw.getString("notice");close();held.buffer.focusSource?.invoke();return true
  }catch(e:CancellationException){throw e}catch(e:Exception){if(id==ticket)notice=e.message.orEmpty();return false}finally{if(id==ticket)busy=false}
 }
}
@Composable fun SourceHistoryView(w:Workspace){val m=w.sourceHistory;val scope=rememberCoroutineScope();var page by remember{mutableIntStateOf(0)}
 Column(Modifier.padding(12.dp).widthIn(max=600.dp).heightIn(max=640.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(6.dp)){
  Text("Local source history");if(m.notice.isNotEmpty())Text(m.notice,fontSize=12.sp);m.reading?.let{value->value.notices.forEach{Text(it,fontSize=12.sp)};value.entries.forEach{entry->Text("${java.text.DateFormat.getDateTimeInstance().format(java.util.Date(entry.at))} · ${entry.outcome} · ${entry.id}",fontSize=12.sp);Row{WithDisabledReason(if (m.busy) "An operation is already in progress." else null) { available -> TextButton(modifier=Modifier.semantics{contentDescription="Read before source save ${entry.id} at ${entry.at}"},enabled = available,onClick={m.choose(entry,"before");page=0}){Text("Before save")} };WithDisabledReason(if (m.busy) "An operation is already in progress." else null) { available -> TextButton(modifier=Modifier.semantics{contentDescription="Read after source save ${entry.id} at ${entry.at}"},enabled = available,onClick={m.choose(entry,"after");page=0}){Text("After save")} }}}}
  if(m.selected!=null&&m.preview==null)Text("This historical side is absent. Removing a source requires guarded resource recovery; no empty text was substituted.",fontSize=12.sp)
  m.preview?.let{text->val count=((text.length+7999)/8000).coerceAtLeast(1);val current=page.coerceIn(0,count-1);Text(text.substring(current*8000,((current+1)*8000).coerceAtMost(text.length)),fontSize=12.sp);Row{WithDisabledReason(if (current <= 0) "You’re on the first page." else null) { available -> TextButton(enabled = available,onClick={page=current-1}){Text("Previous text")} };Text("${current+1}/$count");WithDisabledReason(if (current >= count-1) "You’re on the last page." else null) { available -> TextButton(enabled = available,onClick={page=current+1}){Text("Next text")} }};WithDisabledReason(if (m.busy) "An operation is already in progress." else if (m.selected?.available != true) "This save record has no retained source text." else null) { available -> TextButton(enabled = available,onClick={scope.launch{m.restore()}}){Text("Restore to draft · not saved")} }}
  TextButton(onClick={m.close();w.currentBuffer?.focusSource?.invoke()}){Text("Close local history")}
 }
}
