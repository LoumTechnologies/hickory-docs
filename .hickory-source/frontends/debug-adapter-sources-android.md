<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read adapter-owned source without borrowing a workspace file

For Android maintainers extending the existing debugger inspector.

A positive source reference belongs to an exact reported stopped frame. It never identifies a writable workspace file by spelling. Require an explicitly true engine source_content capability, current session/stop/thread/frame, positive signed32 reference, inspector lifetime and latest request ID. Missing older-engine capability sends no operation. Thread/frame changes, movement and inspector close retire pending observations independently from watches, stepping and Stop; captured text remains readable as historical data.

The response is literal read-only text. Validate immutable envelope fields immediately, then count UTF8 bytes, scalar/page boundaries and SHA256 on the worker dispatcher. Recheck the request, inspector and stopped scope after that await before admitting text. Bound the response at 8 MiB and MIME metadata at 256 UTF8 bytes. Local pages contain at most 4000 Unicode scalars and do not split surrogate pairs. These facts supply no workspace editor, source position, breakpoint or edit authority. A provider refusal preserves the paused session and stack, and never triggers disk fallback or automatic retry.

<ui:copy id="debug-adapter-sources-android">
package com.hickorydocs.simple
import androidx.compose.runtime.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.*
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.*
import org.json.JSONObject
import java.util.UUID

