<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read captured compiler constructions

For maintainers changing workspace services independently of the editor renderer.

The reading owns a replaceable read adapter and retires pending replies on close or a newer request. Check the captured entry and focus identity before accepting a response. The compiler owns contribution ranges and output changes; the frontend does not infer test execution from declarations or authorize source edits from these facts.

<ui:copy id="construction-reading-android-model">
package com.hickorydocs.simple
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import org.json.JSONObject

data class ConstructedSnapshot(val kind:String,val executable:Boolean,val size:Long,val preview:String?,val abbreviated:Boolean,val hash:String?=null)
data class ConstructionFragment(val name:String,val kind:String,val body:List<Int>)
data class ConstructionOrigin(val phase:String,val path:String,val source:List<Int>,val output:List<Int>,val exact:Boolean,val point:TestSourcePoint?=null,val fragment:ConstructionFragment?=null)
data class ConstructionOutput(val path:String,val change:String,val before:ConstructedSnapshot?,val after:ConstructedSnapshot?,val contributors:List<String>,val origins:List<ConstructionOrigin>,val truncated:Boolean)
data class ConstructionReading(val entry:String,val focus:String?,val outputs:List<ConstructionOutput>,val count:Int,val truncated:Boolean,val tests:List<String>,val testCount:Int,val testsTruncated:Boolean,val relationship:String,val evidence:String,val notices:List<String>,val unused:Int)
private fun org.json.JSONArray.strings()=(0 until length()).map{getString(it)}
fun constructionReading(raw:JSONObject,entry:String,focus:String?):ConstructionReading {
 check(raw.getInt("version")==1&&raw.getString("entry")==entry&&(if(raw.isNull("focus"))null else raw.getString("focus"))==focus&&!raw.getBoolean("materialized")){"Construction identity changed or unsupported response. Refresh the reading."}
 fun snapshot(raw:JSONObject?):ConstructedSnapshot?=raw?.let{ConstructedSnapshot(it.getString("kind"),it.getBoolean("executable"),it.getLong("size"),if(it.isNull("preview"))null else it.getString("preview"),it.getBoolean("preview_truncated"),it.optString("hash").takeIf{hash->hash.matches(Regex("[0-9a-f]{64}"))})}
 val outputs=raw.getJSONArray("outputs");val tests=raw.getJSONArray("tests")
 return ConstructionReading(entry,focus,(0 until outputs.length()).map{index->val item=outputs.getJSONObject(index);val origins=item.getJSONArray("origins");ConstructionOutput(item.getString("path"),item.getString("change"),snapshot(item.optJSONObject("before")),snapshot(item.optJSONObject("after")),item.getJSONArray("contributors").strings(),(0 until origins.length()).map{val origin=origins.getJSONObject(it);val bytes = {key:String -> origin.getJSONArray(key).let{array->(0 until array.length()).map{array.getInt(it)}}};ConstructionOrigin(origin.getString("phase"),origin.getString("path"),bytes("source_bytes"),bytes("output_bytes"),origin.getBoolean("exact"),origin.optJSONObject("source_point")?.let{TestSourcePoint.decode(it)},origin.optJSONObject("fragment")?.let{fragment->val body=fragment.getJSONArray("body");ConstructionFragment(fragment.getString("name"),fragment.getString("kind"),(0 until body.length()).map{body.getInt(it)})})},item.getBoolean("origins_truncated"))},raw.getInt("output_count"),raw.getBoolean("outputs_truncated"),(0 until tests.length()).map{val test=tests.getJSONObject(it);val point=test.optJSONObject("source") ?: test.getJSONObject("target");"${test.getString("name")} · ${point.getString("path")}:${point.getInt("line")}"},raw.getInt("test_count"),raw.getBoolean("tests_truncated"),raw.getString("test_relationship"),raw.getString("evidence_status"),raw.getJSONArray("notices").strings(),raw.getJSONArray("unused_sources").length())
}
class ConstructionReadingModel {
 var entry by mutableStateOf("project.md");var focus by mutableStateOf("");var selected by mutableStateOf("");var after by mutableStateOf(true);var value by mutableStateOf<ConstructionReading?>(null);var notice by mutableStateOf("");var loading by mutableStateOf(false);private var ticket=0L;private var linkTicket=0L
 fun sourceLink():()->Boolean {val reading=ticket;val link=++linkTicket;return {ticket==reading&&linkTicket==link&&value!=null}}
 fun close(){ticket++;loading=false;value=null}
 suspend fun read(entryOverride:String?=null,load:suspend (String,String?)->ConstructionReading){val entry=entryOverride ?: entry.trim();val focus=focus.trim().takeIf{it.isNotEmpty()};val id=++ticket;value=null;loading=true;notice="Reading construction…"
 try{val reply=load(entry,focus);if(ticket!=id)return;check(reply.entry==entry&&reply.focus==focus){"Construction identity changed. Refresh the reading."};value=reply;selected="";notice="Captured construction and draft consequences. Editor changes after capture are not included."}catch(error:CancellationException){throw error}catch(error:Exception){if(ticket==id)notice=error.message.orEmpty()}finally{if(ticket==id)loading=false}}
}
</ui:copy>

