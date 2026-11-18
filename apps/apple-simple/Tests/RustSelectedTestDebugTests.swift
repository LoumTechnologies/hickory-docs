import XCTest
@testable import HickorySimple
#if os(macOS)
import AppKit
import SwiftUI
@MainActor final class RustSelectedTestDebugTests:XCTestCase{
 func testLiveSavedRustSelectionAndHistory()async throws{
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],let path=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"]else{throw XCTSkip("Use the maintained Rust test Debug fixture")}
  let root=URL(fileURLWithPath:path);XCTAssertEqual(try String(contentsOf:root.appendingPathComponent(".hickory-rust-test-debug-fixture"),encoding:.utf8),"owned-rust-test-debug-v1\n");_ = NSApplication.shared;NSApp.setActivationPolicy(.regular);let engine=try Engine(address:origin),listing=try await engine.files()
  for path in ["physical/src/lib.rs","tests.md"]{
   let w=Workspace();w.engine=engine;w.rootFiles=listing.tree;w.sourceRootAddress=listing.root_path;w.sourceWorkspaceID=listing.workspace_id
   func find(_ path:String,_ nodes:[FileNode])->FileNode?{for node in nodes{if node.path==path{return node};if let value=find(path,node.children ?? []){return value}};return nil}
   let file=try XCTUnwrap(find(path,listing.tree)),b=try await w.loadBuffer(file),source=b.text;w.layout.activateSource(path,in:"document-group");let debug=NativeDebugger(engine:engine);w.debugger=debug;debug.sourceBuffers={Array(w.buffers.values)}
   let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:920,height:620),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
   defer{if debug.active{debug.stop()};debug.close();w.sourceTestIndex?.stop();for value in w.buffers.values{value.language?.close()};window.orderOut(nil);window.contentView=nil}
   func native(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{native($0)}.first}
   func wait(_ label:String,_ ready:()->Bool)async throws{for _ in 0..<1500{host.layoutSubtreeIfNeeded();if ready(){return};try await Task.sleep(for:.milliseconds(20))};throw EngineError(message:label+": "+debug.message+" · "+debug.status)}
   try await wait("Native editor did not mount"){native(host) != nil};let text=try XCTUnwrap(native(host)),scroll=try XCTUnwrap(text.enclosingScrollView),identity=try XCTUnwrap(b.sourceEditorState?()).identity,geometry=scroll.convert(scroll.bounds,to:host);let selection=text.selectedRange(),position=scroll.contentView.bounds.origin
   let line=try XCTUnwrap(source.components(separatedBy:"\n").firstIndex{$0.contains("owned-rust-stop")});debug.toggle(b,line:line);w.showTests=true;await w.testSources.refresh(w);let facts=try XCTUnwrap(w.testSources.observation?.1),candidate=try XCTUnwrap(facts.candidates.first{$0.name=="test_selected"});XCTAssertEqual(candidate.language,"rust");await w.testSources.debug(candidate,.nameFilter,workspace:w)
   try await wait("Selected Rust test did not pause"){debug.status=="paused"||debug.status=="failed"};XCTAssertEqual(debug.status,"paused",debug.message);XCTAssertEqual(debug.pausedLine,line);XCTAssertEqual(w.testSources.debugRuns.count,1);XCTAssertEqual(b.text,source);XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("sibling-ran").path));debug.toggle(b,line:line);debug.step("continue");try await wait("Selected Rust test did not finish"){debug.status=="finished"||debug.status=="failed"};XCTAssertEqual(debug.status,"finished",debug.message);XCTAssertEqual(b.text,source);XCTAssertEqual(try String(contentsOf:root.appendingPathComponent(path),encoding:.utf8),source);XCTAssertEqual(try XCTUnwrap(b.sourceEditorState?()).identity,identity);XCTAssertEqual(text.selectedRange(),selection);XCTAssertEqual(scroll.contentView.bounds.origin,position);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("sibling-ran").path));XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("Cargo.toml").path))
  }
 }
}
#endif
