package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
import org.json.JSONArray
class ActionChoiceTest {
 @Test fun unopenedActionTargetsAcquireThenRequeryAndRefuseAmbiguousChoices()=runBlocking {
  val workspace=Workspace(this);val engine=Engine("http://127.0.0.1:1");val file=FileNode("a.py","a.py",false,emptyList(),null,null)
  val buffer=DocumentBuffer(file,FileText("old\n","base"));buffer.language=NativeLanguage(engine,buffer,this);workspace.buffers[file.path]=buffer;workspace.show(file.path)
  val changes=workspace.changes;val action=JSONObject().put("title","Fix both").put("kind","quickfix").put("data",JSONObject().put("hickory_action","first"))
  var queries=0;var resolutions=0;val loads=mutableListOf<String>();var ambiguous=false
  val propose:suspend (String,String)->Any={_,_->queries++;JSONArray().put(action).also{if(ambiguous&&queries>1)it.put(action)}}
  val resolve:suspend (JSONObject)->JSONObject={
   resolutions++;if(resolutions==1)JSONObject().put("title","Fix both").put("data",JSONObject().put("hickory_acquire",JSONArray().put(JSONObject().put("uri",languageURI("other.py"))))) else {val replacement=JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",3))).put("newText","new")
   JSONObject().put("title","Fix both").put("edit",JSONObject().put("changes",JSONObject().put(languageURI("a.py"),JSONArray().put(replacement)).put(languageURI("other.py"),JSONArray().put(replacement))))}
  }
  val load:suspend (String)->SourceSnapshot={path->loads.add(path);val target=DocumentBuffer(FileNode(path,path,false,emptyList(),null,null),FileText("old\n","base"));workspace.buffers[path]=target;SourceSnapshot(target.text,true)}
  changes.open("actions",workspace);changes.prepare(workspace,propose,resolve,load);changes.choose(0,workspace)
  assertEquals(2,queries);assertEquals(2,resolutions);assertEquals(listOf("other.py"),loads);assertEquals(2,changes.plan?.patches?.size);assertEquals("a.py",workspace.currentBuffer?.file?.path);assertEquals("old\n",buffer.text)
  workspace.buffers.getValue("other.py").state.edit{replace(0,length,"later\n")};changes.apply(workspace);assertTrue(changes.notice.contains("Source changed"));assertEquals("old\n",buffer.text)
  workspace.buffers.remove("other.py");queries=0;ambiguous=true
  changes.open("actions",workspace);changes.prepare(workspace,propose,resolve,load);changes.choose(0,workspace);assertNull(changes.plan);assertTrue(changes.notice.contains("ambiguous"));assertEquals("old\n",buffer.text)
 }

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
