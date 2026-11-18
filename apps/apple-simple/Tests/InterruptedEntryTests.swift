import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple

@MainActor final class InterruptedEntryTests: XCTestCase {
 #if os(macOS)
 func testReviewedEntryCompletion() async throws {
  let environment=ProcessInfo.processInfo.environment
  guard let origin=environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],
        let directory=environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"] else {throw XCTSkip("Use the owned interrupted Move fixture.")}
  let root=URL(fileURLWithPath:directory).resolvingSymlinksInPath()
  func disk(_ path:String)throws->String {try String(contentsOf:root.appendingPathComponent(path),encoding:.utf8)}
  let bytes=try Data(contentsOf:root.appendingPathComponent(".hick-cache/entry-completion-fixture.json"))
  let marker=try XCTUnwrap(JSONSerialization.jsonObject(with:bytes) as? [String:Any])
  guard marker["version"] as? Int==1,marker["kind"] as? String=="entry-interrupted-completion-fixture",
        exactUTF8(marker["source_root"] as? String,root.path) else {throw EngineError(message:"Unowned interruption fixture.")}
  let old=try XCTUnwrap(marker["old_path"] as? String),destination=try XCTUnwrap(marker["destination"] as? String)
  let draft=try XCTUnwrap(marker["live_text"] as? String),saved=try XCTUnwrap(marker["saved_text"] as? String)
  let engine=try Engine(address:origin),tree=try await engine.files(),workspace=Workspace()
  guard exactUTF8(tree.root_path,root.path) else {throw EngineError(message:"Engine serves a different fixture root.")}
  workspace.engine=engine;workspace.rootFiles=tree.tree;workspace.sourceRootAddress=tree.root_path;workspace.sourceWorkspaceID=tree.workspace_id
  let model=workspace.resourceRestoration
  let result=try await model.readInterruptedEntry(workspace)
  let reading=try XCTUnwrap(result)
  guard reading.material.can_complete,exactUTF8(reading.source,old),exactUTF8(reading.destination,destination),
        reading.material.request_id==(marker["original_request_id"] as? String) else {
   throw EngineError(message:"Engine returned a different interruption.")
  }
  let buffer=try await workspace.loadBuffer(XCTUnwrap(tree.tree.first{$0.path==old}))
  workspace.layout.activate(old,in:"document-group")
  _=NSApplication.shared
  let editor=MacEditor(buffer)
  let window=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:500),styleMask:[.titled],backing:.buffered,defer:false)
  window.contentView=editor.scroll;window.makeKeyAndOrderFront(nil)
  defer{window.orderOut(nil);for source in workspace.buffers.values {source.language?.close()}}
  let range=NSRange(location:0,length:buffer.text.utf16.count)
  editor.text.setSelectedRange(range);editor.text.insertText(draft,replacementRange:range)
  let identity=try XCTUnwrap(buffer.sourceEditorState?()).identity
  let undo=try XCTUnwrap(editor.text.undoManager)
  let selection=editor.text.selectedRange(),geometry=editor.scroll.frame
  XCTAssertEqual(buffer.text,draft);XCTAssertTrue(undo.canUndo)

  workspace.showRecovery=true
  await model.prepareEntryCompletion(workspace,reading:reading,current:{workspace.showRecovery})
  let review=try XCTUnwrap(model.review,model.notice)
  XCTAssertEqual(review.entryIntent?.after_entry.path,destination)
  XCTAssertEqual(review.result.restoration.saved_before[old]?.text,saved)
  XCTAssertEqual(review.result.restoration.saved_after[destination]?.text,saved)
  XCTAssertEqual(try disk(old),saved);XCTAssertEqual(try disk(destination),saved)
  XCTAssertTrue(workspace.currentBuffer === buffer);XCTAssertEqual(buffer.text,draft)
  await model.publish(workspace)
  XCTAssertFalse(model.hasPending,model.notice)
  let stamp=try await model.readEntry(workspace,current:{true}).proof.entry
  XCTAssertEqual(stamp.path,destination);XCTAssertEqual(stamp.identity,reading.material.current_entry.identity)
  XCTAssertEqual(stamp.generation,reading.material.reservation_floor+1)
  XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent(old).path))
  XCTAssertEqual(try disk(destination),saved);XCTAssertEqual(buffer.base,saved)
  XCTAssertTrue(workspace.currentBuffer === buffer);XCTAssertEqual(buffer.text,draft)
  XCTAssertEqual(try XCTUnwrap(buffer.sourceEditorState?()).identity,identity)
  XCTAssertEqual(editor.text.selectedRange(),selection);XCTAssertEqual(editor.scroll.frame,geometry)
  XCTAssertTrue(editor.text.undoManager === undo)
  undo.undo();XCTAssertEqual(buffer.text,saved)
  undo.redo();XCTAssertEqual(buffer.text,draft)
 }
 #endif
}
