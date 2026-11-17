@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import androidx.compose.ui.text.TextRange
class EditorRecoveryManagementAcceptanceTest {
 @Test(timeout=120000) fun ownedCapacityRevisionAndForgetFlow()=runBlocking {
  val origin=System.getenv("HICKORY_RECOVERY_MANAGEMENT_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_RECOVERY_MANAGEMENT_WORKSPACE") ?: error("Use owned recovery fixture"));check(root.resolve(".hickory-native-recovery-management-fixture").readText()=="owned-recovery-management-v1\n")
  val scope=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(scope);val engine=Engine(origin!!);w.engine=engine;val (_,tree)=engine.files();w.rootFiles=tree
  fun find(nodes:List<FileNode>):FileNode?{for(node in nodes){if(node.path=="note.py")return node;find(node.children)?.let{return it}};return null}
  val b=w.loadBuffer(find(tree) ?: error("Missing owned source"));w.show(sourceTab(b.file.path));val state=b.state;assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"# independent dirty draft\n")),b.text));state.edit{selection=TextRange(2,5)};val text=b.text;val selection=state.selection;val m=w.editorRecovery
  suspend fun allPages(){repeat(32){val previous=m.catalogue?.next ?: return;m.more(w);check(m.catalogue?.next!=previous){"Recovery page did not advance: "+m.notice}};check(m.catalogue?.next==null){"Recovery exceeded bounded page count."}}
  try{
   m.open(w);allPages();assertEquals(m.notice,31,m.catalogue!!.sessions.size);assertEquals(1,m.catalogue!!.unreadable.size)
   val row=m.catalogue!!.sessions.first{it.client=="%032x".format(32)}
   for(revision in 1L..3L){m.read(row,w,revision);assertEquals(m.notice,"private session 32 revision $revision\n",m.snapshot?.buffers?.first()?.text);assertTrue(m.notice,m.canRestore)}
   m.read(row,w,1);assertTrue(m.notice,m.restore(w));assertTrue(w.buffers.values.any{it.isUntitledDraft&&it.text=="private session 32 revision 1\n"})
   m.open(w);allPages();m.forget(m.catalogue!!.unreadable.first().removal,w);assertFalse(m.notice,m.hasPendingForget);m.forget(m.catalogue!!.sessions.first{it.client==row.client}.removal,w);assertFalse(m.notice,m.hasPendingForget)
   m.open(w);allPages();assertFalse(m.catalogue!!.sessions.any{it.client==row.client});assertSame(state,b.state);assertEquals(text,b.text);assertEquals(selection,state.selection);assertFalse(root.resolve("note.py").readText().contains("independent dirty"))
  }finally{m.close();w.buffers.values.forEach{it.language?.close()};scope.cancel()}
 }
}
