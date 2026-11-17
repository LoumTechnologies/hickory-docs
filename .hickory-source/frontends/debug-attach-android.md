<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Attach without borrowing the displayed source revision

Android maintainers use the existing debugger profile picker and floating inspector. An endpoint profile names a declared loopback port; only the shared engine owns its arguments and attachment effect. Decode the port and metadata before offering the profile. Older ordinary profiles remain ordinary launches. The explicit chooser retains endpoint access when a viewed application would otherwise be automatic.

Attachment sends the profile selection, catalogue fingerprint and captured host-source hash required by Start admission, never a displayed program override. That host-source receipt does not prove the external runtime loaded the displayed bytes. It retains session/thread/frame/watch/Stop controls while refusing source positions, breakpoint receipts and later disk-derived source binding. A matching path or body does not establish the external process's loaded revision. The inspector keeps this caveat visible without changing editor geometry.

<ui:copy id="debug-attach-android-policy">
package com.hickorydocs.simple
import org.json.JSONObject
internal data class NativeAttachProfile(val port:Int) {
 val notice get()="Attach on loopback port $port · external source revision unverified"
}
internal fun nativeAttachProfile(profile:JSONObject):NativeAttachProfile? {
 if(!profile.has("attach"))return null
 val value=profile.get("attach") as? JSONObject ?: error("Malformed Attach profile; no debugger request was sent.")
 val raw=value.get("port")
 val number=(raw as? Number)?.toString()?.toIntOrNull() ?: error("Attach profile has no canonical loopback port.")
 check(number in 1..65535){"Attach profile loopback port is out of range."}
 check(value.get("arguments") is JSONObject){"Attach profile arguments are unavailable."}
 return NativeAttachProfile(number)
}
internal fun nativeDebugProfileLabel(profile:JSONObject):String =
 (if(nativeAttachProfile(profile)!=null)"Attach " else "Debug ")+profile.getString("name")
</ui:copy>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/NativeDebugAttach.kt"><ui:paste select="#debug-attach-android-policy"/></ui:file>

The focused boundary tests reject malformed endpoint metadata, keep old profiles available, and inspect the real controller's Start envelope and receipt routing. The optional maintained debugpy fixture exercises the native controller and transport; rendered Compose, physical input, IME, geometry and accessibility acceptance remain unverified.

