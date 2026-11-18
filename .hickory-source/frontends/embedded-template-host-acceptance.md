<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Apply templates in authored embedded source

For maintainers connecting captured compiler language contexts to the source editor. A filename does not identify a reused fragment's language. This witness opens a Markdown code fence and an external fragment pasted into two Python outputs, retains unsaved original graph and source changes, and drives the real menu, Preview and Apply paths. The language-specific comment variable and uppercase mirror must come from the same captured context; the filename remains the authored Markdown filename.

The script checks source bytes, linked typing, Undo/Redo, editor identity and geometry. Its opt-in `--batch` path establishes two native selections, then uses the same visible catalog, Preview and Apply controls. It requires distinct cursor variables under one clock and seed, nested continuation of both original mirror groups and refusal after only the secondary selection changes; it does not apply through the selection adapter. The shared service and controller model checks separately cover mixed-language ambiguity, stale contributor leases, readonly contributors, missing inputs and newly generated ownership. Those checks do not establish physical input or accessibility acceptance. Native embedded-context paths and iOS runtime remain separate acceptance until executed.

<ui:copy id="embedded-template-browser-runner">
"""Drive embedded templates through the real source editor and shared compiler."""
import importlib.util,json,os,socket,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('simple_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
NS='http://www.hickorydocs.com/1.0'
def doc(body):return '<h:doc xmlns:h="'+NS+'">'+body+'</h:doc>'
def project(target='saved.py',mixed=False):
 fence='<h:paste select="#fence_tick"/>'*3
 body='<h:project sources="part.md"/><h:copy id="fence_tick">`</h:copy><h:file path="'+target+'"><h:paste select="#body"/></h:file><h:file path="second.py"><h:paste select="#body"/></h:file><h:file path="guide.md">'+fence+'python\n<h:paste select="#body"/>\n'+fence+'\n</h:file>'
 if mixed:body+='<h:file path="mixed.js"><h:paste select="#body"/></h:file>'
 return doc(body)
def exercise(driver,root,path,source):
 driver.act('open',path=path);driver.act('remember-editor',path=path);size=driver.act('editor-size',path=path);position=source.index('old');driver.act('editor-position',path=path,anchor=position,head=position+3)
 catalog=driver.act('templates-open',path=path);assert 'Embedded context' in catalog,catalog
 preview=driver.act('templates-preview',label='Embedded context');expanded='#'+path+'|old|old|OLD';assert expanded in preview,preview;assert driver.act('construction-source',path=path)==source
 driver.act('templates-apply',path=path);after=source[:position]+expanded+source[position+3:];assert driver.act('construction-source',path=path)==after
 driver.act('type-source-selection',path=path,text='ßa');typed=source[:position]+'#'+path+'|ßa|ßa|SSA'+source[position+3:];assert driver.act('construction-source',path=path)==typed
 driver.act('undo',path=path);assert driver.act('construction-source',path=path)==after
 driver.act('undo',path=path);assert driver.act('construction-source',path=path)==source
 driver.act('redo',path=path);assert driver.act('construction-source',path=path)==after
 driver.act('redo',path=path);assert driver.act('construction-source',path=path)==typed
 assert driver.act('editor-is-retained',path=path);assert driver.act('editor-size',path=path)==size
 return typed

def exercise_batch(driver,root,path,source):
 import re
 driver.act('open',path=path);driver.act('remember-editor',path=path);size=driver.act('editor-size',path=path)
 first=source.index('old');second=source.index('two',first+3);ranges=[{'from':first,'to':first+3},{'from':second,'to':second+3}]
 assert driver.act('editor-selections',path=path,ranges=ranges,primary=0)=={'ranges':ranges,'primary':0}
 catalog=driver.act('templates-open',path=path);assert 'Batch context' in catalog,catalog
 preview=driver.act('templates-preview',label='Batch context');assert '2 captured selections' in preview,preview;assert driver.act('construction-source',path=path)==source
 driver.act('templates-apply',path=path);after=driver.act('construction-source',path=path)
 selected=after[first:len(after)-(len(source)-second-3)]
 match=re.fullmatch(r'#0:1:(\d{4}):([0-9a-f]{6}):old\|old\|OLD #1:2:\1:\2:two\|two\|TWO',selected);assert match,selected
 year,seed=match.groups();prefix=lambda i:f'#{i}:{i+1}:{year}:{seed}:'
 def complete(value,upper):return source[:first]+prefix(0)+value+'|'+value+'|'+upper+' '+prefix(1)+value+'|'+value+'|'+upper+source[second+3:]
 driver.act('type-source-selection',path=path,text='ßa');typed=complete('ßa','SSA');assert driver.act('construction-source',path=path)==typed
 # Typing leaves collapsed carets. The existing field commands explicitly select both current fields before replacing them.
 driver.act('template-next');driver.act('template-previous');assert [row['to']-row['from'] for row in driver.act('editor-selections',path=path)['ranges']]==[2,2]
 # A second invocation must preserve the first group's mirrors and computed fields.
 driver.act('templates-open',path=path);nested_preview=driver.act('templates-preview',label='Batch nested');assert '2 captured selections' in nested_preview,nested_preview
 driver.act('templates-apply',path=path);nested=complete('kid-kid','KID-KID');assert driver.act('construction-source',path=path)==nested,driver.act('construction-source',path=path)
 driver.act('type-source-selection',path=path,text='ßa');nested_typed=complete('ßa-ßa','SSA-SSA');assert driver.act('construction-source',path=path)==nested_typed
 for expected in [nested,typed,after,source]:driver.act('undo',path=path);assert driver.act('construction-source',path=path)==expected
 for expected in [after,typed,nested,nested_typed]:driver.act('redo',path=path);assert driver.act('construction-source',path=path)==expected
 for _ in range(4):driver.act('undo',path=path)
 assert driver.act('construction-source',path=path)==source
 # Moving only a secondary selection retires a held Apply even when bytes and primary stay equal.
 driver.act('editor-selections',path=path,ranges=ranges,primary=0);driver.act('templates-open',path=path);driver.act('templates-preview',label='Batch context');driver.act('templates-hold-apply')
 moved=[ranges[0],{'from':second+3,'to':second+3}];driver.act('editor-selections',path=path,ranges=moved,primary=0);driver.act('templates-release-apply')
 assert driver.act('construction-source',path=path)==source;assert not driver.act('templates-apply-enabled');driver.act('templates-close')
 assert driver.act('editor-is-retained',path=path);assert driver.act('editor-size',path=path)==size
 return source

def main():
 batch='--batch' in sys.argv
 with tempfile.TemporaryDirectory(prefix='embedded-template-browser-',dir=ROOT/'target') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();original=project();part=doc('<h:copy id="body">saved</h:copy>');code='Narrative\n\n```python\n'+('old two' if batch else 'old')+'\n```\n';(root/'project.md').write_text(original);(root/'part.md').write_text(part);(root/'notes.md').write_text(code)
  (root/'.hickory').mkdir();(root/'.hickory/templates.json').write_text(json.dumps({'version':1,'templates':[{'id':'embedded.context','label':'Embedded context','description':'Compiler-bound comments and linked Unicode transforms','language':'python','syntax':'${LINE_COMMENT}${TM_FILENAME}|${1:${TM_SELECTED_TEXT}}|$1|${1/(.*)/${1:/upcase}/}$0'}, {'id':'batch.context','label':'Batch context','description':'One captured graph and clock for the complete caret roster','language':'python','syntax':'${LINE_COMMENT}${CURSOR_INDEX}:${CURSOR_NUMBER}:${CURRENT_YEAR}:${RANDOM_HEX}:${1:${TM_SELECTED_TEXT}}|$1|${1/(.*)/${1:/upcase}/}$0'}, {'id':'batch.nested','label':'Batch nested','description':'Continue each original outer mirror group','language':'python','syntax':'${1:kid}-$1$0'}]}))
  processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};ui=f'http://127.0.0.1:{port}';engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',ui],env=env);processes.append(engine);backend=fixture.origin(engine)
   vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':backend});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html')
   if batch:
    exercise_batch(driver,root,'notes.md',code)
    driver.act('open',path='project.md');driver.act('construction-set-source',path='project.md',source=project('draft.py'))
    driver.act('open',path='part.md');dirty_part=doc('<h:copy id="body">old two</h:copy>');driver.act('construction-set-source',path='part.md',source=dirty_part);exercise_batch(driver,root,'part.md',dirty_part)
    for path,source in [('project.md',original),('part.md',part),('notes.md',code)]:assert (root/path).read_bytes()==source.encode(),path
    assert all(not(root/name).exists() for name in ['saved.py','draft.py','second.py','mixed.js','guide.md'])
    print('PASS: real batch code-fence and dirty reused-fragment catalogs, complete visible Preview/Apply, distinct caret indices/values under one clock/seed/graph, nested Unicode computed mirrors, atomic group UndoRedo, stale secondary selection refusal, retained editor/geometry and unchanged source disks')
    return
   exercise(driver,root,'notes.md',code)
   driver.act('open',path='project.md');dirty=project('draft.py');driver.act('construction-set-source',path='project.md',source=dirty);assert driver.act('construction-source',path='project.md')==dirty
   driver.act('open',path='part.md');dirty_part=doc('<h:copy id="body">old</h:copy>');driver.act('construction-set-source',path='part.md',source=dirty_part);typed=exercise(driver,root,'part.md',dirty_part)
   # A physically changed entry must not renew an already reviewed proof.
   selected=typed.index('ßa');driver.act('editor-position',path='part.md',anchor=selected,head=selected+2);driver.act('templates-open',path='part.md');driver.act('templates-preview',label='Embedded context');driver.act('templates-hold-apply')
   try:
    (root/'project.md').write_text(original+'\n');refused=driver.act('templates-release-apply');assert refused['status']>=400,refused;assert driver.act('construction-source',path='part.md')==typed
   finally:
    (root/'project.md').write_text(original)
   driver.act('templates-close')
   # The original retained graph, rather than filename or saved output, controls reuse.
   driver.act('open',path='project.md');driver.act('construction-set-source',path='project.md',source=project('draft.py',True));driver.act('open',path='part.md');driver.act('editor-position',path='part.md',anchor=selected,head=selected+2)
   unavailable=driver.act('templates-open',path='part.md');assert 'Preview Embedded context' not in unavailable,unavailable;assert any(word in unavailable.lower() for word in ['language','ambiguous','multiple','disagree']),unavailable;assert driver.act('construction-source',path='part.md')==typed;driver.act('templates-close')
   driver.act('open',path='project.md');claimed=doc('<h:project sources="part.md"/><h:file path="part.md">new generated ownership</h:file>');driver.act('construction-set-source',path='project.md',source=claimed);driver.act('open',path='part.md');driver.act('editor-position',path='part.md',anchor=selected,head=selected+2)
   command=driver.act('templates-command-state')
   if command['disabled']:assert command['reason'],command
   else:
    refused=driver.act('templates-open',path='part.md');assert 'Preview Embedded context' not in refused,refused;assert any(word in refused.lower() for word in ['generat','ownership','output']),refused;driver.act('templates-close')
   assert driver.act('construction-source',path='part.md')==typed
   for path,source in [('project.md',original),('part.md',part),('notes.md',code)]:assert (root/path).read_bytes()==source.encode(),path
   assert all(not(root/name).exists() for name in ['saved.py','draft.py','second.py','mixed.js','guide.md'])
   print('PASS: real code-fence and twice-reused external fragment catalog/Preview/Apply, dirty original compiler graph, Python comments/authored filenames, Unicode transform mirrors, atomic UndoRedo, retained editors/geometry, old entry proof/ambiguous reuse/new generated location refusal, unchanged source disks and no generated outputs')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-embedded-templates-browser.py"><ui:paste select="#embedded-template-browser-runner"/></ui:file>