## Compare captured outputs without resizing the editor

The change badge compares reconstructions from the saved entry and the captured drafts.
Use Files to inspect the working copy. The preview includes the digest of the
full reconstructed output so an abbreviated display can still identify its bytes.

Use the existing floating surface and menus. A blank field reads the bound
workspace entry. A typed address retains its literal bytes. Capture the draft
roster and original intent before requesting the workspace entry. Changing
the address, contributor, or captured source retires the pending reading.

Label the entry, focus, preview side, binary output and every truncation.
Only service-supplied source points with matching acquired source hashes and
valid scalar positions enable navigation. Closing retires the pending reading
and leaves the current editor's focus, selection and viewport intact. A focus
request on Close can reopen the keyboard and scroll the editor to its caret.

<ui:copy id="construction-reading-android-view">
@Composable fun ConstructionReadingView(workspace:Workspace){
 val model=remember{ConstructionReadingModel()}
 val scope=rememberCoroutineScope()
 var address by remember{mutableStateOf("")}
 var readingIntent by remember{mutableLongStateOf(0L)}
 var acquiring by remember{mutableStateOf(false)}
 fun retireReading(){readingIntent++;acquiring=false;model.close();workspace.behaviorReading.close();model.notice=""}
 DisposableEffect(model){onDispose{readingIntent++;model.close();workspace.behaviorReading.close()}}
 suspend fun read(connectedOnly:Boolean=false) {
  val engine=workspace.engine ?: run{model.notice="Connect to a workspace server first.";return}
  val id=++readingIntent
  val namespace=engine.sourceRootAddress
  val runtime=engine.sourceWorkspaceID
  val lease=workspace.captureRetainedSourceNamespace()
  val focus=model.focus
  fun current()=id==readingIntent&&model.focus==focus&&workspace.showConstruction&&workspace.engine===engine&&
   engine.sourceRootAddress==namespace&&engine.sourceWorkspaceID==runtime&&lease()
  if(connectedOnly)workspace.behaviorReading.close() else model.close()
  try {
   var entry=address.takeUnless{it.isBlank()}.orEmpty()
   var proof:WorkspaceEntrySelectionProof?=null
   if(entry.isEmpty()) {
    check(namespace!=null&&runtime!=null){"Refresh Files to bind the current workspace entry."}
    acquiring=true
    model.notice="Reading the current workspace entry…"
    val selected=WorkspaceEntrySelections.read(
     load={engine.request("GET","/api/workspace/entry")},workspace=runtime,namespace=namespace,current=::current,
    )
    entry=selected.entryPath ?: error("This workspace has no default construction entry. Enter an authored path.")
    proof=selected.proof
   }
   if(!current())return
   acquiring=false
   if(connectedOnly) {
    workspace.behaviorReading.read(workspace,entry,focus.trim().takeIf{it.isNotEmpty()},proof,::current)
   }else{
    model.entry=entry
    val sources=JSONObject(workspace.buffers.values.filter{!it.readOnly}.associate{it.file.path to it.text})
    model.read(entryOverride=entry){path,focus->
     val body=JSONObject().put("entry",path).put("focus",focus ?: JSONObject.NULL).put("sources",sources)
     proof?.let{body.put("entry_selection",it.body())}
     val reply=engine.request("POST","/api/construction/reading",body)
     check(current()){"Original construction selection changed. Refresh the reading."}
     constructionReading(reply,path,focus)
    }
   }
  }catch(error:CancellationException){throw error}
  catch(error:Exception){if(readingIntent==id)model.notice=error.message.orEmpty()}
  finally{if(readingIntent==id)acquiring=false}
 }
 Column(Modifier.widthIn(max=680.dp).heightIn(max=680.dp).padding(16.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(8.dp)){
 WithDisabledReason(if (workspace.behaviorReading.loading) "Wait for the current reading to load." else if (acquiring) "A source capture is in progress." else null) { available -> TextButton(enabled = available,onClick={scope.launch{read(connectedOnly=true)}}){Text("Read connected behavior and change impact")} }
 BehaviorReadingView(workspace)
 Text("Construction reading",style=MaterialTheme.typography.titleMedium);TextButton(onClick={workspace.showConstruction=false}){Text("Close")}
 OutlinedTextField(value=address,onValueChange={retireReading();address=it},label={Text("Construction entry")},placeholder={Text("Current workspace entry")})
 OutlinedTextField(value=model.focus,onValueChange={retireReading();model.focus=it},label={Text("Contributor or output (optional)")});WithDisabledReason(if (model.loading) "Wait for the current reading to load." else if (acquiring) "A source capture is in progress." else null) { available -> TextButton(enabled = available,onClick={scope.launch{read()}}){Text("Read captured drafts")} };Text(model.notice,fontSize=12.sp)
 model.value?.let{value->Text("Captured entry: ${value.entry} · focus: ${value.focus ?: "all"}",fontSize=12.sp);Text("Draft changes compare two reconstructions. Use Files to inspect the working copy.",fontSize=12.sp);Text("${value.count} related outputs${if(value.truncated)" · first 300 shown" else ""}. ${value.testCount} detected tests${if(value.testsTruncated)" · test list truncated" else ""}. Evidence: ${value.evidence}. Outputs have not been written.",fontSize=12.sp);Text(value.relationship,fontSize=12.sp);value.notices.forEach{Text(it,fontSize=12.sp)};if(value.unused>0)Text("${value.unused} supplied drafts did not contribute to this entry.",fontSize=12.sp)
 value.outputs.forEach{output->TextButton(onClick={model.selected=output.path}){Text("${output.change} in draft · ${output.path}")}}
 (value.outputs.firstOrNull{it.path==model.selected} ?: value.outputs.firstOrNull())?.let{output->Text("${output.change} in draft · ${output.path}",style=MaterialTheme.typography.titleSmall);Text("Contributors: ${output.contributors.joinToString()}",fontSize=12.sp);Row{TextButton(onClick={model.after=false}){Text("From saved entry")};TextButton(onClick={model.after=true}){Text("From captured drafts")}}
 val snapshot=if(model.after)output.after else output.before;if(snapshot==null)Text("Output absent from this construction.")else{Text("${snapshot.kind} · ${snapshot.size} bytes · executable ${snapshot.executable}",fontSize=12.sp);snapshot.hash?.let{hash->SelectionContainer{Text("SHA-256 $hash",fontFamily=FontFamily.Monospace,fontSize=12.sp)}};SelectionContainer{Text(snapshot.preview ?: "Binary output; text preview unavailable.",fontFamily=FontFamily.Monospace,fontSize=12.sp)};if(snapshot.abbreviated)Text("Preview abbreviated.",fontSize=12.sp)}
 Text("Compiler contribution ranges",style=MaterialTheme.typography.titleSmall);output.origins.forEach{origin->Text("${origin.phase} · ${origin.path} bytes ${origin.source.joinToString("–")} → output bytes ${origin.output.joinToString("–")} · ${if(origin.exact)"equal byte slices" else "reported origin; exact bytes unconfirmed"}",fontSize=12.sp);origin.fragment?.let{Text("Declared ${it.kind} #${it.name}",fontSize=12.sp)};origin.point?.let{point->TextButton(onClick={val navigate=model.sourceNavigation(workspace,point);scope.launch{navigate()}}){Text("Open ${origin.phase} contributor ${origin.path}")}}};if(output.truncated)Text("First 100 origins shown.",fontSize=12.sp)}
 Text("Detected test declarations",style=MaterialTheme.typography.titleSmall);value.tests.forEach{Text(it,fontSize=12.sp)}
 }
 }
}
</ui:copy>

## Validate a captured point before moving the editor

A source link captures the current foreground tab choice and reading before it
starts loading. If the user chooses another tab during the read or hash, the
source can remain cached without switching the editor. A current link still
requires exact SHA-256, an in-range UTF-16 column and a scalar boundary before
revealing the source and closing the reading.

<ui:copy id="construction-reading-android-source-point">
internal fun ConstructionReadingModel.sourceNavigation(workspace: Workspace, point: TestSourcePoint): suspend () -> Unit {
 val currentLink = sourceLink()
 val epoch = workspace.sourceNavigationEpoch
 val reading = value
 val engine = workspace.engine
 val foreground = workspace.captureForegroundNavigation()
 fun current() = currentLink() && value === reading && workspace.showConstruction &&
  workspace.sourceNavigationEpoch == epoch && workspace.engine === engine
 return navigate@{
  if (!current() || !foreground()) return@navigate
  fun find(files: List<FileNode>): FileNode? {
   for (file in files) {
    if (file.path == point.path) return file
    find(file.children)?.let { return it }
   }
   return null
  }
  try {
   val file = (workspace.buffers[point.path]?.file ?: find(workspace.rootFiles))?.takeUnless { it.dir }
    ?: error("Source no longer listed. Refresh the construction reading.")
   val buffer = workspace.loadBuffer(file)
   val source = buffer.text
   val revision = buffer.sourceRevision
   val selection = buffer.state.selection
   val hash = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Default) { debugHash(source) }
   if (!current() || !foreground()) return@navigate
   fun sourceCurrent() = current() && workspace.buffers[point.path] === buffer && buffer.text == source &&
    buffer.sourceRevision == revision && buffer.state.selection == selection
   check(sourceCurrent() && constructionPointOffset(source, point, hash) != null) {
    "Source changed or position is unavailable. Refresh the construction reading."
   }
   workspace.openSemanticLocation(SemanticLocation(languageURI(point.path), point.line - 1, point.character),
    allowDirty = true, expectedSource = source, isCurrent = ::sourceCurrent, onRevealed = { workspace.showConstruction = false })
  } catch (cancelled: CancellationException) { throw cancelled }
  catch (error: Exception) { if (current() && foreground()) notice = error.message.orEmpty() }
 }
}

