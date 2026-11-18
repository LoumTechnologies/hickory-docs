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

data class ConstructedSnapshot(val kind:String,val executable:Boolean,val size:Long,val preview:String?,val abbreviated:Boolean)
data class ConstructionFragment(val name:String,val kind:String,val body:List<Int>)
data class ConstructionOrigin(val phase:String,val path:String,val source:List<Int>,val output:List<Int>,val exact:Boolean,val point:TestSourcePoint?=null,val fragment:ConstructionFragment?=null)
data class ConstructionOutput(val path:String,val change:String,val before:ConstructedSnapshot?,val after:ConstructedSnapshot?,val contributors:List<String>,val origins:List<ConstructionOrigin>,val truncated:Boolean)
data class ConstructionReading(val entry:String,val focus:String?,val outputs:List<ConstructionOutput>,val count:Int,val truncated:Boolean,val tests:List<String>,val testCount:Int,val testsTruncated:Boolean,val relationship:String,val evidence:String,val notices:List<String>,val unused:Int)
private fun org.json.JSONArray.strings()=(0 until length()).map{getString(it)}
fun constructionReading(raw:JSONObject,entry:String,focus:String?):ConstructionReading {
 check(raw.getInt("version")==1&&raw.getString("entry")==entry&&(if(raw.isNull("focus"))null else raw.getString("focus"))==focus&&!raw.getBoolean("materialized")){"Construction identity changed or unsupported response. Refresh the reading."}
 fun snapshot(raw:JSONObject?):ConstructedSnapshot?=raw?.let{ConstructedSnapshot(it.getString("kind"),it.getBoolean("executable"),it.getLong("size"),if(it.isNull("preview"))null else it.getString("preview"),it.getBoolean("preview_truncated"))}
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

Use the existing floating surface and menus. A blank field reads the bound
workspace entry. A typed address retains its literal bytes. Capture the draft
roster and original intent before requesting the workspace entry. Changing
the address, contributor, or captured source retires the pending reading.

Label the entry, focus, preview side, binary output and every truncation.
Only service-supplied source points with matching acquired source hashes and
valid scalar positions enable navigation. Closing restores source focus and
retires the request.

<ui:copy id="construction-reading-android-view">
@Composable fun ConstructionReadingView(workspace:Workspace){
 val model=remember{ConstructionReadingModel()}
 val scope=rememberCoroutineScope()
 var address by remember{mutableStateOf("")}
 var readingIntent by remember{mutableLongStateOf(0L)}
 var acquiring by remember{mutableStateOf(false)}
 fun retireReading(){readingIntent++;acquiring=false;model.close();workspace.behaviorReading.close();model.notice=""}
 DisposableEffect(model){onDispose{readingIntent++;model.close();workspace.behaviorReading.close()}}
 suspend fun openPoint(point:TestSourcePoint){
  val current=model.sourceLink();val epoch=workspace.sourceNavigationEpoch
  fun valid()=current()&&workspace.showConstruction&&workspace.sourceNavigationEpoch==epoch
  fun find(files:List<FileNode>):FileNode?{for(file in files){if(file.path==point.path)return file;find(file.children)?.let{return it}};return null}
  try{
   val file=(workspace.buffers[point.path]?.file ?: find(workspace.rootFiles))?.takeUnless{it.dir} ?: error("Source no longer listed. Refresh the construction reading.")
   val buffer=workspace.loadBuffer(file);val source=buffer.text
   val hash=kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Default){java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}}
   if(!valid())return
   check(buffer.text==source&&constructionPointOffset(source,point,hash)!=null){"Source changed or position is unavailable. Refresh the construction reading."}
   workspace.openSemanticLocation(SemanticLocation(languageURI(point.path),point.line-1,point.character),allowDirty=true,expectedSource=source,isCurrent=::valid,onRevealed={if(valid())workspace.showConstruction=false})
  }catch(error:CancellationException){throw error}catch(error:Exception){if(valid())model.notice=error.message.orEmpty()}
 }
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
 TextButton(enabled=!workspace.behaviorReading.loading&&!acquiring,onClick={scope.launch{read(connectedOnly=true)}}){Text("Read connected behavior and change impact")}
 BehaviorReadingView(workspace)
 Text("Construction reading",style=MaterialTheme.typography.titleMedium);TextButton(onClick={workspace.showConstruction=false;workspace.currentBuffer?.focusSource?.invoke()}){Text("Close")}
 OutlinedTextField(value=address,onValueChange={retireReading();address=it},label={Text("Construction entry")},placeholder={Text("Current workspace entry")})
 OutlinedTextField(value=model.focus,onValueChange={retireReading();model.focus=it},label={Text("Contributor or output (optional)")});TextButton(enabled=!model.loading&&!acquiring,onClick={scope.launch{read()}}){Text("Read captured drafts")};Text(model.notice,fontSize=12.sp)
 model.value?.let{value->Text("Captured entry: ${value.entry} · focus: ${value.focus ?: "all"}",fontSize=12.sp);Text("${value.count} related outputs${if(value.truncated)" · first 300 shown" else ""}. ${value.testCount} detected tests${if(value.testsTruncated)" · test list truncated" else ""}. Evidence: ${value.evidence}. Outputs have not been written.",fontSize=12.sp);Text(value.relationship,fontSize=12.sp);value.notices.forEach{Text(it,fontSize=12.sp)};if(value.unused>0)Text("${value.unused} supplied drafts did not contribute to this entry.",fontSize=12.sp)
 value.outputs.forEach{output->TextButton(onClick={model.selected=output.path}){Text("${output.change} · ${output.path}")}}
 (value.outputs.firstOrNull{it.path==model.selected} ?: value.outputs.firstOrNull())?.let{output->Text("${output.change} · ${output.path}",style=MaterialTheme.typography.titleSmall);Text("Contributors: ${output.contributors.joinToString()}",fontSize=12.sp);Row{TextButton(onClick={model.after=false}){Text("Saved construction")};TextButton(onClick={model.after=true}){Text("Draft construction")}}
 val snapshot=if(model.after)output.after else output.before;if(snapshot==null)Text("Output absent from this construction.")else{Text("${snapshot.kind} · ${snapshot.size} bytes · executable ${snapshot.executable}",fontSize=12.sp);SelectionContainer{Text(snapshot.preview ?: "Binary output; text preview unavailable.",fontFamily=FontFamily.Monospace,fontSize=12.sp)};if(snapshot.abbreviated)Text("Preview abbreviated.",fontSize=12.sp)}
 Text("Compiler contribution ranges",style=MaterialTheme.typography.titleSmall);output.origins.forEach{origin->Text("${origin.phase} · ${origin.path} bytes ${origin.source.joinToString("–")} → output bytes ${origin.output.joinToString("–")} · ${if(origin.exact)"equal byte slices" else "reported origin; exact bytes unconfirmed"}",fontSize=12.sp);origin.fragment?.let{Text("Declared ${it.kind} #${it.name}",fontSize=12.sp)};origin.point?.let{point->TextButton(onClick={scope.launch{openPoint(point)}}){Text("Open ${origin.phase} contributor ${origin.path}")}}};if(output.truncated)Text("First 100 origins shown.",fontSize=12.sp)}
 Text("Detected test declarations",style=MaterialTheme.typography.titleSmall);value.tests.forEach{Text(it,fontSize=12.sp)}
 }
 }
}
</ui:copy>

## Validate a captured point before moving the editor

A source link needs a matching SHA-256 value computed over the acquired text. Require a positive line, an in-range UTF-16 column and a scalar boundary; never clamp a stale point into the document. File identity, workspace lifetime and captured-reading lifetime are checked separately by the navigation adapter.

<ui:copy id="construction-reading-android-source-point">
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

Release held compiler replies after a newer reading or after closing. Neither may replace the retained result. A different entry refuses instead of appearing under the selected construction. Source-point cases also require exact hashes and Unicode scalar boundaries, and a newer link retires an older link. These service witnesses do not establish native physical input or assistive technology acceptance.

<ui:copy id="construction-reading-android-lifetime-witness">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
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

}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/ConstructionReadingTest.kt"><ui:paste select="#construction-reading-android-lifetime-witness"/></ui:file>
</ui:doc>
