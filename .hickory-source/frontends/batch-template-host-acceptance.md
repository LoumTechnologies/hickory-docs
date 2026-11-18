<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Review and apply one complete template group

For maintainers joining compiler-backed template plans to the existing multiple-selection editor. These opt-in native witnesses use the same guarded service, catalog, Preview and explicit Apply actions as production. They require different original selected values and cursor numbers under one clock and seed, nested Unicode computed mirrors, a single denied clipboard attempt for the complete roster, refusal after only a secondary caret moves, retained editor/history/scroll and unchanged saved source bytes.

The fixture also retains a dirty canonical graph and reuses an original fragment in Python files and a generated Markdown code fence. Every replacement stays in its original authored source; no generated file is materialized. Apple mounts the actual editor and uses its input and undo manager. Android exercises the production controller and TextFieldState. Both exact cases passed on 2026-11-17 with zero skips or failures: mounted Mac 1.207 seconds and Android controller 0.856 seconds (suite 0.858). These receipts are not evidence of physical input, accessibility, device IME or iOS runtime behavior. Run exactly one through `scripts/test-native-templates.py <host> --batch --engine <current-built-engine>`; the shared runner retains an exact one-case producer receipt and refuses skips or zero executed cases.

The new program-3 mode retains all complete-group and returned-roster assertions, adds exact negotiated-list and computed-field program checks, and requires lookbehind/Unicode-property edits through both native adapters. Its receipt label is distinct from earlier batch evidence. Both new program-3 actual paths and ordinary compatibility replays passed on the activated snapshot.

