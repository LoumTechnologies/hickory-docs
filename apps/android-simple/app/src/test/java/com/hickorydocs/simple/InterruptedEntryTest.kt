package com.hickorydocs.simple

import androidx.compose.ui.text.TextRange
import kotlinx.coroutines.*
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Assume.assumeNotNull
import org.junit.Test
import java.io.File

@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class InterruptedEntryTest {
 @Test(timeout=90000) fun reviewedEntryCompletion()=runBlocking {
  val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN")
  assumeNotNull(origin)
  val root=File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE") ?: error("Use the owned interrupted Move fixture.")).canonicalFile
  val marker=JSONObject(root.resolve(".hick-cache/entry-completion-fixture.json").readText())
  check(marker.getInt("version")==1&&marker.getString("kind")=="entry-interrupted-completion-fixture"&&
   marker.getString("source_root")==root.path){"Unowned interruption fixture."}
  val old=marker.getString("old_path")
  val destination=marker.getString("destination")
  val draft=marker.getString("live_text")
  val saved=marker.getString("saved_text")
  val engine=Engine(origin!!)
  val (_,tree)=engine.files()
  check(engine.sourceRootAddress==root.path){"Engine serves a different fixture root."}
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined)
  val workspace=Workspace(background)
  workspace.engine=engine;workspace.rootFiles=tree
  try {
   val model=workspace.resourceChanges
   val reading=checkNotNull(model.readInterruptedEntry(workspace))
   check(reading.canComplete&&reading.source==old&&reading.destination==destination&&
    reading.request==marker.getString("original_request_id")){"Unexpected interruption."}
   val buffer=workspace.loadBuffer(tree.first{it.path==old})
   workspace.show(old)
   val state=buffer.state
   state.edit{replace(0,length,draft);selection=TextRange(draft.length)}
   buffer.advanceSourceRevision()
   val selection=state.selection
   val scroll=buffer.sourceScroll
   assertEquals(draft,buffer.text);assertTrue(state.undoState.canUndo)

   workspace.showRecovery=true
   model.prepareEntryCompletion(workspace,reading){workspace.showRecovery}
   val review=checkNotNull(model.review){model.notice}
   assertEquals(destination,review.entryIntent?.after?.path)
   assertEquals(saved,review.saved[destination]?.text)
   assertEquals(saved,root.resolve(old).readText());assertEquals(saved,root.resolve(destination).readText())
   assertSame(buffer,workspace.currentBuffer);assertEquals(draft,buffer.text)
   model.apply(workspace,review)
   assertFalse(model.notice,model.hasPending)
   val stamp=model.readEntry(workspace){true}.proof.entry
   assertEquals(destination,stamp.path);assertEquals(reading.currentEntry.identity,stamp.identity)
   assertEquals(reading.reservationFloor+1,stamp.generation)
   assertFalse(root.resolve(old).exists());assertEquals(saved,root.resolve(destination).readText())
   assertEquals(saved,buffer.base);assertEquals(draft,buffer.text)
   assertSame(buffer,workspace.currentBuffer);assertSame(state,buffer.state);assertSame(scroll,buffer.sourceScroll)
   assertEquals(selection,state.selection)
   state.undoState.undo();assertEquals(saved,buffer.text)
   state.undoState.redo();assertEquals(draft,buffer.text)
  } finally {
   workspace.buffers.values.forEach{it.language?.close()}
   background.cancel()
  }
 }
}
