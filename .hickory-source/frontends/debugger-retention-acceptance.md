<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retain breakpoint wishes without granting stale coordinates

For maintainers changing the boundary between local breakpoint intentions, complete adapter receipts and stopped stack observations.

An edit does not authorize the adapter to reuse a line number. Retain its old source wishes as a readonly reading, hide obsolete positional marks, and admit a new binding only from a complete receipt for the exact source and latest intention. Explicit placement on current text archives the older wishes; it cannot change the running snapshot. The existing floating controls page retained wishes without reserving editor space. History is bounded and in memory, not crash-persistent breakpoint evidence.

## Refuse incomplete binding evidence

These tests exercise the actual native setters and browser source model. Missing, duplicated, foreign and malformed rows must not promote a partial setter response into source binding. A later complete response can still confirm the same request; earlier wishes and current drafts survive refusal.

<ui:copy id="debugger-retention-apple-tests">
import XCTest
import SwiftUI
@testable import HickorySimple
#if os(macOS)
import AppKit
import Vision
import WebKit
#endif
@MainActor final class DebuggerRetentionTests:XCTestCase {
 func testCompleteBreakpointReceiptCoverageAndStates()throws {
  let debug=NativeDebugger(engine:try Engine(address:"http://127.0.0.1:1")),file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:FileNode(name:"main.py",path:"main.py",dir:false),text:FileText(path:"main.py",content:"first = 1\nsecond = 2\n",hash:"saved"));defer{debug.close()}
  debug.owner=file.path;debug.launchSource=buffer.text;debug.sourceBuffers={ [buffer] };var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)};let launch=debug.beginLaunchRequest();debug.receive(["event":"started","source_hash":nativeSourceHash(buffer.text),"request_id":launch,"doc":languageURI(file.path),"session":"owned","breakpoints":[]])
  debug.toggle(buffer,line:0,condition:"first == 1");debug.toggle(buffer,line:1,condition:"second == 2");let request=try XCTUnwrap(requests.last)
  let good:[[String:Any]]=[["line":0,"state":"bound"],["line":1,"state":"pending"]]
  let bad:[[[String:Any]]]=[[],[good[0]],[good[0],good[0]],[good[0],["line":2,"state":"bound"]],[good[0],["line":1,"state":"invented"]],[good[0],["line":1,"state":"bound","moved_to":999]],[good[0],["line":1,"state":"bound","message":["foreign":true]]],[good[0],["line":true,"state":"bound"]]]
  func receipt(_ rows:[[String:Any]])->[String:Any]{["event":"breakpoints","session":"owned","request_id":request["request_id"]!,"source_hash":request["source_hash"]!,"breakpoints":rows]}
  for rows in bad {debug.receive(receipt(rows));XCTAssertFalse(debug.breakpointMarkers(buffer).contains{$0["state"] as? String=="bound"});XCTAssertEqual(debug.breakpoints[file.path]?.count,2);XCTAssertEqual(buffer.text,"first = 1\nsecond = 2\n")}
  debug.receive(receipt(good));XCTAssertEqual(debug.breakpointMarkers(buffer).filter{$0["state"] as? String=="bound"}.count,1);XCTAssertEqual(debug.breakpoints[file.path]?.first?["condition"] as? String,"first == 1")
 }
 func testEditedSourceRetainsEarlierAndCurrentWishesWithoutRuntimeSetters()throws {
  let debug=NativeDebugger(engine:try Engine(address:"http://127.0.0.1:1")),buffer=DocumentBuffer(file:FileNode(name:"main.py",path:"main.py",dir:false),text:FileText(path:"main.py",content:"value = 1\n",hash:"saved"));defer{debug.close()};debug.owner=buffer.file.path;debug.launchSource=buffer.text;debug.sourceBuffers={ [buffer] };var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)};let launch=debug.beginLaunchRequest();debug.receive(["event":"started","source_hash":nativeSourceHash(buffer.text),"request_id":launch,"doc":languageURI(buffer.file.path),"session":"owned","breakpoints":[]]);debug.toggle(buffer,line:0,condition:"value == 1");let before=requests.count
  buffer.text="# changed\nvalue = 1\n";debug.sourceChanged(buffer);XCTAssertTrue(debug.breakpointMarkers(buffer).isEmpty);debug.toggle(buffer,line:1,condition:"value >= 1");XCTAssertEqual(requests.count,before);let reading=debug.retainedBreakpointReading(buffer.file.path);XCTAssertTrue(reading.contains{$0.contains("Earlier")&&$0.contains("value == 1")});XCTAssertTrue(reading.contains{$0.contains("Retained")&&$0.contains("value >= 1")});XCTAssertTrue(debug.breakpointMarkers(buffer).isEmpty);XCTAssertEqual(buffer.text,"# changed\nvalue = 1\n")
 }
 func testRecordedRunKeepsOnlyExactTerminalFromSelectedArchivedSide() async throws {
  let configuration = URLSessionConfiguration.ephemeral
  configuration.protocolClasses = [ResponseProtocol.self]
  let transport = URLSession(configuration: configuration)
  let engine = try Engine(address: "http://recorded-run.invalid", session: transport)
  let model = NativeRecordedFenceRun(), agent = AgentSession()
  let root = "/owned workspace", turn = "recorded-run", terminal = "term-" + String(repeating: "a", count: 32)
  let endpoint = "/api/docs/workspace/agent/turns/recorded-run/fence-run"
  let output = "/api/terminals/" + terminal + "/output"
  let text = "Recorded write-up 😀\r\n~~~python\r\nprint(41)\r\n~~~\r\n"
  let hash = nativeSourceHash(text)
  agent.turns = [AgentTurn(id: turn, prompt: text, answer: text, status: "ok")]
  ResponseProtocol.resetRequests(); ResponseProtocol.status = 200; ResponseProtocol.beforeResponse = nil
  defer { model.close(); transport.invalidateAndCancel(); ResponseProtocol.beforeResponse = nil; ResponseProtocol.responsesByPath = [:] }
  let summary: [String: Any] = ["id": terminal, "title": "Run recorded fence", "state": "done", "exit_code": 0]
  ResponseProtocol.responsesByPath["/api/terminals"] = try JSONSerialization.data(withJSONObject: ["sessions": [summary]])
  ResponseProtocol.responsesByPath[output] = try JSONSerialization.data(withJSONObject: [
    "bytes": Array("41\r\n".utf8), "next": NSNull(), "revision": "screen"
  ])
  var ownsWorkspace = true
  func setup(_ side: RecordedMessageSide) throws -> (DocumentBuffer, NativeRecordedMessageSource, RecordedDebugFence) {
   let buffer = agent.document("\(turn)-\(side.bufferSuffix)", text)
   buffer.inlineMarkup = InlineSnapshot(source: text, debug: inlineDebugContext(text, path: buffer.file.path))
   let source = NativeRecordedMessageSource(turn: turn, side: side, workspace: root) {
    ownsWorkspace && agent.owns(buffer, turn: turn, side: side)
   }
   let fence = try XCTUnwrap(buffer.inlineMarkup.debug.recorded[1])
   return (buffer, source, fence)
  }
  func reply(_ side: RecordedMessageSide, workspace: String) throws {
   ResponseProtocol.responsesByPath[endpoint] = try JSONSerialization.data(withJSONObject: [
    "version": 1, "terminal": summary, "source_hash": hash, "opening_line": 1,
    "workspace": workspace, "turn": turn, "side": side.rawValue,
   ])
  }
  for side in [RecordedMessageSide.user, .assistant] {
   let (buffer, source, fence) = try setup(side)
   try reply(side, workspace: root)
   await model.start(buffer, source: source, fence: fence, engine: engine)
   XCTAssertEqual(model.line, 1); XCTAssertEqual(model.terminal?.id, terminal)
   let running = await model.refresh(engine)
   XCTAssertFalse(running); XCTAssertEqual(model.terminal?.exit_code, 0)
   XCTAssertEqual(model.output, "41\r\n"); XCTAssertEqual(buffer.text, text)
   let body = try XCTUnwrap(ResponseProtocol.requestBodies(endpoint).last)
   XCTAssertEqual(Set(body.keys), Set(["side", "source_hash", "opening_line"]))
   XCTAssertEqual(body["side"] as? String, side.rawValue)
   XCTAssertEqual(body["source_hash"] as? String, hash); XCTAssertEqual(body["opening_line"] as? Int, 1)
   model.close()
  }
  let (buffer, source, fence) = try setup(.assistant)
  try reply(.assistant, workspace: "/foreign")
  await model.start(buffer, source: source, fence: fence, engine: engine)
  XCTAssertNil(model.terminal); XCTAssertTrue(model.notice.contains("did not confirm"))
  model.close()

  try reply(.assistant, workspace: root)
  let hold = DispatchSemaphore(value: 0)
  ResponseProtocol.beforeResponse = { request in
   if request.url?.path == endpoint { _ = hold.wait(timeout: .now() + 3) }
  }
  let prior = ResponseProtocol.requestCount(endpoint)
  let pending = Task { await model.start(buffer, source: source, fence: fence, engine: engine) }
  for _ in 0..<100 where ResponseProtocol.requestCount(endpoint) == prior {
   try await Task.sleep(for: .milliseconds(10))
  }
  XCTAssertEqual(ResponseProtocol.requestCount(endpoint), prior + 1)
  ownsWorkspace = false
  hold.signal()
  await pending.value
  XCTAssertNil(model.line); XCTAssertNil(model.terminal); XCTAssertTrue(model.output.isEmpty)
  XCTAssertEqual(ResponseProtocol.requestCount(output), 2)
  XCTAssertEqual(buffer.text, text)
 }
 func testDeclaredPatchReadingBindsExactSourceAndNamespace() async throws {
  let configuration = URLSessionConfiguration.ephemeral
  configuration.protocolClasses = [ResponseProtocol.self]
  let transport = URLSession(configuration: configuration)
  let engine = try Engine(address: "http://recorded-construction.invalid", session: transport)
  let model = NativeRecordedConstruction(), agent = AgentSession()
  let root = "/owned", id = String(repeating: "a", count: 32), turn = "owned-construction"
  let endpoint = "/api/docs/workspace/agent/turns/\(turn)/fence-construction"
  let text = "Archived 😀\r\n```patch id=\"first\"\r\nchange\r\n```\r\n\r\n\u{3C}hick:patch id=\"second\">\r\nchange\r\n\u{3C}/hick:patch>\r\n```diff\r\nchange\r\n```"
  agent.turns = [AgentTurn(id: turn, prompt: text, answer: text, status: "ok")]
  let buffer = agent.document("\(turn)-agent", text)
  buffer.inlineMarkup = InlineSnapshot(source: text, debug: inlineDebugContext(text, path: buffer.file.path))
  XCTAssertEqual(buffer.inlineMarkup.debug.recordedConstructions, [1, 5])
  XCTAssertEqual(buffer.inlineMarkup.debug.recordedPatches, [8])
  let source = NativeRecordedMessageSource(turn: turn, side: .assistant, workspace: root, workspaceID: id) {
   agent.owns(buffer, turn: turn, side: .assistant)
  }
  ResponseProtocol.resetRequests(); ResponseProtocol.status = 200; ResponseProtocol.beforeResponse = nil
  defer { model.close(); transport.invalidateAndCancel(); ResponseProtocol.responsesByPath = [:] }
  let after = "produced\n", hash = nativeSourceHash(text)
  let opening = try XCTUnwrap(Array(text.utf8).firstIndex(of: 10)) + 1
  let change: [String: Any] = ["path": "out.txt", "before": NSNull(), "after": [
   "kind": "file", "executable": false, "bytes": after.utf8.count, "sha256": nativeSourceHash(after),
   "text": after, "text_status": "exact",
  ]]
  var receipt: [String: Any] = ["version": 1, "read_only": true, "turn": turn, "side": "assistant",
   "source_hash": hash, "opening_line": 1, "workspace": root, "workspace_id": id, "entry": "entry.md",
   "document": "entry.md", "operation_id": "first", "opening_byte": opening,
   "completed_fingerprint": String(repeating: "b", count: 64), "changes": [change],
  ]
  for (workspace, admitted) in [(id, true), (String(repeating: "c", count: 32), false)] {
   receipt["workspace_id"] = workspace
   ResponseProtocol.responsesByPath[endpoint] = try JSONSerialization.data(withJSONObject: receipt)
   await model.inspect(buffer, source: source, line: 1, engine: engine)
   XCTAssertEqual(model.receipt?.changes.first?.after?.text, admitted ? after : nil, model.notice)
   XCTAssertTrue(exactUTF8(buffer.text, text)); XCTAssertFalse(buffer.dirty)
  }
  let request = try XCTUnwrap(ResponseProtocol.requestBodies(endpoint).first)
  XCTAssertEqual(Set(request.keys), ["source_hash", "side", "opening_line"])
  XCTAssertEqual(request["source_hash"] as? String, hash)
  XCTAssertEqual(request["side"] as? String, "assistant")
  XCTAssertEqual(request["opening_line"] as? Int, 1)
 }
 func testRecordedMessageDebugBindsOnlySelectedFenceAndExactFrames() async throws {
  let configuration = URLSessionConfiguration.ephemeral
  configuration.protocolClasses = [ResponseProtocol.self]
  let transport = URLSession(configuration: configuration)
  let debug = NativeDebugger(engine: try Engine(address: "http://recorded-fence.invalid", session: transport))
  defer { debug.close(); transport.invalidateAndCancel(); ResponseProtocol.responsesByPath = [:] }
  let agent = AgentSession(), turn = "0123456789abcdef", id = "aaaaaaaaaaaaaaaa"
  let uri = "hick-project:///\(id)/message.md", root = "/owned workspace"
  let message = "Example 😀\r\n  ```python\r\n  value = 41\r\n  print(value + 1)\r\n  ```\r\n\r\n~~~py\r\nraise RuntimeError('wrong fence')\r\n~~~\r\n\r\n```python\r\nunfinished\r\n"
  agent.turns = [AgentTurn(id: turn, prompt: message, answer: message, status: "running")]
  let user = agent.document("\(turn)-user", message), assistant = agent.document("\(turn)-agent", message)
  XCTAssertTrue(agent.owns(user, turn: turn, side: .user))
  XCTAssertFalse(agent.owns(assistant, turn: turn, side: .assistant))
  agent.turns[0].status = "ok"
  XCTAssertTrue(agent.owns(assistant, turn: turn, side: .assistant))

  let path = "/api/docs/workspace/agent/turns/\(turn)/fence-debug"
  let hash = nativeSourceHash(message)
  var sent: [[String: Any]] = []
  debug.emitForTest = { sent.append($0) }
  ResponseProtocol.status = 200
  for (side, buffer) in [(RecordedMessageSide.user, user), (.assistant, assistant)] {
   XCTAssertTrue(buffer.readOnly)
   buffer.inlineMarkup = InlineSnapshot(source: message, debug: inlineDebugContext(message, path: buffer.file.path))
   XCTAssertEqual(Set(buffer.inlineMarkup.debug.recorded.keys), Set([1, 6]))
   let fence = try XCTUnwrap(buffer.inlineMarkup.debug.recorded[1])
   XCTAssertEqual(fence.closing, 4)
   let source = NativeRecordedMessageSource(turn: turn, side: side, workspace: root) { agent.owns(buffer, turn: turn, side: side) }
   let reply: [String: Any] = ["version": 1, "id": id, "uri": uri, "program": "fence.py", "source_hash": hash,
     "opening_line": 1, "workspace": root, "started": false, "turn": turn, "side": side.rawValue]
   ResponseProtocol.responsesByPath[path] = try JSONSerialization.data(withJSONObject: reply)
   debug.toggle(buffer, line: 3)
   debug.toggle(buffer, line: 7)
   XCTAssertEqual(debug.breakpointMarkers(buffer, recorded: true).count, 2)
   await debug.prepareRecordedFence(buffer, source: source, fence: fence)
   let start = try XCTUnwrap(sent.last)
   XCTAssertEqual(start["op"] as? String, "start", debug.message)
   XCTAssertEqual(start["doc"] as? String, uri)
   XCTAssertEqual(start["source_hash"] as? String, hash)
   XCTAssertNil(start["profile"])
   XCTAssertNil(start["sources"])
   XCTAssertEqual((start["breakpoints"] as? [[String: Any]])?.compactMap { $0["line"] as? Int }, [3])
   let request = try XCTUnwrap(start["request_id"] as? String)
   debug.receive(["event": "started", "doc": uri, "request_id": request, "session": "owned", "source_hash": hash,
     "breakpoints": [["line": 3, "state": "bound"]]])
   XCTAssertEqual(debug.breakpointMarkers(buffer, recorded: true).map { $0["state"] as? String }, ["bound", "pending"])
   var revealed: [Int] = []
   buffer.revealDebugLine = { revealed.append($0); return true }
   func stop(_ stop: Int, hash: String) {
    debug.receive(["event": "stopped", "session": "owned", "stop": stop, "thread": 0, "line": 3,
      "variables": [], "frames": [["id": 0, "name": "recorded frame", "source": uri, "line": 3,
        "source_line": 3, "in_document": true, "source_hash": hash]]])
   }
   stop(1, hash: String(repeating: "b", count: 64))
   XCTAssertNil(debug.line(in: buffer.file.path))
   XCTAssertEqual(revealed, [])
   stop(2, hash: hash)
   XCTAssertEqual(debug.line(in: buffer.file.path), 3)
   XCTAssertEqual(revealed, [3])
   debug.step("over")
   XCTAssertEqual(sent.last?["op"] as? String, "step")
   XCTAssertEqual(sent.last?["stop"] as? UInt64, 2)
   XCTAssertTrue(exactUTF8(buffer.text, message))
   XCTAssertFalse(buffer.dirty)
   debug.close()
  }
 }
 #if os(macOS)
 #if SWIFT_PACKAGE
 func testRecordedRunUsesPackagedTerminalPageInXCTest() throws {
  let bundle = Bundle(for: MathCoordinator.self)
  XCTAssertEqual(bundle.bundleURL.pathExtension, "xctest")
  let html = try XCTUnwrap(nativeMathResource(roots: [], bundles: [bundle]))
  XCTAssertEqual(html.lastPathComponent, "math.html")
  let source = try String(contentsOf: html, encoding: .utf8)
  XCTAssertTrue(source.contains("globalThis.renderTerminal"))
  XCTAssertTrue(FileManager.default.isReadableFile(atPath: html.deletingLastPathComponent().appendingPathComponent("node_modules/xterm.js").path))
 }
 #endif

 func testMountedRecordedUserAndAssistantRunsShowTerminalWithoutChangingEditors() async throws {
  guard let origin = ProcessInfo.processInfo.environment["HICKORY_RECORDED_FENCE_ORIGIN"] else {
   throw XCTSkip("Use the owned recorded-message fixture.")
  }
  _ = NSApplication.shared
  NSApp.setActivationPolicy(.regular)
  let workspace = Workspace()
  workspace.address = origin
  guard await workspace.connect() else { throw EngineError(message: workspace.status) }
  let engine = try XCTUnwrap(workspace.engine), root = try XCTUnwrap(workspace.sourceRootAddress)
  let record = URL(fileURLWithPath: root).appendingPathComponent("sessions/agent.md")
  let original = try Data(contentsOf: record), agent = workspace.agent
  agent.prompt = "Retained response draft"
  workspace.showUtility(.agent)
  let host = NSHostingView(rootView: WorkspaceView(workspace: workspace))
  let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 1024, height: 900),
                        styleMask: [.titled], backing: .buffered, defer: false)
  window.contentView = host
  window.makeKeyAndOrderFront(nil)
  NSApp.activate()
  defer { workspace.debugger?.close(); workspace.editorRecovery.close(); window.orderOut(nil); window.contentView = nil }
  func views() -> [NSView] {
   var windows = [window] + NSApp.windows.filter(\.isVisible), seen = Set<ObjectIdentifier>()
   var queue: [NSView] = [], result: [NSView] = []
   while let candidate = windows.popLast() {
    guard seen.insert(ObjectIdentifier(candidate)).inserted else { continue }
    if let view = candidate.contentView { queue.append(view) }
    windows += candidate.childWindows ?? []
    if let sheet = candidate.attachedSheet { windows.append(sheet) }
   }
   while let view = queue.popLast() { result.append(view); queue += view.subviews }
   return result
  }
  func dismiss(_ popoverWindow: NSWindow) throws {
   let escape = try XCTUnwrap(NSEvent.keyEvent(with: .keyDown, location: .zero, modifierFlags: [],
    timestamp: ProcessInfo.processInfo.systemUptime, windowNumber: popoverWindow.windowNumber, context: nil,
    characters: "\u{1b}", charactersIgnoringModifiers: "\u{1b}", isARepeat: false, keyCode: 53))
   popoverWindow.sendEvent(escape)
  }
  func wait(_ message: String, ready: () async -> Bool) async throws {
   for _ in 0..<500 {
    host.layoutSubtreeIfNeeded()
    if await ready() { return }
    try await Task.sleep(for: .milliseconds(25))
   }
   XCTFail(message)
   throw EngineError(message: message)
  }
  struct Listing: Decodable { var sessions: [NativeRecordedRunTerminal] }
  func terminalCount() async throws -> Int {
   let listing: Listing = try await engine.request("GET", "/api/terminals")
   return listing.sessions.count
  }
  let id = "recorded-fence-host"
  try await wait("The retained message did not mount") { agent.turns.contains { $0.id == id } }
  let turn = try XCTUnwrap(agent.turns.first { $0.id == id })
  for (side, source, expected) in [(RecordedMessageSide.user, turn.prompt, "42"),
                                    (.assistant, try XCTUnwrap(turn.answer), "53")] {
   let buffer = agent.document("\(id)-\(side.bufferSuffix)", source)
   let originalSelection = NSRange(location: 2, length: 4)
   var editor: NSTextView?
   try await wait("Recorded source editor did not mount") {
    editor = views().compactMap { $0 as? NSTextView }.first { exactUTF8($0.string, source) }
    return editor != nil && buffer.inlineMarkup.debug.recorded[2] != nil
     && buffer.visibleDebugLines.contains { $0.line == 2 }
   }
   let text = try XCTUnwrap(editor), scroll = try XCTUnwrap(text.enclosingScrollView)
   text.setSelectedRange(originalSelection)
   let state = try XCTUnwrap(buffer.sourceEditorState?()), frame = scroll.convert(scroll.bounds, to: host)
   let row = try XCTUnwrap(buffer.visibleDebugLines.first { $0.line == 2 })
   let count = try await terminalCount()
   // The fixture separates the two fences by five rows. Run occupies the first slot.
   // The right rail starts after the 40-point minimap and four-point divider.
   let y = scroll.isFlipped ? scroll.bounds.minY + row.y + 9 : scroll.bounds.maxY - row.y - 9
   let point = scroll.convert(NSPoint(x: scroll.bounds.maxX + 40 + 4 + 11, y: y), to: nil)
   try clickRecordedControl(at: point, in: window)
   var outputWebView: WKWebView?
   var renderer = "No visible terminal WebKit view"
   do {
    try await wait("The recorded \(side.rawValue) Run did not show its terminal output") {
     for web in views().compactMap({ $0 as? WKWebView }) {
      let value = try? await web.evaluateJavaScript("""
       (() => {
         const terminal = typeof outputTerminal === 'undefined' ? undefined : outputTerminal;
         const bounds = terminal?.element?.getBoundingClientRect();
         const screen = terminal?.buffer?.active;
         const lines = screen ? Array.from({length: terminal.rows}, (_, row) =>
           screen.getLine(screen.viewportY + row)?.translateToString(true) ?? '') : [];
         return [document.readyState, String(typeof globalThis.Terminal), document.body.className,
           String(bounds?.width ?? 0), String(bounds?.height ?? 0), lines.join('\\n'),
           document.querySelector('#status')?.textContent ?? '',
           document.querySelector('#formula')?.textContent?.slice(0, 128) ?? '',
           document.querySelector('#formula')?.getAttribute('aria-label') ?? ''];
       })()
       """)
      guard let probe = value as? [String], probe.count == 9 else { continue }
      renderer = "\(web.url?.lastPathComponent ?? "no page") · \(probe.joined(separator: " · "))"
      if probe[1] == "function", probe[2].contains("terminal-mode"), probe[8] == "Terminal output",
         web.window != nil, web.bounds.width > 0, web.bounds.height > 0,
         (Double(probe[3]) ?? 0) > 0, (Double(probe[4]) ?? 0) > 0,
         probe[5].split(whereSeparator: \.isNewline).contains(where: { $0.trimmingCharacters(in: .whitespaces) == expected }) {
       outputWebView = web
       return true
      }
     }
     return false
    }
   } catch {
    XCTFail(renderer)
    throw error
   }
   let after = try await terminalCount()
   XCTAssertEqual(after, count + 1)
   XCTAssertTrue(try XCTUnwrap(views().compactMap { $0 as? NSTextView }.first { exactUTF8($0.string, source) }) === text)
   XCTAssertEqual(buffer.sourceEditorState?()?.identity, state.identity)
   XCTAssertEqual(scroll.convert(scroll.bounds, to: host), frame)
   XCTAssertEqual(text.selectedRange(), originalSelection); XCTAssertFalse(text.isEditable)
   XCTAssertEqual(agent.prompt, "Retained response draft"); XCTAssertEqual(try Data(contentsOf: record), original)
   let web = try XCTUnwrap(outputWebView)
   try dismiss(try XCTUnwrap(web.window))
   try await wait("The Run terminal popover stayed open after Escape") { web.window?.isVisible != true }
  }
 }

 func testMountedRecordedUserAndAssistantFencesPauseAndStepInTheirCards() async throws {
  guard let origin = ProcessInfo.processInfo.environment["HICKORY_RECORDED_FENCE_ORIGIN"] else {
   throw XCTSkip("Use the owned recorded-message fixture with debugpy available")
  }
  _ = NSApplication.shared
  NSApp.setActivationPolicy(.regular)
  let workspace = Workspace()
  workspace.address = origin
  let connected = await workspace.connect()
  XCTAssertTrue(connected, workspace.status)
  let debug = try XCTUnwrap(workspace.debugger)
  workspace.showUtility(.agent)
  let host = NSHostingView(rootView: WorkspaceView(workspace: workspace))
  let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 1024, height: 840), styleMask: [.titled], backing: .buffered, defer: false)
  window.contentView = host
  window.makeKeyAndOrderFront(nil)
  defer { debug.close(); workspace.editorRecovery.close(); window.orderOut(nil); window.contentView = nil }
  func wait(_ message: String, ready: () -> Bool) async throws {
   let clock = ContinuousClock(), deadline = clock.now + .seconds(45)
   while clock.now < deadline {
    host.layoutSubtreeIfNeeded()
    if ready() { return }
    try await Task.sleep(for: .milliseconds(25))
   }
   throw EngineError(message: message + " · " + debug.message)
  }
  func nativeEditor(_ view: NSView, source: String) -> NSTextView? {
   if let editor = view as? NSTextView, exactUTF8(editor.string, source) { return editor }
   return view.subviews.lazy.compactMap { nativeEditor($0, source: source) }.first
  }
  let turnID = "recorded-fence-host", agent = workspace.agent
  try await wait("Agent did not mount its retained turn") { agent.turns.contains { $0.id == turnID } }
  let turn = try XCTUnwrap(agent.turns.first { $0.id == turnID })
  var sent: [[String: Any]] = []
  debug.observeRequestForTest = { sent.append($0) }
  for (side, content, result) in [(RecordedMessageSide.user, turn.prompt, "42"), (.assistant, try XCTUnwrap(turn.answer), "53")] {
   let buffer = agent.document("\(turnID)-\(side.bufferSuffix)", content)
   try await wait("Recorded \(side.rawValue) editor did not mount") {
    nativeEditor(host, source: content) != nil && buffer.inlineMarkup.debug.recorded[2] != nil
   }
   let editor = try XCTUnwrap(nativeEditor(host, source: content))
   let identity = try XCTUnwrap(buffer.sourceEditorState?()?.identity)
   XCTAssertFalse(editor.isEditable)
   XCTAssertTrue(agent.owns(buffer, turn: turnID, side: side))
   let fence = try XCTUnwrap(buffer.inlineMarkup.debug.recorded[2])
   let sibling = try XCTUnwrap(buffer.inlineMarkup.debug.recorded[7])
   let prior = sent.count
   debug.toggle(buffer, line: fence.opening + 1)
   debug.toggle(buffer, line: sibling.opening + 1)
   XCTAssertEqual(sent.count, prior)
   let source = NativeRecordedMessageSource(turn: turnID, side: side, workspace: workspace.sourceRootAddress) { agent.owns(buffer, turn: turnID, side: side) }
   await debug.prepareRecordedFence(buffer, source: source, fence: fence)
   try await wait("\(side.rawValue) Debug did not reach its first authored line") { debug.status == "paused" || debug.status == "failed" }
   XCTAssertEqual(debug.status, "paused", debug.message)
   XCTAssertEqual(debug.line(in: buffer.file.path), 3)
   XCTAssertTrue(debug.sourceMatches(buffer))
   XCTAssertEqual((sent.dropFirst(prior).first?["breakpoints"] as? [[String: Any]])?.compactMap { $0["line"] as? Int }, [3])
   debug.step("over")
   try await wait("\(side.rawValue) Step did not reach its next authored line") { debug.line(in: buffer.file.path) == 4 }
   debug.evaluate("value + 1", watch: true)
   try await wait("\(side.rawValue) Watch was unavailable") { debug.values["value + 1"] == result }
   XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
   XCTAssertTrue(nativeEditor(host, source: content) === editor)
   XCTAssertFalse(buffer.dirty)
   debug.step("continue")
   try await wait("\(side.rawValue) Debug did not finish with its own output") {
    debug.status == "finished" && debug.output.map(\.text).joined().contains(result)
   }
   XCTAssertTrue(exactUTF8(buffer.text, content))
  }
  XCTAssertEqual(sent.filter { $0["op"] as? String == "start" }.count, 2)
 }

 func testLiveOwnedDeepStackAndEditedBreakpointRetention()async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_DEBUG_RETENTION_ORIGIN"],let root=ProcessInfo.processInfo.environment["HICKORY_DEBUG_RETENTION_WORKSPACE"] else{throw XCTSkip("Use the maintained owned debugger-retention fixture.")}
  let directory=URL(fileURLWithPath:root);guard try String(contentsOf:directory.appendingPathComponent(".hickory-debug-retention-fixture"),encoding:.utf8)=="owned-debug-retention-v1\n" else{throw EngineError(message:"Owned fixture required.")}
  _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);let e=try Engine(address:origin),w=Workspace();w.engine=e;let listing=try await e.files();w.rootFiles=listing.tree;w.sourceRootAddress=listing.root_path;w.sourceWorkspaceID=listing.workspace_id
  let file=FileNode(name:"deep.py",path:"deep.py",dir:false),b=try await w.loadBuffer(file),source=b.text,debug=NativeDebugger(engine:e);w.debugger=debug;w.layout.activateSource(file.path,in:"document-group");debug.sourceBuffers={Array(w.buffers.values)}
  let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:920,height:620),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
  defer{debug.close();w.sourceTestIndex?.stop();for value in w.buffers.values{value.language?.close()};window.orderOut(nil);window.contentView=nil}
  func native(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{native($0)}.first}
  func wait(_ reason:String,_ predicate:()->Bool)async throws{for _ in 0..<1200{if predicate(){return};try await Task.sleep(for:.milliseconds(20))};throw EngineError(message:reason)}
  try await wait("Native editor did not mount"){native(host) != nil};let text=try XCTUnwrap(native(host)),scroll=try XCTUnwrap(text.enclosingScrollView),editor=try XCTUnwrap(b.sourceEditorState?()).identity,geometry=scroll.convert(scroll.bounds,to:host)
  let line=try XCTUnwrap(source.components(separatedBy:"\n").firstIndex{$0.contains("# deep-stop")});debug.toggle(b,line:line,condition:"depth == 0");await debug.prepare(b);try await wait("Deep program did not pause: \(debug.message)"){debug.status=="paused"||debug.status=="failed"};XCTAssertEqual(debug.status,"paused",debug.message);XCTAssertGreaterThanOrEqual(debug.frames.count,131)
  let frame=try XCTUnwrap(debug.frames.dropFirst(100).first);debug.select(frame,snapshot:debug.frameSnapshot);debug.readFrameLocals();try await wait("Deep caller locals unavailable"){!debug.frameBusy};debug.evaluate("depth");try await wait("Deep caller evaluation unavailable: \(debug.evaluationNotice)"){debug.values["depth"] != nil};XCTAssertEqual(debug.values["depth"],"100");XCTAssertEqual(b.sourceEditorState?()?.identity,editor);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry)
  var requests:[[String:Any]]=[];debug.observeRequestForTest={requests.append($0)};text.setSelectedRange(NSRange(location:text.string.utf16.count,length:0));text.insertText("# retained unsaved breakpoint edit\n",replacementRange:text.selectedRange());let edited=b.text,selection=text.selectedRange();debug.sourceChanged(b);debug.toggle(b,line:line,condition:"depth <= 0");XCTAssertFalse(requests.contains{["breakpoints","source_breakpoints"].contains($0["op"] as? String ?? "")});XCTAssertTrue(debug.breakpointMarkers(b).isEmpty);XCTAssertTrue(debug.retainedBreakpointReading(file.path).contains{$0.contains("depth == 0")});XCTAssertTrue(debug.retainedBreakpointReading(file.path).contains{$0.contains("depth <= 0")});XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(file.path),encoding:.utf8),source);XCTAssertEqual(text.selectedRange(),selection)
  debug.stop();try await wait("Deep session did not stop"){!debug.active};await b.save(using:e);XCTAssertFalse(b.dirty,b.error);XCTAssertEqual(b.text,edited);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(file.path),encoding:.utf8),edited)
  await debug.prepare(b);try await wait("Edited deep program did not pause: \(debug.message)"){debug.status=="paused"||debug.status=="failed"};XCTAssertEqual(debug.status,"paused",debug.message);XCTAssertGreaterThanOrEqual(debug.frames.count,131);XCTAssertTrue(debug.breakpoints[file.path]?.contains{$0["condition"] as? String=="depth <= 0"}==true);XCTAssertTrue(debug.breakpointMarkers(b).contains{$0["state"] as? String=="bound"});debug.stop();try await wait("Edited deep session did not stop"){!debug.active}
  XCTAssertEqual(b.sourceEditorState?()?.identity,editor);XCTAssertTrue(native(host) === text);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);try XCTUnwrap(text.undoManager).undo();XCTAssertEqual(b.text,source);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(file.path),encoding:.utf8),edited)
 }
 #endif
}
</ui:copy>
## Read the selected declared stage in the mounted Apple gutter

