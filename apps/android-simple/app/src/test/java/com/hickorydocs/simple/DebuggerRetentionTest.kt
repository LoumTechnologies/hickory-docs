package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import org.json.JSONArray
import kotlinx.coroutines.*
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class DebuggerRetentionTest {
 @Test fun completeBreakpointReceiptsAndArchiveReadings(){
  val model=NativeBreakpointModel();val source=NativeSourceSnapshot("owned","main.py","first = 1\nsecond = 2\n",true,true);val hash="a".repeat(64);model.toggle(source,0,"first == 1");val intent=model.toggle(source,1,"second == 2");model.stamp(source.identity,intent.revision,hash);model.request(source,"session","current")
  fun row(line:Any,state:Any="bound")=JSONObject().put("line",line).put("state",state)
  val good=listOf(row(0),row(1,"pending"));val bad=listOf(emptyList(),listOf(good[0]),listOf(good[0],good[0]),listOf(good[0],row(2)),listOf(good[0],row(1,"invented")),listOf(good[0],row(1).put("moved_to",999)),listOf(good[0],row(1).put("message",JSONObject().put("foreign",true))),listOf(good[0],row(true)))
  for(rows in bad){assertFalse(model.admit(source,JSONObject().put("breakpoints",JSONArray(rows)),"session","current",hash));assertTrue(model.receipts.isEmpty());assertEquals(2,model.intents.getValue(source.identity).marks.size)}
  assertTrue(model.admit(source,JSONObject().put("breakpoints",JSONArray(good)),"session","current",hash));assertEquals("first == 1",model.rows(source,"session",hash).first().getString("condition"))
  val changed=source.copy(text="# changed\n"+source.text,saved=false);assertTrue(model.rows(changed,"session",hash).isEmpty());model.toggle(changed,1,"first >= 1");val reading=model.readingPage(changed,0);assertTrue(reading.rows.any{it.historical&&it.condition=="first == 1"});assertTrue(reading.rows.any{!it.historical&&it.condition=="first >= 1"});assertTrue(model.rows(changed,"session",hash).isEmpty());model.discard(changed.identity);assertTrue(model.readingPage(changed,0).rows.any{it.condition=="first >= 1"})
 }
 @Test fun retainedIntentionPagesDoNotExposeMutableModelRows(){val model=NativeBreakpointModel();val source=NativeSourceSnapshot("owned","main.py",(0..200).joinToString("\n"){"line $it"},true,true);for(line in 0..129)model.toggle(source,line,"test $line");val first=model.readingPage(source,0);val second=model.readingPage(source,1);val third=model.readingPage(source,999);assertEquals(130,first.total);assertEquals(64,first.rows.size);assertEquals(64,second.rows.size);assertEquals(2,third.page);assertEquals(listOf(128,129),third.rows.map{it.line});assertEquals(0,model.readingPage(source,-1).page)}
 @Test fun liveOwnedDeepStackAndEditedBreakpointRetention()=runBlocking {
  val origin=System.getenv("HICKORY_DEBUG_RETENTION_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_DEBUG_RETENTION_WORKSPACE") ?: error("Owned workspace required"));check(java.io.File(root,".hickory-debug-retention-fixture").readText()=="owned-debug-retention-v1\n");val ownedScope=CoroutineScope(SupervisorJob()+Dispatchers.Default);val engine=Engine(origin!!);val w=Workspace(ownedScope);w.engine=engine
  try{val (_,tree)=engine.files();w.rootFiles=tree;val file=tree.first{it.path=="deep.py"};val b=w.loadBuffer(file);w.show(file.path);val source=b.text;val state=b.state;val scroll=b.sourceScroll;val debug=NativeDebugger(engine,ownedScope);w.debugger=debug
   try{
    suspend fun wait(reason:String,predicate:()->Boolean){withTimeout(30000){while(!predicate())delay(20)}}
    val line=source.lines().indexOfFirst{it.contains("# deep-stop")};check(line>=0);debug.toggle(b,line,"depth == 0");debug.prepare(b);wait("Deep program pause"){debug.status=="paused"||debug.status=="failed"};assertEquals(debug.message,"paused",debug.status);assertTrue(debug.frames.size>=131)
    val frame=debug.frames[100];debug.select(frame);debug.readFrameVariables();wait("Deep caller locals"){debug.variables.any{it.optString("name")=="depth"&&it.optString("value")=="100"}};debug.evaluate("depth");wait("Deep caller evaluation"){debug.values["depth"]!=null};assertEquals("100",debug.values["depth"]);assertSame(state,b.state);assertSame(scroll,b.sourceScroll)
    val requests=java.util.concurrent.CopyOnWriteArrayList<JSONObject>();debug.observeRequestForTest={requests.add(it)};assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"# retained unsaved breakpoint edit\n")),b.text));val edited=b.text;val selection=b.state.selection;debug.toggle(b,line,"depth <= 0");delay(100);assertFalse(requests.any{it.optString("op") in listOf("breakpoints","source_breakpoints")});assertTrue(debug.breakpointRows(b).isEmpty());val reading=debug.breakpointReadingPage(b,0);assertTrue(reading.rows.any{it.condition=="depth == 0"});assertTrue(reading.rows.any{it.condition=="depth <= 0"});assertEquals(selection,b.state.selection);assertEquals(source,java.io.File(root,file.path).readText())
    debug.stop();wait("Stopped deep session"){!debug.active};b.save(engine);assertFalse(b.error,b.dirty);assertEquals(edited,java.io.File(root,file.path).readText());debug.prepare(b);wait("Edited deep program pause"){debug.status=="paused"||debug.status=="failed"};assertEquals(debug.message,"paused",debug.status);assertTrue(debug.frames.size>=131);assertTrue(debug.breakpointRows(b).any{it.optString("state")=="bound"&&it.optString("condition")=="depth <= 0"});debug.stop();wait("Stopped edited session"){!debug.active};assertSame(state,b.state);assertSame(scroll,b.sourceScroll);assertTrue(b.snippets.undoSource());assertEquals(source,b.text);assertEquals(edited,java.io.File(root,file.path).readText())
   }finally{debug.close()}
  }finally{w.buffers.values.forEach{it.language?.close()};ownedScope.cancel()}
 }
}
