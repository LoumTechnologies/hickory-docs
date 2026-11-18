package com.hickorydocs.simple
import androidx.compose.ui.text.TextRange
import kotlinx.coroutines.*
import org.junit.Assert.*
import org.junit.Assume.assumeNotNull
import org.junit.Test
import java.io.File
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class WorkspaceEntryLineageTest {
 @Test(timeout=120000) fun liveOlderMainMoveUndoRedo()=runBlocking {
  val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN");assumeNotNull(origin)
  val root=File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE") ?: error("Use test-workspace-entry-lineage.py --host android")).canonicalFile
  val historyID=System.getenv("HICKORY_ENTRY_LINEAGE_HISTORY") ?: error("Missing owned ordinary source history seed")
  assertEquals("owned-entry-move-v1\n",root.resolve(".hickory-entry-move-fixture").readText())
  val a="project.md";val b="literal #main.spec";val c="second #main.md";val engine=Engine(origin!!);val (_,tree)=engine.files();val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val space=Workspace(background);space.engine=engine;space.rootFiles=tree
  try{
   val buffer=space.loadBuffer(tree.first{it.path==a});space.show(a);val saved=buffer.text;val draft=saved.replace("saved.txt","draft.txt");val earlier=saved.replace("saved.txt","before.txt");val state=buffer.state;val at=saved.indexOf("saved.txt")
   state.edit{replace(at,at+"saved.txt".length,"draft.txt");selection=TextRange(at+"draft.txt".length)};buffer.advanceSourceRevision();val selection=state.selection;val scroll=buffer.sourceScroll;val original=space.resourceChanges.readEntry(space){true}.proof.entry
   val descriptors=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()})
   fun command(id:String){val selected=space.commands(descriptors).single{it.id==id};assertNull(selected.reason);selected.run()}
   suspend fun wait(ready:()->Boolean){repeat(400){if(ready())return;delay(25)};error(space.changes.notice+" "+space.resourceChanges.notice)}
   suspend fun check(path:String,generation:Long){
    val stamp=space.resourceChanges.readEntry(space){true}.proof.entry;assertEquals(path,stamp.path);assertEquals(original.identity,stamp.identity);assertEquals(original.generation+generation,stamp.generation)
    assertEquals(saved,root.resolve(path).readText());for(other in listOf(a,b,c).filter{it!=path})assertFalse(root.resolve(other).exists())
    assertSame(buffer,space.currentBuffer);assertSame(state,buffer.state);assertSame(scroll,buffer.sourceScroll);assertEquals(selection,state.selection);assertEquals(saved,buffer.base);assertEquals(draft,buffer.text);for(output in listOf("saved.txt","draft.txt"))assertFalse(root.resolve(output).exists())
   }
   suspend fun publish(){space.resourceChanges.apply(space);assertFalse(space.resourceChanges.notice,space.resourceChanges.hasPending)}
   suspend fun move(destination:String){command("workspace.moveEntry");wait{space.changes.entrySelection?.entryPath==buffer.file.path};space.changes.updateName(destination);space.changes.prepare(space);assertEquals(space.resourceChanges.notice,destination,space.resourceChanges.review?.entryIntent?.after?.path);publish()}
   check(a,0);move(b);check(b,1);move(c);check(c,2)
   for((id,path,generation) in listOf(Triple("workspace.undo",b,3L),Triple("workspace.undo",a,4L),Triple("workspace.redo",b,5L),Triple("workspace.redo",c,6L))){command(id);wait{space.resourceChanges.review!=null};assertEquals(space.resourceChanges.notice,path,space.resourceChanges.review?.entryIntent?.after?.path);publish();check(path,generation)}
   space.sourceHistory.open(space);val historical=space.sourceHistory.reading?.entries?.singleOrNull{it.id==historyID} ?: error(space.sourceHistory.notice);assertTrue(historical.available);assertEquals(earlier,historical.before.text);assertEquals(saved,historical.after.text);space.sourceHistory.close()
   assertTrue(state.undoState.canUndo);state.undoState.undo();assertEquals(saved,buffer.text);state.undoState.redo();check(c,6)
   command("workspace.undo");wait{space.resourceChanges.review!=null}
   val path=root.resolve(c).toPath();val before=java.nio.file.Files.readAttributes(path,java.nio.file.attribute.BasicFileAttributes::class.java).fileKey();val replacement=root.resolve("independent-replacement");replacement.writeText(saved)
   java.nio.file.Files.move(replacement.toPath(),path,java.nio.file.StandardCopyOption.ATOMIC_MOVE,java.nio.file.StandardCopyOption.REPLACE_EXISTING);assertNotEquals(before,java.nio.file.Files.readAttributes(path,java.nio.file.attribute.BasicFileAttributes::class.java).fileKey())
   space.resourceChanges.apply(space)
   assertTrue(space.resourceChanges.notice,space.resourceChanges.hasPending)
   assertTrue(space.resourceChanges.notice,space.resourceChanges.notice.contains("physical source guard"))
   space.resourceChanges.status(space)
   assertTrue(space.resourceChanges.notice,space.resourceChanges.hasPending)
   assertTrue(space.resourceChanges.notice,space.resourceChanges.notice.contains("physical source guard"))
   assertSame(buffer,space.currentBuffer);assertEquals(c,buffer.file.path);assertEquals(draft,buffer.text);assertEquals(saved,buffer.base);assertEquals(selection,state.selection);assertSame(scroll,buffer.sourceScroll);assertEquals(saved,root.resolve(c).readText());for(absent in listOf(a,b,"saved.txt","draft.txt"))assertFalse(root.resolve(absent).exists())
  }finally{background.cancel()}
 }
}