The saved fixture changes `old` to `middle` in one declaration and `middle` to `final` in a later declaration. Click the first line's native gutter and open the affected output in its floating comparison. Read the rendered comparison from its own view so the later value in the adjacent message cannot satisfy the assertion. Keep the source selection and editor frame intact, and compare the archived session and working file bytes after reading.

<ui:copy id="recorded-construction-apple-view-test">
#if os(macOS)
extension DebuggerRetentionTests {
 private func clickRecordedControl(at point: NSPoint, in window: NSWindow) throws {
  func mouse(_ type: NSEvent.EventType) throws -> NSEvent {
   try XCTUnwrap(NSEvent.mouseEvent(with: type, location: point, modifierFlags: [],
    timestamp: ProcessInfo.processInfo.systemUptime, windowNumber: window.windowNumber, context: nil,
    eventNumber: 0, clickCount: 1, pressure: type == .leftMouseDown ? 1 : 0))
  }
  // Supply the paired release for native controls that track mouse-down synchronously.
  NSApp.postEvent(try mouse(.leftMouseUp), atStart: false)
  window.sendEvent(try mouse(.leftMouseDown))
  if let release = NSApp.nextEvent(matching: .leftMouseUp, until: .distantPast, inMode: .default, dequeue: true) {
   window.sendEvent(release)
  }
 }

