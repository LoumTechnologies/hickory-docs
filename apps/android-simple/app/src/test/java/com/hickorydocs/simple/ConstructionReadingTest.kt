package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import kotlinx.coroutines.*
class ConstructionReadingTest {
 @Test fun boundEntryOverridePreservesLiteralAddress()=kotlinx.coroutines.runBlocking {
  val model=ConstructionReadingModel()
  val address=" folder/entry.md "
  model.read(entryOverride=address){entry,focus->
   assertEquals(address,entry)
   assertNull(focus)
   value(entry)
  }
  assertEquals(address,model.value?.entry)
 }
 fun value(entry:String="project.md")=ConstructionReading(entry,null,emptyList(),0,false,emptyList(),0,false,"declaration","not_queried",emptyList(),0)
 @Test fun closedAndSupersededRepliesCannotReplaceTheCapturedReading()=runBlocking{
  val model=ConstructionReadingModel();val pending=CompletableDeferred<ConstructionReading>();val old=async(start=CoroutineStart.UNDISPATCHED){model.read{_,_->pending.await()}}
  model.read{_,_->value().copy(notices=listOf("new"))};pending.complete(value());old.await();assertEquals(listOf("new"),model.value?.notices)
  val late=CompletableDeferred<ConstructionReading>();val closed=async(start=CoroutineStart.UNDISPATCHED){model.read{_,_->late.await()}};model.close();late.complete(value());closed.await();assertNull(model.value)
  model.read{_,_->value("other.md")};assertNull(model.value);assertTrue(model.notice.contains("identity changed"))
 }
 @Test fun sourcePointsRequireExactHashesAndScalarBoundariesAndRetireOlderLinks()=runBlocking{
  val source="😀e\u0301\né😀code";val hash=java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}
  val point=TestSourcePoint("part.md",2,3,hash)
  assertEquals(source.indexOf("code"),constructionPointOffset(source,point,hash))
  assertNull(constructionPointOffset(source,point.copy(character=2),hash));assertNull(constructionPointOffset(source,point.copy(line=0),hash));assertNull(constructionPointOffset(source,point.copy(character=99),hash));assertNull(constructionPointOffset(source,point,"b".repeat(64)))
  val model=ConstructionReadingModel();model.read{_,_->value()};val old=model.sourceLink();val latest=model.sourceLink();assertFalse(old());assertTrue(latest());model.close();assertFalse(latest())
 }

 @Test fun heldConstructionSourceRespectsNewerForegroundChoice() = runBlocking {
  val source = "first\né😀code\n"
  val path = "part.md"
  val arrived = CompletableDeferred<Unit>()
  val release = java.util.concurrent.CountDownLatch(1)
  val listener = com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1", 0), 0)
  fun respond(exchange: com.sun.net.httpserver.HttpExchange, body: JSONObject) {
   val bytes = body.toString().toByteArray(Charsets.UTF_8)
   exchange.sendResponseHeaders(200, bytes.size.toLong())
   exchange.responseBody.use { it.write(bytes) }
  }
  listener.createContext("/api/construction/reading") { exchange ->
   check(exchange.requestMethod == "POST")
   check(JSONObject(exchange.requestBody.bufferedReader().use { it.readText() }).getString("entry") == "project.md")
   respond(exchange, JSONObject("""{
    "version":1,"entry":"project.md","focus":null,"materialized":false,"output_count":1,"outputs_truncated":false,
    "test_count":0,"tests_truncated":false,"test_relationship":"declaration","evidence_status":"not_queried",
    "notices":[],"unused_sources":[],"tests":[],"outputs":[{"path":"output.txt","change":"modified",
    "contributors":["$path"],"origins_truncated":false,"origins":[{"phase":"draft","path":"$path",
    "source_bytes":[0,1],"output_bytes":[0,1],"exact":true,"source_point":{
    "path":"$path","line":2,"character":3,"hash":"${debugHash(source)}"}}]}]}
   """))
  }
  listener.createContext("/api/file") { exchange ->
   check(exchange.requestURI.query == "path=$path")
   arrived.complete(Unit)
   check(release.await(8, java.util.concurrent.TimeUnit.SECONDS))
   respond(exchange, JSONObject().put("content", source).put("hash", "saved"))
  }
  listener.start()
  fun file(path: String) = FileNode(path, path, false, emptyList(), null, null)
  val engine = Engine("http://127.0.0.1:${listener.address.port}")
  val workspace = Workspace(this)
  workspace.engine = engine
  workspace.rootFiles = listOf(file(path))
  val draft = DocumentBuffer(file("project.md"), FileText("local", "saved"))
  val other = DocumentBuffer(file("other.md"), FileText("other tab", "saved"))
  val selection = androidx.compose.ui.text.TextRange(2, 4)
  draft.state.edit { append(" draft"); this.selection = selection }
  draft.sourceScroll.scrollTo(9)
  workspace.buffers[draft.file.path] = draft
  workspace.buffers[other.file.path] = other
  workspace.show(sourceTab(other.file.path))
  workspace.show(sourceTab(draft.file.path))
  workspace.showConstruction = true
  val model = ConstructionReadingModel()
  try {
   model.read { entry, focus ->
    val body = JSONObject().put("entry", entry).put("focus", focus ?: JSONObject.NULL)
    constructionReading(engine.request("POST", "/api/construction/reading", body), entry, focus)
   }
   val reading = checkNotNull(model.value)
   val point = checkNotNull(reading.outputs.single().origins.single().point)
   val navigate = model.sourceNavigation(workspace, point)
   val pending = async { navigate() }
   withTimeout(4000) { arrived.await() }
   workspace.show(sourceTab(other.file.path))
   workspace.show(sourceTab(draft.file.path))
   val layout = workspace.layout
   val trail = workspace.navigationTrail
   release.countDown()
   pending.await()
   assertSame(draft, workspace.currentBuffer)
   assertSame(layout, workspace.layout)
   assertSame(trail, workspace.navigationTrail)
   assertSame(reading, model.value)
   assertTrue(workspace.showConstruction)
   assertEquals("local draft", draft.text)
   assertEquals(selection, draft.state.selection)
   assertEquals(9, draft.sourceScroll.value)
   val cached = workspace.buffers.getValue(path)
   assertEquals(source, cached.text)

   model.sourceNavigation(workspace, point)()
   assertSame(cached, workspace.currentBuffer)
   assertEquals(androidx.compose.ui.text.TextRange(source.indexOf("code")), cached.state.selection)
   assertFalse(workspace.showConstruction)
   assertEquals(selection, draft.state.selection)
  } finally {
   release.countDown()
   workspace.buffers.values.forEach { it.language?.close() }
   listener.stop(0)
  }
 }

}
