import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple

@MainActor final class WorkspaceEntryMoveTests:XCTestCase {
 #if os(macOS)
 func testLiveMainMoveUndoRedo()async throws {
  guard let address=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],
        let directory=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"] else {
   throw XCTSkip("Use the owned workspace entry Move fixture.")
  }
  let foreign=ProcessInfo.processInfo.environment["HICKORY_ENTRY_MOVE_FOREIGN_ROOM"]
  let root=URL(fileURLWithPath:directory),old="project.md",new=foreign ?? "literal #main.spec"
  func disk(_ path:String)throws->String {try String(contentsOf:root.appendingPathComponent(path),encoding:.utf8)}
  XCTAssertEqual(try disk(".hickory-entry-move-fixture"),"owned-entry-move-v1\n")
  let engine=try Engine(address:address),tree=try await engine.files(),space=Workspace()
  space.engine=engine;space.rootFiles=tree.tree;space.sourceRootAddress=tree.root_path;space.sourceWorkspaceID=tree.workspace_id
  let buffer=try await space.loadBuffer(XCTUnwrap(tree.tree.first{$0.path==old}))
  let saved=buffer.text,draft=saved.replacingOccurrences(of:"saved.txt",with:"draft.txt")
  space.layout.activate(old,in:"document-group")
  let original=try await space.resourceRestoration.readEntry(space,current:{true}).proof.entry
  XCTAssertEqual(original.path,old)

  _=NSApplication.shared
  let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:500),styleMask:[.titled],backing:.buffered,defer:false)
  window.contentView=editor.scroll;window.makeKeyAndOrderFront(nil)
  defer{window.orderOut(nil);for value in space.buffers.values{value.language?.close()}}
  let native=editor.text,view=try XCTUnwrap(buffer.sourceEditorState?()),manager=try XCTUnwrap(native.undoManager)
  let range=(saved as NSString).range(of:"saved.txt")
  native.setSelectedRange(range);native.insertText("draft.txt",replacementRange:range)
  XCTAssertEqual(buffer.text,draft)
  let selection=native.selectedRange(),geometry=editor.scroll.frame

  func command(_ id:String)throws {
   let selected=try XCTUnwrap(space.commands().first{$0.id==id})
   XCTAssertNil(selected.reason);selected.run()
  }
  func wait(_ ready:()->Bool)async throws {
   for _ in 0..<400 {if ready(){return};try await Task.sleep(for:.milliseconds(25))}
   throw EngineError(message:space.changes.notice+" "+space.resourceRestoration.notice)
  }
  func assertState(_ path:String,absent:String,advance:UInt32)async throws {
   let stamp=try await space.resourceRestoration.readEntry(space,current:{true}).proof.entry
   XCTAssertEqual(stamp.path,path);XCTAssertEqual(stamp.identity,original.identity);XCTAssertEqual(stamp.generation,original.generation+advance)
   XCTAssertEqual(try disk(path),saved);XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent(absent).path))
   XCTAssertTrue(space.currentBuffer===buffer);XCTAssertEqual(buffer.text,draft);XCTAssertEqual(buffer.base,saved)
   let retained=try XCTUnwrap(buffer.sourceEditorState?())
   XCTAssertEqual(retained.identity,view.identity);XCTAssertEqual(native.selectedRange(),selection);XCTAssertEqual(editor.scroll.frame,geometry)
  }
  try command("workspace.moveEntry")
  try await wait{space.changes.entrySelection?.entryPath==old}
  space.changes.name=new
  await space.changes.prepare(space)
  let move=try XCTUnwrap(space.resourceRestoration.review,space.resourceRestoration.notice)
  XCTAssertEqual(move.entryIntent?.after_entry.path,new)
  try await assertState(old,absent:new,advance:0)
  await space.resourceRestoration.publish(space)
  XCTAssertFalse(space.resourceRestoration.hasPending,space.resourceRestoration.notice)
  if foreign != nil {
   XCTAssertTrue(space.resourceRestoration.notice.contains("Publication ended before any source write"),space.resourceRestoration.notice)
   try await assertState(old,absent:new,advance:0)
   XCTAssertTrue(native.undoManager===manager);XCTAssertTrue(manager.canUndo)
   manager.undo();XCTAssertEqual(buffer.text,saved)
   manager.redo();XCTAssertEqual(buffer.text,draft)
   try command("workspace.moveEntry")
   try await wait{space.changes.entrySelection?.entryPath==old}
   space.changes.name=new;await space.changes.prepare(space)
   XCTAssertNotNil(space.resourceRestoration.review,space.resourceRestoration.notice)
   return
  }
  try await assertState(new,absent:old,advance:1)

  for (id,path,absent,advance) in [("workspace.undo",old,new,UInt32(2)),("workspace.redo",new,old,UInt32(3))] {
   try command(id)
   try await wait{space.resourceRestoration.review != nil}
   XCTAssertEqual(space.resourceRestoration.review?.entryIntent?.after_entry.path,path)
   await space.resourceRestoration.publish(space)
   XCTAssertFalse(space.resourceRestoration.hasPending,space.resourceRestoration.notice)
   try await assertState(path,absent:absent,advance:advance)
  }
  XCTAssertTrue(native.undoManager===manager);XCTAssertTrue(manager.canUndo)
  manager.undo();XCTAssertEqual(buffer.text,saved)
  manager.redo();XCTAssertEqual(buffer.text,draft)
  XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("saved.txt").path))
  XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("draft.txt").path))
 }
 #endif
}