 func testMountedRecordedConstructionShowsOnlySelectedStage() async throws {
  guard let origin = ProcessInfo.processInfo.environment["HICKORY_RECORDED_CONSTRUCTION_ORIGIN"] else {
   throw XCTSkip("Use the owned recorded construction fixture.")
  }
  _ = NSApplication.shared
  NSApp.setActivationPolicy(.regular)
  let workspace = Workspace()
  workspace.address = origin
  guard await workspace.connect() else { throw EngineError(message: workspace.status) }
  let root = URL(fileURLWithPath: try XCTUnwrap(workspace.sourceRootAddress))
  let record = root.appendingPathComponent("sessions/agent.md")
  let output = root.appendingPathComponent("materialized.txt")
  let originalRecord = try Data(contentsOf: record), originalOutput = try Data(contentsOf: output)
  XCTAssertEqual(originalOutput, Data("Independent working file\n".utf8))
  workspace.showUtility(.agent)
  let host = NSHostingView(rootView: WorkspaceView(workspace: workspace))
  let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 1024, height: 900),
                        styleMask: [.titled], backing: .buffered, defer: false)
  window.contentView = host
  window.makeKeyAndOrderFront(nil)
  NSApp.activate()
  defer { workspace.debugger?.close(); workspace.editorRecovery.close(); window.orderOut(nil); window.contentView = nil }
  func wait(_ message: @autoclosure () -> String, ready: () -> Bool) async throws {
   let clock = ContinuousClock(), deadline = clock.now + .seconds(20)
   while clock.now < deadline {
    host.layoutSubtreeIfNeeded()
    if ready() { return }
    try await Task.sleep(for: .milliseconds(100))
   }
   let failure = message()
   XCTFail(failure)
   throw EngineError(message: failure)
  }
  func editor(in view: NSView, source: String) -> NSTextView? {
   if let text = view as? NSTextView, exactUTF8(text.string, source) { return text }
   return view.subviews.lazy.compactMap { editor(in: $0, source: source) }.first
  }
  func visibleWindows() -> [NSWindow] {
   var seen = Set<ObjectIdentifier>()
   return (NSApp.windows + (window.childWindows ?? [])).filter { $0.isVisible && seen.insert(ObjectIdentifier($0)).inserted }
  }
  func renderedText(_ view: NSView, phase: String) -> [(text: String, box: CGRect)] {
   view.layoutSubtreeIfNeeded()
   view.displayIfNeeded()
   let bounds = view.bounds
   guard bounds.width > 0, bounds.height > 0,
         let bitmap = view.bitmapImageRepForCachingDisplay(in: bounds) else { return [] }
   view.cacheDisplay(in: bounds, to: bitmap)
   // The popover material is supplied by its window, outside the view capture.
   guard let raster = bitmap.cgImage,
         let composite = CGContext(data: nil, width: raster.width, height: raster.height,
          bitsPerComponent: 8, bytesPerRow: 0, space: CGColorSpaceCreateDeviceRGB(),
          bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue) else { return [] }
   var background = NSColor.white
   view.effectiveAppearance.performAsCurrentDrawingAppearance {
    background = NSColor.windowBackgroundColor.usingColorSpace(.deviceRGB) ?? .white
   }
   let area = CGRect(x: 0, y: 0, width: CGFloat(raster.width), height: CGFloat(raster.height))
   composite.setFillColor(background.cgColor)
   composite.fill(area)
   composite.draw(raster, in: area)
   guard let pixels = composite.makeImage() else { return [] }
   if let directory = ProcessInfo.processInfo.environment["HICKORY_RECORDED_CONSTRUCTION_IMAGE_DIR"],
      let image = NSBitmapImageRep(cgImage: pixels).representation(using: .png, properties: [:]) {
    let path = URL(fileURLWithPath: directory).appendingPathComponent("\(phase)-\(view.window?.windowNumber ?? 0).png")
    try? image.write(to: path, options: .atomic)
   }
   let reading = VNRecognizeTextRequest()
   reading.recognitionLevel = .accurate
   reading.usesLanguageCorrection = false
   reading.recognitionLanguages = ["en-US"]
   guard (try? VNImageRequestHandler(cgImage: pixels, options: [:]).perform([reading])) != nil else { return [] }
   return (reading.results ?? []).compactMap { item in
    item.topCandidates(1).first.map { ($0.string.trimmingCharacters(in: .whitespacesAndNewlines), item.boundingBox) }
   }
  }

  let id = "declared-construction", agent = workspace.agent, opener = 2
  try await wait("The persisted declared turn did not mount") { agent.turns.contains { $0.id == id } }
  let turn = try XCTUnwrap(agent.turns.first { $0.id == id })
  let source = try XCTUnwrap(turn.answer)
  XCTAssertTrue(source.contains("+final"))
  let buffer = agent.document("\(id)-agent", source)
  try await wait("The declared gutter did not mount. Scanned \(buffer.inlineMarkup.debug.recordedConstructions.sorted()), visible \(buffer.visibleDebugLines.map(\.line)), editor \(editor(in: host, source: source) != nil)") {
   editor(in: host, source: source) != nil && buffer.inlineMarkup.debug.recordedConstructions.contains(opener)
     && buffer.visibleDebugLines.contains { $0.line == opener }
  }
  let text = try XCTUnwrap(editor(in: host, source: source)), scroll = try XCTUnwrap(text.enclosingScrollView)
  let selection = NSRange(location: 2, length: 4)
  text.setSelectedRange(selection)
  let state = try XCTUnwrap(buffer.sourceEditorState?()), frame = scroll.convert(scroll.bounds, to: host)
  let row = try XCTUnwrap(buffer.visibleDebugLines.first { $0.line == opener })
  let baselineWindows = Set(visibleWindows().map(ObjectIdentifier.init))
  let y = scroll.isFlipped ? scroll.bounds.minY + row.y + 9 : scroll.bounds.maxY - row.y - 9
  // The selected declaration is the only action in this row's first gutter slot.
  let gutter = scroll.convert(NSPoint(x: scroll.bounds.maxX + 40 + 4 + 11, y: y), to: nil)
  try clickRecordedControl(at: gutter, in: window)

  var comparison: NSView?, outputLabel: (text: String, box: CGRect)?, currentLabels: [String] = []
  try await wait("The selected declared comparison did not show its output. Visible labels: \(currentLabels)") {
   currentLabels = []
   for candidate in visibleWindows() where !baselineWindows.contains(ObjectIdentifier(candidate)) {
    guard let content = candidate.contentView else { continue }
    let rendered = renderedText(content, phase: "closed")
    let labels = rendered.map(\.text)
    currentLabels += ["Window \(candidate.windowNumber) · \(content.bounds.size)"] + labels
    if labels.contains("Recorded construction patch"),
       let label = rendered.first(where: { $0.text.hasSuffix("materialized.txt") }) {
     comparison = content; outputLabel = label
     return true
    }
   }
   return false
  }
  let panel = try XCTUnwrap(comparison), label = try XCTUnwrap(outputLabel)
  // Recognition can include the disclosure chevron in the same text bounds.
  let chevronOffset: CGFloat = label.text == "materialized.txt" ? -8 : 3
  let inPanel = NSPoint(x: panel.bounds.minX + label.box.minX * panel.bounds.width + chevronOffset,
                       y: panel.bounds.minY + (panel.isFlipped ? 1 - label.box.midY : label.box.midY) * panel.bounds.height)
  try clickRecordedControl(at: panel.convert(inPanel, to: nil), in: try XCTUnwrap(panel.window))
  try await wait("The rendered selected stage did not show old above middle. Visible labels: \(currentLabels)") {
   let reading = renderedText(panel, phase: "expanded")
   currentLabels = reading.map(\.text)
   guard let before = reading.first(where: { $0.text == "Before" }),
         let old = reading.first(where: { $0.text == "old" }),
         let after = reading.first(where: { $0.text == "After" }),
         let middle = reading.first(where: { $0.text == "middle" }) else { return false }
   return before.box.midY > old.box.midY && old.box.midY > after.box.midY
     && after.box.midY > middle.box.midY && currentLabels.contains("Close")
  }
  XCTAssertFalse(currentLabels.contains { $0.localizedCaseInsensitiveContains("final") })
  XCTAssertFalse(currentLabels.contains { label in
   ["Apply", "Save", "Write", "Stage", "Undo"].contains { label.localizedCaseInsensitiveContains($0) }
  }, currentLabels.joined(separator: ", "))
  XCTAssertTrue(editor(in: host, source: source) === text)
  XCTAssertEqual(buffer.sourceEditorState?()?.identity, state.identity)
  XCTAssertEqual(scroll.convert(scroll.bounds, to: host), frame)
  XCTAssertEqual(text.selectedRange(), selection); XCTAssertFalse(text.isEditable)
  XCTAssertEqual(try Data(contentsOf: record), originalRecord)
  XCTAssertEqual(try Data(contentsOf: output), originalOutput)
 }
}
#endif
</ui:copy>
<ui:file path="apps/apple-simple/Tests/DebuggerRetentionTests.swift"><ui:paste select="#debugger-retention-apple-tests"/><ui:paste select="#recorded-construction-apple-view-test"/></ui:file>

