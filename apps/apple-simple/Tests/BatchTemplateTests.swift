import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class BatchTemplateTests:XCTestCase {
 #if os(macOS)
 func testLiveCompleteTemplateGroupAndHistory()async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],let directory=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"] else{throw XCTSkip("Use the owned native-template --batch fixture.")}
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
   try await open();let catalog=try XCTUnwrap(model.catalog,model.notice);XCTAssertEqual(catalog.carets?.selections,ranges.map(SourceTemplateRange.init));XCTAssertEqual(catalog.language,"python");XCTAssertEqual(catalog.source_context?.source_hashes["project.md"],nativeSourceHash(project.text))
   try await choose("workspace.batch-context");let originalPreview=try XCTUnwrap(model.batchPreviewSource,model.notice);XCTAssertEqual(buffer.text,source);let accepted=await model.apply();XCTAssertTrue(accepted,model.notice);XCTAssertEqual(buffer.text,originalPreview)
   XCTAssertTrue(buffer.text.contains("#0:1:2024:012345:old|old|OLD #1:2:2024:012345:two|two|TWO"));XCTAssertNotNil(buffer.snippets.parallel)
   native.insertText("ßa",replacementRange:native.selectedRange());let typed=complete("ßa","SSA");XCTAssertEqual(buffer.text,typed)
   let beforeMove=buffer.multiRanges;try command("editor.snippetNext").run();XCTAssertNotEqual(buffer.multiRanges,beforeMove);try command("editor.snippetPrevious").run();XCTAssertEqual(buffer.multiRanges.map(\.length),[2,2])
   try await open();try await choose("workspace.batch-nested");XCTAssertEqual(model.batchPreview?.carets.continuing,true);let nestedAccepted=await model.apply();XCTAssertTrue(nestedAccepted,model.notice);let nested=complete("kid-kid","KID-KID");XCTAssertEqual(buffer.text,nested)
   native.insertText("ßa",replacementRange:native.selectedRange());let nestedTyped=complete("ßa-ßa","SSA-SSA");XCTAssertEqual(buffer.text,nestedTyped)
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
