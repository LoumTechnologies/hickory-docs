// Contract: .hickory-source/frontends/contract.md
import XCTest
@testable import HickorySimple

final class ResponseProtocol: URLProtocol {
    static var requestedURL: URL?
    static var beforeResponse: ((URLRequest) -> Void)?
    private static let requestLock = NSLock()
    private static var requests: [URL] = []
    private static var bodies:[String:[[String:Any]]]=[:]
    static func requestBodies(_ path:String)->[[String:Any]]{requestLock.lock();defer{requestLock.unlock()};return bodies[path] ?? []}
    static func resetRequests() { requestLock.lock(); defer { requestLock.unlock() }; requests = [];bodies=[:] }
    static func requestCount(_ path: String) -> Int { requestLock.lock(); defer { requestLock.unlock() }; return requests.filter { $0.path == path }.count }
    static var status = 200
    static var response = Data()
    static var responsesByPath: [String: Data] = [:]
    static var captured: [String: Any] = [:]
    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }
    override func startLoading() {
        Self.requestedURL = request.url
        if let url=request.url { Self.requestLock.lock(); Self.requests.append(url); Self.requestLock.unlock() }
        Self.beforeResponse?(request)
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
        let parsed=(try? JSONSerialization.jsonObject(with:body)) as? [String:Any] ?? [:];Self.captured=parsed
        if let path=request.url?.path{Self.requestLock.lock();Self.bodies[path,default:[]].append(parsed);Self.requestLock.unlock()}
        let response = HTTPURLResponse(url: request.url!, statusCode: Self.status, httpVersion: nil, headerFields: ["Content-Type": "application/json"])!
        client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
        client?.urlProtocol(self, didLoad: Self.responsesByPath[request.url!.path] ?? Self.response)
        client?.urlProtocolDidFinishLoading(self)
    }
    override func stopLoading() {}
}
final class SaveTests: XCTestCase {
    @MainActor func testSharedLanguageWireRoutesRepliesDetachesOneSourceAndReplaysAfterFailure() async throws {
        // Contract: .hickory-source/frontends/contract.md. Own protocol fixture, real native WebSocket.
        #if os(macOS)
        let root=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let process=Process(),pipe=Pipe();process.executableURL=URL(fileURLWithPath:"/usr/bin/env");process.arguments=["python3",root.appendingPathComponent("scripts/simple-language-wire-fixture.py").path];process.standardOutput=pipe;process.standardError=pipe
        try process.run();defer {if process.isRunning {process.terminate()};process.waitUntilExit()}
        let greeting=try XCTUnwrap(JSONSerialization.jsonObject(with:pipe.fileHandleForReading.availableData) as? [String:Any])
        let engine=try Engine(address:try XCTUnwrap(greeting["origin"] as? String)),connection=NativeLanguageConnection(engine:engine)
        defer {connection.close()}
        struct Metrics: Decodable {var connections:Int;var ids:[Int];var documents:[String:String]}
        func metrics() async throws -> Metrics {try await engine.request("GET","/state")}
        func params(_ uri:String)->[String:Any] {["textDocument":["uri":uri]]}
        let a="hick:///a.py",b="hick:///b.py",c="hick:///c.py"
        var reportsA:[String]=[],reportsB:[String]=[],reportsC:[String]=[],failuresA=0,failuresB=0,failuresC=0
        let ownerA=connection.attach(uri:a,source:"first",version:1,receive:{if let uri=($0["params"] as? [String:Any])?["uri"] as? String {reportsA.append(uri)}},failed:{_ in failuresA += 1})
        let ownerB=connection.attach(uri:b,source:"second",version:1,receive:{if let uri=($0["params"] as? [String:Any])?["uri"] as? String {reportsB.append(uri)}},failed:{_ in failuresB += 1})
        _=connection.attach(uri:c,source:"third",version:1,receive:{if let uri=($0["params"] as? [String:Any])?["uri"] as? String {reportsC.append(uri)}},failed:{_ in failuresC += 1})
        async let slow=connection.request("fixture/slow",params:params(a),owner:ownerA)
        async let fast=connection.request("fixture/fast",params:params(b),owner:ownerB)
        let (one,two)=try await(slow,fast)
        XCTAssertEqual((one as? [String:Any])?["uri"] as? String,a);XCTAssertEqual((two as? [String:Any])?["text"] as? String,"second")
        XCTAssertEqual(reportsA,[a]);XCTAssertEqual(reportsB,[b]);XCTAssertEqual(reportsC,[c]);let initial=try await metrics();XCTAssertEqual(initial.connections,1)
        var rejected=false
        let duplicate=connection.attach(uri:a,source:"conflicting draft",version:2,receive:{_ in XCTFail("Duplicate source received a notification")},failed:{_ in rejected=true})
        XCTAssertTrue(rejected);connection.change(owner:duplicate,source:"corrupt",version:3)
        do {_=try await connection.request("fixture/fast",params:params(a),owner:duplicate);XCTFail("Duplicate source borrowed request authority")} catch {}
        let refused=try await connection.request("fixture/askApply",params:params(b),owner:ownerB);XCTAssertEqual((refused as? [String:Any])?["applied"] as? Bool,false)
        let preceding=try await metrics().ids.count
        let blocking=Task {try await connection.request("fixture/block",params:params(a),owner:ownerA)}
        _=try await connection.request("fixture/fast",params:params(b),owner:ownerB)
        let deadline=Date().addingTimeInterval(2)
        while try await metrics().ids.count < preceding+2,Date()<deadline {try await Task.sleep(for:.milliseconds(10))}
        connection.detach(ownerA)
        do {_=try await blocking.value;XCTFail("Detached owner retained a pending request")} catch {}
        let stillAlive=try await connection.request("fixture/fast",params:params(b),owner:ownerB)
        XCTAssertEqual((stillAlive as? [String:Any])?["text"] as? String,"second")
        do {_=try await connection.request("fixture/drop",params:params(b),owner:ownerB);XCTFail("Dropped wire returned a successful reply")} catch {}
        XCTAssertEqual(failuresA,0);XCTAssertEqual(failuresB,1);XCTAssertEqual(failuresC,1)
        connection.change(owner:ownerB,source:"new second",version:2)
        try await Task.sleep(for:.milliseconds(1100))
        let reopened=try await connection.request("fixture/fast",params:params(b),owner:ownerB)
        XCTAssertEqual((reopened as? [String:Any])?["text"] as? String,"new second")
        let final=try await metrics();XCTAssertEqual(final.connections,2);XCTAssertNil(final.documents[a]);XCTAssertEqual(final.documents[b],"new second");XCTAssertEqual(final.documents[c],"third");XCTAssertEqual(Set(final.ids).count,final.ids.count)
        let file=FileNode(name:"outline.py",path:"outline.py",dir:false,language:"python")
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"source",hash:"base"))
        let service=NativeLanguage(engine:engine,buffer:buffer,connection:connection);service.start();defer {service.close()}
        let reading=Task {try await service.readOutline(source:"source")}
        try await Task.sleep(for:.milliseconds(10));buffer.selection=NSRange(location:3,length:0)
        let outline=try await reading.value
        XCTAssertEqual(outline.first?.name,"background");XCTAssertEqual(buffer.selection.location,3);XCTAssertFalse(service.showingSuggestions)
        await service.navigate(.supertypes);XCTAssertEqual(service.locations.first?.name,"supertype")
        await service.navigate(.subtypes);XCTAssertEqual(service.locations.first?.name,"subtype")
        let hierarchy=Task {await service.navigate(.supertypes)};try await Task.sleep(for:.milliseconds(10));buffer.selection=NSRange(location:4,length:0);await hierarchy.value
        XCTAssertTrue(service.locations.isEmpty);XCTAssertTrue(service.locationStatus.contains("changed"));buffer.selection=NSRange(location:3,length:0)
        let stale=Task {try await service.readOutline(source:"source")}
        try await Task.sleep(for:.milliseconds(10));buffer.text="draft"
        do {_=try await stale.value;XCTFail("Source changed during outline read")} catch {}
        XCTAssertEqual(buffer.text,"draft")
        #endif
    }

    @MainActor func testBackgroundSourceAcquisitionSharesReadWithoutActivatingPane() async throws {
        // Contract: .hickory-source/frontends/contract.md
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath=[:];ResponseProtocol.resetRequests()
        ResponseProtocol.response=Data(#"{"path":"background.py","content":"value = 1","hash":"disk"}"#.utf8)
        let arrived = expectation(description: "shared read reached the fixture")
        let joined = expectation(description: "second caller joined the read")
        let gate = DispatchSemaphore(value: 0)
        ResponseProtocol.beforeResponse = { request in
            guard request.url?.query == "path=background.py" else { return }
            arrived.fulfill()
            _ = gate.wait(timeout: .now() + 5)
        }
        let config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self]
        let session=URLSession(configuration:config),workspace=Workspace();workspace.engine=try Engine(address:"http://fixture.invalid",session:session)
        let file=FileNode(name:"background.py",path:"background.py",dir:false,language:"python")
        defer {
            gate.signal()
            ResponseProtocol.beforeResponse = nil
            workspace.buffers.values.forEach { $0.language?.close() }
            session.invalidateAndCancel()
        }
        let first = Task { try await workspace.loadBuffer(file) }
        await fulfillment(of: [arrived], timeout: 3)
        let second = Task {
            joined.fulfill()
            return try await workspace.loadBuffer(file)
        }
        await fulfillment(of: [joined], timeout: 3)
        gate.signal()
        let (a, b) = try await (first.value, second.value)
        XCTAssertTrue(a === b);XCTAssertEqual(ResponseProtocol.requestCount("/api/file"),1)
        XCTAssertEqual(a.text,"value = 1");XCTAssertNil(workspace.active);XCTAssertNotNil(a.language)
        ResponseProtocol.response=Data(#"{"path":"other.py","content":"value = 2","hash":"other-disk"}"#.utf8)
        let other=try await workspace.loadBuffer(FileNode(name:"other.py",path:"other.py",dir:false,language:"python"))
        XCTAssertTrue(a.language?.connection === other.language?.connection);XCTAssertNil(workspace.active)
    }
    @MainActor func testDelayedBackgroundSourceCannotReplaceAvailableDraft() async throws {
        // Contract: .hickory-source/frontends/contract.md
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath=[:];ResponseProtocol.resetRequests()
        ResponseProtocol.response=Data(#"{"path":"background.py","content":"late disk text","hash":"disk"}"#.utf8)
        let arrived=expectation(description:"raw file request"),gate=DispatchSemaphore(value:0)
        ResponseProtocol.beforeResponse={request in if request.url?.path == "/api/file" {arrived.fulfill();_ = gate.wait(timeout:.now()+5)}}
        let config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self]
        let session=URLSession(configuration:config),workspace=Workspace();workspace.engine=try Engine(address:"http://fixture.invalid",session:session)
        let file=FileNode(name:"background.py",path:"background.py",dir:false,language:"python")
        defer {gate.signal();ResponseProtocol.beforeResponse=nil;session.invalidateAndCancel()}
        let read=Task {try await workspace.loadBuffer(file)}
        await fulfillment(of:[arrived],timeout:3)
        let draft=DocumentBuffer(file:file,text:FileText(path:file.path,content:"base",hash:"base"));draft.text="retained draft";workspace.buffers[file.path]=draft
        gate.signal();let loaded=try await read.value
        XCTAssertTrue(loaded === draft);XCTAssertEqual(draft.text,"retained draft");XCTAssertTrue(draft.dirty);XCTAssertNil(workspace.active)
    }

    @MainActor func testDelayedOpenKeepsNewerNativeTabSelection() async throws {
        ResponseProtocol.status = 200
        ResponseProtocol.responsesByPath = [:]
        ResponseProtocol.response = Data(#"{"path":"earlier.txt","content":"saved earlier","hash":"disk"}"#.utf8)
        let arrived = expectation(description: "earlier file request")
        let gate = DispatchSemaphore(value: 0)
        ResponseProtocol.beforeResponse = { request in
            if request.url?.path == "/api/file" {
                arrived.fulfill()
                _ = gate.wait(timeout: .now() + 5)
            }
        }
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [ResponseProtocol.self]
        let session = URLSession(configuration: configuration)
        let workspace = Workspace()
        workspace.engine = try Engine(address: "http://fixture.invalid", session: session)
        defer {
            gate.signal()
            ResponseProtocol.beforeResponse = nil
            workspace.buffers.values.forEach { $0.language?.close() }
            session.invalidateAndCancel()
        }
        let file = FileNode(name: "draft.txt", path: "draft.txt", dir: false)
        let draft = DocumentBuffer(file: file, text: FileText(path: file.path, content: "saved", hash: "saved"))
        draft.text = "unsaved draft"
        draft.selection = NSRange(location: 3, length: 2)
        workspace.buffers[file.path] = draft
        workspace.showSource(file.path)
        workspace.showUtility(.agent)
        let earlier = FileNode(name: "earlier.txt", path: "earlier.txt", dir: false)
        let read = Task { await workspace.open(earlier) }
        await fulfillment(of: [arrived], timeout: 3)
        workspace.show(nativeSourceTab(file.path), in: workspace.focused)
        gate.signal()
        await read.value

        XCTAssertEqual(workspace.active, file.path)
        XCTAssertTrue(workspace.currentBuffer === draft)
        XCTAssertTrue(draft.dirty)
        XCTAssertEqual(draft.text, "unsaved draft")
        XCTAssertEqual(draft.selection, NSRange(location: 3, length: 2))
        XCTAssertEqual(workspace.buffers[earlier.path]?.text, "saved earlier")
    }

    // Retiring a provider result permits a cached read and leaves the chosen tab in place.
    @MainActor func testRetiredSemanticResultDoesNotActivateLateSource() async throws {
        let arrived = expectation(description: "semantic target read")
        let gate = DispatchSemaphore(value: 0)
        ResponseProtocol.status = 200
        ResponseProtocol.responsesByPath = [:]
        ResponseProtocol.response = Data(#"{"path":"target.py","content":"value = 1","hash":"disk"}"#.utf8)
        ResponseProtocol.beforeResponse = { request in
            guard request.url?.path == "/api/file" else { return }
            arrived.fulfill()
            _ = gate.wait(timeout: .now() + 5)
        }
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [ResponseProtocol.self]
        let session = URLSession(configuration: configuration)
        let workspace = Workspace()
        workspace.engine = try Engine(address: "http://fixture.invalid", session: session)
        defer {
            gate.signal()
            ResponseProtocol.beforeResponse = nil
            workspace.buffers.values.forEach { $0.language?.close() }
            session.invalidateAndCancel()
        }
        let caller = FileNode(name: "caller.py", path: "caller.py", dir: false)
        let target = FileNode(name: "target.py", path: "target.py", dir: false)
        let buffer = DocumentBuffer(file: caller, text: FileText(path: caller.path, content: "value", hash: "caller"))
        workspace.buffers[caller.path] = buffer
        workspace.rootFiles = [caller, target]
        workspace.showSource(caller.path)
        let language = NativeLanguage(engine: try XCTUnwrap(workspace.engine), buffer: buffer)
        defer { language.close() }
        let location = SemanticLocation(uri: languageURI(target.path), line: 0, character: 2)
        language.locations = [location]
        language.locationSource = buffer.text
        language.locationSelection = buffer.selection
        var pending: Task<Void, Never>?
        buffer.openSemanticLocation = { location, current in
            pending = Task { await workspace.openSemanticLocation(location, isCurrent: current) }
        }
        language.open(location)
        await fulfillment(of: [arrived], timeout: 3)
        buffer.selection = NSRange(location: 1, length: 0)
        gate.signal()
        await pending?.value
        buffer.openSemanticLocation = nil

        XCTAssertTrue(workspace.currentBuffer === buffer)
        XCTAssertEqual(buffer.selection.location, 1)
        XCTAssertEqual(workspace.buffers[target.path]?.text, "value = 1")
        XCTAssertFalse(workspace.layout.groups().contains { $0.tabs.containsTab(nativeSourceTab(target.path)) })
        XCTAssertFalse(workspace.navigationTrail.canBack)
    }

    @MainActor func testNamedCrossFileLocationChecksTheLoadedIdentifier() async throws {
        let path = "target.py", source = "header\n🧪; old_name_suffix = 1\n"
        ResponseProtocol.status = 200
        ResponseProtocol.beforeResponse = nil
        ResponseProtocol.responsesByPath = [:]
        ResponseProtocol.response = try JSONSerialization.data(withJSONObject: ["path": path, "content": source, "hash": "disk"])
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [ResponseProtocol.self]
        let session = URLSession(configuration: configuration)
        let workspace = Workspace()
        workspace.engine = try Engine(address: "http://fixture.invalid", session: session)
        defer {
            workspace.buffers.values.forEach { $0.language?.close() }
            session.invalidateAndCancel()
        }
        let origin = FileNode(name: "origin.py", path: "origin.py", dir: false)
        let target = FileNode(name: path, path: path, dir: false)
        workspace.rootFiles = [origin, target]
        workspace.buffers[origin.path] = DocumentBuffer(file: origin, text: FileText(path: origin.path, content: "print(old_name)", hash: "origin"))
        workspace.showSource(origin.path)
        XCTAssertNil(workspace.buffers[path])
        func at(_ name: String, character: Int = 4) -> SemanticLocation {
            SemanticLocation(uri: languageURI(path), line: 1, character: character, name: name)
        }
        var revealed = 0
        await workspace.openSemanticLocation(at("old_name")) { revealed += 1 }
        let loaded = try XCTUnwrap(workspace.buffers[path])
        XCTAssertTrue(workspace.currentBuffer === loaded)
        XCTAssertEqual(loaded.text, source)
        XCTAssertEqual(loaded.selection, NSRange(location: 0, length: 0))
        XCTAssertTrue(workspace.status.contains("The reported symbol no longer matches"))
        XCTAssertEqual(revealed, 0)
        XCTAssertFalse(workspace.navigationTrail.canBack)

        var selections: [NSRange] = []
        loaded.selectSourceRange = { range, available in
            XCTAssertTrue(exactUTF8(available, source))
            selections.append(range)
            return true
        }
        workspace.showSource(origin.path)
        await workspace.openSemanticLocation(at("name_suffix", character: 8)) { revealed += 1 }
        XCTAssertTrue(selections.isEmpty)
        XCTAssertFalse(workspace.navigationTrail.canBack)

        workspace.showSource(origin.path)
        await workspace.openSemanticLocation(at("old_name_suffix")) { revealed += 1 }
        workspace.showSource(origin.path)
        await workspace.openSemanticLocation(at("Module.old_name")) { revealed += 1 }
        workspace.showSource(origin.path)
        await workspace.openSemanticLocation(at("old_name"), expectedSource: source) { revealed += 1 }
        let nameRange = (source as NSString).range(of: "old_name_suffix")
        XCTAssertEqual(selections, Array(repeating: NSRange(location: nameRange.location, length: 0), count: 3))
        XCTAssertEqual(revealed, 3)
        XCTAssertTrue(workspace.navigationTrail.canBack)
    }

    // Leaving and reselecting the same tab while mounting retires its old position request.
    @MainActor func testSemanticMountDoesNotRevealAfterTabReselection() async {
        let workspace = Workspace()
        let target = FileNode(name: "target.txt", path: "target.txt", dir: false)
        let buffer = DocumentBuffer(file: target, text: FileText(path: target.path, content: "abcdef", hash: "target"))
        workspace.buffers[target.path] = buffer
        let firstAttempt = expectation(description: "target editor awaiting mount")
        buffer.selectSourceRange = { _, _ in firstAttempt.fulfill(); return false }
        let location = SemanticLocation(uri: languageURI(target.path), line: 0, character: 4)
        let pending = Task { await workspace.openSemanticLocation(location) }
        await fulfillment(of: [firstAttempt], timeout: 3)
        workspace.showUtility(.agent)
        workspace.showSource(target.path)
        var selected: NSRange?
        buffer.selectSourceRange = { range, _ in selected = range; return true }
        await pending.value
        XCTAssertNil(selected)
        XCTAssertTrue(workspace.currentBuffer === buffer)
        XCTAssertFalse(workspace.navigationTrail.canBack)

        await workspace.openSemanticLocation(location)
        XCTAssertEqual(selected, NSRange(location: 4, length: 0))
    }

    // A newer row in the same saved search owns navigation while the first row reads.
    @MainActor func testNewerWorkspaceSearchRowKeepsItsEditor() async throws {
        let arrived = expectation(description: "earlier search result file read")
        let gate = DispatchSemaphore(value: 0)
        ResponseProtocol.status = 200
        ResponseProtocol.responsesByPath = [:]
        ResponseProtocol.response = Data(#"{"path":"earlier.txt","content":"first","hash":"disk"}"#.utf8)
        ResponseProtocol.beforeResponse = { request in
            guard request.url?.path == "/api/file" else { return }
            arrived.fulfill()
            _ = gate.wait(timeout: .now() + 5)
        }
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [ResponseProtocol.self]
        let session = URLSession(configuration: configuration)
        let workspace = Workspace()
        workspace.engine = try Engine(address: "http://fixture.invalid", session: session)
        defer {
            gate.signal()
            ResponseProtocol.beforeResponse = nil
            workspace.buffers.values.forEach { $0.language?.close() }
            session.invalidateAndCancel()
        }
        let earlier = FileNode(name: "earlier.txt", path: "earlier.txt", dir: false)
        let latter = FileNode(name: "latter.txt", path: "latter.txt", dir: false)
        workspace.rootFiles = [earlier, latter]
        let retained = DocumentBuffer(file: latter, text: FileText(path: latter.path, content: "second", hash: "saved"))
        workspace.buffers[latter.path] = retained
        workspace.showSource(latter.path)
        var selected: NSRange?
        retained.selectSourceRange = { range, _ in selected = range; return true }
        func row(_ file: FileNode, _ source: String) -> WorkspaceSearchHit {
            WorkspaceSearchHit(path: file.path, line: 1, text: source, column: 2, sourceHash: nativeSourceHash(source), generatedBy: nil)
        }
        workspace.showWorkspaceSearch = true
        let pending = Task { await workspace.openSearchHit(row(earlier, "first")) }
        await fulfillment(of: [arrived], timeout: 3)
        await workspace.openSearchHit(WorkspaceSearchHit(path: latter.path, line: 1, text: "second", column: 2, sourceHash: nil, generatedBy: nil))
        gate.signal()
        await pending.value

        XCTAssertTrue(workspace.currentBuffer === retained)
        XCTAssertNil(selected)
        XCTAssertTrue(workspace.showWorkspaceSearch)
        XCTAssertEqual(workspace.buffers[earlier.path]?.text, "first")
        XCTAssertFalse(workspace.layout.groups().contains { $0.tabs.containsTab(nativeSourceTab(earlier.path)) })

        await workspace.openSearchHit(row(latter, "second"))
        XCTAssertEqual(selected, NSRange(location: 2, length: 0))
        XCTAssertFalse(workspace.showWorkspaceSearch)
    }
    @MainActor func testDirtyTabCloseCancelAndDiscard() async throws {
        let (engine, buffer) = try fixture()
        let workspace = Workspace()
        workspace.engine = engine
        let tab = nativeSourceTab(buffer.file.path)
        workspace.buffers[buffer.file.path] = buffer
        workspace.show(tab)
        buffer.text = "discard me"
        buffer.selection = NSRange(location: 3, length: 2)
        workspace.newTab()
        let active = workspace.activeTab

        workspace.close(tab)
        let stale = try XCTUnwrap(workspace.tabClosePrompt)
        workspace.tabClosePrompt = nil
        XCTAssertTrue(workspace.bufferForTab(tab) === buffer)
        XCTAssertEqual(buffer.selection, NSRange(location: 3, length: 2))
        XCTAssertEqual(buffer.text, "discard me")

        buffer.text = "a newer draft"
        workspace.discardAndClose(stale)
        XCTAssertTrue(workspace.bufferForTab(tab) === buffer)
        workspace.close(tab)
        workspace.discardAndClose(try XCTUnwrap(workspace.tabClosePrompt))
        XCTAssertFalse(workspace.layout.groups().contains { $0.tabs.containsTab(tab) })
        XCTAssertNil(workspace.bufferForTab(tab))
        XCTAssertEqual(workspace.activeTab, active)

        ResponseProtocol.status = 200
        ResponseProtocol.responsesByPath = [:]
        ResponseProtocol.response = Data(#"{"path":"notes.txt","content":"before","hash":"disk"}"#.utf8)
        await workspace.open(buffer.file)
        XCTAssertEqual(workspace.currentBuffer?.text, "before")
        XCTAssertFalse(workspace.currentBuffer === buffer)
    }

    @MainActor func testDirtyTabCloseWaitsForGuardedSave() async throws {
        let (engine, buffer) = try fixture()
        let workspace = Workspace()
        workspace.engine = engine
        let tab = nativeSourceTab(buffer.file.path)
        workspace.buffers[buffer.file.path] = buffer
        workspace.show(tab)
        buffer.text = "next"
        workspace.newTab()
        let focused = workspace.activeTab
        ResponseProtocol.responsesByPath = [:]
        ResponseProtocol.status = 409
        ResponseProtocol.response = Data(#"{"error":"changed on disk"}"#.utf8)

        workspace.close(tab)
        let request = try XCTUnwrap(workspace.tabClosePrompt)
        await workspace.saveAndClose(request)
        XCTAssertTrue(workspace.layout.groups().contains { $0.tabs.containsTab(tab) })
        XCTAssertEqual(buffer.text, "next")
        XCTAssertTrue(workspace.status.contains("changed on disk"))

        let attempt = try XCTUnwrap(buffer.saveHistoryAttempt)
        ResponseProtocol.status = 200
        ResponseProtocol.response = try JSONSerialization.data(withJSONObject: [
            "version": 1, "history_protocol": 1, "path": buffer.file.path, "request_id": attempt.id,
            "source_hash": nativeSourceHash("next"), "save_guard": "after",
            "history": ["available": true, "outcome": "published"],
        ])
        let arrived = expectation(description: "close save reached the fixture")
        let gate = DispatchSemaphore(value: 0)
        ResponseProtocol.beforeResponse = { incoming in
            guard incoming.url?.path == "/api/source-history/file" else { return }
            arrived.fulfill()
            _ = gate.wait(timeout: .now() + 5)
        }
        defer { gate.signal(); ResponseProtocol.beforeResponse = nil }
        let pending = Task { await workspace.saveAndClose(request) }
        await fulfillment(of: [arrived], timeout: 3)
        workspace.close(tab)
        let busy = try XCTUnwrap(workspace.tabClosePrompt)
        XCTAssertTrue(busy.saving)
        workspace.discardAndClose(busy)
        XCTAssertTrue(workspace.bufferForTab(tab) === buffer)
        buffer.text = "newer edits"
        gate.signal()
        await pending.value
        ResponseProtocol.beforeResponse = nil
        XCTAssertEqual(buffer.base, "next")
        XCTAssertEqual(buffer.text, "newer edits")
        XCTAssertTrue(workspace.layout.groups().contains { $0.tabs.containsTab(tab) })

        let latest = SourceSaveAttempt(source: buffer.text,
            base: buffer.base, hash: buffer.hash, guardToken: "after", revision: buffer.sourceEditIdentity)
        buffer.saveHistoryAttempt = latest
        ResponseProtocol.response = try JSONSerialization.data(withJSONObject: [
            "version": 1, "history_protocol": 1, "path": buffer.file.path, "request_id": latest.id,
            "source_hash": nativeSourceHash("newer edits"), "save_guard": "last",
            "history": ["available": true, "outcome": "published"],
        ])
        workspace.close(tab)
        await workspace.saveAndClose(try XCTUnwrap(workspace.tabClosePrompt))
        XCTAssertFalse(buffer.dirty)
        XCTAssertEqual(buffer.base, "newer edits")
        XCTAssertFalse(workspace.layout.groups().contains { $0.tabs.containsTab(tab) })
        XCTAssertEqual(workspace.activeTab, focused)
    }

    @MainActor func testDirtyTabCloseSaveAsUsesTheRequestedDraft() async throws {
        let (engine, _) = try fixture()
        let workspace = Workspace()
        workspace.engine = engine
        workspace.newTab()
        let draft = try XCTUnwrap(workspace.currentBuffer)
        let tab = try XCTUnwrap(workspace.activeTab)
        draft.text = "untitled draft"
        workspace.newTab()
        let focused = workspace.activeTab

        workspace.close(tab)
        let prompt = try XCTUnwrap(workspace.tabClosePrompt)
        XCTAssertTrue(prompt.needsPath)
        await workspace.saveAndClose(prompt)
        XCTAssertEqual(workspace.fileCommand, "Save As")
        workspace.cancelFileCommand()
        XCTAssertNil(workspace.fileCommand)
        XCTAssertTrue(workspace.layout.groups().contains { $0.tabs.containsTab(tab) })
        XCTAssertTrue(workspace.bufferForTab(tab) === draft)
        XCTAssertEqual(draft.text, "untitled draft")

        workspace.sourceRootAddress = "/fixture"
        workspace.sourceWorkspaceID = "workspace"
        ResponseProtocol.status = 409
        ResponseProtocol.responsesByPath = [:]
        ResponseProtocol.response = Data(#"{"error":"destination exists"}"#.utf8)
        await workspace.saveAndClose(prompt)
        workspace.filePath = "copy.txt"
        await workspace.performFileCommand()
        XCTAssertTrue(workspace.layout.groups().contains { $0.tabs.containsTab(tab) })
        XCTAssertTrue(workspace.status.contains("destination exists"))

        let attempt = try XCTUnwrap(draft.creationAttempt)
        ResponseProtocol.status = 200
        ResponseProtocol.responsesByPath = [
            "/api/source-history/create": try JSONSerialization.data(withJSONObject: [
                "version": 1, "history_protocol": 1, "workspace": "workspace", "namespace": "/fixture",
                "path": "copy.txt", "source": draft.text, "source_hash": nativeSourceHash(draft.text),
                "hash": "confirmed", "request_id": attempt.id, "save_guard": "copy-before",
                "history": ["available": true, "outcome": "published"],
            ]),
            "/api/files": Data(#"{"root":"fixture","root_path":"/fixture","workspace_id":"workspace","tree":[]}"#.utf8),
        ]
        defer { ResponseProtocol.responsesByPath = [:] }
        await workspace.saveAndClose(prompt)
        workspace.filePath = "copy.txt"
        await workspace.performFileCommand()
        XCTAssertFalse(draft.dirty)
        XCTAssertTrue(workspace.buffers["copy.txt"] === draft)
        XCTAssertEqual(draft.base, "untitled draft")
        XCTAssertFalse(workspace.layout.groups().contains { $0.tabs.containsTab(tab) || $0.tabs.containsTab("copy.txt") })
        XCTAssertEqual(workspace.activeTab, focused)
    }

    @MainActor func testAgentRequestUsesSubscriptionAndPythonBackend() async throws {
        let (engine, _) = try fixture()
        ResponseProtocol.status = 200; ResponseProtocol.responsesByPath = [:]
        ResponseProtocol.response = Data("{\"session_id\":\"turn\"}".utf8)
        _ = try await engine.send("# Task", parent: nil, provider: "claude-subscription", model: "sonnet", effort: "high")
        XCTAssertEqual(ResponseProtocol.captured["backend"] as? String, "python")
        XCTAssertEqual(ResponseProtocol.captured["provider"] as? String, "claude-subscription")
        XCTAssertEqual(ResponseProtocol.captured["model"] as? String, "sonnet")
        XCTAssertEqual(ResponseProtocol.captured["effort"] as? String, "high")
        XCTAssertEqual(ResponseProtocol.captured["prompt"] as? String, "# Task")
    }

    @MainActor private func fixture(doc: String? = nil, generated: String? = nil) throws -> (Engine, DocumentBuffer) {
        let configuration = URLSessionConfiguration.ephemeral; configuration.protocolClasses = [ResponseProtocol.self]
        let engine = try Engine(address: "http://fixture.invalid", session: URLSession(configuration: configuration))
        let file = FileNode(name: "notes.txt", path: "notes.txt", dir: false, doc_id: doc, generated_by: generated)
        return (engine, DocumentBuffer(file: file, text: FileText(path: file.path, content: "before", hash: "hash-before",save_guard:"captured-before")))
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
        let (engine,buffer)=try fixture(doc:"doc");buffer.text="next"
        let id=String(repeating:"1",count:32);buffer.saveHistoryAttempt=SourceSaveAttempt(id:id,source:"next",base:"before",hash:"hash-before",guardToken:"captured-before",revision:buffer.sourceEditIdentity)
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath=[:];ResponseProtocol.beforeResponse=nil
        ResponseProtocol.response=try JSONSerialization.data(withJSONObject:["version":1,"history_protocol":1,"path":"notes.txt","request_id":id,"source_hash":nativeSourceHash("next"),"save_guard":"after","history":["available":true,"outcome":"published"]])
        await buffer.save(using:engine)
        let expected=ResponseProtocol.captured["expected"] as? [String:String]
        XCTAssertEqual(expected,["source":"before","disk":"before"]);XCTAssertFalse(buffer.dirty);XCTAssertEqual(buffer.savedGuard,"after");XCTAssertEqual(ResponseProtocol.requestedURL?.path,"/api/source-history/docs/doc")
    }
    @MainActor func testGeneratedOutputDoesNotDispatchSave() async throws {
        let (engine, buffer) = try fixture(generated: "owner")
        ResponseProtocol.captured = [:]; buffer.text = "replacement"; await buffer.save(using: engine)
        XCTAssertTrue(ResponseProtocol.captured.isEmpty); XCTAssertTrue(buffer.readOnly)
    }
    // Contract: .hickory-source/frontends/contract.md — reopening retains the PTY identity.
    @MainActor func testTerminalReopenKeepsExistingSession() async throws {
        let (engine, _) = try fixture()
        ResponseProtocol.status = 200; ResponseProtocol.response = Data(#"{"id":"pty"}"#.utf8)
        let session = TerminalSession()
        await session.open(using: engine)
        ResponseProtocol.status = 409; ResponseProtocol.response = Data(#"{"error":"must not open twice"}"#.utf8)
        await session.open(using: engine)
        XCTAssertEqual(session.id, "pty"); XCTAssertTrue(session.error.isEmpty)
    }

}
#if os(macOS)
import AppKit
import SwiftUI
#endif
final class NativeDebugTests: XCTestCase {
    @MainActor func testLiteralDebuggerURIAndReceiptRoutePreserveExactPhysicalName()throws {
        let path="%2F λ?#.py",uri=nativeDebugURI(path);XCTAssertEqual(uri,"hick:///%252F%20%CE%BB%3F%23.py");XCTAssertEqual(URLComponents(string:uri)?.path,"/"+path);XCTAssertEqual(nativeDebugURI("hick://captured-project/main.py"),"hick://captured-project/main.py")
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid")),file=FileNode(name:path,path:path,dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:path,content:"print(1)\n",hash:"saved"))
        debug.owner="main.py";debug.launchSource="print(0)\n";let launch=debug.beginLaunchRequest();var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)};debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]]);debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[["id":7,"name":"literal","source":path,"source_line":0,"source_hash":nativeSourceHash(buffer.text)]],"variables":[]]);debug.toggle(buffer,line:0,condition:"true");let request=try XCTUnwrap(sent.last)
        // The secondary receipt route uses opaque request+URI matching;
        // a twice-decoded slash or raw query/fragment cannot identify this path.
        func receipt(_ source:String)->[String:Any] {["event":"source_breakpoints","session":"ours","source":source,"request_id":request["request_id"]!,"source_hash":nativeSourceHash(buffer.text),"breakpoints":[["line":0,"state":"bound"]]]}
        debug.receive(receipt("hick:///%2F%20%CE%BB%3F%23.py"));XCTAssertFalse(debug.breakpointMarkers(buffer).first?["state"] as? String=="bound")
        debug.receive(receipt(uri));XCTAssertEqual(debug.breakpointMarkers(buffer).first?["state"] as? String,"bound");debug.close()
    }
    @MainActor func testBreakpointReceiptsRequireExactSourceAndLatestIntent()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid")),file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"value = 1\n",hash:"saved"))
        debug.owner=file.path;debug.launchSource=buffer.text;debug.sourceBuffers={ [buffer] };let launch=debug.beginLaunchRequest();var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)}
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]])
        debug.toggle(buffer,line:0,condition:"value == 1");let old=try XCTUnwrap(sent.last)
        debug.toggle(buffer,line:0,condition:"value == 2");let current=try XCTUnwrap(sent.last)
        func receipt(_ request:[String:Any])->[String:Any] {["event":"breakpoints","session":"ours","request_id":request["request_id"]!,"source_hash":request["source_hash"]!,"breakpoints":[["line":0,"moved_to":1,"state":"bound"]]]}
        debug.receive(receipt(old));XCTAssertNil(debug.breakpointMarkers(buffer).first?["display_line"])
        var malformed=receipt(current);malformed["breakpoints"]=[["line":0,"moved_to":true,"state":"bound"]];debug.receive(malformed);XCTAssertNil(debug.breakpointMarkers(buffer).first?["display_line"])
        debug.receive(receipt(current));XCTAssertEqual(debug.breakpointMarkers(buffer).first?["line"] as? Int,1);XCTAssertEqual(debug.breakpoints[file.path]?.first?["line"] as? Int,0);XCTAssertEqual(debug.breakpoints[file.path]?.first?["condition"] as? String,"value == 2")
        debug.toggle(buffer,line:1,condition:"moved display condition");XCTAssertEqual(debug.breakpoints[file.path]?.count,1);XCTAssertEqual(debug.breakpoints[file.path]?.first?["line"] as? Int,0,"Moved receipt does not become the intent coordinate");XCTAssertEqual(debug.breakpoints[file.path]?.first?["condition"] as? String,"moved display condition")
        let validIntentCount=sent.count;debug.toggle(buffer,line:999,condition:"outside snapshot");XCTAssertEqual(sent.count,validIntentCount);XCTAssertEqual(debug.breakpoints[file.path]?.count,1)
        buffer.text="changed\n";debug.sourceChanged(buffer);XCTAssertTrue(debug.breakpointMarkers(buffer).isEmpty)
        let count=sent.count;debug.toggle(buffer,line:0,condition:"changed");XCTAssertEqual(sent.count,count);debug.receive(receipt(current));XCTAssertTrue(debug.breakpointMarkers(buffer).isEmpty);XCTAssertFalse(debug.breakpointNotice.isEmpty)
        let currentIntent=debug.breakpoints[file.path]?.first?["condition"] as? String;debug.toggle(file.path,line:0,condition:"raw stale caller",source:debug.launchSource);XCTAssertEqual(sent.count,count,"Raw callers cannot bypass the model’s live dirty-buffer guard");XCTAssertEqual(debug.breakpoints[file.path]?.first?["condition"] as? String,currentIntent,"Stale sources cannot replace current intentions")
        buffer.base=buffer.text;debug.toggle(buffer,line:0,condition:"saved");XCTAssertEqual(sent.count,count,"Saving changed text does not bind it to an earlier runtime capture");XCTAssertFalse(debug.retainedBreakpointReading(file.path).isEmpty);debug.close()
    }
    @MainActor func testBoundBreakpointMarkersRetireWithTheirSession()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid")),file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"print(1)\n",hash:"saved"));debug.owner=file.path;debug.launchSource=buffer.text;debug.sourceBuffers={ [buffer] };let launch=debug.beginLaunchRequest();var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)}
        debug.receive(["event":"started","source_hash":nativeSourceHash(buffer.text),"request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]]);debug.toggle(buffer,line:0);let request=try XCTUnwrap(sent.last),receipt:[String:Any]=["event":"breakpoints","session":"ours","source_hash":nativeSourceHash(buffer.text),"request_id":request["request_id"]!,"breakpoints":[["line":0,"state":"bound"]]];debug.receive(receipt);XCTAssertEqual(debug.breakpointMarkers(buffer).first?["state"] as? String,"bound")
        debug.receive(["event":"finished","session":"ours"]);XCTAssertEqual(debug.breakpointMarkers(buffer).first?["state"] as? String,"pending");_ = debug.beginLaunchRequest();debug.receive(receipt);XCTAssertEqual(debug.breakpointMarkers(buffer).first?["state"] as? String,"pending");debug.close()
    }
    @MainActor func testNativeBreakpointCanonicalPathAliasesRefuseInsteadOfSharingIntent()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="é.py";debug.launchSource="print(1)\n";let launch=debug.beginLaunchRequest();var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)};debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":launch,"doc":nativeDebugURI(debug.owner),"session":"ours","breakpoints":[]]);debug.toggle(debug.owner,line:0,source:debug.launchSource);let count=sent.count
        let path = "e\u{301}.py"
        debug.toggle(path, line: 1, source: debug.launchSource)
        XCTAssertTrue(debug.breakpointNotice.contains("unsupported"))
        let alias = DocumentBuffer(file: FileNode(name: path, path: path, dir: false),
                                   text: FileText(path: path, content: debug.launchSource, hash: "saved"))
        debug.toggle(alias, line: 0)
        XCTAssertTrue(debug.breakpointMarkers(alias).isEmpty)
        XCTAssertEqual(sent.count, count)
        XCTAssertEqual(debug.breakpoints[debug.owner]?.count, 1)
        debug.close()
    }
    @MainActor func testStartedRequiresActualCapturedHashBeforePrimaryPositionAuthority()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid")),file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"print(1)\n",hash:"saved"))
        debug.owner=file.path;debug.launchSource=buffer.text;debug.sourceBuffers={ [buffer] };let launch=debug.beginLaunchRequest();debug.receive(["event":"started","request_id":launch,"source_hash":nativeSourceHash("other"),"doc":"hick:///main.py","session":"ours","breakpoints":[]])
        debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[["id":7,"name":"main","source":"main.py","source_line":0]],"variables":[]]);XCTAssertFalse(debug.sourceMatches(buffer));XCTAssertTrue(debug.canResume);XCTAssertFalse(debug.message.isEmpty);debug.close()
    }
    @MainActor func testMovementCarriesObservedStopThreadAndUniqueCausalTicket()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";let launch=debug.beginLaunchRequest();var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)}
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]])
        debug.receive(["event":"stopped","session":"ours","stop":4,"thread":-2,"frames":[["id":0,"name":"main"]],"variables":[]]);debug.step("over")
        XCTAssertEqual(sent.last?["stop"] as? UInt64,4);XCTAssertEqual(sent.last?["thread"] as? Int64,-2);XCTAssertNotNil(sent.last?["request_id"] as? String);let count=sent.count;debug.step("over");XCTAssertEqual(sent.count,count);debug.close()
    }
    @MainActor func testPostEffectMovementFailureRetiresPauseUntilActualNewerStop()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";let launch=debug.beginLaunchRequest();var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)};debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]])
        func pause(_ stop:Int){debug.receive(["event":"stopped","session":"ours","stop":stop,"thread":1,"frames":[["id":7,"name":"retained frame","source":"main.py","source_line":0]],"variables":[["name":"old","value":"1","variables_reference":10]]])}
        pause(1);debug.step("over");let request=try XCTUnwrap(sent.last);debug.receive(["event":"failed","session":"ours","about":"step","request_id":request["request_id"]!,"scope_retired":true,"message":"Stop observation failed"])
        XCTAssertTrue(debug.active);XCTAssertEqual(debug.status,"unconfirmed");XCTAssertTrue(debug.pauseUnconfirmed);XCTAssertFalse(debug.canResume);XCTAssertFalse(debug.canEvaluate);XCTAssertFalse(debug.canReadThreads);XCTAssertNil(debug.line(in:"main.py"));XCTAssertFalse(debug.frames.isEmpty);XCTAssertTrue(debug.stackNotice.contains("historical"));XCTAssertTrue(debug.visibleVariableRows.isEmpty)
        let count=sent.count;debug.evaluate("old");debug.step("over");XCTAssertEqual(sent.count,count);pause(1);XCTAssertTrue(debug.pauseUnconfirmed,"A held previous stop cannot restore retired authority")
        debug.receive(["event":"failed","session":"ours","about":"step","request_id":request["request_id"]!,"scope_retired":false,"message":"late contradiction"]);XCTAssertTrue(debug.pauseUnconfirmed)
        pause(2);XCTAssertFalse(debug.pauseUnconfirmed);XCTAssertTrue(debug.canResume);debug.stop();XCTAssertEqual(sent.last?["op"] as? String,"stop");debug.close()
    }
    @MainActor func testMovementRefusalCannotPromoteHistoricalFramesOrCancelStop()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";let launch=debug.beginLaunchRequest();var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)};debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]])
        debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[["id":7,"name":"main"]],"variables":[]]);debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[],"variables":[],"stack_error":"provider refused"]);XCTAssertFalse(debug.canInspectFrame);XCTAssertFalse(debug.frames.isEmpty)
        debug.step("continue");let request=try XCTUnwrap(sent.last);debug.receive(["event":"failed","session":"ours","about":"step","request_id":request["request_id"]!,"message":"continue refused"]);XCTAssertEqual(debug.status,"paused");XCTAssertFalse(debug.canInspectFrame);XCTAssertTrue(debug.stackNotice.contains("provider refused"));XCTAssertTrue(debug.canResume)
        debug.step("continue");let retired=try XCTUnwrap(sent.last);debug.stop();debug.receive(["event":"failed","session":"ours","about":"step","request_id":retired["request_id"]!,"message":"late refusal"]);XCTAssertEqual(debug.status,"stopping");debug.close()
    }
    @MainActor func testPrimaryPositionMovementRefusesEditedAndForeignSource()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid")),file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"print(1)\n",hash:"saved"))
        debug.owner=file.path;debug.launchSource=buffer.text;debug.sourceBuffers={ [buffer] };let launch=debug.beginLaunchRequest();var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)}
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[],"capabilities":["goto_targets":true,"run_to":true]])
        func pause(_ stop:Int){debug.receive(["event":"stopped","session":"ours","stop":stop,"thread":1,"frames":[["id":7,"name":"main","source":"main.py","source_line":0]],"variables":[]])}
        pause(1);debug.moveToSource(buffer,line:0,how:"run_to");let request=try XCTUnwrap(sent.last);XCTAssertEqual(request["source_hash"] as? String,nativeSourceHash(buffer.text));XCTAssertEqual(request["stop"] as? UInt64,1)
        debug.receive(["event":"failed","session":"ours","about":"run_to","request_id":"older","message":"obsolete"]);XCTAssertEqual(debug.status,"running")
        debug.receive(["event":"failed","session":"ours","about":"run_to","request_id":request["request_id"]!,"message":"provider refused"]);XCTAssertEqual(debug.status,"paused");XCTAssertTrue(debug.canResume)
        pause(2);buffer.text="changed\n";let count=sent.count;debug.moveToSource(buffer,line:0,how:"jump");XCTAssertEqual(sent.count,count)
        buffer.base=buffer.text;debug.moveToSource(buffer,line:0,how:"jump");XCTAssertEqual(sent.count,count,"Saved changed source is still not launch source")
        let other=DocumentBuffer(file:FileNode(name:"other.py",path:"other.py",dir:false),text:FileText(path:"other.py",content:debug.launchSource,hash:"saved"));debug.moveToSource(other,line:0,how:"run_to");XCTAssertEqual(sent.count,count);debug.close()
    }
    @MainActor func testRecursiveVariablesRequireCurrentScopeAndRetainRefusalEscape()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";let launch=debug.beginLaunchRequest();var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)}
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]])
        func pause(_ stop:Int){debug.receive(["event":"stopped","session":"ours","stop":stop,"thread":1,"frames":[["id":7,"name":"main"]],"variables":[["name":"root","value":"object","variables_reference":10]]])}
        func reply(_ request:[String:Any],_ vars:[[String:Any]])->[String:Any] {["event":"children","session":"ours","stop":request["stop"]!,"thread":request["thread"]!,"frame":request["frame"]!,"reference":request["reference"]!,"request_id":request["request_id"]!,"variables":vars]}
        pause(1);let root=try XCTUnwrap(debug.visibleVariableRows.first);debug.toggleVariable(root.id);let first=try XCTUnwrap(sent.last);XCTAssertTrue(debug.visibleVariableRows.first?.pending==true)
        debug.receive(reply(first,[["name":"nested","value":"child","variables_reference":11]]));let nested=try XCTUnwrap(debug.visibleVariableRows.last);XCTAssertEqual(nested.depth,1)
        debug.toggleVariable(nested.id);let next=try XCTUnwrap(sent.last);debug.receive(reply(next,[]));XCTAssertEqual(debug.visibleVariableRows.last?.notice,"No child variables reported.")
        debug.toggleVariable(root.id);XCTAssertEqual(debug.visibleVariableRows.count,1);debug.toggleVariable(root.id);XCTAssertEqual(debug.visibleVariableRows.count,2)
        pause(2);let fresh=try XCTUnwrap(debug.visibleVariableRows.first);debug.toggleVariable(fresh.id);let request=try XCTUnwrap(sent.last)
        debug.receive(reply(first,[["name":"obsolete","value":"bad","variables_reference":0]]));XCTAssertEqual(debug.visibleVariableRows.count,1);XCTAssertTrue(debug.visibleVariableRows.first?.pending==true)
        var wrongScope=reply(request,[]);wrongScope["frame"]=8;debug.receive(wrongScope);XCTAssertTrue(debug.visibleVariableRows.first?.retryable==true);XCTAssertFalse(debug.visibleVariableRows.first?.pending==true)
        debug.readChildren(fresh.id);let failedRequest=try XCTUnwrap(sent.last)
        debug.receive(["event":"failed","session":"ours","about":"children","request_id":failedRequest["request_id"]!,"message":"provider refused"]);XCTAssertTrue(debug.visibleVariableRows.first?.notice.contains("provider refused")==true);XCTAssertTrue(debug.canResume);XCTAssertTrue(debug.canReadThreads)
        debug.readChildren(fresh.id);let retry=try XCTUnwrap(sent.last);debug.receive(reply(retry,[["name":"done","value":"42","variables_reference":0]]));XCTAssertEqual(debug.visibleVariableRows.last?.value,"42")
        debug.evaluate("object");let evaluation=try XCTUnwrap(sent.last);debug.receive(["event":"value","session":"ours","stop":2,"thread":1,"frame":7,"request_id":evaluation["request_id"]!,"expression":"object","value":"composite","reference":20])
        let expressionRoot=try XCTUnwrap(debug.visibleVariableRows.last);XCTAssertEqual(expressionRoot.name,"object");debug.toggleVariable(expressionRoot.id);let held=try XCTUnwrap(sent.last)
        debug.evaluate("object");debug.receive(reply(held,[["name":"obsolete expression child","value":"bad","variables_reference":0]]));XCTAssertFalse(debug.visibleVariableRows.contains{$0.id==expressionRoot.id || $0.name=="obsolete expression child"});XCTAssertNil(debug.values["object"]);debug.close()
    }

    @MainActor func testCatalogExclusionsSurviveAutomaticLaunch()async throws {
        let config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self];let engine=try Engine(address:"http://fixture.invalid",session:URLSession(configuration:config)),debug=NativeDebugger(engine:engine),file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"print(1)\n",hash:"saved"))
        var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)};defer{debug.close();ResponseProtocol.responsesByPath=[:]}
        ResponseProtocol.status=200;ResponseProtocol.response=Data(#"{"debug_protocol":3,"profiles":[{"id":"python","name":"Python","program":"main.py","can_debug":true}],"fingerprint":"current","notices":["Unreadable physical input excluded: opaque.bin"]}"#.utf8)
        await debug.prepare(buffer);XCTAssertEqual(sent.first?["op"] as? String,"start");XCTAssertTrue(debug.profiles.isEmpty);XCTAssertEqual(debug.catalogNotices,["Unreadable physical input excluded: opaque.bin"])
    }
    @MainActor func testBreakpointFailureRevokesAllReceiptsWithoutRetiringPause()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid")),text="print(1)\n",a=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:text,hash:"saved")),b=DocumentBuffer(file:FileNode(name:"b.py",path:"b.py",dir:false),text:FileText(path:"b.py",content:text,hash:"saved"))
        debug.owner="a.py";debug.launchSource=text;debug.sourceBuffers={ [a,b] };let launch=debug.beginLaunchRequest();var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)}
        debug.receive(["event":"started","request_id":launch,"doc":"hick:///a.py","source_hash":nativeSourceHash(text),"session":"ours","breakpoints":[]]);debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[["id":7,"name":"b","source":"b.py","source_line":0,"source_hash":nativeSourceHash(text)]],"variables":[]])
        func receipt(_ request:[String:Any],_ path:String)->[String:Any] {var value:[String:Any]=["event":path=="a.py" ? "breakpoints":"source_breakpoints","session":"ours","request_id":request["request_id"]!,"source_hash":nativeSourceHash(text),"breakpoints":[["line":0,"state":"bound"]]];if path != "a.py"{value["source"]=nativeDebugURI(path)};return value}
        debug.toggle(a,line:0);let oldA=try XCTUnwrap(sent.last);debug.receive(receipt(oldA,"a.py"));debug.toggle(b,line:0);let oldB=try XCTUnwrap(sent.last);debug.receive(receipt(oldB,"b.py"));XCTAssertEqual(debug.breakpointMarkers(a).first?["state"] as? String,"bound");XCTAssertEqual(debug.breakpointMarkers(b).first?["state"] as? String,"bound")
        debug.toggle(a,line:0,condition:"new");let failed=try XCTUnwrap(sent.last);debug.receive(["event":"failed","session":"ours","about":"breakpoints","request_id":failed["request_id"]!,"message":"partial adapter failure"])
        debug.receive(receipt(oldA,"a.py"));debug.receive(receipt(oldB,"b.py"));var uncorrelated=receipt(oldB,"b.py");uncorrelated.removeValue(forKey:"request_id");debug.receive(uncorrelated)
        XCTAssertFalse(debug.breakpointMarkers(a).first?["state"] as? String=="bound");XCTAssertFalse(debug.breakpointMarkers(b).first?["state"] as? String=="bound");XCTAssertTrue(debug.canResume);XCTAssertTrue(debug.sourceMatches(b));XCTAssertEqual(debug.status,"paused");debug.close()
    }
    @MainActor func testResourceLineageRejectsSameByteSwapDeleteAndRecreatedName()async throws {
        let space=Workspace(),engine=try Engine(address:"http://fixture.invalid"),debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid")),text="print(1)\n"
        space.engine=engine;space.debugger=debug
        let a=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:text,hash:"saved")),b=DocumentBuffer(file:FileNode(name:"b.py",path:"b.py",dir:false),text:FileText(path:"b.py",content:text,hash:"saved"))
        space.buffers=["a.py":a,"b.py":b];space.rootFiles=[a.file,b.file];space.layout.activate("a.py",in:"document-group");a.selection=NSRange(location:2,length:0)
        debug.sourceBuffers={ [weak space] in guard let space else{return []};return Array(space.buffers.values) };debug.owner="a.py";debug.launchSource=text;let launch=debug.beginLaunchRequest();var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)}
        debug.receive(["event":"started","request_id":launch,"doc":"hick:///a.py","source_hash":nativeSourceHash(text),"session":"ours","breakpoints":[]])
        func pause(_ stop:Int){debug.receive(["event":"stopped","session":"ours","stop":stop,"thread":1,"frames":[["id":7,"name":"a","source":"a.py","source_line":0,"source_hash":nativeSourceHash(text)]],"variables":[]])};pause(1);XCTAssertTrue(debug.sourceMatches(a))
        let saved=ResourceSaved(kind:"file",text:text,sha256:nativeSourceHash(text),readonly:false,executable:false,mode:nil),missing=ResourceSaved(kind:"missing"),sa=ResourceSource(identity:"original-a",text:text),sb=ResourceSource(identity:"original-b",text:text)
        func adopt(_ patches:[ResourceEditorPatch],saved values:[String:ResourceSaved],restoring:[String:DocumentBuffer]=[:])throws {
            let files=values.compactMap{path,state->ResourcePublication<FileNode>.Entry? in guard state.kind=="file" else{return nil};return .init(file:FileNode(name:path,path:path,dir:false),saved:state,hash:"fresh")}
            let receipt=ResourcePublication(version:1,request_id:"request",workspace:"workspace",context:"context",outcome:ResourcePublication<FileNode>.Outcome(complete:true,recovery:"record",applied:patches.map{$0.path}),saved_sources:values,source_files:files)
            _ = try space.adoptResourceReceipt(patches,receipt:receipt,request:"request",workspace:"workspace",context:"context",expected:values,current:space.captureResourceEditors(patches,restoring:restoring),restoring:restoring)
        }
        let held=try XCTUnwrap(debug.sourceLocation(path:"a.py",line:0)),snapshot=debug.frameSnapshot
        try adopt([.init(path:"a.py",before:sa,after:sb),.init(path:"b.py",before:sb,after:sa)],saved:["a.py":saved,"b.py":saved]);XCTAssertTrue(space.buffers["a.py"] === b);XCTAssertTrue(space.buffers["b.py"] === a);XCTAssertFalse(debug.locationCurrent(held));XCTAssertFalse(debug.sourceMatches(a));XCTAssertFalse(debug.sourceMatches(b))
        let count=sent.count;debug.toggle(b,line:0);XCTAssertEqual(sent.count,count);debug.select(debug.frames[0],snapshot:snapshot);pause(2);XCTAssertFalse(debug.sourceMatches(b));var reveals=0;b.revealDebugLine={_ in reveals+=1;return true};await space.revealDebug(try XCTUnwrap(debug.sourceLocation(path:"a.py",line:0)),debug:debug);XCTAssertEqual(reveals,0)
        try adopt([.init(path:"a.py",before:sb,after:sa),.init(path:"b.py",before:sa,after:sb)],saved:["a.py":saved,"b.py":saved]);XCTAssertFalse(debug.sourceMatches(a));pause(3);XCTAssertTrue(debug.sourceMatches(a));XCTAssertFalse(debug.breakpointMarkers(a).first?["state"] as? String=="bound")
        try adopt([.init(path:"a.py",before:sa,after:nil)],saved:["a.py":missing]);XCTAssertTrue(a.resourceDeleted);XCTAssertFalse(debug.sourceMatches(a));let deletedCount=sent.count;debug.toggle(a,line:0);XCTAssertEqual(sent.count,deletedCount)
        try adopt([.init(path:"a.py",before:nil,after:.init(identity:"independent",text:text))],saved:["a.py":saved]);let replacement=try XCTUnwrap(space.buffers["a.py"]);XCTAssertFalse(replacement === a);pause(4);XCTAssertFalse(debug.sourceMatches(replacement));XCTAssertFalse(debug.admitSourceReading(replacement,path:"a.py",proof:nativeSourceHash(text)));let before=sent.count;debug.toggle(replacement,line:0);XCTAssertEqual(sent.count,before)
        XCTAssertEqual(a.selection,NSRange(location:2,length:0));XCTAssertEqual(a.text,text);XCTAssertTrue(debug.canResume);debug.stop();XCTAssertEqual(sent.last?["op"] as? String,"stop");debug.close();for buffer in space.buffers.values{buffer.language?.close()}
    }
    @MainActor func testDelayedLaunchProfileCannotStartReaddressedEditor()async throws {
        let config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self];let session=URLSession(configuration:config),engine=try Engine(address:"http://fixture.invalid",session:session),debug=NativeDebugger(engine:engine)
        let file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"print(1)\n",hash:"saved")),arrived=expectation(description:"launch profile requested"),gate=DispatchSemaphore(value:0);var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)}
        ResponseProtocol.status=200;ResponseProtocol.response=Data(#"{"debug_protocol":3,"profiles":[{"id":"python","name":"Python","program":"main.py","can_debug":true}],"fingerprint":"current"}"#.utf8);ResponseProtocol.beforeResponse={request in if request.url?.path=="/api/launch-profiles"{arrived.fulfill();_ = gate.wait(timeout:.now()+5)}}
        defer{gate.signal();ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:];debug.close();session.invalidateAndCancel()}
        let preparation=Task{await debug.prepare(buffer)};await fulfillment(of:[arrived],timeout:3);buffer.file.path="renamed.py";buffer.file.name="renamed.py";gate.signal();await preparation.value
        XCTAssertTrue(sent.isEmpty);XCTAssertFalse(debug.active);XCTAssertTrue(debug.message.contains("Source identity"));XCTAssertEqual(buffer.text,"print(1)\n")
    }
    @MainActor func testUnknownSourceIntentCannotBorrowLaterLaunchSnapshot()async throws {
        let config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self];let engine=try Engine(address:"http://fixture.invalid",session:URLSession(configuration:config)),debug=NativeDebugger(engine:engine)
        let file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"print(2)\n",hash:"saved"));var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)};defer{debug.close();ResponseProtocol.responsesByPath=[:]}
        debug.toggle(file.path,line:0);ResponseProtocol.status=200;ResponseProtocol.response=Data(#"{"debug_protocol":3,"profiles":[{"id":"python","name":"Python","program":"main.py","can_debug":true}],"fingerprint":"current"}"#.utf8)
        await debug.prepare(buffer);let request=try XCTUnwrap(sent.first(where:{$0["op"] as? String=="start"}));XCTAssertTrue((request["breakpoints"] as? [[String:Any]])?.isEmpty==true);XCTAssertTrue(debug.breakpointMarkers(buffer).isEmpty);XCTAssertTrue(debug.retainedBreakpointReading(file.path).contains{$0.contains("unknown")})
    }
    @MainActor func testLaunchProfileSelectionUsesExactPhysicalPathBytes()async throws {
        let config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self]
        let engine=try Engine(address:"http://fixture.invalid",session:URLSession(configuration:config)),debug=NativeDebugger(engine:engine),wanted="e\u{301}.py",alias="é.py",source="print(1)\n"
        let file=FileNode(name:wanted,path:wanted,dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:wanted,content:source,hash:"saved"));var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)};defer{debug.close();ResponseProtocol.responsesByPath=[:]}
        ResponseProtocol.status=200;ResponseProtocol.response=try JSONSerialization.data(withJSONObject:["debug_protocol":3,"profiles":[["id":"app:"+alias,"name":"alias","program":alias,"can_debug":true],["id":"app:"+wanted,"name":"wanted","program":wanted,"can_debug":true]],"fingerprint":"current","viewed":wanted])
        await debug.prepare(buffer);let start=try XCTUnwrap(sent.first(where:{$0["op"] as? String=="start"})),profile=try XCTUnwrap(start["profile"] as? [String:Any]);XCTAssertTrue(exactUTF8(profile["id"] as? String,"app:"+wanted));XCTAssertEqual(start["doc"] as? String,nativeDebugURI(wanted));XCTAssertFalse(exactUTF8(nativeDebugURI(wanted),nativeDebugURI(alias)))
    }
    @MainActor func testLegacyCatalogRefusesDebugBeforeStartAndRetainsSource()async throws {
        let config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self]
        let engine=try Engine(address:"http://fixture.invalid",session:URLSession(configuration:config)),debug=NativeDebugger(engine:engine)
        let source="print(1)\n",file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"saved"))
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)};defer{debug.close();ResponseProtocol.responsesByPath=[:]}
        let protocols:[Int?]=[nil,1,2]
        for version in protocols {
            var catalog:[String:Any]=["profiles":[["id":"python","name":"Python","can_debug":true,"program":"main.py"]],"fingerprint":"current"]
            if let version{catalog["debug_protocol"]=version}
            ResponseProtocol.status=200;ResponseProtocol.response=try JSONSerialization.data(withJSONObject:catalog)
            await debug.prepare(buffer);XCTAssertTrue(requests.isEmpty);XCTAssertEqual(debug.status,"unavailable");XCTAssertTrue(debug.message.contains("No Start command was sent"));XCTAssertEqual(buffer.text,source);XCTAssertFalse(buffer.dirty)
        }
    }

    @MainActor func testCanonicalDistinctExpressionsKeepExactRequestsValuesAndWatchRows()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";let launch=debug.beginLaunchRequest()
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)};debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]]);debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[["id":11,"name":"main"]],"variables":[]])
        let first="'é'",second="'e\u{301}'"
        XCTAssertEqual(first,second,"Swift linguistic equality must not become debugger expression identity.")
        debug.evaluate(first,watch:true);let one=requests.last!;debug.evaluate(second,watch:true);let two=requests.last!
        XCTAssertEqual(debug.watches.count,2);XCTAssertEqual(Set(debug.watches.map{Data($0.utf8)}).count,2)
        func value(_ request:[String:Any],_ expression:String,_ text:String)->[String:Any] {["event":"value","request_id":request["request_id"]!,"session":"ours","stop":1,"thread":1,"frame":11,"expression":expression,"value":text]}
        debug.receive(value(one,first,"first result"));XCTAssertEqual(debug.values[first],"first result");XCTAssertNil(debug.values[second])
        debug.receive(value(two,second,"second result"));XCTAssertEqual(debug.values[first],"first result");XCTAssertEqual(debug.values[second],"second result");XCTAssertEqual(Set(debug.values.readings.map{$0.id}).count,2)
        debug.evaluate(first,watch:true);let newer=requests.last!;debug.receive(value(two,first,"wrong canonical row"));XCTAssertNil(debug.values[first],"A pending newer reading does not display an older value as its result");debug.receive(value(newer,first,"updated first"))
        debug.removeWatch(first);XCTAssertFalse(debug.isWatched(first));XCTAssertTrue(debug.isWatched(second));XCTAssertNil(debug.values[first]);XCTAssertEqual(debug.values[second],"second result");XCTAssertEqual(debug.values.readings.count,1);debug.close()
    }

    @MainActor func testZeroNegativeAndMalformedRuntimeIDsStayDistinct()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";debug.launchSource="value\n";let launch=debug.beginLaunchRequest()
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)};debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]])
        func stopped(_ stop:Int,_ thread:Any,_ frame:Any)->[String:Any] {["event":"stopped","session":"ours","stop":stop,"thread":thread,"frames":[["id":frame,"name":"frame","source":"main.py","source_line":0]],"variables":[]]}
        debug.receive(stopped(1,-1,-7));XCTAssertEqual(debug.selectedThread,-1);XCTAssertEqual(debug.selectedFrame,-7);XCTAssertEqual(debug.line(in:"main.py"),0)
        debug.readFrameLocals();let locals=requests.last!;XCTAssertEqual(locals["thread"] as? Int64,-1);XCTAssertEqual(locals["frame"] as? Int,-7)
        debug.receive(["event":"frame_variables","request_id":locals["request_id"]!,"session":"ours","stop":1,"thread":-1,"frame":-7,"variables":[]]);XCTAssertTrue(debug.frameVariablesCurrent)
        debug.receive(stopped(2,0,0));XCTAssertEqual(debug.selectedThread,0);XCTAssertEqual(debug.selectedFrame,0);debug.evaluate("zero");let zero=requests.last!
        debug.receive(["event":"value","request_id":zero["request_id"]!,"session":"ours","stop":2,"thread":0,"frame":0,"expression":"zero","value":"accepted"]);XCTAssertEqual(debug.values["zero"],"accepted")
        var current=2
        let malformed:[Any]=[true,1.5,Int64(Int32.max)+1]
        for bad in malformed {
            debug.readThreads();debug.receive(["event":"threads","request_id":requests.last!["request_id"]!,"session":"ours","stop":current,"selected":0,"threads":[["id":bad,"name":"malformed"]]])
            XCTAssertTrue(debug.threads.isEmpty);XCTAssertFalse(debug.threadBusy)
            debug.receive(stopped(current+1,bad,0));XCTAssertFalse(debug.canResume);XCTAssertFalse(debug.canReadThreads);XCTAssertNil(debug.line(in:"main.py"))
            debug.receive(stopped(current+2,0,0));current+=2;XCTAssertTrue(debug.canResume)
            debug.receive(stopped(current+1,0,bad));XCTAssertFalse(debug.canInspectFrame);XCTAssertFalse(debug.frameVariablesCurrent);XCTAssertNil(debug.line(in:"main.py"))
            debug.receive(stopped(current+2,0,0));current+=2
        }
        debug.close()
    }
    @MainActor func testAdapterSourceReferenceCannotBorrowPrimaryDiskProof()async throws {
        let engine=try Engine(address:"http://fixture.invalid"),space=Workspace(),debug=NativeDebugger(engine:engine);space.engine=engine;space.debugger=debug
        let text="value = 1\n",file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:text,hash:"saved"))
        space.buffers[file.path]=buffer;space.rootFiles=[file];debug.owner=file.path;debug.launchSource=text;let launch=debug.beginLaunchRequest();var location:NativeDebugLocation?,reveals=0
        debug.onLocation={location=$0};buffer.revealDebugLine={_ in reveals+=1;return true};debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]])
        func stopped(_ stop:Int,_ reference:Int)->[String:Any] {["event":"stopped","session":"ours","stop":stop,"thread":1,"frames":[["id":11,"name":"adapter source","source":"main.py","source_line":0,"source_hash":nativeSourceHash(text),"source_reference":reference]],"variables":[]]}
        debug.receive(stopped(1,17));XCTAssertFalse(debug.sourceMatches(buffer));XCTAssertNil(debug.line(in:"main.py"));let adapter=try XCTUnwrap(location);XCTAssertNil(adapter.sourceHash);XCTAssertEqual(adapter.sourceReference,17)
        await space.revealDebug(adapter,debug:debug);XCTAssertEqual(space.active,"main.py");XCTAssertEqual(reveals,0);XCTAssertTrue(space.status.contains("adapter-provided source reference"));XCTAssertEqual(buffer.text,text)
        debug.receive(stopped(2,0));let disk=try XCTUnwrap(location);await space.revealDebug(disk,debug:debug);XCTAssertEqual(reveals,1);XCTAssertTrue(debug.sourceMatches(buffer));debug.close()
    }

    @MainActor func testRealInspectorSurvivesRetiredGutterRowAndDirtyEditor()async throws {
        #if os(macOS)
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);NSApp.activate(ignoringOtherApps:true)
        let workspace=Workspace(),engine=try Engine(address:"http://fixture.invalid"),debug=NativeDebugger(engine:engine)
        let source="value = 1\nprint(value)\n",file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"saved"))
        workspace.buffers[file.path]=buffer;workspace.rootFiles=[file];workspace.debugger=debug;workspace.show(file.path)
        debug.owner=file.path;debug.launchSource=source;debug.sourceBuffers={ [buffer] };let launch=debug.beginLaunchRequest()
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)}
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]])
        debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[["id":11,"name":"main frame","source":"main.py","source_line":1]],"variables":[]])
        let host=NSHostingView(rootView:LanguageEditor(buffer:buffer,debugger:debug)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:480),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded();defer{debug.close();window.orderOut(nil)}
        func editor(_ view:NSView)->NSTextView? {if let text=view as? NSTextView,text.string==buffer.text{return text};return view.subviews.lazy.compactMap{editor($0)}.first}
        func children(_ object:NSObject)->[Any] {((object as? any NSAccessibilityProtocol)?.accessibilityChildren() ?? []) + (object.accessibilityAttributeValue(.children) as? [Any] ?? [])}
        func role(_ object:NSObject)->String {((object as? any NSAccessibilityProtocol)?.accessibilityRole()?.rawValue) ?? object.accessibilityAttributeValue(.role) as? String ?? ""}
        func label(_ object:NSObject)->String {((object as? any NSAccessibilityProtocol)?.accessibilityLabel()) ?? ((object as? any NSAccessibilityProtocol)?.accessibilityTitle()) ?? object.accessibilityAttributeValue(.description) as? String ?? object.accessibilityAttributeValue(.title) as? String ?? ""}
        func press(_ object:NSObject)->Bool {if let node=object as? any NSAccessibilityProtocol{return node.accessibilityPerformPress()};guard object.accessibilityActionNames().contains(.press) else{return false};object.accessibilityPerformAction(.press);return true}
        func elements()->[NSObject] {
            var pending:[Any]=[NSApp,host]+NSApp.windows.filter{$0.isVisible},seen=Set<ObjectIdentifier>(),result:[NSObject]=[],visits=0
            while !pending.isEmpty,visits<10000 {visits+=1;let item=pending.removeLast();guard let object=item as? NSObject,seen.insert(ObjectIdentifier(object)).inserted else{continue};if let window=object as? NSWindow,let content=window.contentView{pending.append(content)};if let view=object as? NSView{pending += view.subviews};result.append(object);pending += children(object)}
            return result
        }
        func pumpNativeEvents(){host.layoutSubtreeIfNeeded();RunLoop.current.run(until:Date().addingTimeInterval(0.01));NSAccessibility.post(element:host,notification:.layoutChanged)}
        func button(_ name:String)->NSObject? {elements().first{role($0)==NSAccessibility.Role.button.rawValue && label($0).contains(name)}}
        func waitButton(_ name:String)async throws->NSObject {for _ in 0..<200 {pumpNativeEvents();if let found=button(name){return found};try await Task.sleep(for:.milliseconds(10))};throw NSError(domain:"NativeDebugView",code:1,userInfo:[NSLocalizedDescriptionKey:"Debugger button not reachable: "+name+" · "+debug.status+" · bound "+String(debug.sourceMatches(buffer))+" · rows "+String(describing:buffer.visibleDebugLines.map{$0.line})+" · tree "+elements().prefix(40).map{String(describing:type(of:$0))+":"+role($0)+":"+label($0)}.joined(separator:" | ")])}
        // An independently mounted stock SwiftUI button distinguishes an
        // unavailable host AX bridge from a missing product control.
        let baselineHost=NSHostingView(rootView:Button("Native debug AX baseline"){}),baselineWindow=NSWindow(contentRect:NSRect(x:720,y:0,width:220,height:80),styleMask:[.titled],backing:.buffered,defer:false)
        baselineWindow.contentView=baselineHost;baselineWindow.makeKeyAndOrderFront(nil);baselineHost.layoutSubtreeIfNeeded();defer{baselineWindow.orderOut(nil)}
        for _ in 0..<100 {pumpNativeEvents();baselineHost.layoutSubtreeIfNeeded();if button("Native debug AX baseline") != nil{break};try await Task.sleep(for:.milliseconds(10))}
        guard button("Native debug AX baseline") != nil else{throw XCTSkip("This XCTest host exposes native text/window AX but no independently mounted stock SwiftUI Button through modern or legacy AX children. Renderer/AX reachability acceptance remains unverified.")}
        baselineWindow.orderOut(nil);window.makeKeyAndOrderFront(nil)
        for _ in 0..<200 {if editor(host) != nil{break};try await Task.sleep(for:.milliseconds(10))}
        let text=try XCTUnwrap(editor(host)),scroll=try XCTUnwrap(text.enclosingScrollView);text.setSelectedRange(NSRange(location:2,length:0));let geometry=scroll.convert(scroll.bounds,to:host)
        let overflow=try await waitButton("More gutter icons on line 2");XCTAssertTrue(press(overflow))
        let inspect=try await waitButton("Inspect on line 2");XCTAssertTrue(press(inspect))
        let read=try await waitButton("Read threads");XCTAssertTrue(press(read))
        for _ in 0..<200 {if requests.last?["op"] as? String=="threads"{break};try await Task.sleep(for:.milliseconds(10))}
        let readID=try XCTUnwrap(requests.last?["request_id"] as? String)
        debug.receive(["event":"threads","request_id":readID,"session":"ours","stop":1,"selected":1,"threads":[["id":1,"name":"main"],["id":2,"name":"worker"]]])
        let worker=try await waitButton("worker (2)");XCTAssertTrue(press(worker))
        for _ in 0..<200 {if requests.last?["op"] as? String=="select_thread"{break};try await Task.sleep(for:.milliseconds(10))}
        let selected=try XCTUnwrap(requests.last?["request_id"] as? String)
        debug.receive(["event":"stopped","session":"ours","stop":2,"thread":2,"frames":[["id":22,"name":"external worker frame","source":"/external/runtime.py","source_line":4]],"variables":[]])
        debug.receive(["event":"threads","request_id":selected,"session":"ours","stop":2,"selected":2,"threads":[["id":1,"name":"main"],["id":2,"name":"worker"]]])
        XCTAssertNil(debug.line(in:"main.py"));_ = try await waitButton("Read threads");_ = try await waitButton("external worker frame");_ = try await waitButton("Read selected-frame variables");_ = try await waitButton("Stop")
        XCTAssertTrue(editor(host) === text);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertEqual(text.selectedRange(),NSRange(location:2,length:0))
        text.insertText("# retained draft\n",replacementRange:NSRange(location:text.string.utf16.count,length:0));let draft=text.string,caret=text.selectedRange(),undo=try XCTUnwrap(text.undoManager)
        XCTAssertTrue(undo.canUndo);XCTAssertTrue(buffer.dirty)
        let locals=try await waitButton("Read selected-frame variables");XCTAssertTrue(press(locals))
        for _ in 0..<200 {if requests.last?["op"] as? String=="state"{break};try await Task.sleep(for:.milliseconds(10))}
        let localID=try XCTUnwrap(requests.last?["request_id"] as? String)
        debug.receive(["event":"frame_variables","request_id":localID,"session":"ours","stop":2,"thread":2,"frame":22,"variables":[["name":"alive","value":"true","variables_reference":0]]])
        _ = try await waitButton("main (1)");XCTAssertTrue(editor(host) === text);XCTAssertEqual(buffer.text,draft);XCTAssertEqual(text.selectedRange(),caret);XCTAssertTrue(undo.canUndo);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry)
        // With no mapped pause, the same launch slot must still expose the
        // retained session after the original inspection popover is dismissed.
        let escape=try XCTUnwrap(NSEvent.keyEvent(with:.keyDown,location:.zero,modifierFlags:[],timestamp:ProcessInfo.processInfo.systemUptime,windowNumber:NSApp.keyWindow?.windowNumber ?? window.windowNumber,context:nil,characters:"\u{1b}",charactersIgnoringModifiers:"\u{1b}",isARepeat:false,keyCode:53))
        NSApp.keyWindow?.sendEvent(escape);try await Task.sleep(for:.milliseconds(50))
        let launchSlot=try await waitButton("Inspect debugger on line 1");XCTAssertTrue(press(launchSlot));_ = try await waitButton("Read threads");_ = try await waitButton("Stop")
        XCTAssertTrue(editor(host) === text);XCTAssertEqual(buffer.text,draft);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry)
        text.insertText("# retained runtime, no launchable code\n",replacementRange:NSRange(location:0,length:text.string.utf16.count));let comments=text.string
        NSApp.keyWindow?.sendEvent(escape);try await Task.sleep(for:.milliseconds(50))
        let retainedSlot=try await waitButton("Inspect debugger on line 1");XCTAssertTrue(press(retainedSlot));_ = try await waitButton("Stop");_ = try await waitButton("Read threads")
        XCTAssertTrue(editor(host) === text);XCTAssertEqual(buffer.text,comments);XCTAssertTrue(undo.canUndo);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry)
        #endif
    }

    @MainActor func testRetiredSameDocumentStartupCannotAdoptNewLaunchOrStopForeignSession()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py"
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)}
        let old=debug.beginLaunchRequest();debug.stop();debug.close();let current=debug.beginLaunchRequest()
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":old,"doc":"hick:///main.py","session":"retired","breakpoints":[]])
        XCTAssertEqual(requests.last?["op"] as? String,"stop");XCTAssertEqual(requests.last?["session"] as? String,"retired");XCTAssertEqual(debug.status,"starting");XCTAssertNil(debug.sessionID)
        let count=requests.count
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":"foreign","doc":"hick:///main.py","session":"foreign-session","breakpoints":[]]);XCTAssertEqual(requests.count,count)
        debug.receive(["event":"build","request_id":old,"doc":"hick:///main.py","events":[["data":"old build"]]])
        debug.receive(["event":"failed","request_id":old,"doc":"hick:///main.py","session":NSNull(),"about":"start","message":"old failure"])
        XCTAssertTrue(debug.message.isEmpty);XCTAssertEqual(debug.status,"starting")
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":current,"doc":"hick:///main.py","session":"current","breakpoints":[]]);XCTAssertEqual(debug.sessionID,"current");XCTAssertEqual(debug.status,"running")
        debug.receive(["event":"build","request_id":old,"doc":"hick:///main.py","events":[["data":"late old build"]]]);XCTAssertTrue(debug.message.isEmpty)
        debug.close();let forgotten=debug.beginLaunchRequest();for _ in 0..<65 {_ = debug.beginLaunchRequest()}
        let bounded=requests.count;debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":forgotten,"doc":"hick:///main.py","session":"no-longer-owned","breakpoints":[]]);XCTAssertEqual(requests.count,bounded);XCTAssertEqual(debug.status,"starting");debug.close()
    }

    @MainActor func testDebugSourceReadsFullRootButNeverLearnsPostStopDiskAuthority()async throws {
        let engine=try Engine(address:"http://fixture.invalid"),space=Workspace(),debug=NativeDebugger(engine:engine)
        space.engine=engine;space.debugger=debug;debug.owner="main.py";debug.launchSource="primary\n";_ = debug.beginLaunchRequest()
        let text="helper\nsecond\n",file=FileNode(name:"helper.py",path:"folder/helper.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:text,hash:"saved"))
        buffer.selection=NSRange(location:2,length:0);space.buffers[file.path]=buffer;space.rootFiles=[FileNode(name:"folder",path:"folder",dir:true,children:[file])];space.files=[]
        var locations:[NativeDebugLocation]=[],revealed:[Int]=[];debug.onLocation={locations.append($0)};buffer.revealDebugLine={revealed.append($0);return true}
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":debug.launchRequestID ?? "missing","doc":"hick:///main.py","session":"ours","breakpoints":[]])
        func stopped(_ stop:Int,_ hash:String?=nil)->[String:Any] {var frame:[String:Any]=["id":11,"name":"helper","source":file.path,"source_line":1];if let hash{frame["source_hash"]=hash};return ["event":"stopped","session":"ours","stop":stop,"thread":1,"frames":[frame],"variables":[]]}
        debug.receive(stopped(1));let unbound=try XCTUnwrap(locations.last);await space.revealDebug(unbound,debug:debug)
        XCTAssertEqual(space.active,file.path);XCTAssertTrue(revealed.isEmpty);XCTAssertFalse(debug.sourceMatches(buffer));XCTAssertTrue(space.status.contains("no captured source revision"));XCTAssertEqual(buffer.selection,NSRange(location:2,length:0))
        debug.receive(stopped(2,nativeSourceHash(text)));let bound=try XCTUnwrap(locations.last);await space.revealDebug(bound,debug:debug)
        XCTAssertEqual(revealed,[1]);XCTAssertTrue(debug.sourceMatches(buffer));XCTAssertEqual(buffer.text,text)
        debug.receive(stopped(3));XCTAssertFalse(debug.sourceMatches(buffer));XCTAssertNil(locations.last?.sourceHash,"An unknown frame must not borrow an earlier frame’s captured hash.")
        debug.receive(stopped(4,nativeSourceHash(text)));let current=try XCTUnwrap(locations.last)
        buffer.text += "# newer unsaved text\n";await space.revealDebug(current,debug:debug);XCTAssertEqual(revealed,[1]);XCTAssertTrue(buffer.text.hasSuffix("# newer unsaved text\n"));XCTAssertTrue(space.status.contains("without a position"))
        debug.close()
    }
    @MainActor func testDelayedDebugRevealRejectsRetiredStopAndUserNavigation()async throws {
        let engine=try Engine(address:"http://fixture.invalid"),space=Workspace(),debug=NativeDebugger(engine:engine);space.engine=engine;space.debugger=debug
        debug.owner="main.py";debug.launchSource="first\nsecond\n";_ = debug.beginLaunchRequest()
        let file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:debug.launchSource,hash:"saved"))
        space.buffers[file.path]=buffer;space.rootFiles=[file];space.files=[]
        var location:NativeDebugLocation?,revealed=0;debug.onLocation={location=$0}
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":debug.launchRequestID ?? "missing","doc":"hick:///main.py","session":"ours","breakpoints":[]])
        func stopped(_ stop:Int)->[String:Any] {["event":"stopped","session":"ours","stop":stop,"thread":1,"frames":[["id":11,"name":"main","source":"main.py","source_line":1]],"variables":[]]}
        debug.receive(stopped(1));let old=try XCTUnwrap(location)
        let waiting=Task {await space.revealDebug(old,debug:debug)};for _ in 0..<200 {if space.active=="main.py"{break};await Task.yield()};XCTAssertEqual(space.active,"main.py")
        debug.receive(stopped(2));buffer.revealDebugLine={_ in revealed+=1;return true};await waiting.value;XCTAssertEqual(revealed,0);XCTAssertEqual(space.active,"main.py")
        buffer.revealDebugLine=nil;space.show("files");let next=try XCTUnwrap(location),userNavigation=Task {await space.revealDebug(next,debug:debug)}
        for _ in 0..<200 {if space.active=="main.py"{break};await Task.yield()};XCTAssertEqual(space.active,"main.py");space.show("agent");buffer.revealDebugLine={_ in revealed+=1;return true};await userNavigation.value;XCTAssertEqual(revealed,0);XCTAssertEqual(space.active,"agent")
        await space.revealDebug(old,debug:debug);XCTAssertEqual(space.active,"agent");debug.close()
    }

    @MainActor func testSelectedFrameLocalsRequireLatestFrameScopeAndPreserveFailure()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";_ = debug.beginLaunchRequest()
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)}
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":debug.launchRequestID ?? "missing","doc":"hick:///main.py","session":"ours","breakpoints":[]])
        let first:[String:Any]=["id":11,"name":"main"],second:[String:Any]=["id":12,"name":"caller"]
        debug.receive(["event":"stopped","session":"ours","stop":1,"thread":7,"frames":[first,second],"variables":[["name":"top","value":"old"]]])
        func locals(_ request:[String:Any],_ frame:Int,_ value:String,_ stop:Int=1)->[String:Any] {["event":"frame_variables","session":"ours","stop":stop,"thread":7,"frame":frame,"request_id":request["request_id"]!,"variables":[["name":"local","value":value,"variables_reference":0]]]}
        debug.select(second,snapshot:debug.frameSnapshot);let old=requests.last!;XCTAssertEqual(old["op"] as? String,"state");XCTAssertEqual(old["frame"] as? Int,12);XCTAssertTrue(debug.frameBusy)
        debug.select(first,snapshot:debug.frameSnapshot);let current=requests.last!;debug.receive(locals(old,12,"wrong caller"));XCTAssertTrue(debug.frameBusy);XCTAssertEqual(debug.variables.first?["name"] as? String,"top")
        debug.receive(locals(current,11,"fresh top"));XCTAssertFalse(debug.frameBusy);XCTAssertTrue(debug.frameVariablesCurrent);XCTAssertEqual(debug.variables.first?["value"] as? String,"fresh top")
        debug.select(second,snapshot:debug.frameSnapshot);let refused=requests.last!;debug.receive(["event":"failed","session":"ours","about":"state","request_id":refused["request_id"]!,"message":"Provider cannot materialize this frame"])
        XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.frames.count,2);XCTAssertEqual(debug.selectedFrame,12);XCTAssertFalse(debug.frameBusy);XCTAssertFalse(debug.frameVariablesCurrent);XCTAssertTrue(debug.frameNotice.contains("selected stack and source are retained"))
        debug.readFrameLocals();let held=requests.last!;debug.receive(["event":"stopped","session":"ours","stop":2,"thread":7,"frames":[first],"variables":[["name":"new stop","value":"2"]]])
        debug.receive(locals(held,12,"stale reply"));XCTAssertEqual(debug.variables.first?["name"] as? String,"new stop");XCTAssertFalse(debug.frameBusy);debug.close()
    }
    @MainActor func testStackAndVariablesErrorsStayExplicitAndAllowThreadEscape()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";_ = debug.beginLaunchRequest()
        var requests:[[String:Any]]=[],locations:[String]=[];debug.emitForTest={requests.append($0)};debug.onLocation={location in locations.append(location.path)}
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":debug.launchRequestID ?? "missing","doc":"hick:///main.py","session":"ours","breakpoints":[]])
        let frame:[String:Any]=["id":11,"name":"main","source":"main.py","source_line":2]
        debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[frame],"variables":[["name":"value","value":"1"]]])
        let count=locations.count
        debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[],"variables":[],"stack_error":"Adapter stack unavailable","variables_error":"No materialized frame"])
        XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.frames.count,1);XCTAssertEqual(locations.count,count);XCTAssertNil(debug.line(in:"main.py"));XCTAssertFalse(debug.canInspectFrame);XCTAssertFalse(debug.canEvaluate);XCTAssertFalse(debug.frameVariablesCurrent)
        XCTAssertTrue(debug.stackNotice.contains("Earlier frames"));XCTAssertTrue(debug.frameNotice.contains("Provider variables read refused"));XCTAssertTrue(debug.canReadThreads)
        debug.readThreads();debug.receive(["event":"threads","session":"ours","request_id":requests.last!["request_id"]!,"stop":1,"selected":1,"threads":[["id":1,"name":"main"],["id":2,"name":"worker"]]])
        debug.selectThread(2,snapshot:debug.threadSnapshot);XCTAssertEqual(requests.last?["op"] as? String,"select_thread")
        let selected=requests.last!["request_id"]!
        debug.receive(["event":"stopped","session":"ours","stop":2,"thread":2,"frames":[["id":22,"name":"worker"]],"variables":[]])
        debug.receive(["event":"threads","session":"ours","request_id":selected,"stop":2,"selected":2,"threads":[["id":1,"name":"main"],["id":2,"name":"worker"]]])
        XCTAssertEqual(debug.selectedThread,2);XCTAssertTrue(debug.stackNotice.isEmpty);XCTAssertTrue(debug.canInspectFrame);XCTAssertTrue(debug.frameVariablesCurrent);debug.close()
    }

    @MainActor func testThreadReadsAndSelectionBindStopRequestAndRenderedSnapshot() throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";_ = debug.beginLaunchRequest()
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)}
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":debug.launchRequestID ?? "missing","doc":"hick:///main.py","session":"ours","breakpoints":[]])
        let first:[String:Any]=["id":11,"name":"main","source":"main.py","source_line":0]
        debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[first],"variables":[]])
        debug.readThreads();let request=try XCTUnwrap(requests.last?["request_id"] as? String)
        func threads(_ id:String,_ stop:Int=1,_ selected:Int=1)->[String:Any] {["event":"threads","session":"ours","request_id":id,"stop":stop,"selected":selected,"threads":[["id":1,"name":"main"],["id":2,"name":"worker"]]]}
        debug.receive(threads("retired"));XCTAssertTrue(debug.threads.isEmpty);XCTAssertTrue(debug.threadBusy)
        debug.receive(threads(request));XCTAssertEqual(debug.threads.map{$0.id},[1,2]);XCTAssertFalse(debug.threadBusy)
        let stale=debug.threadSnapshot;debug.readThreads();let newer=try XCTUnwrap(requests.last?["request_id"] as? String);debug.receive(threads(request));XCTAssertTrue(debug.threadBusy);debug.receive(threads(newer))
        let count=requests.count;debug.selectThread(2,snapshot:stale);XCTAssertEqual(requests.count,count)
        debug.selectThread(2,snapshot:debug.threadSnapshot);let selection=try XCTUnwrap(requests.last?["request_id"] as? String)
        XCTAssertEqual(requests.last?["op"] as? String,"select_thread");XCTAssertEqual(requests.last?["stop"] as? UInt64,1);XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.frames.first?["id"] as? Int,11);XCTAssertEqual(debug.selectedThread,1)
        debug.receive(["event":"failed","session":"ours","about":"select_thread","request_id":"retired","message":"old failure"]);XCTAssertTrue(debug.threadBusy)
        debug.receive(["event":"failed","session":"ours","about":"select_thread","request_id":selection,"message":"Thread is running"]);XCTAssertFalse(debug.threadBusy);XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.frames.first?["id"] as? Int,11)
        debug.selectThread(2,snapshot:debug.threadSnapshot);let confirmed=try XCTUnwrap(requests.last?["request_id"] as? String)
        debug.receive(["event":"stopped","session":"ours","stop":2,"thread":2,"frames":[["id":22,"name":"worker","source":"worker.py","source_line":3]],"variables":[]])
        XCTAssertTrue(debug.threadBusy);debug.receive(threads(confirmed,2,2));XCTAssertEqual(debug.selectedThread,2);XCTAssertEqual(debug.threadStop,2);XCTAssertFalse(debug.threadBusy)
        debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[first],"variables":[]]);XCTAssertEqual(debug.frames.first?["id"] as? Int,22)
        debug.step("over");XCTAssertEqual(debug.status,"running");XCTAssertTrue(debug.threads.isEmpty);debug.receive(["event":"stopped","session":"ours","stop":2,"thread":2,"frames":[first],"variables":[]]);XCTAssertEqual(debug.status,"running");debug.receive(threads(confirmed,2,2));XCTAssertTrue(debug.threads.isEmpty)
        debug.close()
    }
    @MainActor func testEvaluationRequiresCurrentStopThreadFrameAndLatestExpressionRequest() throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";_ = debug.beginLaunchRequest()
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)}
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":debug.launchRequestID ?? "missing","doc":"hick:///main.py","session":"ours","breakpoints":[]])
        let first:[String:Any]=["id":11,"name":"main"],second:[String:Any]=["id":12,"name":"caller"]
        debug.receive(["event":"stopped","session":"ours","stop":10,"thread":7,"frames":[first,second],"variables":[]])
        func response(_ request:[String:Any],_ text:String,_ stop:Int=10,_ thread:Int=7,_ frame:Int=11)->[String:Any] {["event":"value","session":"ours","expression":"value","value":text,"stop":stop,"thread":thread,"frame":frame,"request_id":request["request_id"]!]}
        debug.evaluate("value");let old=requests.last!;debug.evaluate("value");let current=requests.last!
        XCTAssertEqual(current["stop"] as? UInt64,10);XCTAssertEqual(current["thread"] as? Int64,7);XCTAssertEqual(current["frame"] as? Int,11)
        debug.receive(response(old,"old"));XCTAssertNil(debug.values["value"]);debug.receive(response(current,"current"));XCTAssertEqual(debug.values["value"],"current")
        debug.evaluate("unsupported");let failure=requests.last!;debug.receive(["event":"failed","session":"ours","about":"eval","request_id":failure["request_id"]!,"message":"Provider cannot evaluate this frame"])
        XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.selectedFrame,11);XCTAssertEqual(debug.frames.count,2);XCTAssertTrue(debug.evaluationNotice.contains("current stopped frame is retained"))
        debug.evaluate("value");let formerFrame=requests.last!,snapshot=debug.frameSnapshot;debug.select(second,snapshot:snapshot);debug.receive(response(formerFrame,"wrong frame"));XCTAssertNil(debug.values["value"])
        debug.evaluate("value");let selected=requests.last!;debug.receive(response(selected,"wrong stop",9,7,12));XCTAssertNil(debug.values["value"]);debug.receive(response(selected,"caller value",10,7,12));XCTAssertEqual(debug.values["value"],"caller value")
        debug.receive(["event":"stopped","session":"ours","stop":11,"thread":8,"frames":[second],"variables":[]]);debug.select(first,snapshot:snapshot);XCTAssertEqual(debug.selectedFrame,12);XCTAssertTrue(debug.values.isEmpty)
        debug.receive(response(selected,"late",10,7,12));XCTAssertTrue(debug.values.isEmpty);debug.close()
    }
    @MainActor func testThreadPagingAndMalformedReadKeepPausedFrames() throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";_ = debug.beginLaunchRequest()
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)}
        debug.receive(["event":"started","source_hash":nativeSourceHash(debug.launchSource),"request_id":debug.launchRequestID ?? "missing","doc":"hick:///main.py","session":"ours","breakpoints":[]]);debug.receive(["event":"stopped","session":"ours","stop":1,"thread":0,"frames":[["id":11,"name":"main"]],"variables":[]])
        debug.readThreads();debug.receive(["event":"threads","session":"ours","request_id":requests.last!["request_id"]!,"stop":1,"selected":0,"threads":(0..<201).map{["id":$0,"name":"Thread \($0)"]}])
        XCTAssertEqual(debug.threadPages,3);XCTAssertEqual(debug.visibleThreads.count,100);debug.threadPage=2;XCTAssertEqual(debug.visibleThreads.map{$0.id},[200])
        debug.readThreads();debug.receive(["event":"threads","session":"ours","request_id":requests.last!["request_id"]!,"stop":1,"selected":0,"threads":[["id":1,"name":"one"],["id":1,"name":"duplicate"]]])
        XCTAssertEqual(debug.threads.count,201);XCTAssertFalse(debug.threadBusy);XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.frames.first?["id"] as? Int,11);XCTAssertTrue(debug.threadNotice.contains("duplicate"))
        debug.readThreads();debug.receive(["event":"failed","session":"ours","request_id":requests.last!["request_id"]!,"about":"threads","message":"Adapter unavailable"])
        XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.frames.first?["id"] as? Int,11);XCTAssertEqual(debug.threads.count,201);XCTAssertFalse(debug.threadBusy);debug.close()
    }

    @MainActor func testOutputIsReadOnlySourceAnchoredAndRevisionBound() throws {
        // Contract: .hickory-source/frontends/contract.md
        let source = "value = 1\nprint(value)\n"
        let buffer = DocumentBuffer(file: FileNode(name: "main.py", path: "main.py", dir: false), text: FileText(path: "main.py", content: source, hash: "base"))
        let debug = NativeDebugger(engine: try Engine(address: "http://fixture.invalid"))
        debug.owner = "main.py"; debug.launchSource = source; debug.sourceBuffers={ [buffer] }; _ = debug.beginLaunchRequest()
        debug.receive(["event": "started", "source_hash":nativeSourceHash(debug.launchSource), "request_id":debug.launchRequestID ?? "missing", "doc": "hick:///main.py", "session": "ours", "breakpoints": []])
        debug.receive(["event": "run_output", "session": "old", "events": [["text": "wrong"]]])
        XCTAssertTrue(debug.output.isEmpty)
        debug.receive(["event": "run_output", "session": "ours", "events": [["category": "stdout", "text": "1\n"], ["category": "stderr", "text": "warning\n", "source": "main.py", "line": 1]]])
        XCTAssertEqual(debug.outputPlacements(buffer)[1], "1\nwarning\n")
        XCTAssertEqual(debug.output.last?.source, "main.py"); XCTAssertEqual(debug.output.last?.line, 1)
        XCTAssertEqual(buffer.text, source); XCTAssertFalse(buffer.dirty)
        buffer.text += "# edited"
        XCTAssertTrue(debug.outputPlacements(buffer).isEmpty)
        let document = "<h:file path=\"main.py\">\nprint(1)\n\nprint(2)\n</h:file>\n"
        let book = DocumentBuffer(file: FileNode(name: "workbook.md", path: "workbook.md", dir: false), text: FileText(path: "workbook.md", content: document, hash: "base"))
        debug.owner = "workbook.md"; debug.launchSource = document; debug.sourceBuffers={ [book] }; _ = debug.beginLaunchRequest()
        debug.receive(["event": "started", "source_hash":nativeSourceHash(debug.launchSource), "request_id":debug.launchRequestID ?? "missing", "doc": "hick:///workbook.md", "session": "document", "breakpoints": []])
        debug.receive(["event": "run_output", "session": "document", "events": [["category": "stdout", "text": "1\n", "source": "workbook.md", "line": 1]]])
        XCTAssertNotNil(debug.outputPlacements(book)[4], "A blank line must not split the producing file block")
    }
    @MainActor func testRecordedOutputUsesByteSpanAfterUnicodeBlock() {
        let source = "<h:exec>\necho café\n</h:exec>\n"
        let buffer = DocumentBuffer(file: FileNode(name: "workbook.md", path: "workbook.md", dir: false), text: FileText(path: "workbook.md", content: source, hash: "base"))
        buffer.receipts = [InlineReceipt(kind: "exec", span: [0, source.utf8.count - 1], command: nil, status: nil, transcript: [InlineReceiptEvent(kind: "out", data: "café\n")], recorded_output: nil)]
        buffer.receiptSource = source
        XCTAssertEqual(buffer.recordedOutputPlacements(), [2: "café\n"])
        XCTAssertEqual(buffer.text, source)
        buffer.text += "changed"
        XCTAssertTrue(buffer.recordedOutputPlacements().isEmpty)
    }

    @MainActor func testNativeDebugRejectsOtherSessionAndCarriesBoundCondition() throws {
        let debug = NativeDebugger(engine: try Engine(address: "http://fixture.invalid"))
        debug.owner = "main.py"; debug.launchSource = "value = 1\nprint(value)\n"; _ = debug.beginLaunchRequest()
        var requests: [[String: Any]] = []; debug.emitForTest = { requests.append($0) }
        debug.toggle("main.py", line: 0, condition: "value > 0",source:debug.launchSource)
        debug.receive(["event": "started", "source_hash":nativeSourceHash(debug.launchSource), "request_id":debug.launchRequestID ?? "missing", "doc": "hick:///other.py", "session": "wrong"])
        XCTAssertNil(debug.sessionID)
        debug.receive(["event": "started", "source_hash":nativeSourceHash(debug.launchSource), "request_id":debug.launchRequestID ?? "missing", "doc": "hick:///main.py", "session": "ours", "breakpoints": [["line": 0, "moved_to": 1, "state": "bound"]]])
        XCTAssertEqual(debug.breakpoints["main.py"]?.first?["condition"] as? String, "value > 0")
        XCTAssertEqual(debug.breakpoints["main.py"]?.first?["line"] as? Int, 0,"Uncorrelated launch receipt must not move retained intentions")
        var revealed = ""; debug.onLocation = {location in revealed="\(location.path):\(location.line)"}
        debug.receive(["event": "stopped", "session": "old", "line": 0]); XCTAssertEqual(debug.status, "running")
        debug.receive(["event": "stopped", "session": "ours", "stop":1,"thread":1,"line": NSNull(), "frames": [["id": 7, "name": "helper", "source": "helper.py", "source_line": 2,"source_hash":nativeSourceHash("before")]], "variables": [["name": "value", "value": "1"]]])
        XCTAssertEqual(revealed, "helper.py:2"); XCTAssertEqual(debug.line(in: "helper.py"), 2)
        let helper = DocumentBuffer(file: FileNode(name: "helper.py", path: "helper.py", dir: false), text: FileText(path: "helper.py", content: "before", hash: "base"))
        XCTAssertTrue(debug.admitSourceReading(helper,path:helper.file.path,proof:nativeSourceHash(helper.text)));XCTAssertTrue(debug.sourceMatches(helper))
        helper.text = "saved replacement"; helper.base = helper.text
        XCTAssertFalse(debug.sourceMatches(helper), "Saving changed callee text must not reattach old positional values")
        debug.evaluate("value", watch: true); XCTAssertEqual(requests.last?["frame"] as? Int, 7)
        debug.step("over"); XCTAssertEqual(requests.last?["how"] as? String, "over"); XCTAssertTrue(debug.variables.isEmpty)
        debug.receive(["event": "value", "session": "ours", "expression": "value", "value": "old"]); XCTAssertTrue(debug.values.isEmpty)
        debug.receive(["event": "finished", "session": "ours"]); debug.step("over"); XCTAssertNil(debug.sessionID); XCTAssertEqual(debug.status, "finished")
    }
    @MainActor func testStopWhileStartingStopsTheAcknowledgedSession() throws {
        let debug = NativeDebugger(engine: try Engine(address: "http://fixture.invalid"))
        debug.owner = "main.py"; _ = debug.beginLaunchRequest()
        var requests: [[String: Any]] = []; debug.emitForTest = { requests.append($0) }
        debug.stop(); XCTAssertEqual(debug.status, "stopping"); XCTAssertTrue(requests.isEmpty)
        debug.receive(["event": "started", "source_hash":nativeSourceHash(debug.launchSource), "request_id":debug.launchRequestID ?? "missing", "doc": "hick:///main.py", "session": "late", "breakpoints": []])
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
        ResponseProtocol.response = Data(#"{"debug_protocol":3,"profiles":[{"id":"other","name":"Other","can_debug":true,"program":"other.py"},{"id":"python","name":"Python","can_debug":true,"program":"main.py"}],"fingerprint":"current"}"#.utf8)
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
        debug.toggle(buffer, line: 1); await debug.prepare(buffer)
        for _ in 0..<200 { if debug.status == "paused" || debug.status == "failed" { break }; try await Task.sleep(for: .milliseconds(50)) }
        XCTAssertEqual(debug.status, "paused", debug.message)
        guard debug.status == "paused" else { return }
        debug.readThreads();for _ in 0..<200 {if !debug.threadBusy{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertFalse(debug.threads.isEmpty,debug.threadNotice)
        debug.toggle(buffer,line:1,condition:"value >= 1")
        for _ in 0..<200 {if debug.breakpointMarkers(buffer).first?["state"] as? String=="bound"{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.breakpointMarkers(buffer).first?["state"] as? String,"bound",debug.breakpointNotice)
        XCTAssertEqual(debug.breakpoints[file.path]?.first?["condition"] as? String,"value >= 1")
        debug.step("over")
        for _ in 0..<200 { if debug.status == "paused" || debug.status == "failed" { break }; try await Task.sleep(for: .milliseconds(50)) }
        XCTAssertEqual(debug.status, "paused", debug.message); XCTAssertFalse(debug.variables.isEmpty)
        debug.evaluate("value", watch: true)
        for _ in 0..<100 { if debug.values["value"] != nil { break }; try await Task.sleep(for: .milliseconds(50)) }
        XCTAssertNotNil(debug.values["value"]); XCTAssertEqual(buffer.text, loaded.content); XCTAssertFalse(buffer.dirty)
        let target=try XCTUnwrap(loaded.content.components(separatedBy:"\n").firstIndex(where:{$0.contains("# thread-stop")}))
        XCTAssertTrue(debug.canMoveToSource(buffer,how:"run_to"));debug.moveToSource(buffer,line:target,how:"run_to")
        for _ in 0..<400 {if debug.status=="paused" || debug.pauseUnconfirmed || debug.status=="failed"{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.status,"paused",debug.message);XCTAssertEqual(debug.line(in:file.path),target);debug.evaluate("main_value")
        for _ in 0..<200 {if debug.values["main_value"] != nil{break};try await Task.sleep(for:.milliseconds(50))};XCTAssertEqual(debug.values["main_value"],"7",debug.evaluationNotice)
        if debug.capabilities["goto_targets"] as? Bool==true {
            let rows=loaded.content.components(separatedBy:"\n"),jump=try XCTUnwrap(rows.indices.first(where:{$0>target && !rows[$0].trimmingCharacters(in:.whitespaces).isEmpty && !rows[$0].trimmingCharacters(in:.whitespaces).hasPrefix("#")}))
            XCTAssertTrue(debug.canMoveToSource(buffer,how:"jump"));debug.moveToSource(buffer,line:jump,how:"jump")
            for _ in 0..<400 {if debug.status=="paused" || debug.pauseUnconfirmed || debug.status=="failed"{break};try await Task.sleep(for:.milliseconds(50))}
            XCTAssertEqual(debug.status,"paused",debug.message);XCTAssertEqual(debug.line(in:file.path),jump,debug.message)
            print("Native v3 actual goto_targets advertised; requested line \(jump), reported \(String(describing:debug.line(in:file.path))), state \(debug.status).")
        }else{XCTAssertFalse(debug.canMoveToSource(buffer,how:"jump"));print("Native v3 actual adapter does not advertise goto_targets; Jump remains explicitly unavailable.")}
        debug.stop()
        for _ in 0..<100 { if !debug.active { break }; try await Task.sleep(for: .milliseconds(50)) }
        XCTAssertFalse(debug.active, debug.message)
    }
    @MainActor func testLivePausedResourceLineageAndMenuUndoRetainNativeEditor()async throws {
        #if os(macOS)
        guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_DEBUG_ORIGIN"] else{throw XCTSkip("Set HICKORY_NATIVE_DEBUG_ORIGIN to an isolated writable v3 fixture for paused resource acceptance.")}
        let engine=try Engine(address:origin),space=Workspace(),file=FileNode(name:"debug.py",path:"debug.py",dir:false),loaded=try await engine.file(file),buffer=DocumentBuffer(file:file,text:loaded),debug=NativeDebugger(engine:engine)
        space.engine=engine;space.buffers[file.path]=buffer;space.rootFiles=[file];space.debugger=debug;space.layout.activate(file.path,in:"document-group");debug.sourceBuffers={ [weak space] in guard let space else{return []};return Array(space.buffers.values)+Array(space.retainedResourceEditors.values)}
        _ = NSApplication.shared;let host=NSHostingView(rootView:NativeEditor(buffer:buffer)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:480),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
        defer{debug.close();window.orderOut(nil);for value in space.buffers.values{value.language?.close()}}
        func editor(_ view:NSView)->NSTextView? {if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{editor($0)}.first}
        for _ in 0..<200 {if editor(host) != nil{break};try await Task.sleep(for:.milliseconds(10))}
        let native=try XCTUnwrap(editor(host));native.insertText("# native undo proof\n",replacementRange:NSRange(location:native.string.utf16.count,length:0));await buffer.save(using:engine);XCTAssertFalse(buffer.dirty,buffer.error)
        native.setSelectedRange(NSRange(location:2,length:0));let manager=try XCTUnwrap(native.undoManager),scroll=try XCTUnwrap(native.enclosingScrollView),geometry=scroll.convert(scroll.bounds,to:host),body=buffer.text
        XCTAssertTrue(manager.canUndo);let line=try XCTUnwrap(body.components(separatedBy:"\n").firstIndex(where:{$0.contains("# thread-stop")}));var outbound:[[String:Any]]=[];debug.observeRequestForTest={outbound.append($0)}
        debug.toggle(buffer,line:line);await debug.prepare(buffer);for _ in 0..<400 {if debug.status=="paused" || debug.status=="failed"{break};try await Task.sleep(for:.milliseconds(50))};XCTAssertEqual(debug.status,"paused",debug.message);guard debug.status=="paused" else{return}
        let session=debug.sessionID,owner=debug.owner,launchText=debug.launchSource,held=try XCTUnwrap(debug.sourceLocation(path:file.path,line:line)),model=space.resourceRestoration
        await model.prepareEdit(space,title:"Move paused captured source",edit:["documentChanges":[["kind":"rename","oldUri":nativeDebugURI(file.path),"newUri":nativeDebugURI("moved-debug.py")]]]);XCTAssertNotNil(model.review,model.notice);await model.publish(space);XCTAssertFalse(model.hasPending,model.notice);XCTAssertTrue(space.buffers["moved-debug.py"] === buffer);XCTAssertFalse(debug.sourceMatches(buffer));XCTAssertFalse(debug.locationCurrent(held));XCTAssertTrue(debug.canResume)
        space.changes.open("undo",workspace:space);for _ in 0..<400 {if model.review != nil{break};try await Task.sleep(for:.milliseconds(25))};XCTAssertNotNil(model.review,model.notice);await model.publish(space);XCTAssertFalse(model.hasPending,model.notice);XCTAssertTrue(space.buffers[file.path] === buffer);XCTAssertFalse(debug.breakpointMarkers(buffer).first?["state"] as? String=="bound")
        space.changes.open("redo",workspace:space);for _ in 0..<400 {if model.review != nil{break};try await Task.sleep(for:.milliseconds(25))};XCTAssertNotNil(model.review,model.notice);await model.publish(space);XCTAssertFalse(model.hasPending,model.notice);XCTAssertTrue(space.buffers["moved-debug.py"] === buffer)
        let create:[String:Any]=["kind":"create","uri":nativeDebugURI(file.path)]
        let insertion:[String:Any]=["range":["start":["line":0,"character":0],"end":["line":0,"character":0]],"newText":body]
        let fill:[String:Any]=["textDocument":["uri":nativeDebugURI(file.path),"version":NSNull()],"edits":[insertion]]
        await model.prepareEdit(space,title:"Independent same-byte old spelling",edit:["documentChanges":[create,fill]]);XCTAssertNotNil(model.review,model.notice);await model.publish(space);XCTAssertFalse(model.hasPending,model.notice)
        let replacement=try XCTUnwrap(space.buffers[file.path]);XCTAssertFalse(replacement === buffer);XCTAssertEqual(replacement.text,body)
        let effects=outbound.filter{["start","breakpoints","source_breakpoints","jump","run_to"].contains($0["op"] as? String ?? "")}.count
        debug.toggle(replacement,line:line);debug.moveToSource(replacement,line:line,how:"jump");let frame=try XCTUnwrap(debug.frames.first);debug.select(frame,snapshot:debug.frameSnapshot)
        var reveals=0;replacement.revealDebugLine={_ in reveals+=1;return true};await space.revealDebug(try XCTUnwrap(debug.sourceLocation(path:file.path,line:line)),debug:debug)
        XCTAssertFalse(debug.sourceMatches(replacement));XCTAssertEqual(reveals,0);XCTAssertEqual(outbound.filter{["start","breakpoints","source_breakpoints","jump","run_to"].contains($0["op"] as? String ?? "")}.count,effects)
        XCTAssertEqual(debug.sessionID,session);XCTAssertEqual(debug.owner,owner);XCTAssertEqual(debug.launchSource,launchText);XCTAssertTrue(editor(host) === native);XCTAssertTrue(native.undoManager === manager);XCTAssertTrue(manager.canUndo);XCTAssertEqual(native.selectedRange(),NSRange(location:2,length:0));XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertEqual(buffer.text,body)
        debug.stop();for _ in 0..<200 {if !debug.active{break};try await Task.sleep(for:.milliseconds(50))};XCTAssertFalse(debug.active,debug.message)
        manager.undo();XCTAssertTrue(manager.canRedo);XCTAssertFalse(native.string.contains("# native undo proof"));XCTAssertEqual(buffer.text,native.string)
        #endif
    }
    @MainActor func testLiveNativeLiteralPhysicalSourceIdentity()async throws {
        guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_DEBUG_ORIGIN"] else{throw XCTSkip("Set HICKORY_NATIVE_DEBUG_ORIGIN for the real v3 literal physical-name fixture.")}
        let engine=try Engine(address:origin),path="%2F λ?#.py",file=FileNode(name:path,path:path,dir:false),loaded=try await engine.file(file),buffer=DocumentBuffer(file:file,text:loaded),debug=NativeDebugger(engine:engine);defer{debug.close()}
        buffer.selection=NSRange(location:2,length:0);debug.toggle(buffer,line:1);await debug.prepare(buffer)
        for _ in 0..<400 {if ["paused","failed","unavailable"].contains(debug.status){break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.status,"paused",debug.message);guard debug.status=="paused" else{return}
        XCTAssertTrue(debug.sourceMatches(buffer),debug.message);XCTAssertEqual(debug.breakpointMarkers(buffer).first?["state"] as? String,"bound",debug.breakpointNotice)
        debug.evaluate("value");for _ in 0..<200 {if debug.values["value"] != nil{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.values["value"],"1",debug.evaluationNotice);XCTAssertEqual(buffer.text,loaded.content);XCTAssertEqual(buffer.selection,NSRange(location:2,length:0));XCTAssertFalse(buffer.dirty)
        debug.stop();for _ in 0..<200 {if !debug.active{break};try await Task.sleep(for:.milliseconds(50))};XCTAssertFalse(debug.active,debug.message)
    }
    @MainActor func testLiveSourceAdjacentPytestDebugPreservesEditorAndRetainsSelection()async throws {
        #if os(macOS)
        guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEST_DEBUG_ORIGIN"] else{throw XCTSkip("Set HICKORY_NATIVE_TEST_DEBUG_ORIGIN to the isolated source-test Debug fixture.")}
        let engine=try Engine(address:origin),space=Workspace(),path=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEST_DEBUG_INPUT"] ?? "test_choice.py",tree=try await engine.files()
        func find(_ files:[FileNode])->FileNode?{for file in files{if exactUTF8(file.path,path){return file};if let nested=find(file.children ?? []){return nested}};return nil}
        let file=try XCTUnwrap(find(tree.tree)),debug=NativeDebugger(engine:engine)
        space.engine=engine;space.rootFiles=tree.tree;space.debugger=debug
        let buffer=try await space.loadBuffer(file)
        XCTAssertFalse(buffer.readOnly,"Fixture input must be a writable physical/authored source, not another document's generated target.");guard !buffer.readOnly else{return}
        space.layout.activate(file.path,in:"document-group");debug.sourceBuffers={ [weak space] in guard let space else{return []};return Array(space.buffers.values)}
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);NSApp.activate(ignoringOtherApps:true)
        let host=NSHostingView(rootView:WorkspaceView(workspace:space)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:480),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
        defer{debug.close();window.orderOut(nil);buffer.language?.close();space.sourceTestIndex?.stop()}
        func editor(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{editor($0)}.first}
        for _ in 0..<200{if editor(host) != nil{break};try await Task.sleep(for:.milliseconds(10))}
        let native=try XCTUnwrap(editor(host)),insertion=native.string.range(of:"</h:file>").map{NSRange($0,in:native.string).location} ?? native.string.utf16.count
        native.insertText("# source test native undo proof\n",replacementRange:NSRange(location:insertion,length:0));XCTAssertTrue(native.string.contains("# source test native undo proof"),"Native insertion must actually change editable fixture text.");await buffer.save(using:engine);XCTAssertFalse(buffer.dirty,buffer.error)
        native.setSelectedRange(NSRange(location:2,length:0));let undo=try XCTUnwrap(native.undoManager),scroll=try XCTUnwrap(native.enclosingScrollView),geometry=scroll.convert(scroll.bounds,to:host),source=buffer.text
        let line=try XCTUnwrap(source.components(separatedBy:"\n").firstIndex(where:{$0.contains("# source-test-breakpoint")}));debug.toggle(buffer,line:line)
        buffer.readTestSources()
        for _ in 0..<400{if buffer.sourceTestRows().contains(where:{$0.candidate.name_filter?.contains("TestChoice::test_selected")==true}){break};try await Task.sleep(for:.milliseconds(25))}
        let row=try XCTUnwrap(buffer.sourceTestRows().first(where:{$0.candidate.name_filter?.contains("TestChoice::test_selected")==true}),space.sourceTestIndex?.records[file.path]?.notice ?? "No current source-adjacent selected test row")
        let model=space.testSources;XCTAssertNotNil(buffer.inspectTests);buffer.inspectTests?(row.line)
        for _ in 0..<400{if model.observation != nil{break};try await Task.sleep(for:.milliseconds(25))}
        XCTAssertTrue(space.showTests);XCTAssertTrue(model.current(space),model.notice)
        let candidate=try XCTUnwrap(model.observation?.1.candidates.first(where:{$0.name_filter?.contains("TestChoice::test_selected")==true}),model.notice)
        if ["md","markdown"].contains((path as NSString).pathExtension){XCTAssertNotNil(candidate.source);XCTAssertTrue(exactUTF8(candidate.source?.path,path))}
        var outbound:[[String:Any]]=[];debug.observeRequestForTest={outbound.append($0)}
        await model.debug(candidate,.nameFilter,workspace:space)
        for _ in 0..<400{if debug.status=="paused" || debug.status=="failed"{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.status,"paused",model.notice+debug.message);guard debug.status=="paused" else{return}
        XCTAssertTrue(debug.sourceMatches(buffer),debug.message);XCTAssertEqual(debug.line(in:file.path),line,"Stopped source position must map to the selected physical/authored input.")
        let session=try XCTUnwrap(debug.sessionID),link=try XCTUnwrap(model.debugRuns.first);XCTAssertEqual(link.id,session);XCTAssertEqual(link.selection,"name_filter");XCTAssertEqual(link.name,candidate.name)
        let start=try XCTUnwrap(outbound.first(where:{$0["op"] as? String=="start_test"}));XCTAssertNotNil(start["source_test"]);XCTAssertNil(start["profile"]);XCTAssertNil(start["program"]);XCTAssertEqual((start["breakpoints"] as? [[String:Any]])?.count,1)
        debug.readFrameLocals();for _ in 0..<200{if !debug.frameBusy{break};try await Task.sleep(for:.milliseconds(50))};XCTAssertTrue(debug.variables.contains{$0["name"] as? String=="selected_value" && $0["value"] as? String=="41"},debug.frameNotice)
        debug.evaluate("selected_value",watch:true);for _ in 0..<200{if debug.values["selected_value"] != nil{break};try await Task.sleep(for:.milliseconds(50))};XCTAssertEqual(debug.values["selected_value"],"41",debug.evaluationNotice)
        await model.inspectDebug(link,workspace:space);XCTAssertFalse(space.showTests);XCTAssertEqual(debug.sessionID,session)
        XCTAssertTrue(editor(host) === native,"Original native editor remains mounted");XCTAssertTrue(native.undoManager === undo,"Original native undo manager retained");XCTAssertTrue(undo.canUndo,"Native insertion undo remains available");XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertEqual(native.selectedRange(),NSRange(location:2,length:0));XCTAssertEqual(buffer.text,source)
        debug.stop();for _ in 0..<200{if !debug.active{break};try await Task.sleep(for:.milliseconds(50))};XCTAssertFalse(debug.active,debug.message);XCTAssertEqual(debug.retainedSessionID,session);XCTAssertEqual(model.debugRuns.first?.id,session)
        // A stopped first process cannot prove sibling exclusion through completion.
        // Explicitly remove the user's BP, then admit the same saved filter anew.
        debug.toggle(buffer,line:line);space.showTests=true;await model.debug(candidate,.nameFilter,workspace:space)
        for _ in 0..<400{if debug.status=="finished" || debug.status=="failed"{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.status,"finished",model.notice+debug.message)
        let completed=try XCTUnwrap(model.debugRuns.first);XCTAssertNotEqual(completed.id,session);XCTAssertEqual(completed.state,"process finished");XCTAssertTrue(completed.exit==nil || completed.exit==0,"Reported nonzero process exit; an absent exit remains uncollected.");XCTAssertEqual(model.debugRuns.count,2);XCTAssertEqual(model.debugRuns.last?.id,session)
        let filteredStarts=outbound.filter{$0["op"] as? String=="start_test"};XCTAssertEqual(filteredStarts.count,2);XCTAssertEqual((filteredStarts.last?["breakpoints"] as? [[String:Any]])?.count,0)
        XCTAssertTrue(editor(host) === native);XCTAssertTrue(native.undoManager === undo);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertEqual(buffer.text,source)
        let finalTree=try await engine.files();func contains(_ files:[FileNode])->Bool{files.contains{$0.path=="sibling-ran.txt" || contains($0.children ?? [])}};XCTAssertFalse(contains(finalTree.tree),"Filtered Debug executed a sibling test")
        undo.undo();XCTAssertTrue(undo.canRedo,"Native insertion undo must retain coordinated redo");XCTAssertFalse(native.string.contains("# source test native undo proof"),"Undo must remove the original native insertion");XCTAssertEqual(buffer.text,native.string)
        #endif
    }
    @MainActor func testLiveNativeThreadSelectionAndScopedEvaluation()async throws {
        guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_DEBUG_ORIGIN"] else{throw XCTSkip("Set HICKORY_NATIVE_DEBUG_ORIGIN for the real multi-thread debugger fixture.")}
        let engine=try Engine(address:origin),file=FileNode(name:"debug.py",path:"debug.py",dir:false),loaded=try await engine.file(file)
        guard let line=loaded.content.components(separatedBy:"\n").firstIndex(where:{$0.contains("# thread-stop")}) else{throw XCTSkip("Use a debug.py fixture with a worker and a # thread-stop breakpoint.")}
        let buffer=DocumentBuffer(file:file,text:loaded),debug=NativeDebugger(engine:engine);defer{debug.close()}
        buffer.selection=NSRange(location:2,length:0);debug.toggle(buffer,line:line);await debug.prepare(buffer)
        for _ in 0..<400 {if debug.status=="paused" || debug.status=="failed"{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.status,"paused",debug.message);guard debug.status=="paused" else{return}
        debug.readThreads();for _ in 0..<200 {if !debug.threadBusy{break};try await Task.sleep(for:.milliseconds(50))}
        let original=try XCTUnwrap(debug.selectedThread),worker=try XCTUnwrap(debug.threads.first(where:{$0.name == "hickory-worker"}),debug.threadNotice)
        debug.selectThread(worker.id,snapshot:debug.threadSnapshot)
        for _ in 0..<200 {if !debug.threadBusy{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.status,"paused",debug.message);XCTAssertEqual(debug.selectedThread,worker.id,debug.threadNotice);XCTAssertFalse(debug.frames.isEmpty)
        let workerFrame=try XCTUnwrap(debug.frames.first(where:{$0["name"] as? String == "worker"}),"Worker frame absent")
        debug.select(workerFrame,snapshot:debug.frameSnapshot)
        debug.readFrameLocals();for _ in 0..<200 {if !debug.frameBusy{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertTrue(debug.frameVariablesCurrent,debug.frameNotice)
        debug.evaluate("worker_value");for _ in 0..<200 {if debug.values["worker_value"] != nil{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.values["worker_value"],"41",debug.message+debug.evaluationNotice)
        let composite="{\"nested\": {\"answer\": worker_value}}"
        debug.evaluate(composite);for _ in 0..<200 {if debug.visibleVariableRows.contains(where:{$0.name==composite}){break};try await Task.sleep(for:.milliseconds(50))}
        let object=try XCTUnwrap(debug.visibleVariableRows.first(where:{$0.name==composite}),debug.evaluationNotice);XCTAssertGreaterThan(object.reference,0);debug.toggleVariable(object.id)
        for _ in 0..<200 {if debug.visibleVariableRows.first(where:{$0.id==object.id})?.pending==false{break};try await Task.sleep(for:.milliseconds(50))}
        let nested=try XCTUnwrap(debug.visibleVariableRows.first(where:{$0.depth==1 && $0.name.contains("nested") && $0.reference>0}),String(describing:debug.visibleVariableRows.map{$0.name+":"+$0.notice}));debug.toggleVariable(nested.id)
        for _ in 0..<200 {if debug.visibleVariableRows.first(where:{$0.id==nested.id})?.pending==false{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertTrue(debug.visibleVariableRows.contains{$0.depth==2 && $0.name.contains("answer") && $0.value=="41"},String(describing:debug.visibleVariableRows.map{$0.name+"="+$0.value+":"+$0.notice}))
        debug.selectThread(original,snapshot:debug.threadSnapshot);for _ in 0..<200 {if !debug.threadBusy{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.selectedThread,original,debug.threadNotice);XCTAssertFalse(debug.visibleVariableRows.contains{$0.id==object.id})
        debug.evaluate("main_value");for _ in 0..<200 {if debug.values["main_value"] != nil{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.values["main_value"],"7",debug.message+debug.evaluationNotice)
        XCTAssertEqual(buffer.text,loaded.content);XCTAssertEqual(buffer.selection,NSRange(location:2,length:0));XCTAssertFalse(buffer.dirty)
        debug.stop();for _ in 0..<200 {if !debug.active{break};try await Task.sleep(for:.milliseconds(50))};XCTAssertFalse(debug.active,debug.message)
    }

}