data class NativeAdapterSourceBinding(val scope:NativeDebugScope,val reference:Long){init{require(reference in 1..Int.MAX_VALUE.toLong())}}
data class NativeAdapterSourceFacts(val hash:String,val scalars:Int,val starts:List<Int>)
internal fun nativeAdapterSourceFacts(content:String,budget:Int=8*1024*1024):NativeAdapterSourceFacts{
 require(budget>=0&&content.length<=budget){"Adapter source exceeds its UTF8 reading budget."};var offset=0;var scalars=0;var bytes=0L;val starts=mutableListOf(0)
 while(offset<content.length){val char=content[offset];val cost=when{char.isHighSurrogate()->{require(offset+1<content.length&&content[offset+1].isLowSurrogate()){"Adapter source contains an unpaired surrogate."};offset+=2;4};char.isLowSurrogate()->error("Adapter source contains an unpaired surrogate.");else->{offset++;when{char.code<=0x7f->1;char.code<=0x7ff->2;else->3}}};bytes+=cost;require(bytes<=budget){"Adapter source exceeds its UTF8 reading budget."};scalars++;if(scalars%4000==0&&offset<content.length)starts.add(offset)}
 return NativeAdapterSourceFacts(debugHash(content),scalars,starts.toList())
}
data class NativeAdapterSourceReading(val binding:NativeAdapterSourceBinding,val request:String,val content:String,val hash:String,val mime:String?,val facts:NativeAdapterSourceFacts){
 fun page(index:Int):String{require(index in facts.starts.indices);return content.substring(facts.starts[index],facts.starts.getOrNull(index+1) ?: content.length)}
}
class NativeAdapterSourceModel(private val validate:suspend(String,String?)->NativeAdapterSourceFacts={text,mime->withContext(Dispatchers.Default){if(mime!=null)nativeAdapterSourceFacts(mime,256);nativeAdapterSourceFacts(text)}}){
 private data class Pending(val binding:NativeAdapterSourceBinding,val request:String,val inspection:String,val generation:Long,var processing:Boolean=false)
 private var inspection:String?=null;private var generation=0L;private var pending:Pending?=null
 var reading by mutableStateOf<NativeAdapterSourceReading?>(null);private set
 var busy by mutableStateOf(false);private set
 var notice by mutableStateOf("");private set
 var current by mutableStateOf(false);private set
 fun open():String{retire("Previous adapter text remains a historical reading until explicitly refreshed.");return UUID.randomUUID().toString().also{inspection=it}}
 fun close(token:String){if(inspection==token){inspection=null;retire("Adapter-source inspector closed. Captured text remains historical.")}}
 fun retire(reason:String="Paused source observation changed; captured adapter text is historical."){generation++;pending=null;busy=false;current=false;if(reading!=null||notice.isNotEmpty())notice=reason}
 fun cancel(token:String){if(inspection==token)retire("Pending source reading cancelled locally. Late replies cannot replace captured text.")}
 fun begin(binding:NativeAdapterSourceBinding?,capability:Any?,token:String):JSONObject?{
  if(token!=inspection){notice="Open the current debugger inspector before reading adapter source.";return null}
  if(capability!=true){notice="This engine/provider does not advertise adapter source content. No workspace text was substituted.";return null}
  if(binding==null){notice="Choose a current paused frame with a positive adapter source reference. No workspace fallback is used.";return null}
  val request=UUID.randomUUID().toString();pending=Pending(binding,request,token,++generation);busy=true;current=false;notice="Reading the selected frame's adapter-owned text…"
  return binding.scope.put(JSONObject().put("op","source").put("reference",binding.reference).put("request_id",request))
 }
 private fun owns(value:Pending)=pending===value&&inspection==value.inspection&&generation==value.generation
 suspend fun receive(event:JSONObject,binding:()->NativeAdapterSourceBinding?):Boolean{
  val value=pending ?: return false;if(event.optString("request_id")!=value.request||!owns(value)||value.processing)return false
  if(event.optString("event")!="adapter_source"||!value.binding.scope.matches(event)||debugInteger(event.opt("reference"))!=value.binding.reference||binding()!=value.binding)return refuse(value.request,"Adapter source reply belongs to an obsolete or foreign frame. Previous text retained.")
  val content=event.opt("content") as? String ?: return refuse(value.request,"Adapter returned no literal source content.")
  val hash=event.opt("source_hash") as? String ?: return refuse(value.request,"Adapter source hash is unavailable.")
  if(content.length>8*1024*1024||!hash.matches(Regex("[a-f0-9]{64}")))return refuse(value.request,"Adapter source exceeds its reading budget or has an invalid source hash.")
  val mime=when(val raw=event.opt("mime_type")){null,JSONObject.NULL->null;is String->raw.takeIf{it.length<=256&&!it.any{char->char.isISOControl()}} ?: return refuse(value.request,"Adapter returned invalid source MIME metadata.");else->return refuse(value.request,"Adapter returned invalid source MIME metadata.")}
  value.processing=true
  val facts=try{validate(content,mime)}catch(cancelled:CancellationException){refuse(value.request,"Adapter source reading cancelled. Refresh explicitly if still paused.");throw cancelled}catch(error:Exception){return refuse(value.request,error.message ?: "Adapter source validation failed.")}
  if(!owns(value))return false
  if(binding()!=value.binding){retire("Source observation changed before confirmation. Captured text retained.");return false}
  if(facts.hash!=hash)return refuse(value.request,"Adapter source bytes do not match their source hash. Previous text retained.")
  reading=NativeAdapterSourceReading(value.binding,value.request,content,hash,mime,facts);pending=null;busy=false;current=true;notice=if(content.isEmpty())"Adapter reported empty literal source content." else "Adapter-owned source only. It grants no workspace position, edit or breakpoint authority.";return true
 }
 fun refuse(request:String,message:String):Boolean{val value=pending?.takeIf{it.request==request&&owns(it)} ?: return false;pending=null;busy=false;current=false;notice=message+" Captured text is retained; refresh explicitly.";return true}
 fun matches(binding:NativeAdapterSourceBinding?)=current&&reading?.binding==binding
}
@Composable fun NativeAdapterSourceView(debug:NativeDebugger,inspection:String?){
 val model=debug.adapterSources;val binding=debug.adapterSourceBinding();val capable=debug.capabilities.opt("source_content")==true
 if(binding!=null||model.reading!=null||model.notice.isNotEmpty())Column{
  Text("Adapter-owned source",fontSize=12.sp)
  if(binding!=null&&!capable)Text("This engine/provider does not advertise source content. Workspace or disk text is not a fallback.",fontSize=12.sp)
  TextButton(enabled=inspection!=null&&binding!=null&&capable&&!model.busy,onClick={inspection?.let{debug.readAdapterSource(it)}}){Text("Read selected frame adapter source")}
  if(model.busy)TextButton(onClick={inspection?.let{model.cancel(it)}}){Text("Cancel pending source reading")}
  if(model.notice.isNotEmpty())Text(model.notice,fontSize=12.sp)
  model.reading?.let{value->var page by remember(value){mutableIntStateOf(0)};val index=page.coerceIn(value.facts.starts.indices)
   Text("${if(model.matches(binding))"Current" else "Historical"} adapter text · pause ${value.binding.scope.stop} · thread ${value.binding.scope.thread} · frame ${value.binding.scope.frame} · reference ${value.binding.reference}",fontSize=11.sp)
   Text("${value.mime ?: "MIME unavailable"} · SHA256 ${value.hash}",fontSize=11.sp)
   SelectionContainer{Text(value.page(index),fontFamily=FontFamily.Monospace,fontSize=12.sp)}
   Text("Page ${index+1}/${value.facts.starts.size} · ${value.facts.scalars} Unicode scalars. Read-only literal text; no workspace mapping is implied.",fontSize=11.sp)
   Row{TextButton(enabled=index>0,onClick={page=index-1}){Text("Previous adapter source page")};TextButton(enabled=index<value.facts.starts.lastIndex,onClick={page=index+1}){Text("Next adapter source page")}}
  }
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/DebugAdapterSources.kt"><ui:paste select="#debug-adapter-sources-android"/></ui:file>

## Keep provider refusal local to the read capability

Focused policy/controller witnesses cover scalar boundaries, UTF8 quota, exact source hashes and MIME, absent/wrongly typed capability, inspector lifetime, reused references across newer pauses, and a delayed validation completion. The real debugger adapter keeps evaluation, movement and Stop available while source reads are pending, rejects optional provider failures without changing paused frames, and suppresses workspace navigation for positive references even when the filename and apparent line match. The six focused model/controller methods passed canonical Kotlin compilation with zero skips. A separately opt-in actual witness uses the maintained owned stdio adapter fixture and exact inspector actions, retains literal content and the same editor state, and exercises held source IO alongside movement, refusal, Watch and Stop. That new actual witness remains unexecuted until canonical generation and an owned fixture run. No rendered Compose or physical input/accessibility acceptance is claimed.

<ui:copy id="debug-adapter-sources-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import org.json.JSONArray
import kotlinx.coroutines.*
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class DebugAdapterSourcesTest {
 private fun binding(stop:String="1",frame:Long=0,reference:Long=123)=NativeAdapterSourceBinding(NativeDebugScope("ours",stop,-1,frame),reference)
 private fun response(request:JSONObject,text:String="adapter <literal> & source\n")=JSONObject(request.toString()).put("event","adapter_source").put("content",text).put("source_hash",debugHash(text)).put("mime_type","text/x-python")
 @Test fun unicodePagesAndUtf8BudgetsRemainLiteralAndBounded(){val text="😀"+"x".repeat(4000);val facts=nativeAdapterSourceFacts(text);assertEquals(4001,facts.scalars);assertEquals(listOf(0,4001),facts.starts);val reading=NativeAdapterSourceReading(binding(),"request",text,facts.hash,null,facts);assertEquals(4000,reading.page(0).codePointCount(0,reading.page(0).length));assertEquals("x",reading.page(1));assertTrue(reading.page(0).startsWith("😀"));assertEquals(debugHash(text),facts.hash)
  assertEquals(1,nativeAdapterSourceFacts("😀",4).scalars);assertTrue(runCatching{nativeAdapterSourceFacts("😀",3)}.isFailure);assertTrue(runCatching{nativeAdapterSourceFacts("\uD800")}.isFailure);assertEquals(listOf(0),nativeAdapterSourceFacts("").starts)
 }
 @Test fun missingOrWrongTypedCapabilityAndForeignInspectionNeverSendSource(){val model=NativeAdapterSourceModel();val token=model.open();for(capability in listOf(null,false,"true",1,JSONObject.NULL))assertNull(model.begin(binding(),capability,token));assertNull(model.begin(binding(),true,"foreign"));assertNull(model.begin(null,true,token));val request=model.begin(binding(),true,token)!!;assertEquals("source",request.getString("op"));assertEquals(0,request.getInt("frame"));assertEquals(-1,request.getInt("thread"));assertEquals(123,request.getInt("reference"));assertFalse(request.has("path"))}
 @Test fun exactScopeHashAndMimeAreRequiredWithoutDiscardingReadableText()=runBlocking{
  val model=NativeAdapterSourceModel();val token=model.open();var scope=binding();val first=model.begin(scope,true,token)!!;assertTrue(model.receive(response(first)){scope});val admitted=model.reading!!
  for((key,bad) in listOf("session" to "foreign","stop" to 2,"thread" to 0,"frame" to 1,"reference" to 456,"source_hash" to "b".repeat(64),"mime_type" to "bad\nmetadata")){
   val request=model.begin(scope,true,token)!!;model.receive(response(request).put(key,bad)){scope};assertSame(key,admitted,model.reading);assertFalse(model.current)
  }
  val request=model.begin(scope,true,token)!!;assertTrue(model.refuse(request.getString("request_id"),"Provider refused source"));assertSame(admitted,model.reading);assertTrue(model.notice.contains("Provider refused"));assertFalse(model.busy)
 }
 @Test fun closedAndReusedReferenceRepliesCannotRebindANewerPause()=runBlocking{
  val entered=CompletableDeferred<Unit>();val released=CompletableDeferred<Unit>();val model=NativeAdapterSourceModel{text,_->entered.complete(Unit);released.await();nativeAdapterSourceFacts(text)};val token=model.open();var scope=binding();val old=model.begin(scope,true,token)!!;val pending=async(start=CoroutineStart.UNDISPATCHED){model.receive(response(old,"old bytes")){scope}};entered.await();model.retire();scope=binding("2",-3);val newer=model.begin(scope,true,token)!!;released.complete(Unit);assertFalse(pending.await());assertNull(model.reading);assertTrue(model.receive(response(newer,"new bytes")){scope});val fresh=model.reading!!;assertEquals("new bytes",fresh.content)
  val closing=model.begin(scope,true,token)!!;model.close(token);val reopened=model.open();assertFalse(model.receive(response(closing)){scope});assertSame(fresh,model.reading);assertFalse(model.current);assertNotNull(model.begin(scope,true,reopened))
 }
 private suspend fun host(capability:Any?=true,block:suspend(NativeDebugger,MutableList<JSONObject>)->Unit)=coroutineScope{
  val listener=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0);listener.createContext("/api/launch-profiles"){exchange->val bytes=JSONObject("""{"debug_protocol":3,"profiles":[{"id":"python","name":"Python","can_debug":true,"program":"main.py"}],"fingerprint":"current"}""").toString().toByteArray();exchange.sendResponseHeaders(200,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}};listener.start()
  val debug=NativeDebugger(Engine("http://127.0.0.1:${listener.address.port}"),this);val requests=mutableListOf<JSONObject>();debug.emitForTest={requests.add(it)}
  try{debug.prepare(DocumentBuffer(FileNode("main.py","main.py",false,emptyList(),null,null),FileText("primary = 1\n","saved")));val start=requests.last();debug.receive(JSONObject().put("event","started").put("doc","hick:///main.py").put("session","ours").put("source_hash",start.getString("source_hash")).put("request_id",start.getString("request_id")).put("breakpoints",JSONArray()).put("capabilities",JSONObject().put("source_content",capability ?: JSONObject.NULL).put("frame_locals",true)));block(debug,requests)}finally{debug.close();listener.stop(0)}
 }
 private fun stopped(stop:Int=1)=JSONObject().put("event","stopped").put("session","ours").put("stop",stop).put("thread",-1).put("line",0).put("frames",JSONArray().put(JSONObject().put("id",0).put("name","virtual frame").put("source","main.py").put("source_line",0).put("source_reference",123).put("line",0).put("in_document",true))).put("variables",JSONArray())
 @Test fun optionalSourceRefusalPreservesFramesAndDoesNotBlockWatchStepOrStop()=runBlocking{host{debug,requests->val locations=mutableListOf<String>();debug.onLocation={path,_->locations.add(path)};debug.receive(stopped());assertTrue(locations.isEmpty());assertNull(debug.line("main.py"));val frames=debug.frames;val token=debug.adapterSources.open();debug.readAdapterSource(token);val source=requests.last();assertEquals("source",source.getString("op"));debug.evaluate("value");assertEquals("eval",requests.last().getString("op"));debug.receive(JSONObject().put("event","failed").put("session","ours").put("request_id",source.getString("request_id")).put("message","Provider has no source"));assertEquals("paused",debug.status);assertSame(frames,debug.frames);assertTrue(debug.adapterSources.notice.contains("Provider has no source"))
  debug.readAdapterSource(token);val pending=requests.last();debug.step("continue");assertEquals("step",requests.last().getString("op"));debug.receive(response(pending));yield();assertNull(debug.adapterSources.reading);assertEquals("running",debug.status);debug.stop();assertEquals("stop",requests.last().getString("op"))
 }}
 @Test fun olderEngineNeverReceivesUnknownSourceOperation()=runBlocking{host("true"){debug,requests->debug.receive(stopped());val token=debug.adapterSources.open();debug.readAdapterSource(token);assertFalse(requests.any{it.optString("op")=="source"});assertEquals("paused",debug.status);assertTrue(debug.adapterSources.notice.contains("does not advertise"))}}
 @Test(timeout=120000) fun liveOwnedAdapterSourceInspectorRetainsEditorAndIndependentControls()=runBlocking{
  val origin=System.getenv("HICKORY_NATIVE_ADAPTER_SOURCE_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_NATIVE_ADAPTER_SOURCE_WORKSPACE") ?: error("Use the maintained owned adapter-source fixture"));check(root.resolve(".hickory-owned-adapter-source-fixture").readText()=="adapter-source-fixture-v1\n"){"Refuse non-owned adapter source fixture"}
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(background);val engine=Engine(origin!!);val debug=NativeDebugger(engine,background);w.engine=engine;w.debugger=debug
  try{
   val (_,tree)=engine.files();assertEquals(root.toPath().toRealPath().toString(),engine.sourceRootAddress);w.rootFiles=tree
   fun find(nodes:List<FileNode>):FileNode?{for(node in nodes){if(node.path=="main.py")return node;find(node.children)?.let{return it}};return null}
   val b=w.loadBuffer(find(tree) ?: error("Fixture source missing"));w.show(b.file.path);val original=b.text;val state=b.state;debug.prepare(b)
   suspend fun await(label:String,condition:()->Boolean){withTimeout(45000){while(!condition()){check(debug.status!="failed"){"$label: ${debug.message}"};delay(20)}}}
   await("Owned provider initial stop"){debug.status=="paused"&&debug.adapterSourceBinding()!=null};assertTrue(debug.capabilities.opt("source_content")==true);val binding=debug.adapterSourceBinding()!!;assertEquals(123L,binding.reference);assertNull(debug.line("main.py"))
   b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"# retained unsaved source\n")),b.text);b.selectSourceRange(2,2,b.text);b.sourceScroll.scrollTo(7);val draft=b.text;val selection=state.selection;val scroll=b.sourceScroll.value;assertTrue(state.undoState.canUndo)
   val inspection=debug.adapterSources.open();debug.readAdapterSource(inspection);await("Literal adapter text"){debug.adapterSources.reading!=null&&!debug.adapterSources.busy};val first=debug.adapterSources.reading!!;assertEquals(binding,first.binding);assertEquals("<b>adapter-owned literal source</b>\nmarker = 41\n",first.page(0));assertEquals("text/html",first.mime);assertEquals(debugHash(first.content),first.hash);assertFalse(first.content.contains("workspace bytes"));assertTrue(debug.adapterSources.matches(binding))
   root.resolve("hold-source").writeText("hold optional source");debug.readAdapterSource(inspection);await("Held provider source IO"){root.resolve("source-held").exists()};assertTrue(debug.adapterSources.busy);debug.step("over");await("Movement while source held"){root.resolve("movement-received").exists()&&debug.status=="paused"&&debug.adapterSourceBinding()!=binding};assertSame(first,debug.adapterSources.reading);assertFalse(debug.adapterSources.current)
   root.resolve("release-source").writeText("release obsolete source");root.resolve("hold-source").delete();delay(100);assertSame(first,debug.adapterSources.reading);assertFalse(debug.adapterSources.current)
   root.resolve("source-error").writeText("refuse optional capability");debug.readAdapterSource(inspection);await("Local provider source refusal"){!debug.adapterSources.busy&&debug.adapterSources.notice.contains("unavailable",true)};assertEquals("paused",debug.status);assertTrue(debug.frames.isNotEmpty());assertSame(first,debug.adapterSources.reading);debug.evaluate("marker");await("Watch independent of refused source"){debug.values["marker"]=="41"}
   assertSame(b,w.currentBuffer);assertSame(state,b.state);assertEquals(draft,b.text);assertEquals(selection,state.selection);assertEquals(scroll,b.sourceScroll.value);assertEquals(original,root.resolve("main.py").readText());assertNull(debug.line("main.py"));debug.adapterSources.close(inspection);debug.stop();await("Owned session stop"){!debug.active};assertTrue(b.snippets.undoSource());assertEquals(original,b.text);assertTrue(b.snippets.redoSource());assertEquals(draft,b.text)
  }finally{root.resolve("release-source").writeText("cleanup");if(debug.active)debug.stop();debug.close();w.buffers.values.forEach{it.language?.close()};background.cancel()}
 }

}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/DebugAdapterSourcesTest.kt"><ui:paste select="#debug-adapter-sources-android-tests"/></ui:file>
</ui:doc>
