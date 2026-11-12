<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Witness native save refusal and revision payloads

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

A local URLProtocol fixture exercises the actual Swift HTTP encoder and buffer behavior without sending an agent request or writing a workspace. This is a protocol safety test, not a substitute for device UI checks.

<ui:copy id="simple-d0dada29d6-0">
// Contract: .hickory-source/frontends/contract.md
import XCTest
@testable import HickorySimple

final class ResponseProtocol: URLProtocol {
    static var requestedURL: URL?
    static var status = 200
    static var response = Data()
    static var responsesByPath: [String: Data] = [:]
    static var captured: [String: Any] = [:]
    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }
    override func startLoading() {
        Self.requestedURL = request.url
        var body = request.httpBody ?? Data()
        if let stream = request.httpBodyStream {
            stream.open(); defer { stream.close() }
            var bytes = [UInt8](repeating: 0, count: 4096)
            while stream.hasBytesAvailable {
                let count = stream.read(&bytes, maxLength: bytes.count)
                if count <= 0 { break }
                body.append(contentsOf: bytes.prefix(count))
            }
        }
        Self.captured = (try? JSONSerialization.jsonObject(with: body)) as? [String: Any] ?? [:]
        let response = HTTPURLResponse(url: request.url!, statusCode: Self.status, httpVersion: nil, headerFields: ["Content-Type": "application/json"])!
        client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
        client?.urlProtocol(self, didLoad: Self.responsesByPath[request.url!.path] ?? Self.response)
        client?.urlProtocolDidFinishLoading(self)
    }
    override func stopLoading() {}
}
final class SaveTests: XCTestCase {
    @MainActor private func fixture(doc: String? = nil, generated: String? = nil) throws -> (Engine, DocumentBuffer) {
        let configuration = URLSessionConfiguration.ephemeral; configuration.protocolClasses = [ResponseProtocol.self]
        let engine = try Engine(address: "http://fixture.invalid", session: URLSession(configuration: configuration))
        let file = FileNode(name: "notes.txt", path: "notes.txt", dir: false, doc_id: doc, generated_by: generated)
        return (engine, DocumentBuffer(file: file, text: FileText(path: file.path, content: "before", hash: "hash-before")))
    }
    @MainActor func testManagedMarkdownOpensThroughDocumentSource() async throws {
        let (engine, buffer) = try fixture(doc: "managed")
        ResponseProtocol.status = 200
        ResponseProtocol.response = Data(##"{"source":"# Readable source"}"##.utf8)
        let text = try await engine.file(buffer.file)
        XCTAssertEqual(ResponseProtocol.requestedURL?.path, "/api/docs/managed")
        XCTAssertEqual(text.content, "# Readable source")
    }
    @MainActor func testPlainTextOpensWithFileHash() async throws {
        let (engine, buffer) = try fixture()
        ResponseProtocol.status = 200
        ResponseProtocol.response = Data(#"{"path":"notes.txt","content":"Plain text","hash":"plain-hash"}"#.utf8)
        let text = try await engine.file(buffer.file)
        XCTAssertEqual(ResponseProtocol.requestedURL?.path, "/api/file")
        XCTAssertEqual(text.content, "Plain text"); XCTAssertEqual(text.hash, "plain-hash")
    }
    @MainActor func testConflictRetainsEditsAndOriginalBaseline() async throws {
        let (engine, buffer) = try fixture()
        ResponseProtocol.status = 409; ResponseProtocol.response = Data(#"{"error":"changed on disk"}"#.utf8)
        buffer.text = "mine"; await buffer.save(using: engine)
        XCTAssertEqual(ResponseProtocol.captured["base_hash"] as? String, "hash-before")
        XCTAssertEqual(buffer.text, "mine"); XCTAssertEqual(buffer.base, "before"); XCTAssertTrue(buffer.dirty)
        XCTAssertTrue(buffer.error.contains("edits are retained"))
    }
    @MainActor func testLiterateSaveUsesBothExpectations() async throws {
        let (engine, buffer) = try fixture(doc: "doc")
        ResponseProtocol.status = 200; ResponseProtocol.response = Data(#"{"source":"next"}"#.utf8)
        buffer.text = "next"; await buffer.save(using: engine)
        let expected = ResponseProtocol.captured["expected"] as? [String: String]
        XCTAssertEqual(expected, ["source": "before", "disk": "before"]); XCTAssertFalse(buffer.dirty)
    }
    @MainActor func testGeneratedOutputDoesNotDispatchSave() async throws {
        let (engine, buffer) = try fixture(generated: "owner")
        ResponseProtocol.captured = [:]; buffer.text = "replacement"; await buffer.save(using: engine)
        XCTAssertTrue(ResponseProtocol.captured.isEmpty); XCTAssertTrue(buffer.readOnly)
    }
    @MainActor func testTerminalRetainsDraftWhenReopenedAndInputFails() async throws {
        let (engine, _) = try fixture()
        ResponseProtocol.status = 200; ResponseProtocol.response = Data(#"{"id":"pty"}"#.utf8)
        let session = TerminalSession(); session.input = "echo hello"
        await session.open(using: engine); await session.open(using: engine)
        XCTAssertEqual(session.id, "pty"); XCTAssertEqual(session.input, "echo hello")
        ResponseProtocol.status = 409; ResponseProtocol.response = Data(#"{"error":"disconnected"}"#.utf8)
        await session.send(using: engine)
        XCTAssertEqual(ResponseProtocol.captured["data"] as? String, "echo hello\r")
        XCTAssertEqual(session.input, "echo hello"); XCTAssertTrue(session.error.contains("not retried"))
    }

}
</ui:copy>

## Check session identity, launch bytes and control boundaries

These focused witnesses drive the native controller with explicit protocol events, inspect outbound verbs and check pending-stop races. The HTTP witness verifies that literate launch resolution carries the selected generated path plus the entire displayed document. The optional live witness requires a real backend and adapter; it is skipped without HICKORY_NATIVE_DEBUG_ORIGIN and is not a native UI acceptance receipt.

<ui:copy id="simple-apple-debug-witnesses">
final class NativeDebugTests: XCTestCase {
    @MainActor func testOutputIsReadOnlySourceAnchoredAndRevisionBound() throws {
        // Contract: .hickory-source/frontends/contract.md
        let source = "value = 1\nprint(value)\n"
        let buffer = DocumentBuffer(file: FileNode(name: "main.py", path: "main.py", dir: false), text: FileText(path: "main.py", content: source, hash: "base"))
        let debug = NativeDebugger(engine: try Engine(address: "http://fixture.invalid"))
        debug.owner = "main.py"; debug.launchSource = source; debug.status = "starting"
        debug.receive(["event": "started", "doc": "hick:///main.py", "session": "ours", "breakpoints": []])
        debug.receive(["event": "run_output", "session": "old", "events": [["text": "wrong"]]])
        XCTAssertTrue(debug.output.isEmpty)
        debug.receive(["event": "run_output", "session": "ours", "events": [["category": "stdout", "text": "1\n"], ["category": "stderr", "text": "warning\n", "source": "main.py", "line": 1]]])
        XCTAssertTrue(debug.outputPlacements(buffer)[1]?.contains("[stdout] 1") == true)
        XCTAssertTrue(debug.outputPlacements(buffer)[1]?.contains("main.py:2") == true)
        XCTAssertEqual(buffer.text, source); XCTAssertFalse(buffer.dirty)
        buffer.text += "# edited"
        XCTAssertTrue(debug.outputPlacements(buffer).isEmpty)
        let document = "<h:file path=\"main.py\">\nprint(1)\n\nprint(2)\n</h:file>\n"
        let book = DocumentBuffer(file: FileNode(name: "workbook.md", path: "workbook.md", dir: false), text: FileText(path: "workbook.md", content: document, hash: "base"))
        debug.owner = "workbook.md"; debug.launchSource = document; debug.status = "starting"
        debug.receive(["event": "started", "doc": "hick:///workbook.md", "session": "document", "breakpoints": []])
        debug.receive(["event": "run_output", "session": "document", "events": [["category": "stdout", "text": "1\n", "source": "workbook.md", "line": 1]]])
        XCTAssertNotNil(debug.outputPlacements(book)[4], "A blank line must not split the producing file block")
    }
    @MainActor func testRecordedOutputUsesByteSpanAfterUnicodeBlock() {
        let source = "<h:exec>\necho café\n</h:exec>\n"
        let buffer = DocumentBuffer(file: FileNode(name: "workbook.md", path: "workbook.md", dir: false), text: FileText(path: "workbook.md", content: source, hash: "base"))
        buffer.receipts = [InlineReceipt(kind: "exec", span: [0, source.utf8.count - 1], command: nil, status: nil, transcript: [InlineReceiptEvent(kind: "out", data: "café\n")], recorded_output: nil)]
        buffer.receiptSource = source
        XCTAssertEqual(buffer.recordedOutputPlacements(), [2: "[out] café\n"])
        XCTAssertEqual(buffer.text, source)
        buffer.text += "changed"
        XCTAssertTrue(buffer.recordedOutputPlacements().isEmpty)
    }

    @MainActor func testNativeDebugRejectsOtherSessionAndCarriesBoundCondition() throws {
        let debug = NativeDebugger(engine: try Engine(address: "http://fixture.invalid"))
        debug.owner = "main.py"; debug.launchSource = "value = 1\nprint(value)\n"; debug.status = "starting"
        var requests: [[String: Any]] = []; debug.emitForTest = { requests.append($0) }
        debug.toggle("main.py", line: 0, condition: "value > 0")
        debug.receive(["event": "started", "doc": "hick:///other.py", "session": "wrong"])
        XCTAssertNil(debug.sessionID)
        debug.receive(["event": "started", "doc": "hick:///main.py", "session": "ours", "breakpoints": [["line": 0, "moved_to": 1, "state": "bound"]]])
        XCTAssertEqual(debug.breakpoints["main.py"]?.first?["condition"] as? String, "value > 0")
        XCTAssertEqual(debug.breakpoints["main.py"]?.first?["line"] as? Int, 1)
        var revealed = ""; debug.onLocation = { path, line in revealed = "\(path):\(line)" }
        debug.receive(["event": "stopped", "session": "old", "line": 0]); XCTAssertEqual(debug.status, "running")
        debug.receive(["event": "stopped", "session": "ours", "line": NSNull(), "frames": [["id": 7, "name": "helper", "source": "helper.py", "source_line": 2]], "variables": [["name": "value", "value": "1"]]])
        XCTAssertEqual(revealed, "helper.py:2"); XCTAssertEqual(debug.line(in: "helper.py"), 2)
        let helper = DocumentBuffer(file: FileNode(name: "helper.py", path: "helper.py", dir: false), text: FileText(path: "helper.py", content: "before", hash: "base"))
        debug.rememberSource(helper); XCTAssertTrue(debug.sourceMatches(helper))
        helper.text = "saved replacement"; helper.base = helper.text
        XCTAssertFalse(debug.sourceMatches(helper), "Saving changed callee text must not reattach old positional values")
        debug.evaluate("value", watch: true); XCTAssertEqual(requests.last?["frame"] as? Int, 7)
        debug.step("over"); XCTAssertEqual(requests.last?["how"] as? String, "over"); XCTAssertTrue(debug.variables.isEmpty)
        debug.receive(["event": "value", "session": "ours", "expression": "value", "value": "old"]); XCTAssertTrue(debug.values.isEmpty)
        debug.receive(["event": "finished", "session": "ours"]); debug.step("over"); XCTAssertNil(debug.sessionID); XCTAssertEqual(debug.status, "finished")
    }
    @MainActor func testStopWhileStartingStopsTheAcknowledgedSession() throws {
        let debug = NativeDebugger(engine: try Engine(address: "http://fixture.invalid"))
        debug.owner = "main.py"; debug.status = "starting"
        var requests: [[String: Any]] = []; debug.emitForTest = { requests.append($0) }
        debug.stop(); XCTAssertEqual(debug.status, "stopping"); XCTAssertTrue(requests.isEmpty)
        debug.receive(["event": "started", "doc": "hick:///main.py", "session": "late", "breakpoints": []])
        XCTAssertEqual(requests.last?["op"] as? String, "stop"); XCTAssertEqual(requests.last?["session"] as? String, "late")
        debug.receive(["event": "ended", "session": "late"]); XCTAssertEqual(debug.status, "finished")
    }
    @MainActor func testLiterateLaunchUsesDocumentExpectationAndRejectsUnsavedText() async throws {
        let config = URLSessionConfiguration.ephemeral; config.protocolClasses = [ResponseProtocol.self]
        let engine = try Engine(address: "http://fixture.invalid", session: URLSession(configuration: config))
        let debug = NativeDebugger(engine: engine)
        let source = "# Prose\n<h:file path=\"main.py\">\nprint(1)\n</h:file>\n"
        let file = FileNode(name: "source.md", path: "source.md", dir: false, doc_id: "managed")
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: source, hash: "base"))
        ResponseProtocol.status = 200
        ResponseProtocol.response = Data(#"{"profiles":[{"id":"other","name":"Other","can_debug":true,"program":"other.py"},{"id":"python","name":"Python","can_debug":true,"program":"main.py"}],"fingerprint":"current"}"#.utf8)
        defer { ResponseProtocol.responsesByPath = [:] }
        ResponseProtocol.responsesByPath["/api/docs/managed/render"] = try JSONSerialization.data(withJSONObject: ["blocks": [["kind": "file", "path": "main.py", "span": [8, source.utf8.count]]]])
        var requests: [[String: Any]] = []; debug.emitForTest = { requests.append($0) }
        defer { debug.close() }
        await debug.prepare(buffer, line: 2)
        XCTAssertEqual(ResponseProtocol.captured["program"] as? String, "main.py")
        XCTAssertEqual(ResponseProtocol.captured["expected_document"] as? String, source)
        XCTAssertNil(ResponseProtocol.captured["expected_source"])
        XCTAssertEqual(debug.status, "starting"); XCTAssertTrue(debug.profiles.isEmpty)
        XCTAssertEqual((requests.last?["profile"] as? [String: Any])?["id"] as? String, "python")
        debug.close()
        buffer.text += "edited"; await debug.prepare(buffer)
        XCTAssertTrue(debug.message.contains("Save this file")); XCTAssertFalse(debug.sourceMatches(buffer))
    }
    @MainActor func testLiveNativeDebugChannel() async throws {
        guard let origin = ProcessInfo.processInfo.environment["HICKORY_NATIVE_DEBUG_ORIGIN"] else { throw XCTSkip("Set HICKORY_NATIVE_DEBUG_ORIGIN for a real backend with debugpy installed") }
        let engine = try Engine(address: origin), file = FileNode(name: "debug.py", path: "debug.py", dir: false)
        let loaded = try await engine.file(file), buffer = DocumentBuffer(file: file, text: loaded)
        let debug = NativeDebugger(engine: engine); defer { debug.close() }
        debug.toggle(file.path, line: 1); await debug.prepare(buffer)
        for _ in 0..<200 { if debug.status == "paused" || debug.status == "failed" { break }; try await Task.sleep(for: .milliseconds(50)) }
        XCTAssertEqual(debug.status, "paused", debug.message)
        guard debug.status == "paused" else { return }
        debug.step("over")
        for _ in 0..<200 { if debug.status == "paused" || debug.status == "failed" { break }; try await Task.sleep(for: .milliseconds(50)) }
        XCTAssertEqual(debug.status, "paused", debug.message); XCTAssertFalse(debug.variables.isEmpty)
        debug.evaluate("value", watch: true)
        for _ in 0..<100 { if debug.values["value"] != nil { break }; try await Task.sleep(for: .milliseconds(50)) }
        XCTAssertNotNil(debug.values["value"]); XCTAssertEqual(buffer.text, loaded.content); XCTAssertFalse(buffer.dirty)
        debug.stop()
        for _ in 0..<100 { if !debug.active { break }; try await Task.sleep(for: .milliseconds(50)) }
        XCTAssertFalse(debug.active, debug.message)
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Tests/SaveTests.swift"><ui:paste select="#simple-d0dada29d6-0"/><ui:paste select="#simple-apple-debug-witnesses"/></ui:file>

</ui:doc>
