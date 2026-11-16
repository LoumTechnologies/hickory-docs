package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import org.json.JSONArray
import kotlinx.coroutines.*
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class GitConflictResolutionTest {
 private fun side(identity:String,text:String)=JSONObject().put("kind","file").put("identity",identity).put("text",text)
 private fun preview()=JSONObject().put("members",JSONArray().put("a.py")).put("remaining_conflicts",JSONArray()).put("version",1).put("resolution_protocol",1).put("token","a".repeat(32)).put("fingerprint","b".repeat(64)).put("source_files_modified",false).put("index_modified",false).put("needs_save",true).put("stage_available",true).put("stage_unavailable",JSONObject.NULL).put("plan",JSONObject().put("patches",JSONArray().put(JSONObject().put("path","a.py").put("before",side("a","before")).put("after",side("a","after"))))).put("saved_sources",JSONObject().put("a.py",JSONObject().put("kind","file").put("text","after").put("sha256",debugHash("after")).put("readonly",false).put("executable",false).put("mode",420))).put("consequences",JSONObject())
 @Test fun immutableReviewRejectsMalformedIdentityAndDistinguishesResourceDrafts(){val raw=preview();val value=gitResolutionPreview(raw){true};assertTrue(gitResolutionDraftable(value));assertFalse(gitResolutionDraftable(value.copy(patches=listOf(ResourceEditorPatch("a.py",ResourceSource("a","same"),ResourceSource("a","same"))))));raw.getJSONObject("plan").getJSONArray("patches").getJSONObject(0).getJSONObject("after").put("text","tampered");assertEquals("after",value.patches.single().after!!.text)
  val staged=JSONObject().put("version",1).put("operation","git_conflict_stage").put("request_id","request").put("token",value.token).put("review_fingerprint",value.fingerprint).put("fingerprint","c".repeat(64)).put("complete",true).put("index_modified",true).put("index_uncertain",false).put("source_files_modified",false).put("resolved_members",JSONArray().put("a.py")).put("remaining_conflicts",JSONArray())
  gitResolutionStageReceipt(staged,"request",value)
  for((key,bad) in listOf("request_id" to "foreign","resolved_members" to JSONArray(),"remaining_conflicts" to JSONArray().put("a.py"),"complete" to "true","fingerprint" to "bad"))assertTrue(runCatching{gitResolutionStageReceipt(JSONObject(staged.toString()).put(key,bad),"request",value)}.isFailure)
  for((key,bad) in listOf("version" to true,"resolution_protocol" to 1.5,"token" to "foreign","fingerprint" to "old","index_modified" to true,"needs_save" to "false","stage_available" to JSONObject.NULL))assertTrue(runCatching{gitResolutionPreview(preview().put(key,bad)){true}}.isFailure)
  assertFalse(gitResolutionDraftable(value.copy(patches=listOf(ResourceEditorPatch("a.py",ResourceSource("a","before"),null)))))
  assertFalse(gitResolutionDraftable(value.copy(patches=listOf(ResourceEditorPatch("a.py",ResourceSource("a","before"),ResourceSource("other","after"))))))
 }
 @Test fun explicitDestinationsDoNotInferRenameAndKeepStageObjectIdentity(){val output=GitResolutionOutput("new λ%?#.py","left.py","composed");val raw=output.json();assertEquals("left.py",raw.getString("identity_from"));assertEquals("text",raw.getJSONObject("content").getString("kind"));assertEquals("composed",raw.getJSONObject("content").getString("text"));val stage=output.copy(stagePath="right.py",stage=3,oid="c".repeat(40));assertEquals("left.py",stage.json().getString("identity_from"));assertEquals("right.py",stage.json().getJSONObject("content").getString("path"));assertEquals("c".repeat(40),stage.json().getJSONObject("content").getString("oid"));assertTrue(output.copy(identityFrom=null).json().isNull("identity_from"))}
 @Test fun closedAndSupersededReadsCannotReviveComposition()=runBlocking{
  val waits=mutableListOf<CompletableDeferred<JSONObject>>();val model=GitResolutionModel{_,_,_->CompletableDeferred<JSONObject>().also{waits.add(it)}.await()};val workspace=Workspace(this)
  fun read(path:String)=JSONObject().put("version",1).put("resolution_protocol",1).put("paths",JSONArray().put(JSONObject().put("path",path).put("working",JSONObject.NULL).put("stages",JSONArray())))
  val pending=async{model.read(workspace,"a.py")};while(waits.isEmpty())yield();model.close();waits[0].complete(read("a.py"));pending.await();assertNull(model.reading);assertFalse(model.busy)
  model.add();model.output(0,GitResolutionOutput("chosen.py",null,"my composition"));val next=async{model.read(workspace,"a.py")};while(waits.size<2)yield();waits[1].complete(read("a.py"));next.await();assertEquals("my composition",model.outputs.single().text);assertEquals("chosen.py",model.outputs.single().path)
 }
 private suspend fun host(block:suspend(Workspace,DocumentBuffer,GitResolutionModel,MutableList<String>,java.util.concurrent.atomic.AtomicReference<String>)->Unit)=coroutineScope{
  val server=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0)
  server.createContext("/api/changes/resource-inputs"){exchange->val bytes=JSONObject().put("saved_guards",JSONObject().put("a.py","saved-guard")).put("sources",JSONObject().put("a.py",JSONObject().put("kind","file"))).toString().toByteArray();exchange.sendResponseHeaders(200,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}};server.start()
  val w=Workspace(this);w.engine=Engine("http://127.0.0.1:${server.address.port}");val file=FileNode("a.py","a.py",false,emptyList(),null,null);val b=DocumentBuffer(file,FileText("before","saved","saved-guard"));w.buffers[file.path]=b;val calls=mutableListOf<String>();val mode=java.util.concurrent.atomic.AtomicReference("pending")
  val model=GitResolutionModel{_,path,body->calls.add(path);when{
   path.endsWith("/read")->JSONObject().put("version",1).put("resolution_protocol",1).put("paths",JSONArray().put(JSONObject().put("path","a.py").put("stages",JSONArray()).put("working",JSONObject().put("kind","file").put("text","before"))))
   path.endsWith("resource-inputs")->JSONObject().put("saved_guards",JSONObject().put("a.py","saved-guard")).put("sources",JSONObject().put("a.py",JSONObject().put("kind","file")))
   path.endsWith("/prepare")->preview()
   path.endsWith("/save")->error("lost response")
   path.contains("/status?")->JSONObject().put("version",1).put("request_id",path.substringAfter("request_id=")).put("state",mode.get()).put("outcome",JSONObject.NULL).put("error",if(mode.get()=="completed")"explicit refusal" else JSONObject.NULL)
   path.endsWith("/validate")->JSONObject().put("current",true).put("stage",body?.optBoolean("stage") ?: false).put("token","a".repeat(32)).put("fingerprint","b".repeat(64)).put("plan",preview().getJSONObject("plan")).put("saved_sources",preview().getJSONObject("saved_sources"))
   else->error("Unexpected route $path")
  }}
  try{block(w,b,model,calls,mode)}finally{server.stop(0)}
 }
 @Test fun returnedTextRevisionAndNewConstructionBufferInvalidateWholeReview()=runBlocking{for(change in listOf("revision","namespace","metadata","baseline"))host{w,b,m,calls,_->m.read(w,"a.py");m.prepare(w);assertNotNull(m.notice,m.preview);val lease=w.captureRetainedSourceNamespace();assertTrue(lease());if(change=="revision")b.advanceSourceRevision() else if(change=="metadata")b.file=b.file.copy(generatedBy="owner") else if(change=="baseline")b.base="foreign baseline" else w.buffers["new.md"]=DocumentBuffer(FileNode("new.md","new.md",false,emptyList(),null,null),FileText("draft","saved"));assertFalse(lease());m.act(w,false);assertFalse(m.hasPending);assertFalse(calls.any{it.endsWith("/save")});assertEquals("before",b.text)}}
 @Test fun uncertainRequestNeverRetriesAndOnlyCompletedOriginalPreimageCanRetire()=runBlocking{host{w,b,m,calls,mode->m.read(w,"a.py");m.prepare(w);m.act(w,false);assertTrue(m.hasPending);m.close();m.act(w,false);assertEquals(1,calls.count{it.endsWith("/save")});m.status(w);assertTrue(m.hasPending);mode.set("unknown");m.status(w);assertTrue(m.hasPending);mode.set("completed");m.status(w);assertFalse(m.notice,m.hasPending);assertNull(m.preview);assertEquals("before",b.text);assertEquals(2,calls.count{it.endsWith("/validate")})}}
 @Test(timeout=120000) fun liveNativeCompositionSaveStageAndEditorHistory()=runBlocking{
  val origin=System.getenv("HICKORY_NATIVE_GIT_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_NATIVE_GIT_WORKSPACE") ?: error("Use the owned Git resolution fixture"));val path=System.getenv("HICKORY_NATIVE_GIT_INPUT") ?: error("Missing literal fixture path");check(root.name=="native"&&root.parentFile.name.startsWith("hickory-git-resolutions-")){"Refuse outside owned temporary fixture"}
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(background)
  try{
  w.engine=Engine(origin!!);val (_,tree)=w.engine!!.files();assertEquals(root.toPath().toRealPath().toString(),w.engine!!.sourceRootAddress);w.rootFiles=tree
  fun find(nodes:List<FileNode>):FileNode?=nodes.firstNotNullOfOrNull{if(it.path==path)it else find(it.children)}
  val b=w.loadBuffer(find(tree) ?: error("Literal conflict source missing"));w.show(path);val state=b.state;val old=b.text;val scroll=b.sourceScroll.value;b.selectSourceRange(0,0,old)
  val m=w.gitResolution;m.read(w,path);assertNotNull(m.notice,m.reading);m.output(0,GitResolutionOutput(path,path,"composed = 41\n"));m.prepare(w);assertNotNull(m.notice,m.preview);assertEquals(old,b.text)
  m.draft(w);assertEquals(m.notice,"composed = 41\n",b.text);w.changes.open("undo",w);assertEquals(old,b.text);w.changes.open("redo",w);assertEquals("composed = 41\n",b.text);w.changes.open("undo",w);assertEquals(old,b.text)
  m.read(w,path,true);m.prepare(w);assertNotNull(m.notice,m.preview);m.act(w,false);assertFalse(m.notice,m.hasPending);assertTrue(m.notice,m.saved);assertEquals("composed = 41\n",root.resolve(path).readText());assertEquals("composed = 41\n",b.base);assertSame(state,b.state);assertSame(b,w.currentBuffer);assertEquals(scroll,b.sourceScroll.value)
  m.act(w,true);assertFalse(m.notice,m.hasPending);assertTrue(m.notice,m.notice.contains("staged"));assertSame(state,b.state);assertSame(b,w.buffers[path]);assertEquals("composed = 41\n",b.text)
  }finally{w.buffers.values.forEach{it.language?.close()};background.cancel()}
 }
}