fun constructionPointOffset(source:String,point:TestSourcePoint,actualHash:String):Int? {
 if(!point.hash.matches(Regex("[0-9a-f]{64}"))||point.hash!=actualHash||point.line<1||point.character<0)return null
 val rows=source.split('\n');if(point.line>rows.size||point.character>rows[point.line-1].length)return null
 val offset=rows.take(point.line-1).sumOf{it.length+1}+point.character
 if(offset>0&&offset<source.length&&source[offset-1].isHighSurrogate()&&source[offset].isLowSurrogate())return null
 return offset
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/ConstructionReading.kt"><ui:paste select="#construction-reading-android-model"/><ui:paste select="#construction-reading-android-view"/><ui:paste select="#construction-reading-android-source-point"/></ui:file>
## Retire replaced and closed readings

Release held compiler replies after a newer reading or after closing. A different entry refuses instead of appearing under the selected construction. The source-link witness decodes a captured contribution, holds its ordinary file response, and chooses another tab before returning to the same dirty editor. A newly clicked link can then reveal the exact UTF-16 position in the cached file. These service witnesses cover the native model and HTTP route.

<ui:copy id="construction-reading-android-lifetime-witness">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import kotlinx.coroutines.*
class ConstructionReadingTest {
 @Test fun reconstructedPreviewKeepsItsFullOutputHash() {
  val hash = "ab".repeat(32)
  val response = JSONObject("""{
    "version":1, "entry":"project.md", "focus":null, "materialized":false,
    "outputs":[{"path":"Example.kt","change":"unchanged",
      "before":{"hash":"$hash","kind":"File","executable":false,"size":4000,"preview":"short","preview_truncated":true},
      "after":{"kind":"File","executable":false,"size":4000,"preview":"short","preview_truncated":true},
      "contributors":[],"origins":[],"origins_truncated":false}],
    "output_count":1,"outputs_truncated":false,"tests":[],"test_count":0,"tests_truncated":false,
    "test_relationship":"declarations","evidence_status":"not_queried","notices":[],"unused_sources":[]
  }""")
  val output = constructionReading(response, "project.md", null).outputs.single()
  assertEquals(hash, output.before?.hash)
  assertTrue(output.before?.abbreviated == true)
  assertNull(output.after?.hash)
 }

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
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/ConstructionReadingTest.kt"><ui:paste select="#construction-reading-android-lifetime-witness"/></ui:file>
</ui:doc>
