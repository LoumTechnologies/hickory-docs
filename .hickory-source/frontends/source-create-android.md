<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Create one complete source from a frozen Android Save As intention

For maintainers following the existing Save As dialog into no-clobber source publication.

Never create an empty file and then learn authority from a GET. Capture the full retained editor roster, exact source revisions, selections, baselines, opaque guards, metadata and deletion states. All retained named authored drafts, regardless of suffix, carry their originally loaded physical guards. Other unnamed, outside-address and deleted authored drafts remain explicit unaddressed inputs; the actor receives the chosen destination instead. The engine owns whether those inputs can prove construction ownership. Missing workspace creation identity or an unbound named construction refuses before sending.

Unnamed status is an explicit editor property, captured alongside the source revision. A real file named `Untitled-report.py` remains a named ownership input and requires its original physical guard. Only confirmed creation converts an unnamed editor into a named source.

The immutable body and lowercase request ID survive an uncertain response only while that entire lease remains current. Replaying cannot silently replace a file: the engine rechecks the retained exact intention and after-identity. Admit only the exact published reply, including source bytes/hash, workspace/rootnamespace/path/request, guard and document identity. Direct receipt data readdresses the same retained editor and initializes its new saved baseline; no post-create read learns a replacement occupant. Original source state and tombstones remain untouched until confirmation. Confirmed adoption rewires only addresses belonging to that exact editor, retaining its native selection, scroll and Undo/Redo; an independently reused old spelling remains untouched. Creation also retires stale workspace namespace observations without replacing unrelated editors. Physical input, rendered Compose geometry and accessibility remain separate acceptance work.

<ui:copy id="source-create-android">
@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
package com.hickorydocs.simple
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONObject
import androidx.compose.ui.text.TextRange

