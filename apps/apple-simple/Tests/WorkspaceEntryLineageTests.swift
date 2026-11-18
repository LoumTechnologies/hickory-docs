import XCTest
#if os(macOS)
import AppKit
import Darwin
#endif
@testable import HickorySimple
@MainActor final class WorkspaceEntryLineageTests:XCTestCase {
 #if os(macOS)
 func testLiveOlderMainMoveUndoRedo()async throws {
  guard let address=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],let directory=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"],let historyID=ProcessInfo.processInfo.environment["HICKORY_ENTRY_LINEAGE_HISTORY"] else{throw XCTSkip("Use test-workspace-entry-lineage.py --host apple")}
  let root=URL(fileURLWithPath:directory),a="project.md",b="literal #main.spec",c="second #main.md"
  func disk(_ path:String)throws->String{try String(contentsOf:root.appendingPathComponent(path),encoding:.utf8)}
  XCTAssertEqual(try disk(".hickory-entry-move-fixture"),"owned-entry-move-v1\n")
  let engine=try Engine(address:address),tree=try await engine.files(),space=Workspace();space.engine=engine;space.rootFiles=tree.tree;space.sourceRootAddress=tree.root_path;space.sourceWorkspaceID=tree.workspace_id
  let buffer=try await space.loadBuffer(XCTUnwrap(tree.tree.first{$0.path==a}));space.layout.activate(a,in:"document-group")
  let saved=buffer.text,draft=saved.replacingOccurrences(of:"saved.txt",with:"draft.txt"),earlier=saved.replacingOccurrences(of:"saved.txt",with:"before.txt")
  let original=try await space.resourceRestoration.readEntry(space,current:{true}).proof.entry
  _=NSApplication.shared
  let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:500),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;window.makeKeyAndOrderFront(nil)
  defer{window.orderOut(nil);for value in space.buffers.values{value.language?.close()}}
  let native=editor.text,view=try XCTUnwrap(buffer.sourceEditorState?()),manager=try XCTUnwrap(native.undoManager),range=(saved as NSString).range(of:"saved.txt")
  native.setSelectedRange(range);native.insertText("draft.txt",replacementRange:range)
  let selection=native.selectedRange(),geometry=editor.scroll.frame
  func command(_ id:String)throws{let selected=try XCTUnwrap(space.commands().first{$0.id==id});XCTAssertNil(selected.reason);selected.run()}
  func wait(_ ready:()->Bool)async throws{for _ in 0..<400{if ready(){return};try await Task.sleep(for:.milliseconds(25))};throw EngineError(message:space.changes.notice+" "+space.resourceRestoration.notice)}
  func check(_ path:String,_ generation:UInt32)async throws{
   let stamp=try await space.resourceRestoration.readEntry(space,current:{true}).proof.entry
   XCTAssertEqual(stamp.path,path);XCTAssertEqual(stamp.identity,original.identity);XCTAssertEqual(stamp.generation,original.generation+generation)
   XCTAssertEqual(try disk(path),saved);for other in [a,b,c] where other != path{XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent(other).path))}
   XCTAssertTrue(space.currentBuffer===buffer);XCTAssertEqual(buffer.text,draft);XCTAssertEqual(buffer.base,saved);XCTAssertEqual(buffer.sourceEditorState?()?.identity,view.identity);XCTAssertTrue(native.undoManager===manager);XCTAssertEqual(native.selectedRange(),selection);XCTAssertEqual(editor.scroll.frame,geometry)
   for output in ["saved.txt","draft.txt"]{XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent(output).path))}
  }
  func move(_ destination:String)async throws{
   try command("workspace.moveEntry");try await wait{space.changes.entrySelection?.entryPath==buffer.file.path};space.changes.name=destination;await space.changes.prepare(space)
   XCTAssertEqual(space.resourceRestoration.review?.entryIntent?.after_entry.path,destination,space.resourceRestoration.notice)
   await space.resourceRestoration.publish(space);XCTAssertFalse(space.resourceRestoration.hasPending,space.resourceRestoration.notice)
  }
  try await check(a,0);try await move(b);try await check(b,1);try await move(c);try await check(c,2)
  for(id,path,generation) in [("workspace.undo",b,UInt32(3)),("workspace.undo",a,UInt32(4)),("workspace.redo",b,UInt32(5)),("workspace.redo",c,UInt32(6))]{
   try command(id);try await wait{space.resourceRestoration.review != nil};XCTAssertEqual(space.resourceRestoration.review?.entryIntent?.after_entry.path,path,space.resourceRestoration.notice)
   await space.resourceRestoration.publish(space);XCTAssertFalse(space.resourceRestoration.hasPending,space.resourceRestoration.notice);try await check(path,generation)
  }
  await space.sourceHistory.open(space);let historical=try XCTUnwrap(space.sourceHistory.reading?.entries.first{$0.id==historyID},space.sourceHistory.notice);XCTAssertTrue(historical.available);XCTAssertEqual(historical.before.text,earlier);XCTAssertEqual(historical.after.text,saved);space.sourceHistory.close()
  manager.undo();XCTAssertEqual(buffer.text,saved);manager.redo();try await check(c,6)
  try command("workspace.undo");try await wait{space.resourceRestoration.review != nil}
  let originalInode=try FileManager.default.attributesOfItem(atPath:root.appendingPathComponent(c).path)[.systemFileNumber] as? NSNumber
  let replacement=root.appendingPathComponent("independent-replacement");try saved.write(to:replacement,atomically:false,encoding:.utf8)
  let result=replacement.path.withCString{source in root.appendingPathComponent(c).path.withCString{destination in Darwin.rename(source,destination)}};XCTAssertEqual(result,0)
  let replacedInode=try FileManager.default.attributesOfItem(atPath:root.appendingPathComponent(c).path)[.systemFileNumber] as? NSNumber;XCTAssertNotEqual(originalInode,replacedInode)
  await space.resourceRestoration.publish(space)
  XCTAssertTrue(space.resourceRestoration.hasPending,space.resourceRestoration.notice)
  XCTAssertTrue(space.resourceRestoration.notice.contains("physical source guard"),space.resourceRestoration.notice)
  await space.resourceRestoration.status(space)
  XCTAssertTrue(space.resourceRestoration.hasPending,space.resourceRestoration.notice)
  XCTAssertTrue(space.resourceRestoration.notice.contains("physical source guard"),space.resourceRestoration.notice)
  XCTAssertTrue(space.currentBuffer===buffer);XCTAssertEqual(buffer.file.path,c);XCTAssertEqual(buffer.text,draft);XCTAssertEqual(buffer.base,saved);XCTAssertEqual(native.selectedRange(),selection);XCTAssertEqual(editor.scroll.frame,geometry);XCTAssertEqual(try disk(c),saved)
  for absent in [a,b,"saved.txt","draft.txt"]{XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent(absent).path))}
 }
 #endif
}
