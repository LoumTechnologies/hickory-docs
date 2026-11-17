<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retain ordinary Save identity and restore historical text to a native draft

For Android maintainers following Local source history from the existing menu into the retained editor.

The loaded baseline retains its opaque physical save guard. Ordinary Save uses only the strict versioned history route and an immutable submitted body, baseline, connection, path and request ID. Lost replies retain that exact intention. Equal text or a fresh history read never supplies a replacement guard, and missing authority leaves the draft readable and editable. A confirmation must name the exact request, source and hash before the baseline or guard changes. Resource publication supplies its own confirmed guard through the existing receipt path; no renderer derives physical identity.

History readings remain separate from publication. The namespace is the exact canonical root_path returned by Files, not the display root label. Entries validate saved hashes and evidence identity. An exact current Missing receipt can restore confirmed historical File text only as an unmaterialized draft. It clears save authority and routes explicit Save to guarded Save As. A historical Missing side remains explicit absence; it is never coerced to empty text or offered as a draft deletion. The restore context binds the current buffer, connection, namespace, source revision, selection and snippet intent. Editing followed by Undo cannot revive it. The engine rechecks physical lineage and ownership before returning one historical side; all echoes and exact bytes must match before one coordinated native source transaction. Restoration changes only the draft, preserves the editor and its undo history, and labels compiler/generated consequences unknown. Closing the floating reading retires held replies.

<ui:copy id="source-history-android">
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
  Text("Local source history");if(m.notice.isNotEmpty())Text(m.notice,fontSize=12.sp);m.reading?.let{value->value.notices.forEach{Text(it,fontSize=12.sp)};value.entries.forEach{entry->Text("${java.text.DateFormat.getDateTimeInstance().format(java.util.Date(entry.at))} · ${entry.outcome} · ${entry.id}",fontSize=12.sp);Row{TextButton(modifier=Modifier.semantics{contentDescription="Read before source save ${entry.id} at ${entry.at}"},enabled=!m.busy,onClick={m.choose(entry,"before");page=0}){Text("Before save")};TextButton(modifier=Modifier.semantics{contentDescription="Read after source save ${entry.id} at ${entry.at}"},enabled=!m.busy,onClick={m.choose(entry,"after");page=0}){Text("After save")}}}}
  if(m.selected!=null&&m.preview==null)Text("This historical side is absent. Removing a source requires guarded resource recovery; no empty text was substituted.",fontSize=12.sp)
  m.preview?.let{text->val count=((text.length+7999)/8000).coerceAtLeast(1);val current=page.coerceIn(0,count-1);Text(text.substring(current*8000,((current+1)*8000).coerceAtMost(text.length)),fontSize=12.sp);Row{TextButton(enabled=current>0,onClick={page=current-1}){Text("Previous text")};Text("${current+1}/$count");TextButton(enabled=current<count-1,onClick={page=current+1}){Text("Next text")}};TextButton(enabled=!m.busy&&m.selected?.available==true,onClick={scope.launch{m.restore()}}){Text("Restore to draft · not saved")}}
  TextButton(onClick={m.close();w.currentBuffer?.focusSource?.invoke()}){Text("Close local history")}
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SourceHistory.kt"><ui:paste select="#source-history-android"/></ui:file>

## Exercise refusal and draft restoration without claiming a physical UI run

The focused controller fixture serves the strict endpoints through the real HTTP transport. Lost and foreign confirmations leave the baseline untouched and reuse only the original exact request. Missing guards send no write. History restoration preserves native editor identity and its Undo/Redo while leaving the saved guard and baseline unchanged; revision changes, replacement editors, namespace mismatches and tampered historical bytes refuse. These are JVM controller/text-state witnesses, not rendered Compose or physical input/accessibility acceptance.

