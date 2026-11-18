package com.hickorydocs.simple
import java.io.File
import kotlinx.coroutines.*
import org.junit.Assert.*
import org.junit.Test
class RustSelectedTestDebugTest{
 @Test(timeout=180000)fun liveSavedRustSelectionAndHistory()=runBlocking{
  val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE")?:error("Use maintained Rust test Debug fixture")).canonicalFile;assertEquals("owned-rust-test-debug-v1\n",root.resolve(".hickory-rust-test-debug-fixture").readText());val scope=CoroutineScope(SupervisorJob()+Dispatchers.Default);val engine=Engine(origin!!)
  try{val(_,tree)=engine.files();fun find(path:String,nodes:List<FileNode> = tree):FileNode?{for(node in nodes){if(node.path==path)return node;find(path,node.children)?.let{return it}};return null}
   for(path in listOf("physical/src/lib.rs","tests.md")){val w=Workspace(scope);w.engine=engine;w.rootFiles=tree;val file=find(path)?:error("Owned Rust source missing");val b=w.loadBuffer(file);w.show(path);val source=b.text;val state=b.state;val selection=state.selection;val scroll=b.sourceScroll;val debug=NativeDebugger(engine,scope);w.debugger=debug
    try{suspend fun wait(label:String,ready:()->Boolean){withTimeout(45000){while(!ready())delay(20)}}
     val line=source.lines().indexOfFirst{it.contains("owned-rust-stop")};assertTrue(line>=0);debug.toggle(b,line);w.showTests=true;w.testSources.refresh(w);val facts=w.testSources.observation?.second?:error(w.testSources.notice);val candidate=facts.candidates.single{it.name=="test_selected"};assertEquals("rust",candidate.language);w.testSources.debug(candidate,TestSourceSelection.NAME_FILTER,w);wait("Selected Rust pause"){debug.status=="paused"||debug.status=="failed"};assertEquals(debug.message,"paused",debug.status);assertEquals(line,debug.line(path)?:-1);assertEquals(1,w.testSources.debugRuns.size);assertFalse(root.resolve("sibling-ran").exists());debug.toggle(b,line);debug.step("continue");wait("Selected Rust completion"){debug.status=="finished"||debug.status=="failed"};assertEquals(debug.message,"finished",debug.status);assertSame(state,b.state);assertSame(scroll,b.sourceScroll);assertEquals(selection,state.selection);assertEquals(source,b.text);assertEquals(source,root.resolve(path).readText());assertFalse(root.resolve("sibling-ran").exists());assertFalse(root.resolve("Cargo.toml").exists())
    }finally{if(debug.active)debug.stop();debug.close();w.buffers.values.forEach{it.language?.close()}}
   }
  }finally{scope.cancel()}
 }
}
