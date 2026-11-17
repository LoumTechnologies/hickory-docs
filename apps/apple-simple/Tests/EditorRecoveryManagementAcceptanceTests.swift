import XCTest
import SwiftUI
@testable import HickorySimple
#if os(macOS)
import AppKit
@MainActor final class EditorRecoveryManagementAcceptanceTests:XCTestCase {
 func testOwnedCapacityRevisionAndForgetFlow() async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_RECOVERY_MANAGEMENT_ORIGIN"],let root=ProcessInfo.processInfo.environment["HICKORY_RECOVERY_MANAGEMENT_WORKSPACE"] else{throw XCTSkip("Use the owned recovery management fixture.")}
  XCTAssertEqual(try String(contentsOfFile:root+"/.hickory-native-recovery-management-fixture",encoding:.utf8),"owned-recovery-management-v1\n")
  _=NSApplication.shared;NSApp.setActivationPolicy(.regular);let engine=try Engine(address:origin),w=Workspace();w.engine=engine;let listing=try await engine.files();w.rootFiles=listing.tree;w.sourceRootAddress=listing.root_path;w.sourceWorkspaceID=listing.workspace_id
  func find(_ nodes:[FileNode])->FileNode?{for node in nodes{if node.path=="note.py"{return node};if let found=find(node.children ?? []){return found}};return nil}
  let b=try await w.loadBuffer(try XCTUnwrap(find(listing.tree)));w.showSource(b.file.path);let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:980,height:700),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
  func settle(_ message:String,_ predicate:()->Bool)async throws{for _ in 0..<600{if predicate(){return};try await Task.sleep(for:.milliseconds(20))};throw EngineError(message:message)}
  func native(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{native($0)}.first}
  try await settle("Original code editor did not attach"){b.sourceEditorState?()?.attached==true};let editor=try XCTUnwrap(native(host)),manager=try XCTUnwrap(editor.undoManager),source=b.text,editorIdentity=try XCTUnwrap(b.sourceEditorState?()).identity,scroll=try XCTUnwrap(editor.enclosingScrollView)
  editor.setSelectedRange(NSRange(location:editor.string.utf16.count,length:0));editor.insertText("# independent dirty draft\n",replacementRange:editor.selectedRange());editor.setSelectedRange(NSRange(location:2,length:3));b.selection=editor.selectedRange();scroll.contentView.scroll(to:CGPoint(x:0,y:90));scroll.reflectScrolledClipView(scroll.contentView)
  let text=b.text,selection=b.selection,identity=ObjectIdentifier(b),viewport=scroll.contentView.bounds.origin,m=w.editorRecovery
  defer{m.close();w.sourceTestIndex?.stop();for buffer in w.buffers.values{buffer.language?.close()};window.orderOut(nil);window.contentView=nil}
  func allPages()async throws{for _ in 0..<32{guard let previous=m.catalogue?.next else{return};await m.more(w);guard m.catalogue?.next != previous else{throw EngineError(message:"Recovery page did not advance: "+m.notice)}};guard m.catalogue?.next==nil else{throw EngineError(message:"Recovery exceeded bounded page count.")}}
  m.showing=true;host.layoutSubtreeIfNeeded();try await settle("Mounted recovery popover did not load"){!m.busy&&m.catalogue != nil};try await allPages()
  XCTAssertEqual(m.catalogue?.sessions.count,31,m.notice);XCTAssertEqual(m.catalogue?.unreadable_sessions?.count,1)
  let row=try XCTUnwrap(m.catalogue?.sessions.first{$0.client_id==String(format:"%032x",32)})
  for revision in [UInt32(1),2,3]{await m.read(row,w,revision:revision);XCTAssertEqual(m.snapshot?.buffers.first?.text,"private session 32 revision \(revision)\n",m.notice);XCTAssertTrue(m.canRestore,m.notice);XCTAssertEqual(b.sourceEditorState?()?.identity,editorIdentity);XCTAssertEqual(scroll.contentView.bounds.origin,viewport);XCTAssertEqual(b.selection,selection)}
  await m.read(row,w,revision:1);m.restore(w);XCTAssertTrue(w.buffers.values.contains{$0.isUntitledDraft&&$0.text=="private session 32 revision 1\n"},m.notice)
  await m.open(w);try await allPages()
  let corrupt=try XCTUnwrap(m.catalogue?.unreadable_sessions?.first);await m.forget(corrupt.removal,w);XCTAssertFalse(m.hasPendingForget,m.notice)
  let chosen=try XCTUnwrap(m.catalogue?.sessions.first{$0.client_id==row.client_id});await m.forget(chosen.removal,w);XCTAssertFalse(m.hasPendingForget,m.notice)
  await m.open(w);try await allPages();XCTAssertFalse(m.catalogue!.sessions.contains{$0.client_id==row.client_id})
  XCTAssertEqual(ObjectIdentifier(b),identity);XCTAssertEqual(b.sourceEditorState?()?.identity,editorIdentity);XCTAssertEqual(b.text,text);XCTAssertEqual(b.selection,selection);XCTAssertTrue(manager.canUndo);m.close();manager.undo();XCTAssertEqual(b.text,source);XCTAssertFalse(try String(contentsOfFile:root+"/note.py",encoding:.utf8).contains("independent dirty"))
 }
}
#endif
