package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
import org.json.JSONArray
class ActionChoiceTest {
 @Test fun latestChoiceWinsAndOldRowsCannotBorrowNewLists()=runBlocking {
  val workspace=Workspace(this);val engine=Engine("http://127.0.0.1:1");val file=FileNode("a.py","a.py",false,emptyList(),null,null)
  val buffer=DocumentBuffer(file,FileText("old\n","base"));buffer.language=NativeLanguage(engine,buffer,this);workspace.buffers[file.path]=buffer;workspace.show(file.path)
  val changes=workspace.changes;changes.open("actions",workspace);val held=mutableMapOf<String,CompletableDeferred<JSONObject>>()
  val actions=JSONArray().put(JSONObject().put("title","first").put("data",JSONObject().put("hickory_action","first"))).put(JSONObject().put("title","second").put("data",JSONObject().put("hickory_action","second")))
  val resolve:suspend (JSONObject)->JSONObject={action->val result=CompletableDeferred<JSONObject>();held[action.getString("title")]=result;result.await()}
  suspend fun prepare(){changes.prepare(workspace,{_,_->actions},resolve)}
  suspend fun settle(condition:()->Boolean){withTimeout(2000){while(!condition())yield()}}
  fun resolved(name:String)=JSONObject().put("title",name).put("edit",JSONObject().put("changes",JSONObject().put(languageURI(file.path),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",3))).put("newText",name)))))
  prepare();val snapshot=changes.actionSnapshot;val first=async{changes.choose(0,workspace,snapshot)};settle{held["first"]!=null};val second=async{changes.choose(1,workspace,snapshot)};settle{held["second"]!=null}
  held.getValue("second").complete(resolved("second"));second.await();held.getValue("first").complete(resolved("first"));first.await();assertEquals("second\n",changes.plan?.patches?.first()?.after)
  held.clear();prepare();changes.choose(0,workspace,snapshot);assertTrue(held.isEmpty());assertNull(changes.plan)
  val current=changes.actionSnapshot;val closing=async{changes.choose(0,workspace,current)};settle{held["first"]!=null};changes.close(workspace);held.getValue("first").complete(resolved("first"));closing.await();assertNull(changes.plan);assertEquals("old\n",buffer.text)
  changes.open("actions",workspace);changes.prepare(workspace,{_,_->actions},{resolved("partial").put("command",JSONObject().put("title","Effect").put("command","provider.effect"))})
  changes.choose(0,workspace,changes.actionSnapshot);assertNull(changes.plan);assertTrue(changes.notice.contains("provider command"));assertEquals("old\n",buffer.text)
 }
}
