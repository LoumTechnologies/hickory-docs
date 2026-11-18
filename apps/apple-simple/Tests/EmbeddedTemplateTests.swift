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
