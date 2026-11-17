package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
import org.json.JSONArray
class ActionChoiceTest {
 @Test fun originalTargetOwnersNeverRenewDuringQueries()=runBlocking {
  for(change in listOf("undo","guard","baseline","handle","unknown")){
   val w=Workspace(this);val e=Engine("http://127.0.0.1:1");val f=FileNode("a.py","a.py",false,emptyList(),null,null);val other=FileNode("other.py","other.py",false,emptyList(),null,null);val b=DocumentBuffer(f,FileText("old\n","base"));val target=DocumentBuffer(other,FileText("old\n","base"));b.language=NativeLanguage(e,b,this);w.buffers[f.path]=b;w.buffers[other.path]=target;w.show(f.path)
   val action=JSONObject().put("title","Fix all").put("kind","source.fixAll.provider");w.changes.open("fixAll",w);w.changes.prepare(w,{_,_->when(change){"undo"->{target.state.edit{replace(0,length,"later")};target.advanceSourceRevision();target.state.edit{replace(0,length,"old\n")};target.advanceSourceRevision()};"guard"->target.saveGuard="foreign";"baseline"->target.base="foreign";"handle"->w.buffers[other.path]=DocumentBuffer(other,FileText(target.text,target.hash));else->w.buffers["unknown.md"]=DocumentBuffer(FileNode("unknown.md","unknown.md",false,emptyList(),null,null),FileText("unmodeled construction","new"))};JSONArray().put(action)});assertTrue(change,w.changes.choices.isEmpty());assertNull(w.changes.plan);assertEquals("old\n",b.text)
  }
 }
 @Test fun acquiredEditUndoCannotBecomeANewProviderBaseline()=runBlocking {
  val w=Workspace(this);val e=Engine("http://127.0.0.1:1");val f=FileNode("a.py","a.py",false,emptyList(),null,null);val b=DocumentBuffer(f,FileText("old\n","base"));b.language=NativeLanguage(e,b,this);w.buffers[f.path]=b;w.show(f.path)
  val action=JSONObject().put("title","Fix all").put("kind","source.fixAll.provider").put("data",JSONObject().put("hickory_acquire",JSONArray().put(JSONObject().put("uri",languageURI("other.py")))))
  w.changes.open("fixAll",w);w.changes.prepare(w,{_,_->JSONArray().put(action)},{it},{path->val target=DocumentBuffer(FileNode(path,path,false,emptyList(),null,null),FileText("old\n","base"));w.buffers[path]=target;target.state.edit{replace(0,length,"later")};target.advanceSourceRevision();target.state.edit{replace(0,length,"old\n")};target.advanceSourceRevision();SourceSnapshot(target.text,true)});w.changes.choose(0,w);assertNull(w.changes.plan);assertEquals("old\n",b.text);assertTrue(w.changes.notice,w.changes.notice.contains("acquired target")||w.changes.notice.contains("sealed"))
 }

 @Test fun liveOwnedBulkFixCommandReviewApplyAndUndo()=runBlocking {
  val origin=System.getenv("HICKORY_BULK_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_BULK_WORKSPACE") ?: error("Owned workspace required"));check(java.io.File(root,".hickory-bulk-fixture").readText()=="owned-bulk-v1\n")
  val ownedScope=CoroutineScope(SupervisorJob()+Dispatchers.Default);val w=Workspace(ownedScope);val e=Engine(origin!!);w.engine=e;val (_,tree)=e.files();w.rootFiles=tree
  try{val f=tree.first{it.path=="a.py"};val b=w.loadBuffer(f);w.show(f.path);val state=b.state;val source=b.text;val scroll=b.sourceScroll;b.state.edit{selection=androidx.compose.ui.text.TextRange(2)};val selection=b.state.selection
   val descriptors=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()});val command=w.commands(descriptors).first{it.id=="symbol.fixAll"};assertNull(command.reason);command.run();assertEquals("fixAll",w.changes.mode)
   withTimeout(20000){while(w.changes.choices.isEmpty()){w.changes.prepare(w);if(w.changes.choices.isEmpty())delay(100)}}
   assertEquals(1,w.changes.choices.size);w.changes.choose(0,w);assertEquals(w.changes.notice,2,w.changes.plan?.patches?.size);assertEquals(source,b.text);assertEquals(selection,b.state.selection);assertSame(state,b.state);assertSame(scroll,b.sourceScroll);assertEquals(source,java.io.File(root,"a.py").readText());assertEquals(source,java.io.File(root,"other.py").readText())
   w.changes.apply(w);assertEquals("python_fixed = 1\n",b.text);assertEquals("other_fixed = 1\n",w.buffers["other.py"]?.text);w.changes.open("undo",w);assertEquals(source,b.text);assertEquals(source,w.buffers["other.py"]?.text);w.changes.open("redo",w);assertEquals("python_fixed = 1\n",b.text);assertSame(state,b.state);assertSame(scroll,b.sourceScroll);assertEquals(source,java.io.File(root,"a.py").readText());assertEquals(source,java.io.File(root,"other.py").readText())
  }finally{w.buffers.values.forEach{it.language?.close()};ownedScope.cancel()}
 }

 @Test fun completeFileFixChoiceFiltersPreviewsAndUsesCoordinatedUndo()=runBlocking {
  val w=Workspace(this);val e=Engine("http://127.0.0.1:1");val f=FileNode("a.py","a.py",false,emptyList(),null,null);val b=DocumentBuffer(f,FileText("old\n","base"));b.language=NativeLanguage(e,b,this);w.buffers[f.path]=b;w.show(f.path)
  val action=JSONObject().put("title","Fix all").put("kind","source.fixAll.provider").put("edit",JSONObject().put("changes",JSONObject().put(languageURI(f.path),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",3))).put("newText","new")))))
  val choices=JSONArray().put(action).put(JSONObject().put("title","single").put("kind","quickfix")).put(JSONObject().put("title","unclassified")).put(JSONObject().put("title","prefix").put("kind","source.fixAllWrong"))
  w.changes.open("fixAll",w);w.changes.prepare(w,{kind,_->assertEquals("fixAll",kind);choices});assertEquals(1,w.changes.choices.size);w.changes.choose(0,w);assertEquals("new\n",w.changes.plan?.patches?.first()?.after);assertEquals("old\n",b.text)
  w.changes.apply(w);assertEquals("new\n",b.text);w.changes.open("undo",w);assertEquals("old\n",b.text);w.changes.open("redo",w);assertEquals("new\n",b.text)
  w.changes.open("fixAll",w);w.changes.prepare(w,{_,_->JSONArray().put(JSONObject().put("kind","quickfix"))});assertTrue(w.changes.choices.isEmpty());assertTrue(w.changes.notice.contains("no file-wide"))
  w.changes.open("fixAll",w);w.changes.prepare(w,{_,_->b.state.edit{replace(0,length,"later")};b.advanceSourceRevision();b.state.edit{replace(0,length,"new\n")};b.advanceSourceRevision();JSONArray().put(action)});assertTrue(w.changes.choices.isEmpty());assertNull(w.changes.plan)
  w.changes.open("fixAll",w);w.changes.prepare(w,{_,_->JSONArray().put(action)},{JSONObject(action.toString()).put("kind","quickfix")});w.changes.choose(0,w);assertNull(w.changes.plan);assertEquals("new\n",b.text)
 }

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
  workspace.buffers.getValue("other.py").state.edit{replace(0,length,"later\n")};changes.apply(workspace);assertTrue(changes.notice.contains("Source, selection or provider changed"));assertEquals("old\n",buffer.text)
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
