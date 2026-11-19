package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
class CoverageBindingTest {
 @Test fun liveOwnedCollectorBridgeResultsRerunAndSourceGuards()=runBlocking {
  val origin=System.getenv("HICKORY_COLLECTOR_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_COLLECTOR_WORKSPACE") ?: error("Owned collector workspace required"));check(java.io.File(root,".hickory-collector-fixture").readText()=="owned-collector-v1\n")
  val dispatcher=java.util.concurrent.Executors.newSingleThreadExecutor().asCoroutineDispatcher();val scope=CoroutineScope(SupervisorJob()+dispatcher);val w=Workspace(scope);val engine=Engine(origin!!);w.engine=engine;val (_,tree)=engine.files();w.rootFiles=tree
  try{val file=tree.first{it.path=="app.py"};val buffer=w.loadBuffer(file);w.show(sourceTab(file.path));w.showCoverage=true;val source=buffer.text;val textState=buffer.state;val scroll=buffer.sourceScroll;val state=w.coverage
   val pending=java.util.concurrent.ConcurrentHashMap<Int,CompletableDeferred<JSONObject>>();var sequence=0;var attached=true
   val bridge=NativeCoverageController(w,scope,{attached},{id,value,error->pending.remove(id)?.let{if(error!=null)it.completeExceptionally(IllegalStateException(error)) else it.complete(value as? JSONObject ?: JSONObject())}},{})
   suspend fun request(kind:String,payload:JSONObject):JSONObject {val id=++sequence;val answer=CompletableDeferred<JSONObject>();pending[id]=answer;val message=JSONObject().put("kind",kind).put("id",id).put("workspace",state.identity);if(kind=="api")message.put("body",payload) else message.put("location",payload);withContext(dispatcher){bridge.receive(message)};return withTimeout(60000){answer.await()}}
   suspend fun api(action:String,id:String?=null)=request("api",JSONObject().put("action",action).put("path",".").put("language","python").also{if(id!=null)it.put("id",id)})
   suspend fun finished(id:String):JSONObject {return withTimeout(60000){var result=api("get",id);while(result.getString("status")=="running"){delay(50);result=api("get",id)};result}}
   val listing=api("discover");assertEquals(2,listing.getJSONArray("tests").length());assertFalse(java.io.File(root,".hick-cache/executions").exists());val first=finished(api("run").getString("id"));assertEquals("failed",first.getString("status"));assertEquals(2,first.getJSONArray("tests").length());assertEquals("current",first.getString("source_state"));assertTrue(first.getJSONObject("evidence").getJSONObject("report").getJSONObject("files").has("app.py"))
   withContext(dispatcher){bridge.receive(JSONObject().put("kind","select").put("workspace",state.identity).put("id",first.getString("id")))};withTimeout(10000){while(buffer.coverageRows().isEmpty())delay(20)}
   val second=finished(api("rerun-failed",first.getString("id")).getString("id"));assertEquals("failed",second.getString("status"));assertEquals(1,second.getJSONArray("tests").length());assertEquals("test_app.py::test_negative",second.getJSONArray("tests").getJSONObject(0).getString("id"));assertNotEquals(first.getString("id"),second.getString("id"));assertEquals(2,api("list").getJSONArray("runs").length())
   val location=JSONObject().put("path",file.path).put("line",2).put("sourceHash",debugHash(source));request("navigate",location);assertFalse(w.showCoverage);assertEquals(source.indexOf('\n')+1,buffer.state.selection.start);assertSame(textState,buffer.state);assertSame(scroll,buffer.sourceScroll)
   buffer.applySourceEdits(listOf(NativeSourceEdit(buffer.text.length,buffer.text.length,"# unsaved after measurement\n")),buffer.text);val draft=buffer.text;val selection=buffer.state.selection;assertTrue(buffer.coverageRows().isEmpty());w.showCoverage=true;val stale=runCatching{request("navigate",location)};assertTrue(stale.isFailure);assertTrue(w.showCoverage);assertEquals(draft,buffer.text);assertEquals(selection,buffer.state.selection);assertSame(textState,buffer.state);assertSame(scroll,buffer.sourceScroll)
   attached=false;withContext(dispatcher){bridge.receive(JSONObject().put("kind","select").put("workspace",state.identity).put("id","foreign"))};assertEquals(first.getString("id"),state.selected);assertEquals(source,java.io.File(root,"app.py").readText())
  }finally{w.buffers.values.forEach{it.language?.close()};w.coverage.stop();w.sourceTestIndex?.stop();scope.cancel();dispatcher.close()}
 }

 @Test fun navigationRetiresClosedForeignAndEqualByteReplies()=runBlocking {
  for(mode in listOf("closed","engine","owner","undo","selection","fraction","boolean","success","unbound")){
   val w=Workspace(this);val engine=Engine("http://127.0.0.1:1");val path="literal %?#.py";val f=FileNode(path,path,false,emptyList(),null,null);val source="first\nsecond\n";val b=DocumentBuffer(f,FileText(source,"base"));w.engine=engine;w.buffers[path]=b;w.show(sourceTab(path));w.showCoverage=true;b.state.edit{selection=androidx.compose.ui.text.TextRange(2)}
   val state=w.coverage;var calls=0;val value=JSONObject().put("path",path).put("line",2).put("sourceHash",debugHash(source));if(mode=="fraction")value.put("line",1.5);if(mode=="boolean")value.put("line",true);if(mode=="unbound"){value.put("line",JSONObject.NULL);value.put("sourceHash",JSONObject.NULL)}
   val current:()->Boolean={calls++;if(mode=="engine"&&calls==2)w.engine=Engine("http://127.0.0.1:2");if(calls==3){when(mode){"undo"->{b.state.edit{replace(0,length,"later")};b.advanceSourceRevision();b.state.edit{replace(0,length,source)};b.advanceSourceRevision()};"selection"->b.state.edit{selection=androidx.compose.ui.text.TextRange(1)};"owner"->w.buffers[path]=DocumentBuffer(f,FileText(source,"base"))}};mode!="closed"||calls<2}
   val result=runCatching{w.navigateCoverage(value,state,current)};if(mode=="success"||mode=="unbound")assertEquals("$mode: "+result.exceptionOrNull(),true,result.getOrNull()) else assertNotEquals(mode,true,result.getOrNull())
   if(mode=="success"){assertEquals(6,b.state.selection.start);assertFalse(w.showCoverage)}else if(mode=="unbound"){assertEquals(2,b.state.selection.start);assertFalse(w.showCoverage)}else assertTrue(mode,w.showCoverage)
   assertEquals(source,b.text);assertEquals(source,b.base)
  }
 }

 @Test fun heldCoverageLinksRespectLatestRequestAndForegroundChoice() = runBlocking {
  val source = "first\nsecond\n"
  val paths = listOf("older.py", "newer.py", "retired.py")
  val arrived = paths.associateWith { CompletableDeferred<Unit>() }
  val gates = paths.associateWith { java.util.concurrent.CountDownLatch(1) }
  val listener = com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1", 0), 0)
  val requests = java.util.concurrent.Executors.newFixedThreadPool(3)
  listener.executor = requests
  listener.createContext("/api/file") { exchange ->
   val path = exchange.requestURI.query.removePrefix("path=")
   arrived.getValue(path).complete(Unit)
   check(gates.getValue(path).await(8, java.util.concurrent.TimeUnit.SECONDS))
   val bytes = JSONObject().put("content", source).put("hash", "saved").toString().toByteArray()
   exchange.sendResponseHeaders(200, bytes.size.toLong())
   exchange.responseBody.use { it.write(bytes) }
  }
  listener.start()
  fun file(path: String) = FileNode(path, path, false, emptyList(), null, null)
  val workspace = Workspace(this)
  workspace.engine = Engine("http://127.0.0.1:${listener.address.port}")
  val draft = DocumentBuffer(file("draft.py"), FileText("local", "saved"))
  val other = DocumentBuffer(file("other.py"), FileText("other", "saved"))
  val selection = androidx.compose.ui.text.TextRange(2, 4)
  draft.state.edit { append(" draft"); this.selection = selection }
  workspace.rootFiles = paths.map(::file)
  workspace.buffers[draft.file.path] = draft
  workspace.buffers[other.file.path] = other
  workspace.show(sourceTab(other.file.path))
  workspace.show(sourceTab(draft.file.path))
  workspace.showCoverage = true
  val state = workspace.coverage
  val pending = mutableMapOf<Int, CompletableDeferred<Pair<Any, String?>>>()
  val bridge = NativeCoverageController(workspace, this, { true }, { id, value, error ->
   pending.getValue(id).complete(value to error)
  }, {})
  fun message(kind: String) = JSONObject().put("kind", kind).put("workspace", state.identity)
  fun navigate(id: Int, path: String, runId: String? = null): CompletableDeferred<Pair<Any, String?>> {
   val answer = CompletableDeferred<Pair<Any, String?>>()
   pending[id] = answer
   bridge.receive(message("navigate").put("id", id).put("location", JSONObject().put("path", path)
    .put("line", 2).put("sourceHash", debugHash(source)).apply { if (runId != null) put("runId", runId) }))
   return answer
  }
  try {
   assertEquals(false to null, withTimeout(4000) { navigate(5, paths[0], "foreign-report").await() })
   assertFalse(arrived.getValue(paths[0]).isCompleted)
   val older = navigate(1, paths[0])
   withTimeout(4000) { arrived.getValue(paths[0]).await() }
   val newer = navigate(2, paths[1])
   withTimeout(4000) { arrived.getValue(paths[1]).await() }
   gates.getValue(paths[0]).countDown()
   assertEquals(false to null, withTimeout(4000) { older.await() })
   assertTrue(workspace.showCoverage)
   assertSame(draft, workspace.currentBuffer)

   workspace.show(sourceTab(other.file.path))
   workspace.show(sourceTab(draft.file.path))
   val layout = workspace.layout
   val trail = workspace.navigationTrail
   gates.getValue(paths[1]).countDown()
   assertEquals(false to null, withTimeout(4000) { newer.await() })

   val retired = navigate(3, paths[2])
   withTimeout(4000) { arrived.getValue(paths[2]).await() }
   bridge.receive(message("retire-navigation"))
   gates.getValue(paths[2]).countDown()
   assertEquals(false to null, withTimeout(4000) { retired.await() })
   assertSame(layout, workspace.layout)
   assertSame(trail, workspace.navigationTrail)
   assertSame(draft, workspace.currentBuffer)
   assertEquals("local draft", draft.text)
   assertEquals(selection, draft.state.selection)
   assertTrue(workspace.showCoverage)
   paths.forEach { assertEquals(source, workspace.buffers.getValue(it).text) }

   assertEquals(true to null, withTimeout(4000) { navigate(4, paths[1]).await() })
   assertFalse(workspace.showCoverage)
   assertSame(workspace.buffers.getValue(paths[1]), workspace.currentBuffer)
   assertEquals(androidx.compose.ui.text.TextRange(6), workspace.currentBuffer?.state?.selection)
   assertEquals(selection, draft.state.selection)
  } finally {
   gates.values.forEach { it.countDown() }
   workspace.buffers.values.forEach { it.language?.close() }
   workspace.coverage.stop()
   listener.stop(0)
   requests.shutdownNow()
  }
 }

 @Test fun mixedOwnerMarksRetireImmediatelyAfterExactByteEdit()=runBlocking {
  val source="// é\nvalue\n";val path="owner.kt";val hash=java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}
  val record=JSONObject().put("id","measured").put("source_changed_during_run",false).put("identity",JSONObject().put("source_hashes",JSONObject().put(path,hash).put("generated.kt","a".repeat(64))))
   .put("evidence",JSONObject().put("report",JSONObject().put("files",JSONObject().put("generated.kt",JSONObject().put("line_counts",false).put("lines",JSONObject().put("1",1).put("2",0))))))
   .put("mappings",JSONObject().put("generated.kt",JSONObject().put("1",JSONObject().put("path",path).put("line",2)).put("2",JSONObject().put("path",path).put("line",2))))
  val listener=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0)
  listener.createContext("/api/tests/coverage"){exchange->val bytes=record.toString().toByteArray();exchange.requestBody.close();exchange.sendResponseHeaders(200,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}};listener.start()
  val workspace=Workspace(this);workspace.engine=Engine("http://127.0.0.1:${listener.address.port}");val state=workspace.coverage;state.digests=SourceDigests(this)
  val buffer=DocumentBuffer(FileNode(path,path,false,emptyList(),null,null),FileText(source,hash));buffer.coverageSource=state;workspace.buffers[path]=buffer
  suspend fun settle(condition:()->Boolean){withTimeout(3000){while(!condition()){delay(2)}}}
  try{state.select("measured",workspace);settle{buffer.coverageRows().isNotEmpty()};val row=buffer.coverageRows().first();assertEquals(1,row.line);assertEquals("mixed",row.state);assertNull(row.count);assertEquals(2,row.origins.size)
   buffer.state.edit{replace(0,length,"// e\u0301\nvalue\n")};assertTrue(buffer.coverageRows().isEmpty());buffer.readCoverageSource();settle{state.digests!!.current(path,buffer.text)!=null};assertTrue(buffer.coverageRows().isEmpty())
   buffer.state.edit{replace(0,length,source)};buffer.readCoverageSource();settle{buffer.coverageRows().isNotEmpty()};assertEquals("mixed",buffer.coverageRows().first().state);state.select(null,workspace);assertTrue(buffer.coverageRows().isEmpty())
  }finally{state.stop();listener.stop(0)}
 }
}