<ui:copy id="source-history-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import org.json.JSONArray
import kotlinx.coroutines.*
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class SourceHistoryTest {
 private fun saved(text:String)=JSONObject().put("kind","file").put("text",text).put("sha256",debugHash(text)).put("readonly",false).put("executable",false).put("mode",420)
 private fun reading()=JSONObject().put("version",1).put("history_protocol",1).put("namespace","/fixture").put("workspace","physical-root").put("path","a.txt").put("current",saved("saved")).put("current_identity","source-inode").put("restore_target","current-source-draft").put("notices",JSONArray()).put("entries",JSONArray().put(JSONObject().put("id","a".repeat(32)).put("fingerprint","b".repeat(64)).put("at",1).put("outcome","published").put("before",saved("old")).put("after",saved("saved")).put("available",true).put("reason",JSONObject.NULL)))
 private suspend fun host(block:suspend(Workspace,DocumentBuffer,java.util.concurrent.atomic.AtomicReference<String>,MutableList<JSONObject>)->Unit)=coroutineScope{
  val mode=java.util.concurrent.atomic.AtomicReference("normal");val saves=java.util.Collections.synchronizedList(mutableListOf<JSONObject>());val server=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0)
  server.createContext("/api"){exchange->var status=200;val route=exchange.requestURI.path;val request=exchange.requestBody.bufferedReader().readText().takeIf{it.isNotEmpty()}?.let{JSONObject(it)}
   val response=when{
    route=="/api/files"->JSONObject().put("root","Display only").put("root_path","/fixture").put("tree",JSONArray())
    route=="/api/source-history/file"->{val body=request!!;saves.add(body);if(mode.get()=="lost"){status=409;JSONObject().put("error","Unconfirmed network publication")}else{val source=body.getString("content");JSONObject().put("version",1).put("history_protocol",1).put("path",if(mode.get()=="foreign")"other.txt" else "a.txt").put("request_id",body.getString("request_id")).put("source_hash",debugHash(source)).put("hash",debugHash(source)).put("save_guard","new-physical-token").put("history",JSONObject().put("available",mode.get() !in listOf("failed","prepared","unknown","unavailable")).put("outcome",if(mode.get() in listOf("failed","prepared","unknown"))mode.get() else "published"))}}
    route=="/api/source-history/restore-draft"->{val body=request!!;JSONObject(body.toString()).put("history_protocol",1).put("source",if(mode.get()=="tampered")"foreign" else "old").put("restore_target",if(mode.get()=="missing")"missing-source-draft" else "current-source-draft").put("source_effects",false).put("consequences","unknown").put("notice","Restored source draft; consequences unknown.")}
    route=="/api/source-history"->reading().apply{if(mode.get()=="missing"){put("current",JSONObject().put("kind","missing"));put("restore_target","missing-source-draft")}}
    else->{status=400;JSONObject().put("error","Unexpected route: $route")}
   };val bytes=response.toString().toByteArray();exchange.sendResponseHeaders(status,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}
  };server.start();val w=Workspace(this);w.engine=Engine("http://127.0.0.1:${server.address.port}");w.engine!!.files();val file=FileNode("a.txt","a.txt",false,emptyList(),null,null);val b=DocumentBuffer(file,FileText("saved",debugHash("saved"),"original-physical-token"));w.buffers[file.path]=b;w.show(file.path)
  try{block(w,b,mode,saves)}finally{server.stop(0)}
 }
 @Test fun readingsBindNamespaceBytesAndConfirmedEvidence(){val raw=reading();val value=sourceHistoryReading(raw,"a.txt","/fixture");assertEquals("old",value.entries.single().before.text);assertEquals(1L,value.entries.single().at);raw.getJSONArray("entries").getJSONObject(0).getJSONObject("before").put("text","tampered");assertEquals("old",value.entries.single().before.text)
  for((key,bad) in listOf("version" to true,"history_protocol" to 1.5,"path" to "other.txt","namespace" to "/elsewhere"))assertTrue(runCatching{sourceHistoryReading(reading().put(key,bad),"a.txt","/fixture")}.isFailure)
  val created=reading();created.getJSONArray("entries").getJSONObject(0).put("before",JSONObject().put("kind","missing"));val creation=sourceHistoryReading(created,"a.txt","/fixture");assertEquals("missing",creation.entries.single().before.kind);assertNull(creation.entries.single().before.text);assertEquals("saved",creation.entries.single().after.text)
  val badTime=reading();badTime.getJSONArray("entries").getJSONObject(0).put("at",1.5);assertTrue(runCatching{sourceHistoryReading(badTime,"a.txt","/fixture")}.isFailure)
  val prepared=reading();prepared.getJSONArray("entries").getJSONObject(0).put("outcome","prepared");assertTrue(runCatching{sourceHistoryReading(prepared,"a.txt","/fixture")}.isFailure)
 }
 @Test fun strictSaveRetainsExactRequestGuardAndDraftAfterLostOrForeignReply()=runBlocking{host{w,b,mode,saves->
  b.applySourceEdits(listOf(NativeSourceEdit(0,b.text.length,"draft")),b.text);mode.set("lost");b.save(w.engine!!);assertEquals("saved",b.base);assertEquals("draft",b.text);assertEquals("original-physical-token",b.saveGuard)
  for(refused in listOf("foreign","failed","prepared","unknown")){mode.set(refused);b.save(w.engine!!);assertEquals("saved",b.base);assertEquals("original-physical-token",b.saveGuard);assertEquals(saves.first().getString("request_id"),saves.last().getString("request_id"));assertEquals("original-physical-token",saves.last().getString("save_guard"))}
  mode.set("normal");b.save(w.engine!!);assertEquals("draft",b.base);assertEquals("new-physical-token",b.saveGuard);assertEquals(saves.first().getString("request_id"),saves.last().getString("request_id"));assertEquals("draft",b.text)
  b.applySourceEdits(listOf(NativeSourceEdit(0,b.text.length,"later")),b.text);mode.set("unavailable");b.save(w.engine!!);assertEquals("later",b.base);assertFalse(b.dirty);assertTrue(b.error.contains("unavailable"))
 }}
 @Test fun missingGuardNeverUsesLegacySaveRoute()=runBlocking{host{w,b,_,saves->b.applySourceEdits(listOf(NativeSourceEdit(0,b.text.length,"draft")),b.text);b.saveGuard=null;b.save(w.engine!!);assertEquals(0,saves.size);assertEquals("saved",b.base);assertEquals("draft",b.text);assertTrue(b.error.contains("authority"))}}
 @Test fun historicalDraftRestoresThroughNativeUndoWithoutChangingSavedBaseline()=runBlocking{host{w,b,_,_->b.applySourceEdits(listOf(NativeSourceEdit(0,b.text.length,"unsaved")),b.text);val state=b.state;val scroll=b.sourceScroll.value;val m=w.sourceHistory;m.open(w);assertNotNull(m.notice,m.reading);m.choose(m.reading!!.entries.single(),"before");assertEquals("unsaved",b.text);assertTrue(m.notice,m.restore());assertEquals("old",b.text);assertEquals("saved",b.base);assertEquals("original-physical-token",b.saveGuard);assertTrue(b.snippets.undoSource());assertEquals("unsaved",b.text);assertTrue(b.snippets.redoSource());assertEquals("old",b.text);assertSame(state,b.state);assertSame(b,w.currentBuffer);assertEquals(scroll,b.sourceScroll.value)}}
 @Test fun editUndoSameSpellingAndTamperedRestorationCannotApply()=runBlocking{for(change in listOf("revision","replacement","tampered"))host{w,b,mode,_->val m=w.sourceHistory;m.open(w);m.choose(m.reading!!.entries.single(),"before");if(change=="revision")b.advanceSourceRevision() else if(change=="replacement")w.buffers["a.txt"]=DocumentBuffer(b.file,FileText(b.text,b.hash,b.saveGuard)) else mode.set("tampered");assertFalse(m.restore());assertEquals("saved",b.text)}}
 @Test fun missingSourceRestoresOnlyAnUnmaterializedDraft()=runBlocking{host{w,b,mode,_ ->mode.set("missing");val state=b.state;val scroll=b.sourceScroll.value;val m=w.sourceHistory;m.open(w);m.choose(m.reading!!.entries.first(),"before");assertTrue(m.notice,m.restore());assertEquals("old",b.text);assertTrue(b.resourceDeleted);assertNull(b.saveGuard);assertSame(state,b.state);assertEquals(scroll,b.sourceScroll.value);assertTrue(b.snippets.undoSource());assertEquals("saved",b.text);assertTrue(b.resourceDeleted);w.save();assertEquals("Save As",w.fileCommand)}}
 @Test(timeout=90000) fun liveNativeOrdinarySaveHistoryRestoreAndUndo()=runBlocking{
  val origin=System.getenv("HICKORY_NATIVE_HISTORY_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_NATIVE_HISTORY_WORKSPACE") ?: error("Use the maintained source-history fixture"));check(root.resolve(".hickory-native-history-fixture").readText()=="owned-native-history-v1\n"){"Refuse non-owned ordinary-history fixture"}
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(background)
  try{
  val engine=Engine(origin!!);w.engine=engine;val (_,tree)=engine.files();assertEquals(root.toPath().toRealPath().toString(),engine.sourceRootAddress);w.rootFiles=tree
  fun find(nodes:List<FileNode>):FileNode? {for(node in nodes){if(node.path=="history.txt")return node;find(node.children)?.let{return it}};return null}
  val file=find(tree) ?: error("History source not listed");val b=w.loadBuffer(file);w.show(file.path);val initial=b.text;val state=b.state;val scroll=b.sourceScroll.value;assertFalse(b.readOnly);assertNotNull(b.saveGuard)
  assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(0,b.text.length,"saved-after\n")),b.text));b.save(engine);assertTrue(b.error,b.error.isEmpty());assertFalse(b.dirty);assertNotNull(b.saveGuard)
  assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"unsaved")),b.text));val draft=b.text;val selection=b.state.selection;val m=w.sourceHistory
  val commands=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()});val action=w.commands(commands).first{it.id=="source.history"};assertNull(action.reason);action.run();withTimeout(5000){while(m.busy||m.reading==null){delay(10)}}
  val entry=m.reading!!.entries.first();assertEquals("published",entry.outcome);assertTrue(entry.available);assertEquals(initial,entry.before.text);m.choose(entry,"before");assertEquals(draft,b.text);assertTrue(m.notice,m.restore());assertEquals(initial,b.text);assertEquals("saved-after\n",b.base);assertEquals("saved-after\n",root.resolve("history.txt").readText());assertTrue(b.snippets.undoSource());assertEquals(draft,b.text);assertEquals(selection,state.selection);assertTrue(b.snippets.redoSource());assertEquals(initial,b.text);assertSame(state,b.state);assertSame(b,w.currentBuffer);assertEquals(scroll,b.sourceScroll.value)
  assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"lost draft")),b.text));val missingDraft=b.text;check(root.resolve("history.txt").delete());m.open(w);assertEquals("missing",m.reading!!.current.kind);m.choose(m.reading!!.entries.first(),"before");assertTrue(m.notice,m.restore());assertEquals(initial,b.text);assertTrue(b.unmaterializedHistoryDraft);assertNull(b.saveGuard);assertFalse(root.resolve("history.txt").exists());assertTrue(b.snippets.undoSource());assertEquals(missingDraft,b.text);assertTrue(b.snippets.redoSource());assertEquals(initial,b.text);assertFalse(root.resolve("history.txt").exists());assertSame(state,b.state);assertSame(b,w.currentBuffer);assertEquals(scroll,b.sourceScroll.value)
  w.fileCommand="Save As";w.filePath="recovered λ%2F?#.txt";w.performFileCommand();assertEquals(w.status,"recovered λ%2F?#.txt",b.file.path);assertFalse(b.unmaterializedHistoryDraft);assertEquals(initial,root.resolve(b.file.path).readText());assertFalse(root.resolve("history.txt").exists())
  val recovered=root.resolve(b.file.path);w.changes.open("undo",w);assertEquals(w.changes.notice,missingDraft,b.text);assertEquals(initial,recovered.readText());assertFalse(root.resolve("history.txt").exists());w.changes.open("redo",w);assertEquals(w.changes.notice,initial,b.text);assertEquals(initial,recovered.readText());assertSame(state,b.state);assertSame(b,w.currentBuffer);assertEquals(scroll,b.sourceScroll.value)

  }finally{w.buffers.values.forEach{it.language?.close()};background.cancel()}
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SourceHistoryTest.kt"><ui:paste select="#source-history-android-tests"/></ui:file>

