<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Review genuine extract and generation edits beside retained code

For maintainers verifying the TypeScript edits-only bridge through the simple editors.

Use an isolated installed TypeScript Language Server6.1.0 and TypeScript6.0.3.
The provider supplies extraction and accessor-generation actions; the engine asks
for their edits without executing the application command. Each host opens the
existing action menu, selects the original action, reviews and applies a draft,
then exercises coordinated Undo/Redo. An earlier unsaved edit, editor identity,
selection and geometry or native state must survive. Saved source stays unchanged.
Compile the resulting drafts separately with the same installed TypeScript.

These are actual provider and host paths, not a claim about every provider or
native physical input/accessibility. The driver checks the exact native test ran
without a skip. Install the two pinned tools only under `target/refactor-provider`.

## Keep the fixture and compiler outside the user's source

<ui:copy id="provider-refactor-host-driver">
#!/usr/bin/env python3
"""Genuine edits-only TypeScript refactoring through one simple host."""
import argparse,importlib.util,json,os,re,socket,subprocess,tempfile,time,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
CASES=[
 {'path':'extract.ts','source':'function greet(name: string) { return "Hello " + name; }\nconsole.log(greet("Ada"));\n','selection':'"Hello " + name','title':"Extract to inner function in function 'greet'",'expected':'function newFunction'},
 {'path':'accessor.ts','source':'class Person { name = "Ada"; }\nconst person = new Person();\nconsole.log(person.name);\n','selection':'name','title':"Generate 'get' and 'set' accessors",'expected':'get name()'},
]
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['browser','apple','android']);parser.add_argument('--skip-build',action='store_true');args=parser.parse_args()
 tools=ROOT/'target/refactor-provider/node_modules';server=tools/'typescript-language-server/lib/cli.mjs';compiler=tools/'typescript/lib/tsc.js'
 for package,version in [('typescript-language-server','6.1.0'),('typescript','6.0.3')]:assert json.loads((tools/package/'package.json').read_text())['version']==version
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-refactor-host-') as temporary:
  folder=Path(temporary).resolve();root=folder/'workspace';root.mkdir();artifacts=folder/'drafts';artifacts.mkdir();processes=[];driver=None
  for case in CASES:(root/case['path']).write_text(case['source'])
  (root/'.hickory-refactor-fixture').write_text('owned-refactor-v1\n');(root/'.hickory-refactor-cases.json').write_text(json.dumps(CASES));(root/'package.json').write_text('{"type":"module"}');(root/'tsconfig.json').write_text('{"compilerOptions":{"strict":true,"target":"ES2022","ignoreDeprecations":"6.0"}}')
  (root/'node_modules').mkdir();(root/'node_modules/typescript').symlink_to(tools/'typescript',target_is_directory=True)
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'typescript':{'command':['node',str(server),'--stdio']}}}))
  env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')}
  with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
  engine=fixture.start([str(ROOT/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window',f'http://127.0.0.1:{port}'],env=env);processes.append(engine)
  try:
   origin=fixture.origin(engine)
   if args.host=='browser':
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html')
    for case in CASES:
     path=case['path'];draft=case['source']+'// retained dirty\n';start=draft.index(case['selection']);driver.act('open',path=path);driver.act('remember-editor',path=path);driver.act('construction-set-source',path=path,source=draft);position=driver.act('editor-position',path=path,anchor=start,head=start+len(case['selection']));size=driver.act('editor-size',path=path);driver.act('change-open',command='Refactor and quick fixes');deadline=time.monotonic()+30
     while case['title'] not in (reading:=driver.act('change-prepare',command='Prepare provider actions')):
      assert time.monotonic()<deadline,reading;time.sleep(.05)
     driver.act('action-choose',title=case['title']);assert driver.act('construction-source',path=path)==draft;assert driver.act('editor-position',path=path)==position;driver.act('change-apply');driver.act('change-close');changed=driver.act('construction-source',path=path);assert case['expected'] in changed and '// retained dirty' in changed,changed;(artifacts/path).write_text(changed)
     driver.act('change-open',command='Undo workspace change');driver.act('change-close');assert driver.act('construction-source',path=path)==draft;driver.act('change-open',command='Redo workspace change');driver.act('change-close');assert driver.act('construction-source',path=path)==changed;assert driver.act('editor-is-retained',path=path) and driver.act('editor-size',path=path)==size
   else:
    env.update(HICKORY_REFACTOR_ORIGIN=origin,HICKORY_REFACTOR_WORKSPACE=str(root),HICKORY_REFACTOR_ARTIFACTS=str(artifacts));name='testGenuineExtractAndGenerationKeepDirtyEditorAndHistory' if args.host=='apple' else 'genuineExtractAndGenerationKeepDirtyEditorAndHistory';report=folder/'native.xml'
    if args.host=='apple':command=['swift','test','--jobs','2','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','ProviderRefactorTests.'+name+'$','--xunit-output',str(report)]+(['--skip-build'] if args.skip_build else [])
    else:report=ROOT/'apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.ProviderRefactorTest.xml';command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.ProviderRefactorTest.'+name,'--rerun']
    report.unlink(missing_ok=True);log=ROOT/'target'/f'provider-refactor-{args.host}-actual.log'
    with log.open('w') as output:result=subprocess.run(command,cwd=ROOT,env=env,stdout=output,stderr=subprocess.STDOUT,timeout=600)
    assert result.returncode==0,str(log)
    if report.exists():
     rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(key) is not None for key in ['failure','error','skipped']),str(report)
    else:assert args.host=='apple' and 'Executed 1 test, with 0 failures' in log.read_text() and re.search(re.escape(name)+r'.*passed',log.read_text()) and 'Test skipped' not in log.read_text()
   for case in CASES:
    assert (root/case['path']).read_text()==case['source'];assert case['expected'] in (artifacts/case['path']).read_text()
   result=subprocess.run(['node',str(compiler),'--noEmit','--strict','--target','ES2022','--ignoreConfig',*[str(artifacts/case['path']) for case in CASES]],capture_output=True,text=True,timeout=60);assert result.returncode==0,result.stdout+result.stderr
   assert not (root/'.hick-cache').exists();print('PASS '+args.host+': genuine extract and accessor generation, edits-only preview, retained dirty editor/history, unchanged saved sources and TypeScript-checked resulting drafts; physical input/accessibility remains unverified.')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-provider-refactor-host.py"><ui:paste select="#provider-refactor-host-driver"/></ui:file>

## Exercise the mounted AppKit editor through the existing action menu

<ui:copy id="provider-refactor-apple-actual">
import XCTest
import SwiftUI
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class ProviderRefactorTests:XCTestCase {
 #if os(macOS)
 func testGenuineExtractAndGenerationKeepDirtyEditorAndHistory()async throws {
  let env=ProcessInfo.processInfo.environment
  guard let origin=env["HICKORY_REFACTOR_ORIGIN"],let root=env["HICKORY_REFACTOR_WORKSPACE"],let artifacts=env["HICKORY_REFACTOR_ARTIFACTS"] else{throw XCTSkip("Use the owned provider-refactor host fixture.")}
  let directory=URL(fileURLWithPath:root);guard try String(contentsOf:directory.appendingPathComponent(".hickory-refactor-fixture"),encoding:.utf8)=="owned-refactor-v1\n" else{throw EngineError(message:"Owned refactor fixture required.")}
  let cases=try XCTUnwrap(JSONSerialization.jsonObject(with:Data(contentsOf:directory.appendingPathComponent(".hickory-refactor-cases.json"))) as? [[String:String]])
  _=NSApplication.shared;NSApp.setActivationPolicy(.regular);let engine=try Engine(address:origin),w=Workspace();w.engine=engine;let listing=try await engine.files();w.rootFiles=listing.tree;w.sourceRootAddress=listing.root_path;w.sourceWorkspaceID=listing.workspace_id
  let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:980,height:700),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil)
  defer{for b in w.buffers.values{b.language?.close()};w.sourceTestIndex?.stop();window.orderOut(nil);window.contentView=nil}
  func settle(_ message:String,_ test:()->Bool)async throws{for _ in 0..<600{if test(){return};try await Task.sleep(for:.milliseconds(20))};throw EngineError(message:message)}
  func native(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{native($0)}.first}
  for item in cases {
   let path=try XCTUnwrap(item["path"]),source=try XCTUnwrap(item["source"]),selected=try XCTUnwrap(item["selection"]),title=try XCTUnwrap(item["title"]),expected=try XCTUnwrap(item["expected"]);let file=try XCTUnwrap(w.rootFiles.first{$0.path==path}),b=try await w.loadBuffer(file);w.showSource(path);host.layoutSubtreeIfNeeded();try await settle("Code editor did not mount"){b.sourceEditorState?()?.attached==true};let editor=try XCTUnwrap(native(host)),identity=try XCTUnwrap(b.sourceEditorState?()).identity,scroll=try XCTUnwrap(editor.enclosingScrollView),geometry=scroll.convert(scroll.bounds,to:host)
   editor.setSelectedRange(NSRange(location:editor.string.utf16.count,length:0));editor.insertText("// retained dirty\n",replacementRange:editor.selectedRange());let draft=b.text;XCTAssertEqual(draft,source+"// retained dirty\n");editor.setSelectedRange((draft as NSString).range(of:selected));let selection=b.selection
   try XCTUnwrap(w.commands().first{$0.id=="symbol.actions"}).run();for _ in 0..<200{await w.changes.prepare(w);if w.changes.choices.contains(where:{$0["title"] as? String==title}){break};try await Task.sleep(for:.milliseconds(50))};let index=try XCTUnwrap(w.changes.choices.firstIndex{$0["title"] as? String==title});await w.changes.choose(index,workspace:w);XCTAssertNotNil(w.changes.plan,w.changes.notice);XCTAssertEqual(b.text,draft);XCTAssertEqual(b.selection,selection);w.changes.apply(w);let changed=b.text;XCTAssertTrue(changed.contains(expected),w.changes.notice);XCTAssertTrue(changed.contains("// retained dirty"));try changed.write(to:URL(fileURLWithPath:artifacts).appendingPathComponent(path),atomically:true,encoding:.utf8)
   w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,draft);w.changes.open("redo",workspace:w);XCTAssertEqual(b.text,changed);w.changes.close(w);XCTAssertEqual(b.sourceEditorState?()?.identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(path),encoding:.utf8),source)
  }
 }
 #endif
}
</ui:copy>
<ui:file path="apps/apple-simple/Tests/ProviderRefactorTests.swift"><ui:paste select="#provider-refactor-apple-actual"/></ui:file>

## Exercise the Compose state and integrated production controller

<ui:copy id="provider-refactor-android-actual">
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
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/ProviderRefactorTest.kt"><ui:paste select="#provider-refactor-android-actual"/></ui:file>
</ui:doc>
