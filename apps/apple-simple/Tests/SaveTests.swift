// Contract: .hickory-source/frontends/contract.md
import XCTest
@testable import HickorySimple

final class ResponseProtocol: URLProtocol {
    static var status = 200
    static var response = Data()
    static var captured: [String: Any] = [:]
    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }
    override func startLoading() {
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
        client?.urlProtocol(self, didLoad: Self.response)
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