The generated Android build passed all five focused history methods together with five graph, five conflict and one resource-receipt guard method (sixteen selected methods, no opt-in cases). The maintained Android ordinary-history runner then passed its one actual controller/TextFieldState witness with zero skips. A subsequent canonical build reran all sixteen selected methods with zero skips, including published-only save outcomes, malformed capability refusal, explicit Missing-side reading, no-op draft availability and complete-group receipt checks. The actual ordinary-save flow precedes those bounded admission refinements; its normal source path remains the same. Rendered Compose layout, physical input/IME and accessibility remain unexecuted.

## Reproduce an ordinary source save on an owned engine

The maintained fixture creates an independent plain history.txt and marked temporary workspace. Both host selectors reuse the existing owned-process harness, preserve failures and require exactly one executed native result. Serve-only preparation is not acceptance. The test saves through the normal strict source path, retains an unsaved draft, opens the shared Local source history command, restores only the historical draft and exercises native Undo/Redo while saved bytes, baseline, guard and editor identity remain distinct. It installs no SDK, compiler or provider dependency.

<ui:copy id="source-history-native-runner">
"""Run a native ordinary-save history witness against one owned plain source fixture."""
import argparse, importlib.util, json, os, re, signal, subprocess, tempfile, threading, xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['apple','android']);parser.add_argument('--engine',type=Path,default=ROOT/'target/source-edit-focus/debug/examples/engine_client');parser.add_argument('--serve-only',action='store_true');parser.add_argument('--skip-build',action='store_true',help='Apple only: use the just-built native test product.');parser.add_argument('--create',action='store_true',help='Select Android strict Save As; Apple uses its same combined history/Save As case.');args=parser.parse_args()
    def interrupted(number,_frame):raise SystemExit(128+number)
    signal.signal(signal.SIGTERM,interrupted)
    spec=importlib.util.spec_from_file_location('simple_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    if not args.engine.is_file():parser.error('Build the existing engine_client example first; this witness installs no dependencies.')
    with tempfile.TemporaryDirectory(prefix='hickory-native-history-') as folder:
        temporary=Path(folder);root=temporary/'workspace';root.mkdir();(root/'.git').mkdir();(root/'history.txt').write_text('saved-before\n');(root/'.hickory-native-history-fixture').write_text('owned-native-history-v1\n')
        engine=fixture.start([str(args.engine.resolve()),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(temporary/'state')})
        try:
            origin=fixture.origin(engine);print(json.dumps({'origin':origin,'workspace':str(root),'input':'history.txt','acceptance':'not run'}),flush=True)
            if args.serve_only:threading.Event().wait();return
            env={**os.environ,'HICKORY_NATIVE_HISTORY_ORIGIN':origin,'HICKORY_NATIVE_HISTORY_WORKSPACE':str(root)}
            if args.host=='apple':
                name='testLiveNativeOrdinarySaveHistoryRestoreAndUndo';report=temporary/'history.xml';command=['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','SourceHistoryTests.'+name+'$','--xunit-output',str(report)]+(['--skip-build'] if args.skip_build else [])
            else:
                name='liveNativeStrictSaveAsMenuRetainsEditorAndGuardsOwnership' if args.create else 'liveNativeOrdinarySaveHistoryRestoreAndUndo';case='SourceCreationTest' if args.create else 'SourceHistoryTest';report=ROOT/('apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.'+case+'.xml');command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.'+case+'.'+name]
            report.unlink(missing_ok=True);log=ROOT/'target/native-history-smoke'/f'{args.host}-last.log';log.parent.mkdir(parents=True,exist_ok=True);process=fixture.start(command,cwd=ROOT,env=env)
            try:output,_=process.communicate(timeout=600)
            except BaseException:
                fixture.stop(process);output,_=process.communicate(timeout=15);log.write_text(output or '');raise
            log.write_text(output or '')
            if process.returncode:raise RuntimeError(f'Native history failed; inspect {log}')
            if report.exists():
                rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(key) is not None for key in ('skipped','failure','error')),str(report)
            else:
                assert args.host=='apple' and re.search(re.escape(name)+r'.*passed',output or '') and re.search(r'Executed 1 test, with 0 failures',output or '') and 'Test skipped' not in (output or ''),str(log)
            print(f'PASS {args.host}: one actual {"guarded Save As" if args.create else "ordinary-save/history/draft restoration"} controller witness including deleted-source draft restoration, Undo/Redo without materialization and separate guarded Save As, no skips. Apple combines history and Save As in this same case. Physical input/accessibility remains unverified.')
        finally:fixture.stop(engine)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-native-source-history.py"><ui:paste select="#source-history-native-runner"/></ui:file>
</ui:doc>