<ui:copy id="debugger-retention-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import org.json.JSONArray
import kotlinx.coroutines.*
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class DebuggerRetentionTest {
 @Test fun completeBreakpointReceiptsAndArchiveReadings(){
  val model=NativeBreakpointModel();val source=NativeSourceSnapshot("owned","main.py","first = 1\nsecond = 2\n",true,true);val hash="a".repeat(64);model.toggle(source,0,"first == 1");val intent=model.toggle(source,1,"second == 2");model.stamp(source.identity,intent.revision,hash);model.request(source,"session","current")
  fun row(line:Any,state:Any="bound")=JSONObject().put("line",line).put("state",state)
  val good=listOf(row(0),row(1,"pending"));val bad=listOf(emptyList(),listOf(good[0]),listOf(good[0],good[0]),listOf(good[0],row(2)),listOf(good[0],row(1,"invented")),listOf(good[0],row(1).put("moved_to",999)),listOf(good[0],row(1).put("message",JSONObject().put("foreign",true))),listOf(good[0],row(true)))
  for(rows in bad){assertFalse(model.admit(source,JSONObject().put("breakpoints",JSONArray(rows)),"session","current",hash));assertTrue(model.receipts.isEmpty());assertEquals(2,model.intents.getValue(source.identity).marks.size)}
  assertTrue(model.admit(source,JSONObject().put("breakpoints",JSONArray(good)),"session","current",hash));assertEquals("first == 1",model.rows(source,"session",hash).first().getString("condition"))
  val changed=source.copy(text="# changed\n"+source.text,saved=false);assertTrue(model.rows(changed,"session",hash).isEmpty());model.toggle(changed,1,"first >= 1");val reading=model.readingPage(changed,0);assertTrue(reading.rows.any{it.historical&&it.condition=="first == 1"});assertTrue(reading.rows.any{!it.historical&&it.condition=="first >= 1"});assertTrue(model.rows(changed,"session",hash).isEmpty());model.discard(changed.identity);assertTrue(model.readingPage(changed,0).rows.any{it.condition=="first >= 1"})
 }
 @Test fun retainedIntentionPagesDoNotExposeMutableModelRows(){val model=NativeBreakpointModel();val source=NativeSourceSnapshot("owned","main.py",(0..200).joinToString("\n"){"line $it"},true,true);for(line in 0..129)model.toggle(source,line,"test $line");val first=model.readingPage(source,0);val second=model.readingPage(source,1);val third=model.readingPage(source,999);assertEquals(130,first.total);assertEquals(64,first.rows.size);assertEquals(64,second.rows.size);assertEquals(2,third.page);assertEquals(listOf(128,129),third.rows.map{it.line});assertEquals(0,model.readingPage(source,-1).page)}
 @Test fun liveOwnedDeepStackAndEditedBreakpointRetention()=runBlocking {
  val origin=System.getenv("HICKORY_DEBUG_RETENTION_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_DEBUG_RETENTION_WORKSPACE") ?: error("Owned workspace required"));check(java.io.File(root,".hickory-debug-retention-fixture").readText()=="owned-debug-retention-v1\n");val ownedScope=CoroutineScope(SupervisorJob()+Dispatchers.Default);val engine=Engine(origin!!);val w=Workspace(ownedScope);w.engine=engine
  try{val (_,tree)=engine.files();w.rootFiles=tree;val file=tree.first{it.path=="deep.py"};val b=w.loadBuffer(file);w.show(file.path);val source=b.text;val state=b.state;val scroll=b.sourceScroll;val debug=NativeDebugger(engine,ownedScope);w.debugger=debug
   try{
    suspend fun wait(reason:String,predicate:()->Boolean){withTimeout(30000){while(!predicate())delay(20)}}
    val line=source.lines().indexOfFirst{it.contains("# deep-stop")};check(line>=0);debug.toggle(b,line,"depth == 0");debug.prepare(b);wait("Deep program pause"){debug.status=="paused"||debug.status=="failed"};assertEquals(debug.message,"paused",debug.status);assertTrue(debug.frames.size>=131)
    val frame=debug.frames[100];debug.select(frame);debug.readFrameVariables();wait("Deep caller locals"){debug.variables.any{it.optString("name")=="depth"&&it.optString("value")=="100"}};debug.evaluate("depth");wait("Deep caller evaluation"){debug.values["depth"]!=null};assertEquals("100",debug.values["depth"]);assertSame(state,b.state);assertSame(scroll,b.sourceScroll)
    val requests=java.util.concurrent.CopyOnWriteArrayList<JSONObject>();debug.observeRequestForTest={requests.add(it)};assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"# retained unsaved breakpoint edit\n")),b.text));val edited=b.text;val selection=b.state.selection;debug.toggle(b,line,"depth <= 0");delay(100);assertFalse(requests.any{it.optString("op") in listOf("breakpoints","source_breakpoints")});assertTrue(debug.breakpointRows(b).isEmpty());val reading=debug.breakpointReadingPage(b,0);assertTrue(reading.rows.any{it.condition=="depth == 0"});assertTrue(reading.rows.any{it.condition=="depth <= 0"});assertEquals(selection,b.state.selection);assertEquals(source,java.io.File(root,file.path).readText())
    debug.stop();wait("Stopped deep session"){!debug.active};b.save(engine);assertFalse(b.error,b.dirty);assertEquals(edited,java.io.File(root,file.path).readText());debug.prepare(b);wait("Edited deep program pause"){debug.status=="paused"||debug.status=="failed"};assertEquals(debug.message,"paused",debug.status);assertTrue(debug.frames.size>=131);assertTrue(debug.breakpointRows(b).any{it.optString("state")=="bound"&&it.optString("condition")=="depth <= 0"});debug.stop();wait("Stopped edited session"){!debug.active};assertSame(state,b.state);assertSame(scroll,b.sourceScroll);assertTrue(b.snippets.undoSource());assertEquals(source,b.text);assertEquals(edited,java.io.File(root,file.path).readText())
   }finally{debug.close()}
  }finally{w.buffers.values.forEach{it.language?.close()};ownedScope.cancel()}
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/DebuggerRetentionTest.kt"><ui:paste select="#debugger-retention-android-tests"/></ui:file>

