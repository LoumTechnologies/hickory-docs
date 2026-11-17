@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import org.json.JSONArray
import kotlinx.coroutines.*
import androidx.compose.ui.text.TextRange
class EditorRecoveryTest {
 private fun buffer(path:String,text:String="draft 😀",unnamed:Boolean=false,owner:String?=null)=DocumentBuffer(FileNode(path,path,false,emptyList(),null,owner),FileText(text,"old","original"),untitledDraft=unnamed)
 private fun meta()=JSONObject().put("version",1).put("recovery_protocol",1).put("engine_workspace_id","e".repeat(32)).put("workspace_id","a".repeat(64)).put("namespace","/owned").put("window_slot","0").put("current_window_slot","0").put("window_slots",JSONArray().put("0")).put("sessions",JSONArray()).put("notices",JSONArray()).put("source_effects",false)
 private suspend fun host(block:suspend(Workspace,DocumentBuffer)->Unit)=coroutineScope {
  val server=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0);server.createContext("/api/files"){exchange->val bytes=JSONObject().put("root","Owned").put("root_path","/owned").put("workspace_id","e".repeat(32)).put("tree",JSONArray()).toString().toByteArray();exchange.sendResponseHeaders(200,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}};server.start()
  val w=Workspace(this);w.engine=Engine("http://127.0.0.1:${server.address.port}");w.engine!!.files();val b=buffer("files");w.buffers[b.file.path]=b;w.show(sourceTab(b.file.path));try{block(w,b)}finally{server.stop(0)}
 }
 @Test fun rawReviewRefusesDuplicateAliasedVersionsAndExcessiveNesting(){for(raw in listOf("{\"version\":1,\"version\":1}","{\"version\":1,\"recovery_protocol\":1.0}","[".repeat(65)+"0"+"]".repeat(65))){assertTrue(runCatching{nativeReviewProtocolJSON(raw,1024,"Recovery",setOf("version","recovery_protocol"))}.isFailure)}}
 @Test fun captureKeepsReservedLiteralUnnamedAndGeneratedSourcesDistinct()=runBlocking {
  val w=Workspace(this);for(path in listOf("files","agent","terminal","Untitled-real.txt","é.txt","e\u0301.txt")){val b=buffer(path);w.buffers[path]=b;w.layout=w.layout.activate(sourceTab(path),"document-group")};val note=buffer("note","anonymous",true);w.buffers[note.file.path]=note;val generated=buffer("out.py","output",owner="project.md");w.buffers[generated.file.path]=generated
  val snapshot=captureEditorRecovery(w);assertEquals(8,snapshot.buffers.size);assertEquals(6,snapshot.buffers.count{it.address.kind=="source"});assertEquals(1,snapshot.buffers.count{it.address.kind=="unnamed"});assertEquals(1,snapshot.buffers.count{it.address.kind=="generated"});assertTrue(snapshot.buffers.all{it.baseline?.guard=="original"})
 }
 @Test fun coldCopiesKeepGeneratedTextReadonlyAndExistingTextStateUndo()=runBlocking {
  val old=Workspace(this);val source=buffer("files");val output=buffer("out.py","generated",owner="project.md");old.buffers[source.file.path]=source;old.buffers[output.file.path]=output;old.show(sourceTab("files"));val snapshot=captureEditorRecovery(old)
  val fresh=Workspace(this);val independent=buffer("files","initial");fresh.buffers["files"]=independent;fresh.show(sourceTab("files"));independent.applySourceEdits(listOf(NativeSourceEdit(0,independent.text.length,"independent draft")),independent.text);val state=independent.state;val selection=state.selection;restoreEditorRecovery(snapshot,fresh)
  assertSame(independent,fresh.buffers["files"]);assertSame(state,independent.state);assertEquals(selection,state.selection);assertEquals("independent draft",independent.text);val recovered=fresh.buffers.values.filter{it!==independent};assertEquals(2,recovered.size);assertTrue(recovered.all{it.saveGuard==null&&it.recoveryBaseline?.guard=="original"});assertEquals(1,recovered.count{it.readOnly});assertEquals(1,recovered.count{it.isUntitledDraft});assertTrue(independent.snippets.undoSource());assertEquals("initial",independent.text)
 }
 @Test fun malformedLayoutAndScalarSelectionNeverImportPartialSources()=runBlocking {
  val w=Workspace(this);val b=buffer("files");w.buffers[b.file.path]=b;val snapshot=captureEditorRecovery(w);val target=Workspace(this);assertTrue(runCatching{restoreEditorRecovery(snapshot.copy(layout="{}"),target)}.isFailure);assertTrue(target.buffers.isEmpty());val raw=snapshot.json();raw.getJSONArray("buffers").getJSONObject(0).getJSONObject("view").getJSONObject("selection").put("head",7);assertTrue(runCatching{editorRecoverySnapshot(raw)}.isFailure)
  for(value in listOf("[".repeat(65)+"0"+"]".repeat(65),"\uD800")){assertTrue(runCatching{nativeReviewMaterialBudget(value,1024,"Fixture")}.isFailure)}
 }
 @Test fun fullLeaseRetiresSelectionScrollAndSameSizedRetainedOwnerReplacement()=runBlocking {
  val w=Workspace(this);val b=buffer("files");w.buffers[b.file.path]=b;val note=buffer("note","old",true);w.retainedResourceEditors["note"]=note;val first=editorRecoveryLease(w);assertTrue(first());b.state.edit{this.selection=TextRange(1)};assertFalse(first());val second=editorRecoveryLease(w);b.sourceScroll.scrollTo(20);assertFalse(second());val third=editorRecoveryLease(w);w.retainedResourceEditors["note"]=buffer("note","old",true);assertFalse(third())
 }
 @Test fun uncertainCheckpointRetainsExactOriginalRequestBeforeNewerCapture()=runBlocking {host{w,b->
  val requests=mutableListOf<String>();var lose=true;val model=EditorRecoveryModel{_,method,_,body->if(method=="GET")meta() else {val value=body!!;requests.add(value.toString());if(lose){lose=false;error("Lost response")};JSONObject().put("version",1).put("recovery_protocol",1).put("client_id",value.getString("client_id")).put("window_slot","0").put("workspace_id","a".repeat(64)).put("namespace","/owned").put("request_id",value.getString("request_id")).put("revision",value.getLong("base_revision")+1).put("snapshot_hash","f".repeat(64)).put("source_effects",false).put("source_saved",false)}}
  model.checkpoint(w);b.applySourceEdits(listOf(NativeSourceEdit(0,b.text.length,"newer draft")),b.text);model.checkpoint(w);assertEquals(requests[0],requests[1]);assertEquals(1L,model.revision);model.checkpoint(w);assertEquals(2L,model.revision);assertNotEquals(requests[1],requests[2])
 }}
 @Test fun sealedReadRefusesChangedBytesAndSourceWriteClaims()=runBlocking {for(mode in listOf("valid","material","permission","source_saved"))host{w,b->
  val value=captureEditorRecovery(w);val material=value.json().toString();val hash=debugHash(material);val row=EditorRecoverySession("c".repeat(32),"0","a".repeat(64),"/owned",1,"d".repeat(32),hash,"android",1)
  val envelope=JSONObject().put("version",1).put("recovery_protocol",1).put("engine_workspace_id","e".repeat(32)).put("client_id",row.client).put("window_slot","0").put("workspace_id",row.workspace).put("namespace","/owned").put("revision",1).put("request_id",row.request).put("snapshot_hash",hash).put("material",material+if(mode=="material")" " else "").put("source_effects",false).put("source_saved",mode=="source_saved").put("authority",JSONArray().put(JSONObject().put("buffer_id",value.buffers.single().id).put("baseline_current",true).put("write_permission",mode=="permission")))
  val model=EditorRecoveryModel{_,_,_,_->envelope};model.read(row,w);if(mode=="valid"){assertTrue(model.notice,model.canRestore);b.advanceSourceRevision();assertFalse(model.restore(w))}else assertNull(mode,model.snapshot);assertEquals("draft 😀",b.text);assertEquals(1,w.buffers.size)
 }}
 @Test(timeout=120000) fun liveOwnedProcessRestartRecovery()=runBlocking {
  val origin=System.getenv("HICKORY_NATIVE_SESSION_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_NATIVE_SESSION_WORKSPACE") ?: error("Use the owned session fixture"));val phase=System.getenv("HICKORY_NATIVE_SESSION_PHASE") ?: error("Select capture or restore");check(root.resolve(".hickory-native-session-fixture").readText()=="owned-native-session-v1\n"){"Refuse non-owned session fixture"};println("NATIVE_SESSION_PROCESS_ID="+ProcessHandle.current().pid())
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(background);val e=Engine(origin!!);w.engine=e;val (_,tree)=e.files();w.rootFiles=tree
  fun find(path:String,nodes:List<FileNode>):FileNode?{for(node in nodes){if(node.path==path)return node;find(path,node.children)?.let{return it}};return null}
  val b=w.loadBuffer(find(if(phase=="capture")"files" else "note.py",tree) ?: error("Missing fixture source"));w.show(sourceTab(b.file.path));val state=b.state;val model=w.editorRecovery;var loop:Job?=null
  try{
   if(phase=="capture"){
    assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"\nunsaved λ😀")),b.text));b.state.edit{this.selection=TextRange(9,12)};b.sourceScroll.scrollTo(180);val note=buffer("anonymous","anonymous retained note",true);w.buffers[note.file.path]=note;val source=b.text;loop=background.launch{model.captureLoop(w)}
    withTimeout(15000){while(model.revision==0L)delay(20)};model.open(w);val row=model.catalogue!!.sessions.first{it.host=="android"&&it.count==2};model.read(row,w);val saved=model.snapshot!!.buffers.first{it.address.kind=="source"&&it.address.label=="files"};assertEquals(source,saved.text);assertEquals(9,saved.view.anchor);assertEquals(12,saved.view.head);assertEquals(180.0,saved.view.y,0.0);assertTrue(model.snapshot!!.buffers.any{it.address.kind=="unnamed"&&it.text=="anonymous retained note"});assertFalse(root.resolve("files").readText().contains("unsaved"));return@runBlocking
   }
   assertEquals("restore",phase);assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"# independent new draft")),b.text));b.state.edit{this.selection=TextRange(2,4)};val source=b.text;val selection=state.selection;val scroll=b.sourceScroll.value
   val commands=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()});val action=w.commands(commands).first{it.id=="workspace.recovery"};assertNull(action.reason);action.run();assertTrue(w.showRecovery);w.showRecovery=false;model.open(w);val row=model.catalogue!!.sessions.first{it.host=="android"&&it.count==2};model.read(row,w);val saved=model.snapshot!!.buffers.first{it.address.kind=="source"&&it.address.label=="files"};assertTrue(model.notice,model.canRestore)
   b.state.edit{this.selection=TextRange(3)};assertFalse(model.restore(w));assertEquals(1,w.buffers.size);b.state.edit{this.selection=selection};model.read(row,w);assertTrue(model.notice,model.restore(w));val recovered=w.buffers.values.single{it.isUntitledDraft&&it.text==saved.text};assertNull(recovered.saveGuard);assertSame(recovered,w.currentBuffer);assertEquals(TextRange(saved.view.anchor,saved.view.head),recovered.state.selection);assertEquals(saved.view.y.toInt(),recovered.sourceScroll.value);assertTrue(w.buffers.values.any{it.isUntitledDraft&&it.text=="anonymous retained note"})
   assertSame(b,w.buffers["note.py"]);assertSame(state,b.state);assertEquals(source,b.text);assertEquals(selection,state.selection);assertEquals(scroll,b.sourceScroll.value);assertTrue(b.snippets.undoSource());assertEquals("value=1\n",b.text);assertFalse(root.resolve("files").readText().contains("unsaved"));assertEquals("value=1\n",root.resolve("note.py").readText())
  }finally{loop?.cancelAndJoin();model.close();w.buffers.values.forEach{it.language?.close()};background.cancel()}
 }
}
