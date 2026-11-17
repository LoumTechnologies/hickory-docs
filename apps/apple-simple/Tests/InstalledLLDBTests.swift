import XCTest
@testable import HickorySimple
@MainActor final class InstalledLLDBTests:XCTestCase {
 func testLiveOwnedInstalledLLDB()async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_LLDB_ORIGIN"],let root=ProcessInfo.processInfo.environment["HICKORY_LLDB_WORKSPACE"],let mode=ProcessInfo.processInfo.environment["HICKORY_LLDB_MODE"] else{throw XCTSkip("Use maintained owned installed-LLDB fixture.")}
  let directory=URL(fileURLWithPath:root);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(".hickory-installed-lldb-fixture"),encoding:.utf8),"owned-installed-lldb-v1\n")
  let engine=try Engine(address:origin),workspace=Workspace();workspace.engine=engine;let listing=try await engine.files();workspace.rootFiles=listing.tree;workspace.sourceRootAddress=listing.root_path;workspace.sourceWorkspaceID=listing.workspace_id
  let file=FileNode(name:"main.c",path:"main.c",dir:false),buffer=try await workspace.loadBuffer(file),source=buffer.text,debug=NativeDebugger(engine:engine);workspace.debugger=debug;debug.sourceBuffers={Array(workspace.buffers.values)}
  defer{debug.close();workspace.sourceTestIndex?.stop();for value in workspace.buffers.values{value.language?.close()}}
  func wait(_ reason:String,_ predicate:()->Bool)async throws{for _ in 0..<1500{if predicate(){return};try await Task.sleep(for:.milliseconds(20))};throw EngineError(message:reason+" · "+debug.status+" · "+debug.message+" · "+debug.breakpointNotice)}
  if mode=="launch"{debug.toggle(buffer,line:5)};await debug.prepare(buffer);try await wait("LLDB did not pause: \(debug.message)"){debug.status=="paused"||debug.status=="failed"};XCTAssertEqual(debug.status,"paused",debug.message);XCTAssertFalse(debug.frames.isEmpty)
  if mode=="launch"{debug.evaluate("value");try await wait("LLDB value unavailable: \(debug.evaluationNotice)"){debug.values["value"] != nil};XCTAssertEqual(debug.values["value"],"(volatile int) $0 = 41")}
  XCTAssertTrue(workspace.buffers[file.path] === buffer);XCTAssertEqual(buffer.text,source);debug.stop();try await wait("LLDB did not stop"){!debug.active};XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(file.path),encoding:.utf8),source)
 }
}
