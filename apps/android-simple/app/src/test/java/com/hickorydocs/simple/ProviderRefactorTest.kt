package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import androidx.compose.ui.text.TextRange
import org.json.JSONArray
class ProviderRefactorTest {
 @Test fun genuineExtractAndGenerationKeepDirtyEditorAndHistory()=runBlocking {
  val origin=System.getenv("HICKORY_REFACTOR_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_REFACTOR_WORKSPACE")?:error("Owned fixture required"));check(java.io.File(root,".hickory-refactor-fixture").readText()=="owned-refactor-v1\n");val artifacts=java.io.File(System.getenv("HICKORY_REFACTOR_ARTIFACTS")?:error("Owned artifacts required"));val cases=JSONArray(java.io.File(root,".hickory-refactor-cases.json").readText());val scope=CoroutineScope(SupervisorJob()+Dispatchers.Default);val w=Workspace(scope);val engine=Engine(origin!!);w.engine=engine;val (_,tree)=engine.files();w.rootFiles=tree
  try{for(i in 0 until cases.length()){
   val item=cases.getJSONObject(i);val path=item.getString("path");val source=item.getString("source");val selected=item.getString("selection");val title=item.getString("title");val b=w.loadBuffer(tree.first{it.path==path});w.show(path);val state=b.state;val scroll=b.sourceScroll;b.state.edit{replace(length,length,"// retained dirty\n")};b.advanceSourceRevision();val draft=b.text;assertEquals(source+"// retained dirty\n",draft);val start=draft.indexOf(selected);b.state.edit{selection=TextRange(start,start+selected.length)};val selection=b.state.selection
   val descriptors=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()});val command=w.commands(descriptors).first{it.id=="symbol.actions"};assertNull(command.reason);command.run();withTimeout(30000){while(w.changes.choices.none{it.optString("title")==title}){w.changes.prepare(w);if(w.changes.choices.none{it.optString("title")==title})delay(50)}};w.changes.choose(w.changes.choices.indexOfFirst{it.optString("title")==title},w);assertNotNull(w.changes.notice,w.changes.plan);assertEquals(draft,b.text);assertEquals(selection,b.state.selection);w.changes.apply(w);val changed=b.text;assertTrue(w.changes.notice,changed.contains(item.getString("expected")));assertTrue(changed.contains("// retained dirty"));java.io.File(artifacts,path).writeText(changed);w.changes.open("undo",w);assertEquals(draft,b.text);w.changes.open("redo",w);assertEquals(changed,b.text);w.changes.close(w);assertSame(state,b.state);assertSame(scroll,b.sourceScroll);assertEquals(source,java.io.File(root,path).readText())
  }}finally{w.buffers.values.forEach{it.language?.close()};scope.cancel()}
 }
}
