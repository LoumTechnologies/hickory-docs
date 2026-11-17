package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class InstalledLLDBTest {
 @Test fun liveOwnedInstalledLLDB()=runBlocking {
  val origin=System.getenv("HICKORY_LLDB_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_LLDB_WORKSPACE")?:error("Owned root required"));val mode=System.getenv("HICKORY_LLDB_MODE")?:error("Owned mode required");check(java.io.File(root,".hickory-installed-lldb-fixture").readText()=="owned-installed-lldb-v1\n")
  val scope=CoroutineScope(SupervisorJob()+Dispatchers.Default);val engine=Engine(origin!!);val workspace=Workspace(scope);workspace.engine=engine
  try{val (_,tree)=engine.files();workspace.rootFiles=tree;val file=tree.first{it.path=="main.c"};val buffer=workspace.loadBuffer(file);val source=buffer.text;val state=buffer.state;val scroll=buffer.sourceScroll;val debug=NativeDebugger(engine,scope);workspace.debugger=debug
   try{suspend fun wait(predicate:()->Boolean){withTimeout(30000){while(!predicate())delay(20)}}
    if(mode=="launch")debug.toggle(buffer,5);debug.prepare(buffer);wait{debug.status=="paused"||debug.status=="failed"};assertEquals(debug.message,"paused",debug.status);assertTrue(debug.frames.isNotEmpty())
    if(mode=="launch"){debug.evaluate("value");wait{debug.values["value"]!=null};assertEquals("(volatile int) $0 = 41",debug.values["value"])}
    assertSame(state,buffer.state);assertSame(scroll,buffer.sourceScroll);assertEquals(source,buffer.text);debug.stop();wait{!debug.active};assertEquals(source,java.io.File(root,file.path).readText())
   }finally{debug.close()}
  }finally{workspace.buffers.values.forEach{it.language?.close()};scope.cancel()}
 }
}