private fun sourceCreationRoster(w:Workspace)=(w.buffers.values+w.retainedResourceEditors.values+w.retainedResourceTabs.values).distinct()
data class SourceCreationInput(val buffer:DocumentBuffer,val path:String,val text:String,val revision:Long,val base:String,val hash:String,val guard:String?,val file:FileNode,val readonly:Boolean,val deleted:Boolean,val untitled:Boolean,val selection:TextRange)
data class SourceCreationAttempt(val engine:Engine,val workspace:String,val namespace:String,val epoch:java.util.UUID,val actor:DocumentBuffer,val tab:String?,val destination:String,val source:String,val hash:String,val id:String,val inputs:List<SourceCreationInput>,val body:String,val addresses:Map<String,DocumentBuffer>,val retained:Map<String,DocumentBuffer>,val tabs:Map<String,DocumentBuffer>){
 fun current(w:Workspace,source:DocumentBuffer):Boolean {
  if(w.engine!==engine||engine.sourceWorkspaceID!=workspace||engine.sourceRootAddress!=namespace||w.sourceNavigationEpoch!=epoch||source!==actor||w.active!=tab||w.bufferForTab(tab)!==actor||source.text!=this.source)return false
  fun same(current:Map<String,DocumentBuffer>,captured:Map<String,DocumentBuffer>)=current.size==captured.size&&captured.all{(path,b)->current[path]===b}
  if(!same(w.buffers,addresses)||!same(w.retainedResourceEditors,retained)||!same(w.retainedResourceTabs,tabs))return false
  val live=sourceCreationRoster(w);if(live.toSet()!=inputs.map{it.buffer}.toSet())return false
  return inputs.all{s->val b=s.buffer;b.file.path==s.path&&b.file===s.file&&b.text==s.text&&b.sourceRevision==s.revision&&b.base==s.base&&b.hash==s.hash&&b.saveGuard==s.guard&&b.readOnly==s.readonly&&b.resourceDeleted==s.deleted&&b.isUntitledDraft==s.untitled&&b.state.selection==s.selection&&b.state.composition==null&&(b===actor||!b.saving)}
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
  val epoch=sourceNavigationEpoch;val tab=active;val addresses=buffers.toMap();val retained=retainedResourceEditors.toMap();val tabs=retainedResourceTabs.toMap();check(addresses.all{(path,b)->path==b.file.path}){"A retained source address aliases another editor. Reconcile it before Save As."};val inputs=sourceCreationRoster(this).map{b->check(b.state.composition==null&&(b===source||!b.saving)){"Finish composition and other source saves before Save As."};SourceCreationInput(b,b.file.path,b.text,b.sourceRevision,b.base,b.hash,b.saveGuard,b.file,b.readOnly,b.resourceDeleted,b.isUntitledDraft,b.state.selection)}
  check(inputs.size<=4096&&inputs.any{it.buffer===source}){"Save As requires one retained source editor and a bounded complete roster."};val submitted=source.text;val id=java.util.UUID.randomUUID().toString().replace("-","")
  val built=withContext(Dispatchers.Default){
   var bytes=creationByteCost(submitted,8L*1024*1024)+creationByteCost(path,8192);val ownership=JSONObject();val guards=JSONObject();val unaddressed=JSONObject()
   fun charge(text:String){val cost=creationByteCost(text,8L*1024*1024);check(cost<=32L*1024*1024-bytes){"Retained source creation context exceeds 32 MiB. Reconcile or reduce the captured drafts first."};bytes+=cost}
   for(input in inputs){if(input.file.generatedBy!=null)continue
    val named=!input.untitled&&!input.path.startsWith('/')&&!input.deleted
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
</ui:copy>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SourceCreation.kt"><ui:paste select="#source-create-android"/></ui:file>

## Exercise no-clobber source creation without a discovery write gap

The focused HTTP/controller fixture checks one complete request, direct receipt initialization, the same retained text state/undo/selection/scroll after confirmed address adoption, all-suffix named physical guards and unaddressed retained inputs. Missing authority sends nothing. Lost replies keep the same frozen request; foreign/nonpublished/wrongly typed replies or a newly retained input cannot adopt a destination. Background capability requests are separate from saving and never supply post-create authority. Canonical Kotlin compilation and all six focused methods passed with zero skips. The maintained `test-native-source-history.py android --create` then passed one actual File menu/controller witness against an isolated real engine: the same Buffer/TextFieldState retained selection, scroll and native Undo/Redo through literal-name and authored-source creation; original bytes stayed unchanged, creation history retained Missing-before, and another unaddressed construction draft refused before creating a destination. These results do not establish rendered Compose or physical input/accessibility acceptance.

<ui:copy id="source-create-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import org.json.JSONArray
import kotlinx.coroutines.*
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class SourceCreationTest {
 private class Host(val w:Workspace,val source:DocumentBuffer,val mode:java.util.concurrent.atomic.AtomicReference<String>,val requests:MutableList<JSONObject>,val routes:MutableList<String>,val entered:java.util.concurrent.CountDownLatch,val release:java.util.concurrent.CountDownLatch)
 private suspend fun host(workspace:Any?="a".repeat(32),block:suspend(Host)->Unit)=coroutineScope{
  val mode=java.util.concurrent.atomic.AtomicReference("normal");val requests=java.util.Collections.synchronizedList(mutableListOf<JSONObject>());val routes=java.util.Collections.synchronizedList(mutableListOf<String>());val entered=java.util.concurrent.CountDownLatch(1);val release=java.util.concurrent.CountDownLatch(1);val server=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0)
  server.createContext("/api"){exchange->val path=exchange.requestURI.path;routes.add(exchange.requestMethod+" "+path);val text=exchange.requestBody.bufferedReader().readText();val body=text.takeIf{it.isNotEmpty()}?.let{JSONObject(it)};var code=200
   val value=when(path){"/api/files"->JSONObject().put("root","Display root").put("root_path","/fixture").put("tree",JSONArray()).apply{if(workspace!=null)put("workspace_id",workspace)}
    "/api/source-history/create"->{val input=body!!;requests.add(input);if(mode.get()=="hold"){entered.countDown();release.await(5,java.util.concurrent.TimeUnit.SECONDS)};if(mode.get()=="lost"){code=409;JSONObject().put("error","Publication response unconfirmed")}else JSONObject().put("version",1).put("history_protocol",1).put("workspace",input.getString("workspace")).put("namespace","/fixture").put("path",if(mode.get()=="foreign")"foreign.py" else input.getString("path")).put("source",input.getString("source")).put("source_hash",debugHash(input.getString("source"))).put("hash","created-fnv").put("request_id",input.getString("request_id")).put("save_guard","created-token").put("doc_id",if(input.getString("path").endsWith(".md"))"created-document" else JSONObject.NULL).put("history",JSONObject().put("available",if(mode.get()=="badtype")"true" else true).put("outcome",if(mode.get() in listOf("failed","prepared","unknown"))mode.get() else "published"))}
    else->{code=400;JSONObject().put("error","Unexpected route $path")}}
   val bytes=value.toString().toByteArray();exchange.sendResponseHeaders(code,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}
  };server.start();val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(background);w.engine=Engine("http://127.0.0.1:${server.address.port}");w.engine!!.files();val source=DocumentBuffer(FileNode("Draft","Untitled-actor",false,emptyList(),null,null),FileText("",""),untitledDraft=true);source.applySourceEdits(listOf(NativeSourceEdit(0,0,"source copy\n")),"");source.selectSourceRange(2,2,source.text);source.sourceScroll.scrollTo(11);w.buffers[source.file.path]=source;w.show(source.file.path)
  try{block(Host(w,source,mode,requests,routes,entered,release))}finally{release.countDown();w.buffers.values.forEach{it.language?.close()};background.cancel();server.stop(0)}
 }
 private suspend fun saveAs(h:Host,path:String="copy λ%?#.py"){h.w.fileCommand="Save As";h.w.filePath=path;h.w.performFileCommand()}
 @Test fun oneCompleteRequestUsesDirectReceiptAndPreservesOriginalTextState()=runBlocking{host{h->val source=h.source;val state=source.state;val selection=state.selection;val scroll=source.sourceScroll.value;val namespace=h.w.sourceNavigationEpoch;assertTrue(state.undoState.canUndo);saveAs(h);val created=h.w.currentBuffer!!;assertSame(source,created);assertEquals("copy λ%?#.py",created.file.path);assertEquals(source.text,created.text);assertEquals(source.text,created.base);assertEquals("created-fnv",created.hash);assertEquals("created-token",created.saveGuard);assertEquals(selection,created.state.selection);assertEquals(scroll,created.sourceScroll.value);assertSame(state,source.state);assertEquals(selection,state.selection);assertTrue(state.undoState.canUndo);assertNotEquals(namespace,h.w.sourceNavigationEpoch);assertEquals(1,h.requests.size);assertEquals(source.text,h.requests.single().getString("source"));assertFalse(h.routes.any{it.endsWith(" /api/file")||it.endsWith(" /api/files/op")||it.endsWith(" /api/projects/local/docs")})}}
 @Test fun allNamedAndUnaddressedAuthoredInputsAreFrozenBeforeCreation()=runBlocking{host{h->
  val named=DocumentBuffer(FileNode("owner.txt","owner.txt",false,emptyList(),null,null),FileText("named draft","named-hash","named-guard"));val note=DocumentBuffer(FileNode("Note","Untitled-note",false,emptyList(),null,null),FileText("plain note",""),untitledDraft=true);val deleted=DocumentBuffer(FileNode("old.md","old.md",false,emptyList(),null,null),FileText("deleted draft","old","old-guard"));deleted.resourceDeleted=true;h.w.buffers[named.file.path]=named;h.w.buffers[note.file.path]=note;h.w.retainedResourceEditors["old"]=deleted;saveAs(h)
  val body=h.requests.single();assertEquals("named draft",body.getJSONObject("ownership").getJSONObject("ownership_sources").getString("owner.txt"));assertEquals("named-guard",body.getJSONObject("saved_guards").getString("owner.txt"));assertEquals("plain note",body.getJSONObject("unaddressed_sources").getString(note.debugIdentity));assertEquals("deleted draft",body.getJSONObject("unaddressed_sources").getString(deleted.debugIdentity));assertFalse(body.getJSONObject("unaddressed_sources").has(h.source.debugIdentity));assertEquals(body.getString("request_id"),body.getJSONObject("ownership").getString("context"));assertSame(deleted,h.w.retainedResourceEditors["old"])
 }}
 @Test fun unboundNamedInputsAndMissingWorkspaceAuthoritySendNoCreation()=runBlocking{
  host{h->val named=DocumentBuffer(FileNode("owner.py","owner.py",false,emptyList(),null,null),FileText("retained","hash"));h.w.buffers[named.file.path]=named;saveAs(h);assertTrue(h.w.status,h.w.status.contains("captured physical baseline"));assertTrue(h.requests.isEmpty());assertSame(h.source,h.w.currentBuffer)}
  host(null){h->saveAs(h);assertTrue(h.requests.isEmpty());assertTrue(h.w.status.contains("Creation authority"));assertSame(h.source,h.w.currentBuffer)}
 }
 @Test fun uncertainRepliesRetainOneExactRequestAndNeverLearnFromARead()=runBlocking{host{h->h.mode.set("lost");saveAs(h);assertSame(h.source,h.w.currentBuffer);assertNotNull(h.source.creationAttempt);val body=h.requests.single().toString();h.mode.set("normal");saveAs(h);assertEquals(2,h.requests.size);assertEquals(body,h.requests.last().toString());assertEquals("created-token",h.w.currentBuffer!!.saveGuard);assertFalse(h.routes.any{it=="GET /api/file"})}}
 @Test fun foreignNonpublishedAndWronglyTypedOutcomesNeverAdoptDestination()=runBlocking{for(mode in listOf("foreign","failed","prepared","unknown","badtype"))host{h->h.mode.set(mode);val state=h.source.state;saveAs(h);assertSame(mode,h.source,h.w.currentBuffer);assertSame(state,h.source.state);assertFalse(h.w.buffers.containsKey("copy λ%?#.py"));assertFalse(h.source.saving);assertNotNull(h.source.creationAttempt)}}
 @Test fun addedRetainedInputDuringRequestRetiresOtherwiseExactReply()=runBlocking{host{h->h.mode.set("hold");val pending=async{saveAs(h)};withTimeout(5000){while(h.entered.count!=0L)delay(10)};val other=DocumentBuffer(FileNode("Other","Untitled-new",false,emptyList(),null,null),FileText("new draft",""));h.w.buffers[other.file.path]=other;h.release.countDown();pending.await();assertSame(h.source,h.w.currentBuffer);assertFalse(h.w.buffers.containsKey("copy λ%?#.py"));assertTrue(h.w.status.contains("obsolete editor namespace"));assertEquals("source copy\n",h.source.text)}}
 @Test(timeout=120000) fun liveNativeStrictSaveAsMenuRetainsEditorAndGuardsOwnership()=runBlocking{
  val origin=System.getenv("HICKORY_NATIVE_HISTORY_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_NATIVE_HISTORY_WORKSPACE") ?: error("Use the maintained native history fixture"));check(root.resolve(".hickory-native-history-fixture").readText()=="owned-native-history-v1\n"){"Refuse non-owned creation fixture"}
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(background);val engine=Engine(origin!!);w.engine=engine
  try{val (_,tree)=engine.files();assertEquals(root.toPath().toRealPath().toString(),engine.sourceRootAddress);w.rootFiles=tree
   fun find(nodes:List<FileNode>):FileNode? {for(node in nodes){if(node.path=="history.txt")return node;find(node.children)?.let{return it}};return null}
   val file=find(tree) ?: error("Original history fixture source missing");val b=w.loadBuffer(file);w.show(file.path);val state=b.state;val original=b.text;b.applySourceEdits(listOf(NativeSourceEdit(0,b.text.length,"saved copy\n")),b.text);b.selectSourceRange(2,2,b.text);b.sourceScroll.scrollTo(9);val selection=state.selection;val scroll=b.sourceScroll.value
   val descriptors=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()})
   suspend fun create(path:String){val command=w.commands(descriptors).single{it.id=="file.saveAs"};assertNull(command.reason);command.run();assertEquals("Save As",w.fileCommand);w.filePath=path;w.performFileCommand()}
   val destination="copy λ%?#.txt";create(destination);assertSame(w.status,b,w.currentBuffer);assertEquals(destination,b.file.path);assertSame(state,b.state);assertEquals(selection,state.selection);assertEquals(scroll,b.sourceScroll.value);assertEquals("saved copy\n",b.base);assertNotNull(b.saveGuard);assertEquals(original,root.resolve("history.txt").readText());assertEquals("saved copy\n",root.resolve(destination).readText());assertTrue(b.snippets.undoSource());assertEquals(original,b.text);assertTrue(b.snippets.redoSource());assertEquals("saved copy\n",b.text)
   val history=SourceHistoryModel();history.open(w);assertNotNull(history.notice,history.reading);val record=history.reading!!.entries.first();assertEquals("missing",record.before.kind);assertEquals("saved copy\n",record.after.text);history.choose(record,"before");assertNull(history.preview);assertFalse(history.restore());history.close()
   val hidden=DocumentBuffer(FileNode("Hidden","Untitled-hidden",false,emptyList(),null,null),FileText("<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"claimed.py\">value=1</h:file></h:doc>",""));w.buffers[hidden.file.path]=hidden;create("blocked.py");assertFalse(root.resolve("blocked.py").exists());assertSame(b,w.currentBuffer);assertEquals(destination,b.file.path);assertTrue(w.status,w.status.lowercase().contains("construction"));w.buffers.remove(hidden.file.path)
   val literate="<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\nReadable copied source.\n</h:doc>\n";b.applySourceEdits(listOf(NativeSourceEdit(0,b.text.length,literate)),b.text);create("copy-note.md");assertSame(w.status,b,w.currentBuffer);assertSame(state,b.state);assertEquals("copy-note.md",b.file.path);assertNotNull(b.file.docID);assertEquals(literate,b.base);assertNotNull(b.saveGuard);assertEquals(literate,root.resolve("copy-note.md").readText());assertEquals("saved copy\n",root.resolve(destination).readText());assertEquals(original,root.resolve("history.txt").readText())
  }finally{w.buffers.values.forEach{it.language?.close()};background.cancel()}
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SourceCreationTest.kt"><ui:paste select="#source-create-android-tests"/></ui:file>
</ui:doc>
