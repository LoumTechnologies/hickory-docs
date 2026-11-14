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