<ui:copy id="batch-template-apple-witness">
import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class BatchTemplateTests:XCTestCase {
 #if os(macOS)
 func testLiveCompleteTemplateGroupAndHistory()async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],let directory=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"] else{throw XCTSkip("Use the owned native-template --batch fixture.")}
  let program3=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_PROGRAM3"]=="1"
  let root=URL(fileURLWithPath:directory);XCTAssertEqual(try String(contentsOf:root.appendingPathComponent(".hickory-native-template-fixture"),encoding:.utf8),"owned-native-templates-v1\n")
  let engine=try Engine(address:origin),tree=try await engine.files(),space=Workspace();space.engine=engine;space.rootFiles=tree.tree;space.sourceRootAddress=tree.root_path;space.sourceWorkspaceID=tree.workspace_id
  func find(_ path:String,_ nodes:[FileNode])->FileNode?{for node in nodes{if exactUTF8(node.path,path){return node};if let child=find(path,node.children ?? []){return child}};return nil}
  let project=try await space.loadBuffer(XCTUnwrap(find("project.md",tree.tree)));let savedProject=project.text;project.text=savedProject.replacingOccurrences(of:"saved.py",with:"draft.py")
  let model=space.sourceTemplates;model.captureVariables={SnippetVariableSnapshot(clock:SnippetVariableClock(unix_millis:1704067200000,utc_offset_minutes:0),random_hex:"0123456789abcdef0123456789abcdef")};var reads=0;model.clipboardReader={reads+=1;return nil}
  _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);NSApp.activate(ignoringOtherApps:true)
  var window:NSWindow?;defer{model.close();window?.orderOut(nil);for buffer in space.buffers.values{buffer.language?.close()}}
  for path in ["notes.md","part.md"] {
   let buffer=try await space.loadBuffer(XCTUnwrap(find(path,tree.tree))),saved=buffer.text
   buffer.text=saved.replacingOccurrences(of:path=="part.md" ? "saved":"old",with:"old two");let source=buffer.text
   space.layout.activate(path,in:"document-group");let editor=MacEditor(buffer),next=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:480),styleMask:[.titled],backing:.buffered,defer:false);next.contentView=editor.scroll;next.makeKeyAndOrderFront(nil);window?.orderOut(nil);window=next
   let native=editor.text,manager=try XCTUnwrap(native.undoManager),identity=try XCTUnwrap(buffer.sourceEditorState?()).identity,geometry=editor.scroll.frame
   let first=(source as NSString).range(of:"old"),second=(source as NSString).range(of:"two"),ranges=[first,second];XCTAssertTrue(buffer.selectMultiple(ranges,source:source))
   func command(_ id:String)throws->WorkspaceCommand{try XCTUnwrap(space.commands().first{$0.id==id})}
   func open()async throws {let action=try command("source.templates");XCTAssertNil(action.reason);action.run();for _ in 0..<400{if model.catalog?.path==path && model.current() && !model.reading{break};try await Task.sleep(for:.milliseconds(25))};XCTAssertNotNil(model.catalog,model.notice)}
   func choose(_ id:String)async throws {let entry=try XCTUnwrap(model.catalog?.entries.first{$0.id==id},model.notice);await model.choose(entry)}
   func complete(_ value:String,_ uppercase:String)->String {
    let a="#0:1:2024:012345:"+value+"|"+value+"|"+uppercase,b="#1:2:2024:012345:"+value+"|"+value+"|"+uppercase
    let original=source as NSString;return original.substring(to:first.location)+a+" "+b+original.substring(from:NSMaxRange(second))
   }
   try await open();let catalog=try XCTUnwrap(model.catalog,model.notice);XCTAssertEqual(catalog.carets?.selections,ranges.map(SourceTemplateRange.init));XCTAssertEqual(catalog.language,"python");XCTAssertEqual(catalog.transform_program_versions,[2,3]);XCTAssertEqual(catalog.source_context?.source_hashes["project.md"],nativeSourceHash(project.text))
   try await choose("workspace.batch-context");let originalPreview=try XCTUnwrap(model.batchPreviewSource,model.notice);let proposed=try XCTUnwrap(model.batchPreview,model.notice);var programs=0;for row in proposed.expansions ?? []{for field in row.snippet.fields{if let transform=field.transform{guard case .object(let raw)=transform,case .integer(let version)?=raw["version"] else{XCTFail("Malformed returned template program");continue};XCTAssertEqual(version,3);programs+=1}}};XCTAssertEqual(programs,2);XCTAssertEqual(buffer.text,source);let accepted=await model.apply();XCTAssertTrue(accepted,model.notice);XCTAssertEqual(buffer.text,originalPreview)
   XCTAssertTrue(buffer.text.contains(program3 ? "#0:1:2024:012345:old|old|oXX #1:2:2024:012345:two|two|tXX":"#0:1:2024:012345:old|old|OLD #1:2:2024:012345:two|two|TWO"));XCTAssertNotNil(buffer.snippets.parallel)
   native.insertText("ßa",replacementRange:native.selectedRange());let typed=complete("ßa",program3 ? "ßX":"SSA");XCTAssertEqual(buffer.text,typed)
   let beforeMove=buffer.multiRanges;try command("editor.snippetNext").run();XCTAssertNotEqual(buffer.multiRanges,beforeMove);try command("editor.snippetPrevious").run();XCTAssertEqual(buffer.multiRanges.map(\.length),[2,2])
   try await open();try await choose("workspace.batch-nested");XCTAssertEqual(model.batchPreview?.carets.continuing,true);let nestedAccepted=await model.apply();XCTAssertTrue(nestedAccepted,model.notice);let nested=complete("kid-kid",program3 ? "kXX-kXX":"KID-KID");XCTAssertEqual(buffer.text,nested)
   native.insertText("ßa",replacementRange:native.selectedRange());let nestedTyped=complete("ßa-ßa",program3 ? "ßX-ßX":"SSA-SSA");XCTAssertEqual(buffer.text,nestedTyped)
   for expected in [nested,typed,originalPreview,source]{manager.undo();XCTAssertEqual(buffer.text,expected)}
   for expected in [originalPreview,typed,nested,nestedTyped]{manager.redo();XCTAssertEqual(buffer.text,expected)}
   for _ in 0..<4{manager.undo()};XCTAssertEqual(buffer.text,source)
   XCTAssertTrue(buffer.selectMultiple(ranges,source:source));let beforeClipboard=reads;try await open();try await choose("workspace.clipboard");XCTAssertEqual(reads,beforeClipboard+1);XCTAssertEqual(model.batchPreview?.expansions?.map{$0.snippet.text},["fallback","fallback"]);let deniedAccepted=await model.apply();XCTAssertTrue(deniedAccepted,model.notice);XCTAssertEqual(reads,beforeClipboard+1);manager.undo();XCTAssertEqual(buffer.text,source)
   XCTAssertTrue(buffer.selectMultiple(ranges,source:source));try await open();try await choose("workspace.batch-context");XCTAssertNotNil(model.batchPreview,model.notice);XCTAssertTrue(buffer.selectMultiple([first,NSRange(location:NSMaxRange(second),length:0)],source:source));let stale=await model.apply();XCTAssertFalse(stale);XCTAssertEqual(buffer.text,source)
   XCTAssertTrue(buffer.selectMultiple(ranges,source:source));try await open();try await choose("workspace.batch-context");XCTAssertNotNil(model.batchPreview,model.notice)
   let heldSelectionIdentity=buffer.selectionIdentity,heldSourceIdentity=buffer.sourceEditIdentity
   XCTAssertTrue(buffer.selectMultiple([first,NSRange(location:NSMaxRange(second),length:0)],source:source));XCTAssertTrue(buffer.selectMultiple(ranges,source:source));XCTAssertEqual(buffer.selection,first);XCTAssertNotEqual(buffer.selectionIdentity,heldSelectionIdentity);XCTAssertEqual(buffer.sourceEditIdentity,heldSourceIdentity)
   let returned=await model.apply();XCTAssertFalse(returned);XCTAssertEqual(buffer.text,source)

   XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertTrue(native.undoManager===manager);XCTAssertEqual(editor.scroll.frame,geometry);XCTAssertEqual(try String(contentsOf:root.appendingPathComponent(path),encoding:.utf8),saved)
  }
  XCTAssertEqual(try String(contentsOf:root.appendingPathComponent("project.md"),encoding:.utf8),savedProject);for name in ["saved.py","draft.py","second.py","guide.md"]{XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent(name).path))}
 }
 #endif
}
</ui:copy>
<ui:file path="apps/apple-simple/Tests/BatchTemplateTests.swift"><ui:paste select="#batch-template-apple-witness"/></ui:file>

