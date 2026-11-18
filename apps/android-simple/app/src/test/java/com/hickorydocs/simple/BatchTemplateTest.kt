package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
class BatchTemplateTest {
 @Test(timeout=90000) fun liveCompleteTemplateGroupAndHistory()=kotlinx.coroutines.runBlocking {
  val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN");org.junit.Assume.assumeNotNull(origin)
  val program3=System.getenv("HICKORY_NATIVE_TEMPLATE_PROGRAM3")=="1"
  val root=java.io.File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE") ?: error("Use the owned --batch fixture")).canonicalFile;check(root.resolve(".hickory-native-template-fixture").readText()=="owned-native-templates-v1\n")
  val engine=Engine(origin!!);val (_,tree)=engine.files();val workspace=Workspace(this);workspace.engine=engine;workspace.rootFiles=tree
  fun find(path:String,nodes:List<FileNode> = tree):FileNode?{for(node in nodes){if(node.path==path)return node;find(path,node.children)?.let{return it}};return null}
  suspend fun load(path:String):DocumentBuffer{workspace.buffers[path]?.let{return it};val file=find(path) ?: error("Missing $path");return DocumentBuffer(file,engine.file(file)).also{workspace.buffers[path]=it}}
  fun replace(buffer:DocumentBuffer,text:String){buffer.state.edit{replace(0,length,text)};buffer.advanceSourceRevision()}
  val project=load("project.md");val savedProject=project.text;replace(project,savedProject.replace("saved.py","draft.py"))
  val descriptors=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()});fun command(id:String)=workspace.commands(descriptors).single{it.id==id}
  val model=workspace.sourceTemplates;model.captureVariables={SnippetVariableSnapshot(1704067200000,0,"0123456789abcdef0123456789abcdef")};var reads=0;engine.snippetClipboardReader={reads++;null}
  try {
   for(path in listOf("notes.md","part.md")){
    val buffer=load(path);val saved=buffer.text;replace(buffer,saved.replace(if(path=="part.md")"saved" else "old","old two"));val source=buffer.text;workspace.show(path);val state=buffer.state;val scroll=buffer.sourceScroll.value
    val first=source.indexOf("old");val second=source.indexOf("two",first+3);check(first>=0&&second>first);val ranges=listOf(NativeSourceEdit(first,first+3,""),NativeSourceEdit(second,second+3,""));assertTrue(buffer.selectMultiple(ranges,source))
    suspend fun open(){val action=command("source.templates");assertNull(action.reason);action.run();kotlinx.coroutines.yield();repeat(500){if(model.catalog?.capture?.path==path&&model.current(workspace,model.catalog!!.capture)&&!model.busy)return;kotlinx.coroutines.delay(20)};error(model.notice)}
    suspend fun choose(id:String){model.prepare(workspace,model.catalog!!.entries.single{it.id==id})}
    fun complete(value:String,uppercase:String)=source.substring(0,first)+"#0:1:2024:012345:$value|$value|$uppercase #1:2:2024:012345:$value|$value|$uppercase"+source.substring(second+3)
    open();val catalog=model.catalog ?: error(model.notice);assertNotNull(catalog.capture.carets);assertEquals("python",catalog.language);assertEquals(listOf(2,3),catalog.capture.supportedPrograms);assertEquals(debugHash(project.text),org.json.JSONObject(catalog.sourceContext!!).getJSONObject("source_hashes").getString("project.md"))
    choose("workspace.batch-context");val originalPreview=model.reviewed?.plan?.completeSource ?: error(model.notice);assertEquals(2,model.reviewed!!.plan.batch!!.expansions.flatMap{it.fields}.count{it.transform?.v3!=null});assertEquals(source,buffer.text);val accepted=model.apply(workspace);assertTrue(model.notice,accepted);assertEquals(originalPreview,buffer.text);assertTrue(buffer.text.contains(if(program3)"#0:1:2024:012345:old|old|oXX #1:2:2024:012345:two|two|tXX" else "#0:1:2024:012345:old|old|OLD #1:2:2024:012345:two|two|TWO"));assertNotNull(buffer.snippets.parallel)
    assertTrue(buffer.snippets.inputParallel("ßa"));val typed=complete("ßa",if(program3)"ßX" else "SSA");assertEquals(typed,buffer.text)
    val beforeMove=buffer.multiRanges;command("editor.snippetNext").run();assertNotEquals(beforeMove,buffer.multiRanges);command("editor.snippetPrevious").run();assertEquals(listOf(2,2),buffer.multiRanges.map{it.to-it.from})
    open();choose("workspace.batch-nested");assertNotNull(model.notice,model.reviewed?.plan?.batch);val nestedAccepted=model.apply(workspace);assertTrue(model.notice,nestedAccepted);val nested=complete("kid-kid",if(program3)"kXX-kXX" else "KID-KID");assertEquals(nested,buffer.text)
    assertTrue(buffer.snippets.inputParallel("ßa"));val nestedTyped=complete("ßa-ßa",if(program3)"ßX-ßX" else "SSA-SSA");assertEquals(nestedTyped,buffer.text)
    for(expected in listOf(nested,typed,originalPreview,source)){command("editor.undo").run();assertEquals(expected,buffer.text)}
    for(expected in listOf(originalPreview,typed,nested,nestedTyped)){command("editor.redo").run();assertEquals(expected,buffer.text)}
    repeat(4){command("editor.undo").run()};assertEquals(source,buffer.text)
    assertTrue(buffer.selectMultiple(ranges,source));val beforeClipboard=reads;open();choose("workspace.clipboard");assertEquals(beforeClipboard+1,reads);assertNotNull(model.notice,model.reviewed?.plan?.batch);val deniedAccepted=model.apply(workspace);assertTrue(model.notice,deniedAccepted);assertEquals(beforeClipboard+1,reads);assertEquals(source.replaceRange(second,second+3,"fallback").replaceRange(first,first+3,"fallback"),buffer.text);command("editor.undo").run();assertEquals(source,buffer.text)
    assertTrue(buffer.selectMultiple(ranges,source));open();choose("workspace.batch-context");assertNotNull(model.notice,model.reviewed);assertTrue(buffer.selectMultiple(listOf(ranges[0],NativeSourceEdit(second+3,second+3,"")),source));assertFalse(model.apply(workspace));assertEquals(source,buffer.text)
    assertTrue(buffer.selectMultiple(ranges,source));open();choose("workspace.batch-context");assertNotNull(model.notice,model.reviewed)
    val heldMultiRevision=buffer.multiSelectionRevision;val heldSourceRevision=buffer.sourceRevision;val heldPrimary=state.selection
    assertTrue(buffer.selectMultiple(listOf(ranges[0],NativeSourceEdit(second+3,second+3,"")),source));assertTrue(buffer.selectMultiple(ranges,source));assertEquals(heldPrimary,state.selection);assertTrue(buffer.multiSelectionRevision>heldMultiRevision);assertEquals(heldSourceRevision,buffer.sourceRevision)
    assertFalse(model.apply(workspace));assertEquals(source,buffer.text)
    assertSame(state,buffer.state);assertSame(buffer,workspace.currentBuffer);assertEquals(scroll,buffer.sourceScroll.value);assertEquals(saved,root.resolve(path).readText())
   }
   assertEquals(savedProject,root.resolve("project.md").readText());for(path in listOf("saved.py","draft.py","second.py","guide.md"))assertFalse(root.resolve(path).exists())
  }finally{model.close();workspace.showTemplates=false;workspace.buffers.values.forEach{it.language?.close()}}
 }
}