<ui:copy id="debug-attach-android-tests">
package com.hickorydocs.simple
import org.json.JSONObject
import org.json.JSONArray
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class NativeDebugAttachTest {
 private fun profile(attach:Any?=null)=JSONObject().put("id","external").put("name","Owned external Python").put("program",JSONObject.NULL).put("can_debug",true).put("default",true).apply{if(attach!=null)put("attach",attach)}
 private fun endpoint(port:Any=5678)=JSONObject().put("port",port).put("arguments",JSONObject())
 @Test fun endpointMetadataAndLabelsAreExplicit(){assertNull(nativeAttachProfile(profile()));assertEquals("Debug Owned external Python",nativeDebugProfileLabel(profile()));assertEquals(5678,nativeAttachProfile(profile(endpoint()))!!.port);assertEquals("Attach Owned external Python",nativeDebugProfileLabel(profile(endpoint())));assertTrue(nativeAttachProfile(profile(endpoint()))!!.notice.contains("source revision unverified"));for(value in listOf<Any>("5678",0,65536,1.5,JSONObject.NULL)){try{nativeAttachProfile(profile(endpoint(value)));fail("Malformed endpoint accepted")}catch(_:Exception){}};try{nativeAttachProfile(profile(JSONObject().put("port",5678).put("arguments","{}")));fail("String arguments accepted")}catch(_:Exception){}}
 @Test fun controllerOmitsProgramAndNeverBindsExternalPositions()=runBlocking{
  val listener=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0);listener.createContext("/api/launch-profiles"){exchange->val bytes=JSONObject().put("debug_protocol",3).put("fingerprint","catalog").put("profiles",JSONArray().put(profile(endpoint()))).toString().toByteArray();exchange.sendResponseHeaders(200,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}};listener.start();val debug=NativeDebugger(Engine("http://127.0.0.1:${listener.address.port}"),this);val sent=mutableListOf<JSONObject>();debug.emitForTest={sent.add(it)};val b=DocumentBuffer(FileNode("main.py","main.py",false,emptyList(),null,null),FileText("value = 41\n","saved"))
  try{debug.prepare(b,chooseProfile=true);assertTrue(sent.isEmpty());assertEquals(1,debug.profiles.size);debug.launch(debug.profiles.single());val start=sent.single();assertFalse(start.has("program"));assertFalse(start.has("attach"));assertEquals(debugHash(b.text),start.getString("source_hash"));assertEquals("external",start.getJSONObject("profile").getString("id"));debug.receive(JSONObject().put("event","started").put("doc",languageURI("main.py")).put("request_id",start.getString("request_id")).put("session","ours").put("source_hash",start.getString("source_hash")).put("capabilities",JSONObject()));assertEquals("running",debug.status);debug.receive(JSONObject().put("event","stopped").put("session","ours").put("stop",1).put("thread",1).put("line",0).put("frames",JSONArray().put(JSONObject().put("id",1).put("name","external").put("source","main.py").put("source_line",0).put("line",0).put("in_document",true))).put("variables",JSONArray()));assertEquals("paused",debug.status);assertFalse(debug.sourceMatches(b));assertNull(debug.line("main.py"));debug.rememberSource(b);assertFalse(debug.sourceMatches(b));assertTrue(debug.attachReading!!.contains("loopback port 5678"));val count=sent.size;debug.toggle(b,0);assertEquals(count,sent.size);debug.step("over");assertEquals("step",sent.last().getString("op"));debug.stop();assertEquals("stop",sent.last().getString("op"))}finally{debug.close();listener.stop(0)}
 }
 @Test(timeout=180000) fun liveOwnedLoopbackAttachRetainsEditorAndExternalTarget()=runBlocking{
  val origin=System.getenv("HICKORY_NATIVE_ATTACH_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_NATIVE_ATTACH_WORKSPACE") ?: error("Use scripts/test-debug-attach.py --serve-only"));val endpoint=System.getenv("HICKORY_NATIVE_ATTACH_ENDPOINT")!!.toInt();val pid=System.getenv("HICKORY_NATIVE_ATTACH_TARGET_PID")!!.toLong();check(pid>0&&java.lang.ProcessHandle.of(pid).orElseThrow().isAlive)
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(background);val server=Engine(origin!!);val debug=NativeDebugger(server,background);w.engine=server;w.debugger=debug
  try{val (_,tree)=server.files();check(root.toPath().toRealPath().toString()==server.sourceRootAddress);w.rootFiles=tree;val b=w.loadBuffer(tree.single{it.path=="main.py"});w.show("main.py");val state=b.state;val original=b.text;b.selectSourceRange(2,2,b.text);b.sourceScroll.scrollTo(9);val scroll=b.sourceScroll.value
   suspend fun waitFor(ready:()->Boolean){withTimeout(20000){while(!ready()){check(debug.status!="failed"){debug.message};delay(20)}}}
   debug.prepare(b,chooseProfile=true);val selected=debug.profiles.single{nativeAttachProfile(it)?.port==endpoint};assertTrue(nativeDebugProfileLabel(selected).startsWith("Attach "));debug.launch(selected);waitFor{debug.status=="paused"};assertFalse(debug.sourceMatches(b));assertNull(debug.line("main.py"));assertTrue(debug.attachReading!!.contains("source revision unverified"));assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"\n# retained attach draft\n")),b.text));val draft=b.text;val selection=b.state.selection;debug.evaluate("value",true);waitFor{debug.values["value"]=="41"};val old=debug.frames;debug.step("over");waitFor{debug.status=="paused"&&debug.frames!==old};debug.evaluate("value",true);waitFor{debug.values["value"]=="41"};assertSame(state,b.state);assertSame(b,w.currentBuffer);assertEquals(draft,b.text);assertEquals(selection,b.state.selection);assertEquals(scroll,b.sourceScroll.value);debug.stop();waitFor{!debug.active};assertTrue(java.lang.ProcessHandle.of(pid).orElseThrow().isAlive);assertEquals(original,root.resolve("main.py").readText());assertTrue(b.snippets.undoSource());assertEquals(original,b.text);assertTrue(b.snippets.redoSource());assertEquals(draft,b.text)
  }finally{if(debug.active)debug.stop();debug.close();w.buffers.values.forEach{it.language?.close()};background.cancel()}
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/NativeDebugAttachTest.kt"><ui:paste select="#debug-attach-android-tests"/></ui:file>
</ui:doc>