<ui:copy id="debugger-retention-browser-tests">
import {act,cleanup,fireEvent,render,renderHook,screen,waitFor} from '@testing-library/react';
import {afterEach,beforeAll,expect,it} from 'vitest';
import {DebugStrip,breakpointReadingPage,type DebugStripProps} from './DebugStrip';
import {DebugBreakpointModel,type DebugSourceReading} from './SourceGuards';
import {useDebuggerOver} from './useDebugger';
afterEach(cleanup);beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});});
const base=(extra:Partial<DebugStripProps>={}):DebugStripProps=>({status:'paused',program:'main.py',message:null,capabilities:null,exceptionFilters:[],onToggleExceptionFilter:()=>{},frames:[],selectedFrame:null,watches:[],onSelectFrame:()=>{},onStep:()=>{},onJumpHere:()=>{},onStart:()=>{},onStop:()=>{},onAddWatch:()=>{},onRemoveWatch:()=>{},...extra});
it('keeps old conditions readable after explicit placement on current edited source',()=>{const model=new DebugBreakpointModel(),identity={},old:DebugSourceReading={uri:'hick:///main.py',workspace:'owned',namespace:'owned',identity,text:'old\n',saved:true,current:()=>true};model.change(old.uri,old,()=>[{line:0,condition:'old condition'}]);model.change(old.uri,{...old,text:'# changed\nold\n',saved:false},()=>[{line:1,condition:'new condition'}]);expect(model.archived[0].marks[0].condition).toBe('old condition');expect(model.desired(old.uri)[0].condition).toBe('new condition');expect(model.rows(old.uri,{...old,text:'# changed\nold\n',saved:false},'session','a'.repeat(64))).toEqual([]);const groups=[...model.archived.map(row=>({...row,historical:true})),{source:old.uri,namespace:old.namespace,marks:model.desired(old.uri)}];render(<DebugStrip {...base({breakpointIntentions:groups})}/>);fireEvent.click(screen.getByLabelText('Debug options'));fireEvent.click(screen.getByText('Retained breakpoint intentions',{exact:true}));expect(screen.getByLabelText('Retained breakpoint reading').textContent).toContain('old condition');expect(screen.getByLabelText('Retained breakpoint reading').textContent).toContain('new condition');});
it('pages every retained wish without silently dropping later rows',()=>{const groups=[{source:'hick:///main.py',namespace:'owned',historical:true,marks:Array.from({length:130},(_,line)=>({line,condition:'condition '+line}))}];expect(breakpointReadingPage(groups,0).rows).toHaveLength(64);expect(breakpointReadingPage(groups,1).rows).toHaveLength(64);expect(breakpointReadingPage(groups,999).page).toBe(2);expect(breakpointReadingPage(groups,999).rows.join(' ')).toContain('condition 129');render(<DebugStrip {...base({breakpointIntentions:groups})}/>);fireEvent.click(screen.getByLabelText('Debug options'));fireEvent.click(screen.getByText('Retained breakpoint intentions',{exact:true}));fireEvent.click(screen.getByRole('button',{name:'Next breakpoint intentions'}));expect(screen.getByLabelText('Retained breakpoint reading').textContent).toContain('condition 64');fireEvent.click(screen.getByRole('button',{name:'Next breakpoint intentions'}));expect(screen.getByLabelText('Retained breakpoint reading').textContent).toContain('condition 129');expect((screen.getByRole('button',{name:'Next breakpoint intentions'}) as HTMLButtonElement).disabled).toBe(true);});
it('refreshes a saved current reading after source capability changes without rebinding runtime coordinates',async()=>{let version=0,notify=()=>{};const identity={},capability={read:(uri:string)=>{const captured=version;return {uri,workspace:'owned',namespace:'owned',identity,text:'one\ntwo\n',saved:true,current:()=>version===captured};},subscribe:(listener:()=>void)=>{notify=listener;return ()=>{};}};const hook=renderHook(()=>useDebuggerOver(null,'main.py',[],'new',undefined,undefined,capability));act(()=>hook.result.current.toggleBreakpoint(0));expect(hook.result.current.breakpoints).toHaveLength(1);act(()=>{version++;notify();});await waitFor(()=>expect(hook.result.current.breakpoints).toHaveLength(1));expect(hook.result.current.breakpoints[0].state).toBe('pending');hook.unmount();});
it('rejects an obsolete asynchronous capability reply instead of replacing a newer reading',async()=>{let version=0,notify=()=>{},defer=false;const identity={},pending:{reading:DebugSourceReading;resolve:(value:DebugSourceReading)=>void}[]=[];const capability={read:(uri:string):DebugSourceReading|Promise<DebugSourceReading>=>{const captured=version,reading={uri,workspace:'owned',namespace:'owned',identity,text:'one\ntwo\n',saved:true,current:()=>version===captured};if(!defer)return reading;return new Promise(resolve=>pending.push({reading,resolve}));},subscribe:(listener:()=>void)=>{notify=listener;return ()=>{};}};const hook=renderHook(()=>useDebuggerOver(null,'main.py',[],'new',undefined,undefined,capability));act(()=>hook.result.current.toggleBreakpoint(0));defer=true;act(()=>{version++;notify();});act(()=>{version++;notify();});expect(pending).toHaveLength(2);await act(async()=>pending[1].resolve(pending[1].reading));await waitFor(()=>expect(hook.result.current.breakpoints).toHaveLength(1));await act(async()=>pending[0].resolve(pending[0].reading));expect(hook.result.current.breakpoints).toHaveLength(1);expect(hook.result.current.breakpoints[0].state).toBe('pending');hook.unmount();});
</ui:copy>
<ui:file path="apps/web/src/debug/BreakpointRetention.test.tsx"><ui:paste select="#debugger-retention-browser-tests"/></ui:file>

