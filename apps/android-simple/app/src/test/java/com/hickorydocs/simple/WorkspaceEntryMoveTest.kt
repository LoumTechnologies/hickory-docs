package com.hickorydocs.simple

import androidx.compose.ui.text.TextRange
import kotlinx.coroutines.*
import org.junit.Assert.*
import org.junit.Assume.assumeNotNull
import org.junit.Test
import java.io.File

@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class WorkspaceEntryMoveTest {
 @Test(timeout=90000) fun liveMainMoveUndoRedo()=runBlocking {
  val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN")
  assumeNotNull(origin)
  val root=File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE") ?: error("Use the owned entry Move fixture")).canonicalFile
  assertEquals("owned-entry-move-v1\n",root.resolve(".hickory-entry-move-fixture").readText())
  val old="project.md"
  val foreign=System.getenv("HICKORY_ENTRY_MOVE_FOREIGN_ROOM")
  val new=foreign ?: "literal #main.spec"
  val engine=Engine(origin!!)
  val (_,tree)=engine.files()
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined)
  val space=Workspace(background)
  space.engine=engine;space.rootFiles=tree
  try {
   val file=tree.first{it.path==old}
   val buffer=space.loadBuffer(file)
   space.show(old)
   val saved=buffer.text
   val draft=saved.replace("saved.txt","draft.txt")
   val state=buffer.state
   val at=saved.indexOf("saved.txt")
   state.edit{replace(at,at+"saved.txt".length,"draft.txt");selection=TextRange(at+"draft.txt".length)}
   buffer.advanceSourceRevision()
   assertEquals(draft,buffer.text)
   val selection=state.selection
   val scroll=buffer.sourceScroll
   val original=space.resourceChanges.readEntry(space){true}.proof.entry
   assertEquals(old,original.path)

   val raw=javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()}
   val descriptors=commandDescriptors(raw)
   fun command(id:String){
    val selected=space.commands(descriptors).single{it.id==id}
    assertNull(selected.reason)
    selected.run()
   }
   suspend fun wait(ready:()->Boolean){
    repeat(400){if(ready())return;delay(25)}
    error(space.changes.notice+" "+space.resourceChanges.notice)
   }
   suspend fun assertState(path:String,absent:String,advance:Long){
    val stamp=space.resourceChanges.readEntry(space){true}.proof.entry
    assertEquals(path,stamp.path)
    assertEquals(original.identity,stamp.identity)
    assertEquals(original.generation+advance,stamp.generation)
    assertEquals(saved,root.resolve(path).readText())
    assertFalse(root.resolve(absent).exists())
    assertSame(buffer,space.currentBuffer)
    assertSame(state,buffer.state)
    assertSame(scroll,buffer.sourceScroll)
    assertEquals(selection,state.selection)
    assertEquals(saved,buffer.base)
    assertEquals(draft,buffer.text)
   }
   suspend fun publish(){
    space.resourceChanges.apply(space)
    assertFalse(space.resourceChanges.notice,space.resourceChanges.hasPending)
   }
   command("workspace.moveEntry")
   wait{space.changes.entrySelection?.entryPath==old}
   space.changes.updateName(new)
   space.changes.prepare(space)
   assertEquals(space.resourceChanges.notice,new,space.resourceChanges.review?.entryIntent?.after?.path)
   assertState(old,new,0)
   publish()
   if(foreign!=null){
    assertTrue(space.resourceChanges.notice,space.resourceChanges.notice.contains("Publication ended before any source write"))
    assertState(old,new,0)
    assertTrue(state.undoState.canUndo)
    state.undoState.undo();assertEquals(saved,buffer.text)
    state.undoState.redo();assertEquals(draft,buffer.text)
    command("workspace.moveEntry");wait{space.changes.entrySelection?.entryPath==old}
    space.changes.updateName(new);space.changes.prepare(space)
    assertNotNull(space.resourceChanges.notice,space.resourceChanges.review)
    return@runBlocking
   }
   assertState(new,old,1)

   command("workspace.undo")
   wait{space.resourceChanges.review!=null}
   assertEquals(old,space.resourceChanges.review?.entryIntent?.after?.path)
   publish()
   assertState(old,new,2)
   command("workspace.redo")
   wait{space.resourceChanges.review!=null}
   assertEquals(new,space.resourceChanges.review?.entryIntent?.after?.path)
   publish()
   assertState(new,old,3)

   assertTrue(state.undoState.canUndo)
   state.undoState.undo();assertEquals(saved,buffer.text)
   state.undoState.redo();assertEquals(draft,buffer.text)
   assertFalse(root.resolve("saved.txt").exists())
   assertFalse(root.resolve("draft.txt").exists())
  }finally{background.cancel()}
 }
}
