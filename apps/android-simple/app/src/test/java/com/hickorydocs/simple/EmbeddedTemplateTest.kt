package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import androidx.compose.ui.text.TextRange
class EmbeddedTemplateTest {
 @Test(timeout=90000) fun liveEmbeddedTemplateCatalogAndHistory()=kotlinx.coroutines.runBlocking {
  val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN");org.junit.Assume.assumeNotNull(origin)
  val root=java.io.File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE") ?: error("Use the owned --embedded fixture")).canonicalFile;check(root.resolve(".hickory-native-template-fixture").readText()=="owned-native-templates-v1\n")
  val engine=Engine(origin!!);val (_,tree)=engine.files();val workspace=Workspace(this);workspace.engine=engine;workspace.rootFiles=tree
  fun find(path:String,nodes:List<FileNode> = tree):FileNode?{for(node in nodes){if(node.path==path)return node;find(path,node.children)?.let{return it}};return null}
  suspend fun load(path:String):DocumentBuffer{workspace.buffers[path]?.let{return it};val file=find(path) ?: error("Missing $path");return DocumentBuffer(file,engine.file(file)).also{workspace.buffers[path]=it}}
  fun replace(buffer:DocumentBuffer,text:String){buffer.state.edit{replace(0,length,text)};buffer.advanceSourceRevision()}
  val project=load("project.md");val savedProject=project.text;replace(project,savedProject.replace("saved.py","draft.py"))
  val descriptors=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()});fun command(id:String)=workspace.commands(descriptors).single{it.id==id}
  var last:DocumentBuffer?=null
  try {
   for(path in listOf("notes.md","part.md")){
    val buffer=load(path);val saved=buffer.text;if(path=="part.md")replace(buffer,saved.replace("saved","old"));val source=buffer.text;workspace.show(path);val state=buffer.state;val scroll=buffer.sourceScroll.value;val from=source.indexOf("old");check(from>=0);buffer.selectSourceRange(from,from+3,source)
    val action=command("source.templates");assertNull(action.reason);action.run();kotlinx.coroutines.yield();val model=workspace.sourceTemplates;repeat(500){if(model.catalog==null&&(model.busy||model.notice.isEmpty()))kotlinx.coroutines.delay(20)}
    val catalog=model.catalog ?: error(model.notice);assertEquals("python",catalog.language);val binding=org.json.JSONObject(catalog.sourceContext!!);assertEquals(debugHash(project.text),binding.getJSONObject("source_hashes").getString("project.md"));val entry=catalog.entries.single{it.id=="workspace.embedded"}
    model.prepare(workspace,entry);val expanded="#$path|old|old|OLD";assertEquals(model.notice,expanded,model.reviewed?.draft?.primary?.text);assertEquals(source,buffer.text);assertTrue(model.notice,model.apply(workspace));val after=source.replaceRange(from,from+3,expanded);assertEquals(after,buffer.text)
    val selected=state.selection;assertTrue(buffer.snippets.applyInput(buffer.text,selected.min,selected.max,"ßa"));val typed=source.replaceRange(from,from+3,"#$path|ßa|ßa|SSA");assertEquals(typed,buffer.text)
    command("editor.undo").run();assertEquals(after,buffer.text);command("editor.undo").run();assertEquals(source,buffer.text);command("editor.redo").run();assertEquals(after,buffer.text);command("editor.redo").run();assertEquals(typed,buffer.text)
    assertSame(state,buffer.state);assertSame(buffer,workspace.currentBuffer);assertEquals(scroll,buffer.sourceScroll.value);assertEquals(saved,root.resolve(path).readText());last=buffer
   }
   val buffer=last!!;val source=buffer.text;buffer.snippets.finish();val from=source.indexOf("ßa");buffer.selectSourceRange(from,from+2,source);workspace.showTemplates=true;val model=workspace.sourceTemplates;model.refresh(workspace);model.prepare(workspace,model.catalog!!.entries.single{it.id=="workspace.embedded"});assertNotNull(model.notice,model.reviewed)
   val original=project.text;replace(project,original+"\n");replace(project,original);assertFalse(model.apply(workspace));assertEquals(source,buffer.text)
   replace(project,original.replace("</h:doc>","<h:file path=\"mixed.js\"><h:paste select=\"#body\"/></h:file></h:doc>"));workspace.showTemplates=true;model.refresh(workspace);assertEquals(model.notice,false,model.catalog?.available);assertTrue(model.catalog?.entries?.isEmpty()==true);assertEquals(source,buffer.text)
   buffer.file=buffer.file.copy(generatedBy="fixture.md");model.refresh(workspace);assertNull(model.catalog);assertEquals(source,buffer.text);assertEquals(savedProject,root.resolve("project.md").readText());for(path in listOf("saved.py","draft.py","second.py","mixed.js"))assertFalse(root.resolve(path).exists())
  }finally{workspace.sourceTemplates.close();workspace.showTemplates=false;workspace.buffers.values.forEach{it.language?.close()}}
 }
}
