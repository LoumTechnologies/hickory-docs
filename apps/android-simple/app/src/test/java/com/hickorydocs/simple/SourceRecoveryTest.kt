package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
import org.json.JSONArray
class SourceRecoveryTest {
 @Test fun reviewedTextPagesKeepEveryCharacterAroundAUnicodeBoundary() {
  val prefix="x".repeat(15_999)
  val text=prefix+"😀 following text"
  assertEquals(prefix,reviewedResourceTextPage(text,0))
  assertEquals("😀 following text",reviewedResourceTextPage(text,1))
  assertEquals(text,reviewedResourceTextPage(text,0)+reviewedResourceTextPage(text,1))
  assertEquals("",reviewedResourceTextPage("",0))
 }
 val id="a".repeat(32);val workspace="b".repeat(32)
 fun page()=RecoveryPage(1,workspace,listOf(id),null,0)
 fun reading(hash:String="c".repeat(64))=RecoveryReading(1,workspace,id,hash,"Move",false,listOf(RecoveryPath("a #😀.md",emptyMap())),false,0,"😀","")
 @Test fun exactSnapshotRequestsAndChangedEvidence()=runBlocking {
  val model=RecoveryReader();model.list{page()};var route="";model.read(id,"a #😀.md","saved_before"){route=it;reading()};assertTrue(route.contains("path=a+%23%F0%9F%98%80.md&saved=before"));assertEquals("",model.value?.savedText);assertEquals("😀",model.value?.draftText)
  model.read(id,"a #😀.md"){reading("d".repeat(64))};assertNull(model.value);assertTrue(model.notice.contains("evidence changed"));model.list{page()};model.list(id){page().copy(workspace="other",ids=emptyList())};assertNull(model.page)
 }
 @Test fun closedAndSupersededReadsCannotReplaceSelection()=runBlocking{
  val model=RecoveryReader();model.list{page()};val pending=CompletableDeferred<RecoveryReading>();val old=async(start=CoroutineStart.UNDISPATCHED){model.read(id){pending.await()}};model.list{page()};pending.complete(reading());old.await();assertNull(model.value)
  val late=CompletableDeferred<RecoveryReading>();val closed=async(start=CoroutineStart.UNDISPATCHED){model.read(id){late.await()}};model.close();late.complete(reading());closed.await();assertNull(model.page);assertNull(model.value)
 }
}
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class ResourceReviewTest {
 private fun saved(text:String)=JSONObject().put("kind","file").put("text",text).put("sha256",digest(text)).put("readonly",false).put("executable",true).put("mode",493)
 private fun digest(text:String)=java.security.MessageDigest.getInstance("SHA-256").digest(text.toByteArray()).joinToString(""){"%02x".format(it)}
 private fun missing()=JSONObject().put("kind","missing")
 private fun source(text:String)=JSONObject().put("kind","file").put("identity","source-id").put("text",text).put("writable",true).put("removable",true).put("version",JSONObject.NULL)
 private fun plan(context:String,from:String,to:String,text:String)=JSONObject().put("context",context).put("title","Move source").put("version",1).put("annotations",JSONArray()).put("patches",JSONArray().put(JSONObject().put("path",from).put("before",source(text)).put("after",missing())).put(JSONObject().put("path",to).put("before",missing()).put("after",source(text))))
 @Test fun publicationRetainsLiveEditorTabsSelectionUndoAndSavedMetadata()=runBlocking {
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined)
  try{
   val workspace=Workspace(background);workspace.engine=Engine("http://127.0.0.1:1")
   val file=FileNode("a.py","a.py",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText("saved","old-hash","old-guard"));buffer.state.edit{replace(0,length,"dirty");selection=androidx.compose.ui.text.TextRange(2)};workspace.buffers[file.path]=buffer;workspace.show(file.path)
   var context="";var post=0;var published:JSONObject?=null
   val review=ResourceChangeReview{method,path,body->when(path){
    "/api/changes/resource-targets"->JSONObject().put("paths",JSONArray().put("a.py").put("b.py"))
    "/api/changes/resource-inputs"->{context=body!!.getString("context");assertEquals("dirty",body.getJSONObject("drafts").getJSONObject("a.py").getString("text"));assertEquals("old-guard",body.getJSONObject("drafts").getJSONObject("a.py").getString("save_guard"));assertEquals("old-guard",body.getJSONObject("saved_guards").getString("a.py"));assertEquals(digest("saved"),body.getJSONObject("drafts").getJSONObject("a.py").getString("base_sha256"));JSONObject().put("context",context).put("sources",JSONObject().put("a.py",source("dirty")).put("b.py",missing())).put("saved_guards",JSONObject().put("a.py","old-guard")).put("saved_sources",JSONObject())}
    "/api/changes/plan-resources"->JSONObject().put("plan",plan(context,"a.py","b.py","dirty"))
    "/api/changes/resource-consequences"->JSONObject().put("context",context).put("workspace","b".repeat(32)).put("fingerprint","c".repeat(64)).put("protected_targets",JSONArray()).put("saved_sources",JSONObject().put("a.py",missing()).put("b.py",saved("dirty")))
    "/api/changes/resource-save"->{assertEquals("POST",method);post++;val request=body!!.getString("request_id");JSONObject().put("version",2).put("request_id",request).put("adoption_current",true).put("workspace","b".repeat(32)).put("context",context).put("outcome",JSONObject().put("complete",true).put("applied",JSONArray().put("a.py").put("b.py")).put("recovery",".hick-cache/resource-changes/$request/manifest.json")).put("saved_sources",JSONObject().put("a.py",missing()).put("b.py",saved("dirty"))).put("source_files",JSONArray().put(JSONObject().put("file",JSONObject().put("name","b.py").put("path","b.py").put("dir",false)).put("saved",saved("dirty")).put("hash","fresh"))).also{published=it}}
    else->error("Unexpected $method $path")
   }}
   review.prepare(workspace,"Move",JSONObject().put("documentChanges",JSONArray()))
   assertNotNull(review.review);assertEquals("a.py",workspace.active);assertEquals("saved",buffer.base)
   review.apply(workspace);assertEquals(1,post);assertFalse(review.hasPending);assertSame(buffer,workspace.currentBuffer);assertSame(buffer,workspace.buffers["b.py"]);assertNull(workspace.buffers["a.py"]);assertEquals("b.py",workspace.active);assertEquals(2,buffer.state.selection.start);assertEquals("dirty",buffer.base);assertTrue(buffer.state.undoState.canUndo);assertNotNull(published)
   buffer.state.undoState.undo();assertEquals("saved",buffer.text)
  }finally{background.cancel()}
 }
 @Test fun fullRetainedRosterUsesTypedUnaddressedInputsAndRetiresOnChanges()=runBlocking{
  val w=Workspace(this);w.engine=Engine("http://127.0.0.1:1")
  fun buffer(path:String,text:String)=DocumentBuffer(FileNode(path,path,false,emptyList(),null,null),FileText(text,"old","guard"))
  val named=buffer("owner.txt","named construction draft");val note=buffer("Untitled-note","ordinary unnamed note");note.isUntitledDraft=true;val deleted=buffer("gone.md","retained deleted draft");deleted.resourceDeleted=true;w.buffers[named.file.path]=named;w.buffers[note.file.path]=note;w.retainedResourceEditors["deleted"]=deleted
  val review=ResourceChangeReview{_,_,body->assertEquals("named construction draft",body!!.getJSONObject("ownership_sources").getString("owner.txt"));assertEquals(1,body.getJSONObject("ownership_sources").length());assertEquals(note.text,body.getJSONObject("unaddressed_sources").getString(note.debugIdentity));assertEquals(deleted.text,body.getJSONObject("unaddressed_sources").getString(deleted.debugIdentity));JSONObject().put("saved_guards",JSONObject().put("owner.txt",named.saveGuard)).put("sources",JSONObject().put("owner.txt",source(named.text)))}
  val first=review.acquire(w,listOf("owner.txt"),JSONObject().put("changes",JSONObject())){true};assertTrue(first.third());deleted.advanceSourceRevision();assertFalse(first.third())
  val second=review.acquire(w,listOf("owner.txt"),JSONObject().put("changes",JSONObject())){true};assertTrue(second.third());named.saveGuard="replacement";assertFalse(second.third())
  val third=review.acquire(w,listOf("owner.txt"),JSONObject().put("changes",JSONObject())){true};assertTrue(third.third());w.retainedResourceEditors["added"]=buffer("Untitled-other","new retained text");assertFalse(third.third())
 }
 @Test fun missingPhysicalDraftOrProviderGuardMapRefusesBeforePublication()=runBlocking{
  val w=Workspace(this);w.engine=Engine("http://127.0.0.1:1");val file=FileNode("a.py","a.py",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText("source","old"));w.buffers[file.path]=buffer;var calls=0
  val review=ResourceChangeReview{_,_,_->calls++;JSONObject().put("sources",JSONObject().put("a.py",source("source")))}
  assertTrue(runCatching{review.acquire(w,listOf("a.py"),JSONObject()){true}}.isFailure);assertEquals(0,calls)
  buffer.saveGuard="original-guard";assertTrue(runCatching{review.acquire(w,listOf("a.py"),JSONObject()){true}}.isFailure);assertEquals(1,calls);assertSame(buffer,w.buffers[file.path]);assertEquals("source",buffer.text)
 }
 @Test fun completedHistoricalReceiptIsReadableButCannotAdoptAndTypesAreStrict(){
  val raw=JSONObject().put("version",2).put("request_id","request").put("workspace","workspace").put("context","context").put("adoption_current",false).put("adoption_reason","Source was replaced after publication").put("outcome",JSONObject().put("complete",true).put("recovery","record").put("applied",JSONArray().put("a.py"))).put("saved_sources",JSONObject().put("a.py",missing())).put("source_files",JSONArray())
  val receipt=resourcePublication(raw);assertTrue(receipt.complete);assertFalse(receipt.adoptionCurrent);assertNotNull(receipt.adoptionReason)
  val patch=ResourceEditorPatch("a.py",ResourceSource("source","old"),null);assertTrue(runCatching{confirmedResourceFiles(listOf(patch),receipt,"request","workspace","context",mapOf("a.py" to ResourceSaved("missing"))){it.path to true}}.isFailure)
  for(version in listOf<Any>(1,"2",true)){raw.put("version",version);assertTrue(runCatching{resourcePublication(raw)}.isFailure)};raw.put("version",2)
  for(value in listOf<Any>("true",JSONObject.NULL)){raw.put("adoption_current",value);assertTrue(runCatching{resourcePublication(raw)}.isFailure)};raw.remove("adoption_current");assertTrue(runCatching{resourcePublication(raw)}.isFailure)
 }
 @Test fun deletionRetainsReadableEditorAndPathReuseNeverClaimsItsIdentity()=runBlocking{
  val workspace=Workspace(this);workspace.engine=Engine("http://127.0.0.1:1");val file=FileNode("a.py","a.py",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText("dirty","old"));workspace.buffers[file.path]=buffer;workspace.show(file.path)
  val patches=listOf(ResourceEditorPatch("a.py",ResourceSource("identity","dirty"),null));val state=ResourceSaved("missing");val receipt=ResourcePublication<FileNode>(2,"request","workspace","context",true,"record",listOf("a.py"),mapOf("a.py" to state),emptyList(),adoptionCurrent=true);val capture=workspace.captureResourceEditors(patches)
  workspace.adoptResourceReceipt(patches,receipt,"request","workspace","context",mapOf("a.py" to state),capture)
  assertSame(buffer,workspace.currentBuffer);assertTrue(buffer.resourceDeleted);assertTrue(buffer.readOnly);assertTrue(workspace.active!!.startsWith("deleted-resource:"));assertNull(workspace.buffers["a.py"])
  val reused=DocumentBuffer(file,FileText("different","new"));workspace.buffers["a.py"]=reused;assertSame(buffer,workspace.currentBuffer);assertNotSame(workspace.buffers["a.py"],workspace.currentBuffer)
 }
 @Test fun partialReceiptRefusesBeforeTabOrEditorMutation()=runBlocking{
  val workspace=Workspace(this);workspace.engine=Engine("http://127.0.0.1:1");val file=FileNode("a.py","a.py",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText("text","old"));workspace.buffers[file.path]=buffer;workspace.show(file.path)
  val patches=listOf(ResourceEditorPatch("a.py",ResourceSource("id","text"),null));val state=ResourceSaved("missing");val receipt=ResourcePublication<FileNode>(2,"request","workspace","context",false,"record",emptyList(),null,emptyList());val layout=workspace.layout
  assertTrue(runCatching{workspace.adoptResourceReceipt(patches,receipt,"request","workspace","context",mapOf("a.py" to state),workspace.captureResourceEditors(patches))}.isFailure)
  assertEquals(layout,workspace.layout);assertSame(buffer,workspace.currentBuffer);assertFalse(buffer.resourceDeleted);assertEquals("text",buffer.text)
 }
}
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class ResourceEngineIntegrationTest {
 @Test(timeout=180000) fun actualEngineSaveMenuUndoRedoOwnershipGuardAndLostReplyStatus()=runBlocking{
  val repo=generateSequence(java.io.File(System.getProperty("user.dir")).absoluteFile){it.parentFile}.first{it.resolve("scripts/test-resource-proposal-service.py").isFile}
  val binary=System.getenv("HICKORY_SIMPLE_TEST_ENGINE")?.let{java.io.File(it)} ?: repo.resolve("target/source-edit-focus/debug/examples/engine_client")
  check(binary.canExecute()){ "Build the focused engine_client example before running this actual-engine controller witness." }
  val temporary=java.nio.file.Files.createTempDirectory("android-resource-controller-").toFile();val root=temporary.resolve("workspace").also{it.mkdirs()};val folder=root.resolve("src").also{it.mkdir()}
  val saved="saved\n";val draft="saved\n# retained draft\n";folder.resolve("a.py").writeText(saved)
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined)
  val process=ProcessBuilder(binary.absolutePath,root.absolutePath,"window","http://127.0.0.1:1").directory(repo).redirectErrorStream(true).apply{environment()["HICKORY_STATE_DIR"]=temporary.resolve("state").absolutePath}.start()
  val lines=java.util.concurrent.LinkedBlockingQueue<String>(256)
  val reader=Thread{runCatching{process.inputStream.bufferedReader().useLines{stream->stream.forEach{lines.offer(it)}}}}.also{it.isDaemon=true;it.start()}
  var workspace:Workspace?=null
  try{
   val origin=withContext(Dispatchers.IO){val deadline=System.nanoTime()+45_000_000_000L;var result:String?=null;val seen=mutableListOf<String>();while(result==null&&System.nanoTime()<deadline){val line=lines.poll(200,java.util.concurrent.TimeUnit.MILLISECONDS);if(line!=null){seen.add(line);result=Regex("http://127\\.0\\.0\\.1:\\d+").find(line)?.value};if(!process.isAlive&&line==null)break};result ?: error("Engine fixture did not report its origin: ${seen.takeLast(5)}")}
   val engine=Engine(origin);val current=Workspace(background);workspace=current;current.engine=engine
   val tree=engine.files().second;current.rootFiles=tree;current.directoryScope="src";current.files=tree.first{it.path=="src"}.children
   val buffer=current.loadBuffer(current.files.first{it.path=="src/a.py"});buffer.language?.close();buffer.language=null;current.show(buffer.file.path)
   buffer.state.edit{replace(0,length,draft);selection=androidx.compose.ui.text.TextRange(2)};val state=buffer.state;val scroll=buffer.sourceScroll
   fun rename(from:String,to:String)=JSONObject().put("documentChanges",JSONArray().put(JSONObject().put("kind","rename").put("oldUri",languageURI(from)).put("newUri",languageURI(to))))
   val resources=current.resourceChanges
   resources.prepare(current,"Move dirty source",rename("src/a.py","src/b.py"));assertNotNull(resources.notice,resources.review);val reviewed=resources.review!!;assertEquals(saved,folder.resolve("a.py").readText());assertEquals(draft,buffer.text)
   resources.apply(current,reviewed);assertFalse(resources.notice,resources.hasPending);assertNotNull(resources.notice,resources.lastRecovery);assertFalse(folder.resolve("a.py").exists());assertEquals(draft,folder.resolve("b.py").readText());assertSame(buffer,current.currentBuffer);assertSame(state,buffer.state);assertSame(scroll,buffer.sourceScroll);assertEquals(2,buffer.state.selection.start);assertEquals("src",current.directoryScope);assertEquals(listOf("src/b.py"),current.files.map{it.path})
   suspend fun history(kind:String){current.changes.open(kind,current);withTimeout(45000){while(resources.busy||resources.review==null){delay(25);if(!resources.busy&&resources.review==null&&resources.notice.isNotEmpty()&&!resources.notice.contains("Preparing"))error(resources.notice)}}}
   history("undo")
   val added=DocumentBuffer(FileNode("new.md","new.md",false,emptyList(),null,null),FileText("<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"/>",""));current.buffers["new.md"]=added
   resources.apply(current);assertTrue(resources.notice.contains("changed",true));assertTrue(folder.resolve("b.py").exists());assertFalse(resources.hasPending);current.buffers.remove("new.md")
   history("undo");resources.apply(current);assertFalse(resources.notice,resources.hasPending);assertEquals(saved,folder.resolve("a.py").readText());assertFalse(folder.resolve("b.py").exists());assertSame(buffer,current.currentBuffer);assertEquals(draft,buffer.text);assertEquals(saved,buffer.base);assertTrue(buffer.dirty);assertSame(state,buffer.state);assertEquals(2,buffer.state.selection.start)
   history("redo");resources.apply(current);assertFalse(resources.notice,resources.hasPending);assertEquals(draft,folder.resolve("b.py").readText());assertSame(buffer,current.currentBuffer);assertFalse(folder.resolve("a.py").exists())
   // The same menu history must now see a later draft inverse before the
   // resource inverse; a new draft branch retires resource redo normally.
   val other=DocumentBuffer(FileNode("other.txt","other.txt",false,emptyList(),null,null),FileText("before",""));current.buffers["other.txt"]=other
   val edit=JSONObject().put("changes",JSONObject().put(languageURI("other.txt"),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",6))).put("newText","after"))))
   current.changes.offer("Later draft",edit,mapOf("other.txt" to "before"),current);current.changes.apply(current);assertEquals("after",other.text);current.changes.open("undo",current);assertEquals("before",other.text);assertTrue(folder.resolve("b.py").exists())
   current.buffers.remove("other.txt")
   var publications=0
   val lost=ResourceChangeReview{method,path,body->val reply=engine.request(method,path,body);if(method=="POST"&&path=="/api/changes/resource-save"){publications++;error("Deliberately lost confirmed reply")};reply}
   lost.prepare(current,"Move with lost response",rename("src/b.py","src/c.py"));assertNotNull(lost.notice,lost.review);lost.apply(current);assertEquals(1,publications);assertTrue(lost.hasPending);assertTrue(folder.resolve("c.py").exists());assertEquals("src/b.py",buffer.file.path)
   lost.apply(current);assertEquals(1,publications);lost.status(current);assertFalse(lost.notice,lost.hasPending);assertEquals(1,publications);assertSame(buffer,current.buffers["src/c.py"]);assertEquals("src/c.py",current.active)
   buffer.state.undoState.undo();assertEquals(saved,buffer.text)
   val deletion=JSONObject().put("documentChanges",JSONArray().put(JSONObject().put("kind","delete").put("uri",languageURI("src/c.py"))))
   resources.prepare(current,"Delete while retaining draft",deletion);assertNotNull(resources.notice,resources.review);resources.apply(current);assertFalse(resources.notice,resources.hasPending);assertSame(buffer,current.currentBuffer);assertTrue(buffer.resourceDeleted);assertTrue(buffer.dirty);assertTrue(current.hasUnsavedChanges)
   engine.request("POST","/api/files/op",JSONObject().put("op","create").put("path","src/c.py"))
   val reusedFile=FileNode("c.py","src/c.py",false,emptyList(),null,null);val empty=engine.file(reusedFile);engine.save(reusedFile,"independent",empty.content,empty.hash);val reused=DocumentBuffer(reusedFile,engine.file(reusedFile));current.buffers["src/c.py"]=reused;current.refresh()
   val deletedTab=current.active
   current.fileCommand="Save As";current.filePath="src/c.py";current.performFileCommand()
   assertTrue(current.status.contains("different unused path"));assertEquals(deletedTab,current.active);assertSame(buffer,current.currentBuffer);assertSame(reused,current.buffers["src/c.py"]);assertTrue(current.retainedResourceEditors.values.any{it===buffer});assertEquals("independent",folder.resolve("c.py").readText())
   engine.request("POST","/api/files/op",JSONObject().put("op","create").put("path","src/occupied.py"));current.fileCommand="Save As";current.filePath="src/occupied.py";current.performFileCommand()
   assertTrue(current.status.contains("Original edits are retained"));assertFalse(buffer.saving);assertEquals(deletedTab,current.active);assertSame(buffer,current.currentBuffer);assertTrue(current.retainedResourceEditors.values.any{it===buffer});assertEquals("",folder.resolve("occupied.py").readText())
   current.fileCommand="Save As";current.filePath="src/recovered.py";current.performFileCommand()
   assertEquals(saved,folder.resolve("recovered.py").readText());assertEquals("independent",folder.resolve("c.py").readText());assertSame(reused,current.buffers["src/c.py"]);assertFalse(current.retainedResourceEditors.values.any{it===buffer});assertFalse(current.retainedResourceTabs.values.any{it===buffer});assertFalse(current.hasUnsavedChanges)
  }finally{
   workspace?.buffers?.values?.forEach{it.language?.close()};background.cancel()
   process.destroy();if(!process.waitFor(5,java.util.concurrent.TimeUnit.SECONDS))process.destroyForcibly();reader.join(1000);temporary.deleteRecursively()
  }
 }
}
