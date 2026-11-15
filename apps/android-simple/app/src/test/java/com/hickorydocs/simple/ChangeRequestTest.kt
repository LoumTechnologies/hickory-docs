package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
import org.json.JSONArray
class ChangeRequestTest {
 @Test fun olderRenameReplyCannotBorrowNewerRequestAuthority()=runBlocking {
  val workspace=Workspace(this);val engine=Engine("http://127.0.0.1:1");val file=FileNode("a.py","a.py",false,emptyList(),null,null)
  val buffer=DocumentBuffer(file,FileText("value\n","base"));buffer.language=NativeLanguage(engine,buffer,this);workspace.buffers[file.path]=buffer;workspace.show(file.path);assertSame(buffer,workspace.currentBuffer)
  val changes=workspace.changes;changes.open("rename",workspace);val held=mutableMapOf<String,CompletableDeferred<Any>>()
  val proposal:suspend (String,String)->Any={kind,name->if(kind=="references")JSONArray() else {val done=CompletableDeferred<Any>();held[name]=done;done.await()}}
  fun edit(name:String)=JSONObject().put("changes",JSONObject().put(languageURI(file.path),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",5))).put("newText",name))))
  suspend fun settle(condition:()->Boolean){withTimeout(2000){while(!condition())yield()}}
  changes.updateName("first");val first=async{changes.prepare(workspace,proposal)};settle{held["first"]!=null}
  changes.updateName("second");val second=async{changes.prepare(workspace,proposal)};settle{held["second"]!=null};held.getValue("second").complete(edit("second"));second.await();assertEquals("second\n",changes.plan?.patches?.first()?.after)
  held.getValue("first").complete(edit("first"));first.await();assertEquals("second\n",changes.plan?.patches?.first()?.after);assertEquals("value\n",buffer.text)
 }
}
class CleanupRequestTest {
 @Test fun cleanupUsesCapturedSourceWithoutAProviderAndRetainsWorkspaceUndo()=runBlocking {
  val workspace=Workspace(this);val file=FileNode("a.py","a.py",false,emptyList(),null,null)
  val before="😀 value \t\r\n";val after="😀 value\r\n";val buffer=DocumentBuffer(file,FileText(before,"base"))
  workspace.buffers[file.path]=buffer;workspace.show(file.path);val changes=workspace.changes
  fun edit()=JSONObject("""{"changes":{"hick:///a.py":[{"range":{"start":{"line":0,"character":8},"end":{"line":0,"character":10}},"newText":""}]}}""")
  changes.open("cleanup",workspace);changes.prepare(workspace,{kind,_->assertEquals("cleanup",kind);edit()})
  assertEquals(before,buffer.text);assertEquals(after,changes.plan?.patches?.first()?.after)
  changes.apply(workspace);assertEquals(after,buffer.text);changes.open("undo",workspace);assertEquals(before,buffer.text)
  changes.open("redo",workspace);assertEquals(after,buffer.text);assertEquals(before,buffer.base)
  changes.open("cleanup",workspace);changes.prepare(workspace,{_,_->JSONObject("""{"changes":{"hick:///a.py":[]}}""")})
  assertNull(changes.plan);assertTrue(changes.notice.contains("No trailing"))
 }
 @Test fun cleanupCannotBorrowNewerAuthorityOrApplyAfterClose()=runBlocking {
  val workspace=Workspace(this);val file=FileNode("a.py","a.py",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText("x ","base"))
  workspace.buffers[file.path]=buffer;workspace.show(file.path);val changes=workspace.changes;changes.open("cleanup",workspace)
  val held=mutableListOf<CompletableDeferred<Any>>()
  val proposal:suspend (String,String)->Any={_,_->CompletableDeferred<Any>().also{held.add(it)}.await()}
  suspend fun settle(count:Int){withTimeout(2000){while(held.size<count)yield()}}
  fun edit(text:String)=JSONObject().put("changes",JSONObject().put("hick:///a.py",JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",1)).put("end",JSONObject().put("line",0).put("character",2))).put("newText",text))))
  val first=async{changes.prepare(workspace,proposal)};settle(1);val second=async{changes.prepare(workspace,proposal)};settle(2)
  held[1].complete(edit(""));second.await();assertEquals("x",changes.plan?.patches?.first()?.after)
  held[0].complete(edit("old"));first.await();assertEquals("x",changes.plan?.patches?.first()?.after)
  val closed=async{changes.prepare(workspace,proposal)};settle(3);changes.close(workspace);held[2].complete(edit(""));closed.await()
  assertNull(changes.plan);assertEquals("x ",buffer.text)
 }
}