## Follow a real stack beyond the first two pages

The owned debugpy fixture recurses 130 times. Each host must select frame 100 and read its own `depth`, then edit the same retained source editor. New local breakpoint wishes preserve earlier conditions without emitting a setter into the obsolete runtime source. Explicit Save and a fresh launch bind only the current wishes. Source Undo after stopping changes the draft, never the saved file. The browser drives existing gutter, condition, stack selector, Save and debug controls; Apple mounts the actual text editor, and Android retains its real TextFieldState and scroll objects. Native pointer, device IME and assistive-technology acceptance remain unverified.

Focused execution on 2026-11-17 passed the browser’s 39 debugger/receipt/reading cases, three Apple and three Android retention/preparation cases, and one exact owned real deep-stack/retention flow per host with no skips. Browser acceptance drove the rendered gutter condition editor and floating stack/reading controls. Apple mounted its actual NSTextView; Android retained its actual TextFieldState and scroll objects while invoking the native controller. These executions establish their stated usable paths and failure guards, while native physical pointer, device input and assistive-technology interaction remain unexecuted. The iOS shared-source build is recorded separately by the integration owner.

<ui:copy id="debugger-retention-owned-runner">
#!/usr/bin/env python3
"""Owned deep-stack and source-edit breakpoint retention acceptance for simple hosts."""
import argparse,importlib.util,json,os,re,signal,socket,subprocess,tempfile,time,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SOURCE="def descend(depth):\n    token = depth\n    if depth == 0:\n        marker = 41  # deep-stop\n        return marker\n    return descend(depth - 1)\n\ndescend(130)\n"
EDIT="# retained unsaved breakpoint edit\n"
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['browser','apple','android']);parser.add_argument('--skip-build',action='store_true');parser.add_argument('--engine',type=Path,default=ROOT/'target/source-edit-focus/debug/examples/engine_client');parser.add_argument('--venv',type=Path,default=ROOT/'target/thread-debug-env');args=parser.parse_args()
 def interrupted(number,_frame):raise SystemExit(128+number)
 signal.signal(signal.SIGTERM,interrupted)
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 python=args.venv.resolve()/'bin/python';assert python.exists(),'An owned prepared debugpy environment is required; use the native-debug fixture setup.'
 def wait(read,reason):
  deadline=time.monotonic()+40
  while time.monotonic()<deadline:
   if read():return
   time.sleep(.03)
  raise RuntimeError(reason)
 with tempfile.TemporaryDirectory(prefix='hickory-debug-retention-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.git').mkdir();(root/'.venv').symlink_to(args.venv.resolve(),target_is_directory=True);(root/'.hickory-debug-retention-fixture').write_text('owned-debug-retention-v1\n');(root/'deep.py').write_text(SOURCE);(root/'project.md').write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources=""/></h:doc>');(root/'.hick-launch.json').write_text(json.dumps({'version':1,'profiles':[{'id':'owned-deep','name':'Owned deep stack','program':'deep.py','default':True}]}))
  env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   engine=fixture.start([str(args.engine.resolve()),str(root),'window',f'http://127.0.0.1:{port}'],env=env);processes.append(engine);origin=fixture.origin(engine)
   env.update(HICKORY_DEBUG_RETENTION_ORIGIN=origin,HICKORY_DEBUG_RETENTION_WORKSPACE=str(root))
   if args.host=='browser':
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path='deep.py');driver.act('remember-editor',path='deep.py');geometry=driver.act('editor-size',path='deep.py')
    line=next(index+1 for index,text in enumerate(SOURCE.splitlines()) if '# deep-stop' in text);driver.act('debug-breakpoint',line=line);driver.act('debug-breakpoint-condition',path='deep.py',line=line,condition='depth == 0');driver.act('debug-start',path='deep.py');wait(lambda:any('paused' in text.lower() for text in driver.act('debug-state')),'Deep program did not pause')
    selected=driver.act('debug-frame-index',index=100);assert selected['count']>=131,selected;assert driver.act('debug-watch',expression='depth',expected='100')=='100';assert driver.act('editor-is-retained',path='deep.py') and driver.act('editor-size',path='deep.py')==geometry
    before=[row for row in driver.act('debug-wire') if row.get('op') in ['breakpoints','source_breakpoints']];driver.act('source-tests-edit',path='deep.py',text=EDIT);selection=driver.act('construction-source',path='deep.py',selection=True);driver.act('debug-breakpoint',line=line);reading=driver.act('debug-breakpoint-reading');assert 'Earlier source' in reading and 'depth == 0' in reading,reading;after=[row for row in driver.act('debug-wire') if row.get('op') in ['breakpoints','source_breakpoints']];assert len(after)==len(before),(before,after);assert (root/'deep.py').read_text()==SOURCE;assert driver.act('construction-source',path='deep.py',selection=True)==selection
    driver.act('debug-stop');wait(lambda:not any(any(word in text.lower() for word in ['paused','running','starting','unconfirmed','stopping']) for text in driver.act('debug-state')),'Deep session did not stop');driver.act('file-command',command='Save');wait(lambda:(root/'deep.py').read_text()==SOURCE+EDIT,'Edited source Save was not confirmed');driver.act('debug-breakpoint-condition',path='deep.py',line=line,condition='depth <= 0');driver.act('debug-start',path='deep.py');wait(lambda:any('paused' in text.lower() for text in driver.act('debug-state')),'Edited deep program did not pause');assert driver.act('debug-frame-index',index=100)['count']>=131;assert driver.act('debug-watch',expression='depth',expected='100')=='100';driver.act('debug-stop');wait(lambda:not any(any(word in text.lower() for word in ['paused','running','starting','unconfirmed','stopping']) for text in driver.act('debug-state')),'Edited deep session did not stop');driver.act('undo',path='deep.py');assert driver.act('construction-source',path='deep.py')==SOURCE;assert (root/'deep.py').read_text()==SOURCE+EDIT;assert driver.act('editor-is-retained',path='deep.py') and driver.act('editor-size',path='deep.py')==geometry;assert not driver.act('debug-diagnostics')['errors']
   else:
    if args.host=='apple':
     name='testLiveOwnedDeepStackAndEditedBreakpointRetention';command=['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','DebuggerRetentionTests.'+name+'$']+(['--skip-build'] if args.skip_build else [])
    else:
     name='liveOwnedDeepStackAndEditedBreakpointRetention';command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.DebuggerRetentionTest.'+name,'--rerun']
    log=ROOT/'target'/f'debugger-retention-{args.host}.log'
    with log.open('w') as output:result=subprocess.run(command,cwd=ROOT,env=env,stdout=output,stderr=subprocess.STDOUT)
    if result.returncode:raise RuntimeError(f'{args.host} actual acceptance failed; inspect {log}')
    if args.host=='apple':
     text=log.read_text();assert re.search(re.escape(name)+r'.*passed',text) and 'Executed 1 test, with 0 failures' in text and 'Test skipped' not in text,str(log)
    else:
     report=ROOT/'apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.DebuggerRetentionTest.xml';rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(key) is not None for key in ['skipped','failure','error']),str(report)
    assert (root/'deep.py').read_text()==SOURCE+EDIT
   print(f'PASS {args.host}: actual debugpy stack beyond 100 callers, scoped deep-frame reading, retained edited-source wishes without stale setters, explicit Save and fresh launch, retained editor/Undo, unchanged saved source during draft Undo. Physical input/accessibility acceptance remains unverified.')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-debugger-retention.py"><ui:paste select="#debugger-retention-owned-runner"/></ui:file>
</ui:doc>