## Exercise the same captured graph in native controllers

The dedicated opt-in cases use the owned native fixture and exact one-method filters. The Apple case mounts the actual native editor and invokes the shared template command; Android invokes the actual command/controller and TextFieldState. Both keep a dirty canonical graph and dirty reused fragment, require Python-specific comments, and recheck linked transforms and ordinary Undo/Redo before refusing stale, mixed-language and readonly proposals. These cases are maintained acceptance to execute, not evidence of an executed physical input or iOS runtime check.

<ui:copy id="embedded-template-apple-witness">
import XCTest
#if os(macOS)
import AppKit
import Darwin
#endif
@testable import HickorySimple
@MainActor final class EmbeddedTemplateTests:XCTestCase {
 #if os(macOS)
 func testLiveEmbeddedTemplateCatalogAndHistory()async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],let directory=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"] else{throw XCTSkip("Use the owned native-template --embedded fixture.")}
  let root=URL(fileURLWithPath:directory);XCTAssertEqual(try String(contentsOf:root.appendingPathComponent(".hickory-native-template-fixture"),encoding:.utf8),"owned-native-templates-v1\n")
  let engine=try Engine(address:origin),tree=try await engine.files(),space=Workspace();space.engine=engine;space.rootFiles=tree.tree;space.sourceRootAddress=tree.root_path;space.sourceWorkspaceID=tree.workspace_id
  func find(_ path:String,_ nodes:[FileNode])->FileNode?{for node in nodes{if exactUTF8(node.path,path){return node};if let child=find(path,node.children ?? []){return child}};return nil}
  let project=try await space.loadBuffer(XCTUnwrap(find("project.md",tree.tree)));let savedProject=project.text;project.text=savedProject.replacingOccurrences(of:"saved.py",with:"draft.py")
  _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);NSApp.activate(ignoringOtherApps:true)
  var lastBuffer:DocumentBuffer?;var lastEditor:MacEditor?;var lastWindow:NSWindow?
  defer{space.sourceTemplates.close();for buffer in space.buffers.values{buffer.language?.close()};lastWindow?.orderOut(nil)}
  for path in ["notes.md","part.md"] {
   let buffer=try await space.loadBuffer(XCTUnwrap(find(path,tree.tree)));let saved=buffer.text;if path=="part.md"{buffer.text=saved.replacingOccurrences(of:"saved",with:"old")};let source=buffer.text
   space.layout.activate(path,in:"document-group");let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:480),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;window.makeKeyAndOrderFront(nil);lastWindow?.orderOut(nil);lastWindow=window;lastEditor=editor;lastBuffer=buffer
   let native=editor.text,manager=try XCTUnwrap(native.undoManager),range=(source as NSString).range(of:"old");XCTAssertNotEqual(range.location,NSNotFound);native.setSelectedRange(range);buffer.selection=range
   let identity=try XCTUnwrap(buffer.sourceEditorState?()).identity,geometry=editor.scroll.frame
   let command=try XCTUnwrap(space.commands().first{$0.id=="source.templates"});XCTAssertNil(command.reason);command.run();let model=space.sourceTemplates
   for _ in 0..<400{if model.catalog?.path==path && model.current() && !model.reading{break};try await Task.sleep(for:.milliseconds(25))}
   let catalog=try XCTUnwrap(model.catalog,model.notice),entry=try XCTUnwrap(catalog.entries.first{$0.id=="workspace.embedded"},model.notice);XCTAssertEqual(catalog.language,"python");XCTAssertEqual(catalog.source_context?.source_hashes["project.md"],nativeSourceHash(project.text));if path=="part.md"{XCTAssertTrue(catalog.source_context?.targets.contains("guide.md")==true)}
   await model.choose(entry);let expanded="#\(path)|old|old|OLD",preview=try XCTUnwrap(model.preview,model.notice);XCTAssertEqual(preview.snippet.text,expanded,model.notice);XCTAssertEqual(buffer.text,source)
   let accepted=await model.apply();XCTAssertTrue(accepted,model.notice);let after=(source as NSString).replacingCharacters(in:range,with:expanded);XCTAssertEqual(buffer.text,after)
   native.insertText("ßa",replacementRange:native.selectedRange());let typed=(source as NSString).replacingCharacters(in:range,with:"#\(path)|ßa|ßa|SSA");XCTAssertEqual(buffer.text,typed)
   manager.undo();XCTAssertEqual(buffer.text,after);manager.undo();XCTAssertEqual(buffer.text,source);manager.redo();XCTAssertEqual(buffer.text,after);manager.redo();XCTAssertEqual(buffer.text,typed)
   XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertTrue(editor.text===native);XCTAssertTrue(native.undoManager===manager);XCTAssertEqual(editor.scroll.frame,geometry);XCTAssertEqual(try String(contentsOf:root.appendingPathComponent(path),encoding:.utf8),saved)
  }
  let buffer=try XCTUnwrap(lastBuffer),editor=try XCTUnwrap(lastEditor),source=buffer.text;buffer.snippets.finish();let range=(source as NSString).range(of:"ßa");editor.text.setSelectedRange(range);buffer.selection=range
  await space.sourceTemplates.open(space);let entry=try XCTUnwrap(space.sourceTemplates.catalog?.entries.first{$0.id=="workspace.embedded"});await space.sourceTemplates.choose(entry);XCTAssertNotNil(space.sourceTemplates.preview,space.sourceTemplates.notice)
  let original=project.text;project.text=original+"\n";project.text=original;let applied=await space.sourceTemplates.apply();XCTAssertFalse(applied);XCTAssertEqual(buffer.text,source)
  project.text=original.replacingOccurrences(of:"</h:doc>",with:"<h:file path=\"mixed.js\"><h:paste select=\"#body\"/></h:file></h:doc>");await space.sourceTemplates.open(space);XCTAssertEqual(space.sourceTemplates.catalog?.available,false,space.sourceTemplates.notice);XCTAssertTrue(space.sourceTemplates.catalog?.entries.isEmpty==true);XCTAssertEqual(buffer.text,source)
  buffer.file.generated_by="fixture.md";await space.sourceTemplates.open(space);XCTAssertNil(space.sourceTemplates.catalog);XCTAssertEqual(buffer.text,source)
  XCTAssertEqual(try String(contentsOf:root.appendingPathComponent("project.md"),encoding:.utf8),savedProject);for path in ["saved.py","draft.py","second.py","mixed.js","guide.md"]{XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent(path).path))}
 }
 #endif
}
</ui:copy>
<ui:file path="apps/apple-simple/Tests/EmbeddedTemplateTests.swift"><ui:paste select="#embedded-template-apple-witness"/></ui:file>
<ui:copy id="embedded-template-android-witness">
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
    val catalog=model.catalog ?: error(model.notice);assertEquals("python",catalog.language);val binding=org.json.JSONObject(catalog.sourceContext!!);assertEquals(debugHash(project.text),binding.getJSONObject("source_hashes").getString("project.md"));if(path=="part.md"){val targets=binding.getJSONArray("targets");assertTrue((0 until targets.length()).any{targets.getString(it)=="guide.md"})};val entry=catalog.entries.single{it.id=="workspace.embedded"}
    model.prepare(workspace,entry);val expanded="#$path|old|old|OLD";assertEquals(model.notice,expanded,model.reviewed?.draft?.primary?.text);assertEquals(source,buffer.text);assertTrue(model.notice,model.apply(workspace));val after=source.replaceRange(from,from+3,expanded);assertEquals(after,buffer.text)
    val selected=state.selection;assertTrue(buffer.snippets.applyInput(buffer.text,selected.min,selected.max,"ßa"));val typed=source.replaceRange(from,from+3,"#$path|ßa|ßa|SSA");assertEquals(typed,buffer.text)
    command("editor.undo").run();assertEquals(after,buffer.text);command("editor.undo").run();assertEquals(source,buffer.text);command("editor.redo").run();assertEquals(after,buffer.text);command("editor.redo").run();assertEquals(typed,buffer.text)
    assertSame(state,buffer.state);assertSame(buffer,workspace.currentBuffer);assertEquals(scroll,buffer.sourceScroll.value);assertEquals(saved,root.resolve(path).readText());last=buffer
   }
   val buffer=last!!;val source=buffer.text;buffer.snippets.finish();val from=source.indexOf("ßa");buffer.selectSourceRange(from,from+2,source);workspace.showTemplates=true;val model=workspace.sourceTemplates;model.refresh(workspace);model.prepare(workspace,model.catalog!!.entries.single{it.id=="workspace.embedded"});assertNotNull(model.notice,model.reviewed)
   val original=project.text;replace(project,original+"\n");replace(project,original);assertFalse(model.apply(workspace));assertEquals(source,buffer.text)
   replace(project,original.replace("</h:doc>","<h:file path=\"mixed.js\"><h:paste select=\"#body\"/></h:file></h:doc>"));workspace.showTemplates=true;model.refresh(workspace);assertEquals(model.notice,false,model.catalog?.available);assertTrue(model.catalog?.entries?.isEmpty()==true);assertEquals(source,buffer.text)
   buffer.file=buffer.file.copy(generatedBy="fixture.md");model.refresh(workspace);assertNull(model.catalog);assertEquals(source,buffer.text);assertEquals(savedProject,root.resolve("project.md").readText());for(path in listOf("saved.py","draft.py","second.py","mixed.js","guide.md"))assertFalse(root.resolve(path).exists())
  }finally{workspace.sourceTemplates.close();workspace.showTemplates=false;workspace.buffers.values.forEach{it.language?.close()}}
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/EmbeddedTemplateTest.kt"><ui:paste select="#embedded-template-android-witness"/></ui:file>
</ui:doc>
