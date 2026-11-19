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
   let selectionAfter = item["selection_after"]
   func checkSelection(_ expectedText: String, after operation: String) {
    guard selectionAfter != nil else { return }
    let range = (b.text as NSString).range(of: expectedText)
    XCTAssertNotEqual(range.location, NSNotFound, operation)
    XCTAssertTrue(w.currentBuffer === b, operation)
    XCTAssertEqual(editor.selectedRange(), range, operation)
    XCTAssertEqual(b.selection, range, operation)
    XCTAssertEqual(b.sourceEditorState?()?.identity, identity, operation)
    print("HickoryAppKitRefactorSelection \(operation) \(range.location) \(range.length)")
   }
   try XCTUnwrap(w.commands().first{$0.id==(item["command"] ?? "symbol.actions")}).run();for _ in 0..<200{await w.changes.prepare(w);if w.changes.choices.contains(where:{$0["title"] as? String==title}){break};try await Task.sleep(for:.milliseconds(50))};let index=try XCTUnwrap(w.changes.choices.firstIndex{$0["title"] as? String==title});await w.changes.choose(index,workspace:w);if item["resources"] != nil {XCTAssertNotNil(w.resourceRestoration.review,w.resourceRestoration.notice)}else{XCTAssertNotNil(w.changes.plan,w.changes.notice)};XCTAssertEqual(b.text,draft);XCTAssertEqual(b.selection,selection);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(path),encoding:.utf8),source);if let created=item["created"] {XCTAssertFalse(FileManager.default.fileExists(atPath:directory.appendingPathComponent(created).path))};try await Task.sleep(for:.milliseconds(50));host.layoutSubtreeIfNeeded();XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry)
   checkSelection(selected, after: "Review")
   if item["resources"] != nil {await w.resourceRestoration.publish(w);XCTAssertNotNil(w.resourceRestoration.lastRecovery,w.resourceRestoration.notice)}else{w.changes.apply(w)}
   let changed=b.text;XCTAssertTrue(changed.contains(expected),w.changes.notice);XCTAssertTrue(changed.contains("// retained dirty"));try changed.write(to:URL(fileURLWithPath:artifacts).appendingPathComponent(path),atomically:true,encoding:.utf8)
   if let selectionAfter { checkSelection(selectionAfter, after: "Save") }
   w.changes.open("undo",workspace:w);if item["resources"] != nil {try await settle("Inverse resource review unavailable"){w.resourceRestoration.review != nil};await w.resourceRestoration.publish(w)};XCTAssertEqual(b.text,draft);if let created=item["created"] {XCTAssertFalse(FileManager.default.fileExists(atPath:directory.appendingPathComponent(created).path));XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(path),encoding:.utf8),source)}
   checkSelection(selected, after: "Undo")
   w.changes.open("redo",workspace:w);if item["resources"] != nil {try await settle("Redo resource review unavailable"){w.resourceRestoration.review != nil};await w.resourceRestoration.publish(w)};XCTAssertEqual(b.text,changed)
   if let selectionAfter { checkSelection(selectionAfter, after: "Redo") }
   w.changes.close(w);XCTAssertEqual(b.sourceEditorState?()?.identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);if item["resources"] == nil {XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(path),encoding:.utf8),source)};if let created=item["created"] {let next=try XCTUnwrap(w.buffers[created]);try next.text.write(to:URL(fileURLWithPath:artifacts).appendingPathComponent(created),atomically:true,encoding:.utf8)}
  }
 }
 #endif
}