The follow-up returned-roster check moves only the secondary selection away and back before Apply, retaining equal source bytes and primary selection. Swift must advance its selection identity; Android must advance its separate multi-selection revision without pretending the source text changed. The existing captured lease then refuses the old preview. Both final exact native replays passed this return-to-equal case with zero skips or failures.

The Android witness uses the same original caret order and complete expected source. Its controller Apply must use the shared parallel planner and one source-history transaction; direct test text changes only prepare the deliberately unsaved original fixture.

<ui:copy id="batch-template-android-witness">
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
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/BatchTemplateTest.kt"><ui:paste select="#batch-template-android-witness"/></ui:file>

## Retained focused evidence

The generated browser policy/view/entry-reader selection passed 34 checks, and the full generated web typecheck passed. The separate rendered `scripts/test-embedded-templates-browser.py --batch` then passed through the real visible catalog/Preview/Apply paths for both code fence and dirty reused fragment, including nested Unicode mirrors, field menu navigation, complete group Undo/Redo and a held Apply refused after only the secondary selection moved. Its final log is `target/template-batch-browser-actual.log`. Earlier bounded fixture failures are retained separately; the final path uses the attached editor selection constructor and explicitly selects fields before nested replacement.

Swift focused template checks ran 11 passes plus one expected opt-in skip; Android focused template XML ran 12 passes plus one expected opt-in skip. The exact actual receipts are `target/native-template-smoke/apple-batch-last.json` (truthful XCTest console producer/log hash) and `target/native-template-smoke/android-batch-last.xml` (producer XML), each exactly one case with zero skips/failures. The final native receipts also cover return-to-equal secondary-roster refusal without changing source identity/revision; the separate multi-selection stamp is captured by the Android template adapter.


## Executed negotiated program modes

On 2026-11-18 both `--program3` exact cases passed with zero skips or failures: mounted Mac 3.223 seconds and Android controller 1.250 seconds (suite 1.253). They assert the captured list and returned program version, then require actual lookbehind/Unicode-property edits, complete nested history, denied clipboard reuse and returned-roster refusal. Ordinary `--batch` compatibility replays also passed under the new default policy: Mac 1.019 seconds and Android case 0.922 seconds (suite 0.924).

Distinct `apple-template3-last.json` and `android-template3-last.xml` receipts and matching logs live in `target/native-template-smoke/`; the ordinary replays were copied to `apple-template3-compatibility.json` and `android-template3-compatibility.xml`. The Mac receipts truthfully identify XCTest console and its log hash. Android producer XML shows exactly one executed case per mode, zero skips/errors/failures. The earlier Android failure is retained separately; structural opcode value equality fixed the production refusal without removing freshness checks. Physical input, accessibility, device IME and iOS runtime acceptance remain unverified.

</ui:doc>
